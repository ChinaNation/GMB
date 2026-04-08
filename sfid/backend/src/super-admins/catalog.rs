use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use crate::business::pubkey::{normalize_admin_pubkey, same_admin_pubkey};
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
            if user.role != AdminRole::InstitutionAdmin {
                return None;
            }
            Some(SuperAdminRow {
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
    if crate::sfid::province::province_code_by_name(province_name.as_str()).is_none() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province not found in code table",
        );
    }
    let new_pubkey = match normalize_admin_pubkey(input.admin_pubkey.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey format invalid"),
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let old_pubkey = store
        .super_admin_province_by_pubkey
        .iter()
        .find(|(pubkey, p)| {
            *p == &province_name
                && store
                    .admin_users_by_pubkey
                    .get(pubkey.as_str())
                    .map(|user| user.role == AdminRole::InstitutionAdmin)
                    .unwrap_or(false)
        })
        .map(|(k, _)| k.clone());
    let Some(old_pubkey) = old_pubkey else {
        return api_error(
            StatusCode::NOT_FOUND,
            1004,
            "province super admin not found",
        );
    };
    if same_admin_pubkey(old_pubkey.as_str(), new_pubkey.as_str()) {
        let Some(existing) = store.admin_users_by_pubkey.get(&old_pubkey) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "super admin not found");
        };
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: SuperAdminRow {
                id: existing.id,
                province: province_name.clone(),
                admin_pubkey: existing.admin_pubkey.clone(),
                admin_name: format!("{province_name}机构管理员"),
                built_in: existing.built_in,
                created_at: existing.created_at,
            },
        })
        .into_response();
    }
    let new_pubkey_exists = store.admin_users_by_pubkey.keys().any(|existing| {
        if same_admin_pubkey(existing.as_str(), old_pubkey.as_str()) {
            return false;
        }
        same_admin_pubkey(existing.as_str(), new_pubkey.as_str())
    });
    if new_pubkey_exists {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "new super admin pubkey already exists",
        );
    }
    let Some(old_user) = store.admin_users_by_pubkey.get(&old_pubkey).cloned() else {
        return api_error(StatusCode::NOT_FOUND, 1004, "super admin not found");
    };
    if old_user.role != AdminRole::InstitutionAdmin {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "target province role is not super admin",
        );
    }

    let preserved_status = old_user.status.clone();
    let replaced_at = Utc::now();
    store.admin_users_by_pubkey.remove(&old_pubkey);
    store.admin_users_by_pubkey.insert(
        new_pubkey.clone(),
        AdminUser {
            id: old_user.id,
            admin_pubkey: new_pubkey.clone(),
            admin_name: old_user.admin_name,
            role: AdminRole::InstitutionAdmin,
            status: preserved_status.clone(),
            built_in: old_user.built_in,
            created_by: old_user.created_by,
            created_at: old_user.created_at,
            updated_at: Some(replaced_at),
            city: String::new(),
        },
    );
    store.super_admin_province_by_pubkey.remove(&old_pubkey);
    store
        .super_admin_province_by_pubkey
        .insert(new_pubkey.clone(), province_name.clone());
    store.admin_sessions.retain(|_, session| {
        !same_admin_pubkey(session.admin_pubkey.as_str(), old_pubkey.as_str())
    });

    for operator in store.admin_users_by_pubkey.values_mut() {
        if operator.role == AdminRole::SystemAdmin
            && same_admin_pubkey(operator.created_by.as_str(), old_pubkey.as_str())
        {
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

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: SuperAdminRow {
            id: old_user.id,
            province: province_name.clone(),
            admin_pubkey: new_pubkey,
            admin_name: format!("{province_name}机构管理员"),
            built_in: old_user.built_in,
            created_at: old_user.created_at,
        },
    })
    .into_response()
}
