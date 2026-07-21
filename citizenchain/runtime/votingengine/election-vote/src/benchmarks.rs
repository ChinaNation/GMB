//! `election-vote` FRAME benchmark。
//!
//! 两个用例都构造“最后一票”路径，并按候选人数 `c` 读取全部候选计票、生成结果，
//! 覆盖普通写票之外最重的终结分支。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use votingengine::CitizenIdentityReader;

use crate::{
    pallet::{
        Config, ElectionCandidates, ElectionMetaStore, ElectionResults, ElectionTallyStore, Pallet,
    },
    types::{ElectionMeta, ElectionMode, ElectionTally},
    Call,
};

fn setup_election<T: Config>(c: u32, mode: ElectionMode) -> (u64, T::AccountId, T::AccountId) {
    let proposal_id = 0u64;
    let voter: T::AccountId = account("voter", 0, 0);
    let actor_cid_number: votingengine::CidNumber = primitives::cid::china::china_cb::CHINA_CB[0]
        .cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("NRC CID fits runtime bound");
    let target_cid_number: votingengine::CidNumber = primitives::cid::china::china_lf::CHINA_LF[0]
        .cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("legislature CID fits runtime bound");
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
            actor_cid_number: Some(actor_cid_number.clone()),
            execution_account: None,
            subject_cid_numbers: Default::default(),
            start: now,
            end: 2u32.saturated_into(),
        },
    );
    ElectionMetaStore::<T>::insert(
        proposal_id,
        ElectionMeta {
            mode,
            population_scope: (mode == ElectionMode::Popular)
                .then_some(votingengine::PopulationScope::Country),
            actor_cid_number,
            target_cid_number: target_cid_number.clone(),
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
    if mode == ElectionMode::Popular {
        let scope = votingengine::PopulationScope::Country;
        <T as votingengine::Config>::CitizenIdentityReader::benchmark_seed_identity(&voter, &scope);
        votingengine::Pallet::<T>::create_population_snapshot(proposal_id, &scope)
            .expect("benchmark proposal population snapshot");
    } else {
        let subject =
            votingengine::AuthorizationSubject::Institution(entity_primitives::RoleSubject {
                cid_number: target_cid_number,
                role_code: b"BENCHMARK_MEMBER"
                    .to_vec()
                    .try_into()
                    .expect("benchmark role code"),
            });
        votingengine::Pallet::<T>::snapshot_role_voters(
            proposal_id,
            subject,
            sp_std::vec![voter.clone()],
        )
        .expect("benchmark mutual role snapshot");
    }
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
        _(
            RawOrigin::Signed(voter),
            proposal_id,
            b"BENCHMARK_MEMBER"
                .to_vec()
                .try_into()
                .expect("benchmark role code"),
            candidate,
        );

        assert!(ElectionResults::<T>::contains_key(proposal_id));
    }
}
