//! 中文注释:权限决策类型占位文件。
//!
//! 当前权限决策(scope::filter_by_scope、scope::province_scope_for_role 等)
//! 实现在 `crate::scope` 模块,DTO 直接复用 `AdminRole` / `AdminStatus`,
//! 无独立 DTO 需要下沉。本文件保留语义占位,后续如需抽出权限请求/响应类型
//! 再迁入此处。
