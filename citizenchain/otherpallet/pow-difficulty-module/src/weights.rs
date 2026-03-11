//! PoW 难度模块权重定义。
//!
//! 当前为保守手动估算值；配套 benchmark 已提供，后续可用实测值替换。

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：声明每个区块内本 pallet 对 `on_initialize + on_finalize` 的总预算。
pub trait WeightInfo {
    fn on_initialize_adjustment() -> Weight;
    fn on_initialize_start_window() -> Weight;
    fn on_initialize_idle() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn on_initialize_adjustment() -> Weight {
        Weight::from_parts(55_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(4, 2))
    }

    fn on_initialize_start_window() -> Weight {
        Weight::from_parts(20_000_000, 2_048).saturating_add(T::DbWeight::get().reads_writes(3, 1))
    }

    fn on_initialize_idle() -> Weight {
        Weight::from_parts(10_000_000, 1_024).saturating_add(T::DbWeight::get().reads_writes(2, 0))
    }
}

impl WeightInfo for () {
    fn on_initialize_adjustment() -> Weight {
        Weight::from_parts(55_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(4, 2))
    }

    fn on_initialize_start_window() -> Weight {
        Weight::from_parts(20_000_000, 2_048)
            .saturating_add(RocksDbWeight::get().reads_writes(3, 1))
    }

    fn on_initialize_idle() -> Weight {
        Weight::from_parts(10_000_000, 1_024)
            .saturating_add(RocksDbWeight::get().reads_writes(2, 0))
    }
}
