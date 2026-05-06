//! Weight functions for `votingengine` 引擎核心。
//!
//! 引擎核心承载三条生命周期 extrinsic 的权重:
//! - `finalize_proposal(proposal_id)` — 任意人触发超时结算,内部经 trait 派发
//!   到 `T::InternalFinalizer / T::JointFinalizer`,实际成本由对应 sub-pallet
//!   的 `finalize_internal_timeout` / `finalize_joint_timeout` /
//!   `finalize_jointreferendum_timeout` 估算累加。本 weight 取三者最大保守值。
//! - `retry_passed_proposal(proposal_id)` — 管理员手动重试。
//! - `cancel_passed_proposal(proposal_id, reason)` — 管理员取消失败提案。
//!
//! mode-specific 权重函数住在各 sub-pallet 自己的 `weights.rs`。

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
    /// `finalize_proposal(proposal_id)` — 静态最大值,实际派发到 mode pallet。
    fn finalize_proposal() -> Weight;
    fn retry_passed_proposal() -> Weight;
    fn cancel_passed_proposal() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn finalize_proposal() -> Weight {
        // 三个 mode finalize 中最重(joint 25_597_000 ps + 19871 proof)的保守上界。
        Weight::from_parts(25_597_000, 0)
            .saturating_add(Weight::from_parts(0, 19871))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn retry_passed_proposal() -> Weight {
        T::DbWeight::get().reads_writes(8, 8)
    }
    fn cancel_passed_proposal() -> Weight {
        T::DbWeight::get().reads_writes(7, 7)
    }
}

impl WeightInfo for () {
    fn finalize_proposal() -> Weight {
        Weight::from_parts(25_597_000, 0)
            .saturating_add(Weight::from_parts(0, 19871))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn retry_passed_proposal() -> Weight {
        RocksDbWeight::get().reads_writes(8, 8)
    }
    fn cancel_passed_proposal() -> Weight {
        RocksDbWeight::get().reads_writes(7, 7)
    }
}
