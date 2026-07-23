//! 机构按行政层级的可见范围过滤
//!
//! 此模块是所有 list/CRUD API 进行范围过滤的**唯一入口**。
//! 业务 handler 只需:
//! 1. `let ctx = require_admin_any(...)?;`
//! 2. `let scope = scope::get_visible_scope(&ctx);`
//! 3. `let filtered = scope::filter_by_scope(&rows, &scope);`
//!
//! 五档范围(按机构 admin_level 派生):
//! - 全国(NATIONAL,部委等)        → 不限省/市/镇
//! - 省级(PROVINCE / 联邦注册局)   → 本省,所有市
//! - 市级(CITY)                    → 本市,所有镇
//! - 镇级(TOWN)                    → 本镇
//! - 自机构(私权法人/非法人,无层级)→ 暂沿用本市范围
//!
//! 详细规则见 `rules.rs` 的 `VisibleScope`。
//!
//! 本目录只保留范围规则；HTTP handler、账户与公钥工具归属对应业务模块。

#![allow(dead_code, unused_imports)]

pub mod filter;
pub mod rules;

pub use filter::{filter_by_scope, HasProvinceCity};
pub use rules::get_visible_scope;
