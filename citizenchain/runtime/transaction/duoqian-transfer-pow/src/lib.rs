//! # 机构多签名地址转账模块 (duoqian-transfer-pow)
//!
//! 本模块为治理机构（NRC/PRC/PRB）和注册多签机构提供链上转账治理流程：
//! - 管理员发起转账提案，经内部投票通过后自动执行转账并扣取手续费。
//! - 自动执行失败时保留提案状态，可通过 `execute_transfer` 手动重试。
//! - 余额在提案创建和执行两个时点双重检查，含手续费和 ED 保留。
//! - 收款地址不能是机构自身，也不能是受保护地址（质押地址）。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, BoundedVec};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::sr25519::{Public as Sr25519Public, Signature as Sr25519Signature};
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};
use sp_runtime::Vec;
use sp_std_btreeset::BTreeSet;

// 中文注释:本 pallet 未声明 sp_std 依赖,从 sp_runtime 重新导出取 Vec;
// BTreeSet 走 alloc::collections。下面的 shim 把 BTreeSet 带入本模块作用域。
mod sp_std_btreeset {
    pub use alloc::collections::BTreeSet;
}
extern crate alloc;

use primitives::china::china_cb::{
    shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB, NRC_ANQUAN_ADDRESS,
};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use primitives::core_const::{
    DUOQIAN_DOMAIN, OP_SIGN_SAFETY_FUND, OP_SIGN_SWEEP, OP_SIGN_TRANSFER,
};
use voting_engine_system::{
    internal_vote::{ORG_DUOQIAN, ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, STATUS_EXECUTED, STATUS_PASSED,
};

pub use pallet::*;
/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"dq-xfer";

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

type BalanceOf<T> = <<T as duoqian_manage_pow::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

/// 转账动作：记录一次转账提案的完整业务参数。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxRemarkLen))]
pub struct TransferAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    /// 转出机构
    pub institution: InstitutionPalletId,
    /// 收款地址
    pub beneficiary: AccountId,
    /// 转账金额
    pub amount: Balance,
    /// 备注
    pub remark: BoundedVec<u8, MaxRemarkLen>,
    /// 发起管理员
    pub proposer: AccountId,
}

/// 安全基金转账动作：从国储会安全基金账户向指定收款地址转账。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxRemarkLen))]
pub struct SafetyFundAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    /// 收款地址
    pub beneficiary: AccountId,
    /// 转账金额
    pub amount: Balance,
    /// 备注
    pub remark: BoundedVec<u8, MaxRemarkLen>,
    /// 发起管理员
    pub proposer: AccountId,
}

/// 手续费划转动作：从机构手续费账户向机构主账户划转。
///
/// Step 2 · 离线聚合改造:新增 `proposer` 字段,用于 `TransferVoteIntent` 构造时
/// 标识提案发起人,与 transfer / safety_fund 两类动作对齐,保证三组业务签名消息
/// 结构一致。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SweepAction<AccountId, Balance> {
    /// 机构标识
    pub institution: InstitutionPalletId,
    /// 划转金额
    pub amount: Balance,
    /// 发起管理员(Tx 1 中锁定)
    pub proposer: AccountId,
}

/// 三类多签转账(transfer / safety_fund / sweep)共享的离线管理员签名意图。
///
/// Step 2 · 离线 QR 聚合签名铁律:
/// - 一个 struct 覆盖三组业务,字段统一
/// - 三个 `op_tag`(`OP_SIGN_TRANSFER` / `_SAFETY_FUND` / `_SWEEP`)做签名域硬隔离
/// - 任一组的签名不能被跨组重放,因为签名消息 hash 的 preimage 含 op_tag
///
/// 每个管理员扫描发起人导出的 QR 后对此 intent 的 `signing_hash(ss58, op_tag)` 做
/// sr25519 签名,回传给发起人聚合,发起人一笔 `finalize_X` 代投。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct TransferVoteIntent<AccountId, Balance> {
    /// 投票引擎分配的提案 ID
    pub proposal_id: u64,
    /// ORG_NRC / ORG_PRC / ORG_PRB / ORG_DUOQIAN
    pub org: u8,
    /// 机构 pallet_id(48 字节)
    pub institution: InstitutionPalletId,
    /// 资金源地址(transfer:主账户 · safety_fund:NRC_ANQUAN · sweep:费用账户)
    pub from: AccountId,
    /// 资金目标地址(transfer/safety_fund:beneficiary · sweep:主账户)
    pub to: AccountId,
    pub amount: Balance,
    /// `blake2_256(remark)`;sweep 无 remark,固定为 `blake2_256(b"")`
    pub remark_hash: [u8; 32],
    /// Tx 1 锁定的发起人
    pub proposer: AccountId,
    /// 恒 true,占位防误签
    pub approve: bool,
}

impl<AccountId: Encode, Balance: Encode> TransferVoteIntent<AccountId, Balance> {
    /// 按签名域铁律构造标准签名消息 hash。
    ///
    /// - `op_tag` 必须是 `OP_SIGN_TRANSFER / _SAFETY_FUND / _SWEEP` 三者之一
    /// - preimage = `DUOQIAN_DOMAIN(10B) || op_tag(1B) || ss58_le(2B) || blake2_256(intent.encode())`
    /// - signing_hash = `blake2_256(preimage)`(32 字节 sr25519 签名消息)
    ///
    /// wuminapp 端必须用同样公式 + 同样 SCALE 布局构造 intent 后计算 signing_hash,
    /// 不一致会导致签名验证失败。
    pub fn signing_hash(&self, ss58_prefix: u16, op_tag: u8) -> [u8; 32] {
        let intent_hash = sp_io::hashing::blake2_256(&self.encode());
        let mut preimage = Vec::with_capacity(10 + 1 + 2 + 32);
        preimage.extend_from_slice(DUOQIAN_DOMAIN);
        preimage.push(op_tag);
        preimage.extend_from_slice(&ss58_prefix.to_le_bytes());
        preimage.extend_from_slice(&intent_hash);
        sp_io::hashing::blake2_256(&preimage)
    }
}

/// 手续费账户最低保留余额：1111.11 元（111111 分）。
const FEE_ADDRESS_MIN_RESERVE_FEN: u128 = 111_111;

/// 单次划转上限：可用余额的 80%。
const FEE_SWEEP_MAX_PERCENT: u128 = 80;

/// 中文注释：判断机构属于 NRC/PRC/PRB（不含注册多签，注册多签由链上存储判断）。
fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    if CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        == Some(institution)
    {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    if CHINA_CH
        .iter()
        .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRB);
    }

    None
}

/// 中文注释：从 CHINA_CB/CHINA_CH 中查找机构的多签账户地址（main_address）。
fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    if let Some(node) = CHINA_CB
        .iter()
        .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
    {
        return Some(node.main_address);
    }

    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .map(|n| n.main_address)
}

