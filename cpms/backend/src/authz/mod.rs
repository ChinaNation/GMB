//! # 鉴权模块 (authz)
//!
//! 提供 Bearer token 校验和角色检查。所有需要登录的 API 通过 `require_auth` 或 `require_role` 守卫。

use axum::{
    http::{header, HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use sqlx::Row;

use crate::{err, ApiError, AppState};

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

pub(crate) async fn require_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    let token = bearer_token(headers)?;

    let row = sqlx::query("SELECT user_id, role, expires_at FROM sessions WHERE access_token = $1")
        .bind(token)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query session failed",
            )
        })?
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "invalid token"))?;

    let expires_at: i64 = row.get("expires_at");
    if expires_at < Utc::now().timestamp() {
        return Err(err(StatusCode::UNAUTHORIZED, 2009, "token expired"));
    }

    Ok(AuthContext {
        user_id: row.get("user_id"),
        role: row.get("role"),
    })
}

pub(crate) fn bearer_token(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ApiError>)> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "missing bearer token"))
}
