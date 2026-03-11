//! 省储行质押利息模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现（benchmark 生成或手动估算）。
pub trait WeightInfo {
    fn force_settle_years(max_years: u32) -> Weight;
    fn force_advance_year() -> Weight;
}

/// 默认保守估算实现（用于未运行 benchmark 时）。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn force_settle_years(max_years: u32) -> Weight {
        // 每年结算：43 个省储行 × 3 读 + 2 写，加上 CPU 开销
        let banks: u64 = 43;
        let years: u64 = max_years as u64;
        T::DbWeight::get()
            .reads_writes(1 + years * (1 + banks * 3), 1 + years * (2 + banks * 3))
            .saturating_add(Weight::from_parts(
                years.saturating_mul(banks).saturating_mul(50_000),
                0,
            ))
    }

    fn force_advance_year() -> Weight {
        T::DbWeight::get().reads_writes(1, 1)
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn force_settle_years(max_years: u32) -> Weight {
        let banks: u64 = 43;
        let years: u64 = max_years as u64;
        RocksDbWeight::get()
            .reads_writes(1 + years * (1 + banks * 3), 1 + years * (2 + banks * 3))
            .saturating_add(Weight::from_parts(
                years.saturating_mul(banks).saturating_mul(50_000),
                0,
            ))
    }

    fn force_advance_year() -> Weight {
        RocksDbWeight::get().reads_writes(1, 1)
    }
}
