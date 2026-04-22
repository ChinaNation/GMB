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

/// Weight functions for `duoqian_manage_pow`.
pub trait WeightInfo {
	fn register_sfid_institution() -> Weight;
	fn propose_create() -> Weight;
	fn propose_create_personal() -> Weight;
	/// `n` = 聚合的签名数量(= 管理员投票数)。
	fn finalize_create(n: u32) -> Weight;
	fn propose_close() -> Weight;
	fn vote_close() -> Weight;
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
	/// 基础成本 + 每签名 40_000_000 增量(sr25519 验签 + cast_internal_vote 开销的占位)。
	fn finalize_create(n: u32) -> Weight {
		Weight::from_parts(60_000_000, 0)
			.saturating_add(Weight::from_parts(40_000_000, 0).saturating_mul(n.into()))
			.saturating_add(Weight::from_parts(0, 4554))
			.saturating_add(T::DbWeight::get().reads(6 + u64::from(n)))
			.saturating_add(T::DbWeight::get().writes(8 + u64::from(n)))
	}
	fn propose_close() -> Weight {
		Weight::from_parts(70_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn vote_close() -> Weight {
		Weight::from_parts(150_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4554))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(12))
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
	fn finalize_create(n: u32) -> Weight {
		Weight::from_parts(60_000_000, 0)
			.saturating_add(Weight::from_parts(40_000_000, 0).saturating_mul(n.into()))
			.saturating_add(Weight::from_parts(0, 4554))
			.saturating_add(RocksDbWeight::get().reads(6 + u64::from(n)))
			.saturating_add(RocksDbWeight::get().writes(8 + u64::from(n)))
	}
	fn propose_close() -> Weight {
		Weight::from_parts(70_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn vote_close() -> Weight {
		Weight::from_parts(150_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4554))
			.saturating_add(RocksDbWeight::get().reads(9))
			.saturating_add(RocksDbWeight::get().writes(12))
	}
	fn cleanup_rejected_proposal() -> Weight {
		Weight::from_parts(30_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3619))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
}
