//! 多候选、多席位计票。
//!
//! 中文注释：当前只实现“得票多数当选”的最小可用规则。若最后一个席位出现同票，
//! 本 pallet 先拒绝本次结果，后续由《选举法》接入同票处理规则后再扩展。

use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;

use crate::pallet::{
    Config, ElectionCandidateTallies, ElectionCandidates, ElectionResults, Error,
    MaxElectionCandidatesOf, Pallet,
};
use crate::types::ElectionWinner;

/// 纯函数：从候选人顺序快照和票数中选出席位。
pub(crate) fn select_winners<AccountId: Clone + PartialEq>(
    candidates: &[(AccountId, u32)],
    seat_count: u16,
) -> Result<Vec<ElectionWinner<AccountId>>, ()> {
    let mut remaining: Vec<(AccountId, u32)> = candidates.to_vec();
    let mut winners: Vec<ElectionWinner<AccountId>> = Vec::new();

    for seat_index in 0..seat_count {
        let Some(max_votes) = remaining.iter().map(|(_, votes)| *votes).max() else {
            return Err(());
        };
        if max_votes == 0 {
            return Err(());
        }

        let tied: Vec<usize> = remaining
            .iter()
            .enumerate()
            .filter_map(|(idx, (_, votes))| (*votes == max_votes).then_some(idx))
            .collect();
        if tied.len() != 1 {
            return Err(());
        }

        let idx = tied[0];
        let (account, votes) = remaining.remove(idx);
        winners.push(ElectionWinner {
            account,
            votes,
            seat_index,
        });
    }

    Ok(winners)
}

impl<T: Config> Pallet<T> {
    pub(crate) fn finalize_election_result(proposal_id: u64) -> DispatchResult {
        let meta = crate::pallet::ElectionMetaStore::<T>::get(proposal_id)
            .ok_or(Error::<T>::ElectionMetaMissing)?;
        let candidates =
            ElectionCandidates::<T>::get(proposal_id).ok_or(Error::<T>::EmptyCandidateSnapshot)?;

        let candidate_votes: Vec<(T::AccountId, u32)> = candidates
            .iter()
            .cloned()
            .map(|candidate| {
                let votes = ElectionCandidateTallies::<T>::get(proposal_id, &candidate);
                (candidate, votes)
            })
            .collect();

        let winners = match select_winners(&candidate_votes, meta.seat_count) {
            Ok(winners) => winners,
            Err(()) => {
                Self::deposit_event(crate::pallet::Event::<T>::ElectionRejectedByTieOrNoVotes {
                    proposal_id,
                });
                return votingengine::Pallet::<T>::set_status_and_emit(
                    proposal_id,
                    votingengine::STATUS_REJECTED,
                );
            }
        };

        let bounded: BoundedVec<ElectionWinner<T::AccountId>, MaxElectionCandidatesOf<T>> = winners
            .try_into()
            .map_err(|_| Error::<T>::TooManyCandidates)?;
        ElectionResults::<T>::insert(proposal_id, bounded);
        Self::deposit_event(crate::pallet::Event::<T>::ElectionResultReady { proposal_id });
        votingengine::Pallet::<T>::set_status_and_emit(proposal_id, votingengine::STATUS_PASSED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_top_seats_without_tie() {
        let winners = select_winners(&[(1u8, 9), (2, 5), (3, 7)], 2).expect("unique winners");
        assert_eq!(winners[0].account, 1);
        assert_eq!(winners[1].account, 3);
    }

    #[test]
    fn rejects_tie_for_open_seat() {
        assert!(select_winners(&[(1u8, 9), (2, 7), (3, 7)], 2).is_err());
    }
}
