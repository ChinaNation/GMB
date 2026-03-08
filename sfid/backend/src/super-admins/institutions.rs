use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use schnorrkel::signing_context;
use serde::Serialize;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};

use crate::business::pubkey::{normalize_cpms_pubkey, same_cpms_pubkey};
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

const MAX_CITY_CHARS: usize = 100;
const MAX_INSTITUTION_CHARS: usize = 100;
const MAX_PROVINCE_CHARS: usize = 100;
const MAX_STATUS_REASON_CHARS: usize = 500;
const CPMS_REGISTER_CHECKSUM_HEX_CHARS: usize = 64;
const CPMS_REGISTER_QR_TTL_SECONDS: i64 = 600;
const CPMS_REGISTER_CLOCK_SKEW_SECONDS: i64 = 120;
const SFID_ID_SEGMENT_COUNT: usize = 5;
const SFID_ID_SEGMENT_A3_LEN: usize = 3;
const SFID_ID_SEGMENT_R5_LEN: usize = 5;
const SFID_ID_SEGMENT_T2P1C1_LEN: usize = 4;
const SFID_ID_SEGMENT_N9_LEN: usize = 9;
const SFID_ID_SEGMENT_D8_LEN: usize = 8;
const SFID_ID_MAX_BYTES: usize = 96;

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
        None => return api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing"),
    };
    if province.chars().count() > MAX_PROVINCE_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province too long");
    }
    let city = input.city.trim().to_string();
    let institution = input.institution.trim().to_string();
    if city.chars().count() > MAX_CITY_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city too long");
    }
    if institution.chars().count() > MAX_INSTITUTION_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "institution too long");
    }
    for _ in 0..5 {
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
        let site_sfid = match validate_sfid_id_format(site_sfid.as_str()) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
        };
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
        let signature = match make_signature_envelope(&state, &claims) {
            Ok(v) => v.signature_hex,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "sign cpms init claims failed",
                )
            }
        };
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
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if store.cpms_site_keys.contains_key(site_sfid.as_str()) {
            drop(store);
            continue;
        }
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
                chain_register_tx_hash: None,
                chain_register_block_number: None,
                chain_register_at: None,
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
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: GenerateCpmsInstitutionSfidOutput {
                site_sfid,
                issued_at,
                expire_at,
                qr_payload,
            },
        })
        .into_response();
    }
    api_error(
        StatusCode::CONFLICT,
        1005,
        "site_sfid collision retry exhausted",
    )
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
    let site_sfid = match validate_sfid_id_format(payload.site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let pubkey_1 = match normalize_cpms_pubkey(payload.pubkey_1.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "pubkey_1 format invalid"),
    };
    let pubkey_2 = match normalize_cpms_pubkey(payload.pubkey_2.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "pubkey_2 format invalid"),
    };
    let pubkey_3 = match normalize_cpms_pubkey(payload.pubkey_3.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "pubkey_3 format invalid"),
    };
    if !cpms_pubkeys_are_distinct(pubkey_1.as_str(), pubkey_2.as_str(), pubkey_3.as_str()) {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "cpms pubkeys must be distinct",
        );
    }
    let now = Utc::now();
    let now_ts = now.timestamp();
    if payload.issued_at > now_ts + CPMS_REGISTER_CLOCK_SKEW_SECONDS {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "register issued_at is in future",
        );
    }
    if payload.issued_at < now_ts - CPMS_REGISTER_QR_TTL_SECONDS {
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
    let checksum = payload.checksum_or_signature.trim();
    if checksum.len() != CPMS_REGISTER_CHECKSUM_HEX_CHARS
        || !checksum.chars().all(|c| c.is_ascii_hexdigit())
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "register checksum format invalid",
        );
    }
    if !checksum.eq_ignore_ascii_case(expected.as_str()) {
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
    let init_payload_site_sfid = match validate_sfid_id_format(init_qr_payload.site_sfid.as_str()) {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::UNAUTHORIZED,
                1006,
                "init_qr_payload site_sfid format invalid",
            )
        }
    };
    if init_payload_site_sfid != site_sfid {
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
    } else {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    }

    let replay_token = compute_cpms_register_replay_token(input.qr_payload.trim());
    {
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
        let Some(site) = store.cpms_site_keys.get(site_sfid.as_str()) else {
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
        let mut inflight = match state.cpms_register_inflight.lock() {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "register inflight lock unavailable",
                )
            }
        };
        if inflight.contains(&replay_token) {
            return api_error(StatusCode::CONFLICT, 1005, "register qr is being processed");
        }
        inflight.insert(replay_token.clone());
    }

    let chain_receipt =
        match submit_register_sfid_institution_extrinsic(&state, site_sfid.as_str()).await {
            Ok(v) => v,
            Err(msg) => {
                clear_cpms_register_inflight(&state, replay_token.as_str());
                if let Ok(mut store) = store_write_or_500(&state) {
                    append_audit_log(
                        &mut store,
                        "CPMS_KEYS_REGISTER_SCAN",
                        &ctx.admin_pubkey,
                        Some(site_sfid.clone()),
                        None,
                        "CHAIN_SUBMIT_FAILED",
                        format!("site_sfid={} error={}", site_sfid, msg),
                    );
                    drop(store);
                    persist_runtime_state(&state);
                }
                return api_error(StatusCode::BAD_GATEWAY, 1004, msg.as_str());
            }
        };

    let commit_at = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => {
            clear_cpms_register_inflight(&state, replay_token.as_str());
            return resp;
        }
    };
    cleanup_consumed_cpms_register_tokens(&mut store, commit_at);
    let Some(site) = store.cpms_site_keys.get_mut(site_sfid.as_str()) else {
        clear_cpms_register_inflight(&state, replay_token.as_str());
        return api_error(StatusCode::NOT_FOUND, 1004, "site_sfid not generated");
    };
    if site.status != CpmsSiteStatus::Pending {
        clear_cpms_register_inflight(&state, replay_token.as_str());
        return api_error(StatusCode::CONFLICT, 1005, "site_sfid already registered");
    }
    if site
        .init_qr_payload
        .as_deref()
        .map(|v| v.trim() != init_qr_payload_text)
        .unwrap_or(true)
    {
        clear_cpms_register_inflight(&state, replay_token.as_str());
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "init_qr_payload not issued by sfid",
        );
    }
    site.pubkey_1 = pubkey_1;
    site.pubkey_2 = pubkey_2;
    site.pubkey_3 = pubkey_3;
    site.init_qr_payload = None;
    site.status = CpmsSiteStatus::Active;
    site.last_register_issued_at = payload.issued_at;
    site.admin_province = init_qr_payload.province.trim().to_string();
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(commit_at);
    site.chain_register_tx_hash = Some(chain_receipt.tx_hash.clone());
    site.chain_register_block_number = Some(chain_receipt.block_number);
    site.chain_register_at = Some(commit_at);
    insert_bounded_map(
        &mut store.consumed_cpms_register_tokens,
        replay_token.clone(),
        commit_at,
        bounded_cache_limit("SFID_CPMS_REGISTER_TOKEN_CACHE_MAX", 50_000),
    );
    append_audit_log(
        &mut store,
        "CPMS_KEYS_REGISTER_SCAN",
        &ctx.admin_pubkey,
        Some(site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} keys_registered=3 issued_at={} province={} tx_hash={} block_number={}",
            site_sfid,
            payload.issued_at,
            init_qr_payload.province,
            chain_receipt.tx_hash,
            chain_receipt.block_number
        ),
    );
    drop(store);
    persist_runtime_state(&state);
    clear_cpms_register_inflight(&state, replay_token.as_str());

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CpmsRegisterScanOutput {
            site_sfid,
            status: "ACTIVE",
            message: "cpms site keys registered",
            chain_register_tx_hash: chain_receipt.tx_hash,
            chain_register_block_number: chain_receipt.block_number,
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
    if site_sfid.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "site_sfid is required");
    }
    let site_sfid = match validate_sfid_id_format(site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let pubkey_1 = match normalize_cpms_pubkey(input.pubkey_1.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "pubkey_1 format invalid"),
    };
    let pubkey_2 = match normalize_cpms_pubkey(input.pubkey_2.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "pubkey_2 format invalid"),
    };
    let pubkey_3 = match normalize_cpms_pubkey(input.pubkey_3.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "pubkey_3 format invalid"),
    };
    if !cpms_pubkeys_are_distinct(pubkey_1.as_str(), pubkey_2.as_str(), pubkey_3.as_str()) {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "cpms pubkeys must be distinct",
        );
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site = match store.cpms_site_keys.get_mut(site_sfid.as_str()) {
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
    if site.status != CpmsSiteStatus::Active {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "only active cpms site can be updated via this endpoint",
        );
    }
    let old_pubkey_1 = site.pubkey_1.clone();
    let old_pubkey_2 = site.pubkey_2.clone();
    let old_pubkey_3 = site.pubkey_3.clone();
    site.pubkey_1 = pubkey_1;
    site.pubkey_2 = pubkey_2;
    site.pubkey_3 = pubkey_3;
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    let output = site.clone();
    let response_row = cpms_site_keys_to_list_row(&output);
    append_audit_log(
        &mut store,
        "CPMS_KEYS_UPDATE",
        &ctx.admin_pubkey,
        Some(output.site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} version={} old_pubkeys=[{},{},{}] new_pubkeys=[{},{},{}]",
            output.site_sfid,
            output.version,
            old_pubkey_1,
            old_pubkey_2,
            old_pubkey_3,
            output.pubkey_1,
            output.pubkey_2,
            output.pubkey_3
        ),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: response_row,
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

