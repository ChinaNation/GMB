//! 公民轻节点奖励模块权重定义。
//!
//! 当前为保守手动估算值；配套 benchmark 已提供，后续可用实测值替换。

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

pub trait WeightInfo {
    fn on_sfid_bound() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn on_sfid_bound() -> Weight {
        Weight::from_parts(60_000_000, 4_096).saturating_add(T::DbWeight::get().reads_writes(5, 5))
    }
}

impl WeightInfo for () {
    fn on_sfid_bound() -> Weight {
        Weight::from_parts(60_000_000, 4_096)
            .saturating_add(RocksDbWeight::get().reads_writes(5, 5))
    }
}
