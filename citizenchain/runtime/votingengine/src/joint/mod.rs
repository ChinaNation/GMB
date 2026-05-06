//! 联合投票模式(含两阶段:管理员阶段 + 联合公投兜底)。
//!
//! - `joint.rs` — 联合管理员阶段实现(NRC/PRC 发起,PRB 只能投票)
//! - `jointreferendum.rs` — 联合公投阶段(管理员阶段未全票通过 → 全国 SFID 公民兜底投票)
//!
//! 阶段流转:
//!   STAGE_INTERNAL → STAGE_JOINT → STAGE_CITIZEN(jointreferendum)
//!
//! 105 票全票通过时跳过 STAGE_CITIZEN 直接终结;否则进入 jointreferendum 兜底。

pub mod joint;
pub mod jointreferendum;
