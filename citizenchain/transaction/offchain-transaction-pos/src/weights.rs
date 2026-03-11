//! 链下交易手续费模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    /// 直接提交会同时覆盖验签、批次校验、转账和 processed tx 写入。
    fn submit_offchain_batch(items: u32) -> Weight;
    /// 入队主要消耗在验签、防重检查和队列持久化。
    fn enqueue_offchain_batch(items: u32) -> Weight;
    /// 出队执行除批次处理外，还包含失败重试/状态回写路径。
    fn process_queued_batch(items: u32) -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn submit_offchain_batch(items: u32) -> Weight {
        let items = items as u64;
        T::DbWeight::get().reads_writes(9 + items.saturating_mul(7), 8 + items.saturating_mul(7))
    }

    fn enqueue_offchain_batch(items: u32) -> Weight {
        let items = items as u64;
        T::DbWeight::get().reads_writes(9 + items.saturating_mul(6), 4 + items.saturating_mul(2))
    }

    fn process_queued_batch(items: u32) -> Weight {
        let items = items as u64;
        T::DbWeight::get().reads_writes(8 + items.saturating_mul(7), 4 + items.saturating_mul(7))
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn submit_offchain_batch(items: u32) -> Weight {
        let items = items as u64;
        RocksDbWeight::get().reads_writes(9 + items.saturating_mul(7), 8 + items.saturating_mul(7))
    }

    fn enqueue_offchain_batch(items: u32) -> Weight {
        let items = items as u64;
        RocksDbWeight::get().reads_writes(9 + items.saturating_mul(6), 4 + items.saturating_mul(2))
    }

    fn process_queued_batch(items: u32) -> Weight {
        let items = items as u64;
        RocksDbWeight::get().reads_writes(8 + items.saturating_mul(7), 4 + items.saturating_mul(7))
    }
}
