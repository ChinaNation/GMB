//! 立法机关表决终局结果边界。
//!
//! 终局结果由 votingengine 核心交给回调元组；法律、任免和预算业务分别依据
//! `ProposalOwner`/`MODULE_TAG` 认领。任一提案只能由一个业务模块返回非 `Ignored`，
//! 本 pallet 不读取任何业务存储。
