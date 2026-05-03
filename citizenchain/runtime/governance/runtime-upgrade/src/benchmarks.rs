//! 运行时升级模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::{pallet_prelude::ConstU32, traits::Get, BoundedVec};
use frame_system::RawOrigin;
use primitives::china::china_cb::CHINA_CB;
use sp_runtime::sp_std::vec;

use crate::pallet::{CodeOf, Config, ReasonOf, SnapshotNonceOf, SnapshotSignatureOf};
use crate::{Call, Pallet};

const BENCH_MAX_REASON_LEN: u32 = 1024;
const BENCH_MAX_CODE_SIZE: u32 = 5 * 1024 * 1024;
const BENCH_MAX_SNAPSHOT_NONCE_LEN: u32 = 64;
const BENCH_MAX_SNAPSHOT_SIGNATURE_LEN: u32 = 64;

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn nrc_admin<T: Config>() -> T::AccountId {
    decode_account::<T>(CHINA_CB[0].duoqian_admins[0])
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

fn signature_max<T: Config>() -> SnapshotSignatureOf<T> {
    assert_eq!(
        T::MaxSnapshotSignatureLength::get(),
        BENCH_MAX_SNAPSHOT_SIGNATURE_LEN,
        "update BENCH_MAX_SNAPSHOT_SIGNATURE_LEN when runtime MaxSnapshotSignatureLength changes"
    );
    vec![b's'; BENCH_MAX_SNAPSHOT_SIGNATURE_LEN as usize]
        .try_into()
        .expect("benchmark snapshot signature should fit")
}

fn province_ok() -> BoundedVec<u8, ConstU32<64>> {
    b"liaoning"
        .to_vec()
        .try_into()
        .expect("benchmark province should fit")
}

fn signer_admin_pubkey_ok() -> [u8; 32] {
    [7u8; 32]
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
        let signature = signature_max::<T>();
        let province = province_ok();
        let signer_admin_pubkey = signer_admin_pubkey_ok();

        #[extrinsic_call]
        propose_runtime_upgrade(
            RawOrigin::Signed(proposer),
            reason,
            code,
            10u64,
            nonce,
            signature,
            province,
            signer_admin_pubkey,
        );

        let proposal_id = voting_engine::Pallet::<T>::next_proposal_id().saturating_sub(1);
        assert!(
            voting_engine::Pallet::<T>::get_proposal_data(proposal_id).is_some(),
            "runtime upgrade benchmark should store proposal data in voting engine"
        );
    }
}
