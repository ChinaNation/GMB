//! Weight functions for `internal-vote`.
//!
//! 2026-07-15 使用 benchmark CLI 53.0.0、WASM compiled、steps=50、repeat=20
//! 实测生成；`cast` 覆盖命中阈值后进入异步执行队列的最后一票。

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
    /// `cast(proposal_id, approve)` — 管理员一人一票,
    /// 命中阈值的最后一票只写 PASSED 并进入异步业务执行队列。
    fn cast() -> Weight;
    /// 内部投票超时结算(由引擎核心 `finalize_proposal` 通过 trait 派发)。
    fn finalize_internal_timeout() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn cast() -> Weight {
        Weight::from_parts(29_000_000, 0)
            .saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().writes(8))
    }
    fn finalize_internal_timeout() -> Weight {
        Weight::from_parts(32_000_000, 0)
            .saturating_add(Weight::from_parts(0, 105_893))
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(10))
    }
}

impl WeightInfo for () {
    fn cast() -> Weight {
        Weight::from_parts(29_000_000, 0)
            .saturating_add(Weight::from_parts(0, 67_187))
            .saturating_add(RocksDbWeight::get().reads(7))
            .saturating_add(RocksDbWeight::get().writes(8))
    }
    fn finalize_internal_timeout() -> Weight {
        Weight::from_parts(32_000_000, 0)
            .saturating_add(Weight::from_parts(0, 105_893))
            .saturating_add(RocksDbWeight::get().reads(9))
            .saturating_add(RocksDbWeight::get().writes(10))
    }
}
