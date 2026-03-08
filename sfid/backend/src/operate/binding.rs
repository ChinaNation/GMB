use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::chain::runtime_align::{
    build_bind_credential, compute_bind_credential_expiry_block, current_chain_block_number,
};
use crate::*;

pub(crate) async fn admin_bind_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<BindScanInput>,
) -> impl IntoResponse {
    let actor_ip = actor_ip_from_headers(&headers);
    let request_id = request_id_from_headers(&headers);
    let admin_ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.qr_payload.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "qr_payload is required");
    }
    let payload: CitizenQrPayload = match serde_json::from_str(input.qr_payload.trim()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid citizen qr_payload"),
    };
    if payload.ver != "1" || payload.issuer_id != "cpms" || payload.sig_alg != "sr25519" {
        return api_error(StatusCode::UNAUTHORIZED, 1006, "qr header invalid");
    }
    if payload.archive_no.trim().is_empty()
        || payload.qr_id.trim().is_empty()
        || payload.site_sfid.trim().is_empty()
    {
        return api_error(StatusCode::BAD_REQUEST, 1001, "qr required fields missing");
    }

    let now = Utc::now().timestamp();
    if payload.expire_at < now {
        return api_error(StatusCode::UNAUTHORIZED, 1006, "qr expired");
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_consumed_qr_ids(&mut store, Utc::now());
    cleanup_pending_bind_scans(&mut store, Utc::now());
    if store.consumed_qr_ids.contains_key(&payload.qr_id) {
        return api_error(StatusCode::CONFLICT, 1005, "qr_id already consumed");
    }
    let Some(site_keys) = store.cpms_site_keys.get(&payload.site_sfid).cloned() else {
        return api_error(StatusCode::FORBIDDEN, 1004, "site_sfid keys not registered");
    };
    if !in_scope_cpms_site(&site_keys, admin_ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot use other province institutions",
        );
    }

    let status_text = match payload.status {
        CitizenStatus::Normal => "NORMAL",
        CitizenStatus::Abnormal => "ABNORMAL",
    };
    let canonical = crate::operate::cpms_qr::canonical_citizen_qr_text(
        &payload.ver,
        &payload.issuer_id,
        &payload.site_sfid,
        &payload.archive_no,
        payload.issued_at,
        payload.expire_at,
        &payload.qr_id,
        &payload.sig_alg,
        status_text,
    );
    let verified = crate::operate::cpms_qr::verify_cpms_qr_signature(
        &[
            &site_keys.pubkey_1,
            &site_keys.pubkey_2,
            &site_keys.pubkey_3,
        ],
        &canonical,
        &payload.signature,
    );
    if !verified {
        return api_error(StatusCode::UNAUTHORIZED, 1006, "qr signature verify failed");
    }
    insert_bounded_map(
        &mut store.consumed_qr_ids,
        payload.qr_id.clone(),
        Utc::now(),
        bounded_cache_limit("SFID_CONSUMED_QR_CACHE_MAX", 50_000),
    );
    let pending = PendingBindScan {
        qr_id: payload.qr_id.clone(),
        archive_no: payload.archive_no.clone(),
        site_sfid: payload.site_sfid.clone(),
        status: payload.status.clone(),
        expire_at: payload.expire_at,
        scanned_at: Utc::now(),
    };
    insert_bounded_map(
        &mut store.pending_bind_scan_by_qr_id,
        payload.qr_id.clone(),
        pending,
        bounded_cache_limit("SFID_PENDING_SCAN_CACHE_MAX", 50_000),
    );
    insert_bounded_map(
        &mut store.pending_status_by_archive_no,
        payload.archive_no.clone(),
        payload.status.clone(),
        bounded_cache_limit("SFID_PENDING_STATUS_CACHE_MAX", 50_000),
    );
    append_audit_log_with_meta(
        &mut store,
        "BIND_SCAN",
        &admin_ctx.admin_pubkey,
        None,
        Some(payload.archive_no.clone()),
        request_id,
        actor_ip,
        "SUCCESS",
        format!("qr_id={} site_sfid={}", payload.qr_id, payload.site_sfid),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: BindScanOutput {
            site_sfid: payload.site_sfid,
            archive_no: payload.archive_no,
            qr_id: payload.qr_id,
            status: payload.status,
            issued_at: payload.issued_at,
            expire_at: payload.expire_at,
        },
    })
    .into_response()
}

