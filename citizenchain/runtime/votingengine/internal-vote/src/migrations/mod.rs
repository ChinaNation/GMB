//! internal-vote runtime 升级迁移合集。
//!
//! - **v1**(sub-pallet 拆分):storage 从 `VotingEngine` 前缀搬到 `InternalVote` 前缀。

pub mod v1;
