//! 审计日志 list handler(二角色只读)
//!
//! 中文注释:审计日志是独立后台能力,不属于权限范围规则本身,因此从
//! `scope` 目录迁到后端根层 `audit.rs`。

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub(crate) struct AuditLogsQuery {
    pub(crate) action: Option<String>,
    pub(crate) actor_pubkey: Option<String>,
    pub(crate) keyword: Option<String>,
    pub(crate) limit: Option<usize>,
}

/// ShengAdmin / ShiAdmin 均可访问审计日志(只读)。
pub(crate) async fn admin_list_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditLogsQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let action = query.action.unwrap_or_default().trim().to_uppercase();
    let actor = query.actor_pubkey.unwrap_or_default().trim().to_string();
    let keyword = query.keyword.unwrap_or_default().trim().to_lowercase();

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<AuditLogEntry> = store
        .audit_logs
        .iter()
        .filter(|e| action.is_empty() || e.action == action)
        .filter(|e| actor.is_empty() || e.actor_pubkey == actor)
        .filter(|e| {
            if keyword.is_empty() {
                return true;
            }
            e.detail.to_lowercase().contains(&keyword)
                || e.action.to_lowercase().contains(&keyword)
                || e.actor_pubkey.to_lowercase().contains(&keyword)
                || e.target_pubkey
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
                || e.target_archive_no
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
        })
        .cloned()
        .collect();

    rows.sort_by(|a, b| b.seq.cmp(&a.seq));
    rows.truncate(limit);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}
