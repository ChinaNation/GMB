//! 占位 weights，后续由 benchmark 生成替换。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for `duoqian_transfer_pow`.
pub trait WeightInfo {
	fn propose_transfer() -> Weight;
	fn vote_transfer() -> Weight;
	fn execute_transfer() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn propose_transfer() -> Weight {
		Weight::from_parts(55_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn vote_transfer() -> Weight {
		Weight::from_parts(140_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4554))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// execute_transfer 参考 resolution-destro-gov::execute_destroy 估算：
	/// 读取 ProposalData + Proposals + Account，执行转账 + 手续费扣取。
	fn execute_transfer() -> Weight {
		Weight::from_parts(75_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}

impl WeightInfo for () {
	fn propose_transfer() -> Weight {
		Weight::from_parts(55_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn vote_transfer() -> Weight {
		Weight::from_parts(140_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4554))
			.saturating_add(RocksDbWeight::get().reads(9))
			.saturating_add(RocksDbWeight::get().writes(12))
	}
	fn execute_transfer() -> Weight {
		Weight::from_parts(75_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
}
