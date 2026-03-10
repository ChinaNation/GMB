//! 决议销毁模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn propose_destroy() -> Weight;
    fn vote_destroy() -> Weight;
    fn execute_destroy() -> Weight;
    fn cancel_stale_destroy() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_destroy() -> Weight {
        Weight::from_parts(80_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(8, 8))
    }

    fn vote_destroy() -> Weight {
        // 最重路径：达成通过阈值并尝试自动执行销毁。
        Weight::from_parts(220_000_000, 12_288)
            .saturating_add(T::DbWeight::get().reads_writes(14, 12))
    }

    fn execute_destroy() -> Weight {
        Weight::from_parts(140_000_000, 8_192).saturating_add(T::DbWeight::get().reads_writes(9, 8))
    }

    fn cancel_stale_destroy() -> Weight {
        Weight::from_parts(70_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(6, 6))
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn propose_destroy() -> Weight {
        Weight::from_parts(80_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(8, 8))
    }

    fn vote_destroy() -> Weight {
        Weight::from_parts(220_000_000, 12_288)
            .saturating_add(RocksDbWeight::get().reads_writes(14, 12))
    }

    fn execute_destroy() -> Weight {
        Weight::from_parts(140_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(9, 8))
    }

    fn cancel_stale_destroy() -> Weight {
        Weight::from_parts(70_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(6, 6))
    }
}
