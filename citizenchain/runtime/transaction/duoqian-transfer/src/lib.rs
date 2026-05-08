//! # 机构多签名地址转账模块 (duoqian-transfer)
//!
//! 本模块为治理机构（NRC/PRC/PRB）和注册多签机构提供链上转账治理流程：
//! - 管理员发起转账提案，经内部投票通过后自动执行转账并扣取手续费。
//! - 自动执行失败时保留提案状态，可通过 `VotingEngine::retry_passed_proposal` 手动重试。
//! - 余额在提案创建和执行两个时点双重检查，含手续费和 ED 保留。
//! - 收款地址不能是机构自身，也不能是受保护地址（质押地址）。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, BoundedVec};
use frame_system::pallet_prelude::*;
use primitives::derive::{subject_id_from_sfid_number, SubjectKind};
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};

// 统一状态机整改:原业务自有投票/finalize 路径所依赖的
// `sr25519::{Public, Signature}` / `Vec` / `BTreeSet` 已全部随投票统一入口
// 改造下线,不再从此处导入。
extern crate alloc;

use primitives::china::china_cb::{CHINA_CB, NRC_ANQUAN_ADDRESS};
use primitives::china::china_ch::CHINA_CH;
use votingengine::{
    types::{ORG_NRC, ORG_PRB, ORG_PRC, ORG_REN},
    InternalVoteResultCallback, ProposalExecutionOutcome, SubjectId, STATUS_PASSED,
};

pub use pallet::*;
/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"dq-xfer";
const SAFETY_FUND_OWNER_DATA: &[u8] = b"dq-xfer:safety";
const SWEEP_OWNER_DATA: &[u8] = b"dq-xfer:sweep";

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

type BalanceOf<T> = <<T as organization_manage::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

/// 转账动作：记录一次转账提案的完整业务参数。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxRemarkLen))]
pub struct TransferAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    /// 转出机构
    pub institution: SubjectId,
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
/// `proposer` 字段与 transfer / safety_fund 两类动作对齐,便于 Executor 在
/// 投票通过 / 否决回调时统一识别提案发起人。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SweepAction<AccountId, Balance> {
    /// 机构标识
    pub institution: SubjectId,
    /// 划转金额
    pub amount: Balance,
    /// 发起管理员(Tx 1 中锁定)
    pub proposer: AccountId,
}

/// 手续费账户最低保留余额：1111.11 元（111111 分）。
const FEE_ADDRESS_MIN_RESERVE_FEN: u128 = 111_111;

/// 单次划转上限：可用余额的 80%。
const FEE_SWEEP_MAX_PERCENT: u128 = 80;

/// 中文注释：判断机构属于 NRC/PRC/PRB（不含注册多签，注册多签由链上存储判断）。
fn subject_org(institution: SubjectId) -> Option<u8> {
    if CHINA_CB
        .first()
        .and_then(|n| subject_id_from_sfid_number(n.sfid_number))
        == Some(institution)
    {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| subject_id_from_sfid_number(n.sfid_number))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    if CHINA_CH
        .iter()
        .filter_map(|n| subject_id_from_sfid_number(n.sfid_number))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRB);
    }

    None
}

/// 中文注释：从 CHINA_CB/CHINA_CH 中查找机构的多签账户地址（main_address）。
fn subject_pallet_address(institution: SubjectId) -> Option<[u8; 32]> {
    if let Some(node) = CHINA_CB
        .iter()
        .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
    {
        return Some(node.main_address);
    }

    CHINA_CH
        .iter()
        .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
        .map(|n| n.main_address)
}