pub(crate) async fn enable_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
    Json(input): Json<UpdateCpmsSiteStatusInput>,
) -> impl IntoResponse {
    update_cpms_site_status(
        state,
        headers,
        site_sfid,
        CpmsSiteStatus::Active,
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
    let site_sfid = match validate_sfid_id_format(site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some(existing) = store.cpms_site_keys.get(site_sfid.as_str()).cloned() else {
        return api_error(StatusCode::NOT_FOUND, 1004, "cpms site not found");
    };
    if !in_scope_cpms_site(&existing, ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province institutions",
        );
    }
    if existing.status != CpmsSiteStatus::Pending {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "only pending cpms site can be deleted",
        );
    }
    store.cpms_site_keys.remove(site_sfid.as_str());
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
    let mut rows: Vec<CpmsSiteKeysListRow> = store
        .cpms_site_keys
        .values()
        .filter(|site| in_scope_cpms_site(site, ctx.admin_province.as_deref()))
        .map(cpms_site_keys_to_list_row)
        .collect();
    rows.sort_by(|a, b| a.site_sfid.cmp(&b.site_sfid));
    let total = rows.len();
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
        data: CpmsKeysListOutput {
            total,
            limit,
            offset,
            rows,
        },
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
    let site_sfid = match validate_sfid_id_format(site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let reason_text = reason.unwrap_or_default().trim().to_string();
    if reason_text.chars().count() > MAX_STATUS_REASON_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "reason too long");
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site = match store.cpms_site_keys.get_mut(site_sfid.as_str()) {
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
    if site.status == target_status {
        let output = site.clone();
        let response_row = cpms_site_keys_to_list_row(&output);
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: response_row,
        })
        .into_response();
    }
    if !can_transition_cpms_site_status(&site.status, &target_status) {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "invalid cpms site status transition",
        );
    }
    site.status = target_status.clone();
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    let output = site.clone();
    let response_row = cpms_site_keys_to_list_row(&output);
    append_audit_log(
        &mut store,
        "CPMS_KEYS_STATUS_UPDATE",
        &ctx.admin_pubkey,
        Some(output.site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} status={:?} version={} reason={}",
            output.site_sfid, output.status, output.version, reason_text
        ),
    );
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: response_row,
    })
    .into_response()
}

