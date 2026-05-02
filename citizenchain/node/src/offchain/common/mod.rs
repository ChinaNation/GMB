//! 清算行节点层共享类型。
//!
//! 中文注释:
//! - 本模块只放跨目录复用的 DTO / 展示结构。
//! - 业务动作必须落在 `duoqian_manage`、`offchain_transaction` 或 `settlement`
//!   目录内,避免共享目录变成新的大杂烩。

pub mod types;
