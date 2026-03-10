//! 管理员治理模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn propose_admin_replacement() -> Weight;
    fn vote_admin_replacement() -> Weight;
    fn execute_admin_replacement() -> Weight;
    fn cancel_stale_proposal() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_admin_replacement() -> Weight {
        Weight::from_parts(80_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(8, 8))
    }

    fn vote_admin_replacement() -> Weight {
        Weight::from_parts(200_000_000, 8_192)
            .saturating_add(T::DbWeight::get().reads_writes(12, 10))
    }

    fn execute_admin_replacement() -> Weight {
        Weight::from_parts(120_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(8, 7))
    }

    fn cancel_stale_proposal() -> Weight {
        Weight::from_parts(60_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(4, 4))
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn propose_admin_replacement() -> Weight {
        Weight::from_parts(80_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(8, 8))
    }

    fn vote_admin_replacement() -> Weight {
        Weight::from_parts(200_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(12, 10))
    }

    fn execute_admin_replacement() -> Weight {
        Weight::from_parts(120_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(8, 7))
    }

    fn cancel_stale_proposal() -> Weight {
        Weight::from_parts(60_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(4, 4))
    }
}
