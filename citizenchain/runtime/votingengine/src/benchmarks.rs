//! `votingengine` 核心 FRAME benchmark。
//!
//! mode-specific 写票由各 Track benchmark；这里测量统一超时入口、管理员恢复入口和
//! 异步执行队列框架成本。业务执行最重的 `set_code` 另由 WeightInfo 显式叠加系统权重。
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

use crate::{
    pallet::{
        Config, EffectiveVoterSnapshot, PendingProposalExecutions, ProposalExecutionRetryStates,
        Proposals,
    },
    Call, ExecutionRetryState, Pallet, PendingExecutionState, Proposal, STATUS_PASSED,
    STATUS_VOTING,
};

fn setup<T: Config>(status: u8) -> (u64, T::AccountId) {
    let proposal_id = 0u64;
    let institution: T::AccountId = account("institution", 0, 0);
    let who: T::AccountId = account("admin", 0, 0);
    let actor_cid_number: crate::types::CidNumber = b"BENCHMARK-CID"
        .to_vec()
        .try_into()
        .expect("benchmark CID fits runtime bound");
    let now = 1u32.saturated_into();
    frame_system::Pallet::<T>::set_block_number(now);
    Proposals::<T>::insert(
        proposal_id,
        Proposal {
            kind: crate::PROPOSAL_KIND_INTERNAL,
            stage: crate::STAGE_INTERNAL,
            status,
            internal_code: Some(primitives::cid::code::PMUL),
            actor_cid_number: Some(actor_cid_number.clone()),
            execution_account: Some(institution.clone()),
            subject_cid_numbers: Default::default(),
            start: 0u32.saturated_into(),
            end: 0u32.saturated_into(),
        },
    );
    let admins: frame_support::BoundedVec<T::AccountId, T::MaxAdminsPerInstitution> =
        sp_std::vec![who.clone()]
            .try_into()
            .expect("single benchmark admin fits runtime bound");
    // 机构业务的重试与取消权限来自岗位有效任职快照；个人多签才读取 AdminSnapshot。
    EffectiveVoterSnapshot::<T>::insert(
        proposal_id,
        crate::ProposalSubject::InstitutionCid(actor_cid_number),
        admins,
    );
    if status == STATUS_PASSED {
        ProposalExecutionRetryStates::<T>::insert(
            proposal_id,
            ExecutionRetryState {
                manual_attempts: 0,
                first_auto_failed_at: now,
                retry_deadline: now,
                last_attempt_at: None,
            },
        );
    }
    (proposal_id, who)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn finalize_proposal() {
        let (proposal_id, who) = setup::<T>(STATUS_VOTING);

        #[extrinsic_call]
        _(RawOrigin::Signed(who), proposal_id);

        assert_eq!(
            Proposals::<T>::get(proposal_id).map(|proposal| proposal.status),
            Some(crate::STATUS_REJECTED)
        );
    }

    #[benchmark]
    fn retry_passed_proposal() {
        let (proposal_id, who) = setup::<T>(STATUS_PASSED);

        #[block]
        {
            let _ = Pallet::<T>::retry_passed_proposal(RawOrigin::Signed(who).into(), proposal_id);
        }

        assert!(Proposals::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn cancel_passed_proposal() {
        let (proposal_id, who) = setup::<T>(STATUS_PASSED);
        let reason = sp_std::vec::Vec::new()
            .try_into()
            .expect("empty reason fits");

        #[block]
        {
            let _ = Pallet::<T>::cancel_passed_proposal(
                RawOrigin::Signed(who).into(),
                proposal_id,
                reason,
            );
        }

        assert!(Proposals::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn process_pending_execution() {
        let (proposal_id, _) = setup::<T>(STATUS_PASSED);
        let now = frame_system::Pallet::<T>::block_number();
        PendingProposalExecutions::<T>::insert(
            proposal_id,
            PendingExecutionState {
                attempts: 0,
                next_attempt_at: now,
            },
        );

        #[block]
        {
            let _ = Pallet::<T>::process_pending_proposal_executions(now);
        }

        assert!(PendingProposalExecutions::<T>::contains_key(proposal_id));
    }
}
