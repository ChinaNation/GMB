use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};

use crate::*;

pub(crate) async fn create_bind_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<BindRequestInput>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }
    let callback_url =
        normalize_optional(input.callback_url.clone()).or_else(default_bind_callback_url);
    if let Some(url) = callback_url.as_ref() {
        if let Err(message) = validate_bind_callback_url(url) {
            return api_error(StatusCode::BAD_REQUEST, 1001, message.as_str());
        }
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let fingerprint = request_fingerprint(&input);
    let chain_auth = match require_chain_request(&mut store, &headers, "bind_request", &fingerprint)
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_chain_request_db(&state, "bind_request", &chain_auth, &fingerprint) {
        return resp;
    }
    store.metrics.bind_requests_total += 1;
    let seq = if let Some(existing) = store.pending_by_pubkey.get(&input.account_pubkey) {
        existing.seq
    } else if let Some(existing) = store.bindings_by_pubkey.get(&input.account_pubkey) {
        existing.seq
    } else {
        store.next_seq += 1;
        store.next_seq
    };
    let admin_province = store
        .pending_by_pubkey
        .get(&input.account_pubkey)
        .and_then(|p| p.admin_province.clone())
        .or_else(|| {
            store
                .bindings_by_pubkey
                .get(&input.account_pubkey)
                .and_then(|b| b.admin_province.clone())
        });
    store.pending_by_pubkey.insert(
        input.account_pubkey.clone(),
        PendingRequest {
            seq,
            account_pubkey: input.account_pubkey.clone(),
            admin_province,
            requested_at: Utc::now(),
            callback_url,
            client_request_id: normalize_optional(input.client_request_id),
        },
    );
    append_audit_log(
        &mut store,
        "CHAIN_BIND_REQUEST",
        "chain",
        Some(input.account_pubkey.clone()),
        None,
        "SUCCESS",
        format!("chain_request_id={}", chain_auth.request_id),
    );
    record_chain_latency(&mut store, started_at);
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_BIND_REQUEST_META",
        "chain",
        Some(input.account_pubkey.clone()),
        None,
        Some(chain_auth.request_id.clone()),
        actor_ip,
        "SUCCESS",
        "chain bind request accepted".to_string(),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: BindRequestOutput {
            account_pubkey: input.account_pubkey,
            chain_request_id: chain_auth.request_id,
            status: "WAITING_ADMIN",
            message: "binding request received",
        },
    })
    .into_response()
}

pub(crate) async fn get_bind_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BindResultQuery>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    if query.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let fingerprint = request_fingerprint(&query);
    let chain_auth = match require_chain_request(&mut store, &headers, "bind_result", &fingerprint)
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_chain_request_db(&state, "bind_result", &chain_auth, &fingerprint) {
        return resp;
    }
    append_audit_log(
        &mut store,
        "CHAIN_BIND_RESULT",
        "chain",
        Some(query.account_pubkey.clone()),
        None,
        "SUCCESS",
        format!("chain_request_id={}", chain_auth.request_id),
    );
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_BIND_RESULT_META",
        "chain",
        Some(query.account_pubkey.clone()),
        None,
        Some(chain_auth.request_id),
        actor_ip,
        "SUCCESS",
        "chain bind result queried".to_string(),
    );
    record_chain_latency(&mut store, started_at);
    if let Some(binding) = store.bindings_by_pubkey.get(&query.account_pubkey) {
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: BindResultOutput {
                account_pubkey: query.account_pubkey,
                is_bound: true,
                sfid_code: Some(binding.sfid_code.clone()),
                sfid_signature: Some(binding.sfid_signature.clone()),
                message: "sfid bind success".to_string(),
            },
        })
        .into_response()
    } else {
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: BindResultOutput {
                account_pubkey: query.account_pubkey,
                is_bound: false,
                sfid_code: None,
                sfid_signature: None,
                message: "not bound yet".to_string(),
            },
        })
        .into_response()
    }
}

pub(crate) async fn chain_binding_validate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ChainBindingValidateInput>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    if input.archive_no.trim().is_empty() || input.account_pubkey.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "archive_no and account_pubkey are required",
        );
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let fingerprint = request_fingerprint(&input);
    let chain_auth =
        match require_chain_request(&mut store, &headers, "chain_binding_validate", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    if let Err(resp) =
        ensure_chain_request_db(&state, "chain_binding_validate", &chain_auth, &fingerprint)
    {
        return resp;
    }
    store.metrics.binding_validate_total += 1;
    let matched = store
        .bindings_by_pubkey
        .get(&input.account_pubkey)
        .filter(|b| b.archive_index == input.archive_no);

    let is_bound = matched.is_some();
    let citizen_status = matched.map(|b| b.citizen_status.clone());
    let is_voting_eligible = matched
        .map(|b| b.citizen_status == CitizenStatus::Normal)
        .unwrap_or(false);
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_BINDING_VALIDATE",
        "chain",
        Some(input.account_pubkey.clone()),
        Some(input.archive_no.clone()),
        Some(chain_auth.request_id),
        actor_ip,
        "SUCCESS",
        format!("is_bound={is_bound}"),
    );
    record_chain_latency(&mut store, started_at);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: ChainBindingValidateOutput {
            is_bound,
            is_voting_eligible,
            citizen_status,
        },
    })
    .into_response()
}

