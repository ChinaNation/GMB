//! 运行时升级模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use primitives::cid::china::china_cb::CHINA_CB;
use sp_runtime::{
    sp_std::vec,
    traits::{Hash, SaturatedConversion, Saturating},
};

use crate::pallet::{CodeOf, Config, ReasonOf};
use crate::Pallet;

const BENCH_MAX_REASON_LEN: u32 = 1024;
const BENCH_MAX_CODE_SIZE: u32 = 5 * 1024 * 1024;

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

fn prepare_population_snapshot<T>(who: &T::AccountId)
where
    T: Config + joint_vote::Config,
{
    let now = frame_system::Pallet::<T>::block_number();
    let prepared_at = now.saturating_add(1u32.saturated_into());
    joint_vote::PendingPopulationSnapshots::<T>::insert(
        who,
        joint_vote::PreparedPopulationSnapshot {
            eligible_total: 10u64,
            scope: votingengine::PopulationScope::Country,
            prepared_at,
        },
    );
}

#[benchmarks(where T: Config + joint_vote::Config)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_runtime_upgrade() {
        let proposer = nrc_admin::<T>();
        let reason = reason_max::<T>();
        let code = code_max::<T>();
        prepare_population_snapshot::<T>(&proposer);

        #[block]
        {
            Pallet::<T>::propose_runtime_upgrade(RawOrigin::Signed(proposer).into(), reason, code)
                .expect("benchmark runtime upgrade proposal should succeed");
        }

        let proposal_id = votingengine::Pallet::<T>::next_proposal_id().saturating_sub(1);
        assert!(
            votingengine::Pallet::<T>::get_proposal_data(proposal_id).is_some(),
            "runtime upgrade benchmark should store proposal data in voting engine"
        );
    }
}
