//! Weight functions for `citizen-identity`.
//!
//! 当前为手工保守上界。身份写入同时维护资格 revision、不可变历史版本和四级人口计数；
//! 在专用 benchmark 落地前，按最重身份资料更新和迁居路径预留数据库预算。

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
    fn register_voting_identity() -> Weight;
    fn upgrade_to_candidate_identity() -> Weight;
    fn update_voting_identity() -> Weight;
    fn update_candidate_identity() -> Weight;
    fn revoke_identity() -> Weight;
    fn occupy_cid() -> Weight;
    fn occupy_cids_batch(n: u32) -> Weight;
    fn revoke_cid() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_voting_identity() -> Weight {
        Weight::from_parts(120_000_000, 100_000)
            .saturating_add(T::DbWeight::get().reads(16))
            .saturating_add(T::DbWeight::get().writes(18))
    }

    fn upgrade_to_candidate_identity() -> Weight {
        Weight::from_parts(130_000_000, 100_000)
            .saturating_add(T::DbWeight::get().reads(16))
            .saturating_add(T::DbWeight::get().writes(19))
    }

    fn update_voting_identity() -> Weight {
        Weight::from_parts(180_000_000, 130_000)
            .saturating_add(T::DbWeight::get().reads(24))
            .saturating_add(T::DbWeight::get().writes(24))
    }

    fn update_candidate_identity() -> Weight {
        Weight::from_parts(190_000_000, 130_000)
            .saturating_add(T::DbWeight::get().reads(24))
            .saturating_add(T::DbWeight::get().writes(25))
    }

    fn revoke_identity() -> Weight {
        Weight::from_parts(170_000_000, 130_000)
            .saturating_add(T::DbWeight::get().reads(22))
            .saturating_add(T::DbWeight::get().writes(24))
    }

    fn occupy_cid() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn occupy_cids_batch(n: u32) -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(Weight::from_parts(25_000_000, 3_000).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads_writes(1, 1).saturating_mul(n.into()))
    }

    fn revoke_cid() -> Weight {
        Weight::from_parts(190_000_000, 130_000)
            .saturating_add(T::DbWeight::get().reads(24))
            .saturating_add(T::DbWeight::get().writes(26))
    }
}

impl WeightInfo for () {
    fn register_voting_identity() -> Weight {
        Weight::from_parts(120_000_000, 100_000)
            .saturating_add(RocksDbWeight::get().reads(16))
            .saturating_add(RocksDbWeight::get().writes(18))
    }

    fn upgrade_to_candidate_identity() -> Weight {
        Weight::from_parts(130_000_000, 100_000)
            .saturating_add(RocksDbWeight::get().reads(16))
            .saturating_add(RocksDbWeight::get().writes(19))
    }

    fn update_voting_identity() -> Weight {
        Weight::from_parts(180_000_000, 130_000)
            .saturating_add(RocksDbWeight::get().reads(24))
            .saturating_add(RocksDbWeight::get().writes(24))
    }

    fn update_candidate_identity() -> Weight {
        Weight::from_parts(190_000_000, 130_000)
            .saturating_add(RocksDbWeight::get().reads(24))
            .saturating_add(RocksDbWeight::get().writes(25))
    }

    fn revoke_identity() -> Weight {
        Weight::from_parts(170_000_000, 130_000)
            .saturating_add(RocksDbWeight::get().reads(22))
            .saturating_add(RocksDbWeight::get().writes(24))
    }

    fn occupy_cid() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn occupy_cids_batch(n: u32) -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(Weight::from_parts(25_000_000, 3_000).saturating_mul(n.into()))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().reads_writes(1, 1).saturating_mul(n.into()))
    }

    fn revoke_cid() -> Weight {
        Weight::from_parts(190_000_000, 130_000)
            .saturating_add(RocksDbWeight::get().reads(24))
            .saturating_add(RocksDbWeight::get().writes(26))
    }
}
