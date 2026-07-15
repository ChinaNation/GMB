//! Weight functions for `joint-vote`.
//!
//! 2026-07-15 使用 benchmark CLI 53.0.0、WASM compiled、steps=50、repeat=20
//! 实测生成。人口快照函数另叠加真实权限/人口读取的防御性数据库预算。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

pub trait WeightInfo {
    /// `prepare_joint_population_snapshot(...)` — 联合公投人口快照准备。
    fn prepare_joint_population_snapshot() -> Weight;
    /// `cast_admin(proposal_id, institution, approve)` — 内部投票阶段。
    fn cast_admin() -> Weight;
    /// `cast_referendum(proposal_id, approve)` — 联合公投阶段。
    fn cast_referendum() -> Weight;
    /// 联合内部投票阶段超时结算(经引擎核心 trait 派发)。
    fn finalize_joint_timeout() -> Weight;
    /// 联合公投阶段超时结算(经引擎核心 trait 派发)。
    fn finalize_jointreferendum_timeout() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn prepare_joint_population_snapshot() -> Weight {
        Weight::from_parts(12_000_000, 0)
            .saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn cast_admin() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    fn cast_referendum() -> Weight {
        Weight::from_parts(22_000_000, 0)
            .saturating_add(Weight::from_parts(0, 11_996))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn finalize_joint_timeout() -> Weight {
        Weight::from_parts(13_000_000, 0)
            .saturating_add(Weight::from_parts(0, 38_752))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn finalize_jointreferendum_timeout() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(0, 12_451))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(9))
    }
}

impl WeightInfo for () {
    fn prepare_joint_population_snapshot() -> Weight {
        Weight::from_parts(12_000_000, 0)
            .saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn cast_admin() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(4))
    }
    fn cast_referendum() -> Weight {
        Weight::from_parts(22_000_000, 0)
            .saturating_add(Weight::from_parts(0, 11_996))
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn finalize_joint_timeout() -> Weight {
        Weight::from_parts(13_000_000, 0)
            .saturating_add(Weight::from_parts(0, 38_752))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn finalize_jointreferendum_timeout() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(0, 12_451))
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(9))
    }
}
