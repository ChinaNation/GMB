//! Conservative fallback weights for `resolution_issuance`.
//!
//! 中文注释：当前仓库的 benchmark 运行依赖带 Benchmark Runtime API 的 WASM。
//! 本次尝试确认现有 CI WASM 不包含该 API，而本地从源码构建 benchmark WASM
//! 仍被上游 `wasm32v1-none`/`std` feature 问题阻塞。这里先使用偏高保守上界，
//! 不把本文件伪装成正式 benchmark 产物；发布前必须用 benchmark WASM 重新生成。

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
    fn set_allowed_recipients() -> Weight;
    fn propose_resolution_issuance() -> Weight;
    fn clear_executed() -> Weight;
    fn set_paused() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn set_allowed_recipients() -> Weight {
        // 中文注释：覆盖读取活跃提案数、读取旧名单、校验并写入新名单的维护路径。
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 6_000))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn propose_resolution_issuance() -> Weight {
        // 中文注释：覆盖收款名单校验、Voting ProposalData 写入和提案计数写入的最重公开入口。
        Weight::from_parts(180_000_000, 0)
            .saturating_add(Weight::from_parts(0, 25_000))
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().writes(7))
    }

    fn clear_executed() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn set_paused() -> Weight {
        Weight::from_parts(18_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn set_allowed_recipients() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 6_000))
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn propose_resolution_issuance() -> Weight {
        Weight::from_parts(180_000_000, 0)
            .saturating_add(Weight::from_parts(0, 25_000))
            .saturating_add(RocksDbWeight::get().reads(7))
            .saturating_add(RocksDbWeight::get().writes(7))
    }

    fn clear_executed() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn set_paused() -> Weight {
        Weight::from_parts(18_000_000, 0)
            .saturating_add(Weight::from_parts(0, 4_000))
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
