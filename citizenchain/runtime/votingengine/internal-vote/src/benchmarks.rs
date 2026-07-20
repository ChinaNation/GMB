//! `internal-vote` FRAME benchmark。
//!
//! `cast` 覆盖命中门槛并进入异步执行队列的最后一票；超时用例覆盖拒绝终态、
//! 业务拒绝通知、名额释放与 90 天清理登记。
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

use crate::{
    pallet::{Config, InternalThresholdSnapshot, Pallet},
    Call, Proposals,
};

fn setup<T: Config>() -> (
    u64,
    T::AccountId,
    votingengine::Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
) {
    let proposal_id = 0u64;
    let institution: T::AccountId = account("institution", 0, 0);
    let voter: T::AccountId = account("voter", 0, 0);
    let actor_cid_number: votingengine::types::CidNumber = b"BENCHMARK-CID"
        .to_vec()
        .try_into()
        .expect("benchmark CID fits runtime bound");
    let admins: frame_support::BoundedVec<T::AccountId, T::MaxAdminsPerInstitution> =
        sp_std::vec![voter.clone()]
            .try_into()
            .expect("single benchmark admin fits runtime bound");
    let now = 1u32.saturated_into();
    frame_system::Pallet::<T>::set_block_number(now);
    let proposal = votingengine::Proposal {
        kind: votingengine::PROPOSAL_KIND_INTERNAL,
        stage: votingengine::STAGE_INTERNAL,
        status: votingengine::STATUS_VOTING,
        internal_code: Some(primitives::cid::code::PMUL),
        actor_cid_number: Some(actor_cid_number.clone()),
        execution_account: Some(institution.clone()),
        subject_cid_numbers: Default::default(),
        start: 0u32.saturated_into(),
        end: 2u32.saturated_into(),
    };
    Proposals::<T>::insert(proposal_id, proposal.clone());
    // 机构业务必须按岗位有效任职快照投票；个人多签才使用 AdminSnapshot。
    votingengine::pallet::EffectiveVoterSnapshot::<T>::insert(
        proposal_id,
        votingengine::ProposalSubject::InstitutionCid(actor_cid_number),
        admins,
    );
    InternalThresholdSnapshot::<T>::insert(proposal_id, 1);
    (proposal_id, voter, proposal)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn cast() {
        let (proposal_id, voter, _) = setup::<T>();

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), proposal_id, true);

        assert!(votingengine::pallet::PendingProposalExecutions::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn finalize_internal_timeout() {
        let (proposal_id, _, proposal) = setup::<T>();
        frame_system::Pallet::<T>::set_block_number(3u32.saturated_into());

        #[block]
        {
            Pallet::<T>::do_finalize_internal_timeout(&proposal, proposal_id)
                .expect("expired internal proposal finalizes");
        }

        assert_eq!(
            Proposals::<T>::get(proposal_id).map(|item| item.status),
            Some(votingengine::STATUS_REJECTED)
        );
    }
}
