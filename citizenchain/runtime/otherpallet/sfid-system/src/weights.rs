//! Weights for `sfid_system`
//!
//! 当前只保留 SFID 绑定、解绑与投票资格消费。签发管理员集合统一来自
//! admins-change,本 pallet 不再维护任何省级签发管理员 storage。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for `sfid_system`.
pub trait WeightInfo {
    fn bind_sfid() -> Weight;
    fn unbind_sfid() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bind_sfid() -> Weight {
        // 中文注释:bind_sfid 触达 BindingId/AccountTo/BoundCount/UsedBindNonce + 回调,reads 6 / writes 5。
        Weight::from_parts(150_000_000, 6_403_489)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    fn unbind_sfid() -> Weight {
        // 中文注释:Root origin → 不读 SFID admin storage;reads = AccountTo + 写 BindingId/AccountTo/BoundCount。
        Weight::from_parts(28_000_000, 3_562)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

}

impl WeightInfo for () {
    fn bind_sfid() -> Weight {
        Weight::from_parts(150_000_000, 6_403_489)
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(5))
    }

    fn unbind_sfid() -> Weight {
        Weight::from_parts(28_000_000, 3_562)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

}
