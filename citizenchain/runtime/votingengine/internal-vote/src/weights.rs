//! Weight functions for `internal-vote`.
//!
//! 当前为手工调高的保守上界,等正式 benchmark fixture 覆盖最重业务回调路径后
//! 用 substrate-benchmark-cli 重生成。

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
    /// 命中阈值的最后一票同步进入业务 executor 链路。
    fn cast() -> Weight;
    /// 内部投票超时结算(由引擎核心 `finalize_proposal` 通过 trait 派发)。
    fn finalize_internal_timeout() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn cast() -> Weight {
        Weight::from_parts(1_000_000_000, 0)
            .saturating_add(Weight::from_parts(0, 200_000))
            .saturating_add(T::DbWeight::get().reads(80))
            .saturating_add(T::DbWeight::get().writes(50))
    }
    fn finalize_internal_timeout() -> Weight {
        Weight::from_parts(16_791_000, 0)
            .saturating_add(Weight::from_parts(0, 3559))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn cast() -> Weight {
        Weight::from_parts(1_000_000_000, 0)
            .saturating_add(Weight::from_parts(0, 200_000))
            .saturating_add(RocksDbWeight::get().reads(80))
            .saturating_add(RocksDbWeight::get().writes(50))
    }
    fn finalize_internal_timeout() -> Weight {
        Weight::from_parts(16_791_000, 0)
            .saturating_add(Weight::from_parts(0, 3559))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
