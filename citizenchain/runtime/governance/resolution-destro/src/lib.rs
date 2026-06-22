#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, Zero};

use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use votingengine::{
    types::{InstitutionCode, NRC, PRB, PRC},
    InternalVoteResultCallback, ProposalExecutionOutcome, STATUS_PASSED,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"res-dst";

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DestroyAction<AccountId, Balance> {
    /// 目标机构主账户。
    pub institution: AccountId,
    /// 销毁数量
    pub amount: Balance,
}

fn decode_account<T: frame_system::Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

fn nrc_account<T: frame_system::Config>() -> Option<T::AccountId> {
    CHINA_CB
        .first()
        .and_then(|n| decode_account::<T>(&n.main_account))
}

fn account_org<T: frame_system::Config>(institution: T::AccountId) -> Option<InstitutionCode> {
    if Some(institution.clone()) == nrc_account::<T>() {
        return Some(NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| decode_account::<T>(&n.main_account))
        .any(|pid| pid == institution)
    {
        return Some(PRC);
    }

    if CHINA_CH
        .iter()
        .filter_map(|n| decode_account::<T>(&n.main_account))
        .any(|pid| pid == institution)
    {
        return Some(PRB);
    }

    None
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use votingengine::InternalAdminProvider;
    use votingengine::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        /// 中文注释：通过统一内部投票引擎创建提案，返回真实 proposal_id。
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        /// 该 pallet 的可配置权重实现。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 提案数据、元数据、活跃提案列表均已移至 votingengine 统一管控。

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起销毁提案（并已在投票引擎创建内部提案）
        DestroyProposed {
            proposal_id: u64,
            institution_code: InstitutionCode,
            institution: T::AccountId,
            proposer: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 提交销毁投票
        DestroyVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        DestroyExecutionFailed { proposal_id: u64 },
        /// 销毁执行完成
        DestroyExecuted {
            proposal_id: u64,
            institution: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InstitutionOrgMismatch,
        UnauthorizedAdmin,
        ZeroAmount,
        ProposalActionNotFound,
        ProposalNotPassed,
        InstitutionAccountDecodeFailed,
        InsufficientBalance,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起“决议销毁”内部投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_destroy())]
        pub fn propose_destroy(
            origin: OriginFor<T>,
            institution_code: InstitutionCode,
            institution: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
            let actual_org =
                account_org::<T>(institution.clone()).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                actual_org == institution_code,
                Error::<T>::InstitutionOrgMismatch
            );
            // 活跃提案数由 votingengine 在 create_internal_proposal 中统一检查
            ensure!(
                Self::is_internal_admin(institution_code, institution.clone(), &who),
                Error::<T>::UnauthorizedAdmin
            );

            let action = DestroyAction {
                institution: institution.clone(),
                amount,
            };
            let mut encoded = Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            let proposal_id = T::InternalVoteEngine::create_general_internal_proposal_with_data(
                who.clone(),
                institution_code,
                institution.clone(),
                crate::MODULE_TAG,
                encoded,
            )?;

            Self::deposit_event(Event::<T>::DestroyProposed {
                proposal_id,
                institution_code,
                institution,
                proposer: who,
                amount,
            });
            Ok(())
        }

        // call_index = 1 永久保留空位,不复用。
    }

    impl<T: Config> Pallet<T> {
        fn is_internal_admin(
            institution_code: InstitutionCode,
            institution: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                institution_code,
                institution,
                who,
            )
        }

        pub(crate) fn try_execute_destroy_from_action(
            proposal_id: u64,
            action: DestroyAction<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            // 中文注释：PASSED 是可执行/可重试态；终态进入后不允许再执行。
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let free = T::Currency::free_balance(&action.institution);
            let ed = T::Currency::minimum_balance();
            // 中文注释：销毁前必须预留 ED，确保机构账户不会因一次销毁被直接 reap。
            let required = action
                .amount
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // 中文注释：slash 会同步减少总发行量，实现链上”销毁”。
            let (_negative_imbalance, remaining) =
                T::Currency::slash(&action.institution, action.amount);
            ensure!(remaining.is_zero(), Error::<T>::InsufficientBalance);

            Self::deposit_event(Event::<T>::DestroyExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
            });
            Ok(())
        }
    }
}

// ──── 投票终态回调:把已通过的销毁提案落地到链上 ────
//
// 投票统一由投票引擎承担,提案通过(或否决)经
// [`votingengine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `MODULE_TAG` 前缀认领本模块的提案,非己方返回 Ignored。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        let raw = match votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(ProposalExecutionOutcome::Ignored),
        };
        if !approved {
            return Ok(ProposalExecutionOutcome::Executed);
        }
        let action = DestroyAction::<T::AccountId, BalanceOf<T>>::decode(
            &mut &raw[crate::MODULE_TAG.len()..],
        )
        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;

        match pallet::Pallet::<T>::try_execute_destroy_from_action(proposal_id, action) {
            Ok(()) => Ok(ProposalExecutionOutcome::Executed),
            Err(_) => {
                pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::DestroyExecutionFailed {
                    proposal_id,
                });
                Ok(ProposalExecutionOutcome::RetryableFailed)
            }
        }
    }
}

#[cfg(test)]
mod tests;
