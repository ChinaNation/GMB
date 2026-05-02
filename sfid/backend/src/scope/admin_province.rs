//! 按 pubkey + role 解析管理员所属省份
//!
//! 中文注释:本文件由 Phase 23c 从 `business/scope.rs` 物理搬迁而来。
//! 这两个函数与 `rules.rs::get_visible_scope`(从 ctx 派生 VisibleScope)互补,
//! 用于尚未走 ctx.admin_province 的旧路径,从 pubkey 兜底查省。

use crate::scope::pubkey::same_admin_pubkey;
use crate::sfid::province::sheng_admin_province;
use crate::{AdminRole, Store};

/// 根据 pubkey + role 解析该管理员所属省份。
///
/// 注意:sheng_admin_province_by_pubkey 的 key 可能带/不带 0x 前缀,
/// 必须用 same_admin_pubkey(大小写+前缀不敏感) 遍历匹配,不能用 HashMap.get() 精确匹配。
pub(crate) fn province_scope_for_role(
    store: &Store,
    admin_pubkey: &str,
    role: &AdminRole,
) -> Option<String> {
    // ADR-008(2026-05-01)后只剩 ShengAdmin / ShiAdmin。
    match role {
        AdminRole::ShengAdmin => find_province_by_pubkey(store, admin_pubkey),
        AdminRole::ShiAdmin => {
            let creator_pubkey = store
                .admin_users_by_pubkey
                .iter()
                .find(|(k, _)| same_admin_pubkey(k.as_str(), admin_pubkey))
                .map(|(_, u)| u.created_by.clone())?;
            find_province_by_pubkey(store, &creator_pubkey)
        }
    }
}

/// 用 same_admin_pubkey 遍历 sheng_admin_province_by_pubkey 查找省份,
/// 回退到内置省份表。
fn find_province_by_pubkey(store: &Store, pubkey: &str) -> Option<String> {
    store
        .sheng_admin_province_by_pubkey
        .iter()
        .find(|(k, _)| same_admin_pubkey(k.as_str(), pubkey))
        .map(|(_, province)| province.clone())
        .or_else(|| sheng_admin_province(pubkey).map(|v| v.to_string()))
}
