#![cfg_attr(not(feature = "std"), no_std)]

//! # offchain-transaction-pos · 清算行(L2)扫码支付清算
//!
//! ADR-006 "省储行退出清算" 之后的唯一链上清算实现。提供:
//!
//! - L3 绑定 / 充值 / 提现 / 切换清算行(Step 1 · call_index 30-33)
//! - 清算行批次上链 + settlement 执行(Step 2a · call_index 34)
//! - L2 费率自治(Step 2a · call_index 40/41)
//! - L3 支付意图签名 / nonce 防重 / 偿付自动保护 / 多签账户登记等配套机制
//!
//! **Step 2b-iv-b 彻底清理完成**:原"省储行即时清算"模型(旧 call_index
//! 0/1/2/9/10/11/14-20/23 + `RecipientClearingInstitution` / `InstitutionRateBp` /
//! `QueuedBatches` 等 Storage + 相关 Events/Errors + 所有辅助函数)已从源码中
//! 物理删除,升级路径由 dev 链统一 setCode 启用,不做 on_runtime_upgrade migration
//! (`STORAGE_VERSION` 从 1 → 2)。

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

// 扫码支付清算体系子模块。
pub mod bank_check;
pub mod batch_item;
pub mod deposit;
pub mod fee_config;
pub mod nonce;
pub mod settlement;
pub mod solvency;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::Currency,
    traits::StorageVersion,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::sr25519::{Public as Sr25519Public, Signature as Sr25519Signature};
use sp_io::crypto::sr25519_verify;

/// Step 2(2026-04-27, ADR-007)新增:清算行节点声明信息。
///
/// 一家清算行机构(sfid_id)在链上声明其对外服务的全节点身份 + RPC 接入点。
/// 用于:
/// - wuminapp 通过 sfid_id 反查清算行节点的 wss URL
/// - wuminapp 校验对端 PeerId 防 DNS 劫持
/// - NodeUI 网络面板统计 clearing_nodes 数量
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub struct ClearingBankNodeInfo<AccountId, BlockNumber> {
    /// libp2p PeerId 字符串(以 "12D3KooW" 开头,~52 字节)。
    pub peer_id: BoundedVec<u8, sp_core::ConstU32<64>>,
    /// 节点对外可达的 RPC 域名(不含 scheme/port,如 "l2.cmb.com.cn")。
    pub rpc_domain: BoundedVec<u8, sp_core::ConstU32<128>>,
    /// 节点 RPC 端口(通常 9944)。
    pub rpc_port: u16,
    /// 注册时所在区块高度。
    pub registered_at: BlockNumber,
    /// 提交注册的清算行管理员账户(审计用)。
    pub registered_by: AccountId,
}

