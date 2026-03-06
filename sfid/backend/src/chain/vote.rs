use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::*;

pub(crate) async fn verify_vote_eligibility(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<VoteVerifyInput>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_vote_cache(&mut store, Utc::now());
    let fingerprint = request_fingerprint(&input);
    let chain_auth = match require_chain_request(&mut store, &headers, "vote_verify", &fingerprint)
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_chain_request_db(&state, "vote_verify", &chain_auth, &fingerprint) {
        return resp;
    }
    store.metrics.vote_verify_total += 1;
    let cache_key = vote_cache_key(&input.account_pubkey, input.proposal_id);
    if let Some(cache) = store.vote_verify_cache.get(&cache_key).cloned() {
        let iat = Utc::now().timestamp();
        let challenge = input
            .challenge
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let vote_token = if cache.has_vote_eligibility {
            cache.sfid_code.clone().map(|sfid_code| {
                make_signature_envelope(
                    &state,
                    &VotePayload {
                        kind: "vote",
                        version: "v1",
                        account_pubkey: input.account_pubkey.clone(),
                        sfid_code,
                        proposal_id: input.proposal_id,
                        challenge,
                        iat,
                        exp: iat + 60,
                        jti: Uuid::new_v4().to_string(),
                    },
                )
            })
        } else {
            None
        };
        append_audit_log_with_meta(
            &mut store,
            "CHAIN_VOTE_VERIFY",
            "chain",
            Some(input.account_pubkey.clone()),
            cache.archive_index.clone(),
            Some(chain_auth.request_id),
            actor_ip,
            "SUCCESS",
            "vote eligibility cache hit".to_string(),
        );
        record_chain_latency(&mut store, started_at);
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: VoteVerifyOutput {
                account_pubkey: input.account_pubkey,
                is_bound: cache.is_bound,
                has_vote_eligibility: cache.has_vote_eligibility,
                sfid_code: cache.sfid_code,
                vote_token,
                message: if cache.has_vote_eligibility {
                    "pubkey bound and vote eligible".to_string()
                } else if cache.is_bound {
                    "pubkey bound but not vote eligible".to_string()
                } else {
                    "pubkey not bound, no vote eligibility".to_string()
                },
            },
        })
        .into_response();
    }
    if let Some(binding) = store.bindings_by_pubkey.get(&input.account_pubkey) {
        let eligible = binding.citizen_status == CitizenStatus::Normal;
        let sfid_code = binding.sfid_code.clone();
        let archive_index = binding.archive_index.clone();
        let citizen_status = binding.citizen_status.clone();
        let iat = Utc::now().timestamp();
        let challenge = input
            .challenge
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let vote_token = if eligible {
            let vote_payload = VotePayload {
                kind: "vote",
                version: "v1",
                account_pubkey: input.account_pubkey.clone(),
                sfid_code: sfid_code.clone(),
                proposal_id: input.proposal_id,
                challenge,
                iat,
                exp: iat + 60,
                jti: Uuid::new_v4().to_string(),
            };
            Some(make_signature_envelope(&state, &vote_payload))
        } else {
            None
        };
        store.vote_verify_cache.insert(
            cache_key,
            VoteVerifyCacheEntry {
                account_pubkey: input.account_pubkey.clone(),
                proposal_id: input.proposal_id,
                is_bound: true,
                has_vote_eligibility: eligible,
                sfid_code: Some(sfid_code.clone()),
                archive_index: Some(archive_index.clone()),
                citizen_status: Some(citizen_status),
                cached_at: Utc::now(),
            },
        );
        append_audit_log_with_meta(
            &mut store,
            "CHAIN_VOTE_VERIFY",
            "chain",
            Some(input.account_pubkey.clone()),
            Some(archive_index),
            Some(chain_auth.request_id),
            actor_ip,
            "SUCCESS",
            format!("eligible={eligible}"),
        );
        record_chain_latency(&mut store, started_at);
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: VoteVerifyOutput {
                account_pubkey: input.account_pubkey,
                is_bound: true,
                has_vote_eligibility: eligible,
                sfid_code: Some(sfid_code),
                vote_token,
                message: if eligible {
                    "pubkey bound and vote eligible".to_string()
                } else {
                    "pubkey bound but not vote eligible".to_string()
                },
            },
        })
        .into_response()
    } else {
        store.vote_verify_cache.insert(
            cache_key,
            VoteVerifyCacheEntry {
                account_pubkey: input.account_pubkey.clone(),
                proposal_id: input.proposal_id,
                is_bound: false,
                has_vote_eligibility: false,
                sfid_code: None,
                archive_index: None,
                citizen_status: None,
                cached_at: Utc::now(),
            },
        );
        append_audit_log_with_meta(
            &mut store,
            "CHAIN_VOTE_VERIFY",
            "chain",
            Some(input.account_pubkey.clone()),
            None,
            Some(chain_auth.request_id),
            actor_ip,
            "SUCCESS",
            "eligible=false".to_string(),
        );
        record_chain_latency(&mut store, started_at);
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: VoteVerifyOutput {
                account_pubkey: input.account_pubkey,
                is_bound: false,
                has_vote_eligibility: false,
                sfid_code: None,
                vote_token: None,
                message: "pubkey not bound, no vote eligibility".to_string(),
            },
        })
        .into_response()
    }
}
