//! 最终性密钥治理模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn propose_replace_finality_key() -> Weight;
    fn vote_replace_finality_key() -> Weight;
    fn execute_replace_finality_key() -> Weight;
    fn cancel_stale_replace_finality_key() -> Weight;
    fn cancel_failed_replace_finality_key() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_replace_finality_key() -> Weight {
        T::DbWeight::get().reads_writes(10, 10)
    }
    fn vote_replace_finality_key() -> Weight {
        T::DbWeight::get().reads_writes(16, 14)
    }
    fn execute_replace_finality_key() -> Weight {
        T::DbWeight::get().reads_writes(14, 12)
    }
    fn cancel_stale_replace_finality_key() -> Weight {
        T::DbWeight::get().reads_writes(8, 8)
    }
    fn cancel_failed_replace_finality_key() -> Weight {
        T::DbWeight::get().reads_writes(12, 10)
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn propose_replace_finality_key() -> Weight {
        RocksDbWeight::get().reads_writes(10, 10)
    }
    fn vote_replace_finality_key() -> Weight {
        RocksDbWeight::get().reads_writes(16, 14)
    }
    fn execute_replace_finality_key() -> Weight {
        RocksDbWeight::get().reads_writes(14, 12)
    }
    fn cancel_stale_replace_finality_key() -> Weight {
        RocksDbWeight::get().reads_writes(8, 8)
    }
    fn cancel_failed_replace_finality_key() -> Weight {
        RocksDbWeight::get().reads_writes(12, 10)
    }
}