/// 清算行清算 pallet 的存储版本。Step 2b-iv-c 增加批次序号防重后从 2 → 3。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::bank_check::SfidAccountQuery;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        /// 单批次最大 item 数(SCALE 安全上限)。
        #[pallet::constant]
        type MaxBatchSize: Get<u32>;

        /// 清算行多签批次签名最大字节(BoundedVec 上限)。Step 2b 起严格校验。
        #[pallet::constant]
        type MaxBatchSignatureLength: Get<u32>;

        /// 资金白名单 / 制度保留地址保护,由 runtime 接入 `institution-asset-guard`。
        type InstitutionAssetGuard: institution_asset_guard::InstitutionAssetGuard<Self::AccountId>;

        /// SFID 机构登记表查询抽象。runtime 层应委托给 `duoqian-manage-pow`;
        /// 测试可用 `()` 的默认空实现(一律返回未登记)。
        type SfidAccountQuery: crate::bank_check::SfidAccountQuery<Self::AccountId>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    /// 清算行多签批次签名(BoundedVec,Step 2b 接入阈值校验时启用完整校验)。
    pub type BatchSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxBatchSignatureLength>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // ================== Storage(清算行 L2 体系) ==================

    /// L3 用户绑定的清算行主账户地址。
    ///
    /// 一个 L3 同时只能绑定一家清算行;切换清算行需先把 `DepositBalance` 清零。
    #[pallet::storage]
    #[pallet::getter(fn user_bank)]
    pub type UserBank<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, OptionQuery>;

    /// `(清算行主账户, L3)` → 该 L3 在该清算行的存款余额(分)。
    ///
    /// 权威账本;清算行节点本地 ledger 只是缓存,最终以链上值为准。
    #[pallet::storage]
    #[pallet::getter(fn deposit_balance)]
    pub type DepositBalance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        u128,
        ValueQuery,
    >;

    /// 清算行主账户 → 该清算行所有 L3 存款的总额(冗余,偿付对账用)。
    ///
    /// 不变式:`BankTotalDeposits[bank] == Σ DepositBalance[bank][*]`。
    /// 偿付能力要求:`Currency::free_balance(bank) >= BankTotalDeposits[bank]`。
    #[pallet::storage]
    #[pallet::getter(fn bank_total_deposits)]
    pub type BankTotalDeposits<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

    /// L3 的单调递增支付 nonce(防 L3 签名被重放)。settlement 批次 `execute`
    /// 时通过 `nonce::consume_nonce` 校验并更新。
    #[pallet::storage]
    #[pallet::getter(fn l3_payment_nonce)]
    pub type L3PaymentNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    /// 清算行当前生效费率(bp)。`settlement::execute_clearing_bank_batch` 按
    /// **收款方清算行** 读此值计算手续费。
    #[pallet::storage]
    #[pallet::getter(fn l2_fee_rate_bp)]
    pub type L2FeeRateBp<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 清算行**待生效**的费率提案。`on_initialize` 到达 `effective_at` 后
    /// 把 `(bank, new_rate_bp)` 搬到 `L2FeeRateBp` 并清除本条。
    #[pallet::storage]
    #[pallet::getter(fn l2_fee_rate_proposed)]
    pub type L2FeeRateProposed<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (u32, BlockNumberFor<T>), OptionQuery>;

    /// 全局费率上限(bp),由联合投票调整。默认 0 → runtime `fee_config` 兜底
    /// 到 `L2_FEE_RATE_BP_MAX`(10 bp = 0.1%)。
    #[pallet::storage]
    #[pallet::getter(fn max_l2_fee_rate_bp)]
    pub type MaxL2FeeRateBp<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// 已处理链下 tx_id 防重放(按省标识 T2 + tx_id 维度)。
    ///
    /// settlement 写入;清算行节点 event_listener 监听 `PaymentSettled` 时
    /// 以此键为索引。
    #[pallet::storage]
    pub type ProcessedOffchainTx<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, [u8; 2], Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 已处理链下 tx_id 的写入高度(用于过期窗口控制,Step 3 开启清理窗口)。
    #[pallet::storage]
    pub type ProcessedOffchainTxAt<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        [u8; 2],
        Blake2_128Concat,
        T::Hash,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// 清算行主账户 → 已成功落账的最新批次序号。
    ///
    /// node 侧 packer 启动时读取本值续跑,链上入口要求下一批必须等于
    /// `last + 1`,避免节点重启或恶意重复提交造成批次级重放。
    #[pallet::storage]
    #[pallet::getter(fn last_clearing_batch_seq)]
    pub type LastClearingBatchSeq<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    /// Step 2(2026-04-27, ADR-007)新增:清算行节点声明 storage。
    ///
    /// `sfid_id` → 节点信息(peer_id / rpc_domain / rpc_port / 注册管理员)
    ///
    /// 链上自证"哪家机构在哪个全节点上对外提供清算服务"。
    /// 写入时机:`register_clearing_bank` 单签即可,要求调用方是该机构的激活管理员。
    /// 删除/更新:`unregister_clearing_bank` / `update_clearing_bank_endpoint`。
    #[pallet::storage]
    #[pallet::getter(fn clearing_bank_nodes)]
    pub type ClearingBankNodes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, sp_core::ConstU32<64>>,
        crate::ClearingBankNodeInfo<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Step 2 新增:节点 PeerId 反向索引(`peer_id → sfid_id`),
    /// 防止同一 PeerId 被多个机构占用。
    #[pallet::storage]
    #[pallet::getter(fn node_peer_to_institution)]
    pub type NodePeerToInstitution<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, sp_core::ConstU32<64>>,
        BoundedVec<u8, sp_core::ConstU32<64>>,
        OptionQuery,
    >;

    // ================== Events ==================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// L3 绑定清算行(绑定即开户,无预存)。
        BankBound {
            user: T::AccountId,
            bank: T::AccountId,
        },
        /// L3 充值到清算行主账户。
        Deposited {
            user: T::AccountId,
            bank: T::AccountId,
            amount: u128,
        },
        /// L3 从清算行主账户提现。
        Withdrawn {
            user: T::AccountId,
            bank: T::AccountId,
            amount: u128,
        },
        /// L3 切换清算行(前置:旧清算行余额已清零)。
        BankSwitched {
            user: T::AccountId,
            old_bank: T::AccountId,
            new_bank: T::AccountId,
        },
        /// 清算行管理员提交了费率变更提案,延迟到 `effective_at` 生效。
        L2FeeRateProposed {
            bank: T::AccountId,
            new_rate_bp: u32,
            effective_at: BlockNumberFor<T>,
        },
        /// 费率提案到期自动激活。
        L2FeeRateActivated { bank: T::AccountId, rate_bp: u32 },
        /// 全局费率上限更新(联合投票)。
        MaxL2FeeRateUpdated { new_max: u32 },
        /// 单笔扫码支付已在链上最终清算。
        PaymentSettled {
            tx_id: T::Hash,
            payer: T::AccountId,
            payer_bank: T::AccountId,
            recipient: T::AccountId,
            recipient_bank: T::AccountId,
            transfer_amount: u128,
            fee_amount: u128,
        },
        /// 一次清算行批次落账汇总。
        ClearingBankBatchSettled {
            bank: T::AccountId,
            submitter: T::AccountId,
            item_count: u32,
            total_debit: u128,
        },
        /// Step 2 新增:清算行节点声明完成,机构对外提供清算服务。
        ClearingBankRegistered {
            sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
            peer_id: BoundedVec<u8, sp_core::ConstU32<64>>,
            rpc_domain: BoundedVec<u8, sp_core::ConstU32<128>>,
            rpc_port: u16,
            registered_by: T::AccountId,
        },
        /// Step 2 新增:清算行节点 RPC 端点更新(域名 / 端口变更,PeerId 不变)。
        ClearingBankEndpointUpdated {
            sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
            new_domain: BoundedVec<u8, sp_core::ConstU32<128>>,
            new_port: u16,
            updated_by: T::AccountId,
        },
        /// Step 2 新增:清算行节点声明注销,机构退出清算网络。
        ClearingBankUnregistered {
            sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
            unregistered_by: T::AccountId,
        },
    }

    // ================== Errors ==================

    #[pallet::error]
    pub enum Error<T> {
        /// 单批次金额或手续费字段非法。
        InvalidTransferAmount,
        InvalidFeeAmount,
        /// 付款方 = 收款方。
        SelfTransferNotAllowed,
        /// 单笔金额超 u128::MAX 溢出。
        TransferAmountTooLarge,
        /// 批次为空。
        EmptyBatch,
        /// 批次中 `payer_bank` 与提交批次的 `institution_main` 不一致(单清算行批次假定)。
        InstitutionMismatch,
        /// 清算行管理员身份校验未通过。
        UnauthorizedAdmin,
        /// 签名已过期(`expires_at` 小于当前高度)。
        ExpiredIntent,
        /// 收款方清算行尚未配置 `L2FeeRateBp`。
        L2FeeRateNotConfigured,
        /// L3 sr25519 签名校验失败。
        InvalidL3Signature,
        /// 新费率越界(`< MIN` 或 `> Max`)。
        InvalidL2FeeRate,
        /// 清算行偿付不足,自动拒绝新扣款。
        SolvencyProtected,
        /// `tx_id` 已在链上被清算,拒绝重复提交。
        TxAlreadyProcessed,
        /// 清算行批次级签名无效。
        InvalidBatchSignature,
        /// 清算行批次序号不等于上一成功序号 + 1。
        InvalidBatchSeq,
        /// 批次 item 内用户声明的清算行与链上 `UserBank[user]` 不一致。
        UserBankMismatch,

        // ========== L3 账户相关 ==========
        /// 目标地址未在链上 `duoqian-manage-pow` 注册为清算行机构。
        NotRegisteredClearingBank,
        /// 目标地址的 `name` 不是 "主账户"(只能绑定主账户,不能绑费用账户)。
        NotMainAccount,
        /// 目标地址的 SFID A3 不是 SFR(私法人)或 FFR(非法人),不属于私权机构。
        NotPrivateInstitution,
        /// 对应的多签账户状态非 Active(可能还在 Pending,或已关闭)。
        ClearingBankNotActive,
        /// 反查费用账户名称过长(SFID name BoundedVec 溢出)。
        FeeAccountNameTooLong,
        /// 清算行未创建配套的 "费用账户",无法清算手续费。
        FeeAccountNotFound,
        /// L3 当前已绑定其他清算行,需先 switch_bank。
        AlreadyHasBank,
        /// L3 尚未绑定任何清算行。
        NoOpenedBank,
        /// switch_bank 时新旧清算行相同。
        NewBankSameAsCurrent,
        /// 切换清算行前旧清算行余额必须为 0。
        MustClearBalanceFirst,
        /// 充值金额必须大于 0。
        DepositAmountTooSmall,
        /// 提现金额必须大于 0。
        WithdrawAmountTooSmall,
        /// 提现金额超过清算行存款余额。
        InsufficientDepositBalance,
        /// 清算行主账户余额不足以兑现提现(偿付异常,应告警并拒绝)。
        InsufficientBankLiquidity,
        /// institution-asset-guard 拒绝了本笔充值动作。
        DepositForbidden,
        /// institution-asset-guard 拒绝了本笔提现动作。
        WithdrawForbidden,
        /// L3 nonce 自增溢出(极小概率,仅防御性错误)。
        L3NonceOverflow,
        /// L3 提交的 nonce 不等于 `链上 nonce + 1`(重放或不同步)。
        InvalidL3Nonce,

        // ========== Step 2 清算行节点声明相关 ==========
        /// 清算行节点 sfid_id 字段不能为空。
        EmptySfidId,
        /// PeerId 字段不能为空。
        EmptyPeerId,
        /// PeerId 格式非法(必须 "12D3KooW" 开头 + 长度 ≥ 46 + 纯 ASCII alphanumeric)。
        InvalidPeerIdFormat,
        /// RPC 域名字段不能为空。
        EmptyRpcDomain,
        /// RPC 域名格式非法(仅允许小写字母 / 数字 / 点 / 横杠)。
        InvalidRpcDomainFormat,
        /// RPC 端口非法(必须 1024-65535)。
        InvalidRpcPort,
        /// 该机构(sfid_id)不满足清算行资格白名单。
        NotEligibleForClearingBank,
        /// 该 sfid_id 已经声明了清算行节点(切换走 unregister + register)。
        ClearingBankAlreadyRegistered,
        /// 该 sfid_id 尚未声明清算行节点(无法 update / unregister)。
        ClearingBankNodeNotFound,
        /// PeerId 已被另一家机构占用(防 PeerId 冒名)。
        PeerIdAlreadyRegistered,
        /// Step 2 第 6 重 bank_check:该机构未声明清算行节点(尚未加入清算网络)。
        ClearingBankNotRegisteredAsNode,
    }

    // ================== Calls ==================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// L3 绑定清算行 = 开户。无预存、无业务开户费。
        ///
        /// 约束:
        /// - 未绑定其他清算行
        /// - `bank_main_address` 必须是 SFR/FFR 私权机构 + 多签 Active + 主账户
        #[pallet::call_index(30)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn bind_clearing_bank(
            origin: OriginFor<T>,
            bank_main_address: T::AccountId,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;
            crate::deposit::do_bind_clearing_bank::<T>(user, bank_main_address)
        }

        /// L3 从自持链上账户充值到绑定的清算行主账户。`amount` 单位分。
        #[pallet::call_index(31)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3))]
        pub fn deposit(origin: OriginFor<T>, amount: u128) -> DispatchResult {
            let user = ensure_signed(origin)?;
            crate::deposit::do_deposit::<T>(user, amount)
        }

        /// L3 从清算行主账户提现到自持链上账户。
        #[pallet::call_index(32)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3))]
        pub fn withdraw(origin: OriginFor<T>, amount: u128) -> DispatchResult {
            let user = ensure_signed(origin)?;
            crate::deposit::do_withdraw::<T>(user, amount)
        }

        /// L3 切换清算行。前置:当前清算行余额必须为 0。
        #[pallet::call_index(33)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 3))]
        pub fn switch_bank(origin: OriginFor<T>, new_bank: T::AccountId) -> DispatchResult {
            let user = ensure_signed(origin)?;
            crate::deposit::do_switch_bank::<T>(user, new_bank)
        }

        /// 清算行批次上链(清算行 L2 体系唯一上链路径)。
        ///
        /// Step 2(2026-04-27, ADR-007)修订:**收款方主导清算**模型。
        /// - `institution_main` 现在 = **收款方清算行主账户**(原为付款方)
        /// - 提交者 = 收款方清算行的某个激活管理员(已解密私钥,自动签)
        /// - 批次内所有 item 的 `recipient_bank` 必须等于 `institution_main`
        ///   (`payer_bank` 可不同,即同一收款方清算行可一次代收来自不同付款方清算行的多笔)
        /// - 链上 gas 由 `RuntimeFeePayerExtractor` 自动从 `fee_account_of(institution_main)` 扣
        ///   (即 fee 收入和 gas 支出都由同一收款方清算行的费用账户承担,自给自足)
        ///
        /// 安全模型:链上验签的核心是 L3 用户对 PaymentIntent 的 sr25519 签名,
        /// PaymentIntent 内含 payer_bank 字段;链上凭 L3 签名授权 mutate
        /// payer_bank 主账户 Currency,与谁提交批次无关。
        ///
        /// [`institution_main`] 批次归属的清算行主账户地址(= **收款方**清算行)
        /// [`batch_seq`] 清算行内单调递增的批次序号(冗余审计字段,Step 2b 启用严格校验)
        /// [`batch`] `OffchainBatchItemV2` 列表(每条带 L3 sr25519 签名 / nonce / 费率)
        /// [`batch_signature`] 清算行多签批次级签名(Step 2b 启用阈值校验)
        #[pallet::call_index(34)]
        #[pallet::weight(
            T::DbWeight::get().reads_writes(9, 7)
                + T::DbWeight::get().reads_writes(7, 6) * batch.len() as u64
        )]
        pub fn submit_offchain_batch_v2(
            origin: OriginFor<T>,
            institution_main: T::AccountId,
            batch_seq: u64,
            batch: BoundedVec<
                crate::batch_item::OffchainBatchItemV2<T::AccountId, BlockNumberFor<T>>,
                T::MaxBatchSize,
            >,
            batch_signature: BatchSignatureOf<T>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            ensure!(!batch.is_empty(), Error::<T>::EmptyBatch);
            ensure!(
                T::SfidAccountQuery::is_admin_of(&institution_main, &submitter),
                Error::<T>::UnauthorizedAdmin
            );
            Self::verify_batch_signature(
                &submitter,
                &institution_main,
                batch_seq,
                batch.as_slice(),
                &batch_signature,
            )?;
            ensure!(
                batch_seq == LastClearingBatchSeq::<T>::get(&institution_main).saturating_add(1),
                Error::<T>::InvalidBatchSeq
            );

            with_transaction(|| {
                match crate::settlement::execute_clearing_bank_batch::<T>(
                    &submitter,
                    &institution_main,
                    batch.as_slice(),
                ) {
                    Ok(()) => {
                        LastClearingBatchSeq::<T>::insert(&institution_main, batch_seq);
                        TransactionOutcome::Commit(Ok(()))
                    }
                    Err(e) => TransactionOutcome::Rollback(Err(e)),
                }
            })?;
            Ok(())
        }

        /// 清算行管理员提案新费率,延迟 7 天生效。
        #[pallet::call_index(40)]
        #[pallet::weight(T::DbWeight::get().reads_writes(5, 2))]
        pub fn propose_l2_fee_rate(
            origin: OriginFor<T>,
            bank: T::AccountId,
            new_rate_bp: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::fee_config::do_propose_l2_fee_rate::<T>(who, bank, new_rate_bp)
        }

        /// 设置全局费率上限(Root Origin;Step 2b 起改为联合投票回调)。
        #[pallet::call_index(41)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_max_l2_fee_rate(origin: OriginFor<T>, new_max: u32) -> DispatchResult {
            ensure_root(origin)?;
            crate::fee_config::do_set_max_l2_fee_rate::<T>(new_max)
        }

        /// Step 2(2026-04-27, ADR-007)新增:声明本节点为某清算行的清算节点。
        ///
        /// 校验链(任一失败立即拒绝):
        /// 1. origin 是签名账户
        /// 2. sfid_id / peer_id / rpc_domain 非空,rpc_port ∈ [1024, 65535]
        /// 3. peer_id 格式合法("12D3KooW" 开头 + 长度 ≥ 46 + 纯 ASCII alphanumeric)
        /// 4. rpc_domain 字符集合法(仅小写字母/数字/点/横杠)
        /// 5. sfid_id 反查得到主账户地址 + 该地址已 Active
        /// 6. 调用方(origin)是该机构的激活管理员之一
        /// 7. 资格白名单:机构必须 (SFR ∧ JOINT_STOCK) ∨ (FFR ∧ parent.SFR.JOINT_STOCK)
        /// 8. sfid_id 未已注册节点(切换走 unregister + register)
        /// 9. peer_id 未被另一机构占用
        ///
        /// 单签即可,不走内部投票(节点声明影响小,损失可逆)。
        #[pallet::call_index(50)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 3))]
        pub fn register_clearing_bank(
            origin: OriginFor<T>,
            sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
            peer_id: BoundedVec<u8, sp_core::ConstU32<64>>,
            rpc_domain: BoundedVec<u8, sp_core::ConstU32<128>>,
            rpc_port: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_register_clearing_bank(who, sfid_id, peer_id, rpc_domain, rpc_port)
        }

        /// Step 2 新增:更新清算行节点的 RPC 端点(域名 / 端口),PeerId 不变。
        ///
        /// 校验:
        /// 1. origin 是签名账户
        /// 2. sfid_id 已注册清算行节点
        /// 3. 调用方是该机构的激活管理员
        /// 4. new_domain / new_port 字段合法
        ///
        /// 不重新校验资格白名单(注册时已校验,后续无需重复)。
        #[pallet::call_index(51)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 1))]
        pub fn update_clearing_bank_endpoint(
            origin: OriginFor<T>,
            sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
            new_domain: BoundedVec<u8, sp_core::ConstU32<128>>,
            new_port: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_update_clearing_bank_endpoint(who, sfid_id, new_domain, new_port)
        }

        /// Step 2 新增:注销清算行节点声明,机构退出清算网络。
        ///
        /// 校验:
        /// 1. origin 是签名账户
        /// 2. sfid_id 已注册清算行节点
        /// 3. 调用方是该机构的激活管理员
        ///
        /// 注销后该机构 sfid_id 不再被 wuminapp 显示为可绑定清算行(SFID 后端
        /// `app_search_clearing_banks` 过滤会去掉该 sfid_id)。
        /// 已绑定到该机构的用户需要主动 switch_bank 切换或继续使用直到迁移完成。
        #[pallet::call_index(52)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 2))]
        pub fn unregister_clearing_bank(
            origin: OriginFor<T>,
            sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_unregister_clearing_bank(who, sfid_id)
        }
    }

    // ================== Hooks ==================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(T::MaxBatchSize::get() > 0);
            assert!(T::MaxBatchSignatureLength::get() > 0);
        }

        /// 每块扫描激活到期的 `L2FeeRateProposed` 提案,搬到 `L2FeeRateBp`。
        /// 清算行规模较小时成本低;若达万级,Step 3 可优化为 cursor/分批。
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            crate::fee_config::activate_pending_rates::<T>(now)
        }
    }

    impl<T: Config> Pallet<T> {
        /// 验证清算行管理员对整批 item 的批次级签名。
        ///
        /// L3 签名仍是资金授权的核心,本签名用于约束“哪个清算行管理员提交了
        /// 哪个 institution + batch_seq + batch_bytes”,与 `LastClearingBatchSeq`
        /// 一起防止节点重启后的批次级重放。
        fn verify_batch_signature(
            submitter: &T::AccountId,
            institution_main: &T::AccountId,
            batch_seq: u64,
            batch: &[crate::batch_item::OffchainBatchItemV2<T::AccountId, BlockNumberFor<T>>],
            batch_signature: &BatchSignatureOf<T>,
        ) -> DispatchResult {
            let sig = Sr25519Signature::try_from(batch_signature.as_slice())
                .map_err(|_| Error::<T>::InvalidBatchSignature)?;
            let public = Self::sr25519_pubkey_from_account(submitter)?;
            let batch_bytes = batch.encode();
            let message =
                crate::batch_item::batch_signing_hash(institution_main, batch_seq, &batch_bytes);
            ensure!(
                sr25519_verify(&sig, &message, &public),
                Error::<T>::InvalidBatchSignature
            );
            Ok(())
        }

        fn sr25519_pubkey_from_account(account: &T::AccountId) -> Result<Sr25519Public, Error<T>> {
            let encoded = account.encode();
            if encoded.len() < 32 {
                return Err(Error::<T>::InvalidBatchSignature);
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&encoded[..32]);
            Ok(Sr25519Public::from_raw(arr))
        }
    }
}

