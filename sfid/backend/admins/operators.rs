//! 省管理员查看市管理员列表与内部变更辅助函数。
//!
//! 中文注释:列表读取和姓名修改只需要登录态;新增、删除市管理员统一由
//! `admins::actions` 的 Passkey + 冷钱包签名挑战提交。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::crypto::pubkey::same_admin_pubkey;
use crate::scope::admin_province::province_scope_for_role;
use crate::*;

pub(crate) const MAX_ADMIN_NAME_CHARS: usize = 200;
pub(crate) const MAX_SHI_ADMINS_PER_CITY: usize = 30;

pub(crate) async fn list_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<OperatorRow> = store
        .admin_users_by_pubkey
        .values()
        .filter(|u| u.role == AdminRole::ShiAdmin)
        .filter(|u| {
            can_manage_operator(
                &store,
                ctx.admin_pubkey.as_str(),
                ctx.admin_province.as_deref(),
                u,
            )
        })
        .map(|u| operator_row_from_user(&store, u))
        .collect();
    rows.sort_by(|a, b| b.id.cmp(&a.id));
    let total = rows.len();
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);
    let rows = rows
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: OperatorListOutput {
            total,
            limit,
            offset,
            rows,
        },
    })
    .into_response()
}

pub(crate) fn allocate_next_admin_user_id(store: &mut Store) -> u64 {
    if store.next_admin_user_id == 0 {
        store.next_admin_user_id = store
            .admin_users_by_pubkey
            .values()
            .map(|u| u.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
    }
    let candidate = store.next_admin_user_id;
    store.next_admin_user_id = store.next_admin_user_id.saturating_add(1);
    candidate
}

pub(crate) fn can_manage_operator(
    store: &Store,
    actor_pubkey: &str,
    actor_province: Option<&str>,
    operator: &AdminUser,
) -> bool {
    // 中文注释:省管理员可管理自己创建的市管理员,也可管理本省范围内的市管理员。
    if same_admin_pubkey(operator.created_by.as_str(), actor_pubkey) {
        return true;
    }
    let Some(scope) = actor_province else {
        return false;
    };
    province_scope_for_role(store, &operator.admin_pubkey, &operator.role)
        .map(|operator_scope| operator_scope == scope)
        .unwrap_or(false)
}

pub(crate) fn find_operator_pubkey_by_id(store: &Store, id: u64) -> Option<String> {
    store
        .admin_users_by_pubkey
        .values()
        .find(|u| u.id == id && u.role == AdminRole::ShiAdmin)
        .map(|u| u.admin_pubkey.clone())
}

pub(crate) fn operator_row_from_user(store: &Store, operator: &AdminUser) -> OperatorRow {
    OperatorRow {
        id: operator.id,
        admin_pubkey: operator.admin_pubkey.clone(),
        admin_name: operator.admin_name.clone(),
        role: operator.role.clone(),
        built_in: operator.built_in,
        created_by: operator.created_by.clone(),
        created_by_name: creator_display_name(store, operator.created_by.as_str()),
        created_at: operator.created_at,
        city: operator.city.clone(),
    }
}

pub(crate) fn count_shi_admins_in_city(store: &Store, province: &str, city: &str) -> usize {
    // 中文注释:市名可能跨省重复,所以必须同时按省份和市名统计。
    let city = city.trim();
    store
        .admin_users_by_pubkey
        .values()
        .filter(|user| user.role == AdminRole::ShiAdmin)
        .filter(|user| user.city == city)
        .filter(|user| {
            province_scope_for_role(store, user.admin_pubkey.as_str(), &user.role)
                .map(|operator_province| operator_province == province)
                .unwrap_or(false)
        })
        .count()
}

pub(crate) fn creator_display_name(store: &Store, creator_pubkey: &str) -> String {
    let Some(creator) = store.admin_users_by_pubkey.get(creator_pubkey) else {
        return creator_pubkey.to_string();
    };
    let province = if creator.role == AdminRole::ShengAdmin {
        store
            .sheng_admin_province_by_pubkey
            .get(creator_pubkey)
            .map(String::as_str)
    } else {
        None
    };
    build_admin_display_name(creator_pubkey, &creator.role, province)
}

pub(crate) fn ensure_city_in_creator_province(
    store: &Store,
    creator_pubkey: &str,
    city: &str,
) -> Result<(String, String), axum::response::Response> {
    let city = city.trim();
    if city.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "city is required"));
    }
    let Some(province_name) =
        province_scope_for_role(store, creator_pubkey, &AdminRole::ShengAdmin)
    else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "cannot resolve province from created_by",
        ));
    };
    let Some(city_code) = crate::sfid::province::city_code_by_name(province_name.as_str(), city)
    else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "city not found in province",
        ));
    };
    if city_code == "000" {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province-level city (000) is not allowed",
        ));
    }
    Ok((province_name, city.to_string()))
}
