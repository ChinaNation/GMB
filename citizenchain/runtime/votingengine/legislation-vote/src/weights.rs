//! `legislation-vote` 权重。
//!
//! 2026-07-15 使用 benchmark CLI 53.0.0、WASM compiled、steps=50、repeat=20
//! 实测生成。签署类路径在实测 storage 主体成本上叠加 Runtime provider 查询预算。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::{constants::RocksDbWeight, Weight}};

pub trait WeightInfo {
    fn prepare_population_snapshot() -> Weight;
    fn cast_representative_vote() -> Weight;
    fn cast_referendum_vote() -> Weight;
    fn executive_sign() -> Weight;
    fn override_sign() -> Weight;
    fn guard_vote() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn prepare_population_snapshot() -> Weight {
        Weight::from_parts(12_000_000, 0).saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(T::DbWeight::get().reads(3)).saturating_add(T::DbWeight::get().writes(1))
    }
    fn cast_representative_vote() -> Weight {
        Weight::from_parts(31_000_000, 0).saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(T::DbWeight::get().reads(6)).saturating_add(T::DbWeight::get().writes(7))
    }
    fn cast_referendum_vote() -> Weight {
        Weight::from_parts(22_000_000, 0).saturating_add(Weight::from_parts(0, 11_996))
            .saturating_add(T::DbWeight::get().reads(6)).saturating_add(T::DbWeight::get().writes(2))
    }
    fn executive_sign() -> Weight {
        Weight::from_parts(38_000_000, 0).saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(T::DbWeight::get().reads(7)).saturating_add(T::DbWeight::get().writes(5))
    }
    fn override_sign() -> Weight {
        Weight::from_parts(45_000_000, 0).saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(T::DbWeight::get().reads(10)).saturating_add(T::DbWeight::get().writes(2))
    }
    fn guard_vote() -> Weight {
        Weight::from_parts(35_000_000, 0).saturating_add(Weight::from_parts(0, 30_000))
            .saturating_add(T::DbWeight::get().reads(5)).saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn prepare_population_snapshot() -> Weight { Weight::from_parts(12_000_000, 67_187).saturating_add(RocksDbWeight::get().reads_writes(3, 1)) }
    fn cast_representative_vote() -> Weight { Weight::from_parts(31_000_000, 67_187).saturating_add(RocksDbWeight::get().reads_writes(6, 7)) }
    fn cast_referendum_vote() -> Weight { Weight::from_parts(22_000_000, 11_996).saturating_add(RocksDbWeight::get().reads_writes(6, 2)) }
    fn executive_sign() -> Weight { Weight::from_parts(38_000_000, 67_187).saturating_add(RocksDbWeight::get().reads_writes(7, 5)) }
    fn override_sign() -> Weight { Weight::from_parts(45_000_000, 67_187).saturating_add(RocksDbWeight::get().reads_writes(10, 2)) }
    fn guard_vote() -> Weight { Weight::from_parts(35_000_000, 30_000).saturating_add(RocksDbWeight::get().reads_writes(5, 1)) }
}