impl<T: pallet::Config> pallet::Pallet<T> {
    /// 反查清算行主账户对应的费用账户(辅助 ops / off-chain ledger)。
    pub fn fee_account_of(bank_main: &T::AccountId) -> Result<T::AccountId, pallet::Error<T>> {
        crate::bank_check::fee_account_of::<T>(bank_main)
    }

    // ============= Step 2(2026-04-27, ADR-007)清算行节点声明实现 =============

    /// PeerId 字节串校验:必须以 "12D3KooW" 开头 + 长度 ≥ 46 + 纯 ASCII alphanumeric。
    /// 与 [citizenchain/node/src/ui/network/network-overview/mod.rs::normalize_peer_id]
    /// 保持一致的语义。链上仅做字节级格式校验,不解析 libp2p 协议。
    fn validate_peer_id_bytes(peer_id: &[u8]) -> Result<(), pallet::Error<T>> {
        if peer_id.len() < 46 {
            return Err(pallet::Error::<T>::InvalidPeerIdFormat);
        }
        if !peer_id.starts_with(b"12D3KooW") {
            return Err(pallet::Error::<T>::InvalidPeerIdFormat);
        }
        if !peer_id.iter().all(|c| c.is_ascii_alphanumeric()) {
            return Err(pallet::Error::<T>::InvalidPeerIdFormat);
        }
        Ok(())
    }

