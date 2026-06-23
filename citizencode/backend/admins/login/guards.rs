//! 登录会话鉴权守卫。
//!
//! 中文注释:会话、管理员身份和 Passkey 绑定状态只读取结构化表。
//! 业务模块通过 `require_admin_any`、`require_federal_registry` 获取认证上下文;
//! 写操作的 Passkey/公民钱包级别由 admins::actions 的安全 grant 单独校验。

use axum::http::{HeaderMap, StatusCode};
use chrono::{Duration, Utc};
use std::sync::atomic::{AtomicI64, Ordering};

use crate::admins::repo;
use crate::*;

use super::model::AdminAuthContext;
use super::signature::build_admin_name;

pub(super) fn admin_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let Some(token) = bearer_token(headers) else {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1002,
            "admin auth required",
        ));
    };

    let now = Utc::now();
    let city_idle_timeout_minutes = std::env::var("CID_ADMIN_IDLE_TIMEOUT_MINUTES")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(10);

    // 中文注释:清理动作节流到 60 秒一次,避免高频鉴权请求都执行批量删除。
    static LAST_CLEANUP: AtomicI64 = AtomicI64::new(0);
    let now_ts = now.timestamp();
    let last = LAST_CLEANUP.load(Ordering::Relaxed);
    let should_cleanup = now_ts - last > 60
        && LAST_CLEANUP
            .compare_exchange(last, now_ts, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok();

    let result = state.db.with_client(move |conn| {
        if should_cleanup {
            repo::cleanup_admin_sessions_conn(conn, now, city_idle_timeout_minutes)?;
        }
        let Some(mut session) = repo::get_admin_session_conn(conn, token.as_str())? else {
            return Err("http:unauthorized:invalid access token".to_string());
        };

        let idle_expired = session.registry_org_code == RegistryOrgCode::CityRegistry
            && now > session.last_active_at + Duration::minutes(city_idle_timeout_minutes);
        if now > session.expire_at || idle_expired {
            conn.execute("DELETE FROM admin_sessions WHERE token = $1", &[&token])
                .map_err(|e| format!("delete expired admin session failed: {e}"))?;
            return Err("http:unauthorized:access token expired".to_string());
        }

        session.last_active_at = now;
        repo::touch_admin_session_conn(conn, &session)?;

        let admin = repo::get_admin_by_account_conn(conn, &session.admin_account)?
            .ok_or_else(|| "http:forbidden:admin not found".to_string())?;
        let scope_province_name = repo::province_scope_for_registry_org_conn(
            conn,
            &admin.admin_account,
            &admin.registry_org_code,
        )?;
        let scope_city_name = if admin.registry_org_code == RegistryOrgCode::CityRegistry
            && !admin.city_name.trim().is_empty()
        {
            Some(admin.city_name.clone())
        } else {
            None
        };
        let admin_name = if admin.admin_name.trim().is_empty() {
            build_admin_name(
                &admin.admin_account,
                &admin.registry_org_code,
                scope_province_name.as_deref(),
            )
        } else {
            admin.admin_name.clone()
        };
        let passkey_bound = repo::admin_has_active_passkey_conn(conn, &admin.admin_account)?;
        let cid_short_name = repo::resolve_home_cid_short_name_conn(
            conn,
            &admin.registry_org_code,
            scope_province_name.as_deref(),
            scope_city_name.as_deref(),
        )?;

        Ok(AdminAuthContext {
            admin_account: admin.admin_account,
            registry_org_code: admin.registry_org_code,
            admin_name,
            scope_province_name,
            scope_city_name,
            passkey_bound,
            cid_short_name,
        })
    });

    match result {
        Ok(ctx) => Ok(ctx),
        Err(err) if err == "http:unauthorized:invalid access token" => Err(api_error(
            StatusCode::UNAUTHORIZED,
            1002,
            "invalid access token",
        )),
        Err(err) if err == "http:unauthorized:access token expired" => Err(api_error(
            StatusCode::UNAUTHORIZED,
            1002,
            "access token expired",
        )),
        Err(err) if err == "http:forbidden:admin not found" => {
            Err(api_error(StatusCode::FORBIDDEN, 2002, "admin not found"))
        }
        Err(err) => {
            let message = format!("admin auth failed: {err}");
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                message.as_str(),
            ))
        }
    }
}

pub(crate) fn require_admin_any(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    admin_auth(state, headers)
}

/// 中文注释:注册局治理与 CPMS 授权治理只允许 FederalRegistry。
pub(crate) fn require_federal_registry(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if ctx.registry_org_code != RegistryOrgCode::FederalRegistry {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "federal admin required",
        ));
    }
    if ctx.scope_province_name.is_none() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        ));
    }
    Ok(ctx)
}

pub(super) fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?.trim();
    let token = auth.strip_prefix("Bearer ")?;
    if token.trim().is_empty() {
        return None;
    }
    Some(token.trim().to_string())
}
