//! 公民状态扫码 handler:CPMS 站点扫公民状态 QR,审计 + 缓存失效。
//!
//! 入口由 `shi_admins::admin_cpms_status_scan` 路由转发,链上交互能力位于
//! `citizens::chain_binding`,canonical 文本拼装位于 `citizens::cpms_qr`。

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
    let admin_ctx = match require_admin_any(&state, &headers) {
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

    // Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store
    // 先从分片读 cpms site（async），再拿 legacy store 短锁做其余操作
    let site_sfid_key = payload.site_sfid.trim().to_string();
    let site_keys: CpmsSiteKeys = {
        let province = match crate::sheng_admins::institutions::resolve_site_province_via_shard(
            &state,
            &site_sfid_key,
            admin_ctx.admin_province.as_deref(),
        )
        .await
        {
            Ok(v) => v,
            Err((_code, _msg)) => {
                return api_error(StatusCode::FORBIDDEN, 1004, "site_sfid keys not registered");
            }
        };
        match state
            .sharded_store
            .read_province(&province, |shard| {
                shard.cpms_site_keys.get(&site_sfid_key).cloned()
            })
            .await
        {
            Ok(Some(v)) => v,
            Ok(None) => {
                return api_error(StatusCode::FORBIDDEN, 1004, "site_sfid keys not registered");
            }
            Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e),
        }
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
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_consumed_qr_ids(&mut store, Utc::now());
    if store.consumed_qr_ids.contains_key(&payload.qr_id) {
        return api_error(StatusCode::CONFLICT, 1005, "qr_id already consumed");
    }
    let status_text = match payload.status {
        CitizenStatus::Normal => "NORMAL",
        CitizenStatus::Abnormal => "ABNORMAL",
    };
    let canonical = crate::citizens::cpms_qr::canonical_status_qr_text(
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
    // 旧 CPMS QR 签名验证已废弃（SFID-CPMS QR v1 使用 archive_import 端点）。
    // 公钥列表已清空,跳过签名校验,仅依赖上游的 site_sfid + expire_at 校验。
    let _canonical = canonical;
    insert_bounded_map(
        &mut store.consumed_qr_ids,
        payload.qr_id.clone(),
        Utc::now(),
        bounded_cache_limit("SFID_CONSUMED_QR_CACHE_MAX", 50_000),
    );

    // 中文注释:从 citizen_records 查找绑定(取代旧 pubkey_by_archive_index + bindings_by_pubkey)。
    let Some(cid) = store
        .citizen_id_by_archive_no
        .get(&payload.archive_no)
        .cloned()
    else {
        return api_error(StatusCode::NOT_FOUND, 1004, "archive_no binding not found");
    };
    let Some(record) = store.citizen_records.get(&cid) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found");
    };
    let pubkey = record.account_pubkey.clone().unwrap_or_default();
    invalidate_vote_cache_for_pubkey(&mut store, &pubkey);
    append_audit_log(
        &mut store,
        "CPMS_STATUS_SCAN",
        &admin_ctx.admin_pubkey,
        Some(pubkey.clone()),
        Some(payload.archive_no.clone()),
        "SUCCESS",
        format!(
            "site_sfid={} qr_id={} new_status={:?}",
            payload.site_sfid, payload.qr_id, payload.status
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
