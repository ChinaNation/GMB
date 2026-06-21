//! CPMS 站点 scope 谓词
//!
//! 中文注释:CPMS 站点是否属于当前联邦注册局机构管理员范围,是 CPMS 专用判断,
//! 不再放入通用 `scope` 权限规则目录。

use crate::CpmsSiteKeys;

pub(crate) fn in_scope_cpms_site(site: &CpmsSiteKeys, province_name: Option<&str>) -> bool {
    match province_name {
        Some(scope) => site.province_name == scope,
        None => true,
    }
}
