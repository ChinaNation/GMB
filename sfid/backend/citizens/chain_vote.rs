//! 公民投票凭证签发 handler。
//!
//! wuminapp 调本接口拿到凭证后,将凭证作为 vote() extrinsic 入参提交上链。
//! 链端 runtime 会用 SFID main 公钥重新验签 + 消费 vote_nonce 防重放。
//!
//! 无 token 鉴权:返回的凭证仅对请求者 `account_pubkey` 有效,链上还会再次验签,
//! 全局 rate limiter 已防滥用。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_core::chain_runtime::build_vote_credential;
use crate::*;

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

/// `POST /api/v1/app/vote/credential`
pub(crate) async fn app_vote_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AppVoteCredentialInput>,
) -> impl IntoResponse {
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
