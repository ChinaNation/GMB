//! 决议发行执行模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use alloc::{vec, vec::Vec};

use frame_benchmarking::v2::*;
use frame_support::traits::{EnsureOrigin, Get};
use sp_runtime::traits::SaturatedConversion;

use crate::{
    pallet, Call, Config, Executed, Pallet, Paused, ResolutionAllocationsOf,
    ResolutionIssuanceExecutor, ResolutionReasonOf,
};

// 中文注释：这里显式对齐当前 mainnet runtime 的上限，若治理常量变更需要同步更新 benchmark 范围。
const BENCH_MAX_REASON_LEN: u32 = 1024;
const BENCH_MAX_ALLOCATIONS: u32 = 43;
const BENCH_AMOUNT_PER_RECIPIENT: u128 = 1_000;

fn benchmark_reason(reason_len: u32) -> Vec<u8> {
    vec![b'x'; reason_len as usize]
}

fn benchmark_allocations<T: Config>(
    allocation_count: u32,
) -> (
    Vec<(T::AccountId, pallet::BalanceOf<T>)>,
    pallet::BalanceOf<T>,
) {
    let allocations = (0..allocation_count)
        .map(|i| {
            (
                frame_benchmarking::account("recipient", i, 0),
                BENCH_AMOUNT_PER_RECIPIENT.saturated_into(),
            )
        })
        .collect();
    let total_amount = BENCH_AMOUNT_PER_RECIPIENT
        .saturating_mul(allocation_count as u128)
        .saturated_into();
    (allocations, total_amount)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn execute_resolution_issuance(
        l: Linear<1, BENCH_MAX_REASON_LEN>,
        a: Linear<1, BENCH_MAX_ALLOCATIONS>,
    ) {
        assert_eq!(
            T::MaxReasonLen::get(),
            BENCH_MAX_REASON_LEN,
            "update BENCH_MAX_REASON_LEN when runtime MaxReasonLen changes"
        );
        assert_eq!(
            T::MaxAllocations::get(),
            BENCH_MAX_ALLOCATIONS,
            "update BENCH_MAX_ALLOCATIONS when runtime MaxAllocations changes"
        );
        let proposal_id = ((l as u64) << 32) | (a as u64);
        let reason: ResolutionReasonOf<T::MaxReasonLen> = benchmark_reason(l)
            .try_into()
            .expect("benchmark reason should fit bounded trait payload");
        let (raw_allocations, total_amount) = benchmark_allocations::<T>(a);
        let allocations: ResolutionAllocationsOf<
            T::AccountId,
            pallet::BalanceOf<T>,
            T::MaxAllocations,
        > = raw_allocations
            .try_into()
            .expect("benchmark allocations should fit bounded trait payload");

        #[block]
        {
            assert!(<Pallet<T> as ResolutionIssuanceExecutor<
                T::AccountId,
                pallet::BalanceOf<T>,
                T::MaxReasonLen,
                T::MaxAllocations,
            >>::execute_resolution_issuance(
                proposal_id,
                reason.clone(),
                total_amount,
                allocations.clone(),
            )
            .is_ok());
        }

        assert!(Executed::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn clear_executed() {
        let proposal_id = 2u64;
        let reason: ResolutionReasonOf<T::MaxReasonLen> = vec![b'c'; 8]
            .try_into()
            .expect("benchmark reason should fit bounded trait payload");
        let recipient: T::AccountId = frame_benchmarking::account("recipient", 2, 0);
        let amount: pallet::BalanceOf<T> = 200u32.saturated_into();
        let allocations: ResolutionAllocationsOf<
            T::AccountId,
            pallet::BalanceOf<T>,
            T::MaxAllocations,
        > = vec![(recipient, amount)]
            .try_into()
            .expect("benchmark allocations should fit bounded trait payload");

        assert!(<Pallet<T> as ResolutionIssuanceExecutor<
            T::AccountId,
            pallet::BalanceOf<T>,
            T::MaxReasonLen,
            T::MaxAllocations,
        >>::execute_resolution_issuance(proposal_id, reason, amount, allocations,)
        .is_ok());

        let origin = T::MaintenanceOrigin::try_successful_origin()
            .expect("MaintenanceOrigin must provide successful benchmark origin");

        #[extrinsic_call]
        clear_executed(origin, proposal_id);

        assert!(!Executed::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn set_paused() {
        assert!(!Paused::<T>::get());

        let origin = T::MaintenanceOrigin::try_successful_origin()
            .expect("MaintenanceOrigin must provide successful benchmark origin");

        #[extrinsic_call]
        set_paused(origin, true);

        assert!(Paused::<T>::get());
    }
}
