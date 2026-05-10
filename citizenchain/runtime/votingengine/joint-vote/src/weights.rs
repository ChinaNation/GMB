//! Weight functions for `joint-vote`.
//!
//! 当前为手工保守上界,等 substrate-benchmark-cli 真实跑测后重生成。

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
    /// `cast_referendum(proposal_id, binding_id, ...)` — 联合公投阶段。
    fn cast_referendum() -> Weight;
    /// 联合内部投票阶段超时结算(经引擎核心 trait 派发)。
    fn finalize_joint_timeout() -> Weight;
    /// 联合公投阶段超时结算(经引擎核心 trait 派发)。
    fn finalize_jointreferendum_timeout() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn prepare_joint_population_snapshot() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(Weight::from_parts(0, 3570))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn cast_admin() -> Weight {
        Weight::from_parts(23_123_000, 0)
            .saturating_add(Weight::from_parts(0, 3559))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn cast_referendum() -> Weight {
        Weight::from_parts(38_031_000, 0)
            .saturating_add(Weight::from_parts(0, 3570))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn finalize_joint_timeout() -> Weight {
        Weight::from_parts(25_597_000, 0)
            .saturating_add(Weight::from_parts(0, 19871))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn finalize_jointreferendum_timeout() -> Weight {
        Weight::from_parts(20_678_000, 0)
            .saturating_add(Weight::from_parts(0, 3559))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn prepare_joint_population_snapshot() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(Weight::from_parts(0, 3570))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn cast_admin() -> Weight {
        Weight::from_parts(23_123_000, 0)
            .saturating_add(Weight::from_parts(0, 3559))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn cast_referendum() -> Weight {
        Weight::from_parts(38_031_000, 0)
            .saturating_add(Weight::from_parts(0, 3570))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn finalize_joint_timeout() -> Weight {
        Weight::from_parts(25_597_000, 0)
            .saturating_add(Weight::from_parts(0, 19871))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn finalize_jointreferendum_timeout() -> Weight {
        Weight::from_parts(20_678_000, 0)
            .saturating_add(Weight::from_parts(0, 3559))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
