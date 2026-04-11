//! # 机构多签名地址转账模块 (duoqian-transfer-pow)
//!
//! 本模块为治理机构（NRC/PRC/PRB）和注册多签机构提供链上转账治理流程：
//! - 管理员发起转账提案，经内部投票通过后自动执行转账并扣取手续费。
//! - 自动执行失败时保留提案状态，可通过 `execute_transfer` 手动重试。
//! - 余额在提案创建和执行两个时点双重检查，含手续费和 ED 保留。
//! - 收款地址不能是机构自身，也不能是受保护地址（质押地址）。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};


use primitives::china::china_cb::{
    shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB, NRC_ANQUAN_ADDRESS,
};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
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
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SweepAction<Balance> {
    /// 机构标识
    pub institution: InstitutionPalletId,
    /// 划转金额
    pub amount: Balance,
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

/// 中文注释：从 CHINA_CB/CHINA_CH 中查找机构的多签账户地址（duoqian_address）。
fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    if let Some(node) = CHINA_CB
        .iter()
        .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
    {
        return Some(node.duoqian_address);
    }

    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .map(|n| n.duoqian_address)
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
        SweepAction<BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 转账提案已创建
        TransferProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 投票已提交
        TransferVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
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
        /// 安全基金转账提案已创建
        SafetyFundTransferProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 安全基金投票已提交
        SafetyFundVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 安全基金转账已执行
        SafetyFundTransferExecuted {
            proposal_id: u64,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 安全基金投票通过但执行失败
        SafetyFundExecutionFailed {
            proposal_id: u64,
        },
        /// 手续费划转提案已创建
        SweepToMainProposed {
            proposal_id: u64,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 手续费划转投票已提交
        SweepToMainVoteSubmitted {
            proposal_id: u64,
            voter: T::AccountId,
            approve: bool,
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
        SweepExecutionFailed {
            proposal_id: u64,
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
                remark,
                proposer: who.clone(),
            };
            let mut encoded = sp_runtime::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, encoded)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(
                proposal_id,
                frame_system::Pallet::<T>::block_number(),
            );

            Self::deposit_event(Event::<T>::TransferProposed {
                proposal_id,
                org,
                institution,
                proposer: who,
                beneficiary,
                amount,
            });
            Ok(())
        }

        /// 对转账提案投票，达到阈值后自动执行转账。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::vote_transfer())]
        pub fn vote_transfer(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(raw.len() >= tag.len() && &raw[..tag.len()] == tag, Error::<T>::ProposalActionNotFound);
            let action = TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                &mut &raw[tag.len()..],
            )
            .map_err(|_| Error::<T>::ProposalActionNotFound)?;
            let org = Self::resolve_actual_org(action.institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            <T as duoqian_manage_pow::Config>::InternalVoteEngine::cast_internal_vote(
                who.clone(),
                proposal_id,
                approve,
            )?;

            Self::deposit_event(Event::<T>::TransferVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            // 检查投票结果
            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    // 投票通过，尝试自动执行转账
                    let institution = action.institution;
                    if Self::try_execute_transfer(proposal_id).is_err() {
                        Self::deposit_event(Event::<T>::TransferExecutionFailed {
                            proposal_id,
                            institution,
                        });
                    }
                }
            }
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
                    ORG_NRC, nrc_institution, &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            // 验证安全基金账户余额
            let safety_fund_account = T::AccountId::decode(
                &mut &NRC_ANQUAN_ADDRESS[..],
            ).map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
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
            let total = amount.checked_add(&fee).ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            let ed: BalanceOf<T> = <T as duoqian_manage_pow::Config>::Currency::minimum_balance();
            let free = <T as duoqian_manage_pow::Config>::Currency::free_balance(&safety_fund_account);
            let required = total.checked_add(&ed).ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            // 创建内部投票提案
            let proposal_id = <T as duoqian_manage_pow::Config>::InternalVoteEngine
                ::create_internal_proposal(who.clone(), ORG_NRC, nrc_institution)?;

            SafetyFundProposalActions::<T>::insert(
                proposal_id,
                SafetyFundAction {
                    beneficiary: beneficiary.clone(),
                    amount,
                    remark,
                    proposer: who.clone(),
                },
            );

            Self::deposit_event(Event::SafetyFundTransferProposed {
                proposal_id,
                proposer: who,
                beneficiary,
                amount,
            });
            Ok(())
        }

        /// 对国储会安全基金转账提案投票；通过后自动执行。
        #[pallet::call_index(4)]
        #[pallet::weight(T::DbWeight::get().reads_writes(7, 6))]
        pub fn vote_safety_fund_transfer(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let _action = SafetyFundProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;

            // 验证国储会管理员
            let nrc_institution = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                    ORG_NRC, nrc_institution, &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            // 投票
            <T as duoqian_manage_pow::Config>::InternalVoteEngine
                ::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::SafetyFundVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            // 投票通过后自动执行
            if approve {
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
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
                }
            }
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
                    org, institution, &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id = <T as duoqian_manage_pow::Config>::InternalVoteEngine
                ::create_internal_proposal(who.clone(), org, institution)?;

            SweepProposalActions::<T>::insert(
                proposal_id,
                SweepAction { institution, amount },
            );

            Self::deposit_event(Event::SweepToMainProposed {
                proposal_id,
                institution,
                proposer: who,
                amount,
            });
            Ok(())
        }

        /// 对手续费划转提案投票；通过后自动执行划转。
        #[pallet::call_index(6)]
        #[pallet::weight(T::DbWeight::get().reads_writes(7, 6))]
        pub fn vote_sweep_to_main(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            let org = Self::resolve_sweep_org(action.institution)?;
            ensure!(
                <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                    org, action.institution, &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            <T as duoqian_manage_pow::Config>::InternalVoteEngine
                ::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::SweepToMainVoteSubmitted {
                proposal_id,
                voter: who,
                approve,
            });

            if approve {
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
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
                }
            }
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

        fn resolve_actual_org(institution: InstitutionPalletId) -> Option<u8> {
            if let Some(org) = institution_org(institution) {
                return Some(org);
            }
            if Self::registered_duoqian_account(institution).is_ok() {
                return Some(ORG_DUOQIAN);
            }
            None
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
        fn resolve_fee_account(institution: InstitutionPalletId) -> Result<T::AccountId, DispatchError> {
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
        fn resolve_main_account(institution: InstitutionPalletId) -> Result<T::AccountId, DispatchError> {
            let raw = institution_pallet_address(institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
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

            let fee_balance_u128: u128 = <T as duoqian_manage_pow::Config>::Currency::free_balance(&fee_account).saturated_into();
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
            ).map_err(|_| Error::<T>::InsufficientFeeReserve)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            let reserve_left = <T as duoqian_manage_pow::Config>::Currency::free_balance(&fee_account);

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

            let safety_fund_account = T::AccountId::decode(
                &mut &NRC_ANQUAN_ADDRESS[..],
            ).map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

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
            let total = action.amount
                .checked_add(&fee)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 余额检查：amount + fee + ED ──
            let free = <T as duoqian_manage_pow::Config>::Currency::free_balance(&safety_fund_account);
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
            ).map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 手续费：从安全基金扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as duoqian_manage_pow::Config>::Currency::withdraw(
                &safety_fund_account,
                fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ).map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            voting_engine_system::Pallet::<T>::set_status_and_emit(
                proposal_id,
                STATUS_EXECUTED,
            )?;

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
            ensure!(raw.len() >= tag.len() && &raw[..tag.len()] == tag, Error::<T>::ProposalActionNotFound);
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
                    Err(_) => return frame_support::storage::TransactionOutcome::Rollback(
                        Err(Error::<T>::InsufficientBalance.into())
                    ),
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
            duoqian_manage_pow::pallet::SfidNameOf<Test>,
            duoqian_manage_pow::pallet::RegisterNonceOf<Test>,
            duoqian_manage_pow::pallet::RegisterSignatureOf<Test>,
        > for TestSfidInstitutionVerifier
    {
        fn verify_institution_registration(
            _sfid_id: &[u8],
            _name: &duoqian_manage_pow::pallet::SfidNameOf<Test>,
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

    pub struct TestInternalAdminProvider;
    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
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

        fn get_admin_list(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<Vec<AccountId32>> {
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.duoqian_admins.iter().copied().map(AccountId32::new).collect()),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.duoqian_admins.iter().copied().map(AccountId32::new).collect()),
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
        type MaxSfidNameLength = ConstU32<128>;
        type MaxRegisterNonceLength = ConstU32<64>;
        type MaxRegisterSignatureLength = ConstU32<64>;
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

    fn nrc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[0].duoqian_admins[index])
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].duoqian_admins[index])
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].duoqian_admins[index])
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
        match index {
            0 => AccountId32::new([0x31; 32]),
            1 => AccountId32::new([0x32; 32]),
            _ => AccountId32::new([0x33; 32]),
        }
    }

    /// 收款人：使用一个不是管理员也不是机构的普通地址
    fn beneficiary() -> AccountId32 {
        AccountId32::new([99u8; 32])
    }

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine_system::Pallet::<Test>::next_proposal_id().saturating_sub(1)
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

        storage.into()
    }

    #[test]
    fn nrc_transfer_executes_when_yes_votes_reach_threshold() {
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

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 转账已执行（含手续费 10）
            assert_eq!(Balances::free_balance(&inst_account), 8_990);
            assert_eq!(Balances::free_balance(&dest), 1_000);
            // 提案数据仍保留（由 voting-engine-system 延迟清理）
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn prc_transfer_executes_when_yes_votes_reach_threshold() {
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

            for i in 0..6 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(prc_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 7_990);
            assert_eq!(Balances::free_balance(&dest), 2_000);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn prb_transfer_executes_when_yes_votes_reach_threshold() {
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

            for i in 0..6 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(prb_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 6_990);
            assert_eq!(Balances::free_balance(&dest), 3_000);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn registered_duoqian_transfer_executes_when_yes_votes_reach_threshold() {
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

            assert_ok!(DuoqianTransferPow::vote_transfer(
                RuntimeOrigin::signed(registered_duoqian_admin(0)),
                pid,
                true
            ));
            assert_ok!(DuoqianTransferPow::vote_transfer(
                RuntimeOrigin::signed(registered_duoqian_admin(1)),
                pid,
                true
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
    fn non_admin_cannot_propose_or_vote() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
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
                dest,
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // PRC 管理员不能给 NRC 投票
            assert_noop!(
                DuoqianTransferPow::vote_transfer(RuntimeOrigin::signed(prc_admin(0)), pid, true),
                Error::<Test>::UnauthorizedAdmin
            );
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
    fn duplicate_vote_is_rejected() {
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
            assert_ok!(DuoqianTransferPow::vote_transfer(
                RuntimeOrigin::signed(nrc_admin(1)),
                pid,
                true
            ));
            assert_noop!(
                DuoqianTransferPow::vote_transfer(RuntimeOrigin::signed(nrc_admin(1)), pid, true),
                voting_engine_system::pallet::Error::<Test>::AlreadyVoted
            );
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

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid1,
                    true
                ));
            }

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

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

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

            // 余额 10_000，提案 9_990（required=9_990+10+1=10_001>10_000）
            // propose 预检也含手续费，所以 9_990 会被拒绝
            // 先用 9_989 创建提案（刚好通过预检），然后手动减余额使执行失败
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 在投票前减少余额，使自动执行失败
            // 转走 9_000 使余额仅剩 1_000，不够 amount(9_000)+fee(10)+ED(1)=9_011
            let drain_dest = AccountId32::new([88u8; 32]);
            let _ = Balances::deposit_creating(&drain_dest, 1);
            assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
                &inst_account,
                &drain_dest,
                9_000,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ));
            assert_eq!(Balances::free_balance(&inst_account), 1_000);

            // 投票通过，自动执行因余额不足失败
            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 提案状态为 PASSED，但转账未执行
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(Balances::free_balance(&dest), 0);
            // 提案数据仍保留
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

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

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
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                1_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

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

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 余额 10_000 - 1(转账) - 10(最低手续费) = 9_989
            assert_eq!(Balances::free_balance(&inst_account), 9_989);
            assert_eq!(Balances::free_balance(&dest), 1);
        });
    }
}
