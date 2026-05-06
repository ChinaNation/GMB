//! 公民投票胜出规则(Phase 3 实现)。
//!
//! ```ignore
//! pub enum BallotRule {
//!     Referendum { threshold_percent: u8 },   // 二元公投 + 阈值
//!     PluralityElection,                       // 简单多数(得票最多者胜)
//!     MajorityElection,                        // 绝对多数(>50% 才能胜)
//!     Runoff,                                  // 两轮决选
//! }
//! ```
//!
//! Phase 1 占位 stub。
