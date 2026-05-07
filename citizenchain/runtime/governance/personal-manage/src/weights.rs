//! Weight functions for `personal_manage`.
//!
//! 当前为零权重占位,与 organization-manage 拆分前的 weights.rs 同步骨架;
//! 实际 benchmark 数值在执行 `cargo run --release --bin citizenchain --features runtime-benchmarks`
//! 后由 frame-benchmarking 自动生成。

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn propose_create() -> Weight;
    fn propose_close() -> Weight;
    fn cleanup_rejected_proposal() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_create() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn propose_close() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn cleanup_rejected_proposal() -> Weight {
        Weight::from_parts(0, 0)
    }
}

impl WeightInfo for () {
    fn propose_create() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn propose_close() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn cleanup_rejected_proposal() -> Weight {
        Weight::from_parts(0, 0)
    }
}
