use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use crate::*;

pub(crate) async fn admin_cpms_status_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsStatusScanInput>,
) -> impl IntoResponse {
    let admin_ctx = match require_super_or_operator_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let actor_ip = actor_ip_from_headers(&headers);
    let req_id = request_id_from_headers(&headers);
    if input.qr_payload.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "qr_payload is required");
    }
    let payload: CitizenStatusQrPayload = match serde_json::from_str(input.qr_payload.trim()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid status qr_payload"),
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
    if payload.expire_at < Utc::now().timestamp() {
        return api_error(StatusCode::UNAUTHORIZED, 1006, "qr expired");
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_consumed_qr_ids(&mut store, Utc::now());
    if store.consumed_qr_ids.contains_key(&payload.qr_id) {
        return api_error(StatusCode::CONFLICT, 1005, "qr_id already consumed");
    }
    let Some(site_keys) = store.cpms_site_keys.get(&payload.site_sfid).cloned() else {
        return api_error(StatusCode::FORBIDDEN, 1004, "site_sfid keys not registered");
    };
    if site_keys.status != CpmsSiteStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 1003, "site_sfid keys are not active");
    }
    if !in_scope_cpms_site(&site_keys, admin_ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province institutions",
        );
    }
    let status_text = match payload.status {
        CitizenStatus::Normal => "NORMAL",
        CitizenStatus::Abnormal => "ABNORMAL",
    };
    let canonical = crate::operate::cpms_qr::canonical_status_qr_text(
        &payload.ver,
        &payload.issuer_id,
        &payload.site_sfid,
        &payload.archive_no,
        status_text,
        payload.issued_at,
        payload.expire_at,
        &payload.qr_id,
        &payload.sig_alg,
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

    let Some(pubkey) = store
        .pubkey_by_archive_index
        .get(&payload.archive_no)
        .cloned()
    else {
        return api_error(StatusCode::NOT_FOUND, 1004, "archive_no binding not found");
    };
    let Some(binding) = store.bindings_by_pubkey.get_mut(&pubkey) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "binding not found");
    };
    if !in_scope(binding, admin_ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province citizens",
        );
    }
    let old_status = binding.citizen_status.clone();
    binding.citizen_status = payload.status.clone();
    invalidate_vote_cache_for_pubkey(&mut store, &pubkey);
    append_audit_log(
        &mut store,
        "CPMS_STATUS_SCAN",
        &admin_ctx.admin_pubkey,
        Some(pubkey.clone()),
        Some(payload.archive_no.clone()),
        "SUCCESS",
        format!(
            "site_sfid={} qr_id={} old_status={:?} new_status={:?}",
            payload.site_sfid, payload.qr_id, old_status, payload.status
        ),
    );
    append_audit_log_with_meta(
        &mut store,
        "CPMS_STATUS_SCAN_META",
        &admin_ctx.admin_pubkey,
        Some(pubkey.clone()),
        Some(payload.archive_no.clone()),
        req_id,
        actor_ip,
        "SUCCESS",
        "status scan metadata".to_string(),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CpmsStatusScanOutput {
            archive_no: payload.archive_no,
            status: payload.status,
            message: "citizen status updated by cpms qr",
        },
    })
    .into_response()
}
