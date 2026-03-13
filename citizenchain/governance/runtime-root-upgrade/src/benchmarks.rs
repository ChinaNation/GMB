//! 运行时升级模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use primitives::china::china_cb::CHINA_CB;
use sp_runtime::sp_std::vec;
use sp_runtime::traits::Hash;

use crate::{
    pallet::{
        CodeOf, Config, Proposal, ProposalStatus, ReasonOf, SnapshotNonceOf, SnapshotSignatureOf,
    },
    Call, GovToJointVote, JointVoteToGov, Pallet, Proposals, RetryCount,
};

const BENCH_MAX_REASON_LEN: u32 = 1024;
const BENCH_MAX_CODE_SIZE: u32 = 5 * 1024 * 1024;
const BENCH_MAX_SNAPSHOT_NONCE_LEN: u32 = 64;
const BENCH_MAX_SNAPSHOT_SIGNATURE_LEN: u32 = 64;

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn nrc_admin<T: Config>() -> T::AccountId {
    decode_account::<T>(CHINA_CB[0].admins[0])
}

fn reason_max<T: Config>() -> ReasonOf<T> {
    assert_eq!(
        T::MaxReasonLen::get(),
        BENCH_MAX_REASON_LEN,
        "update BENCH_MAX_REASON_LEN when runtime MaxReasonLen changes"
    );
    vec![b'r'; BENCH_MAX_REASON_LEN as usize]
        .try_into()
        .expect("benchmark reason should fit")
}

fn code_max<T: Config>() -> CodeOf<T> {
    assert_eq!(
        T::MaxRuntimeCodeSize::get(),
        BENCH_MAX_CODE_SIZE,
        "update BENCH_MAX_CODE_SIZE when runtime MaxRuntimeCodeSize changes"
    );
    vec![b'c'; BENCH_MAX_CODE_SIZE as usize]
        .try_into()
        .expect("benchmark runtime code should fit")
}

fn snapshot_nonce_max<T: Config>() -> SnapshotNonceOf<T> {
    assert_eq!(
        T::MaxSnapshotNonceLength::get(),
        BENCH_MAX_SNAPSHOT_NONCE_LEN,
        "update BENCH_MAX_SNAPSHOT_NONCE_LEN when runtime MaxSnapshotNonceLength changes"
    );
    vec![b'n'; BENCH_MAX_SNAPSHOT_NONCE_LEN as usize]
        .try_into()
        .expect("benchmark snapshot nonce should fit")
}

fn snapshot_signature_max<T: Config>() -> SnapshotSignatureOf<T> {
    assert_eq!(
        T::MaxSnapshotSignatureLength::get(),
        BENCH_MAX_SNAPSHOT_SIGNATURE_LEN,
        "update BENCH_MAX_SNAPSHOT_SIGNATURE_LEN when runtime MaxSnapshotSignatureLength changes"
    );
    vec![b's'; BENCH_MAX_SNAPSHOT_SIGNATURE_LEN as usize]
        .try_into()
        .expect("benchmark snapshot signature should fit")
}

fn insert_voting_proposal<T: Config>(proposal_id: u64, joint_vote_id: u64) {
    let proposer = nrc_admin::<T>();
    let reason = reason_max::<T>();
    let code = code_max::<T>();
    let code_hash = T::Hashing::hash(code.as_slice());
    let proposal = Proposal::<T> {
        proposer,
        reason,
        code_hash,
        code,
        status: ProposalStatus::Voting,
    };
    Proposals::<T>::insert(proposal_id, proposal);
    GovToJointVote::<T>::insert(proposal_id, joint_vote_id);
    JointVoteToGov::<T>::insert(joint_vote_id, proposal_id);
}

fn insert_failed_proposal<T: Config>(proposal_id: u64, retry_count: u32) {
    let proposer = nrc_admin::<T>();
    let reason = reason_max::<T>();
    let code = code_max::<T>();
    let code_hash = T::Hashing::hash(code.as_slice());
    let proposal = Proposal::<T> {
        proposer,
        reason,
        code_hash,
        code,
        status: ProposalStatus::ExecutionFailed,
    };
    Proposals::<T>::insert(proposal_id, proposal);
    RetryCount::<T>::insert(proposal_id, retry_count);
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_runtime_upgrade() {
        let proposer = nrc_admin::<T>();
        let reason = reason_max::<T>();
        let code = code_max::<T>();
        let nonce = snapshot_nonce_max::<T>();
        let signature = snapshot_signature_max::<T>();

        #[extrinsic_call]
        propose_runtime_upgrade(
            RawOrigin::Signed(proposer),
            reason,
            code,
            10u64,
            nonce,
            signature,
        );

        assert!(Proposals::<T>::contains_key(0u64));
        assert!(GovToJointVote::<T>::contains_key(0u64));
    }

    #[benchmark]
    fn finalize_joint_vote_approved() {
        insert_voting_proposal::<T>(0u64, 100u64);

        #[extrinsic_call]
        finalize_joint_vote(RawOrigin::Root, 0u64, true);

        let proposal = Proposals::<T>::get(0u64).expect("proposal should exist");
        assert!(matches!(proposal.status, ProposalStatus::Passed));
        assert!(
            proposal.code.is_empty(),
            "successful finalize should clear code"
        );
        assert!(GovToJointVote::<T>::get(0u64).is_none());
        assert!(JointVoteToGov::<T>::get(100u64).is_none());
    }

    #[benchmark]
    fn finalize_joint_vote_rejected() {
        insert_voting_proposal::<T>(1u64, 101u64);

        #[extrinsic_call]
        finalize_joint_vote(RawOrigin::Root, 1u64, false);

        let proposal = Proposals::<T>::get(1u64).expect("proposal should exist");
        assert!(matches!(proposal.status, ProposalStatus::Rejected));
        assert!(
            proposal.code.is_empty(),
            "rejected finalize should clear code"
        );
        assert!(GovToJointVote::<T>::get(1u64).is_none());
        assert!(JointVoteToGov::<T>::get(101u64).is_none());
    }

    #[benchmark]
    fn retry_failed_execution() {
        let proposer = nrc_admin::<T>();
        insert_failed_proposal::<T>(2u64, 0u32);

        #[extrinsic_call]
        retry_failed_execution(RawOrigin::Signed(proposer), 2u64);

        let proposal = Proposals::<T>::get(2u64).expect("proposal should exist");
        assert!(matches!(proposal.status, ProposalStatus::Passed));
        assert!(
            proposal.code.is_empty(),
            "successful retry should clear code"
        );
        assert_eq!(RetryCount::<T>::get(2u64), 0u32);
    }

    #[benchmark]
    fn cancel_failed_proposal() {
        let proposer = nrc_admin::<T>();
        insert_failed_proposal::<T>(3u64, T::MaxExecutionRetries::get());

        #[extrinsic_call]
        cancel_failed_proposal(RawOrigin::Signed(proposer), 3u64);

        let proposal = Proposals::<T>::get(3u64).expect("proposal should exist");
        assert!(matches!(proposal.status, ProposalStatus::Cancelled));
        assert!(
            proposal.code.is_empty(),
            "cancel should clear retained code"
        );
        assert_eq!(RetryCount::<T>::get(3u64), 0u32);
    }
}
