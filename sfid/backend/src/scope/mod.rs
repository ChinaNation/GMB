//! sfid 系统三角色权限过滤
//!
//! 中文注释:此模块是所有 list/CRUD API 进行角色范围过滤的**唯一入口**。
//! 业务 handler 只需:
//! 1. `let ctx = require_admin_any(...)?;`
//! 2. `let scope = scope::get_visible_scope(&ctx);`
//! 3. `let filtered = scope::filter_by_scope(&rows, &scope);`
//!
//! 各角色范围:
//! - KeyAdmin    → 全国,所有省市
//! - ShengAdmin  → 本省,所有市
//! - ShiAdmin    → 本市
//!
//! 详细规则见 `rules.rs` 的 `VisibleScope`。
//!
//! 任务卡 2 建立。feedback_scope_auto_filter.md 固化。

#![allow(dead_code)]

pub mod filter;
pub mod rules;

pub use filter::{filter_by_scope, filter_map_by_scope, HasProvinceCity};
pub use rules::{get_visible_scope, VisibleScope};
