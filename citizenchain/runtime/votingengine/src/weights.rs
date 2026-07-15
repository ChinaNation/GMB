//! Weight functions for `votingengine` 引擎核心。
//!
//! 引擎核心承载三条生命周期 extrinsic 的权重:
//! - `finalize_proposal(proposal_id)` — 任意人触发超时结算，经 `TrackHandlers`
//!   派发到提案所属 sub-pallet；本 weight 是核心调度保守包络，调用注解另叠加
//!   提案所属 Track 的动态判定权重。
//! - `retry_passed_proposal(proposal_id)` — 管理员手动重试。
//! - `cancel_passed_proposal(proposal_id, reason)` — 管理员取消失败提案。
//!
//! mode-specific 权重函数住在各 sub-pallet 自己的 `weights.rs`。核心数值于
//! 2026-07-15 使用 benchmark CLI 53.0.0、WASM compiled、steps=50、repeat=20 实测。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 从 Runtime 最大区块权重派生独立维护管线预算。
pub struct BlockWeightFraction<T, const DIVISOR: u64>(PhantomData<T>);

impl<T: frame_system::Config, const DIVISOR: u64> Get<Weight>
    for BlockWeightFraction<T, DIVISOR>
{
    fn get() -> Weight {
        let divisor = DIVISOR.max(1);
        let max = <T as frame_system::Config>::BlockWeights::get().max_block;
        Weight::from_parts(max.ref_time() / divisor, max.proof_size() / divisor)
    }
}

pub trait WeightInfo {
    /// `finalize_proposal(proposal_id)` — 核心调度保守包络，另叠加所属 Track 权重。
    fn finalize_proposal() -> Weight;
    fn retry_passed_proposal() -> Weight;
    fn cancel_passed_proposal() -> Weight;
    fn process_pending_execution() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn finalize_proposal() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(Weight::from_parts(0, 105_893))
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(10))
    }
    fn retry_passed_proposal() -> Weight {
        Weight::from_parts(24_000_000, 105_893)
            .saturating_add(T::DbWeight::get().reads(7))
    }
    fn cancel_passed_proposal() -> Weight {
        Weight::from_parts(10_000_000, 105_893)
            .saturating_add(T::DbWeight::get().reads(3))
    }
    fn process_pending_execution() -> Weight {
        // 异步业务回调可能执行 runtime set_code，执行管线按最重路径预留。
        Weight::from_parts(22_000_000, 105_893)
            .saturating_add(T::DbWeight::get().reads_writes(7, 1))
            .saturating_add(
                <<T as frame_system::Config>::SystemWeightInfo as frame_system::weights::WeightInfo>::set_code(),
            )
    }
}

impl WeightInfo for () {
    fn finalize_proposal() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(Weight::from_parts(0, 105_893))
            .saturating_add(RocksDbWeight::get().reads(9))
            .saturating_add(RocksDbWeight::get().writes(10))
    }
    fn retry_passed_proposal() -> Weight {
        Weight::from_parts(24_000_000, 105_893)
            .saturating_add(RocksDbWeight::get().reads(7))
    }
    fn cancel_passed_proposal() -> Weight {
        Weight::from_parts(10_000_000, 105_893)
            .saturating_add(RocksDbWeight::get().reads(3))
    }
    fn process_pending_execution() -> Weight {
        Weight::from_parts(22_000_000, 105_893)
            .saturating_add(RocksDbWeight::get().reads_writes(7, 1))
    }
}