    /// RPC 域名字节串校验:仅允许小写字母 / 数字 / 点 / 横杠;
    /// 不解析 DNS,真实可达性由 NodeUI 提交前自测保证。
    fn validate_rpc_domain_bytes(domain: &[u8]) -> Result<(), pallet::Error<T>> {
        if domain.is_empty() {
            return Err(pallet::Error::<T>::EmptyRpcDomain);
        }
        let valid = domain
            .iter()
            .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.' || c == b'-');
        if !valid {
            return Err(pallet::Error::<T>::InvalidRpcDomainFormat);
        }
        Ok(())
    }

    /// 反查 sfid_id 对应的清算行主账户地址(用于校验机构合法性)。
    fn lookup_main_account_by_sfid(sfid_id: &[u8]) -> Result<T::AccountId, pallet::Error<T>> {
        use crate::bank_check::{SfidAccountQuery, ACCOUNT_NAME_MAIN};
        T::SfidAccountQuery::find_address(sfid_id, ACCOUNT_NAME_MAIN)
            .ok_or(pallet::Error::<T>::NotRegisteredClearingBank)
    }

    /// `register_clearing_bank` 完整业务逻辑(供 extrinsic 调用)。
    pub(crate) fn do_register_clearing_bank(
        who: T::AccountId,
        sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
        peer_id: BoundedVec<u8, sp_core::ConstU32<64>>,
        rpc_domain: BoundedVec<u8, sp_core::ConstU32<128>>,
        rpc_port: u16,
    ) -> DispatchResult {
        use crate::bank_check::SfidAccountQuery;

        // 1-2. 非空 + 端口范围
        ensure!(!sfid_id.is_empty(), pallet::Error::<T>::EmptySfidId);
        ensure!(!peer_id.is_empty(), pallet::Error::<T>::EmptyPeerId);
        ensure!(rpc_port >= 1024, pallet::Error::<T>::InvalidRpcPort);

        // 3. PeerId 格式
        Self::validate_peer_id_bytes(peer_id.as_slice())?;

        // 4. 域名字符集
        Self::validate_rpc_domain_bytes(rpc_domain.as_slice())?;

        // 5. sfid_id → 主账户 + Active
        let bank_main = Self::lookup_main_account_by_sfid(sfid_id.as_slice())?;
        ensure!(
            T::SfidAccountQuery::is_active(&bank_main),
            pallet::Error::<T>::ClearingBankNotActive
        );

        // 6. 调用方是该机构的激活管理员
        ensure!(
            T::SfidAccountQuery::is_admin_of(&bank_main, &who),
            pallet::Error::<T>::UnauthorizedAdmin
        );

        // 7. 资格白名单(委托给 trait 实现层查 InstitutionMetadata)
        ensure!(
            T::SfidAccountQuery::is_clearing_bank_eligible(&bank_main),
            pallet::Error::<T>::NotEligibleForClearingBank
        );

        // 8. sfid_id 未已注册
        ensure!(
            !pallet::ClearingBankNodes::<T>::contains_key(&sfid_id),
            pallet::Error::<T>::ClearingBankAlreadyRegistered
        );

        // 9. peer_id 未被另一机构占用
        ensure!(
            !pallet::NodePeerToInstitution::<T>::contains_key(&peer_id),
            pallet::Error::<T>::PeerIdAlreadyRegistered
        );

        let now = frame_system::Pallet::<T>::block_number();
        let info = crate::ClearingBankNodeInfo {
            peer_id: peer_id.clone(),
            rpc_domain: rpc_domain.clone(),
            rpc_port,
            registered_at: now,
            registered_by: who.clone(),
        };

        pallet::ClearingBankNodes::<T>::insert(&sfid_id, &info);
        pallet::NodePeerToInstitution::<T>::insert(&peer_id, &sfid_id);

        Self::deposit_event(pallet::Event::ClearingBankRegistered {
            sfid_id,
            peer_id,
            rpc_domain,
            rpc_port,
            registered_by: who,
        });
        Ok(())
    }