fn cpms_site_keys_to_list_row(site: &CpmsSiteKeys) -> CpmsSiteKeysListRow {
    CpmsSiteKeysListRow {
        site_sfid: site.site_sfid.clone(),
        pubkey_1: site.pubkey_1.clone(),
        pubkey_2: site.pubkey_2.clone(),
        pubkey_3: site.pubkey_3.clone(),
        status: site.status.clone(),
        version: site.version,
        last_register_issued_at: site.last_register_issued_at,
        admin_province: site.admin_province.clone(),
        created_by: site.created_by.clone(),
        created_at: site.created_at,
        updated_by: site.updated_by.clone(),
        updated_at: site.updated_at,
        chain_register_tx_hash: site.chain_register_tx_hash.clone(),
        chain_register_block_number: site.chain_register_block_number,
        chain_register_at: site.chain_register_at,
    }
}

#[derive(Debug, Clone)]
struct ChainInstitutionRegisterReceipt {
    tx_hash: String,
    block_number: u64,
}

fn validate_sfid_id_format(raw: &str) -> Result<String, &'static str> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err("site_sfid is required");
    }
    if !normalized.is_ascii() {
        return Err("site_sfid must be ascii");
    }
    if normalized.len() > SFID_ID_MAX_BYTES {
        return Err("site_sfid length exceeds chain max");
    }
    if normalized
        .bytes()
        .any(|b| !(b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-'))
    {
        return Err("site_sfid charset invalid");
    }
    let segments = normalized.split('-').collect::<Vec<_>>();
    if segments.len() != SFID_ID_SEGMENT_COUNT {
        return Err("site_sfid format invalid");
    }
    if segments[0].len() != SFID_ID_SEGMENT_A3_LEN
        || !segments[0].chars().all(|c| c.is_ascii_uppercase())
    {
        return Err("site_sfid a3 segment invalid");
    }
    if segments[1].len() != SFID_ID_SEGMENT_R5_LEN
        || !segments[1].chars().all(|c| c.is_ascii_digit())
    {
        return Err("site_sfid r5 segment invalid");
    }
    if segments[2].len() != SFID_ID_SEGMENT_T2P1C1_LEN
        || !segments[2]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("site_sfid t2p1c1 segment invalid");
    }
    if segments[3].len() != SFID_ID_SEGMENT_N9_LEN
        || !segments[3].chars().all(|c| c.is_ascii_digit())
    {
        return Err("site_sfid n9 segment invalid");
    }
    if segments[4].len() != SFID_ID_SEGMENT_D8_LEN
        || !segments[4].chars().all(|c| c.is_ascii_digit())
    {
        return Err("site_sfid date segment invalid");
    }
    Ok(normalized.to_string())
}

