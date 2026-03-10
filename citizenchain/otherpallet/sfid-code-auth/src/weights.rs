// SFID 绑定与资格校验模块权重定义。
//
// 当前为保守手动估算值，后续应由 `frame-benchmarking` 自动生成替换。

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 权重接口：由 runtime 注入实现。
pub trait WeightInfo {
    fn bind_sfid() -> Weight;
    fn unbind_sfid() -> Weight;
    fn rotate_sfid_keys() -> Weight;
}

/// 默认保守估算实现（Runtime 使用）。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// bind_sfid 基础权重（不含 OnSfidBound 回调）。
    fn bind_sfid() -> Weight {
        T::DbWeight::get().reads_writes(7, 7)
    }

    fn unbind_sfid() -> Weight {
        T::DbWeight::get().reads_writes(2, 2)
    }

    fn rotate_sfid_keys() -> Weight {
        T::DbWeight::get().reads_writes(3, 3)
    }
}

/// 单元测试用实现。
impl WeightInfo for () {
    fn bind_sfid() -> Weight {
        RocksDbWeight::get().reads_writes(7, 7)
    }

    fn unbind_sfid() -> Weight {
        RocksDbWeight::get().reads_writes(2, 2)
    }

    fn rotate_sfid_keys() -> Weight {
        RocksDbWeight::get().reads_writes(3, 3)
    }
}
