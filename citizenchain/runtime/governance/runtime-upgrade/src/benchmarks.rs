//! 运行时升级模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use primitives::china::china_cb::CHINA_CB;
use sp_runtime::sp_std::vec;

use crate::pallet::{CodeOf, Config, ReasonOf};
use crate::{Call, Pallet};

const BENCH_MAX_REASON_LEN: u32 = 1024;
const BENCH_MAX_CODE_SIZE: u32 = 5 * 1024 * 1024;

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

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_runtime_upgrade() {
        let proposer = nrc_admin::<T>();
        let reason = reason_max::<T>();
        let code = code_max::<T>();

        #[extrinsic_call]
        propose_runtime_upgrade(RawOrigin::Signed(proposer), reason, code);

        let proposal_id = votingengine::Pallet::<T>::next_proposal_id().saturating_sub(1);
        assert!(
            votingengine::Pallet::<T>::get_proposal_data(proposal_id).is_some(),
            "runtime upgrade benchmark should store proposal data in voting engine"
        );
    }
}
