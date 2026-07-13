//! 立法机关表决本地账本清理边界。
//!
//! 具体分块清理由 `LegislationCleanupHandler` 实现统一调用，所有代表机构阶段按
//! `proposal_id` 前缀清理，不能只清理当前机构。