fn normalize_chain_ws_url(input: &str) -> String {
    if let Some(rest) = input.strip_prefix("http://") {
        return format!("ws://{rest}");
    }
    if let Some(rest) = input.strip_prefix("https://") {
        return format!("wss://{rest}");
    }
    input.to_string()
}

fn resolve_chain_ws_url() -> Result<String, String> {
    let ws_url = std::env::var("SFID_CHAIN_WS_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("SFID_CHAIN_RPC_URL")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
        .ok_or_else(|| "SFID_CHAIN_RPC_URL or SFID_CHAIN_WS_URL not configured".to_string())?;
    Ok(normalize_chain_ws_url(ws_url.as_str()))
}

fn parse_account_id32(pubkey: &str) -> Result<[u8; 32], String> {
    crate::login::parse_sr25519_pubkey_bytes(pubkey)
        .ok_or_else(|| "invalid sr25519 account pubkey".to_string())
}

fn resolve_chain_signer_material(state: &AppState) -> Result<(String, SensitiveSeed), String> {
    let signer_pubkey = state
        .public_key_hex
        .read()
        .map_err(|_| "signer public key read lock poisoned".to_string())?
        .clone();
    let signer_seed = state
        .signing_seed_hex
        .read()
        .map_err(|_| "signer seed read lock poisoned".to_string())?
        .clone();
    if signer_seed.expose_secret().trim().is_empty() {
        return Err("signer seed unavailable".to_string());
    }
    if parse_account_id32(signer_pubkey.as_str()).is_err() {
        return Err("signer public key invalid".to_string());
    }
    Ok((signer_pubkey, signer_seed))
}

async fn submit_register_sfid_institution_extrinsic(
    state: &AppState,
    site_sfid: &str,
) -> Result<ChainInstitutionRegisterReceipt, String> {
    let sfid_id = validate_sfid_id_format(site_sfid)
        .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let ws_url = resolve_chain_ws_url()
        .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let client = OnlineClient::<PolkadotConfig>::from_url(ws_url)
        .await
        .map_err(|e| {
            format!("register_sfid_institution submit failed: chain websocket connect failed: {e}")
        })?;

    let (signer_pubkey, signer_seed) = resolve_chain_signer_material(state)
        .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let signer_account = AccountId32(
        parse_account_id32(signer_pubkey.as_str())
            .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?,
    );
    let payload = tx(
        "DuoqianTransactionPow",
        "register_sfid_institution",
        vec![Value::from_bytes(sfid_id.as_bytes().to_vec())],
    );
    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, Default::default())
        .await
        .map_err(|e| {
            format!("register_sfid_institution submit failed: build extrinsic failed: {e}")
        })?;
    let signing_key =
        key_admins::chain_keyring::try_load_signing_key_from_seed(signer_seed.expose_secret())
            .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let signature = signing_key
        .sign(signing_context(b"substrate").bytes(&partial_tx.signer_payload()))
        .to_bytes();
    let extrinsic = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(extrinsic.hash().as_ref()));

    let submitted = extrinsic.submit_and_watch().await.map_err(|e| {
        format!("register_sfid_institution submit failed: submit_and_watch failed: {e}")
    })?;
    let in_block = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        submitted.wait_for_finalized(),
    )
    .await
    .map_err(|_| {
        "register_sfid_institution submit failed: timed out waiting for finalization".to_string()
    })?
    .map_err(|e| {
        format!("register_sfid_institution submit failed: wait_for_finalized failed: {e}")
    })?;
    in_block
        .wait_for_success()
        .await
        .map_err(|e| format!("register_sfid_institution included failed: {e}"))?;

    let block = client
        .blocks()
        .at(in_block.block_hash())
        .await
        .map_err(|e| {
            format!("register_sfid_institution included failed: fetch block failed: {e}")
        })?;
    let block_number = block.number().to_string().parse::<u64>().map_err(|e| {
        format!("register_sfid_institution included failed: parse block number failed: {e}")
    })?;

    Ok(ChainInstitutionRegisterReceipt {
        tx_hash,
        block_number,
    })
}

fn cpms_pubkeys_are_distinct(pubkey_1: &str, pubkey_2: &str, pubkey_3: &str) -> bool {
    !same_cpms_pubkey(pubkey_1, pubkey_2)
        && !same_cpms_pubkey(pubkey_1, pubkey_3)
        && !same_cpms_pubkey(pubkey_2, pubkey_3)
}

fn can_transition_cpms_site_status(current: &CpmsSiteStatus, target: &CpmsSiteStatus) -> bool {
    matches!(
        (current, target),
        (CpmsSiteStatus::Active, CpmsSiteStatus::Disabled)
            | (CpmsSiteStatus::Active, CpmsSiteStatus::Revoked)
            | (CpmsSiteStatus::Disabled, CpmsSiteStatus::Active)
            | (CpmsSiteStatus::Disabled, CpmsSiteStatus::Revoked)
    )
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

fn clear_cpms_register_inflight(state: &AppState, replay_token: &str) {
    if let Ok(mut inflight) = state.cpms_register_inflight.lock() {
        inflight.remove(replay_token);
    }
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
