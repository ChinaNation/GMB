//! 三种投票模式的垂直切片(internal / joint / citizen)。
//!
//! 每个子模块涵盖一种投票模式的全链路:
//! - 提案创建(`do_create_*`)
//! - 投票(`*_vote` extrinsic 主体)
//! - 终结与回调分发
//! - 模式 trait 实现(`InternalVoteEngine` / `JointVoteEngine`)
//!
//! 三种模式共享 `votingengine` 的 storage/types/mutex/snapshot/data 等基础设施,
//! 但模式之间几乎无耦合,所以选择**垂直切片**而非按操作切片。

pub mod citizen;
pub mod internal;
pub mod joint;
