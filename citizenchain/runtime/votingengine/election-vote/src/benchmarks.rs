//! `election-vote` FRAME benchmark。
//!
//! 两个用例都构造“最后一票”路径，并按候选人数 `c` 读取全部候选计票、生成结果，
//! 覆盖普通写票之外最重的终结分支。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

use crate::{
    pallet::{
        Config, ElectionCandidates, ElectionMetaStore, ElectionResults, ElectionTallyStore,
        ElectionVoters, Pallet,
    },
    types::{ElectionMeta, ElectionMode, ElectionTally},
    Call,
};

fn setup_election<T: Config>(c: u32, mode: ElectionMode) -> (u64, T::AccountId, T::AccountId) {
    let proposal_id = 0u64;
    let voter: T::AccountId = account("voter", 0, 0);
    let organizer: T::AccountId = account("organizer", 0, 0);
    let target: T::AccountId = account("target", 0, 0);
    let candidates: sp_std::vec::Vec<T::AccountId> =
        (0..c).map(|index| account("candidate", index, 0)).collect();
    let selected = candidates[0].clone();
    let bounded: frame_support::BoundedVec<T::AccountId, T::MaxElectionCandidates> = candidates
        .try_into()
        .expect("runtime candidate bound covers benchmark range");
    let now = 1u32.saturated_into();
    frame_system::Pallet::<T>::set_block_number(now);
    votingengine::pallet::Proposals::<T>::insert(
        proposal_id,
        votingengine::Proposal {
            kind: votingengine::PROPOSAL_KIND_ELECTION,
            stage: mode.stage(),
            status: votingengine::STATUS_VOTING,
            internal_code: None,
            account_context: Some(target.clone()),
            subject_cid_numbers: Default::default(),
            start: now,
            end: 2u32.saturated_into(),
            citizen_eligible_total: 1,
        },
    );
    ElectionMetaStore::<T>::insert(
        proposal_id,
        ElectionMeta {
            mode,
            population_scope: (mode == ElectionMode::Popular)
                .then_some(votingengine::PopulationScope::Country),
            organizer_code: [0; 4],
            organizer,
            target_code: [0; 4],
            target,
            office_code: b"benchmark"
                .to_vec()
                .try_into()
                .expect("bounded office code"),
            rule_id: 0,
            seat_count: 1,
            term_start: 0,
            term_end: 1,
        },
    );
    ElectionCandidates::<T>::insert(proposal_id, bounded);
    ElectionVoters::<T>::insert(proposal_id, &voter, ());
    ElectionTallyStore::<T>::insert(proposal_id, ElectionTally::default());
    (proposal_id, voter, selected)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn cast_popular_vote(c: Linear<1, 256>) {
        let (proposal_id, voter, candidate) = setup_election::<T>(c, ElectionMode::Popular);

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), proposal_id, candidate);

        assert!(ElectionResults::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn cast_mutual_vote(c: Linear<1, 256>) {
        let (proposal_id, voter, candidate) = setup_election::<T>(c, ElectionMode::Mutual);

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), proposal_id, candidate);

        assert!(ElectionResults::<T>::contains_key(proposal_id));
    }
}