pub(crate) async fn chain_reward_ack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RewardAckInput>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    if input.account_pubkey.trim().is_empty() || input.callback_id.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "account_pubkey and callback_id are required",
        );
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let fingerprint = request_fingerprint(&input);
    let chain_auth =
        match require_chain_request(&mut store, &headers, "chain_reward_ack", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    if let Err(resp) =
        ensure_chain_request_db(&state, "chain_reward_ack", &chain_auth, &fingerprint)
    {
        return resp;
    }
    let Some(existing) = store
        .reward_state_by_pubkey
        .get(&input.account_pubkey)
        .cloned()
    else {
        return api_error(StatusCode::NOT_FOUND, 3006, "reward state not found");
    };
    if existing.callback_id != input.callback_id {
        return api_error(StatusCode::CONFLICT, 3007, "callback_id mismatch");
    }

    let mut next = existing.clone();
    next.updated_at = Utc::now();
    match input.status {
        RewardAckStatusInput::Success => {
            next.reward_status = RewardStatus::Rewarded;
            next.reward_tx_hash = normalize_optional(input.reward_tx_hash);
            next.last_error = None;
            next.next_retry_at = None;
        }
        RewardAckStatusInput::Failed => {
            let retry_after = input.retry_after_seconds.unwrap_or(60).min(3600) as i64;
            next.retry_count += 1;
            next.reward_status = if next.retry_count >= next.max_retries {
                RewardStatus::Failed
            } else {
                RewardStatus::RetryWaiting
            };
            next.last_error = normalize_optional(input.error_message);
            next.next_retry_at = if next.reward_status == RewardStatus::RetryWaiting {
                Some(Utc::now() + Duration::seconds(retry_after))
            } else {
                None
            };
        }
    }
    store
        .reward_state_by_pubkey
        .insert(input.account_pubkey.clone(), next.clone());
    persist_reward_state_db(&state, &next);
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_REWARD_ACK",
        "chain",
        Some(input.account_pubkey.clone()),
        Some(next.archive_index.clone()),
        Some(chain_auth.request_id),
        actor_ip,
        "SUCCESS",
        format!(
            "callback_id={} reward_status={:?} retry_count={}",
            input.callback_id, next.reward_status, next.retry_count
        ),
    );
    record_chain_latency(&mut store, started_at);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: RewardAckOutput {
            account_pubkey: next.account_pubkey,
            callback_id: next.callback_id,
            reward_status: next.reward_status,
            retry_count: next.retry_count,
            next_retry_at: next.next_retry_at.map(|v| v.timestamp()),
            message: "reward ack accepted".to_string(),
        },
    })
    .into_response()
}

pub(crate) async fn chain_reward_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RewardStateQuery>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    if query.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let fingerprint = request_fingerprint(&query);
    let chain_auth =
        match require_chain_request(&mut store, &headers, "chain_reward_state", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    if let Err(resp) =
        ensure_chain_request_db(&state, "chain_reward_state", &chain_auth, &fingerprint)
    {
        return resp;
    }
    let Some(reward) = store
        .reward_state_by_pubkey
        .get(&query.account_pubkey)
        .cloned()
    else {
        return api_error(StatusCode::NOT_FOUND, 3006, "reward state not found");
    };
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_REWARD_STATE",
        "chain",
        Some(query.account_pubkey),
        Some(reward.archive_index.clone()),
        Some(chain_auth.request_id),
        actor_ip,
        "SUCCESS",
        format!("reward_status={:?}", reward.reward_status),
    );
    record_chain_latency(&mut store, started_at);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: RewardStateOutput {
            account_pubkey: reward.account_pubkey,
            archive_index: reward.archive_index,
            callback_id: reward.callback_id,
            reward_status: reward.reward_status,
            retry_count: reward.retry_count,
            max_retries: reward.max_retries,
            reward_tx_hash: reward.reward_tx_hash,
            last_error: reward.last_error,
            next_retry_at: reward.next_retry_at.map(|v| v.timestamp()),
            updated_at: reward.updated_at.timestamp(),
            created_at: reward.created_at.timestamp(),
        },
    })
    .into_response()
}
