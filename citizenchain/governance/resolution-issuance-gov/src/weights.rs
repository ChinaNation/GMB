//! 决议发行治理模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use resolution_issuance_iss::weights::WeightInfo as IssuanceWeightInfoT;

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn propose_resolution_issuance(allocation_count: u32, reason_len: u32) -> Weight;
    fn finalize_joint_vote_approved() -> Weight;
    fn finalize_joint_vote_rejected() -> Weight;
    fn set_allowed_recipients(recipient_count: u32) -> Weight;
    fn retry_failed_execution() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: crate::pallet::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_resolution_issuance(allocation_count: u32, reason_len: u32) -> Weight {
        T::DbWeight::get()
            .reads_writes(4, 7)
            .saturating_add(Weight::from_parts(80_000, 128).saturating_mul(allocation_count as u64))
            .saturating_add(Weight::from_parts(500, 1).saturating_mul(reason_len as u64))
    }

    fn finalize_joint_vote_approved() -> Weight {
        T::DbWeight::get()
            .reads_writes(2, 4)
            .saturating_add(T::IssuanceWeightInfo::execute_resolution_issuance(
                T::MaxReasonLen::get(),
                T::MaxAllocations::get(),
            ))
    }

    fn finalize_joint_vote_rejected() -> Weight {
        T::DbWeight::get().reads_writes(3, 4)
    }

    fn set_allowed_recipients(recipient_count: u32) -> Weight {
        T::DbWeight::get()
            .reads_writes(1, 1)
            .saturating_add(Weight::from_parts(80_000, 128).saturating_mul(recipient_count as u64))
    }

    fn retry_failed_execution() -> Weight {
        T::DbWeight::get()
            .reads_writes(2, 2)
            .saturating_add(T::IssuanceWeightInfo::execute_resolution_issuance(
                T::MaxReasonLen::get(),
                T::MaxAllocations::get(),
            ))
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn propose_resolution_issuance(allocation_count: u32, reason_len: u32) -> Weight {
        RocksDbWeight::get()
            .reads_writes(4, 7)
            .saturating_add(Weight::from_parts(80_000, 128).saturating_mul(allocation_count as u64))
            .saturating_add(Weight::from_parts(500, 1).saturating_mul(reason_len as u64))
    }

    fn finalize_joint_vote_approved() -> Weight {
        RocksDbWeight::get()
            .reads_writes(2, 4)
            .saturating_add(<() as IssuanceWeightInfoT>::execute_resolution_issuance(1024, 128))
    }

    fn finalize_joint_vote_rejected() -> Weight {
        RocksDbWeight::get().reads_writes(3, 4)
    }

    fn set_allowed_recipients(recipient_count: u32) -> Weight {
        RocksDbWeight::get()
            .reads_writes(1, 1)
            .saturating_add(Weight::from_parts(80_000, 128).saturating_mul(recipient_count as u64))
    }

    fn retry_failed_execution() -> Weight {
        RocksDbWeight::get()
            .reads_writes(2, 2)
            .saturating_add(<() as IssuanceWeightInfoT>::execute_resolution_issuance(1024, 128))
    }
}
