use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use crate::business::pubkey::{normalize_admin_pubkey, same_admin_pubkey};
use crate::*;

const MAX_ADMIN_NAME_CHARS: usize = 200;

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
        .filter(|u| {
            can_manage_operator(
                &store,
                &ctx.role,
                ctx.admin_pubkey.as_str(),
                ctx.admin_province.as_deref(),
                u,
            )
        })
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
    if admin_name.chars().count() > MAX_ADMIN_NAME_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "admin_name too long");
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store
        .admin_users_by_pubkey
        .keys()
        .any(|existing| same_admin_pubkey(existing.as_str(), admin_pubkey.as_str()))
    {
        return api_error(StatusCode::CONFLICT, 1005, "operator already exists");
    }
    let next_id = allocate_next_admin_user_id(&mut store);
    let created_at = Utc::now();
    let row = AdminUser {
        id: next_id,
        admin_pubkey: admin_pubkey.clone(),
        admin_name: admin_name.clone(),
        role: AdminRole::OperatorAdmin,
        status: AdminStatus::Active,
        built_in: false,
        created_by: ctx.admin_pubkey.clone(),
        created_at,
        updated_at: Some(created_at),
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
    {
        let store = match store_read_or_500(&state) {
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
        let operator = match store.admin_users_by_pubkey.get(&current_pubkey).cloned() {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "operator not found"),
        };
        if !can_manage_operator(
            &store,
            &ctx.role,
            ctx.admin_pubkey.as_str(),
            ctx.admin_province.as_deref(),
            &operator,
        ) {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot manage other province operators",
            );
        }
    }

    let next_pubkey_input = if let Some(new_pubkey) = input.admin_pubkey {
        if new_pubkey.trim().is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey is invalid");
        }
        let Some(normalized_pubkey) = normalize_admin_pubkey(new_pubkey.as_str()) else {
            return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey format invalid");
        };
        Some(normalized_pubkey)
    } else {
        None
    };
    let next_name_input = if let Some(next_name) = input.admin_name {
        let name = next_name.trim();
        if name.is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "admin_name is invalid");
        }
        if name.chars().count() > MAX_ADMIN_NAME_CHARS {
            return api_error(StatusCode::BAD_REQUEST, 1001, "admin_name too long");
        }
        Some(name.to_string())
    } else {
        None
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
    let mut operator = match store.admin_users_by_pubkey.get(&current_pubkey).cloned() {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "operator not found"),
    };
    if !can_manage_operator(
        &store,
        &ctx.role,
        ctx.admin_pubkey.as_str(),
        ctx.admin_province.as_deref(),
        &operator,
    ) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province operators",
        );
    }
    let mut pubkey_changed = false;
    if let Some(normalized_pubkey) = next_pubkey_input {
        let pubkey_conflict = store.admin_users_by_pubkey.keys().any(|existing| {
            if same_admin_pubkey(existing.as_str(), current_pubkey.as_str()) {
                return false;
            }
            same_admin_pubkey(existing.as_str(), normalized_pubkey.as_str())
        });
        if pubkey_conflict {
            return api_error(StatusCode::CONFLICT, 1005, "admin_pubkey already exists");
        }
        pubkey_changed = normalized_pubkey != current_pubkey;
        operator.admin_pubkey = normalized_pubkey;
    }
    let mut name_changed = false;
    if let Some(name) = next_name_input {
        name_changed = operator.admin_name != name;
        operator.admin_name = name;
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
    if !pubkey_changed && !name_changed {
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: response_row,
        })
        .into_response();
    }
    operator.updated_at = Some(Utc::now());
    store.admin_users_by_pubkey.remove(&current_pubkey);
    store
        .admin_users_by_pubkey
        .insert(operator.admin_pubkey.clone(), operator);
    if pubkey_changed {
        store.admin_sessions.retain(|_, session| {
            !same_admin_pubkey(session.admin_pubkey.as_str(), current_pubkey.as_str())
        });
    }
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
    {
        let store = match store_read_or_500(&state) {
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
        if !can_manage_operator(
            &store,
            &ctx.role,
            ctx.admin_pubkey.as_str(),
            ctx.admin_province.as_deref(),
            &operator,
        ) {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot manage other province operators",
            );
        }
    }
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
    if !can_manage_operator(
        &store,
        &ctx.role,
        ctx.admin_pubkey.as_str(),
        ctx.admin_province.as_deref(),
        &operator,
    ) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province operators",
        );
    }
    let operator_pubkey = operator.admin_pubkey.clone();
    store.admin_users_by_pubkey.remove(&operator_pubkey);
    store.admin_sessions.retain(|_, session| {
        !same_admin_pubkey(session.admin_pubkey.as_str(), operator_pubkey.as_str())
    });
    append_audit_log(
        &mut store,
        "OPERATOR_DELETE",
        &ctx.admin_pubkey,
        Some(operator_pubkey),
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
    let operator = match store.admin_users_by_pubkey.get(&pubkey).cloned() {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "operator not found"),
    };
    if !can_manage_operator(
        &store,
        &ctx.role,
        ctx.admin_pubkey.as_str(),
        ctx.admin_province.as_deref(),
        &operator,
    ) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province operators",
        );
    }
    let status_changed = operator.status != input.status;
    let operator_status = if status_changed {
        let operator_mut = match store.admin_users_by_pubkey.get_mut(&pubkey) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "operator not found"),
        };
        operator_mut.status = input.status.clone();
        operator_mut.updated_at = Some(Utc::now());
        operator_mut.status.clone()
    } else {
        operator.status.clone()
    };
    let response = OperatorRow {
        id: operator.id,
        admin_pubkey: operator.admin_pubkey.clone(),
        admin_name: operator.admin_name.clone(),
        role: operator.role.clone(),
        status: operator_status,
        built_in: operator.built_in,
        created_by: operator.created_by.clone(),
        created_by_name: creator_display_name(&store, operator.created_by.as_str()),
        created_at: operator.created_at,
    };
    if !status_changed {
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: response,
        })
        .into_response();
    }
    if response.status == AdminStatus::Disabled {
        let target_pubkey = response.admin_pubkey.clone();
        store.admin_sessions.retain(|_, session| {
            !same_admin_pubkey(session.admin_pubkey.as_str(), target_pubkey.as_str())
        });
    }
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

fn allocate_next_admin_user_id(store: &mut Store) -> u64 {
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

fn can_manage_operator(
    store: &Store,
    actor_role: &AdminRole,
    actor_pubkey: &str,
    actor_province: Option<&str>,
    operator: &AdminUser,
) -> bool {
    if *actor_role == AdminRole::KeyAdmin {
        return true;
    }
    if same_admin_pubkey(operator.created_by.as_str(), actor_pubkey) {
        return true;
    }
    let Some(scope) = actor_province else {
        return false;
    };
    crate::business::scope::province_scope_for_role(store, &operator.admin_pubkey, &operator.role)
        .map(|operator_scope| operator_scope == scope)
        .unwrap_or(false)
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
