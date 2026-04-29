//! 手工估算占位 weights，待 benchmark CLI 生成后替换。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for `duoqian_manage`.
pub trait WeightInfo {
	fn register_sfid_institution() -> Weight;
	fn propose_create() -> Weight;
	fn propose_create_personal() -> Weight;
	/// `n` = 聚合的签名数量(= 管理员投票数)。
	fn propose_close() -> Weight;
	fn cleanup_rejected_proposal() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn register_sfid_institution() -> Weight {
		Weight::from_parts(45_334_000, 0)
			.saturating_add(Weight::from_parts(0, 3619))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn propose_create() -> Weight {
		Weight::from_parts(80_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn propose_create_personal() -> Weight {
		Weight::from_parts(80_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(9))
	}
	fn propose_close() -> Weight {
		Weight::from_parts(70_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn cleanup_rejected_proposal() -> Weight {
		Weight::from_parts(30_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3619))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}

impl WeightInfo for () {
	fn register_sfid_institution() -> Weight {
		Weight::from_parts(45_334_000, 0)
			.saturating_add(Weight::from_parts(0, 3619))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	fn propose_create() -> Weight {
		Weight::from_parts(80_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(8))
	}
	fn propose_create_personal() -> Weight {
		Weight::from_parts(80_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(9))
	}
	fn propose_close() -> Weight {
		Weight::from_parts(70_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn cleanup_rejected_proposal() -> Weight {
		Weight::from_parts(30_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3619))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
}
