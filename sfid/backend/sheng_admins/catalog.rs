use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::scope::admin_province::province_scope_for_role;
use crate::*;

/// 二角色均可访问,按 scope 过滤:
/// - SHENG_ADMIN:仅返回自己所属省
/// - SHI_ADMIN:仅返回自己上级省管理员所属省
pub(crate) async fn list_sheng_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // ADR-008 后只保留 SHENG/SHI,按登录管理员省域 scope 过滤。
    let scope_province = province_scope_for_role(&store, &ctx.admin_pubkey, &ctx.role);
    let mut rows: Vec<ShengAdminRow> = store
        .sheng_admin_province_by_pubkey
        .iter()
        .filter_map(|(pubkey, province)| {
            // scope 过滤:有省域的管理员只能看自己所属省。
            if let Some(ref scope) = scope_province {
                if province != scope {
                    return None;
                }
            }
            let user = store.admin_users_by_pubkey.get(pubkey)?;
            if user.role != AdminRole::ShengAdmin {
                return None;
            }
            Some(ShengAdminRow {
                id: user.id,
                province: province.clone(),
                admin_pubkey: user.admin_pubkey.clone(),
                admin_name: if user.admin_name.is_empty() {
                    format!("{province}机构管理员")
                } else {
                    user.admin_name.clone()
                },
                built_in: user.built_in,
                created_at: user.created_at,
                updated_at: user.updated_at,
                signing_pubkey: user.signing_pubkey.clone(),
                signing_created_at: user.signing_created_at,
            })
        })
        .collect();
    rows.sort_by(|a, b| a.province.cmp(&b.province));
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}
