//! 运行时升级模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn propose_runtime_upgrade() -> Weight;
    fn finalize_joint_vote() -> Weight;
    fn retry_failed_execution() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_runtime_upgrade() -> Weight {
        // 含跨模块 JointVoteEngine::create_joint_proposal 调用。
        Weight::from_parts(100_000_000, 8_192).saturating_add(T::DbWeight::get().reads_writes(3, 5))
    }

    fn finalize_joint_vote() -> Weight {
        // 最坏路径：通过后 code 执行失败，保留 code + 写 RetryCount + 清理映射。
        Weight::from_parts(150_000_000, 8_192).saturating_add(T::DbWeight::get().reads_writes(3, 6))
    }

    fn retry_failed_execution() -> Weight {
        Weight::from_parts(80_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(2, 2))
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn propose_runtime_upgrade() -> Weight {
        Weight::from_parts(100_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(3, 5))
    }

    fn finalize_joint_vote() -> Weight {
        Weight::from_parts(150_000_000, 8_192)
            .saturating_add(RocksDbWeight::get().reads_writes(3, 6))
    }

    fn retry_failed_execution() -> Weight {
        Weight::from_parts(80_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(2, 2))
    }
}
