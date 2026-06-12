//! 审计日志 list handler(二角色只读)
//!
//! 中文注释:审计日志是独立后台能力,不属于权限范围规则本身,因此从
//! `scope` 目录迁到后端根层 `audit.rs`。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuditLogEntry {
    pub(crate) seq: u64,
    pub(crate) action: String,
    pub(crate) actor_pubkey: String,
    pub(crate) target_pubkey: Option<String>,
    pub(crate) target_archive_no: Option<String>,
    #[serde(default)]
    pub(crate) request_id: Option<String>,
    #[serde(default)]
    pub(crate) actor_ip: Option<String>,
    pub(crate) result: String,
    /// 结构化事实字段(JSON 对象,键小写蛇形,值为系统原值);人话翻译归前端渲染器。
    pub(crate) detail: serde_json::Value,
    pub(crate) created_at: DateTime<Utc>,
}

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
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let action = query.action.unwrap_or_default().trim().to_uppercase();
    let actor = query.actor_pubkey.unwrap_or_default().trim().to_string();
    let keyword = query.keyword.unwrap_or_default().trim().to_lowercase();
    let province_code = ctx
        .admin_province
        .as_deref()
        .and_then(crate::china::province_code_by_name);
    let city_code = match (ctx.admin_province.as_deref(), ctx.admin_city.as_deref()) {
        (Some(province), Some(city)) => crate::china::city_code_by_name(province, city),
        _ => None,
    };
    let result = state.db.with_client(move |conn| {
        let limit_i64 = i64::try_from(limit).map_err(|_| "limit too large".to_string())?;
        let rows = conn
            .query(
                "SELECT id, action, actor, target_sfid, detail, created_at
                 FROM audit
                 WHERE ($1::text IS NULL OR p_code = $1)
                   AND ($2::text IS NULL OR c_code = $2)
                   AND ($3::text = '' OR action = $3)
                   AND ($4::text = '' OR lower(actor) = lower($4))
                   AND (
                        $5::text = ''
                        OR lower(detail::text) LIKE '%' || $5 || '%'
                        OR lower(action) LIKE '%' || $5 || '%'
                        OR lower(actor) LIKE '%' || $5 || '%'
                        OR lower(COALESCE(target_sfid, '')) LIKE '%' || $5 || '%'
                   )
                 ORDER BY created_at DESC, id DESC
                 LIMIT $6",
                &[
                    &province_code,
                    &city_code,
                    &action,
                    &actor,
                    &keyword,
                    &limit_i64,
                ],
            )
            .map_err(|e| format!("query audit logs failed: {e}"))?;
        let mut output = Vec::with_capacity(rows.len());
        for row in rows {
            let seq: i64 = row.get(0);
            output.push(AuditLogEntry {
                seq: u64::try_from(seq).unwrap_or(0),
                action: row.get(1),
                actor_pubkey: row.get(2),
                target_pubkey: row.get(3),
                target_archive_no: None,
                request_id: None,
                actor_ip: None,
                result: "SUCCESS".to_string(),
                detail: row.get(4),
                created_at: row.get(5),
            });
        }
        Ok(output)
    });
    let rows = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query audit logs failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}
