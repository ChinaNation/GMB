//! 省储行质押利息模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use crate::{pallet, Call, Config, Pallet};
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
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

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test,);
}
