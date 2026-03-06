use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use serde::Serialize;

use crate::sfid::{generate_sfid_code, GenerateSfidInput};
use crate::*;

#[derive(Debug, Clone, Serialize)]
struct CpmsInstitutionInitClaims {
    ver: String,
    issuer_id: String,
    purpose: String,
    site_sfid: String,
    a3: String,
    p1: String,
    province: String,
    city: String,
    institution: String,
    issued_at: i64,
    expire_at: i64,
    qr_id: String,
    sig_alg: String,
    key_id: String,
    key_version: String,
    public_key: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct ListQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

pub(crate) async fn generate_cpms_institution_sfid_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GenerateCpmsInstitutionSfidInput>,
) -> impl IntoResponse {
    let ctx = match require_super_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.city.trim().is_empty() || input.institution.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "city and institution are required",
        );
    }
    let province = match ctx.admin_province.as_deref() {
        Some(scope) => {
            if let Some(raw) = input.province.as_deref() {
                if !raw.trim().is_empty() && raw.trim() != scope {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        1003,
                        "province out of current admin scope",
                    );
                }
            }
            scope.to_string()
        }
        None => {
            let Some(raw) = input.province.as_deref() else {
                return api_error(StatusCode::BAD_REQUEST, 1001, "province is required");
            };
            let province = raw.trim();
            if province.is_empty() {
                return api_error(StatusCode::BAD_REQUEST, 1001, "province is required");
            }
            province.to_string()
        }
    };
    let city = input.city.trim().to_string();
    let institution = input.institution.trim().to_string();
    let random_account = Uuid::new_v4().to_string();
    let site_sfid = match generate_sfid_code(GenerateSfidInput {
        account_pubkey: random_account.as_str(),
        a3: "GFR",
        p1: "0",
        province: province.as_str(),
        city: city.as_str(),
        institution: institution.as_str(),
    }) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store.cpms_site_keys.contains_key(site_sfid.as_str()) {
        return api_error(StatusCode::CONFLICT, 1005, "site_sfid already exists");
    }
    let issued_at = Utc::now().timestamp();
    let expire_at = 0_i64;
    let public_key = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "public key unavailable",
            )
        }
    };
    let claims = CpmsInstitutionInitClaims {
        ver: "1".to_string(),
        issuer_id: "sfid".to_string(),
        purpose: "cpms_init".to_string(),
        site_sfid: site_sfid.clone(),
        a3: "GFR".to_string(),
        p1: "0".to_string(),
        province: province.clone(),
        city: city.clone(),
        institution: institution.clone(),
        issued_at,
        expire_at,
        qr_id: Uuid::new_v4().to_string(),
        sig_alg: "sr25519".to_string(),
        key_id: state.key_id.clone(),
        key_version: state.key_version.clone(),
        public_key,
    };
    let claims_text = match serde_json::to_string(&claims) {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "serialize cpms init claims failed",
            )
        }
    };
    let signature = make_signature_envelope(&state, &claims).signature_hex;
    let qr_payload = match serde_json::to_string(&CpmsInstitutionInitQrPayload {
        ver: claims.ver.clone(),
        issuer_id: claims.issuer_id.clone(),
        purpose: claims.purpose.clone(),
        site_sfid: claims.site_sfid.clone(),
        a3: claims.a3.clone(),
        p1: claims.p1.clone(),
        province: claims.province.clone(),
        city: claims.city.clone(),
        institution: claims.institution.clone(),
        issued_at: claims.issued_at,
        expire_at: claims.expire_at,
        qr_id: claims.qr_id.clone(),
        sig_alg: claims.sig_alg.clone(),
        key_id: claims.key_id.clone(),
        key_version: claims.key_version.clone(),
        public_key: claims.public_key.clone(),
        signature,
    }) {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "serialize cpms init qr payload failed",
            )
        }
    };
    let created_at = Utc::now();
    store.cpms_site_keys.insert(
        site_sfid.clone(),
        CpmsSiteKeys {
            site_sfid: site_sfid.clone(),
            pubkey_1: String::new(),
            pubkey_2: String::new(),
            pubkey_3: String::new(),
            status: CpmsSiteStatus::Pending,
            version: 1,
            last_register_issued_at: 0,
            init_qr_payload: Some(qr_payload.clone()),
            admin_province: province.clone(),
            created_by: ctx.admin_pubkey.clone(),
            created_at,
            updated_by: Some(ctx.admin_pubkey.clone()),
            updated_at: Some(created_at),
        },
    );

    append_audit_log(
        &mut store,
        "CPMS_SFID_GENERATE",
        &ctx.admin_pubkey,
        Some(site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} province={} city={} institution={} issued_at={} payload_hash={}",
            site_sfid,
            province,
            city,
            institution,
            issued_at,
            hex::encode(blake3::hash(claims_text.as_bytes()).as_bytes()),
        ),
    );
    drop(store);
    persist_runtime_state(&state);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: GenerateCpmsInstitutionSfidOutput {
            site_sfid,
            issued_at,
            expire_at,
            qr_payload,
        },
    })
    .into_response()
}

