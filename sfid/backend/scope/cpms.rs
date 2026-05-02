//! CPMS 站点 scope 谓词
//!
//! 中文注释:本文件由 Phase 23c 从 `business/scope.rs` 物理搬迁而来。

use crate::CpmsSiteKeys;

pub(crate) fn in_scope_cpms_site(site: &CpmsSiteKeys, admin_province: Option<&str>) -> bool {
    match admin_province {
        Some(scope) => site.admin_province == scope,
        None => true,
    }
}