    /// `update_clearing_bank_endpoint` 完整业务逻辑。
    pub(crate) fn do_update_clearing_bank_endpoint(
        who: T::AccountId,
        sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
        new_domain: BoundedVec<u8, sp_core::ConstU32<128>>,
        new_port: u16,
    ) -> DispatchResult {
        use crate::bank_check::SfidAccountQuery;

        ensure!(!sfid_id.is_empty(), pallet::Error::<T>::EmptySfidId);
        ensure!(new_port >= 1024, pallet::Error::<T>::InvalidRpcPort);
        Self::validate_rpc_domain_bytes(new_domain.as_slice())?;

        let mut info = pallet::ClearingBankNodes::<T>::get(&sfid_id)
            .ok_or(pallet::Error::<T>::ClearingBankNodeNotFound)?;

        let bank_main = Self::lookup_main_account_by_sfid(sfid_id.as_slice())?;
        ensure!(
            T::SfidAccountQuery::is_admin_of(&bank_main, &who),
            pallet::Error::<T>::UnauthorizedAdmin
        );

        info.rpc_domain = new_domain.clone();
        info.rpc_port = new_port;
        pallet::ClearingBankNodes::<T>::insert(&sfid_id, &info);

        Self::deposit_event(pallet::Event::ClearingBankEndpointUpdated {
            sfid_id,
            new_domain,
            new_port,
            updated_by: who,
        });
        Ok(())
    }

    /// `unregister_clearing_bank` 完整业务逻辑。
    pub(crate) fn do_unregister_clearing_bank(
        who: T::AccountId,
        sfid_id: BoundedVec<u8, sp_core::ConstU32<64>>,
    ) -> DispatchResult {
        use crate::bank_check::SfidAccountQuery;

        ensure!(!sfid_id.is_empty(), pallet::Error::<T>::EmptySfidId);

        let info = pallet::ClearingBankNodes::<T>::get(&sfid_id)
            .ok_or(pallet::Error::<T>::ClearingBankNodeNotFound)?;

        let bank_main = Self::lookup_main_account_by_sfid(sfid_id.as_slice())?;
        ensure!(
            T::SfidAccountQuery::is_admin_of(&bank_main, &who),
            pallet::Error::<T>::UnauthorizedAdmin
        );

        // 删除主索引 + 反向索引
        pallet::ClearingBankNodes::<T>::remove(&sfid_id);
        pallet::NodePeerToInstitution::<T>::remove(&info.peer_id);

        Self::deposit_event(pallet::Event::ClearingBankUnregistered {
            sfid_id,
            unregistered_by: who,
        });
        Ok(())
    }
}