pub(crate) async fn register_cpms_keys_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsRegisterScanInput>,
) -> impl IntoResponse {
    let ctx = match require_super_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.qr_payload.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "qr_payload is required");
    }

    let payload: CpmsRegisterQrPayload = match serde_json::from_str(input.qr_payload.trim()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid register qr_payload"),
    };
    if payload.site_sfid.trim().is_empty()
        || payload.pubkey_1.trim().is_empty()
        || payload.pubkey_2.trim().is_empty()
        || payload.pubkey_3.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "register payload required fields missing",
        );
    }
    let now = Utc::now();
    let now_ts = now.timestamp();
    if payload.issued_at > now_ts + 120 {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "register issued_at is in future",
        );
    }
    if payload.issued_at < now_ts - 600 {
        return api_error(StatusCode::UNAUTHORIZED, 1006, "register qr expired");
    }

    let Some(init_qr_payload_raw) = payload.init_qr_payload.as_deref() else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "init_qr_payload is required");
    };
    let init_qr_payload_text = init_qr_payload_raw.trim();
    if init_qr_payload_text.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "init_qr_payload is required");
    }
    let expected = compute_cpms_register_checksum(
        &payload.site_sfid,
        &payload.pubkey_1,
        &payload.pubkey_2,
        &payload.pubkey_3,
        payload.issued_at,
        init_qr_payload_text,
    );
    if !payload
        .checksum_or_signature
        .eq_ignore_ascii_case(expected.as_str())
    {
        return api_error(
            StatusCode::UNAUTHORIZED,
            1006,
            "register checksum invalid for init_qr_payload",
        );
    }
    let init_qr_payload: CpmsInstitutionInitQrPayload =
        match serde_json::from_str(init_qr_payload_text) {
            Ok(v) => v,
            Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid init_qr_payload"),
        };
    if init_qr_payload.ver != "1"
        || init_qr_payload.issuer_id != "sfid"
        || init_qr_payload.purpose != "cpms_init"
        || init_qr_payload.sig_alg != "sr25519"
        || init_qr_payload.a3 != "GFR"
        || init_qr_payload.p1 != "0"
    {
        return api_error(
            StatusCode::UNAUTHORIZED,
            1006,
            "init_qr_payload header invalid",
        );
    }
    if init_qr_payload.site_sfid.trim() != payload.site_sfid.trim() {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "site_sfid mismatch with init_qr_payload",
        );
    }
    if !is_trusted_attestor_pubkey(&state, init_qr_payload.public_key.as_str()) {
        return api_error(
            StatusCode::UNAUTHORIZED,
            1006,
            "init_qr_payload signer not trusted",
        );
    }
    let claims = CpmsInstitutionInitClaims {
        ver: init_qr_payload.ver.clone(),
        issuer_id: init_qr_payload.issuer_id.clone(),
        purpose: init_qr_payload.purpose.clone(),
        site_sfid: init_qr_payload.site_sfid.clone(),
        a3: init_qr_payload.a3.clone(),
        p1: init_qr_payload.p1.clone(),
        province: init_qr_payload.province.clone(),
        city: init_qr_payload.city.clone(),
        institution: init_qr_payload.institution.clone(),
        issued_at: init_qr_payload.issued_at,
        expire_at: init_qr_payload.expire_at,
        qr_id: init_qr_payload.qr_id.clone(),
        sig_alg: init_qr_payload.sig_alg.clone(),
        key_id: init_qr_payload.key_id.clone(),
        key_version: init_qr_payload.key_version.clone(),
        public_key: init_qr_payload.public_key.clone(),
    };
    let claims_text = match serde_json::to_string(&claims) {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "serialize init claims failed",
            )
        }
    };
    if !verify_admin_signature(
        init_qr_payload.public_key.as_str(),
        claims_text.as_str(),
        init_qr_payload.signature.as_str(),
    ) {
        return api_error(
            StatusCode::UNAUTHORIZED,
            1006,
            "init_qr_payload signature verify failed",
        );
    }
    if let Some(scope) = ctx.admin_province.as_deref() {
        if scope != init_qr_payload.province.trim() {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot register other province institutions",
            );
        }
    }

    let replay_token = compute_cpms_register_replay_token(input.qr_payload.trim());
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_consumed_cpms_register_tokens(&mut store, now);
    if store
        .consumed_cpms_register_tokens
        .contains_key(&replay_token)
    {
        return api_error(StatusCode::CONFLICT, 1005, "register qr already consumed");
    }
    let Some(site) = store.cpms_site_keys.get_mut(payload.site_sfid.as_str()) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "site_sfid not generated");
    };
    if site.status != CpmsSiteStatus::Pending {
        return api_error(StatusCode::CONFLICT, 1005, "site_sfid already registered");
    }
    if site
        .init_qr_payload
        .as_deref()
        .map(|v| v.trim() != init_qr_payload_text)
        .unwrap_or(true)
    {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "init_qr_payload not issued by sfid",
        );
    }
    site.pubkey_1 = payload.pubkey_1.clone();
    site.pubkey_2 = payload.pubkey_2.clone();
    site.pubkey_3 = payload.pubkey_3.clone();
    site.status = CpmsSiteStatus::Active;
    site.last_register_issued_at = payload.issued_at;
    site.admin_province = init_qr_payload.province.trim().to_string();
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(now);
    insert_bounded_map(
        &mut store.consumed_cpms_register_tokens,
        replay_token,
        now,
        bounded_cache_limit("SFID_CPMS_REGISTER_TOKEN_CACHE_MAX", 50_000),
    );
    append_audit_log(
        &mut store,
        "CPMS_KEYS_REGISTER_SCAN",
        &ctx.admin_pubkey,
        Some(payload.site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} keys_registered=3 issued_at={} province={}",
            payload.site_sfid, payload.issued_at, init_qr_payload.province
        ),
    );
    drop(store);
    persist_runtime_state(&state);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CpmsRegisterScanOutput {
            site_sfid: payload.site_sfid,
            status: "ACTIVE",
            message: "cpms site keys registered",
        },
    })
    .into_response()
}

