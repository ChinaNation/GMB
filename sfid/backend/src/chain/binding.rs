use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::chain::runtime_align::{
    build_bind_credential, compute_bind_credential_expiry_block, current_chain_block_number,
    RuntimeBindCredential,
};
use crate::*;

fn has_persisted_runtime_bind_credential(
    binding: &BindingRecord,
    state: &AppState,
    active_signer_pubkey: &str,
    current_block: u32,
) -> bool {
    let has_value = |v: &Option<String>| v.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
    let has_expected = |v: &Option<String>, expected: &str| {
        v.as_deref()
            .map(|s| !s.is_empty() && s == expected)
            .unwrap_or(false)
    };
    let has_unexpired = binding
        .runtime_bind_expires_at_block
        .map(|expires_at| expires_at >= current_block)
        .unwrap_or(false);
    has_value(&binding.runtime_bind_sfid_code_hash)
        && has_value(&binding.runtime_bind_nonce)
        && has_unexpired
        && has_value(&binding.runtime_bind_signature)
        && has_expected(&binding.runtime_bind_key_id, state.key_id.as_str())
        && has_expected(
            &binding.runtime_bind_key_version,
            state.key_version.as_str(),
        )
        && has_expected(&binding.runtime_bind_alg, state.key_alg.as_str())
        && has_expected(&binding.runtime_bind_signer_pubkey, active_signer_pubkey)
}

fn apply_runtime_bind_credential(
    binding: &mut BindingRecord,
    credential: RuntimeBindCredential,
    signer_pubkey: &str,
) {
    binding.runtime_bind_sfid_code_hash = Some(credential.sfid_code_hash);
    binding.runtime_bind_nonce = Some(credential.nonce);
    binding.runtime_bind_expires_at_block = Some(credential.expires_at_block);
    binding.runtime_bind_signature = Some(credential.signature);
    binding.runtime_bind_key_id = Some(credential.meta.key_id);
    binding.runtime_bind_key_version = Some(credential.meta.key_version);
    binding.runtime_bind_alg = Some(credential.meta.alg);
    binding.runtime_bind_signer_pubkey = Some(signer_pubkey.to_string());
}

fn bind_result_not_bound(account_pubkey: String) -> axum::response::Response {
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: BindResultOutput {
            account_pubkey,
            is_bound: false,
            sfid_code: None,
            sfid_code_hash: None,
            nonce: None,
            expires_at_block: None,
            signature: None,
            key_id: None,
            key_version: None,
            alg: None,
            sfid_signature: None,
            message: "not bound yet".to_string(),
        },
    })
    .into_response()
}

