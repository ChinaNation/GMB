//! 立法投票 sub-pallet 权重(ADR-027)。
//!
//! 当前使用固定保守权重占位,后续补 benchmark 派生的真实权重。

use frame_support::weights::Weight;

/// 立法机关表决六个 extrinsic 的权重接口。
pub trait WeightInfo {
    fn prepare_population_snapshot() -> Weight;
    fn cast_representative_vote() -> Weight;
    fn cast_referendum_vote() -> Weight;
    fn executive_sign() -> Weight;
    fn override_sign() -> Weight;
    fn guard_vote() -> Weight;
}

impl WeightInfo for () {
    fn prepare_population_snapshot() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }
    fn cast_representative_vote() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }
    fn cast_referendum_vote() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }
    fn executive_sign() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }
    fn override_sign() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }
    fn guard_vote() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }
}
