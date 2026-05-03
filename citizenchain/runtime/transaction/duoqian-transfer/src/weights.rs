//! 占位 weights,后续由 benchmark 生成替换。
//!
//! Phase 2 整改后:聚合签名 `finalize_X` 统一删除,投票改走
//! `voting-engine::internal_vote`;execute_xxx wrapper 已统一到
//! `voting_engine::retry_passed_proposal` 入口,本 pallet 只保留 `propose_X`。

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
