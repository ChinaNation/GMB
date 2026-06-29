//! 目标身份主体分区表结构。
//!
//! 中文注释:本模块只保存数据库目标结构常量,实际执行入口在 `main.rs` 的
//! `init_current_schema`。所有表从第一版目标结构开始按 `province_code` 省级分区。

pub(crate) const PARTITIONED_TABLES: &[&str] = &[
    "subjects",
    "citizens",
    "gov",
    "private",
    "accounts",
    "docs",
    "audit",
    "institution_admins",
];
