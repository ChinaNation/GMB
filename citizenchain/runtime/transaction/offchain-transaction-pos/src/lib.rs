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

use frame_support::{
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::Currency,
    traits::StorageVersion,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;

/// 清算行清算 pallet 的存储版本。Step 2b-iv-b 清理老省储行 Storage 后从 1 → 2。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[frame_support::pallet]
pub mod pallet {
    use super::*;

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
        /// [`institution_main`] 批次归属的清算行主账户地址(= 付款方清算行)
        /// [`batch_seq`] 清算行内单调递增的批次序号(冗余审计字段,Step 2b 启用严格校验)
        /// [`batch`] `OffchainBatchItemV2` 列表(每条带 L3 sr25519 签名 / nonce / 费率)
        /// [`batch_signature`] 清算行多签批次级签名(Step 2b 启用阈值校验)
        #[pallet::call_index(34)]
        #[pallet::weight(
            T::DbWeight::get().reads_writes(6, 6)
                + T::DbWeight::get().reads_writes(5, 6) * batch.len() as u64
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
            // batch_signature 与 batch_seq 暂仅作审计冗余,Step 2b 接多签阈值后严格校验。
            let _ = (batch_seq, batch_signature);

            with_transaction(|| {
                match crate::settlement::execute_clearing_bank_batch::<T>(
                    &submitter,
                    &institution_main,
                    batch.as_slice(),
                ) {
                    Ok(()) => TransactionOutcome::Commit(Ok(())),
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
}

impl<T: pallet::Config> pallet::Pallet<T> {
    /// 反查清算行主账户对应的费用账户(辅助 ops / off-chain ledger)。
    pub fn fee_account_of(bank_main: &T::AccountId) -> Result<T::AccountId, pallet::Error<T>> {
        crate::bank_check::fee_account_of::<T>(bank_main)
    }
}
