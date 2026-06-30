//! 公民人数查询 handler。
//!
//! 本接口只返回 OnChina 本地公民档案统计,用于 CitizenApp 展示或诊断。
//! 链端联合投票人口快照由 runtime 从 `citizen-identity` 链上状态按 scope 读取。
//!
//! 无 token 鉴权:只返回聚合人数,不包含个人档案字段。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::core::chain_runtime::normalize_account_pubkey;
use crate::*;

#[derive(Deserialize)]
pub(crate) struct AppVotersCountQuery {
    pub(crate) who: Option<String>,
    pub(crate) account_pubkey: Option<String>,
}

#[derive(Serialize)]
struct AppVotersCountOutput {
    eligible_total: u64,
    who: String,
}

/// `GET /api/v1/app/voters/count?account_pubkey=<hex>`
pub(crate) async fn app_voters_count(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AppVotersCountQuery>,
) -> impl IntoResponse {
    let who_raw = query.account_pubkey.or(query.who).unwrap_or_default();
    let Some(who) = normalize_account_pubkey(who_raw.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    };

    let eligible_total = match state.db.with_client(|conn| {
        let row = conn
            .query_one(
                "SELECT COUNT(*)::BIGINT
                 FROM citizens
                 WHERE citizen_status = 'NORMAL'
                   AND voting_eligible = true",
                &[],
            )
            .map_err(|e| format!("query eligible voters failed: {e}"))?;
        let total: i64 = row.get(0);
        Ok(u64::try_from(total).unwrap_or(0))
    }) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query voters count failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "voters query failed",
            );
        }
    };

    crate::core::runtime_ops::append_audit_log(
        &state,
        "APP_VOTERS_COUNT",
        "app",
        Some(who.clone()),
        serde_json::json!({
            "eligible_total": eligible_total,
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppVotersCountOutput {
            eligible_total,
            who,
        },
    })
    .into_response()
}
