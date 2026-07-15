//! `election-vote` 权重。
//!
//! 候选人数 `c` 决定最后一票生成结果时读取计票和排序的成本。2026-07-15
//! 使用 benchmark CLI 53.0.0、WASM compiled、steps=50、repeat=20 实测生成。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::{constants::RocksDbWeight, Weight}};

pub trait WeightInfo {
    fn cast_popular_vote(c: u32) -> Weight;
    fn cast_mutual_vote(c: u32) -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn cast_popular_vote(c: u32) -> Weight {
        Weight::from_parts(38_212_644, 0)
            .saturating_add(Weight::from_parts(1_524_772, 0).saturating_mul(c.into()))
            .saturating_add(Weight::from_parts(0, 11_996))
            .saturating_add(Weight::from_parts(0, 2_551).saturating_mul(c.into()))
            .saturating_add(T::DbWeight::get().reads(7_u64.saturating_add(c.into())))
            .saturating_add(T::DbWeight::get().writes(9))
    }
    fn cast_mutual_vote(c: u32) -> Weight {
        Weight::from_parts(36_834_244, 0)
            .saturating_add(Weight::from_parts(1_534_883, 0).saturating_mul(c.into()))
            .saturating_add(Weight::from_parts(0, 11_996))
            .saturating_add(Weight::from_parts(0, 2_551).saturating_mul(c.into()))
            .saturating_add(T::DbWeight::get().reads(7_u64.saturating_add(c.into())))
            .saturating_add(T::DbWeight::get().writes(9))
    }
}

impl WeightInfo for () {
    fn cast_popular_vote(c: u32) -> Weight {
        Weight::from_parts(38_212_644, 0)
            .saturating_add(Weight::from_parts(1_524_772, 0).saturating_mul(c.into()))
            .saturating_add(Weight::from_parts(0, 11_996))
            .saturating_add(Weight::from_parts(0, 2_551).saturating_mul(c.into()))
            .saturating_add(RocksDbWeight::get().reads(7_u64.saturating_add(c.into())))
            .saturating_add(RocksDbWeight::get().writes(9))
    }
    fn cast_mutual_vote(c: u32) -> Weight {
        Weight::from_parts(36_834_244, 0)
            .saturating_add(Weight::from_parts(1_534_883, 0).saturating_mul(c.into()))
            .saturating_add(Weight::from_parts(0, 11_996))
            .saturating_add(Weight::from_parts(0, 2_551).saturating_mul(c.into()))
            .saturating_add(RocksDbWeight::get().reads(7_u64.saturating_add(c.into())))
            .saturating_add(RocksDbWeight::get().writes(9))
    }
}
