//! 全节点 PoW 铸块奖励模块权重定义。
//!
//! 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn bind_reward_wallet() -> Weight;
    fn rebind_reward_wallet() -> Weight;
}

/// 默认保守估算实现。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bind_reward_wallet() -> Weight {
        T::DbWeight::get().reads_writes(1, 1)
    }
    fn rebind_reward_wallet() -> Weight {
        T::DbWeight::get().reads_writes(1, 1)
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn bind_reward_wallet() -> Weight {
        RocksDbWeight::get().reads_writes(1, 1)
    }
    fn rebind_reward_wallet() -> Weight {
        RocksDbWeight::get().reads_writes(1, 1)
    }
}
