//! 公民人数快照凭证签发 handler。
//!
//! 节点端 `citizenchain/node/src/governance/sfid_api.rs::fetch_population_snapshot`
//! 在用户发起联合投票提案前调本接口,把签好的人口快照随 extrinsic 一起带上链。
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

use crate::chain::runtime_align::build_population_snapshot_credential;
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

    let eligible_total = {
        let store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        store
            .citizen_records
            .values()
            .filter(|r| r.archive_no.is_some())
            .count() as u64
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

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    append_audit_log_with_meta(
        &mut store,
        "APP_VOTERS_COUNT",
        "app",
        Some(who.clone()),
        None,
        None,
        actor_ip_from_headers(&headers),
        "SUCCESS",
        format!("eligible_total={eligible_total}"),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppVotersCountOutput {
            genesis_hash: snapshot.genesis_hash,
            eligible_total,
            who: snapshot.who,
            snapshot_nonce: snapshot.snapshot_nonce,
            signature: snapshot.signature,
        },
    })
    .into_response()
}
