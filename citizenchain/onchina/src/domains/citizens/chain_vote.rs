//! 公民投票凭证签发 handler。
//!
//! CitizenApp 调本接口拿到凭证后,将凭证作为 vote() extrinsic 入参提交上链。
//! 链端 runtime 会按签发机构主账户的 admins 真源确认 signer_pubkey,并消费 vote_nonce 防重放。
//!
//! 无 token 鉴权:返回的凭证仅对请求者 `account_pubkey` 有效,链上还会再次验签,
//! 全局 rate limiter 已防滥用。

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::chain_runtime::{
    build_vote_credential, is_chain_runtime_config_error, normalize_account_pubkey,
};
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
    issuer_cid_number: String,
    issuer_main_account: String,
    signer_pubkey: String,
    scope_province_name: String,
    scope_city_name: String,
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

    let (is_bound, has_vote_eligibility, binding_seed) =
        match state.db.find_bound_citizen_by_wallet(&account_pubkey) {
            Ok(Some(record)) => {
                let bound = record.bind_status() == CitizenBindStatus::Bound;
                (
                    bound,
                    bound && record.computed_vote_status() == CitizenStatus::Normal,
                    record.cid_number.clone(),
                )
            }
            Ok(None) => (false, false, None),
            Err(err) => {
                tracing::error!(error = %err, "query vote binding failed");
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "binding query failed",
                );
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
                if is_chain_runtime_config_error(message.as_str()) {
                    let detail = format!("链端签发配置未完成: {message}");
                    return api_error(StatusCode::SERVICE_UNAVAILABLE, 1006, detail.as_str());
                }
                let detail = format!("vote credential sign failed: {message}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
            }
        }
    } else if is_bound {
        return api_error(StatusCode::FORBIDDEN, 1003, "binding not vote eligible");
    } else {
        return api_error(StatusCode::NOT_FOUND, 1004, "binding not found");
    };

    crate::core::runtime_ops::append_audit_log(
        &state,
        "APP_VOTE_CREDENTIAL",
        "app",
        Some(account_pubkey.clone()),
        serde_json::json!({
            "proposal_id": proposal_id,
            "eligible": has_vote_eligibility,
            "actor_ip": actor_ip_from_headers(&headers),
        }),
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
            issuer_cid_number: credential.issuer_cid_number,
            issuer_main_account: credential.issuer_main_account,
            signer_pubkey: credential.signer_pubkey,
            scope_province_name: credential.scope_province_name,
            scope_city_name: credential.scope_city_name,
            signature: credential.signature,
        },
    })
    .into_response()
}
