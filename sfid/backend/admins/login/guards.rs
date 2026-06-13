//! 登录会话鉴权守卫。
//!
//! 中文注释:会话、管理员身份和 Passkey 绑定状态只读取结构化表。
//! 业务模块通过 `require_admin_any`、`require_federal_admin` 获取认证上下文;
//! 写操作的 Passkey/公民钱包级别由 admins::actions 的安全 grant 单独校验。

use axum::http::{HeaderMap, StatusCode};
use chrono::{Duration, Utc};
use std::sync::atomic::{AtomicI64, Ordering};

use crate::admins::repo;
use crate::*;

use super::model::AdminAuthContext;
use super::signature::build_admin_display_name;

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
    let city_idle_timeout_minutes = std::env::var("SFID_ADMIN_IDLE_TIMEOUT_MINUTES")
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

        let idle_expired = session.role == AdminRole::CityAdmin
            && now > session.last_active_at + Duration::minutes(city_idle_timeout_minutes);
        if now > session.expire_at || idle_expired {
            conn.execute("DELETE FROM admin_sessions WHERE token = $1", &[&token])
                .map_err(|e| format!("delete expired admin session failed: {e}"))?;
            return Err("http:unauthorized:access token expired".to_string());
        }

        session.last_active_at = now;
        repo::touch_admin_session_conn(conn, &session)?;

        let admin = repo::get_admin_by_pubkey_conn(conn, &session.admin_pubkey)?
            .ok_or_else(|| "http:forbidden:admin not found".to_string())?;
        let admin_province =
            repo::province_scope_for_role_conn(conn, &admin.admin_pubkey, &admin.role)?;
        let admin_city = if admin.role == AdminRole::CityAdmin && !admin.city.trim().is_empty() {
            Some(admin.city.clone())
        } else {
            None
        };
        let admin_name = if admin.admin_name.trim().is_empty() {
            build_admin_display_name(&admin.admin_pubkey, &admin.role, admin_province.as_deref())
        } else {
            admin.admin_name.clone()
        };
        let passkey_bound = repo::admin_has_active_passkey_conn(conn, &admin.admin_pubkey)?;

        Ok(AdminAuthContext {
            admin_pubkey: admin.admin_pubkey,
            role: admin.role,
            admin_name,
            admin_province,
            admin_city,
            passkey_bound,
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

/// 中文注释:注册局治理与 CPMS 授权治理只允许 FederalAdmin。
pub(crate) fn require_federal_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if ctx.role != AdminRole::FederalAdmin {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "federal admin required",
        ));
    }
    if ctx.admin_province.is_none() {
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
