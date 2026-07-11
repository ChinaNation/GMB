//! Weight functions for `citizen-identity`.
//!
//! 当前为手工保守上界,真实 benchmark 跑通后由 substrate-benchmark-cli 重生成。

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
    fn prepare_population_snapshot() -> Weight;
    fn occupy_cid() -> Weight;
    fn occupy_cids_batch(n: u32) -> Weight;
    fn revoke_cid() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_voting_identity() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(9))
    }

    fn upgrade_to_candidate_identity() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(Weight::from_parts(0, 7_000))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    fn update_voting_identity() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(10))
    }

    fn update_candidate_identity() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(7))
    }

    fn revoke_identity() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(Weight::from_parts(0, 7_000))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(10))
    }

    fn prepare_population_snapshot() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
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
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(11))
    }
}

impl WeightInfo for () {
    fn register_voting_identity() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(9))
    }

    fn upgrade_to_candidate_identity() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(Weight::from_parts(0, 7_000))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(5))
    }

    fn update_voting_identity() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(10))
    }

    fn update_candidate_identity() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(7))
    }

    fn revoke_identity() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(Weight::from_parts(0, 7_000))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(10))
    }

    fn prepare_population_snapshot() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
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
        Weight::from_parts(55_000_000, 0)
            .saturating_add(Weight::from_parts(0, 8_000))
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(11))
    }
}
