use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};

use crate::*;

pub(crate) async fn admin_list_citizens(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CitizensQuery>,
) -> impl IntoResponse {
    let auth_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let keyword = query.keyword.unwrap_or_default().trim().to_lowercase();
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<CitizenRow> = Vec::new();

    if auth_ctx.role != AdminRole::QueryOnly {
        for pending in store.pending_by_pubkey.values() {
            if store
                .bindings_by_pubkey
                .contains_key(&pending.account_pubkey)
            {
                continue;
            }
            if !in_scope_pending(pending, auth_ctx.admin_province.as_deref()) {
                continue;
            }
            rows.push(CitizenRow {
                seq: pending.seq,
                account_pubkey: pending.account_pubkey.clone(),
                archive_index: None,
                sfid_code: store
                    .generated_sfid_by_pubkey
                    .get(&pending.account_pubkey)
                    .cloned(),
                citizen_status: None,
                is_bound: false,
            });
        }
    }

    for b in store.bindings_by_pubkey.values() {
        if !in_scope(b, auth_ctx.admin_province.as_deref()) {
            continue;
        }
        rows.push(CitizenRow {
            seq: b.seq,
            account_pubkey: b.account_pubkey.clone(),
            archive_index: Some(b.archive_index.clone()),
            sfid_code: Some(b.sfid_code.clone()),
            citizen_status: Some(b.citizen_status.clone()),
            is_bound: true,
        });
    }

    rows.sort_by_key(|r| r.seq);

    if auth_ctx.role == AdminRole::QueryOnly && keyword.is_empty() {
        rows.clear();
    }

    if !keyword.is_empty() {
        rows.retain(|r| {
            r.account_pubkey.to_lowercase().contains(&keyword)
                || r.archive_index
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
                || r.sfid_code
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
        });
    }
    let rows = rows
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}

pub(crate) async fn admin_query_by_pubkey(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(input): Query<AdminQueryInput>,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let pending = store
        .pending_by_pubkey
        .get(&input.account_pubkey)
        .filter(|p| in_scope_pending(p, admin_ctx.admin_province.as_deref()))
        .is_some();
    let binding = store
        .bindings_by_pubkey
        .get(&input.account_pubkey)
        .filter(|b| in_scope(b, admin_ctx.admin_province.as_deref()));

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQueryOutput {
            account_pubkey: input.account_pubkey,
            found_pending: pending,
            found_binding: binding.is_some(),
            archive_index: binding.map(|b| b.archive_index.clone()),
            sfid_code: binding.map(|b| b.sfid_code.clone()),
        },
    })
    .into_response()
}

pub(crate) async fn public_identity_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PublicIdentitySearchQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_public_search_auth(&headers) {
        return resp;
    }
    let archive_no = query.archive_no.as_deref().map(str::trim).unwrap_or("");
    let identity_code = query.identity_code.as_deref().map(str::trim).unwrap_or("");
    let account_pubkey = query.account_pubkey.as_deref().map(str::trim).unwrap_or("");
    if archive_no.is_empty() && identity_code.is_empty() && account_pubkey.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "archive_no or identity_code or account_pubkey is required",
        );
    }

    let actor_ip = actor_ip_from_headers(&headers);
    let request_id = request_id_from_headers(&headers);
    let found = {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if !account_pubkey.is_empty() {
            store.bindings_by_pubkey.get(account_pubkey).cloned()
        } else if !archive_no.is_empty() {
            store
                .bindings_by_pubkey
                .values()
                .find(|b| b.archive_index == archive_no)
                .cloned()
        } else {
            store
                .bindings_by_pubkey
                .values()
                .find(|b| b.sfid_code == identity_code)
                .cloned()
        }
    };
    let output = PublicIdentitySearchOutput {
        found: found.is_some(),
        archive_no: found.as_ref().map(|b| b.archive_index.clone()),
        identity_code: found.as_ref().map(|b| b.sfid_code.clone()),
        account_pubkey: found.as_ref().map(|b| b.account_pubkey.clone()),
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    append_audit_log_with_meta(
        &mut store,
        "PUBLIC_IDENTITY_SEARCH",
        "public",
        output.account_pubkey.clone(),
        output.archive_no.clone(),
        request_id,
        actor_ip,
        "SUCCESS",
        format!("found={}", output.found),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

pub(crate) async fn chain_voters_count(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let fingerprint = request_fingerprint(&"chain_voters_count");
    let chain_auth =
        match require_chain_request(&mut store, &headers, "chain_voters_count", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    if let Err(resp) =
        ensure_chain_request_db(&state, "chain_voters_count", &chain_auth, &fingerprint)
    {
        return resp;
    }
    store.metrics.voters_count_total += 1;
    let total_voters = store
        .bindings_by_pubkey
        .values()
        .filter(|b| b.citizen_status == CitizenStatus::Normal)
        .count();
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_VOTERS_COUNT",
        "chain",
        None,
        None,
        Some(chain_auth.request_id),
        actor_ip,
        "SUCCESS",
        format!("total_voters={total_voters}"),
    );
    record_chain_latency(&mut store, started_at);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: ChainVotersCountOutput {
            total_voters,
            as_of: Utc::now().timestamp(),
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
