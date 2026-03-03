use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use crate::*;

#[derive(serde::Deserialize)]
pub(crate) struct ListQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

pub(crate) async fn list_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_super_or_key_admin(&state, &headers) {
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
        .filter(|u| u.role == AdminRole::OperatorAdmin)
        .filter(|u| ctx.role == AdminRole::KeyAdmin || u.created_by == ctx.admin_pubkey)
        .map(|u| OperatorRow {
            id: u.id,
            admin_pubkey: u.admin_pubkey.clone(),
            admin_name: u.admin_name.clone(),
            role: u.role.clone(),
            status: u.status.clone(),
            built_in: u.built_in,
            created_by: u.created_by.clone(),
            created_by_name: creator_display_name(&store, u.created_by.as_str()),
            created_at: u.created_at,
        })
        .collect();
    rows.sort_by(|a, b| b.id.cmp(&a.id));
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
        data: rows,
    })
    .into_response()
}

pub(crate) async fn create_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateOperatorInput>,
) -> impl IntoResponse {
    let ctx = match require_super_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.admin_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey is required");
    }
    let admin_pubkey = match normalize_admin_pubkey(input.admin_pubkey.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey format invalid"),
    };
    let admin_name = input.admin_name.trim().to_string();
    if admin_name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "admin_name is required");
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store.admin_users_by_pubkey.contains_key(&admin_pubkey) {
        return api_error(StatusCode::CONFLICT, 1005, "operator already exists");
    }
    let next_id = store
        .admin_users_by_pubkey
        .values()
        .map(|u| u.id)
        .max()
        .unwrap_or(0)
        + 1;
    let row = AdminUser {
        id: next_id,
        admin_pubkey: admin_pubkey.clone(),
        admin_name: admin_name.clone(),
        role: AdminRole::OperatorAdmin,
        status: AdminStatus::Active,
        built_in: false,
        created_by: ctx.admin_pubkey.clone(),
        created_at: Utc::now(),
    };
    store
        .admin_users_by_pubkey
        .insert(admin_pubkey, row.clone());
    append_audit_log(
        &mut store,
        "OPERATOR_CREATE",
        &ctx.admin_pubkey,
        Some(row.admin_pubkey.clone()),
        None,
        "SUCCESS",
        format!("operator_id={} created_by={}", row.id, row.created_by),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: OperatorRow {
            id: row.id,
            admin_pubkey: row.admin_pubkey,
            admin_name: row.admin_name,
            role: row.role,
            status: row.status,
            built_in: row.built_in,
            created_by: row.created_by,
            created_by_name: ctx.admin_name,
            created_at: row.created_at,
        },
    })
    .into_response()
}

pub(crate) async fn update_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(input): Json<UpdateOperatorInput>,
) -> impl IntoResponse {
    let ctx = match require_super_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let current_pubkey = store
        .admin_users_by_pubkey
        .values()
        .find(|u| u.id == id && u.role == AdminRole::OperatorAdmin)
        .map(|u| u.admin_pubkey.clone());
    let Some(current_pubkey) = current_pubkey else {
        return api_error(StatusCode::NOT_FOUND, 1004, "operator not found");
    };
    let mut operator = match store.admin_users_by_pubkey.get(&current_pubkey) {
        Some(v) => v.clone(),
        None => return api_error(StatusCode::NOT_FOUND, 1004, "operator not found"),
    };
    if ctx.role != AdminRole::KeyAdmin && operator.created_by != ctx.admin_pubkey {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other creator operators",
        );
    }
    let mut pubkey_changed = false;
    if let Some(new_pubkey) = input.admin_pubkey {
        if new_pubkey.trim().is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey is invalid");
        }
        let Some(normalized_pubkey) = normalize_admin_pubkey(new_pubkey.as_str()) else {
            return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey format invalid");
        };
        if normalized_pubkey != current_pubkey
            && store.admin_users_by_pubkey.contains_key(&normalized_pubkey)
        {
            return api_error(StatusCode::CONFLICT, 1005, "admin_pubkey already exists");
        }
        pubkey_changed = normalized_pubkey != current_pubkey;
        operator.admin_pubkey = normalized_pubkey;
    }
    let mut name_changed = false;
    if let Some(next_name) = input.admin_name {
        let name = next_name.trim();
        if name.is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "admin_name is invalid");
        }
        name_changed = operator.admin_name != name;
        operator.admin_name = name.to_string();
    }
    let response_row = OperatorRow {
        id: operator.id,
        admin_pubkey: operator.admin_pubkey.clone(),
        admin_name: operator.admin_name.clone(),
        role: operator.role.clone(),
        status: operator.status.clone(),
        built_in: operator.built_in,
        created_by: operator.created_by.clone(),
        created_by_name: creator_display_name(&store, operator.created_by.as_str()),
        created_at: operator.created_at,
    };
    store.admin_users_by_pubkey.remove(&current_pubkey);
    store
        .admin_users_by_pubkey
        .insert(operator.admin_pubkey.clone(), operator);
    append_audit_log(
        &mut store,
        "OPERATOR_UPDATE",
        &ctx.admin_pubkey,
        Some(response_row.admin_pubkey.clone()),
        None,
        "SUCCESS",
        format!(
            "operator_id={} old_pubkey={} new_pubkey={} pubkey_changed={} name_changed={}",
            response_row.id,
            current_pubkey,
            response_row.admin_pubkey,
            pubkey_changed,
            name_changed
        ),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: response_row,
    })
    .into_response()
}

