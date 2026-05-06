//! 占位 weights,后续由 benchmark 生成替换。
//!
//! 本 pallet 只保留 `propose_X`;投票走 `votingengine::internal_vote`,
//! 执行重试走 `votingengine::retry_passed_proposal`。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for `duoqian_transfer`.
pub trait WeightInfo {
    fn propose_transfer() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_transfer() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 19871))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(7))
    }
}

impl WeightInfo for () {
    fn propose_transfer() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 19871))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(7))
    }
}
