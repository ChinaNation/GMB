//! App 专用 API 路由 handler。
//!
//! 提供给手机 App (wuminapp) 调用的接口，使用简单的 `x-app-token` 认证，
//! 核心签名逻辑复用 `runtime_align` 模块的函数。
//!
//! 与 chain 路由的区别：
//! - 认证方式：`x-app-token` header（简单 token），而非 chain auth（HMAC 签名）
//! - 无 nonce 防重放（人口快照和投票凭证自身已包含 nonce）
//! - 无请求追踪（`prepare_chain_request` / `track_chain_request`）

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::chain::runtime_align::{build_population_snapshot_credential, build_vote_credential};
use crate::*;

// ─── 人口快照 ─────────────────────────────────────────────

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

/// GET /api/v1/app/voters/count?who=<pubkey_hex>
pub(crate) async fn app_voters_count(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AppVotersCountQuery>,
) -> impl IntoResponse {
    // App 接口无需 token 认证：
    // 1. 返回的数据是 SFID 签名后的凭证，仅对请求者的账户有效
    // 2. 链上会验签，伪造无用
    // 3. 全局 rate limiter 已防滥用

    let who_raw = query.account_pubkey.or(query.who).unwrap_or_default();
    let Some(who) = normalize_account_pubkey(who_raw.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    };

    let eligible_total = {
        let store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let total = store
            .citizen_records
            .values()
            .filter(|r| r.archive_no.is_some())
            .count() as u64;
        total
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

// ─── 投票凭证 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct AppVoteCredentialInput {
    pub(crate) who: Option<String>,
    pub(crate) account_pubkey: Option<String>,
    pub(crate) proposal_id: u64,
}

#[derive(Serialize)]
struct AppVoteCredentialOutput {
    genesis_hash: String,
    who: String,
    binding_id: String,
    proposal_id: u64,
    vote_nonce: String,
    signature: String,
}

/// POST /api/v1/app/vote/credential
pub(crate) async fn app_vote_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AppVoteCredentialInput>,
) -> impl IntoResponse {
    // App 接口无需 token 认证：
    // 1. 返回的数据是 SFID 签名后的凭证，仅对请求者的账户有效
    // 2. 链上会验签，伪造无用
    // 3. 全局 rate limiter 已防滥用

    let who_raw = input.account_pubkey.or(input.who).unwrap_or_default();
    let Some(account_pubkey) = normalize_account_pubkey(who_raw.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    };
    let proposal_id = input.proposal_id;

    let (is_bound, has_vote_eligibility, binding_seed) = {
        let store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if let Some(cid) = store.citizen_id_by_pubkey.get(account_pubkey.as_str()) {
            if let Some(record) = store.citizen_records.get(cid) {
                let bound = record.archive_no.is_some();
                (bound, bound, record.archive_no.clone())
            } else {
                (false, false, None)
            }
        } else {
            (false, false, None)
        }
    };

    let credential = if has_vote_eligibility {
        let binding_seed = binding_seed.as_deref().unwrap_or("");
        match build_vote_credential(
            &state,
            &account_pubkey,
            binding_seed,
            proposal_id,
            Uuid::new_v4().to_string(),
        ) {
            Ok(cred) => cred,
            Err(message) => {
                let detail = format!("vote credential sign failed: {message}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
            }
        }
    } else if is_bound {
        return api_error(StatusCode::FORBIDDEN, 1003, "binding not vote eligible");
    } else {
        return api_error(StatusCode::NOT_FOUND, 1004, "binding not found");
    };

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    append_audit_log_with_meta(
        &mut store,
        "APP_VOTE_CREDENTIAL",
        "app",
        Some(account_pubkey.clone()),
        None,
        None,
        actor_ip_from_headers(&headers),
        if has_vote_eligibility {
            "SUCCESS"
        } else {
            "INELIGIBLE"
        },
        format!("proposal_id={proposal_id} eligible={has_vote_eligibility}"),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppVoteCredentialOutput {
            genesis_hash: credential.genesis_hash,
            who: credential.who,
            binding_id: credential.binding_id,
            proposal_id,
            vote_nonce: credential.vote_nonce,
            signature: credential.signature,
        },
    })
    .into_response()
}

// 中文注释:legacy app_bind_request 已删除(依赖 pending_by_pubkey)。
// 绑定流程走 citizen_bind_challenges 新模型。
