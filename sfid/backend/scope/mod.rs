//! SFID 系统二角色权限范围过滤
//!
//! 中文注释:此模块是所有 list/CRUD API 进行角色范围过滤的**唯一入口**。
//! 业务 handler 只需:
//! 1. `let ctx = require_admin_any(...)?;`
//! 2. `let scope = scope::get_visible_scope(&ctx);`
//! 3. `let filtered = scope::filter_by_scope(&rows, &scope);`
//!
//! 各角色范围(ADR-008 后):
//! - ShengAdmin  → 本省,所有市
//! - ShiAdmin    → 本市
//!
//! 详细规则见 `rules.rs` 的 `VisibleScope`。
//!
//! 2026-05-02 models/scope 边界整改后,本目录只保留权限范围规则。
//! HTTP handler、CPMS 专用判断、pubkey 工具已归还对应业务模块。

#![allow(dead_code)]

pub mod admin_province;
pub mod filter;
pub mod rules;

pub use filter::{filter_by_scope, HasProvinceCity};
pub use rules::get_visible_scope;
