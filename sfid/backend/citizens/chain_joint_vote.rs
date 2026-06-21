//! 公民人数快照凭证签发 handler。
//!
//! 本接口只服务投票引擎的人口快照凭证流程,业务模块不得直接调用或转发。
//!
//! 无 token 鉴权:返回的凭证仅对请求者 `account_pubkey` 有效,链上还会再次验签,
//! 全局 rate limiter 已防滥用。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::chain_runtime::{build_population_snapshot_credential, normalize_account_pubkey};
use crate::*;

#[derive(Deserialize)]
pub(crate) struct AppVotersCountQuery {
    pub(crate) who: Option<String>,
    pub(crate) account_pubkey: Option<String>,
}

#[derive(Serialize)]
struct AppVotersCountOutput {
    genesis_hash: String,
    eligible_total: u64,
    who: String,
    snapshot_nonce: String,
    issuer_sfid_number: String,
    issuer_main_account: String,
    signer_pubkey: String,
    scope_province_name: String,
    scope_city_name: String,
    signature: String,
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
                 WHERE bind_status = 'BOUND'
                   AND citizen_status = 'NORMAL'
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

    let snapshot = match build_population_snapshot_credential(
        &state,
        who.as_str(),
        eligible_total,
        Uuid::new_v4().to_string(),
    ) {
        Ok(v) => v,
        Err(message) => {
            let detail = format!("snapshot signature sign failed: {message}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
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
            genesis_hash: snapshot.genesis_hash,
            eligible_total,
            who: snapshot.who,
            snapshot_nonce: snapshot.snapshot_nonce,
            issuer_sfid_number: snapshot.issuer_sfid_number,
            issuer_main_account: snapshot.issuer_main_account,
            signer_pubkey: snapshot.signer_pubkey,
            scope_province_name: snapshot.scope_province_name,
            scope_city_name: snapshot.scope_city_name,
            signature: snapshot.signature,
        },
    })
    .into_response()
}
