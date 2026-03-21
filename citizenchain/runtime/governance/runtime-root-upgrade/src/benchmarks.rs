//! 运行时升级模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use primitives::china::china_cb::CHINA_CB;
use sp_runtime::sp_std::vec;
use sp_runtime::traits::Hash;

use crate::pallet::{
    CodeOf, Config, Proposal, ProposalStatus, ReasonOf, SnapshotNonceOf, SnapshotSignatureOf,
};
use crate::{Call, Pallet};

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

/// 向 voting-engine-system 的 ProposalData 中插入一个处于 Voting 状态的提案。
fn insert_voting_proposal<T: Config>(proposal_id: u64) {
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
    let data = proposal.encode();
    voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)
        .expect("benchmark store_proposal_data should succeed");
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

        // 提案数据应已存入 voting engine
        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(0u64).is_some()
            || voting_engine_system::Pallet::<T>::get_proposal_data(100u64).is_some());
    }

    #[benchmark]
    fn finalize_joint_vote_approved() {
        insert_voting_proposal::<T>(0u64);

        #[extrinsic_call]
        finalize_joint_vote(RawOrigin::Root, 0u64, true);

        let raw = voting_engine_system::Pallet::<T>::get_proposal_data(0u64)
            .expect("proposal should exist");
        let proposal = Proposal::<T>::decode(&mut &raw[..]).expect("should decode");
        assert!(matches!(proposal.status, ProposalStatus::Passed));
        assert!(
            proposal.code.is_empty(),
            "successful finalize should clear code"
        );
    }

    #[benchmark]
    fn finalize_joint_vote_rejected() {
        insert_voting_proposal::<T>(1u64);

        #[extrinsic_call]
        finalize_joint_vote(RawOrigin::Root, 1u64, false);

        let raw = voting_engine_system::Pallet::<T>::get_proposal_data(1u64)
            .expect("proposal should exist");
        let proposal = Proposal::<T>::decode(&mut &raw[..]).expect("should decode");
        assert!(matches!(proposal.status, ProposalStatus::Rejected));
        assert!(
            proposal.code.is_empty(),
            "rejected finalize should clear code"
        );
    }
}
