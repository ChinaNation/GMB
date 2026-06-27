//! 立法院模块权重(ADR-027)。
//!
//! 中文注释:当前使用固定保守权重占位,后续补 benchmark 派生的真实权重。

use frame_support::weights::Weight;

/// 立法院三个提案入口的权重接口。
pub trait WeightInfo {
    fn propose_enact_law() -> Weight;
    fn propose_amend_law() -> Weight;
    fn propose_repeal_law() -> Weight;
}

/// 默认空实现:固定保守权重(读写若干 storage + 一次投票引擎建提案)。
impl WeightInfo for () {
    fn propose_enact_law() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }
    fn propose_amend_law() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }
    fn propose_repeal_law() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }
}
