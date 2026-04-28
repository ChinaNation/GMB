use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use tracing::warn;

use crate::*;

// 中文注释:legacy create_bind_request / get_bind_result 已删除(依赖 pending_by_pubkey / bindings_by_pubkey)。
// 绑定流程走 citizen_bind_challenges 新模型。
// has_persisted_runtime_bind_credential / apply_runtime_bind_credential 也随 BindingRecord 一起删除。

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
    let fingerprint = match request_fingerprint(&input) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
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

    // 中文注释:从 citizen_records 查询绑定状态(取代旧 bindings_by_pubkey)。
    let matched = store
        .citizen_id_by_pubkey
        .get(&input.account_pubkey)
        .and_then(|cid| store.citizen_records.get(cid))
        .filter(|r| r.archive_no.as_deref() == Some(input.archive_no.as_str()));

    let is_bound = matched.is_some();
    let citizen_status = if is_bound {
        Some(CitizenStatus::Normal)
    } else {
        None
    };
    let is_voting_eligible = is_bound;
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
    let fingerprint = match request_fingerprint(&input) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
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
    let fingerprint = match request_fingerprint(&query) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
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
