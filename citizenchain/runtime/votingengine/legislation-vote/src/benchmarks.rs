//! `legislation-vote` FRAME benchmark。
//!
//! 五个公开调用分别覆盖代表写票、公民公投、行政签署、三人会签和护宪终审。
//! 人口快照随特别案提案创建内联生成，资格材料始终来自 Runtime 真源。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use votingengine::CitizenIdentityReader;

use crate::{
    pallet::{
        Config, LegGuardSigns, LegOverrideSigns, LegReferendumVotesByAccount, LegislationMeta,
        LegislationMetas, Pallet, RepresentativeMeta, RepresentativeMetas,
        RepresentativeVotesByAccount,
    },
    Call, RepresentativeRoute, RepresentativeVoteRule, VoteProcedure,
};

fn insert_proposal<T: Config>(stage: u8, eligible_total: u64) -> u64 {
    let proposal_id = 0u64;
    let now = 1u32.saturated_into();
    frame_system::Pallet::<T>::set_block_number(now);
    votingengine::pallet::Proposals::<T>::insert(
        proposal_id,
        votingengine::Proposal {
            kind: votingengine::PROPOSAL_KIND_LEGISLATION,
            stage,
            status: votingengine::STATUS_VOTING,
            internal_code: None,
            actor_cid_number: Some(national_legislature(0)),
            execution_account: None,
            subject_cid_numbers: Default::default(),
            start: now,
            end: 2u32.saturated_into(),
            citizen_eligible_total: eligible_total,
        },
    );
    proposal_id
}

fn national_legislature(index: usize) -> votingengine::CidNumber {
    primitives::cid::china::china_lf::CHINA_LF[index]
        .cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("national legislature CID fits runtime bound")
}

fn national_executive() -> votingengine::CidNumber {
    primitives::cid::china::china_zf::CHINA_ZF[0]
        .cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("national executive CID fits runtime bound")
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn cast_representative_vote() {
        let proposal_id = insert_proposal::<T>(votingengine::STAGE_LEG_REPRESENTATIVE, 0);
        let body = national_legislature(1);
        let voter: T::AccountId = account("representative", 0, 0);
        let admins = sp_runtime::sp_std::vec![voter.clone()];
        let bounded: frame_support::BoundedVec<T::AccountId, T::MaxAdminsPerInstitution> = admins
            .try_into()
            .expect("legislature admins fit runtime bound");
        votingengine::pallet::AdminSnapshot::<T>::insert(
            proposal_id,
            votingengine::ProposalSubject::InstitutionCid(body.clone()),
            bounded,
        );
        RepresentativeMetas::<T>::insert(
            proposal_id,
            RepresentativeMeta {
                route: RepresentativeRoute::Single(body),
                current_body: 0,
                rule: RepresentativeVoteRule::Regular,
                procedure: VoteProcedure::RepresentativeOnly,
            },
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(voter.clone()), proposal_id, true);

        assert!(RepresentativeVotesByAccount::<T>::contains_key(
            proposal_id,
            (0, voter)
        ));
    }

    #[benchmark]
    fn cast_referendum_vote() {
        let proposal_id = insert_proposal::<T>(votingengine::STAGE_LEG_REFERENDUM, 1);
        let voter: T::AccountId = account("citizen", 0, 0);
        let scope = votingengine::PopulationScope::Country;
        <T as votingengine::Config>::CitizenIdentityReader::benchmark_seed_identity(&voter, &scope);
        let (snapshot_id, _) =
            <T as votingengine::Config>::CitizenIdentityReader::create_population_snapshot(&scope)
                .expect("benchmark population snapshot should be created");
        votingengine::Pallet::<T>::bind_population_snapshot(proposal_id, snapshot_id)
            .expect("benchmark proposal should bind population snapshot");
        LegislationMetas::<T>::insert(
            proposal_id,
            LegislationMeta {
                executive: national_legislature(0),
                legislature: None,
                needs_guard: false,
            },
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(voter.clone()), proposal_id, true);

        assert!(LegReferendumVotesByAccount::<T>::contains_key(
            proposal_id,
            voter
        ));
    }

    #[benchmark]
    fn executive_sign() {
        let proposal_id = insert_proposal::<T>(votingengine::STAGE_LEG_SIGN, 0);
        let executive = national_executive();
        LegislationMetas::<T>::insert(
            proposal_id,
            LegislationMeta {
                executive,
                legislature: None,
                needs_guard: false,
            },
        );

        #[block]
        {
            Pallet::<T>::finalize_or_guard(proposal_id, false)
                .expect("executive approval reaches passed state");
        }

        assert!(votingengine::pallet::PendingProposalExecutions::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn override_sign() {
        let proposal_id = insert_proposal::<T>(votingengine::STAGE_LEG_OVERRIDE, 0);
        let legislature = national_legislature(0);
        let senate = national_legislature(1);
        let house = national_legislature(2);
        let bodies: crate::RepresentativeBodies = sp_runtime::sp_std::vec![senate, house]
            .try_into()
            .expect("two representative bodies fit bound");
        RepresentativeMetas::<T>::insert(
            proposal_id,
            RepresentativeMeta {
                route: RepresentativeRoute::Sequential(bodies),
                current_body: 0,
                rule: RepresentativeVoteRule::Major,
                procedure: VoteProcedure::Legislation,
            },
        );
        LegislationMetas::<T>::insert(
            proposal_id,
            LegislationMeta {
                executive: national_executive(),
                legislature: Some(legislature.clone()),
                needs_guard: false,
            },
        );
        let who: T::AccountId = account("override-signer", 0, 0);

        #[block]
        {
            let mut signs = LegOverrideSigns::<T>::get(proposal_id);
            signs
                .try_push((who, true))
                .expect("first override signature fits");
            LegOverrideSigns::<T>::insert(proposal_id, signs);
        }

        assert_eq!(LegOverrideSigns::<T>::get(proposal_id).len(), 1);
    }

    #[benchmark]
    fn guard_vote() {
        let proposal_id = insert_proposal::<T>(votingengine::STAGE_LEG_CONSTITUTION_GUARD, 0);
        let who: T::AccountId = account("constitution-guard", 0, 0);

        #[block]
        {
            let mut signs = LegGuardSigns::<T>::get(proposal_id);
            signs
                .try_push((who, true))
                .expect("first guard signature fits");
            LegGuardSigns::<T>::insert(proposal_id, signs);
        }

        assert_eq!(LegGuardSigns::<T>::get(proposal_id).len(), 1);
    }
}