pub(crate) async fn create_bind_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<BindRequestInput>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    let mut input = input;
    let Some(account_pubkey) = normalize_account_pubkey(input.account_pubkey.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is invalid");
    };
    input.account_pubkey = account_pubkey;
    let callback_url =
        normalize_optional(input.callback_url.clone()).or_else(default_bind_callback_url);
    if let Some(url) = callback_url.as_ref() {
        if let Err(message) = validate_bind_callback_url(url) {
            return api_error(StatusCode::BAD_REQUEST, 1001, message.as_str());
        }
    }

    let fingerprint = request_fingerprint(&input);
    let chain_auth = match prepare_chain_request(&state, &headers, "bind_request", &fingerprint) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    store.metrics.bind_requests_total += 1;
    maybe_cleanup_pending_bind_requests(&mut store, Utc::now());
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
    insert_bounded_map(
        &mut store.pending_by_pubkey,
        input.account_pubkey.clone(),
        PendingRequest {
            seq,
            account_pubkey: input.account_pubkey.clone(),
            admin_province,
            requested_at: Utc::now(),
            callback_url,
            client_request_id: normalize_optional(input.client_request_id),
        },
        bounded_cache_limit("SFID_PENDING_BIND_REQUEST_CACHE_MAX", 50_000),
    );
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
    record_chain_latency(&mut store, started_at);

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
    let mut query = query;
    let Some(account_pubkey) = normalize_account_pubkey(query.account_pubkey.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is invalid");
    };
    query.account_pubkey = account_pubkey;
    let current_block = match current_chain_block_number().await {
        Ok(v) => v,
        Err(err) => {
            let detail = format!("resolve chain block failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
        }
    };
    let bind_credential_expires_at = compute_bind_credential_expiry_block(current_block);
    let active_signer_pubkey = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "active signer read lock poisoned",
            )
        }
    };

    let fingerprint = request_fingerprint(&query);
    let chain_auth = match prepare_chain_request(&state, &headers, "bind_result", &fingerprint) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
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
    let mut binding = store.bindings_by_pubkey.get(&query.account_pubkey).cloned();
    drop(store);
    if binding.is_none() {
        return bind_result_not_bound(query.account_pubkey);
    }

    if !binding
        .as_ref()
        .map(|v| {
            has_persisted_runtime_bind_credential(
                v,
                &state,
                active_signer_pubkey.as_str(),
                current_block,
            )
        })
        .unwrap_or(false)
    {
        let generated = match build_bind_credential(
            &state,
            query.account_pubkey.as_str(),
            binding
                .as_ref()
                .map(|v| v.sfid_code.as_str())
                .unwrap_or_default(),
            Uuid::new_v4().to_string(),
            bind_credential_expires_at,
        ) {
            Ok(v) => v,
            Err(message) => {
                let detail = format!("bind credential sign failed: {message}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
            }
        };
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        binding = if let Some(existing) = store.bindings_by_pubkey.get_mut(&query.account_pubkey) {
            if !has_persisted_runtime_bind_credential(
                existing,
                &state,
                active_signer_pubkey.as_str(),
                current_block,
            ) {
                apply_runtime_bind_credential(existing, generated, active_signer_pubkey.as_str());
            }
            Some(existing.clone())
        } else {
            None
        };
        drop(store);
        if binding.is_none() {
            return bind_result_not_bound(query.account_pubkey);
        }
    }

    let binding = binding.expect("checked is_some above");
    let Some(sfid_code_hash) = binding.runtime_bind_sfid_code_hash else {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "bind credential missing",
        );
    };
    let Some(nonce) = binding.runtime_bind_nonce else {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "bind credential missing",
        );
    };
    let Some(expires_at_block) = binding.runtime_bind_expires_at_block else {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "bind credential missing",
        );
    };
    let Some(signature) = binding.runtime_bind_signature else {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "bind credential missing",
        );
    };
    let Some(key_id) = binding.runtime_bind_key_id else {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "bind credential missing",
        );
    };
    let Some(key_version) = binding.runtime_bind_key_version else {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "bind credential missing",
        );
    };
    let Some(alg) = binding.runtime_bind_alg else {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "bind credential missing",
        );
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: BindResultOutput {
            account_pubkey: query.account_pubkey,
            is_bound: true,
            sfid_code: Some(binding.sfid_code),
            sfid_code_hash: Some(sfid_code_hash),
            nonce: Some(nonce),
            expires_at_block: Some(expires_at_block),
            signature: Some(signature.clone()),
            key_id: Some(key_id),
            key_version: Some(key_version),
            alg: Some(alg),
            sfid_signature: Some(binding.sfid_signature),
            message: "sfid bind success".to_string(),
        },
    })
    .into_response()
}

pub(crate) async fn chain_binding_validate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ChainBindingValidateInput>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    let mut input = input;
    let Some(account_pubkey) = normalize_account_pubkey(input.account_pubkey.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is invalid");
    };
    input.account_pubkey = account_pubkey;
    if input.archive_no.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "archive_no and account_pubkey are required",
        );
    }
    let fingerprint = request_fingerprint(&input);
    let chain_auth =
        match prepare_chain_request(&state, &headers, "chain_binding_validate", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
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
    let mut input = input;
    let Some(account_pubkey) = normalize_account_pubkey(input.account_pubkey.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is invalid");
    };
    input.account_pubkey = account_pubkey;
    if input.callback_id.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "account_pubkey and callback_id are required",
        );
    }
    let fingerprint = request_fingerprint(&input);
    let chain_auth = match prepare_chain_request(&state, &headers, "chain_reward_ack", &fingerprint)
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let (existing, next) = {
        let store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
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
        if matches!(
            existing.reward_status,
            RewardStatus::Rewarded | RewardStatus::Failed
        ) {
            return api_error(StatusCode::CONFLICT, 3008, "reward already finalized");
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
        (existing, next)
    };

    match persist_reward_state_db(&state, &next, Some(existing.updated_at)) {
        Ok(true) => {}
        Ok(false) => {
            return api_error(
                StatusCode::CONFLICT,
                3009,
                "reward state changed, retry request",
            );
        }
        Err(err) => {
            warn!(
                error = %err,
                account_pubkey = input.account_pubkey,
                callback_id = input.callback_id,
                "failed to persist reward state for chain ack"
            );
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1502,
                "reward state persistence failed",
            );
        }
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    store
        .reward_state_by_pubkey
        .insert(input.account_pubkey.clone(), next.clone());
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
    let output = RewardAckOutput {
        account_pubkey: next.account_pubkey.clone(),
        callback_id: next.callback_id.clone(),
        reward_status: next.reward_status.clone(),
        retry_count: next.retry_count,
        next_retry_at: next.next_retry_at.map(|v| v.timestamp()),
        message: "reward ack accepted".to_string(),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
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
    let mut query = query;
    let Some(account_pubkey) = normalize_account_pubkey(query.account_pubkey.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is invalid");
    };
    query.account_pubkey = account_pubkey;
    let fingerprint = request_fingerprint(&query);
    let chain_auth =
        match prepare_chain_request(&state, &headers, "chain_reward_state", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
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
