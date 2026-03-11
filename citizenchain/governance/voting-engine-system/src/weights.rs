//! 投票引擎模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn create_internal_proposal() -> Weight;
    fn submit_joint_institution_vote() -> Weight;
    fn citizen_vote() -> Weight;
    fn finalize_proposal_internal() -> Weight;
    fn finalize_proposal_joint() -> Weight;
    fn finalize_proposal_citizen() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_internal_proposal() -> Weight {
        Weight::from_parts(50_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(2, 2))
    }

    fn submit_joint_institution_vote() -> Weight {
        // 最坏路径含阶段推进与联合回调估算。
        Weight::from_parts(150_000_000, 8_192).saturating_add(T::DbWeight::get().reads_writes(8, 6))
    }

    fn citizen_vote() -> Weight {
        // 最坏路径含提案通过后的联合回调估算。
        Weight::from_parts(120_000_000, 8_192).saturating_add(T::DbWeight::get().reads_writes(9, 4))
    }

    fn finalize_proposal_internal() -> Weight {
        Weight::from_parts(40_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(3, 1))
    }

    fn finalize_proposal_joint() -> Weight {
        // 最坏路径含推进到公民阶段或联合回调估算。
        Weight::from_parts(100_000_000, 8_192).saturating_add(T::DbWeight::get().reads_writes(8, 3))
    }

    fn finalize_proposal_citizen() -> Weight {
        // 最坏路径含联合回调估算。
        Weight::from_parts(80_000_000, 8_192).saturating_add(T::DbWeight::get().reads_writes(7, 2))
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn create_internal_proposal() -> Weight {
        Weight::from_parts(50_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(2, 2))
    }

    fn submit_joint_institution_vote() -> Weight {
        Weight::from_parts(150_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(8, 6))
    }

    fn citizen_vote() -> Weight {
        Weight::from_parts(120_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(9, 4))
    }

    fn finalize_proposal_internal() -> Weight {
        Weight::from_parts(40_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(3, 1))
    }

    fn finalize_proposal_joint() -> Weight {
        Weight::from_parts(100_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(8, 3))
    }

    fn finalize_proposal_citizen() -> Weight {
        Weight::from_parts(80_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(7, 2))
    }
}
