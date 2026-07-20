//! 选举快照校验与写入。
//!
//! 投票期间只读取创建时固化的候选人/选民快照，
//! 不实时回读业务模块状态，避免换届、管理员变更或 CID 数据更新影响已发起选举。

use frame_support::{ensure, pallet_prelude::*};
use sp_runtime::DispatchError;

use crate::pallet::{Config, ElectionCandidates, Error, MaxElectionCandidatesOf, Pallet};

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

    pub(crate) fn candidate_exists(proposal_id: u64, candidate: &T::AccountId) -> bool {
        ElectionCandidates::<T>::get(proposal_id)
            .map(|items| items.iter().any(|item| item == candidate))
            .unwrap_or(false)
    }
}