/// 中文注释:从账户级 SubjectId 中取出 AccountId。
///
/// D/ADR-015 协议:
/// - `0x03 PersonalDuoqian`:个人多签账户；
/// - `0x05 InstitutionAccount`:SFID 机构下面的某个具体可操作账户；
/// - `0x02 SfidInstitution` 只表示机构归属/检索,不能作为资金账户发起转账。
fn account_bytes_from_subject_id(institution: SubjectId, kind: SubjectKind) -> Option<[u8; 32]> {
    if institution[0] != kind as u8 || !institution[33..].iter().all(|b| *b == 0) {
        return None;
    }
    let mut account = [0u8; 32];
    account.copy_from_slice(&institution[1..33]);
    Some(account)
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::traits::ExistenceRequirement;
    use frame_support::traits::OnUnbalanced;
    use institution_asset::{InstitutionAsset, InstitutionAssetAction};
    use organization_manage::ProtectedSourceChecker;
    use votingengine::InternalAdminProvider;
    use votingengine::InternalVoteEngine;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + votingengine::Config + organization_manage::Config
    {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 备注最大长度
        #[pallet::constant]
        type MaxRemarkLen: Get<u32>;

        /// 手续费分账路由（复用 OnchainFeeRouter）
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <<Self as organization_manage::Config>::Currency as Currency<
                Self::AccountId,
            >>::NegativeImbalance,
        >;

        /// 个人多签账户状态查询,由 personal-manage 实现。
        type PersonalQuery: personal_manage::traits::PersonalMultisigQuery<Self::AccountId>;

        /// 注册机构账户状态查询,由 organization-manage 实现。
        type InstitutionQuery: organization_manage::traits::InstitutionMultisigQuery<
            Self::AccountId,
        >;

        /// Weight 配置
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 活跃提案数限制已移至 votingengine::active_proposal_limit 全局管控。
    // 提案业务数据和元数据已统一存储到 votingengine（ProposalData / ProposalMeta）。

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
    pub type SweepProposalActions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, SweepAction<T::AccountId, BalanceOf<T>>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 转账提案已创建。wuminapp 可扫描此事件展示投票详情。
        TransferProposed {
            proposal_id: u64,
            org: u8,
            institution: SubjectId,
            proposer: T::AccountId,
            /// 资金源(= 机构主账户)。
            from: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            /// 原文 remark,供管理员投票前核对。
            remark: BoundedVec<u8, T::MaxRemarkLen>,
            /// 投票引擎分配的超时区块,供 wuminapp 倒计时
            expires_at: BlockNumberFor<T>,
        },
        /// 投票通过但执行失败（投票已记录，提案数据保留，可通过 VotingEngine 统一入口手动重试）
        TransferExecutionFailed {
            proposal_id: u64,
            institution: SubjectId,
        },
        /// 转账已执行（投票通过后自动触发，含手续费分账）
        TransferExecuted {
            proposal_id: u64,
            institution: SubjectId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 安全基金转账提案已创建。
        SafetyFundTransferProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            /// 资金源(= NRC_ANQUAN_ADDRESS 常量)
            from: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            /// 原文 remark,供管理员投票前核对。
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
        /// 手续费划转提案已创建。
        SweepToMainProposed {
            proposal_id: u64,
            institution: SubjectId,
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
            institution: SubjectId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
            reserve_left: BalanceOf<T>,
        },
        /// 手续费划转投票通过但执行失败
        SweepExecutionFailed { proposal_id: u64 },
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起多签资金账户转账提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_transfer())]
        pub fn propose_transfer(
            origin: OriginFor<T>,
            org: u8,
            institution: SubjectId,
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
                <T as organization_manage::Config>::InstitutionAsset::can_spend(
                    &institution_account,
                    InstitutionAssetAction::DuoqianTransferExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // 转账金额必须 >= ED，防止收款地址不存在时创建失败
            let ed = <T as organization_manage::Config>::Currency::minimum_balance();
            ensure!(amount >= ed, Error::<T>::AmountBelowExistentialDeposit);

            // 不允许自转账
            ensure!(
                beneficiary != institution_account,
                Error::<T>::SelfTransferNotAllowed
            );

            // 不允许转到受保护地址（质押地址）
            ensure!(
                !<T as organization_manage::Config>::ProtectedSourceChecker::is_protected(
                    &beneficiary,
                ),
                Error::<T>::BeneficiaryIsProtectedAddress
            );

            // 活跃提案数由 votingengine 在 create_internal_proposal 中统一检查

            // 预检余额（含手续费，与执行时检查一致，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;
            let free =
                <T as organization_manage::Config>::Currency::free_balance(&institution_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            let action = TransferAction {
                institution,
                beneficiary: beneficiary.clone(),
                amount,
                remark: remark.clone(),
                proposer: who.clone(),
            };
            let mut encoded = sp_runtime::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            // 中文注释：创建提案时同步写入 owner/data/meta，禁止后续跨模块覆写业务数据。
            let proposal_id =
                <T as organization_manage::Config>::InternalVoteEngine::create_internal_proposal_with_data(
                    who.clone(),
                    org,
                    institution,
                    crate::MODULE_TAG,
                    encoded,
                )?;

            // 从投票引擎回读 proposal.end 作为 expires_at,供 wuminapp 倒计时。
            let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
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

        /// 发起国储会安全基金转账提案（内部投票）。
        ///
        /// 从安全基金账户（`NRC_ANQUAN_ADDRESS`）向指定收款地址转账。
        /// 仅国储会管理员可发起。
        #[pallet::call_index(1)]
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
            let nrc_institution = subject_id_from_sfid_number(CHINA_CB[0].sfid_number)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
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
                <T as organization_manage::Config>::InstitutionAsset::can_spend(
                    &safety_fund_account,
                    InstitutionAssetAction::NrcSafetyFundTransfer,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // 预检余额（含手续费，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            let ed: BalanceOf<T> = <T as organization_manage::Config>::Currency::minimum_balance();
            let free =
                <T as organization_manage::Config>::Currency::free_balance(&safety_fund_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            let proposal_id =
                <T as organization_manage::Config>::InternalVoteEngine::create_internal_proposal_with_data(
                    who.clone(),
                    ORG_NRC,
                    nrc_institution,
                    crate::MODULE_TAG,
                    sp_runtime::Vec::from(SAFETY_FUND_OWNER_DATA),
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
            let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
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

        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_sweep_to_main(
            origin: OriginFor<T>,
            institution: SubjectId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let amount_u128: u128 = amount.saturated_into();
            ensure!(amount_u128 > 0, Error::<T>::InvalidSweepAmount);

            // 动态判断 org 类型
            let org = Self::resolve_sweep_org(institution)?;
            ensure!(
                <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    org,
                    institution,
                    &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                <T as organization_manage::Config>::InternalVoteEngine::create_internal_proposal_with_data(
                    who.clone(),
                    org,
                    institution,
                    crate::MODULE_TAG,
                    sp_runtime::Vec::from(SWEEP_OWNER_DATA),
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
            let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
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

        // call_index = 3, 4, 5 已废弃: execute_transfer /
        // execute_safety_fund_transfer / execute_sweep_to_main 已统一到
        // VotingEngine::retry_passed_proposal —— 前端必须直接调用投票引擎入口。
    }

    impl<T: Config> Pallet<T> {
        fn registered_duoqian_account(institution: SubjectId) -> Result<T::AccountId, Error<T>> {
            use organization_manage::traits::InstitutionMultisigQuery;
            use personal_manage::traits::PersonalMultisigQuery;

            if let Some(raw) =
                account_bytes_from_subject_id(institution, SubjectKind::PersonalDuoqian)
            {
                let account = T::AccountId::decode(&mut &raw[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
                ensure!(
                    <T as Config>::PersonalQuery::is_active(&account),
                    Error::<T>::InvalidInstitution
                );
                return Ok(account);
            }

            if let Some(raw) =
                account_bytes_from_subject_id(institution, SubjectKind::InstitutionAccount)
            {
                let account = T::AccountId::decode(&mut &raw[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
                ensure!(
                    <T as Config>::InstitutionQuery::is_active(&account),
                    Error::<T>::InvalidInstitution
                );
                return Ok(account);
            }

            Err(Error::<T>::InvalidInstitution)
        }

        fn resolve_institution_account(
            institution: SubjectId,
        ) -> Result<(u8, T::AccountId), Error<T>> {
            if let Some(actual_org) = subject_org(institution) {
                let raw_account =
                    subject_pallet_address(institution).ok_or(Error::<T>::InvalidInstitution)?;
                let institution_account = T::AccountId::decode(&mut &raw_account[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
                return Ok((actual_org, institution_account));
            }

            let institution_account = Self::registered_duoqian_account(institution)?;
            Ok((ORG_REN, institution_account))
        }

        fn is_internal_admin(org: u8, institution: SubjectId, who: &T::AccountId) -> bool {
            <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        /// 判断机构的 org 类型用于 sweep 提案。
        fn resolve_sweep_org(institution: SubjectId) -> Result<u8, Error<T>> {
            // 国储会
            if CHINA_CB
                .first()
                .and_then(|n| subject_id_from_sfid_number(n.sfid_number))
                == Some(institution)
            {
                return Ok(ORG_NRC);
            }
            // 省储行
            if CHINA_CH
                .iter()
                .filter_map(|n| subject_id_from_sfid_number(n.sfid_number))
                .any(|pid| pid == institution)
            {
                return Ok(ORG_PRB);
            }
            Err(Error::<T>::InvalidInstitution)
        }

        /// 解析机构手续费账户。
        fn resolve_fee_account(institution: SubjectId) -> Result<T::AccountId, DispatchError> {
            // 国储会：使用常量地址
            if CHINA_CB
                .first()
                .and_then(|n| subject_id_from_sfid_number(n.sfid_number))
                == Some(institution)
            {
                return T::AccountId::decode(&mut &CHINA_CB[0].fee_address[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into());
            }
            // 省储行：使用 fee_address（BLAKE2-256 派生）
            let node = CHINA_CH
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &node.fee_address[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        /// 解析机构主账户。
        fn resolve_main_account(institution: SubjectId) -> Result<T::AccountId, DispatchError> {
            let raw = subject_pallet_address(institution).ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &raw[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        pub(crate) fn try_execute_sweep_from_callback(
            proposal_id: u64,
            _callback_context: bool,
        ) -> DispatchResult {
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SweepProposalNotPassed
            );

            let fee_account = Self::resolve_fee_account(action.institution)?;
            let main_account = Self::resolve_main_account(action.institution)?;

            ensure!(
                <T as organization_manage::Config>::InstitutionAsset::can_spend(
                    &fee_account,
                    InstitutionAssetAction::OffchainFeeSweepExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费 ──
            let amount_u128: u128 = action.amount.saturated_into();
            let tx_fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let tx_fee: BalanceOf<T> = tx_fee_u128.saturated_into();

            let fee_balance_u128: u128 =
                <T as organization_manage::Config>::Currency::free_balance(&fee_account)
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
            <T as organization_manage::Config>::Currency::transfer(
                &fee_account,
                &main_account,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // ── 手续费：从费用账户扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as organization_manage::Config>::Currency::withdraw(
                &fee_account,
                tx_fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientFeeReserve)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            let reserve_left =
                <T as organization_manage::Config>::Currency::free_balance(&fee_account);

            Self::deposit_event(Event::SweepToMainExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
                fee: tx_fee,
                reserve_left,
            });
            Ok(())
        }

        pub(crate) fn try_execute_safety_fund_from_callback(
            proposal_id: u64,
            _callback_context: bool,
        ) -> DispatchResult {
            let action = SafetyFundProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;

            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SafetyFundProposalNotPassed
            );

            let safety_fund_account = T::AccountId::decode(&mut &NRC_ANQUAN_ADDRESS[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            ensure!(
                <T as organization_manage::Config>::InstitutionAsset::can_spend(
                    &safety_fund_account,
                    InstitutionAssetAction::NrcSafetyFundTransfer,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费 ──
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = action
                .amount
                .checked_add(&fee)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 余额检查：amount + fee + ED ──
            let free =
                <T as organization_manage::Config>::Currency::free_balance(&safety_fund_account);
            let ed = <T as organization_manage::Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            // ── 执行转账 ──
            <T as organization_manage::Config>::Currency::transfer(
                &safety_fund_account,
                &action.beneficiary,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 手续费：从安全基金扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as organization_manage::Config>::Currency::withdraw(
                &safety_fund_account,
                fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            Self::deposit_event(Event::SafetyFundTransferExecuted {
                proposal_id,
                beneficiary: action.beneficiary,
                amount: action.amount,
                fee,
            });

            Ok(())
        }

        pub(crate) fn try_execute_transfer_from_callback(
            proposal_id: u64,
            _callback_context: bool,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
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
                <T as organization_manage::Config>::InstitutionAsset::can_spend(
                    &institution_account,
                    InstitutionAssetAction::DuoqianTransferExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费（复用 onchain-transaction 公共接口） ──
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = action
                .amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;

            // ── 余额检查：需要 total + ED ──
            let free =
                <T as organization_manage::Config>::Currency::free_balance(&institution_account);
            let ed = <T as organization_manage::Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // ── 原子执行：手续费扣取 + 转账，任一失败整体回滚 ──
            let exec_result = frame_support::storage::with_transaction(|| {
                // 先扣手续费
                let fee_imbalance = match <T as organization_manage::Config>::Currency::withdraw(
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
                match <T as organization_manage::Config>::Currency::transfer(
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

// ──── 投票终态回调:把已通过的 3 组业务提案(转账/安全基金/手续费划转)落地到链上 ────
//
// 统一状态机整改后业务模块不再持有独立 vote/finalize call,提案通过(或否决)
// 由投票引擎通过 [`votingengine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `MODULE_TAG` 前缀 + 独立存储键认领对应业务:
// - `MODULE_TAG` 前缀 `dq-xfer` → transfer
// - `SafetyFundProposalActions[id]` 存在 → safety_fund
// - `SweepProposalActions[id]` 存在 → sweep
//
// 失败语义:执行失败发 ExecutionFailed 事件,提案保留 PASSED 状态,快照管理员
// 可通过 VotingEngine::retry_passed_proposal 手动重试,实际权限由 votingengine 统一校验。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        let is_transfer = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
            .map(|raw| raw.starts_with(crate::MODULE_TAG))
            .unwrap_or(false);
        let is_safety_fund = SafetyFundProposalActions::<T>::contains_key(proposal_id);
        let is_sweep = SweepProposalActions::<T>::contains_key(proposal_id);

        if !is_transfer && !is_safety_fund && !is_sweep {
            return Ok(ProposalExecutionOutcome::Ignored); // 非本模块提案
        }

        if approved {
            let exec_result = if is_transfer {
                pallet::Pallet::<T>::try_execute_transfer_from_callback(proposal_id, true)
            } else if is_safety_fund {
                pallet::Pallet::<T>::try_execute_safety_fund_from_callback(proposal_id, true)
            } else {
                pallet::Pallet::<T>::try_execute_sweep_from_callback(proposal_id, true)
            };
            if let Err(_e) = exec_result {
                // 执行失败:发事件,提案保留 PASSED,供 VotingEngine 统一重试入口处理。
                if is_transfer {
                    if let Some(raw) = votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
                        if let Ok(action) =
                            TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                                &mut &raw[crate::MODULE_TAG.len()..],
                            )
                        {
                            pallet::Pallet::<T>::deposit_event(
                                pallet::Event::<T>::TransferExecutionFailed {
                                    proposal_id,
                                    institution: action.institution,
                                },
                            );
                        }
                    }
                } else if is_safety_fund {
                    pallet::Pallet::<T>::deposit_event(
                        pallet::Event::<T>::SafetyFundExecutionFailed { proposal_id },
                    );
                } else if is_sweep {
                    pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::SweepExecutionFailed {
                        proposal_id,
                    });
                }
                return Ok(ProposalExecutionOutcome::RetryableFailed);
            }
            return Ok(ProposalExecutionOutcome::Executed);
        } else {
            // 否决:清理独立存储,避免僵尸数据。
            SafetyFundProposalActions::<T>::remove(proposal_id);
            SweepProposalActions::<T>::remove(proposal_id);
        }
        Ok(ProposalExecutionOutcome::Executed)
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        // 中文注释：普通转账仅依赖 ProposalData；安全基金和 sweep 还有独立动作存储，需要终态清理。
        SafetyFundProposalActions::<T>::remove(proposal_id);
        SweepProposalActions::<T>::remove(proposal_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
