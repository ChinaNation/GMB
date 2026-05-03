//! `admins_change` 的保守临时权重。
//!
//! 当前文件用于清理旧 benchmark 产物中的过期 storage proof 注释。
//! 数值沿用旧权重的保守量级；正式发布前应在修复 benchmark 后重新运行
//! `citizenchain/scripts/benchmark.sh admins_change` 生成精确权重。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};

pub trait WeightInfo {
	fn propose_admin_replacement() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn propose_admin_replacement() -> Weight {
		Weight::from_parts(87_473_000, 19_871)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(12))
	}
}

impl WeightInfo for () {
	fn propose_admin_replacement() -> Weight {
		Weight::from_parts(87_473_000, 19_871)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(12))
	}
}
