//! `joint-vote` FRAME benchmark。
//!
//! 覆盖机构形成最终票、公民公投写票及两个超时阶段；人口快照随提案创建内联生成。
#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use votingengine::CitizenIdentityReader;

use crate::pallet::{
    Config, JointInstitutionTallies, JointVotesByInstitution, Pallet, ReferendumVotesByAccount,
};
use crate::Call;

fn decode<T: frame_system::Config>(raw: &[u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("fixed governance account decodes")
}

fn setup_proposal<T: Config>(
    stage: u8,
) -> (
    u64,
    votingengine::Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
) {
    let proposal_id = 0u64;
    let nrc = &primitives::cid::china::china_cb::CHINA_CB[0];
    let now = 1u32.saturated_into();
    frame_system::Pallet::<T>::set_block_number(now);
    let proposal = votingengine::Proposal {
        kind: votingengine::PROPOSAL_KIND_JOINT,
        stage,
        status: votingengine::STATUS_VOTING,
        internal_code: None,
        actor_cid_number: Some(
            nrc.cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("NRC CID fits"),
        ),
        execution_account: None,
        subject_cid_numbers: Default::default(),
        start: 0u32.saturated_into(),
        end: 2u32.saturated_into(),
    };
    votingengine::pallet::Proposals::<T>::insert(proposal_id, proposal.clone());
    (proposal_id, proposal)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn cast_admin() {
        let (proposal_id, _) = setup_proposal::<T>(votingengine::STAGE_JOINT);
        let entry = &primitives::cid::china::china_cb::CHINA_CB[0];
        let actor_cid_number: votingengine::CidNumber = entry
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NRC CID fits runtime bound");
        let voter = decode::<T>(&entry.admins[0]);
        let voters: frame_support::BoundedVec<_, T::MaxAdminsPerInstitution> = entry
            .admins
            .iter()
            .map(decode::<T>)
            .collect::<sp_std::vec::Vec<_>>()
            .try_into()
            .expect("fixed NRC admins fit runtime bound");
        votingengine::pallet::EffectiveVoterSnapshot::<T>::insert(
            proposal_id,
            votingengine::ProposalSubject::InstitutionCid(actor_cid_number.clone()),
            voters,
        );
        let threshold = votingengine::fixed_governance_pass_threshold(&votingengine::NRC)
            .expect("NRC threshold");
        JointInstitutionTallies::<T>::insert(
            proposal_id,
            &actor_cid_number,
            votingengine::VoteCountU32 {
                yes: threshold.saturating_sub(1),
                no: 0,
            },
        );

        #[extrinsic_call]
        _(
            RawOrigin::Signed(voter),
            proposal_id,
            actor_cid_number.clone(),
            true,
        );

        assert!(JointVotesByInstitution::<T>::contains_key(
            proposal_id,
            actor_cid_number
        ));
    }

    #[benchmark]
    fn cast_referendum() {
        let (proposal_id, _) = setup_proposal::<T>(votingengine::STAGE_REFERENDUM);
        let voter: T::AccountId = account("citizen", 0, 0);
        let scope = votingengine::PopulationScope::Country;
        <T as votingengine::Config>::CitizenIdentityReader::benchmark_seed_identity(&voter, &scope);
        votingengine::Pallet::<T>::create_population_snapshot(proposal_id, &scope)
            .expect("benchmark proposal should create population snapshot");

        #[extrinsic_call]
        _(RawOrigin::Signed(voter.clone()), proposal_id, true);

        assert!(ReferendumVotesByAccount::<T>::contains_key(
            proposal_id,
            voter
        ));
    }

    #[benchmark]
    fn finalize_joint_timeout() {
        let (proposal_id, proposal) = setup_proposal::<T>(votingengine::STAGE_JOINT);
        votingengine::Pallet::<T>::create_population_snapshot(
            proposal_id,
            &votingengine::PopulationScope::Country,
        )
        .expect("joint proposal population snapshot");
        frame_system::Pallet::<T>::set_block_number(3u32.saturated_into());

        #[block]
        {
            Pallet::<T>::do_finalize_joint_timeout(&proposal, proposal_id)
                .expect("expired joint stage advances");
        }

        assert_eq!(
            votingengine::pallet::Proposals::<T>::get(proposal_id).map(|item| item.stage),
            Some(votingengine::STAGE_REFERENDUM)
        );
    }

    #[benchmark]
    fn finalize_jointreferendum_timeout() {
        let (proposal_id, proposal) = setup_proposal::<T>(votingengine::STAGE_REFERENDUM);
        votingengine::Pallet::<T>::create_population_snapshot(
            proposal_id,
            &votingengine::PopulationScope::Country,
        )
        .expect("joint proposal population snapshot");
        frame_system::Pallet::<T>::set_block_number(3u32.saturated_into());

        #[block]
        {
            Pallet::<T>::do_finalize_jointreferendum_timeout(&proposal, proposal_id)
                .expect("expired referendum finalizes");
        }

        assert_eq!(
            votingengine::pallet::Proposals::<T>::get(proposal_id).map(|item| item.status),
            Some(votingengine::STATUS_REJECTED)
        );
    }
}
