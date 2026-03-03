use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use crate::*;

pub(crate) async fn list_super_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_key_admin(&state, &headers) {
        return resp;
    }
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<SuperAdminRow> = store
        .super_admin_province_by_pubkey
        .iter()
        .filter_map(|(pubkey, province)| {
            let user = store.admin_users_by_pubkey.get(pubkey)?;
            if user.role != AdminRole::SuperAdmin {
                return None;
            }
            Some(SuperAdminRow {
                id: user.id,
                province: province.clone(),
                admin_pubkey: user.admin_pubkey.clone(),
                status: user.status.clone(),
                built_in: user.built_in,
                created_at: user.created_at,
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

pub(crate) async fn replace_super_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(province): Path<String>,
    Json(input): Json<ReplaceSuperAdminInput>,
) -> impl IntoResponse {
    let ctx = match require_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if province.trim().is_empty() || input.admin_pubkey.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province and admin_pubkey are required",
        );
    }

    let province_name = province.trim().to_string();
    let new_pubkey = input.admin_pubkey.trim().to_string();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let old_pubkey = store
        .super_admin_province_by_pubkey
        .iter()
        .find(|(_, p)| *p == &province_name)
        .map(|(k, _)| k.clone());
    let Some(old_pubkey) = old_pubkey else {
        return api_error(
            StatusCode::NOT_FOUND,
            1004,
            "province super admin not found",
        );
    };
    if old_pubkey == new_pubkey {
        let Some(existing) = store.admin_users_by_pubkey.get(&old_pubkey) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "super admin not found");
        };
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: SuperAdminRow {
                id: existing.id,
                province: province_name,
                admin_pubkey: existing.admin_pubkey.clone(),
                status: existing.status.clone(),
                built_in: existing.built_in,
                created_at: existing.created_at,
            },
        })
        .into_response();
    }
    if store.admin_users_by_pubkey.contains_key(&new_pubkey) {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "new super admin pubkey already exists",
        );
    }
    let Some(old_user) = store.admin_users_by_pubkey.get(&old_pubkey).cloned() else {
        return api_error(StatusCode::NOT_FOUND, 1004, "super admin not found");
    };
    if old_user.role != AdminRole::SuperAdmin {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "target province role is not super admin",
        );
    }

    store.admin_users_by_pubkey.remove(&old_pubkey);
    store.admin_users_by_pubkey.insert(
        new_pubkey.clone(),
        AdminUser {
            id: old_user.id,
            admin_pubkey: new_pubkey.clone(),
            admin_name: old_user.admin_name,
            role: AdminRole::SuperAdmin,
            status: AdminStatus::Active,
            built_in: true,
            created_by: "SYSTEM".to_string(),
            created_at: Utc::now(),
        },
    );
    store.super_admin_province_by_pubkey.remove(&old_pubkey);
    store
        .super_admin_province_by_pubkey
        .insert(new_pubkey.clone(), province_name.clone());

    for operator in store.admin_users_by_pubkey.values_mut() {
        if operator.role == AdminRole::OperatorAdmin && operator.created_by == old_pubkey {
            operator.created_by = new_pubkey.clone();
        }
    }

    append_audit_log(
        &mut store,
        "SUPER_ADMIN_REPLACE",
        &ctx.admin_pubkey,
        Some(new_pubkey.clone()),
        None,
        "SUCCESS",
        format!(
            "province={} old_pubkey={} new_pubkey={}",
            province_name, old_pubkey, new_pubkey
        ),
    );
    drop(store);
    persist_runtime_state(&state);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: SuperAdminRow {
            id: old_user.id,
            province: province_name,
            admin_pubkey: new_pubkey,
            status: AdminStatus::Active,
            built_in: true,
            created_at: Utc::now(),
        },
    })
    .into_response()
}