pub(crate) async fn update_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
    Json(input): Json<UpdateCpmsKeysInput>,
) -> impl IntoResponse {
    let ctx = match require_super_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if site_sfid.trim().is_empty()
        || input.pubkey_1.trim().is_empty()
        || input.pubkey_2.trim().is_empty()
        || input.pubkey_3.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "site_sfid and pubkeys are required",
        );
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site = match store.cpms_site_keys.get_mut(site_sfid.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "cpms site not found"),
    };
    if !in_scope_cpms_site(site, ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province institutions",
        );
    }
    if site.status == CpmsSiteStatus::Revoked {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "revoked cpms site cannot be updated",
        );
    }
    site.pubkey_1 = input.pubkey_1.trim().to_string();
    site.pubkey_2 = input.pubkey_2.trim().to_string();
    site.pubkey_3 = input.pubkey_3.trim().to_string();
    site.status = CpmsSiteStatus::Active;
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    let output = site.clone();
    append_audit_log(
        &mut store,
        "CPMS_KEYS_UPDATE",
        &ctx.admin_pubkey,
        Some(output.site_sfid.clone()),
        None,
        "SUCCESS",
        format!("site_sfid={} version={}", output.site_sfid, output.version),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

pub(crate) async fn disable_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
    Json(input): Json<UpdateCpmsSiteStatusInput>,
) -> impl IntoResponse {
    update_cpms_site_status(
        state,
        headers,
        site_sfid,
        CpmsSiteStatus::Disabled,
        input.reason,
    )
    .await
}

