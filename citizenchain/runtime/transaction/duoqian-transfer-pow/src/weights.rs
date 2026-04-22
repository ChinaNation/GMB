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
///
/// Step 2 · 离线 QR 聚合改造:`vote_transfer` / `vote_safety_fund_transfer` /
/// `vote_sweep_to_main` 被替换为 `finalize_X(n: u32)`,`n` 为聚合签名数。
pub trait WeightInfo {
	fn propose_transfer() -> Weight;
	/// `n` = 聚合的签名数量(= 管理员投票数)。
	fn finalize_transfer(n: u32) -> Weight;
	fn execute_transfer() -> Weight;
	fn finalize_safety_fund_transfer(n: u32) -> Weight;
	fn finalize_sweep_to_main(n: u32) -> Weight;
}

/// 基础权重 + 每签名增量的通用公式,用于三个 finalize_X 占位权重。
fn finalize_base<T: frame_system::Config>(n: u32) -> Weight {
	Weight::from_parts(60_000_000, 0)
		.saturating_add(Weight::from_parts(40_000_000, 0).saturating_mul(n.into()))
		.saturating_add(Weight::from_parts(0, 4554))
		.saturating_add(T::DbWeight::get().reads(6 + u64::from(n)))
		.saturating_add(T::DbWeight::get().writes(8 + u64::from(n)))
}

fn finalize_base_rocks(n: u32) -> Weight {
	Weight::from_parts(60_000_000, 0)
		.saturating_add(Weight::from_parts(40_000_000, 0).saturating_mul(n.into()))
		.saturating_add(Weight::from_parts(0, 4554))
		.saturating_add(RocksDbWeight::get().reads(6 + u64::from(n)))
		.saturating_add(RocksDbWeight::get().writes(8 + u64::from(n)))
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn propose_transfer() -> Weight {
		Weight::from_parts(55_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn finalize_transfer(n: u32) -> Weight {
		finalize_base::<T>(n)
	}
	/// execute_transfer 读取 ProposalData + Proposals + Account,执行转账 + 手续费扣取。
	fn execute_transfer() -> Weight {
		Weight::from_parts(75_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn finalize_safety_fund_transfer(n: u32) -> Weight {
		finalize_base::<T>(n)
	}
	fn finalize_sweep_to_main(n: u32) -> Weight {
		finalize_base::<T>(n)
	}
}

impl WeightInfo for () {
	fn propose_transfer() -> Weight {
		Weight::from_parts(55_000_000, 0)
			.saturating_add(Weight::from_parts(0, 19871))
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn finalize_transfer(n: u32) -> Weight {
		finalize_base_rocks(n)
	}
	fn execute_transfer() -> Weight {
		Weight::from_parts(75_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn finalize_safety_fund_transfer(n: u32) -> Weight {
		finalize_base_rocks(n)
	}
	fn finalize_sweep_to_main(n: u32) -> Weight {
		finalize_base_rocks(n)
	}
}
