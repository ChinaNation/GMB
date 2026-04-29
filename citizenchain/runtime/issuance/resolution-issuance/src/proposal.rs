//! 决议发行提案与联合投票回调逻辑。

use crate::pallet::{
    AllocationOf, BalanceOf, Config, Error, Event, FinalizeOutcome, Pallet, ReasonOf,
    SnapshotNonceOf, SnapshotSignatureOf, VotingProposalCount,
};
use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::Zero;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;
use voting_engine::JointVoteEngine;

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
/// 中文注释：单条收款分配项，包含收款账户和分配金额。
pub struct RecipientAmount<AccountId, Balance> {
    pub recipient: AccountId,
    pub amount: Balance,
}

/// 存入 voting-engine ProposalData 的业务数据结构。
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct IssuanceProposalData<AccountId, Balance> {
    pub proposer: AccountId,
    pub reason: Vec<u8>,
    pub total_amount: Balance,
    pub allocations: Vec<RecipientAmount<AccountId, Balance>>,
}

impl<T: Config> Pallet<T> {
    pub(crate) fn create_resolution_issuance_proposal(
        proposer: T::AccountId,
        reason: ReasonOf<T>,
        total_amount: BalanceOf<T>,
        allocations: AllocationOf<T>,
        eligible_total: u64,
        snapshot_nonce: SnapshotNonceOf<T>,
        signature: SnapshotSignatureOf<T>,
    ) -> DispatchResult {
        ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
        Self::validate_proposal_allocations(&total_amount, allocations.as_slice())?;

        // 中文注释：联合投票提案创建、业务数据写入和计数递增必须原子提交；
        // 任一步失败都不能留下孤儿提案或错误的 VotingProposalCount。
        with_transaction(|| {
            let proposal_id = match T::JointVoteEngine::create_joint_proposal(
                proposer.clone(),
                eligible_total,
                snapshot_nonce.as_slice(),
                signature.as_slice(),
            ) {
                Ok(id) => id,
                Err(_) => {
                    return TransactionOutcome::Rollback(Err(
                        Error::<T>::JointVoteCreateFailed.into()
                    ))
                }
            };

            let data = IssuanceProposalData {
                proposer: proposer.clone(),
                reason: reason.to_vec(),
                total_amount: total_amount.clone(),
                allocations: allocations.to_vec(),
            };
            let mut encoded = Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&data.encode());
            if voting_engine::Pallet::<T>::store_proposal_data(proposal_id, encoded).is_err() {
                return TransactionOutcome::Rollback(Err(
                    Error::<T>::ProposalDataStoreFailed.into()
                ));
            }
            let now = frame_system::Pallet::<T>::block_number();
            voting_engine::Pallet::<T>::store_proposal_meta(proposal_id, now);

            if let Err(err) = Self::increment_voting_proposal_count() {
                return TransactionOutcome::Rollback(Err(err));
            }

            Self::deposit_event(Event::<T>::ResolutionIssuanceProposed {
                proposal_id,
                proposer,
                total_amount,
                allocation_count: allocations.len() as u32,
            });
            TransactionOutcome::Commit(Ok(()))
        })
    }

    /// 从投票引擎 ProposalData 中读取并解码本模块的业务数据。
    pub fn load_proposal_data(
        proposal_id: u64,
    ) -> Option<IssuanceProposalData<T::AccountId, BalanceOf<T>>> {
        let raw = voting_engine::Pallet::<T>::get_proposal_data(proposal_id)?;
        Self::decode_tagged_data(&raw)
    }

    /// 判断指定提案是否属于本模块。
    pub fn owns_proposal(proposal_id: u64) -> bool {
        voting_engine::Pallet::<T>::get_proposal_data(proposal_id)
            .map(|raw| raw.starts_with(crate::MODULE_TAG))
            .unwrap_or(false)
    }

    fn decode_tagged_data(raw: &[u8]) -> Option<IssuanceProposalData<T::AccountId, BalanceOf<T>>> {
        let tag = crate::MODULE_TAG;
        if raw.len() < tag.len() || &raw[..tag.len()] != tag {
            return None;
        }
        IssuanceProposalData::decode(&mut &raw[tag.len()..]).ok()
    }

    pub(crate) fn apply_joint_vote_result(
        proposal_id: u64,
        approved: bool,
    ) -> Result<FinalizeOutcome, DispatchError> {
        // 中文注释：联合投票终结、发行执行、提案清理和计数递减必须在同一事务里提交。
        with_transaction(|| {
            let data = match Self::load_proposal_data(proposal_id) {
                Some(data) => data,
                None => {
                    return TransactionOutcome::Rollback(Err(Error::<T>::ProposalNotFound.into()))
                }
            };

            if approved {
                let execute_reason: ReasonOf<T> = match data.reason.clone().try_into() {
                    Ok(v) => v,
                    Err(_) => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::ProposalNotFound.into()
                        ))
                    }
                };
                let execute_allocations: AllocationOf<T> = match data.allocations.clone().try_into()
                {
                    Ok(v) => v,
                    Err(_) => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::InvalidAllocationCount.into(),
                        ))
                    }
                };

                if Self::execute_approved_issuance(
                    proposal_id,
                    &execute_reason,
                    data.total_amount.clone(),
                    &execute_allocations,
                )
                .is_ok()
                {
                    T::JointVoteEngine::cleanup_joint_proposal(proposal_id);
                    if let Err(err) = Self::decrement_voting_proposal_count() {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                    Self::deposit_event(Event::<T>::JointVoteFinalized {
                        proposal_id,
                        approved: true,
                    });
                    Self::deposit_event(Event::<T>::IssuanceExecutionTriggered {
                        proposal_id,
                        total_amount: data.total_amount,
                    });
                    return TransactionOutcome::Commit(Ok(
                        FinalizeOutcome::ApprovedExecutionSucceeded,
                    ));
                }

                // 中文注释：执行失败不保留重试分支；统一清理提案并交由回调覆盖为失败终态。
                T::JointVoteEngine::cleanup_joint_proposal(proposal_id);
                if let Err(err) = Self::decrement_voting_proposal_count() {
                    return TransactionOutcome::Rollback(Err(err));
                }
                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: true,
                });
                Self::deposit_event(Event::<T>::IssuanceExecutionFailed { proposal_id });
                return TransactionOutcome::Commit(Ok(FinalizeOutcome::ApprovedExecutionFailed));
            }

            T::JointVoteEngine::cleanup_joint_proposal(proposal_id);
            if let Err(err) = Self::decrement_voting_proposal_count() {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::deposit_event(Event::<T>::JointVoteFinalized {
                proposal_id,
                approved: false,
            });
            TransactionOutcome::Commit(Ok(FinalizeOutcome::Rejected))
        })
    }

    pub(crate) fn increment_voting_proposal_count() -> DispatchResult {
        VotingProposalCount::<T>::try_mutate(|count| -> DispatchResult {
            *count = count
                .checked_add(1)
                .ok_or(Error::<T>::VotingProposalCountOverflow)?;
            Ok(())
        })
    }

    pub(crate) fn decrement_voting_proposal_count() -> DispatchResult {
        VotingProposalCount::<T>::try_mutate(|count| -> DispatchResult {
            *count = count
                .checked_sub(1)
                .ok_or(Error::<T>::VotingProposalCountUnderflow)?;
            Ok(())
        })
    }

    pub(crate) fn set_allowed_recipients_inner(
        recipients: BoundedVec<T::AccountId, T::MaxAllocations>,
    ) -> DispatchResult {
        ensure!(!recipients.is_empty(), Error::<T>::RecipientsNotConfigured);
        // 中文注释：存在 Voting 中提案时禁止切换收款集合，避免同一提案投票前后口径漂移。
        ensure!(
            VotingProposalCount::<T>::get() == 0,
            Error::<T>::ActiveVotingProposalsExist
        );
        Self::ensure_unique_recipients(recipients.as_slice())?;
        Self::ensure_recipients_only_added(&recipients)?;
        Self::ensure_recipients_in_china_cb(&recipients)?;
        crate::pallet::AllowedRecipients::<T>::put(recipients.clone());
        Self::deposit_event(Event::<T>::AllowedRecipientsUpdated {
            count: recipients.len() as u32,
        });
        Ok(())
    }

    pub(crate) fn ensure_nonzero_total(total_amount: &BalanceOf<T>) -> DispatchResult {
        ensure!(!total_amount.is_zero(), Error::<T>::ZeroAmount);
        Ok(())
    }
}