pub(crate) async fn delete_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let ctx = match require_super_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let operator = store
        .admin_users_by_pubkey
        .values()
        .find(|u| u.id == id && u.role == AdminRole::OperatorAdmin)
        .cloned();
    let Some(operator) = operator else {
        return api_error(StatusCode::NOT_FOUND, 1004, "operator not found");
    };
    if ctx.role != AdminRole::KeyAdmin && operator.created_by != ctx.admin_pubkey {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other creator operators",
        );
    }
    store.admin_users_by_pubkey.remove(&operator.admin_pubkey);
    append_audit_log(
        &mut store,
        "OPERATOR_DELETE",
        &ctx.admin_pubkey,
        Some(operator.admin_pubkey),
        None,
        "SUCCESS",
        format!(
            "operator_id={} created_by={}",
            operator.id, operator.created_by
        ),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "deleted",
    })
    .into_response()
}

pub(crate) async fn update_operator_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(input): Json<UpdateOperatorStatusInput>,
) -> impl IntoResponse {
    let ctx = match require_super_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let pubkey = store
        .admin_users_by_pubkey
        .values()
        .find(|u| u.id == id && u.role == AdminRole::OperatorAdmin)
        .map(|u| u.admin_pubkey.clone());
    let Some(pubkey) = pubkey else {
        return api_error(StatusCode::NOT_FOUND, 1004, "operator not found");
    };
    let (
        operator_id,
        operator_pubkey,
        operator_admin_name,
        operator_role,
        operator_status,
        operator_built_in,
        operator_created_by,
        operator_created_at,
    ) = {
        let operator = match store.admin_users_by_pubkey.get_mut(&pubkey) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "operator not found"),
        };
        if ctx.role != AdminRole::KeyAdmin && operator.created_by != ctx.admin_pubkey {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot manage other creator operators",
            );
        }
        operator.status = input.status.clone();
        (
            operator.id,
            operator.admin_pubkey.clone(),
            operator.admin_name.clone(),
            operator.role.clone(),
            operator.status.clone(),
            operator.built_in,
            operator.created_by.clone(),
            operator.created_at,
        )
    };
    let response = OperatorRow {
        id: operator_id,
        admin_pubkey: operator_pubkey,
        admin_name: operator_admin_name,
        role: operator_role,
        status: operator_status,
        built_in: operator_built_in,
        created_by: operator_created_by.clone(),
        created_by_name: creator_display_name(&store, operator_created_by.as_str()),
        created_at: operator_created_at,
    };
    append_audit_log(
        &mut store,
        "OPERATOR_STATUS_UPDATE",
        &ctx.admin_pubkey,
        Some(response.admin_pubkey.clone()),
        None,
        "SUCCESS",
        format!("operator_id={} status={:?}", response.id, response.status),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: response,
    })
    .into_response()
}

fn normalize_admin_pubkey(input: &str) -> Option<String> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(input) {
        return Some(hex_pubkey);
    }
    if parse_sr25519_pubkey_bytes(input).is_some() {
        return Some(input.trim().to_string());
    }
    None
}

fn creator_display_name(store: &Store, creator_pubkey: &str) -> String {
    let Some(creator) = store.admin_users_by_pubkey.get(creator_pubkey) else {
        return creator_pubkey.to_string();
    };
    let province = if creator.role == AdminRole::SuperAdmin {
        store
            .super_admin_province_by_pubkey
            .get(creator_pubkey)
            .map(String::as_str)
    } else {
        None
    };
    build_admin_display_name(creator_pubkey, &creator.role, province)
}
