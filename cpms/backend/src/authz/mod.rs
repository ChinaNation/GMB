use axum::{
    http::{header, HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;

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

    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&token)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "invalid token"))?;
    if session.expires_at < Utc::now().timestamp() {
        return Err(err(StatusCode::UNAUTHORIZED, 2009, "token expired"));
    }

    Ok(AuthContext {
        user_id: session.user_id.clone(),
        role: session.role.clone(),
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
