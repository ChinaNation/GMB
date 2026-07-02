//! election-vote 存储清理。
//!
//! 核心 votingengine 只维护清理状态机，具体选票、候选、计票账本
//! 住在 election-vote，因此通过 `ElectionCleanupHandler` 派发到这里分块删除。

use crate::pallet::{
    ElectionCandidateTallies, ElectionCandidates, ElectionMetaStore, ElectionResults,
    ElectionTallyStore, ElectionVoters, ElectionVotesByVoter,
};

impl<T: crate::pallet::Config> votingengine::ElectionCleanupHandler for crate::pallet::Pallet<T> {
    fn cleanup_election_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::CleanupChunkResult {
        let result = ElectionVotesByVoter::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_election_voters_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::CleanupChunkResult {
        let result = ElectionVoters::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_election_tallies_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::CleanupChunkResult {
        let result = ElectionCandidateTallies::<T>::clear_prefix(proposal_id, limit, None);
        if result.maybe_cursor.is_none() {
            ElectionTallyStore::<T>::remove(proposal_id);
        }
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_election_terminal(proposal_id: u64) {
        ElectionMetaStore::<T>::remove(proposal_id);
        ElectionCandidates::<T>::remove(proposal_id);
        ElectionResults::<T>::remove(proposal_id);
    }
}