pub(crate) async fn admin_bind_confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminBindInput>,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let actor_ip = actor_ip_from_headers(&headers);
    let req_id = request_id_from_headers(&headers);

    if input.account_pubkey.trim().is_empty()
        || input.archive_index.trim().is_empty()
        || input.qr_id.trim().is_empty()
    {
        return api_error(StatusCode::BAD_REQUEST, 1001, "invalid request params");
    }
    let current_block = match current_chain_block_number().await {
        Ok(v) => v,
        Err(err) => {
            let detail = format!("resolve chain block failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
        }
    };
    let bind_credential_expires_at = compute_bind_credential_expiry_block(current_block);

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_pending_bind_scans(&mut store, Utc::now());
    let Some(pending_scan) = store.pending_bind_scan_by_qr_id.get(&input.qr_id).cloned() else {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "bind qr_id not scanned or expired",
        );
    };
    if pending_scan.archive_no != input.archive_index {
        return api_error(StatusCode::CONFLICT, 3001, "archive_index mismatch with qr");
    }
    let Some(site_keys) = store.cpms_site_keys.get(&pending_scan.site_sfid).cloned() else {
        return api_error(StatusCode::FORBIDDEN, 1004, "site_sfid keys not registered");
    };
    if !in_scope_cpms_site(&site_keys, admin_ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province institutions",
        );
    }
    if let Some(pending_request) = store.pending_by_pubkey.get(&input.account_pubkey) {
        if !in_scope_pending(pending_request, admin_ctx.admin_province.as_deref()) {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot manage other province citizens",
            );
        }
    }

    if let Some(bound_pubkey) = store.pubkey_by_archive_index.get(&input.archive_index) {
        if bound_pubkey != &input.account_pubkey {
            return api_error(StatusCode::CONFLICT, 3001, "archive_index already bound");
        }
    }
    if let Some(existing) = store.bindings_by_pubkey.get(&input.account_pubkey) {
        if !in_scope(existing, admin_ctx.admin_province.as_deref()) {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot manage other province citizens",
            );
        }
        if existing.archive_index != input.archive_index {
            return api_error(
                StatusCode::CONFLICT,
                3002,
                "pubkey already bound to another archive_index",
            );
        }
        let payload = BindingPayload {
            kind: "bind",
            version: "v1",
            account_pubkey: existing.account_pubkey.clone(),
            archive_index: existing.archive_index.clone(),
            sfid_code: existing.sfid_code.clone(),
            issued_at: existing.bound_at.timestamp(),
        };
        let proof = match make_signature_envelope(&state, &payload) {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "failed to sign binding proof",
                )
            }
        };
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: AdminBindOutput {
                account_pubkey: existing.account_pubkey.clone(),
                archive_index: existing.archive_index.clone(),
                sfid_code: existing.sfid_code.clone(),
                proof,
                status: "BOUND",
                message: "already bound",
            },
        })
        .into_response();
    }
    if let Err(resp) = ensure_binding_lock_db(&state, &input.account_pubkey, &input.archive_index) {
        return resp;
    }

    let sfid_code = store
        .generated_sfid_by_pubkey
        .remove(&input.account_pubkey)
        .unwrap_or_else(|| {
            deterministic_sfid_code(&state, &input.archive_index, &input.account_pubkey)
        });
    let birth_date = parse_birth_date_from_archive_no(&input.archive_index);
    let citizen_status = pending_scan.status.clone();
    store.pending_bind_scan_by_qr_id.remove(&input.qr_id);
    store
        .pending_status_by_archive_no
        .remove(&input.archive_index);
    let pending_request = store.pending_by_pubkey.get(&input.account_pubkey).cloned();
    let seq = pending_request.as_ref().map(|p| p.seq).unwrap_or_else(|| {
        store.next_seq += 1;
        store.next_seq
    });
    let bound_at = Utc::now();
    let binding_payload = BindingPayload {
        kind: "bind",
        version: "v1",
        account_pubkey: input.account_pubkey.clone(),
        archive_index: input.archive_index.clone(),
        sfid_code: sfid_code.clone(),
        issued_at: bound_at.timestamp(),
    };
    let proof = match make_signature_envelope(&state, &binding_payload) {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "failed to sign binding proof",
            )
        }
    };
    let runtime_bind_credential = match build_bind_credential(
        &state,
        input.account_pubkey.as_str(),
        sfid_code.as_str(),
        Uuid::new_v4().to_string(),
        bind_credential_expires_at,
    ) {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "failed to sign runtime bind credential",
            )
        }
    };
    let runtime_bind_signer_pubkey = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "active signer read lock poisoned",
            )
        }
    };
    let bound_by = admin_ctx.admin_pubkey.clone();
    let client_request_id = pending_request
        .as_ref()
        .and_then(|p| p.client_request_id.clone());
    let callback_url = pending_request
        .as_ref()
        .and_then(|p| p.callback_url.clone())
        .or_else(default_bind_callback_url);
    let binding = BindingRecord {
        seq,
        account_pubkey: input.account_pubkey.clone(),
        archive_index: input.archive_index.clone(),
        birth_date,
        citizen_status,
        sfid_code: sfid_code.clone(),
        sfid_signature: proof.signature_hex.clone(),
        runtime_bind_sfid_code_hash: Some(runtime_bind_credential.sfid_code_hash),
        runtime_bind_nonce: Some(runtime_bind_credential.nonce),
        runtime_bind_expires_at_block: Some(runtime_bind_credential.expires_at_block),
        runtime_bind_signature: Some(runtime_bind_credential.signature),
        runtime_bind_key_id: Some(runtime_bind_credential.meta.key_id),
        runtime_bind_key_version: Some(runtime_bind_credential.meta.key_version),
        runtime_bind_alg: Some(runtime_bind_credential.meta.alg),
        runtime_bind_signer_pubkey: Some(runtime_bind_signer_pubkey),
        bound_at,
        bound_by,
        admin_province: admin_ctx.admin_province.clone(),
        client_request_id: client_request_id.clone(),
    };

    store
        .pubkey_by_archive_index
        .insert(input.archive_index.clone(), input.account_pubkey.clone());
    store
        .bindings_by_pubkey
        .insert(input.account_pubkey.clone(), binding);
    store.pending_by_pubkey.remove(&input.account_pubkey);
    store.metrics.bind_confirms_total += 1;
    let callback_id = Uuid::new_v4().to_string();
    let callback_signable = BindCallbackSignablePayload {
        callback_id: callback_id.clone(),
        event: "BIND_CONFIRMED".to_string(),
        account_pubkey: input.account_pubkey.clone(),
        archive_index: input.archive_index.clone(),
        sfid_code: sfid_code.clone(),
        status: "BOUND".to_string(),
        bound_at: bound_at.timestamp(),
        proof: proof.clone(),
        client_request_id: client_request_id.clone(),
    };
    let callback_payload = BindCallbackPayload {
        callback_id: callback_id.clone(),
        event: "BIND_CONFIRMED".to_string(),
        account_pubkey: input.account_pubkey.clone(),
        archive_index: input.archive_index.clone(),
        sfid_code: sfid_code.clone(),
        status: "BOUND".to_string(),
        bound_at: bound_at.timestamp(),
        proof: proof.clone(),
        client_request_id: client_request_id.clone(),
        callback_attestation: match make_signature_envelope(&state, &callback_signable) {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "failed to sign callback attestation",
                )
            }
        },
    };
    let reward = RewardStateRecord {
        account_pubkey: input.account_pubkey.clone(),
        archive_index: input.archive_index.clone(),
        callback_id: callback_payload.callback_id.clone(),
        reward_status: RewardStatus::Pending,
        retry_count: 0,
        max_retries: 5,
        reward_tx_hash: None,
        last_error: None,
        next_retry_at: None,
        updated_at: Utc::now(),
        created_at: Utc::now(),
    };
    match persist_reward_state_db(&state, &reward, None) {
        Ok(true) => {
            store
                .reward_state_by_pubkey
                .insert(input.account_pubkey.clone(), reward.clone());
        }
        Ok(false) => {
            warn!(
                account_pubkey = input.account_pubkey,
                callback_id = reward.callback_id,
                "pending reward state persistence was not applied"
            );
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1502,
                "reward state persistence failed",
            );
        }
        Err(err) => {
            warn!(
                error = %err,
                account_pubkey = input.account_pubkey,
                callback_id = reward.callback_id,
                "failed to persist pending reward state after bind confirm"
            );
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1502,
                "reward state persistence failed",
            );
        }
    }
    invalidate_vote_cache_for_pubkey(&mut store, &input.account_pubkey);
    enqueue_bind_callback_job(&mut store, callback_url, callback_payload);
    append_audit_log(
        &mut store,
        "BIND_CONFIRM",
        &admin_ctx.admin_pubkey,
        Some(input.account_pubkey.clone()),
        Some(input.archive_index.clone()),
        "SUCCESS",
        format!(
            "binding activated (qr_id={}, site_sfid={})",
            pending_scan.qr_id, pending_scan.site_sfid
        ),
    );
    append_audit_log_with_meta(
        &mut store,
        "BIND_CONFIRM_META",
        &admin_ctx.admin_pubkey,
        Some(input.account_pubkey.clone()),
        Some(input.archive_index.clone()),
        req_id,
        actor_ip,
        "SUCCESS",
        "bind confirm metadata".to_string(),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminBindOutput {
            account_pubkey: input.account_pubkey,
            archive_index: input.archive_index,
            sfid_code,
            proof,
            status: "BOUND",
            message: "bind success",
        },
    })
    .into_response()
}

