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
        .bind(&token)
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

    let role: String = row.get("role");
    let expires_at: i64 = row.get("expires_at");

    // 超管 session 不过期；普通管理员检查过期 + 滑动续期 30 分钟
    if role != "SUPER_ADMIN" {
        if expires_at < Utc::now().timestamp() {
            // 过期则删除 session，强制重新登录
            let _ = sqlx::query("DELETE FROM sessions WHERE access_token = $1")
                .bind(&token)
                .execute(&state.db)
                .await;
            return Err(err(StatusCode::UNAUTHORIZED, 2009, "token expired"));
        }
        // 滑动续期：每次请求刷新过期时间
        let new_expires = (Utc::now() + chrono::Duration::seconds(
            crate::login::TOKEN_EXPIRES_SECONDS,
        )).timestamp();
        let _ = sqlx::query("UPDATE sessions SET expires_at = $1 WHERE access_token = $2")
            .bind(new_expires)
            .bind(&token)
            .execute(&state.db)
            .await;
    }

    Ok(AuthContext {
        user_id: row.get("user_id"),
        role,
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
