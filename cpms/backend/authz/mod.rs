//! # 鉴权模块 (authz)
//!
//! 提供 Cookie session 校验和角色检查。
//!
//! 中文注释：CPMS 只有 SUPER_ADMIN / OPERATOR_ADMIN 两级管理员；
//! SUPER_ADMIN 是上级角色，必须能执行所有档案业务操作。

use axum::{
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use sqlx::Row;

use crate::{
    common::{err, ApiError},
    AppState,
};

pub(crate) const SESSION_COOKIE_NAME: &str = "cpms_session";

#[derive(Clone)]
pub(crate) struct AuthContext {
    pub(crate) user_id: String,
    pub(crate) role: String,
}

pub(crate) async fn require_role(
    state: &AppState,
    headers: &HeaderMap,
    role: &str,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    let ctx = require_auth(state, headers).await?;
    if ctx.role != role {
        return Err(err(StatusCode::FORBIDDEN, 2008, "permission denied"));
    }
    Ok(ctx)
}

pub(crate) async fn require_any_role(
    state: &AppState,
    headers: &HeaderMap,
    roles: &[&str],
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    let ctx = require_auth(state, headers).await?;
    if !roles.iter().any(|role| ctx.role == *role) {
        return Err(err(StatusCode::FORBIDDEN, 2008, "permission denied"));
    }
    Ok(ctx)
}

pub(crate) async fn require_archive_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    require_any_role(state, headers, &["SUPER_ADMIN", "OPERATOR_ADMIN"]).await
}

pub(crate) async fn require_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    let token = session_token(headers)?;

    let row = sqlx::query(
        "SELECT s.user_id, a.role, s.expires_at
         FROM sessions s
         JOIN admin_users a ON a.user_id = s.user_id
         WHERE s.access_token = $1",
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query session failed",
        )
    })?;

    let Some(row) = row else {
        let _ = sqlx::query("DELETE FROM sessions WHERE access_token = $1")
            .bind(&token)
            .execute(&state.db)
            .await;
        return Err(err(StatusCode::UNAUTHORIZED, 2001, "invalid session"));
    };

    let role: String = row.get("role");
    let expires_at: i64 = row.get("expires_at");

    if role == "OPERATOR_ADMIN" {
        match crate::dangan::ensure_operator_annual_export_unlocked(state).await {
            Ok(()) => {}
            Err((status, body)) if status == StatusCode::LOCKED => {
                let _ = sqlx::query("DELETE FROM sessions WHERE access_token = $1")
                    .bind(&token)
                    .execute(&state.db)
                    .await;
                return Err((status, body));
            }
            Err(e) => return Err(e),
        }
    }

    if expires_at < Utc::now().timestamp() {
        // 中文注释：所有管理员都按角色使用滑动空闲期；超管 15 分钟，操作管理员 30 分钟。
        let _ = sqlx::query("DELETE FROM sessions WHERE access_token = $1")
            .bind(&token)
            .execute(&state.db)
            .await;
        return Err(err(StatusCode::UNAUTHORIZED, 2009, "session expired"));
    }

    let new_expires = (Utc::now()
        + chrono::Duration::seconds(crate::login::session_ttl_seconds(&role)))
    .timestamp();
    let _ = sqlx::query("UPDATE sessions SET expires_at = $1 WHERE access_token = $2")
        .bind(new_expires)
        .bind(&token)
        .execute(&state.db)
        .await;

    Ok(AuthContext {
        user_id: row.get("user_id"),
        role,
    })
}

pub(crate) fn session_token(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ApiError>)> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| {
            raw.split(';').find_map(|part| {
                let (name, value) = part.trim().split_once('=')?;
                (name == SESSION_COOKIE_NAME && !value.trim().is_empty())
                    .then(|| value.trim().to_string())
            })
        })
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "missing session cookie"))
}