pub(crate) async fn admin_unbind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminUnbindInput>,
) -> impl IntoResponse {
    let actor_ip = actor_ip_from_headers(&headers);
    let request_id = request_id_from_headers(&headers);
    let admin_ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some(binding) = store.bindings_by_pubkey.get(&input.account_pubkey).cloned() else {
        return api_error(StatusCode::NOT_FOUND, 3005, "binding not found");
    };
    if !in_scope(&binding, admin_ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province citizens",
        );
    }
    store.bindings_by_pubkey.remove(&input.account_pubkey);
    store.pubkey_by_archive_index.remove(&binding.archive_index);
    store.pending_by_pubkey.remove(&input.account_pubkey);
    store.reward_state_by_pubkey.remove(&input.account_pubkey);
    invalidate_vote_cache_for_pubkey(&mut store, &input.account_pubkey);
    release_binding_lock_db(&state, &input.account_pubkey);
    remove_reward_state_db(&state, &input.account_pubkey);
    append_audit_log(
        &mut store,
        "BIND_UNBIND",
        &admin_ctx.admin_pubkey,
        Some(input.account_pubkey.clone()),
        Some(binding.archive_index),
        "SUCCESS",
        "binding removed".to_string(),
    );
    append_audit_log_with_meta(
        &mut store,
        "BIND_UNBIND_META",
        &admin_ctx.admin_pubkey,
        Some(input.account_pubkey),
        None,
        request_id,
        actor_ip,
        "SUCCESS",
        "unbind metadata".to_string(),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "unbind success and citizen removed",
    })
    .into_response()
}
