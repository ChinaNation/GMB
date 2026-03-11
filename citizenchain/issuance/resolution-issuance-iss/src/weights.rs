//! 决议发行执行模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn execute_resolution_issuance(reason_len: u32, allocation_count: u32) -> Weight;
    fn clear_executed() -> Weight;
    fn set_paused() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn execute_resolution_issuance(reason_len: u32, allocation_count: u32) -> Weight {
        let reason_len = reason_len as u64;
        let allocation_count = allocation_count as u64;
        Weight::from_parts(120_000_000, 4_096)
            .saturating_add(Weight::from_parts(20_000_000, 256).saturating_mul(allocation_count))
            .saturating_add(Weight::from_parts(40_000, 1).saturating_mul(reason_len))
            .saturating_add(
                T::DbWeight::get().reads_writes(4 + allocation_count, 5 + allocation_count),
            )
    }

    fn clear_executed() -> Weight {
        Weight::from_parts(10_000_000, 128).saturating_add(T::DbWeight::get().reads_writes(1, 2))
    }

    fn set_paused() -> Weight {
        Weight::from_parts(5_000_000, 64).saturating_add(T::DbWeight::get().reads_writes(1, 2))
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn execute_resolution_issuance(reason_len: u32, allocation_count: u32) -> Weight {
        let reason_len = reason_len as u64;
        let allocation_count = allocation_count as u64;
        Weight::from_parts(120_000_000, 4_096)
            .saturating_add(Weight::from_parts(20_000_000, 256).saturating_mul(allocation_count))
            .saturating_add(Weight::from_parts(40_000, 1).saturating_mul(reason_len))
            .saturating_add(
                RocksDbWeight::get().reads_writes(4 + allocation_count, 5 + allocation_count),
            )
    }

    fn clear_executed() -> Weight {
        Weight::from_parts(10_000_000, 128).saturating_add(RocksDbWeight::get().reads_writes(1, 2))
    }

    fn set_paused() -> Weight {
        Weight::from_parts(5_000_000, 64).saturating_add(RocksDbWeight::get().reads_writes(1, 2))
    }
}
