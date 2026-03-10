//! 决议发行执行模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use alloc::vec;

use frame_benchmarking::v2::*;
use frame_support::traits::EnsureOrigin;
use sp_runtime::traits::SaturatedConversion;

use crate::{pallet, Call, Config, Executed, Pallet, Paused, ResolutionIssuanceExecutor};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn execute_resolution_issuance() {
        let proposal_id = 1u64;
        let reason = vec![b'x'; 32];

        let recipient_a: T::AccountId = frame_benchmarking::account("recipient", 0, 0);
        let recipient_b: T::AccountId = frame_benchmarking::account("recipient", 1, 0);
        let amount_a: pallet::BalanceOf<T> = 100u32.saturated_into();
        let amount_b: pallet::BalanceOf<T> = 200u32.saturated_into();
        let total_amount: pallet::BalanceOf<T> = 300u32.saturated_into();

        let allocations = vec![(recipient_a, amount_a), (recipient_b, amount_b)];

        #[block]
        {
            assert!(
                <Pallet<T> as ResolutionIssuanceExecutor<T::AccountId, pallet::BalanceOf<T>>>::execute_resolution_issuance(
                    proposal_id,
                    reason.clone(),
                    total_amount,
                    allocations.clone(),
                )
                .is_ok()
            );
        }

        assert!(Executed::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn clear_executed() {
        let proposal_id = 2u64;
        let reason = vec![b'c'; 8];
        let recipient: T::AccountId = frame_benchmarking::account("recipient", 2, 0);
        let amount: pallet::BalanceOf<T> = 100u32.saturated_into();

        assert!(
            <Pallet<T> as ResolutionIssuanceExecutor<T::AccountId, pallet::BalanceOf<T>>>::execute_resolution_issuance(
                proposal_id,
                reason,
                amount,
                vec![(recipient, amount)],
            )
            .is_ok()
        );

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
