use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use crate::business::pubkey::{normalize_admin_pubkey, same_admin_pubkey};
use crate::business::scope::province_scope_for_role;
use crate::sfid::province::city_code_by_name;
use crate::*;

const MAX_ADMIN_NAME_CHARS: usize = 200;

pub(crate) async fn list_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
            city: u.city.clone(),
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
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
    // ── 解析 created_by ──
    // - ShengAdmin 调用：created_by 必须为空或等于自身
    // - KeyAdmin 调用：created_by 可指定为任意已存在的 ShengAdmin pubkey
    let created_by_pubkey = match input.created_by.as_deref().map(str::trim) {
        None | Some("") => ctx.admin_pubkey.clone(),
        Some(raw) => {
            let normalized = match normalize_admin_pubkey(raw) {
                Some(v) => v,
                None => {
                    return api_error(StatusCode::BAD_REQUEST, 1001, "created_by format invalid")
                }
            };
            match ctx.role {
                AdminRole::KeyAdmin => {
                    let creator = store
                        .admin_users_by_pubkey
                        .iter()
                        .find(|(k, _)| same_admin_pubkey(k.as_str(), normalized.as_str()))
                        .map(|(_, v)| v.clone());
                    match creator {
                        Some(u) if u.role == AdminRole::ShengAdmin => normalized,
                        Some(_) => {
                            return api_error(
                                StatusCode::BAD_REQUEST,
                                1001,
                                "created_by must be an ShengAdmin",
                            )
                        }
                        None => {
                            return api_error(
                                StatusCode::NOT_FOUND,
                                1004,
                                "created_by ShengAdmin not found",
                            )
                        }
                    }
                }
                AdminRole::ShengAdmin => {
                    if !same_admin_pubkey(normalized.as_str(), ctx.admin_pubkey.as_str()) {
                        return api_error(
                            StatusCode::FORBIDDEN,
                            1003,
                            "ShengAdmin can only create operators under itself",
                        );
                    }
                    normalized
                }
                AdminRole::ShiAdmin => {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        1003,
                        "ShiAdmin cannot create operators",
                    )
                }
            }
        }
    };
    if store
        .admin_users_by_pubkey
        .keys()
        .any(|existing| same_admin_pubkey(existing.as_str(), admin_pubkey.as_str()))
    {
        return api_error(StatusCode::CONFLICT, 1005, "operator already exists");
    }
    // ── 校验 city：必须属于 created_by 对应机构管理员的省份，且不可为省辖市 ──
    let city_input = input.city.trim().to_string();
    if city_input.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city is required");
    }
    let scope_province = province_scope_for_role(
        &store,
        created_by_pubkey.as_str(),
        &AdminRole::ShengAdmin,
    );
    let province_name = match scope_province {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from created_by",
            )
        }
    };
    let city_code = match city_code_by_name(province_name.as_str(), city_input.as_str()) {
        Some(c) => c,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "city not found in province",
            )
        }
    };
    if city_code == "000" {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province-level city (000) is not allowed",
        );
    }
    let next_id = allocate_next_admin_user_id(&mut store);
    let created_at = Utc::now();
    let row = AdminUser {
        id: next_id,
        admin_pubkey: admin_pubkey.clone(),
        admin_name: admin_name.clone(),
        role: AdminRole::ShiAdmin,
        status: AdminStatus::Active,
        built_in: false,
        created_by: created_by_pubkey,
        created_at,
        updated_at: Some(created_at),
        city: city_input,
        encrypted_signing_privkey: None,
        signing_pubkey: None,
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
    let created_by_display = creator_display_name(&store, row.created_by.as_str());
    drop(store);
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
            created_by_name: created_by_display,
            created_at: row.created_at,
            city: row.city,
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
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
            .find(|u| u.id == id && u.role == AdminRole::ShiAdmin)
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
    let next_city_input = if let Some(next_city) = input.city {
        let city = next_city.trim();
        if city.is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "city is invalid");
        }
        Some(city.to_string())
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
        .find(|u| u.id == id && u.role == AdminRole::ShiAdmin)
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
    let mut city_changed = false;
    if let Some(city) = next_city_input {
        // 校验：必须属于该 operator 所属机构管理员的省份，且不可为省辖市
        let scope_province = province_scope_for_role(
            &store,
            operator.created_by.as_str(),
            &AdminRole::ShengAdmin,
        );
        let province_name = match scope_province {
            Some(v) => v,
            None => {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "cannot resolve province from operator created_by",
                )
            }
        };
        let city_code = match city_code_by_name(province_name.as_str(), city.as_str()) {
            Some(c) => c,
            None => {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "city not found in province",
                )
            }
        };
        if city_code == "000" {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "province-level city (000) is not allowed",
            );
        }
        city_changed = operator.city != city;
        operator.city = city;
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
        city: operator.city.clone(),
    };
    if !pubkey_changed && !name_changed && !city_changed {
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
            "operator_id={} old_pubkey={} new_pubkey={} pubkey_changed={} name_changed={} city_changed={}",
            response_row.id,
            current_pubkey,
            response_row.admin_pubkey,
            pubkey_changed,
            name_changed,
            city_changed
        ),
    );
    drop(store);
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
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
            .find(|u| u.id == id && u.role == AdminRole::ShiAdmin)
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
        .find(|u| u.id == id && u.role == AdminRole::ShiAdmin)
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
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
        .find(|u| u.id == id && u.role == AdminRole::ShiAdmin)
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
        city: operator.city.clone(),
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
