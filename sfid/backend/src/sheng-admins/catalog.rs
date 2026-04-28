use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use crate::business::pubkey::{normalize_admin_pubkey, same_admin_pubkey};
use crate::business::scope::province_scope_for_role;
use crate::*;

/// 三角色均可访问,按 scope 过滤:
/// - KEY_ADMIN:返回全部 43 省
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
    // scope:KEY_ADMIN → None(全局), SHENG/SHI → Some(province)
    let scope_province = province_scope_for_role(&store, &ctx.admin_pubkey, &ctx.role);
    let mut rows: Vec<ShengAdminRow> = store
        .sheng_admin_province_by_pubkey
        .iter()
        .filter_map(|(pubkey, province)| {
            // scope 过滤:非 KEY_ADMIN 只能看自己所属省
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

pub(crate) async fn replace_sheng_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(province): Path<String>,
    Json(input): Json<ReplaceShengAdminInput>,
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
        .sheng_admin_province_by_pubkey
        .iter()
        .find(|(pubkey, p)| {
            *p == &province_name
                && store
                    .admin_users_by_pubkey
                    .get(pubkey.as_str())
                    .map(|user| user.role == AdminRole::ShengAdmin)
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
            data: ShengAdminRow {
                id: existing.id,
                province: province_name.clone(),
                admin_pubkey: existing.admin_pubkey.clone(),
                admin_name: if existing.admin_name.is_empty() {
                    format!("{province_name}机构管理员")
                } else {
                    existing.admin_name.clone()
                },
                built_in: existing.built_in,
                created_at: existing.created_at,
                updated_at: existing.updated_at,
                signing_pubkey: existing.signing_pubkey.clone(),
                signing_created_at: existing.signing_created_at,
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
    if old_user.role != AdminRole::ShengAdmin {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "target province role is not super admin",
        );
    }

    let preserved_status = old_user.status.clone();
    let replaced_at = Utc::now();
    store.admin_users_by_pubkey.remove(&old_pubkey);
    // 新管理员姓名：优先使用请求体中的 admin_name，否则保留旧值
    let resolved_name = input
        .admin_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .unwrap_or(old_user.admin_name.clone());
    store.admin_users_by_pubkey.insert(
        new_pubkey.clone(),
        AdminUser {
            id: old_user.id,
            admin_pubkey: new_pubkey.clone(),
            admin_name: resolved_name.clone(),
            role: AdminRole::ShengAdmin,
            status: preserved_status.clone(),
            built_in: old_user.built_in,
            created_by: old_user.created_by,
            created_at: old_user.created_at,
            updated_at: Some(replaced_at),
            city: String::new(),
            encrypted_signing_privkey: None,
            signing_pubkey: None,
            signing_created_at: None,
        },
    );
    store.sheng_admin_province_by_pubkey.remove(&old_pubkey);
    store
        .sheng_admin_province_by_pubkey
        .insert(new_pubkey.clone(), province_name.clone());
    store.admin_sessions.retain(|_, session| {
        !same_admin_pubkey(session.admin_pubkey.as_str(), old_pubkey.as_str())
    });

    for operator in store.admin_users_by_pubkey.values_mut() {
        if operator.role == AdminRole::ShiAdmin
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

    // 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 9：
    // KEY_ADMIN 更换省登录管理员时，级联清理链上 signing pubkey + 内存 cache。
    // 新管理员首次登录时会重新 bootstrap 新的密钥对并再次推链。
    {
        use subxt::backend::legacy::LegacyRpcMethods;
        use subxt::{OnlineClient, PolkadotConfig};
        let ws_url_res = crate::chain::url::chain_ws_url();
        match ws_url_res {
            Ok(ws) => match OnlineClient::<PolkadotConfig>::from_insecure_url(ws.clone()).await {
                Ok(client) => {
                    match subxt::backend::rpc::RpcClient::from_insecure_url(ws.as_str()).await {
                        Ok(rpc) => {
                            let legacy = LegacyRpcMethods::<PolkadotConfig>::new(rpc);
                            let main_pair = state.sheng_signer_cache.sfid_main_signer();
                            if let Err(e) = crate::key_admins::chain_sheng_signing::submit_set_sheng_signing_pubkey_with_client(
                                    &client,
                                    &legacy,
                                    &main_pair,
                                    province_name.as_str(),
                                    None,
                                )
                                .await
                                {
                                    tracing::warn!(province = %province_name, error = %e, "clear sheng signing pubkey on chain failed");
                                }
                        }
                        Err(e) => {
                            tracing::warn!(province = %province_name, error = %e, "legacy rpc connect failed");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(province = %province_name, error = %e, "chain connect failed");
                }
            },
            Err(e) => {
                tracing::warn!(province = %province_name, error = %e, "resolve ws url failed");
            }
        }
    }
    // 清理新管理员残留的密文（一般是 None）+ 驱逐本省 cache。
    {
        if let Ok(mut store) = state.store.write() {
            if let Some(user) = store.admin_users_by_pubkey.get_mut(&new_pubkey) {
                user.encrypted_signing_privkey = None;
                user.signing_pubkey = None;
            }
        }
    }
    state
        .sheng_signer_cache
        .unload_province(province_name.as_str());

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: ShengAdminRow {
            id: old_user.id,
            province: province_name.clone(),
            admin_pubkey: new_pubkey,
            admin_name: resolved_name,
            built_in: old_user.built_in,
            created_at: old_user.created_at,
            updated_at: Some(replaced_at),
            // 更换后 signing pubkey 已清理，新管理员登录前为 None
            signing_pubkey: None,
            signing_created_at: None,
        },
    })
    .into_response()
}
