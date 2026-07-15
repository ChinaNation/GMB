//! 选举快照校验与写入。
//!
//! 投票期间只读取创建时固化的候选人/选民快照，
//! 不实时回读业务模块状态，避免换届、管理员变更或 CID 数据更新影响已发起选举。

use frame_support::{ensure, pallet_prelude::*};
use sp_runtime::DispatchError;

use crate::pallet::{
    Config, ElectionCandidates, Error, MaxElectionCandidatesOf, MaxMutualVotersOf, MutualVoters,
    Pallet,
};

impl<T: Config> Pallet<T> {
    pub(crate) fn ensure_unique_accounts(accounts: &[T::AccountId]) -> DispatchResult {
        for i in 0..accounts.len() {
            for j in i.saturating_add(1)..accounts.len() {
                ensure!(accounts[i] != accounts[j], Error::<T>::DuplicateAccount);
            }
        }
        Ok(())
    }

    pub(crate) fn bounded_candidates(
        candidates: sp_std::vec::Vec<T::AccountId>,
    ) -> Result<BoundedVec<T::AccountId, MaxElectionCandidatesOf<T>>, DispatchError> {
        ensure!(!candidates.is_empty(), Error::<T>::EmptyCandidateSnapshot);
        Self::ensure_unique_accounts(&candidates)?;
        candidates
            .try_into()
            .map_err(|_| Error::<T>::TooManyCandidates.into())
    }

    pub(crate) fn bounded_mutual_voters(
        voters: sp_std::vec::Vec<T::AccountId>,
    ) -> Result<BoundedVec<T::AccountId, MaxMutualVotersOf<T>>, DispatchError> {
        ensure!(!voters.is_empty(), Error::<T>::EmptyVoterSnapshot);
        Self::ensure_unique_accounts(&voters)?;
        voters
            .try_into()
            .map_err(|_| Error::<T>::TooManyVoters.into())
    }

    pub(crate) fn write_mutual_voter_snapshot(
        proposal_id: u64,
        voters: &BoundedVec<T::AccountId, MaxMutualVotersOf<T>>,
    ) {
        for voter in voters.iter() {
            MutualVoters::<T>::insert(proposal_id, voter, ());
        }
    }

    pub(crate) fn candidate_exists(proposal_id: u64, candidate: &T::AccountId) -> bool {
        ElectionCandidates::<T>::get(proposal_id)
            .map(|items| items.iter().any(|item| item == candidate))
            .unwrap_or(false)
    }

    pub(crate) fn mutual_voter_exists(proposal_id: u64, voter: &T::AccountId) -> bool {
        MutualVoters::<T>::contains_key(proposal_id, voter)
    }
}
