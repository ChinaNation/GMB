//! 省储行利息模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use crate::{pallet, Call, Config, Pallet};
use frame_benchmarking::v2::*;
use frame_support::traits::{Get, Hooks};
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn force_settle_years(y: Linear<1, 8>) {
        // 设置区块号使 current_year >= y，确保有 y 个年度待结算
        let blocks_per_year = T::BlocksPerYear::get();
        let target_block: u32 = ((y as u64) * blocks_per_year + 1).min(u32::MAX as u64) as u32;
        frame_system::Pallet::<T>::set_block_number(target_block.into());

        #[extrinsic_call]
        force_settle_years(RawOrigin::Root, y);
    }

    #[benchmark]
    fn force_advance_year() {
        // 确保可推进到年度 1
        let blocks_per_year = T::BlocksPerYear::get();
        let target_block = u32::try_from(blocks_per_year.max(1)).unwrap_or(u32::MAX);
        frame_system::Pallet::<T>::set_block_number(target_block.into());
        pallet::LastSettledYear::<T>::put(0u32);

        #[extrinsic_call]
        force_advance_year(RawOrigin::Root, 1u32);

        assert_eq!(pallet::LastSettledYear::<T>::get(), 1u32);
    }

    /// on_initialize 结算路径：年度边界块触发 1 个年度结算。
    #[benchmark]
    fn on_initialize_settlement() {
        let blocks_per_year = T::BlocksPerYear::get();
        // 设置到第 1 年边界块，确保触发结算
        let n: frame_system::pallet_prelude::BlockNumberFor<T> =
            u32::try_from(blocks_per_year.max(1))
                .unwrap_or(u32::MAX)
                .into();
        frame_system::Pallet::<T>::set_block_number(n);
        pallet::LastSettledYear::<T>::put(0u32);

        #[block]
        {
            let _ = Pallet::<T>::on_initialize(n);
        }

        assert_eq!(pallet::LastSettledYear::<T>::get(), 1u32);
    }

    /// on_initialize 年度边界无待结算路径：只读取年度状态，不执行发币。
    #[benchmark]
    fn on_initialize_boundary_noop() {
        let blocks_per_year = T::BlocksPerYear::get();
        // 中文注释：设置到第 1 年边界且已结算，覆盖“边界块但无待结算年度”的权重。
        let n: frame_system::pallet_prelude::BlockNumberFor<T> =
            u32::try_from(blocks_per_year.max(1))
                .unwrap_or(u32::MAX)
                .into();
        frame_system::Pallet::<T>::set_block_number(n);
        pallet::LastSettledYear::<T>::put(1u32);

        #[block]
        {
            let _ = Pallet::<T>::on_initialize(n);
        }

        assert_eq!(pallet::LastSettledYear::<T>::get(), 1u32);
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test,);
}