/// 中文注释：检查机构 ID 后 16 字节是否全零（注册多签机构的 ID 格式要求）。
fn institution_id_has_zero_suffix(institution: InstitutionPalletId) -> bool {
    institution[32..].iter().all(|b| *b == 0)
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use duoqian_manage_pow::ProtectedSourceChecker;
    use frame_support::traits::ExistenceRequirement;
    use frame_support::traits::OnUnbalanced;
    use institution_asset_guard::{InstitutionAssetAction, InstitutionAssetGuard};
    use voting_engine_system::InternalAdminProvider;
    use voting_engine_system::InternalThresholdProvider;
    use voting_engine_system::InternalVoteEngine;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + voting_engine_system::Config + duoqian_manage_pow::Config
    {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 备注最大长度
        #[pallet::constant]
        type MaxRemarkLen: Get<u32>;

        /// 手续费分账路由（复用 PowOnchainFeeRouter）
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <<Self as duoqian_manage_pow::Config>::Currency as Currency<
                Self::AccountId,
            >>::NegativeImbalance,
        >;

        /// Weight 配置
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 活跃提案数限制已移至 voting-engine-system::active_proposal_limit 全局管控。
    // 提案业务数据和元数据已统一存储到 voting-engine-system（ProposalData / ProposalMeta）。

    /// 安全基金转账提案动作存储。
    #[pallet::storage]
    pub type SafetyFundProposalActions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        SafetyFundAction<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>,
        OptionQuery,
    >;

    /// 手续费划转提案动作存储（省储行 + 国储会共用）。
    #[pallet::storage]
    pub type SweepProposalActions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        SweepAction<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 转账提案已创建(Tx 1)。wuminapp 扫描此事件后即可构造 QR 分发给各管理员签名。
        TransferProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            /// 资金源(= 机构主账户),Step 2 新增供 QR 生成
            from: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            /// 原文 remark,供管理员扫 QR 时人眼核对;链上验签用 `blake2_256(remark)`
            remark: BoundedVec<u8, T::MaxRemarkLen>,
            /// 投票引擎分配的超时区块,供 wuminapp 倒计时
            expires_at: BlockNumberFor<T>,
        },
        /// 投票通过但执行失败（投票已记录，提案数据保留，可通过 execute_transfer 手动重试）
        TransferExecutionFailed {
            proposal_id: u64,
            institution: InstitutionPalletId,
        },
        /// 转账已执行（投票通过后自动触发，含手续费分账）
        TransferExecuted {
            proposal_id: u64,
            institution: InstitutionPalletId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// finalize_transfer 代投完成(无论最终状态):统计接受签名数 + 投票引擎当前状态。
        TransferFinalized {
            proposal_id: u64,
            signatures_accepted: u32,
            final_status: u8,
        },
        /// 安全基金转账提案已创建(Tx 1)。
        SafetyFundTransferProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            /// 资金源(= NRC_ANQUAN_ADDRESS 常量)
            from: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            /// 原文 remark,供管理员扫 QR 核对
            remark: BoundedVec<u8, T::MaxRemarkLen>,
            expires_at: BlockNumberFor<T>,
        },
        /// 安全基金转账已执行
        SafetyFundTransferExecuted {
            proposal_id: u64,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 安全基金投票通过但执行失败
        SafetyFundExecutionFailed { proposal_id: u64 },
        /// finalize_safety_fund_transfer 代投完成。
        SafetyFundFinalized {
            proposal_id: u64,
            signatures_accepted: u32,
            final_status: u8,
        },
        /// 手续费划转提案已创建(Tx 1)。
        SweepToMainProposed {
            proposal_id: u64,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            /// 资金源(= 机构费用账户)
            from: T::AccountId,
            /// 资金目标(= 机构主账户)
            to: T::AccountId,
            amount: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// 手续费划转已执行
        SweepToMainExecuted {
            proposal_id: u64,
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
            reserve_left: BalanceOf<T>,
        },
        /// 手续费划转投票通过但执行失败
        SweepExecutionFailed { proposal_id: u64 },
        /// finalize_sweep_to_main 代投完成。
        SweepToMainFinalized {
            proposal_id: u64,
            signatures_accepted: u32,
            final_status: u8,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释：机构不属于 NRC/PRC/PRB 且非注册多签机构。
        InvalidInstitution,
        /// 中文注释：调用者声明的 org 类型与机构实际类型不一致。
        InstitutionOrgMismatch,
        /// 中文注释：调用者不是该机构的管理员。
        UnauthorizedAdmin,
        /// 中文注释：机构资产保护检查未通过（如冻结期间禁止支出）。
        InstitutionSpendNotAllowed,
        /// 中文注释：转账金额不能为零。
        ZeroAmount,
        /// 中文注释：转账金额低于 ED（存在性保证金），收款地址可能无法创建。
        AmountBelowExistentialDeposit,
        /// 中文注释：不允许转账给机构自身。
        SelfTransferNotAllowed,
        /// 中文注释：收款地址是受保护地址（如质押地址），不允许作为收款方。
        BeneficiaryIsProtectedAddress,
        /// 中文注释：提案动作数据未找到或解码失败。
        ProposalActionNotFound,
        /// 中文注释：机构账户地址解码失败。
        InstitutionAccountDecodeFailed,
        /// 中文注释：机构余额不足（需 amount + fee + ED）。
        InsufficientBalance,
        /// 中文注释：提案未达到通过状态，不可执行。
        ProposalNotPassed,
        /// 中文注释：链上转账操作失败。
        TransferFailed,
        /// 中文注释：安全基金提案未找到。
        SafetyFundProposalNotFound,
        /// 中文注释：安全基金余额不足。
        SafetyFundInsufficientBalance,
        /// 中文注释：安全基金提案未通过。
        SafetyFundProposalNotPassed,
        /// 中文注释：手续费划转提案未找到。
        SweepProposalNotFound,
        /// 中文注释：手续费划转金额无效。
        InvalidSweepAmount,
        /// 中文注释：手续费账户余额不足（需保留最低余额）。
        InsufficientFeeReserve,
        /// 中文注释：手续费划转金额超过上限（可用余额的 80%）。
        SweepAmountExceedsCap,
        /// 中文注释：手续费划转提案未通过。
        SweepProposalNotPassed,
        // ── Step 2 · 离线 QR 聚合签名改造新增错误 ──
        /// finalize_X 提交的签名对应的 admin 不在该机构管理员列表
        UnauthorizedSignature,
        /// finalize_X 同一 admin 在同一批签名里重复出现
        DuplicateSignature,
        /// finalize_X sr25519 签名验证失败
        InvalidSignature,
        /// finalize_X 提交的签名数量少于阈值
        InsufficientSignatures,
        /// finalize_X sr25519 签名长度必须恰好为 64 字节
        MalformedSignature,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起机构多签名地址转账提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_transfer())]
        pub fn propose_transfer(
            origin: OriginFor<T>,
            org: u8,
            institution: InstitutionPalletId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            remark: BoundedVec<u8, T::MaxRemarkLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
            let (actual_org, institution_account) = Self::resolve_institution_account(institution)?;
            ensure!(actual_org == org, Error::<T>::InstitutionOrgMismatch);
            ensure!(
                Self::is_internal_admin(org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            ensure!(
                <T as duoqian_manage_pow::Config>::InstitutionAssetGuard::can_spend(
                    &institution_account,
                    InstitutionAssetAction::DuoqianTransferExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // 转账金额必须 >= ED，防止收款地址不存在时创建失败
            let ed = <T as duoqian_manage_pow::Config>::Currency::minimum_balance();
            ensure!(amount >= ed, Error::<T>::AmountBelowExistentialDeposit);

            // 不允许自转账
            ensure!(
                beneficiary != institution_account,
                Error::<T>::SelfTransferNotAllowed
            );

            // 不允许转到受保护地址（质押地址）
            ensure!(
                !<T as duoqian_manage_pow::Config>::ProtectedSourceChecker::is_protected(
                    &beneficiary,
                ),
                Error::<T>::BeneficiaryIsProtectedAddress
            );

            // 活跃提案数由 voting-engine-system 在 create_internal_proposal 中统一检查

            // 预检余额（含手续费，与执行时检查一致，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;
            let free =
                <T as duoqian_manage_pow::Config>::Currency::free_balance(&institution_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // 创建内部投票提案
            let proposal_id =
                <T as duoqian_manage_pow::Config>::InternalVoteEngine::create_internal_proposal(
                    who.clone(),
                    org,
                    institution,
                )?;

            let action = TransferAction {
                institution,
                beneficiary: beneficiary.clone(),
                amount,
                remark: remark.clone(),
                proposer: who.clone(),
            };
            let mut encoded = sp_runtime::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, encoded)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(
                proposal_id,
                frame_system::Pallet::<T>::block_number(),
            );

            // 从投票引擎回读 proposal.end 作为 expires_at,供 wuminapp 倒计时。
            let expires_at = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::ProposalActionNotFound)?;

            Self::deposit_event(Event::<T>::TransferProposed {
                proposal_id,
                org,
                institution,
                proposer: who,
                from: institution_account,
                beneficiary,
                amount,
                remark,
                expires_at,
            });
            Ok(())
        }

        /// Step 2 · 离线 QR 聚合转账:发起人一笔代投 + 自动执行。
        ///
        /// 流程:
        /// 1. 读取 Tx 1 存入的 `TransferAction`
        /// 2. 构造 `TransferVoteIntent`(op_tag = `OP_SIGN_TRANSFER`)
        /// 3. 共用 helper 循环验签 + 代投(`verify_and_cast_votes`)
        /// 4. 投票引擎达阈值自动 `STATUS_PASSED` → 原子执行 `try_execute_transfer`
        /// 5. 执行失败保留提案状态供 `execute_transfer` 手动重试
        ///
        /// 任意签名账户可调用(代付 gas)。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::finalize_transfer(sigs.len() as u32))]
        pub fn finalize_transfer(
            origin: OriginFor<T>,
            proposal_id: u64,
            sigs: BoundedVec<
                (T::AccountId, duoqian_manage_pow::pallet::AdminSignatureOf<T>),
                <T as duoqian_manage_pow::Config>::MaxAdmins,
            >,
        ) -> DispatchResult {
            let _submitter = ensure_signed(origin)?;

            // 1. 读取提案业务数据
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() >= tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            let action = TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                &mut &raw[tag.len()..],
            )
            .map_err(|_| Error::<T>::ProposalActionNotFound)?;

            // 2. 解析 (org, from = 机构主账户)
            let (org, institution_account) = Self::resolve_institution_account(action.institution)?;

            // 3. 从投票引擎查阈值
            let threshold = <T as voting_engine_system::Config>::InternalThresholdProvider::pass_threshold(
                org,
                action.institution,
            )
            .ok_or(Error::<T>::InvalidInstitution)?;

            // 4. 构造 intent + signing_hash
            let remark_hash = sp_io::hashing::blake2_256(action.remark.as_slice());
            let intent = TransferVoteIntent::<T::AccountId, BalanceOf<T>> {
                proposal_id,
                org,
                institution: action.institution,
                from: institution_account.clone(),
                to: action.beneficiary.clone(),
                amount: action.amount,
                remark_hash,
                proposer: action.proposer.clone(),
                approve: true,
            };
            let signing_hash = intent.signing_hash(T::SS58Prefix::get(), OP_SIGN_TRANSFER);

            // 5. 验签 + 代投
            let accepted = Self::verify_and_cast_votes(
                proposal_id,
                org,
                action.institution,
                threshold,
                &signing_hash,
                &sigs,
            )?;

            // 6. 达阈值则自动执行;失败保留提案供 execute_transfer 重试
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            if proposal.status == STATUS_PASSED {
                let institution = action.institution;
                if Self::try_execute_transfer(proposal_id).is_err() {
                    Self::deposit_event(Event::<T>::TransferExecutionFailed {
                        proposal_id,
                        institution,
                    });
                }
            }

            // 读回最新 status(try_execute_transfer 成功推到 EXECUTED)
            let final_status = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.status)
                .unwrap_or(proposal.status);
            Self::deposit_event(Event::<T>::TransferFinalized {
                proposal_id,
                signatures_accepted: accepted,
                final_status,
            });

            Ok(())
        }

        /// 手动执行已通过的转账提案。
        ///
        /// 当投票通过后自动执行失败（如余额不足），可在补充余额后通过此接口重试。
        /// 任何签名账户都可调用，避免因管理员离线导致通过的提案无法落地。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::execute_transfer())]
        pub fn execute_transfer(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::try_execute_transfer(proposal_id)
        }

        /// 发起国储会安全基金转账提案（内部投票）。
        ///
        /// 从安全基金账户（`NRC_ANQUAN_ADDRESS`）向指定收款地址转账。
        /// 仅国储会管理员可发起。
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_safety_fund_transfer(
            origin: OriginFor<T>,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            remark: BoundedVec<u8, T::MaxRemarkLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);

            // 验证国储会管理员
            let nrc_institution = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                    ORG_NRC,
                    nrc_institution,
                    &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            // 验证安全基金账户余额
            let safety_fund_account = T::AccountId::decode(&mut &NRC_ANQUAN_ADDRESS[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
            ensure!(
                <T as duoqian_manage_pow::Config>::InstitutionAssetGuard::can_spend(
                    &safety_fund_account,
                    InstitutionAssetAction::NrcSafetyFundTransfer,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // 预检余额（含手续费，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            let ed: BalanceOf<T> = <T as duoqian_manage_pow::Config>::Currency::minimum_balance();
            let free =
                <T as duoqian_manage_pow::Config>::Currency::free_balance(&safety_fund_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            // 创建内部投票提案
            let proposal_id =
                <T as duoqian_manage_pow::Config>::InternalVoteEngine::create_internal_proposal(
                    who.clone(),
                    ORG_NRC,
                    nrc_institution,
                )?;

            SafetyFundProposalActions::<T>::insert(
                proposal_id,
                SafetyFundAction {
                    beneficiary: beneficiary.clone(),
                    amount,
                    remark: remark.clone(),
                    proposer: who.clone(),
                },
            );

            // 从投票引擎回读 proposal.end 作为 expires_at。
            let expires_at = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;

            Self::deposit_event(Event::SafetyFundTransferProposed {
                proposal_id,
                proposer: who,
                from: safety_fund_account,
                beneficiary,
                amount,
                remark,
                expires_at,
            });
            Ok(())
        }

        /// Step 2 · 离线 QR 聚合安全基金转账:发起人一笔代投 + 自动执行。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::finalize_safety_fund_transfer(sigs.len() as u32))]
        pub fn finalize_safety_fund_transfer(
            origin: OriginFor<T>,
            proposal_id: u64,
            sigs: BoundedVec<
                (T::AccountId, duoqian_manage_pow::pallet::AdminSignatureOf<T>),
                <T as duoqian_manage_pow::Config>::MaxAdmins,
            >,
        ) -> DispatchResult {
            let _submitter = ensure_signed(origin)?;

            // 1. 读取 SafetyFundAction
            let action = SafetyFundProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;

            // 2. org + institution 固定(NRC)
            let org = ORG_NRC;
            let nrc_institution = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
                .ok_or(Error::<T>::InvalidInstitution)?;
            let safety_fund_account = T::AccountId::decode(&mut &NRC_ANQUAN_ADDRESS[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            // 3. 阈值
            let threshold = <T as voting_engine_system::Config>::InternalThresholdProvider::pass_threshold(
                org,
                nrc_institution,
            )
            .ok_or(Error::<T>::InvalidInstitution)?;

            // 4. 构造 intent + signing_hash(用 OP_SIGN_SAFETY_FUND 做签名域隔离)
            let remark_hash = sp_io::hashing::blake2_256(action.remark.as_slice());
            let intent = TransferVoteIntent::<T::AccountId, BalanceOf<T>> {
                proposal_id,
                org,
                institution: nrc_institution,
                from: safety_fund_account.clone(),
                to: action.beneficiary.clone(),
                amount: action.amount,
                remark_hash,
                proposer: action.proposer.clone(),
                approve: true,
            };
            let signing_hash = intent.signing_hash(T::SS58Prefix::get(), OP_SIGN_SAFETY_FUND);

            // 5. 验签 + 代投
            let accepted = Self::verify_and_cast_votes(
                proposal_id,
                org,
                nrc_institution,
                threshold,
                &signing_hash,
                &sigs,
            )?;

            // 6. 达阈值 → 原子执行,失败发 SafetyFundExecutionFailed
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;
            if proposal.status == STATUS_PASSED {
                let exec_result = frame_support::storage::with_transaction(|| {
                    match Self::try_execute_safety_fund(proposal_id) {
                        Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                        Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
                    }
                });
                if exec_result.is_err() {
                    Self::deposit_event(Event::SafetyFundExecutionFailed { proposal_id });
                }
            }

            let final_status = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.status)
                .unwrap_or(proposal.status);
            Self::deposit_event(Event::SafetyFundFinalized {
                proposal_id,
                signatures_accepted: accepted,
                final_status,
            });

            Ok(())
        }

        /// 发起手续费划转提案（省储行或国储会管理员）。
        ///
        /// 从机构手续费账户向机构主账户划转。划转后手续费账户至少保留 1111.11 元。
        #[pallet::call_index(5)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_sweep_to_main(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let amount_u128: u128 = amount.saturated_into();
            ensure!(amount_u128 > 0, Error::<T>::InvalidSweepAmount);

            // 动态判断 org 类型
            let org = Self::resolve_sweep_org(institution)?;
            ensure!(
                <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                    org,
                    institution,
                    &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                <T as duoqian_manage_pow::Config>::InternalVoteEngine::create_internal_proposal(
                    who.clone(),
                    org,
                    institution,
                )?;

            SweepProposalActions::<T>::insert(
                proposal_id,
                SweepAction {
                    institution,
                    amount,
                    proposer: who.clone(),
                },
            );

            let fee_account = Self::resolve_fee_account(institution)?;
            let main_account = Self::resolve_main_account(institution)?;
            let expires_at = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            Self::deposit_event(Event::SweepToMainProposed {
                proposal_id,
                institution,
                proposer: who,
                from: fee_account,
                to: main_account,
                amount,
                expires_at,
            });
            Ok(())
        }

        /// Step 2 · 离线 QR 聚合费用→主账户划转:发起人一笔代投 + 自动执行。
        ///
        /// sweep 无 remark,intent 里 `remark_hash = blake2_256(b"")`。
        #[pallet::call_index(6)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::finalize_sweep_to_main(sigs.len() as u32))]
        pub fn finalize_sweep_to_main(
            origin: OriginFor<T>,
            proposal_id: u64,
            sigs: BoundedVec<
                (T::AccountId, duoqian_manage_pow::pallet::AdminSignatureOf<T>),
                <T as duoqian_manage_pow::Config>::MaxAdmins,
            >,
        ) -> DispatchResult {
            let _submitter = ensure_signed(origin)?;

            // 1. 读取 SweepAction
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            // 2. 解析 org + from(fee_account) + to(main_account)
            let org = Self::resolve_sweep_org(action.institution)?;
            let fee_account = Self::resolve_fee_account(action.institution)?;
            let main_account = Self::resolve_main_account(action.institution)?;

            // 3. 阈值
            let threshold = <T as voting_engine_system::Config>::InternalThresholdProvider::pass_threshold(
                org,
                action.institution,
            )
            .ok_or(Error::<T>::InvalidInstitution)?;

            // 4. intent(sweep 无 remark,用 blake2_256(b""))+ OP_SIGN_SWEEP 隔离签名域
            let remark_hash = sp_io::hashing::blake2_256(&[][..]);
            let intent = TransferVoteIntent::<T::AccountId, BalanceOf<T>> {
                proposal_id,
                org,
                institution: action.institution,
                from: fee_account,
                to: main_account,
                amount: action.amount,
                remark_hash,
                proposer: action.proposer.clone(),
                approve: true,
            };
            let signing_hash = intent.signing_hash(T::SS58Prefix::get(), OP_SIGN_SWEEP);

            // 5. 验签 + 代投
            let accepted = Self::verify_and_cast_votes(
                proposal_id,
                org,
                action.institution,
                threshold,
                &signing_hash,
                &sigs,
            )?;

            // 6. 达阈值 → 原子执行,失败发 SweepExecutionFailed
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;
            if proposal.status == STATUS_PASSED {
                let exec_result = frame_support::storage::with_transaction(|| {
                    match Self::try_execute_sweep(proposal_id) {
                        Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                        Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
                    }
                });
                if exec_result.is_err() {
                    Self::deposit_event(Event::SweepExecutionFailed { proposal_id });
                }
            }

            let final_status = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.status)
                .unwrap_or(proposal.status);
            Self::deposit_event(Event::SweepToMainFinalized {
                proposal_id,
                signatures_accepted: accepted,
                final_status,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn registered_duoqian_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, Error<T>> {
            ensure!(
                institution_id_has_zero_suffix(institution),
                Error::<T>::InvalidInstitution
            );
            let account = T::AccountId::decode(&mut &institution[..32])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
            let duoqian = duoqian_manage_pow::DuoqianAccounts::<T>::get(&account)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                duoqian.status == duoqian_manage_pow::DuoqianStatus::Active,
                Error::<T>::InvalidInstitution
            );
            Ok(account)
        }

        fn resolve_institution_account(
            institution: InstitutionPalletId,
        ) -> Result<(u8, T::AccountId), Error<T>> {
            if let Some(actual_org) = institution_org(institution) {
                let raw_account = institution_pallet_address(institution)
                    .ok_or(Error::<T>::InvalidInstitution)?;
                let institution_account = T::AccountId::decode(&mut &raw_account[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
                return Ok((actual_org, institution_account));
            }

            let institution_account = Self::registered_duoqian_account(institution)?;
            Ok((ORG_DUOQIAN, institution_account))
        }

        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        /// 三类 finalize_X 共用:循环验签 + 代投。
        ///
        /// 任一签名失败(成员校验 / 去重 / 长度 / 验签 / 代投)都让整笔交易回滚,
        /// 不接受部分签名语义。
        ///
        /// `signing_hash` 由调用方按 `TransferVoteIntent::signing_hash(ss58, op_tag)`
        /// 事先算好传入,本函数只做"签名有效性 + 管理员身份 + 投票引擎代投"。
        ///
        /// 返回接受并代投成功的签名数(= sigs.len(),若循环中途失败则函数已 Err 返回)。
        fn verify_and_cast_votes(
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            threshold: u32,
            signing_hash: &[u8; 32],
            sigs: &BoundedVec<
                (T::AccountId, duoqian_manage_pow::pallet::AdminSignatureOf<T>),
                <T as duoqian_manage_pow::Config>::MaxAdmins,
            >,
        ) -> Result<u32, DispatchError> {
            ensure!(
                sigs.len() as u32 >= threshold,
                Error::<T>::InsufficientSignatures
            );

            let mut seen: BTreeSet<T::AccountId> = BTreeSet::new();
            let mut accepted: u32 = 0;
            for (admin, sig_bytes) in sigs.iter() {
                // 成员校验:必须是该机构的内部管理员
                ensure!(
                    Self::is_internal_admin(org, institution, admin),
                    Error::<T>::UnauthorizedSignature
                );
                // 同批次去重
                ensure!(
                    seen.insert(admin.clone()),
                    Error::<T>::DuplicateSignature
                );
                // sr25519 签名长度必须恰好 64 字节
                ensure!(sig_bytes.len() == 64, Error::<T>::MalformedSignature);
                let sig = Sr25519Signature::try_from(sig_bytes.as_slice())
                    .map_err(|_| Error::<T>::MalformedSignature)?;
                // AccountId 前 32 字节 = sr25519 公钥(复用 Step 1 建立的铁律)
                let pubkey: Sr25519Public =
                    duoqian_manage_pow::Pallet::<T>::pubkey_from_accountid(admin)
                        .map_err(|_| Error::<T>::MalformedSignature)?;
                // 验签
                ensure!(
                    sp_io::crypto::sr25519_verify(&sig, signing_hash, &pubkey),
                    Error::<T>::InvalidSignature
                );
                // 代投(投票引擎内部自动做快照检查 / AlreadyVoted / 阈值 / 状态推进)
                <T as duoqian_manage_pow::Config>::InternalVoteEngine::cast_internal_vote(
                    admin.clone(),
                    proposal_id,
                    true,
                )?;
                accepted = accepted.saturating_add(1);
            }
            Ok(accepted)
        }

        /// 判断机构的 org 类型用于 sweep 提案。
        fn resolve_sweep_org(institution: InstitutionPalletId) -> Result<u8, Error<T>> {
            // 国储会
            if CHINA_CB
                .first()
                .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
                == Some(institution)
            {
                return Ok(ORG_NRC);
            }
            // 省储行
            if CHINA_CH
                .iter()
                .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
                .any(|pid| pid == institution)
            {
                return Ok(ORG_PRB);
            }
            Err(Error::<T>::InvalidInstitution)
        }

        /// 解析机构手续费账户。
        fn resolve_fee_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            // 国储会：使用常量地址
            if CHINA_CB
                .first()
                .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
                == Some(institution)
            {
                return T::AccountId::decode(&mut &CHINA_CB[0].fee_address[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into());
            }
            // 省储行：使用 fee_address（BLAKE2-256 派生）
            let node = CHINA_CH
                .iter()
                .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                .ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &node.fee_address[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        /// 解析机构主账户。
        fn resolve_main_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            let raw =
                institution_pallet_address(institution).ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &raw[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        /// 执行手续费划转。
        fn try_execute_sweep(proposal_id: u64) -> DispatchResult {
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SweepProposalNotPassed
            );

            let fee_account = Self::resolve_fee_account(action.institution)?;
            let main_account = Self::resolve_main_account(action.institution)?;

            ensure!(
                <T as duoqian_manage_pow::Config>::InstitutionAssetGuard::can_spend(
                    &fee_account,
                    InstitutionAssetAction::OffchainFeeSweepExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费 ──
            let amount_u128: u128 = action.amount.saturated_into();
            let tx_fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let tx_fee: BalanceOf<T> = tx_fee_u128.saturated_into();

            let fee_balance_u128: u128 =
                <T as duoqian_manage_pow::Config>::Currency::free_balance(&fee_account)
                    .saturated_into();
            let reserve_u128 = FEE_ADDRESS_MIN_RESERVE_FEN;

            // ── 余额检查：amount + tx_fee + reserve ──
            let total_deduct_u128 = amount_u128.saturating_add(tx_fee_u128);
            ensure!(
                fee_balance_u128 >= total_deduct_u128
                    && fee_balance_u128.saturating_sub(total_deduct_u128) >= reserve_u128,
                Error::<T>::InsufficientFeeReserve
            );
            // ── cap 检查：划转金额不超过可用余额的 80%（可用 = 余额 - reserve） ──
            let available_u128 = fee_balance_u128.saturating_sub(reserve_u128);
            let cap_u128 = available_u128
                .saturating_mul(FEE_SWEEP_MAX_PERCENT)
                .saturating_div(100);
            ensure!(amount_u128 <= cap_u128, Error::<T>::SweepAmountExceedsCap);

            // ── 执行划转 ──
            <T as duoqian_manage_pow::Config>::Currency::transfer(
                &fee_account,
                &main_account,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // ── 手续费：从费用账户扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as duoqian_manage_pow::Config>::Currency::withdraw(
                &fee_account,
                tx_fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientFeeReserve)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            let reserve_left =
                <T as duoqian_manage_pow::Config>::Currency::free_balance(&fee_account);

            Self::deposit_event(Event::SweepToMainExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
                fee: tx_fee,
                reserve_left,
            });
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;
            Ok(())
        }

        /// 执行安全基金转账（投票通过后自动调用）。
        fn try_execute_safety_fund(proposal_id: u64) -> DispatchResult {
            let action = SafetyFundProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SafetyFundProposalNotPassed
            );

            let safety_fund_account = T::AccountId::decode(&mut &NRC_ANQUAN_ADDRESS[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            ensure!(
                <T as duoqian_manage_pow::Config>::InstitutionAssetGuard::can_spend(
                    &safety_fund_account,
                    InstitutionAssetAction::NrcSafetyFundTransfer,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费 ──
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = action
                .amount
                .checked_add(&fee)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 余额检查：amount + fee + ED ──
            let free =
                <T as duoqian_manage_pow::Config>::Currency::free_balance(&safety_fund_account);
            let ed = <T as duoqian_manage_pow::Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            // ── 执行转账 ──
            <T as duoqian_manage_pow::Config>::Currency::transfer(
                &safety_fund_account,
                &action.beneficiary,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 手续费：从安全基金扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as duoqian_manage_pow::Config>::Currency::withdraw(
                &safety_fund_account,
                fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;

            Self::deposit_event(Event::SafetyFundTransferExecuted {
                proposal_id,
                beneficiary: action.beneficiary,
                amount: action.amount,
                fee,
            });

            Ok(())
        }

        /// 从 voting-engine-system 读取提案数据并执行转账。
        /// vote_transfer（自动执行）和 execute_transfer（手动重试）共用此逻辑。
        fn try_execute_transfer(proposal_id: u64) -> DispatchResult {
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() >= tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            let action = TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                &mut &raw[tag.len()..],
            )
            .map_err(|_| Error::<T>::ProposalActionNotFound)?;
            let (_, institution_account) = Self::resolve_institution_account(action.institution)?;
            ensure!(
                <T as duoqian_manage_pow::Config>::InstitutionAssetGuard::can_spend(
                    &institution_account,
                    InstitutionAssetAction::DuoqianTransferExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费（复用 onchain-transaction-pow 公共接口） ──
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = action
                .amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;

            // ── 余额检查：需要 total + ED ──
            let free =
                <T as duoqian_manage_pow::Config>::Currency::free_balance(&institution_account);
            let ed = <T as duoqian_manage_pow::Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // ── 原子执行：手续费扣取 + 转账，任一失败整体回滚 ──
            let exec_result = frame_support::storage::with_transaction(|| {
                // 先扣手续费
                let fee_imbalance = match <T as duoqian_manage_pow::Config>::Currency::withdraw(
                    &institution_account,
                    fee,
                    frame_support::traits::WithdrawReasons::FEE,
                    ExistenceRequirement::KeepAlive,
                ) {
                    Ok(imbalance) => imbalance,
                    Err(_) => {
                        return frame_support::storage::TransactionOutcome::Rollback(Err(
                            Error::<T>::InsufficientBalance.into(),
                        ))
                    }
                };
                <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

                // 再转账
                match <T as duoqian_manage_pow::Config>::Currency::transfer(
                    &institution_account,
                    &action.beneficiary,
                    action.amount,
                    ExistenceRequirement::KeepAlive,
                ) {
                    Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                    Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
                }
            });
            exec_result?;

            // ── 标记为已执行，防止双重执行 ──
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;

            Self::deposit_event(Event::<T>::TransferExecuted {
                proposal_id,
                institution: action.institution,
                beneficiary: action.beneficiary,
                amount: action.amount,
                fee,
            });
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU128, ConstU32},
    };
    use frame_system as system;
    use sp_core::{sr25519, Pair as PairT};
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use voting_engine_system::STATUS_REJECTED;

    type Balance = u128;
    type Block = frame_system::mocking::MockBlock<Test>;

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type Balances = pallet_balances;

        #[runtime::pallet_index(2)]
        pub type VotingEngineSystem = voting_engine_system;

        #[runtime::pallet_index(3)]
        pub type DuoqianManagePow = duoqian_manage_pow;

        #[runtime::pallet_index(4)]
        pub type DuoqianTransferPow = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
        type AccountData = pallet_balances::AccountData<Balance>;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = ConstU32<0>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
        type WeightInfo = ();
    }

    pub struct TestAddressValidator;
    impl duoqian_manage_pow::DuoqianAddressValidator<AccountId32> for TestAddressValidator {
        fn is_valid(address: &AccountId32) -> bool {
            address != &AccountId32::new([0u8; 32])
        }
    }

    pub struct TestReservedAddressChecker;
    impl duoqian_manage_pow::DuoqianReservedAddressChecker<AccountId32> for TestReservedAddressChecker {
        fn is_reserved(address: &AccountId32) -> bool {
            *address == AccountId32::new([0xAA; 32])
        }
    }

    pub struct TestSfidInstitutionVerifier;
    impl
        duoqian_manage_pow::SfidInstitutionVerifier<
            duoqian_manage_pow::pallet::AccountNameOf<Test>,
            duoqian_manage_pow::pallet::RegisterNonceOf<Test>,
            duoqian_manage_pow::pallet::RegisterSignatureOf<Test>,
        > for TestSfidInstitutionVerifier
    {
        fn verify_institution_registration(
            _sfid_id: &[u8],
            _account_name: &duoqian_manage_pow::pallet::AccountNameOf<Test>,
            nonce: &duoqian_manage_pow::pallet::RegisterNonceOf<Test>,
            signature: &duoqian_manage_pow::pallet::RegisterSignatureOf<Test>,
            _signing_province: Option<&[u8]>,
        ) -> bool {
            !nonce.is_empty() && signature.as_slice() == b"register-ok"
        }
    }

    pub struct TestSfidEligibility;
    impl voting_engine_system::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
        ) -> bool {
            true
        }
    }

    pub struct TestPopulationSnapshotVerifier;
    impl
        voting_engine_system::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine_system::pallet::VoteNonceOf<Test>,
            voting_engine_system::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine_system::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine_system::pallet::VoteSignatureOf<Test>,
        ) -> bool {
            true
        }
    }

    // Step 2 · 测试扩展:
    // 原 TestInternalAdminProvider 只读 CHINA_CB/CHINA_CH 硬编码 admin(非真实 sr25519 公钥,无法签名)。
    // 为支持 finalize_X 的 sr25519 验签路径,新增 thread_local 覆盖层:
    //   - EXTRA_ADMINS 按 (org, institution) 注入 sr25519 派生 admin 集合
    //   - EXTRA_THRESHOLDS 按 (org, institution) 覆盖阈值(方便用 2 个签名就达标)
    // 若某 (org, institution) 在 thread_local 有注入,优先用;否则 fallback 到原硬编码逻辑。
    thread_local! {
        static EXTRA_ADMINS: core::cell::RefCell<
            alloc::collections::BTreeMap<(u8, InstitutionPalletId), alloc::vec::Vec<AccountId32>>,
        > = core::cell::RefCell::new(alloc::collections::BTreeMap::new());
        static EXTRA_THRESHOLDS: core::cell::RefCell<
            alloc::collections::BTreeMap<(u8, InstitutionPalletId), u32>,
        > = core::cell::RefCell::new(alloc::collections::BTreeMap::new());
    }

    fn set_extra_admins(org: u8, institution: InstitutionPalletId, admins: Vec<AccountId32>) {
        EXTRA_ADMINS.with(|m| {
            m.borrow_mut().insert((org, institution), admins);
        });
    }

    fn set_extra_threshold(org: u8, institution: InstitutionPalletId, threshold: u32) {
        EXTRA_THRESHOLDS.with(|m| {
            m.borrow_mut().insert((org, institution), threshold);
        });
    }

    fn get_extra_admins(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
        EXTRA_ADMINS.with(|m| m.borrow().get(&(org, institution)).cloned())
    }

    fn get_extra_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
        EXTRA_THRESHOLDS.with(|m| m.borrow().get(&(org, institution)).copied())
    }

    pub struct TestInternalAdminProvider;
    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            // 优先:测试注入的 sr25519 派生 admin
            if let Some(admins) = get_extra_admins(org, institution) {
                return admins.iter().any(|a| a == who);
            }
            // Fallback:原硬编码 admin
            let who_bytes = who.encode();
            if who_bytes.len() != 32 {
                return false;
            }
            let mut who_arr = [0u8; 32];
            who_arr.copy_from_slice(&who_bytes);
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                ORG_DUOQIAN => {
                    let Ok(account) = AccountId32::decode(&mut &institution[..32]) else {
                        return false;
                    };
                    if let Some(duoqian) =
                        duoqian_manage_pow::DuoqianAccounts::<Test>::get(&account)
                    {
                        duoqian.duoqian_admins.iter().any(|admin| admin == who)
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }

        fn get_admin_list(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
            if let Some(admins) = get_extra_admins(org, institution) {
                return Some(admins);
            }
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    }),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    }),
                ORG_DUOQIAN => {
                    let account = AccountId32::decode(&mut &institution[..32]).ok()?;
                    let duoqian = duoqian_manage_pow::DuoqianAccounts::<Test>::get(&account)?;
                    Some(duoqian.duoqian_admins.into_inner())
                }
                _ => None,
            }
        }
    }

    pub struct TestInternalAdminCountProvider;
    impl voting_engine_system::InternalAdminCountProvider for TestInternalAdminCountProvider {
        fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
                ORG_DUOQIAN => {
                    let account = AccountId32::decode(&mut &institution[..32]).ok()?;
                    let duoqian = duoqian_manage_pow::DuoqianAccounts::<Test>::get(&account)?;
                    u32::try_from(duoqian.duoqian_admins.len()).ok()
                }
                _ => None,
            }
        }
    }

    pub struct TestInternalThresholdProvider;
    impl voting_engine_system::InternalThresholdProvider for TestInternalThresholdProvider {
        fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            // 优先:测试注入的阈值覆盖(用于把 NRC/PRC/PRB 的大阈值降到 2,方便 finalize 测试)
            if let Some(t) = get_extra_threshold(org, institution) {
                return Some(t);
            }
            match org {
                ORG_NRC | ORG_PRC | ORG_PRB => {
                    voting_engine_system::internal_vote::governance_org_pass_threshold(org)
                }
                ORG_DUOQIAN => {
                    let account = AccountId32::decode(&mut &institution[..32]).ok()?;
                    let duoqian = duoqian_manage_pow::DuoqianAccounts::<Test>::get(&account)?;
                    Some(duoqian.threshold)
                }
                _ => None,
            }
        }
    }

    thread_local! {
        static PROTECTED_ADDRESS: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
        static DENIED_SPEND_SOURCE: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
    }

    pub struct TestProtectedSourceChecker;
    impl duoqian_manage_pow::ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
        fn is_protected(address: &AccountId32) -> bool {
            PROTECTED_ADDRESS.with(|pa| pa.borrow().as_ref() == Some(address))
        }
    }

    pub struct TestInstitutionAssetGuard;
    impl institution_asset_guard::InstitutionAssetGuard<AccountId32> for TestInstitutionAssetGuard {
        fn can_spend(
            source: &AccountId32,
            _action: institution_asset_guard::InstitutionAssetAction,
        ) -> bool {
            DENIED_SPEND_SOURCE.with(|blocked| blocked.borrow().as_ref() != Some(source))
        }
    }

    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalAdminCountProvider = TestInternalAdminCountProvider;
        type InternalThresholdProvider = TestInternalThresholdProvider;
        type MaxAdminsPerInstitution = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<1024>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl duoqian_manage_pow::pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type AddressValidator = TestAddressValidator;
        type ReservedAddressChecker = TestReservedAddressChecker;
        type ProtectedSourceChecker = TestProtectedSourceChecker;
        type InstitutionAssetGuard = TestInstitutionAssetGuard;
        type SfidInstitutionVerifier = TestSfidInstitutionVerifier;
        type FeeRouter = ();
        type MaxAdmins = ConstU32<10>;
        type MaxSfidIdLength = ConstU32<96>;
        type MaxAccountNameLength = ConstU32<128>;
        type MaxRegisterNonceLength = ConstU32<64>;
        type MaxRegisterSignatureLength = ConstU32<64>;
        type MaxAdminSignatureLength = ConstU32<64>;
        type MinCreateAmount = ConstU128<111>;
        type MinCloseBalance = ConstU128<111>;
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxRemarkLen = ConstU32<256>;
        type FeeRouter = ();
        type WeightInfo = ();
    }

    /// Step 2 · 测试 helper:从 (org, institution, index) 派生 sr25519 keypair。
    ///
    /// 同 (org, institution, index) 每次调用返回相同 keypair,保证测试确定性。
    /// 公钥的 32 字节直接作为 AccountId32,满足 `pubkey_from_accountid` 的铁律。
    fn derive_admin_pair(
        org: u8,
        institution: &InstitutionPalletId,
        index: u8,
    ) -> (AccountId32, sr25519::Pair) {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[0] = org;
        seed_bytes[1] = index;
        // 后 30 字节由 institution_pallet_id 前 30 字节填充,保证不同机构的 seed 不同
        seed_bytes[2..32].copy_from_slice(&institution[..30]);
        let pair = sr25519::Pair::from_seed(&seed_bytes);
        let account = AccountId32::new(pair.public().0);
        (account, pair)
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        derive_admin_pair(ORG_NRC, &nrc_pallet_id(), index as u8).0
    }

    fn prc_admin(index: usize) -> AccountId32 {
        derive_admin_pair(ORG_PRC, &prc_pallet_id(), index as u8).0
    }

    fn prb_admin(index: usize) -> AccountId32 {
        derive_admin_pair(ORG_PRB, &prb_pallet_id(), index as u8).0
    }

    /// Step 2 · 离线聚合签名 helper:
    /// 对 `TransferVoteIntent` 做 sr25519 签名,按管理员 pair 数组批量产出 sigs。
    fn make_transfer_sigs(
        pairs: &[(AccountId32, sr25519::Pair)],
        take: usize,
        proposal_id: u64,
        org: u8,
        institution: InstitutionPalletId,
        from: AccountId32,
        to: AccountId32,
        amount: Balance,
        remark_hash: [u8; 32],
        proposer: AccountId32,
        op_tag: u8,
    ) -> BoundedVec<
        (
            AccountId32,
            duoqian_manage_pow::pallet::AdminSignatureOf<Test>,
        ),
        <Test as duoqian_manage_pow::Config>::MaxAdmins,
    > {
        let intent = TransferVoteIntent::<AccountId32, Balance> {
            proposal_id,
            org,
            institution,
            from,
            to,
            amount,
            remark_hash,
            proposer,
            approve: true,
        };
        let ss58 = <Test as frame_system::Config>::SS58Prefix::get();
        let msg = intent.signing_hash(ss58, op_tag);
        let sigs: Vec<_> = pairs
            .iter()
            .take(take)
            .map(|(a, p)| {
                let sig_bytes: duoqian_manage_pow::pallet::AdminSignatureOf<Test> = p
                    .sign(&msg)
                    .0
                    .to_vec()
                    .try_into()
                    .expect("sig 64 bytes fits");
                (a.clone(), sig_bytes)
            })
            .collect();
        sigs.try_into().expect("sigs vec fits MaxAdmins")
    }

    fn nrc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id).expect("nrc id should be valid")
    }

    fn prc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("prc id should be valid")
    }

    fn prb_pallet_id() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id).expect("prb id should be valid")
    }

    fn institution_account(institution: InstitutionPalletId) -> AccountId32 {
        let raw =
            institution_pallet_address(institution).expect("institution pallet address must exist");
        AccountId32::new(raw)
    }

    fn registered_duoqian_account() -> AccountId32 {
        AccountId32::new([0x55; 32])
    }

    fn registered_duoqian_institution() -> InstitutionPalletId {
        duoqian_manage_pow::account_to_institution_id(&registered_duoqian_account())
    }

    fn registered_duoqian_admin(index: usize) -> AccountId32 {
        registered_duoqian_pair(index).0
    }

    /// 注册多签(ORG_DUOQIAN)的 admin sr25519 keypair helper。
    /// seed 按 (ORG_DUOQIAN, registered_duoqian_institution, index) 派生,保证确定性。
    fn registered_duoqian_pair(index: usize) -> (AccountId32, sr25519::Pair) {
        derive_admin_pair(ORG_DUOQIAN, &registered_duoqian_institution(), index as u8)
    }

    fn registered_duoqian_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        (0..count).map(|i| registered_duoqian_pair(i as usize)).collect()
    }

    /// 收款人：使用一个不是管理员也不是机构的普通地址
    fn beneficiary() -> AccountId32 {
        AccountId32::new([99u8; 32])
    }

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine_system::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    /// 返回 (org, institution) 对应的前 `count` 个 sr25519 admin keypair。
    fn admin_pairs(
        org: u8,
        institution: InstitutionPalletId,
        count: u8,
    ) -> Vec<(AccountId32, sr25519::Pair)> {
        (0..count)
            .map(|i| derive_admin_pair(org, &institution, i))
            .collect()
    }

    fn nrc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        admin_pairs(ORG_NRC, nrc_pallet_id(), count)
    }

    fn prc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        admin_pairs(ORG_PRC, prc_pallet_id(), count)
    }

    fn prb_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        admin_pairs(ORG_PRB, prb_pallet_id(), count)
    }

    /// Step 2 · finalize_transfer 测试辅助:生成 N 个签名 + 调 extrinsic。
    ///
    /// 入参 `remark` 必须与 Tx 1 `propose_transfer` 传入的 remark 字节完全一致,
    /// 否则 `remark_hash` 不同,验签失败。
    fn finalize_transfer_n(
        pairs: &[(AccountId32, sr25519::Pair)],
        n: usize,
        pid: u64,
        org: u8,
        institution: InstitutionPalletId,
        from: AccountId32,
        to: AccountId32,
        amount: Balance,
        remark: &[u8],
        proposer: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        let remark_hash = sp_io::hashing::blake2_256(remark);
        let sigs = make_transfer_sigs(
            pairs,
            n,
            pid,
            org,
            institution,
            from,
            to,
            amount,
            remark_hash,
            proposer.clone(),
            OP_SIGN_TRANSFER,
        );
        DuoqianTransferPow::finalize_transfer(RuntimeOrigin::signed(proposer), pid, sigs)
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");

        let balances = vec![
            (institution_account(nrc_pallet_id()), 10_000),
            (institution_account(prc_pallet_id()), 10_000),
            (institution_account(prb_pallet_id()), 10_000),
        ];
        pallet_balances::GenesisConfig::<Test> {
            balances,
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .expect("balances should assimilate");

        let mut ext: sp_io::TestExternalities = storage.into();
        ext.execute_with(|| {
            // Step 2 · 离线聚合改造:为 4 种 org 注入 sr25519 派生 admin(每组前 3 个)+
            // 覆盖阈值 2,让 finalize_X 测试只需 2 个签名即可达阈值。
            // Provider 的 is_internal_admin / get_admin_list / pass_threshold 会优先
            // 读 thread_local 注入,未注入时 fallback 到 CHINA_CB / CHINA_CH 硬编码。
            let nrc = nrc_pallet_id();
            let prc = prc_pallet_id();
            let prb = prb_pallet_id();
            let dq = registered_duoqian_institution();
            let nrc_accts: Vec<AccountId32> = nrc_pairs(3).into_iter().map(|(a, _)| a).collect();
            let prc_accts: Vec<AccountId32> = prc_pairs(3).into_iter().map(|(a, _)| a).collect();
            let prb_accts: Vec<AccountId32> = prb_pairs(3).into_iter().map(|(a, _)| a).collect();
            set_extra_admins(ORG_NRC, nrc, nrc_accts);
            set_extra_admins(ORG_PRC, prc, prc_accts);
            set_extra_admins(ORG_PRB, prb, prb_accts);
            set_extra_threshold(ORG_NRC, nrc, 2);
            set_extra_threshold(ORG_PRC, prc, 2);
            set_extra_threshold(ORG_PRB, prb, 2);
            // ORG_DUOQIAN 的 admin / threshold 直接从 DuoqianAccounts 读,测试需要时
            // 显式写入 DuoqianAccounts(见 `registered_duoqian_admin` 路径)。
            let _ = dq;
        });
        ext
    }

    #[test]
    fn nrc_transfer_executes_when_finalize_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                1_000,
                &[],
                nrc_admin(0),
            ));

            // 转账已执行（含手续费 10）
            assert_eq!(Balances::free_balance(&inst_account), 8_990);
            assert_eq!(Balances::free_balance(&dest), 1_000);
            // 提案数据仍保留（由 voting-engine-system 延迟清理）
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn prc_transfer_executes_when_finalize_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                dest.clone(),
                2_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &prc_pairs(2),
                2,
                pid,
                ORG_PRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                2_000,
                &[],
                prc_admin(0),
            ));

            assert_eq!(Balances::free_balance(&inst_account), 7_990);
            assert_eq!(Balances::free_balance(&dest), 2_000);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn prb_transfer_executes_when_finalize_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                dest.clone(),
                3_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &prb_pairs(2),
                2,
                pid,
                ORG_PRB,
                institution,
                inst_account.clone(),
                dest.clone(),
                3_000,
                &[],
                prb_admin(0),
            ));

            assert_eq!(Balances::free_balance(&inst_account), 6_990);
            assert_eq!(Balances::free_balance(&dest), 3_000);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn registered_duoqian_transfer_executes_when_finalize_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = registered_duoqian_institution();
            let inst_account = registered_duoqian_account();
            let dest = beneficiary();
            let admins = BoundedVec::try_from(vec![
                registered_duoqian_admin(0),
                registered_duoqian_admin(1),
                registered_duoqian_admin(2),
            ])
            .expect("admins should fit");

            duoqian_manage_pow::DuoqianAccounts::<Test>::insert(
                &inst_account,
                duoqian_manage_pow::DuoqianAccount {
                    admin_count: 3,
                    threshold: 2,
                    duoqian_admins: admins,
                    creator: registered_duoqian_admin(0),
                    created_at: 1,
                    status: duoqian_manage_pow::DuoqianStatus::Active,
                },
            );
            let _ = Balances::deposit_creating(&inst_account, 10_000);

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(registered_duoqian_admin(0)),
                ORG_DUOQIAN,
                institution,
                dest.clone(),
                1_500,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &registered_duoqian_pairs(2),
                2,
                pid,
                ORG_DUOQIAN,
                institution,
                inst_account.clone(),
                dest.clone(),
                1_500,
                &[],
                registered_duoqian_admin(0),
            ));

            assert_eq!(Balances::free_balance(&inst_account), 8_490);
            assert_eq!(Balances::free_balance(&dest), 1_500);
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTED
            );
        });
    }

    #[test]
    fn non_admin_cannot_propose_or_finalize() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // PRC 管理员不能给 NRC 提案
            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(prc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest.clone(),
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 把 PRC 管理员的签名 + 其他非 NRC 管理员的公钥塞进 finalize_transfer
            // 应被 UnauthorizedSignature 拒绝
            let res = finalize_transfer_n(
                &prc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                100,
                &[],
                nrc_admin(0),
            );
            assert_noop!(res, Error::<Test>::UnauthorizedSignature);
        });
    }

    #[test]
    fn zero_amount_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest,
                    0,
                    BoundedVec::default(),
                ),
                Error::<Test>::ZeroAmount
            );
        });
    }

    #[test]
    fn self_transfer_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    inst_account,
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::SelfTransferNotAllowed
            );
        });
    }

    #[test]
    fn insufficient_balance_is_rejected_on_propose() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            // 余额 10_000，fee=10，ED=1：最多 amount=9_989（9_989+10+1=10_000）
            // amount=9_990 时 required=9_990+10+1=10_001 > 10_000 → 拒绝
            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest.clone(),
                    9_990,
                    BoundedVec::default(),
                ),
                Error::<Test>::InsufficientBalance
            );

            // amount=9_989 时 required=9_989+10+1=10_000 → 刚好通过
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                9_989,
                BoundedVec::default(),
            ));
        });
    }

    #[test]
    fn duplicate_signature_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 同一 admin 签名两次,同批次内应被 DuplicateSignature 拒绝
            let dup_pair = nrc_pairs(1)[0].clone();
            let pairs = vec![dup_pair.clone(), dup_pair];
            let res = finalize_transfer_n(
                &pairs,
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account,
                dest,
                100,
                &[],
                nrc_admin(0),
            );
            assert_noop!(res, Error::<Test>::DuplicateSignature);
        });
    }

    #[test]
    fn multiple_proposals_allowed_within_limit() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));

            // 活跃提案数限制由 voting-engine-system 全局管控（上限 10），第二个提案可以成功
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                200,
                BoundedVec::default(),
            ));
        });
    }

    #[test]
    fn executed_transfer_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid1 = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid1,
                ORG_NRC,
                institution,
                inst_account,
                dest.clone(),
                100,
                &[],
                nrc_admin(0),
            ));

            // 转账已执行，可以创建新提案
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                200,
                BoundedVec::default(),
            ));
        });
    }

    #[test]
    fn rejected_proposal_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid1 = last_proposal_id();

            let end = voting_engine_system::Pallet::<Test>::proposals(pid1)
                .expect("proposal should exist")
                .end;
            System::set_block_number(end + 1);
            assert_ok!(voting_engine_system::Pallet::<Test>::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid1
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid1)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );

            // 被拒绝后可以创建新提案
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                50,
                BoundedVec::default(),
            ));
        });
    }

    #[test]
    fn existential_deposit_is_preserved() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // 余额 10_000，ED=1，手续费=10，提案 9_989 刚好使剩余 = ED
            // required = 9_989 + 10(fee) + 1(ED) = 10_000
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_989,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                9_989,
                &[],
                nrc_admin(0),
            ));

            assert_eq!(Balances::free_balance(&inst_account), 1);
            assert_eq!(Balances::free_balance(&dest), 9_989);
        });
    }

    #[test]
    fn execute_transfer_succeeds_after_failed_auto_execution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // 余额 10_000,提案 9_000(预检通过),然后在 finalize 前转走 9_000
            // 使余额仅 1_000,finalize 时自动执行因余额不足失败,但提案保留,可 execute_transfer 重试。
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // finalize 前转走余额,使自动执行失败
            let drain_dest = AccountId32::new([88u8; 32]);
            let _ = Balances::deposit_creating(&drain_dest, 1);
            assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
                &inst_account,
                &drain_dest,
                9_000,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ));
            assert_eq!(Balances::free_balance(&inst_account), 1_000);

            // finalize:签名验证 + 代投通过,但 try_execute_transfer 因余额不足失败。
            // 提案仍为 PASSED,转账未执行。
            assert_ok!(finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                9_000,
                &[],
                nrc_admin(0),
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(Balances::free_balance(&dest), 0);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());

            // 补充余额后手动执行
            let _ = Balances::deposit_creating(&inst_account, 9_000);
            assert_eq!(Balances::free_balance(&inst_account), 10_000);
            assert_ok!(DuoqianTransferPow::execute_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid
            ));
            // 转账成功：9_000 转出 + 10 手续费
            assert_eq!(Balances::free_balance(&inst_account), 990);
            assert_eq!(Balances::free_balance(&dest), 9_000);
        });
    }

    #[test]
    fn execute_transfer_rejects_non_passed_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 提案仍在投票中，不能手动执行
            assert_noop!(
                DuoqianTransferPow::execute_transfer(RuntimeOrigin::signed(nrc_admin(0)), pid),
                Error::<Test>::ProposalNotPassed
            );
        });
    }

    #[test]
    fn execute_transfer_is_callable_by_non_admin() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();
            let outsider = AccountId32::new([88u8; 32]);
            let _ = Balances::deposit_creating(&outsider, 1);

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 减余额使自动执行失败
            let drain_dest = AccountId32::new([77u8; 32]);
            let _ = Balances::deposit_creating(&drain_dest, 1);
            assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
                &inst_account,
                &drain_dest,
                9_900,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ));

            assert_ok!(finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                100,
                &[],
                nrc_admin(0),
            ));

            // 自动执行失败，补充余额
            assert_eq!(Balances::free_balance(&dest), 0);
            let _ = Balances::deposit_creating(&inst_account, 10_000);

            // 非管理员也能调用 execute_transfer
            assert_ok!(DuoqianTransferPow::execute_transfer(
                RuntimeOrigin::signed(outsider),
                pid
            ));
            assert_eq!(Balances::free_balance(&dest), 100);
        });
    }

    #[test]
    fn executed_transfer_cannot_be_executed_again() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account,
                dest,
                1_000,
                &[],
                nrc_admin(0),
            ));

            // 自动执行成功，状态变为 EXECUTED
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTED
            );

            // 再次调用 execute_transfer 应被拒绝
            assert_noop!(
                DuoqianTransferPow::execute_transfer(RuntimeOrigin::signed(nrc_admin(0)), pid),
                Error::<Test>::ProposalNotPassed
            );
        });
    }

    #[test]
    fn protected_address_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let protected = AccountId32::new([77u8; 32]);

            // 标记为受保护地址
            PROTECTED_ADDRESS.with(|pa| *pa.borrow_mut() = Some(protected.clone()));

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    protected,
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::BeneficiaryIsProtectedAddress
            );
        });
    }

    #[test]
    fn institution_spend_guard_blocks_transfer_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let source = institution_account(institution);
            let dest = beneficiary();
            DENIED_SPEND_SOURCE.with(|blocked| *blocked.borrow_mut() = Some(source.clone()));

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest,
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::InstitutionSpendNotAllowed
            );

            DENIED_SPEND_SOURCE.with(|blocked| *blocked.borrow_mut() = None);
        });
    }

    #[test]
    fn fee_respects_minimum_on_small_amount() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // amount=1, 费率计算 1×0.1%=0.001 < 最低 10 分，手续费应为 10
            // required = 1 + 10 + 1(ED) = 12
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                1,
                &[],
                nrc_admin(0),
            ));

            // 余额 10_000 - 1(转账) - 10(最低手续费) = 9_989
            assert_eq!(Balances::free_balance(&inst_account), 9_989);
            assert_eq!(Balances::free_balance(&dest), 1);
        });
    }

    // ──── Step 2 · 离线 QR 聚合专项测试 ────

    /// finalize_transfer 签名不足阈值时必须拒绝。
    #[test]
    fn finalize_transfer_insufficient_sigs_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 只提交 1 个签名,阈值 2 → InsufficientSignatures
            let res = finalize_transfer_n(
                &nrc_pairs(1),
                1,
                pid,
                ORG_NRC,
                institution,
                inst_account,
                dest,
                100,
                &[],
                nrc_admin(0),
            );
            assert_noop!(res, Error::<Test>::InsufficientSignatures);
        });
    }

    /// 篡改 amount 后签名验证失败。
    #[test]
    fn finalize_transfer_tampered_amount_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 管理员用 amount=999 签名,但链上实际 amount=100
            // → 链上重算 intent.amount=100,signing_hash 与 sig 不匹配 → InvalidSignature
            let res = finalize_transfer_n(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account,
                dest,
                999, // 错误 amount
                &[],
                nrc_admin(0),
            );
            assert_noop!(res, Error::<Test>::InvalidSignature);
        });
    }

    /// 签名长度非 64 字节必须拒绝。
    #[test]
    fn finalize_transfer_malformed_sig_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 构造一个合法 sig 和一个长度 32 的非法 sig
            let sigs: Vec<_> = {
                let pairs = nrc_pairs(2);
                let remark_hash = sp_io::hashing::blake2_256(&[][..]);
                let intent = TransferVoteIntent::<AccountId32, Balance> {
                    proposal_id: pid,
                    org: ORG_NRC,
                    institution,
                    from: inst_account.clone(),
                    to: dest.clone(),
                    amount: 100,
                    remark_hash,
                    proposer: nrc_admin(0),
                    approve: true,
                };
                let msg = intent.signing_hash(
                    <Test as frame_system::Config>::SS58Prefix::get(),
                    OP_SIGN_TRANSFER,
                );
                let good: duoqian_manage_pow::pallet::AdminSignatureOf<Test> = pairs[0]
                    .1
                    .sign(&msg)
                    .0
                    .to_vec()
                    .try_into()
                    .expect("good sig fits");
                let bad: duoqian_manage_pow::pallet::AdminSignatureOf<Test> =
                    vec![0u8; 32].try_into().expect("32 bytes fits");
                vec![
                    (pairs[0].0.clone(), good),
                    (pairs[1].0.clone(), bad),
                ]
            };
            let sigs_bounded: BoundedVec<_, <Test as duoqian_manage_pow::Config>::MaxAdmins> =
                sigs.try_into().expect("sigs vec fits");
            assert_noop!(
                DuoqianTransferPow::finalize_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    pid,
                    sigs_bounded,
                ),
                Error::<Test>::MalformedSignature
            );
        });
    }

    /// finalize_safety_fund_transfer 端到端成功路径。
    #[test]
    fn finalize_safety_fund_end_to_end() {
        new_test_ext().execute_with(|| {
            let safety_fund_account = AccountId32::decode(
                &mut &primitives::china::china_cb::NRC_ANQUAN_ADDRESS[..],
            )
            .expect("NRC_ANQUAN decodes");
            // 为安全基金预置余额
            let _ = Balances::deposit_creating(&safety_fund_account, 100_000);

            let dest = beneficiary();
            let amount: Balance = 5_000;

            assert_ok!(DuoqianTransferPow::propose_safety_fund_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                dest.clone(),
                amount,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 构造 finalize_safety_fund_transfer 的 sigs(用 OP_SIGN_SAFETY_FUND 签名域)
            let remark_hash = sp_io::hashing::blake2_256(&[][..]);
            let sigs = make_transfer_sigs(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                nrc_pallet_id(),
                safety_fund_account.clone(),
                dest.clone(),
                amount,
                remark_hash,
                nrc_admin(0),
                OP_SIGN_SAFETY_FUND,
            );
            assert_ok!(DuoqianTransferPow::finalize_safety_fund_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid,
                sigs,
            ));

            // 转账已执行(amount 5_000 + fee 10)
            assert_eq!(Balances::free_balance(&dest), 5_000);
            assert_eq!(Balances::free_balance(&safety_fund_account), 100_000 - 5_000 - 10);
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal exists")
                    .status,
                STATUS_EXECUTED
            );
        });
    }

    /// finalize_sweep_to_main 端到端成功路径(省储行 ORG_PRB)。
    #[test]
    fn finalize_sweep_end_to_end() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();
            let fee_account = AccountId32::new(CHINA_CH[0].fee_address);
            let main_account = AccountId32::new(CHINA_CH[0].main_address);

            // 给费用账户预置大额余额,满足 1111.11 元 reserve + 80% cap
            let _ = Balances::deposit_creating(&fee_account, 1_000_000);

            let amount: Balance = 100_000; // 1000 元,远低于 80% cap

            assert_ok!(DuoqianTransferPow::propose_sweep_to_main(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                amount,
            ));
            let pid = last_proposal_id();

            // 构造 finalize_sweep_to_main sigs(用 OP_SIGN_SWEEP + 空 remark)
            let remark_hash = sp_io::hashing::blake2_256(&[][..]);
            let sigs = make_transfer_sigs(
                &prb_pairs(2),
                2,
                pid,
                ORG_PRB,
                institution,
                fee_account.clone(),
                main_account.clone(),
                amount,
                remark_hash,
                prb_admin(0),
                OP_SIGN_SWEEP,
            );
            assert_ok!(DuoqianTransferPow::finalize_sweep_to_main(
                RuntimeOrigin::signed(prb_admin(0)),
                pid,
                sigs,
            ));

            // 转账已执行:费用账户 -amount -fee,主账户 +amount
            let main_after = Balances::free_balance(&main_account);
            assert!(main_after >= 10_000 + amount, "main account should receive amount");
        });
    }

    /// 跨业务签名隔离铁律:transfer 的签名不能在 finalize_safety_fund_transfer 里通过。
    /// 即使字段内容"看起来对",由于 op_tag 不同 → signing_hash 不同 → sr25519_verify 失败。
    #[test]
    fn cross_op_signature_rejected_transfer_to_safety_fund() {
        new_test_ext().execute_with(|| {
            let safety_fund_account = AccountId32::decode(
                &mut &primitives::china::china_cb::NRC_ANQUAN_ADDRESS[..],
            )
            .expect("NRC_ANQUAN decodes");
            let _ = Balances::deposit_creating(&safety_fund_account, 100_000);

            let dest = beneficiary();
            let amount: Balance = 5_000;

            assert_ok!(DuoqianTransferPow::propose_safety_fund_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                dest.clone(),
                amount,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 管理员错误地用 OP_SIGN_TRANSFER 签名,企图给 safety_fund 提案投票
            let remark_hash = sp_io::hashing::blake2_256(&[][..]);
            let sigs = make_transfer_sigs(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                nrc_pallet_id(),
                safety_fund_account,
                dest,
                amount,
                remark_hash,
                nrc_admin(0),
                OP_SIGN_TRANSFER, // 错误 op_tag
            );
            assert_noop!(
                DuoqianTransferPow::finalize_safety_fund_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    pid,
                    sigs,
                ),
                Error::<Test>::InvalidSignature
            );
        });
    }

    /// 跨业务签名隔离铁律:sweep 的签名不能在 finalize_transfer 里通过。
    #[test]
    fn cross_op_signature_rejected_sweep_to_transfer() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 管理员错误地用 OP_SIGN_SWEEP 签名,企图给 transfer 提案投票
            let remark_hash = sp_io::hashing::blake2_256(&[][..]);
            let sigs = make_transfer_sigs(
                &nrc_pairs(2),
                2,
                pid,
                ORG_NRC,
                institution,
                inst_account,
                dest,
                100,
                remark_hash,
                nrc_admin(0),
                OP_SIGN_SWEEP, // 错误 op_tag
            );
            assert_noop!(
                DuoqianTransferPow::finalize_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    pid,
                    sigs,
                ),
                Error::<Test>::InvalidSignature
            );
        });
    }
}
