//! PoW 难度模块 benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_support::traits::Hooks;
use primitives::pow_const::{DIFFICULTY_ADJUSTMENT_INTERVAL, DIFFICULTY_TARGET_WINDOW_MS};
use sp_runtime::traits::SaturatedConversion;

use crate::pallet::{Config, CurrentDifficulty, Pallet, WindowStartMs};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn on_initialize_adjustment() {
        let n: frame_system::pallet_prelude::BlockNumberFor<T> =
            (DIFFICULTY_ADJUSTMENT_INTERVAL + 1).saturated_into();
        frame_system::Pallet::<T>::set_block_number(n);
        CurrentDifficulty::<T>::put(1_000u64);
        WindowStartMs::<T>::put(1_000u64);
        pallet_timestamp::Pallet::<T>::set_timestamp(
            (1_000u64.saturating_add(DIFFICULTY_TARGET_WINDOW_MS)).saturated_into(),
        );

        #[block]
        {
            let _ = Pallet::<T>::on_initialize(n);
            Pallet::<T>::on_finalize(n);
        }

        assert_eq!(
            WindowStartMs::<T>::get(),
            Some(1_000u64.saturating_add(DIFFICULTY_TARGET_WINDOW_MS))
        );
    }

    #[benchmark]
    fn on_initialize_start_window() {
        let n: frame_system::pallet_prelude::BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(n);
        WindowStartMs::<T>::kill();
        pallet_timestamp::Pallet::<T>::set_timestamp(6_000u64.saturated_into());

        #[block]
        {
            let _ = Pallet::<T>::on_initialize(n);
            Pallet::<T>::on_finalize(n);
        }

        assert_eq!(WindowStartMs::<T>::get(), Some(6_000u64));
    }

    #[benchmark]
    fn on_initialize_idle() {
        let n: frame_system::pallet_prelude::BlockNumberFor<T> = 2u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(n);
        WindowStartMs::<T>::put(1_000u64);
        pallet_timestamp::Pallet::<T>::set_timestamp(12_000u64.saturated_into());

        #[block]
        {
            let _ = Pallet::<T>::on_initialize(n);
            Pallet::<T>::on_finalize(n);
        }

        assert_eq!(WindowStartMs::<T>::get(), Some(1_000u64));
    }
}