pub(crate) async fn revoke_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
    Json(input): Json<UpdateCpmsSiteStatusInput>,
) -> impl IntoResponse {
    update_cpms_site_status(
        state,
        headers,
        site_sfid,
        CpmsSiteStatus::Revoked,
        input.reason,
    )
    .await
}

pub(crate) async fn delete_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_super_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if site_sfid.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "site_sfid is required");
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some(existing) = store.cpms_site_keys.get(site_sfid.trim()).cloned() else {
        return api_error(StatusCode::NOT_FOUND, 1004, "cpms site not found");
    };
    if !in_scope_cpms_site(&existing, ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province institutions",
        );
    }
    store.cpms_site_keys.remove(site_sfid.trim());
    append_audit_log(
        &mut store,
        "CPMS_KEYS_DELETE",
        &ctx.admin_pubkey,
        Some(existing.site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} status={:?} version={}",
            existing.site_sfid, existing.status, existing.version
        ),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "deleted",
    })
    .into_response()
}

pub(crate) async fn list_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_super_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<CpmsSiteKeys> = store
        .cpms_site_keys
        .values()
        .filter(|site| in_scope_cpms_site(site, ctx.admin_province.as_deref()))
        .cloned()
        .collect();
    rows.sort_by(|a, b| a.site_sfid.cmp(&b.site_sfid));
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);
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

async fn update_cpms_site_status(
    state: AppState,
    headers: HeaderMap,
    site_sfid: String,
    target_status: CpmsSiteStatus,
    reason: Option<String>,
) -> axum::response::Response {
    let ctx = match require_super_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if site_sfid.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "site_sfid is required");
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site = match store.cpms_site_keys.get_mut(site_sfid.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "cpms site not found"),
    };
    if !in_scope_cpms_site(site, ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province institutions",
        );
    }
    site.status = target_status.clone();
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    let output = site.clone();
    append_audit_log(
        &mut store,
        "CPMS_KEYS_STATUS_UPDATE",
        &ctx.admin_pubkey,
        Some(output.site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} status={:?} version={} reason={}",
            output.site_sfid,
            output.status,
            output.version,
            reason.unwrap_or_default().trim()
        ),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

fn compute_cpms_register_checksum(
    site_sfid: &str,
    pubkey_1: &str,
    pubkey_2: &str,
    pubkey_3: &str,
    issued_at: i64,
    init_qr_payload: &str,
) -> String {
    let init_hash = hex::encode(blake3::hash(init_qr_payload.as_bytes()).as_bytes());
    let payload = format!(
        "site_sfid={site_sfid}&pubkey_1={pubkey_1}&pubkey_2={pubkey_2}&pubkey_3={pubkey_3}&issued_at={issued_at}&init_qr_hash={init_hash}"
    );
    hex::encode(blake3::hash(payload.as_bytes()).as_bytes())
}

fn compute_cpms_register_replay_token(raw_payload: &str) -> String {
    hex::encode(blake3::hash(raw_payload.trim().as_bytes()).as_bytes())
}

fn cleanup_consumed_cpms_register_tokens(store: &mut Store, now: chrono::DateTime<Utc>) {
    store
        .consumed_cpms_register_tokens
        .retain(|_, consumed_at| *consumed_at > now - Duration::hours(24));
}

fn is_trusted_attestor_pubkey(state: &AppState, public_key: &str) -> bool {
    let Some(candidate) = parse_sr25519_pubkey(public_key) else {
        return false;
    };
    let current = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => return false,
    };
    if parse_sr25519_pubkey(current.as_str())
        .map(|v| v == candidate)
        .unwrap_or(false)
    {
        return true;
    }
    let known = match state.known_key_seeds.read() {
        Ok(v) => v,
        Err(_) => return false,
    };
    known.keys().any(|k| {
        parse_sr25519_pubkey(k.as_str())
            .map(|v| v == candidate)
            .unwrap_or(false)
    })
}
