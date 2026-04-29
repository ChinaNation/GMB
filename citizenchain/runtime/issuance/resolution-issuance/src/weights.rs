//! Weight functions for `resolution_issuance`.
//!
//! 中文注释：该文件在模块合并时先使用保守权重占位，后续必须通过
//! `citizenchain/scripts/benchmark.sh` 对 `resolution_issuance` 重新生成。

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

pub trait WeightInfo {
    fn set_allowed_recipients() -> Weight;
    fn propose_resolution_issuance() -> Weight;
    fn finalize_joint_vote_approved() -> Weight;
    fn finalize_joint_vote_rejected() -> Weight;
    fn clear_executed() -> Weight;
    fn set_paused() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn set_allowed_recipients() -> Weight {
        Weight::from_parts(13_000_000, 2_000).saturating_add(T::DbWeight::get().reads_writes(2, 1))
    }

    fn propose_resolution_issuance() -> Weight {
        Weight::from_parts(80_000_000, 9_000).saturating_add(T::DbWeight::get().reads_writes(4, 4))
    }

    fn finalize_joint_vote_approved() -> Weight {
        Weight::from_parts(140_000_000, 12_000)
            .saturating_add(T::DbWeight::get().reads_writes(8, 8))
    }

    fn finalize_joint_vote_rejected() -> Weight {
        Weight::from_parts(35_000_000, 5_000).saturating_add(T::DbWeight::get().reads_writes(3, 4))
    }

    fn clear_executed() -> Weight {
        Weight::from_parts(12_000_000, 2_500).saturating_add(T::DbWeight::get().reads_writes(1, 1))
    }

    fn set_paused() -> Weight {
        Weight::from_parts(10_000_000, 1_500).saturating_add(T::DbWeight::get().reads_writes(1, 1))
    }
}

impl WeightInfo for () {
    fn set_allowed_recipients() -> Weight {
        Weight::from_parts(13_000_000, 2_000)
    }

    fn propose_resolution_issuance() -> Weight {
        Weight::from_parts(80_000_000, 9_000)
    }

    fn finalize_joint_vote_approved() -> Weight {
        Weight::from_parts(140_000_000, 12_000)
    }

    fn finalize_joint_vote_rejected() -> Weight {
        Weight::from_parts(35_000_000, 5_000)
    }

    fn clear_executed() -> Weight {
        Weight::from_parts(12_000_000, 2_500)
    }

    fn set_paused() -> Weight {
        Weight::from_parts(10_000_000, 1_500)
    }
}
