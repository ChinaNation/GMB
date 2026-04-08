use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use serde::Serialize;
use sp_core::Pair;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};

use crate::business::pubkey::{normalize_cpms_pubkey, same_cpms_pubkey};
use crate::chain::runtime_align::build_institution_credential;
use crate::sfid::{generate_sfid_code, GenerateSfidInput};
use crate::*;

type Blake2b256 = Blake2b<U32>;

const MAX_CITY_CHARS: usize = 100;
const MAX_INSTITUTION_CHARS: usize = 100;
const MAX_PROVINCE_CHARS: usize = 100;
const MAX_STATUS_REASON_CHARS: usize = 500;
const SFID_ID_SEGMENT_COUNT: usize = 5;
const SFID_ID_SEGMENT_A3_LEN: usize = 3;
const SFID_ID_SEGMENT_R5_LEN: usize = 5;
const SFID_ID_SEGMENT_T2P1C1_LEN: usize = 4;
const SFID_ID_SEGMENT_N9_LEN: usize = 9;
const SFID_ID_SEGMENT_D8_LEN: usize = 8;
const SFID_ID_MAX_BYTES: usize = 96;

/// 生成机构 SFID + QR1 安装授权二维码。
///
/// SFID-CPMS QR v1 协议：生成 site_sfid 和 install_token，
/// 用 SFID 主密钥签名，返回 QR1 payload。
pub(crate) async fn generate_cpms_institution_sfid_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GenerateCpmsInstitutionSfidInput>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
        None => match input.province.as_deref() {
            Some(raw) if !raw.trim().is_empty() => raw.trim().to_string(),
            _ => return api_error(StatusCode::BAD_REQUEST, 1001, "province is required"),
        },
    };
    if province.chars().count() > MAX_PROVINCE_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province too long");
    }
    let city = input.city.trim().to_string();
    let institution = input.institution.trim().to_string();
    if city.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city is required");
    }
    if institution.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "institution is required");
    }
    if city.chars().count() > MAX_CITY_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city too long");
    }
    if institution.chars().count() > MAX_INSTITUTION_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "institution too long");
    }
    let institution_name = input.institution_name.as_deref().unwrap_or("").trim().to_string();
    if institution_name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "institution_name is required");
    }
    if institution_name.chars().count() > 30 {
        return api_error(StatusCode::BAD_REQUEST, 1001, "institution_name too long (max 30)");
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

        // 从 site_sfid 的 r5 段提取省代码（前两位字母）
        let province_code = extract_province_code_from_sfid(&site_sfid);

        // 生成一次性安装令牌
        let install_token = Uuid::new_v4().to_string().replace('-', "");

        // 用 SFID 主密钥签名 QR1
        let sign_source = format!(
            "sfid-cpms-v1|install|{}|{}",
            site_sfid, install_token
        );
        let signature = match sign_with_main_key(&state, &sign_source) {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "sign QR1 failed",
                )
            }
        };

        // 获取 RSA 公钥 PEM
        let rsa_pubkey_pem = match key_admins::rsa_blind::get_public_key_pem() {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "RSA public key not available",
                )
            }
        };

        // 构造 QR1 payload
        let rsa_raw = strip_pem_envelope(&rsa_pubkey_pem);
        let qr1 = serde_json::json!({
            "proto": "SFID_CPMS_V1",
            "type": "INSTALL",
            "sfid": site_sfid,
            "token": install_token,
            "rsa": rsa_raw,
            "sig": signature,
        });
        let qr1_payload = match serde_json::to_string(&qr1) {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "serialize QR1 failed",
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
                install_token: install_token.clone(),
                install_token_status: InstallTokenStatus::Pending,
                status: CpmsSiteStatus::Pending,
                version: 1,
                province_code: province_code.clone(),
                admin_province: province.clone(),
                city_name: city.clone(),
                institution_code: institution.clone(),
                institution_name: institution_name.clone(),
                qr1_payload: qr1_payload.clone(),
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
                "site_sfid={} province={} city={} institution={} province_code={}",
                site_sfid, province, city, institution, province_code,
            ),
        );
        drop(store);
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: GenerateCpmsInstallOutput {
                site_sfid,
                qr1_payload,
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

/// 处理 QR2 注册请求，返回 QR3 匿名证书。
///
/// SFID-CPMS QR v1 协议：校验 install_token，执行 RSABSSA 盲签名，
/// 返回 QR3 payload 供 CPMS 解盲。
pub(crate) async fn register_cpms(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsRegisterInput>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // 解析 QR2
    let qr2: CpmsRegisterReqPayload = match serde_json::from_str(input.qr_payload.trim()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid QR2 payload"),
    };
    if qr2.r#type != "REGISTER" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "type must be REGISTER");
    }
    if qr2.blind.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "blind is required");
    }

    // 查找 sfid 并校验 token
    let province_code = {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let site = match store.cpms_site_keys.get_mut(qr2.sfid.trim()) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "sfid not found"),
        };
        if site.install_token_status != InstallTokenStatus::Pending {
            return api_error(StatusCode::CONFLICT, 1007, "token already used or revoked");
        }
        if site.install_token != qr2.token.trim() {
            return api_error(StatusCode::UNAUTHORIZED, 2004, "token mismatch");
        }
        // 标记 token 已使用，状态改为 ACTIVE
        site.install_token_status = InstallTokenStatus::Used;
        site.status = CpmsSiteStatus::Active;
        site.version += 1;
        site.updated_by = Some(ctx.admin_pubkey.clone());
        site.updated_at = Some(Utc::now());
        let pc = site.province_code.clone();
        append_audit_log(
            &mut store,
            "CPMS_REGISTER",
            &ctx.admin_pubkey,
            Some(qr2.sfid.clone()),
            None,
            "SUCCESS",
            format!("site_sfid={} province_code={}", qr2.sfid, pc),
        );
        drop(store);
        pc
    };

    // 执行 RSABSSA 盲签名
    let blind_anon_req_bytes = match hex::decode(qr2.blind.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "blind hex decode failed"),
    };
    let blind_anon_sig = match key_admins::rsa_blind::blind_sign(&blind_anon_req_bytes, &province_code) {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("blind sign failed: {e}"),
            )
        }
    };

    // 构造 QR3
    let qr3 = serde_json::json!({
        "proto": "SFID_CPMS_V1",
        "type": "CERT",
        "prov": province_code,
        "bsig": format!("0x{}", hex::encode(&blind_anon_sig)),
    });
    let qr3_payload = match serde_json::to_string(&qr3) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "serialize QR3 failed"),
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CpmsRegisterOutput { qr3_payload },
    })
    .into_response()
}

/// 处理 QR4 档案业务二维码，验证并录入档案。
///
/// SFID-CPMS QR v1 协议：验证 anon_cert 签名、archive_sig，
/// 去重后录入 imported_archives。
pub(crate) async fn archive_import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsArchiveImportInput>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // 解析 QR4
    let qr4: CpmsArchiveQrPayload = match serde_json::from_str(input.qr_payload.trim()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid QR4 payload"),
    };
    if qr4.r#type != "ARCHIVE" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "qr_type must be CPMS_ARCHIVE");
    }

    // 1. 验证 anon_cert.sfid_sig（RSA 盲签名验签）
    let sfid_sig_bytes = match hex::decode(qr4.cert.sig.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "anon_cert.sfid_sig hex decode failed"),
    };
    let msg_randomizer = qr4.cert.mr.as_deref().and_then(|r| {
        hex::decode(r.trim().trim_start_matches("0x")).ok()
    });
    let cert_valid = match key_admins::rsa_blind::verify_anon_cert(
        &qr4.cert.prov,
        &qr4.cert.pk,
        &sfid_sig_bytes,
        msg_randomizer.as_deref(),
    ) {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("anon_cert verify error: {e}"),
            )
        }
    };
    if !cert_valid {
        return api_error(StatusCode::UNAUTHORIZED, 2004, "anon_cert.sfid_sig invalid");
    }

    // 2. 验证 province_code 一致性
    if qr4.cert.prov != qr4.prov {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province_code mismatch between anon_cert and QR4");
    }

    // 3. 验证 archive_sig（sr25519）
    let archive_sign_source = format!(
        "sfid-cpms-v1|archive|{}|{}|{}|{}",
        qr4.prov, qr4.ano, qr4.cs, qr4.ve
    );
    let anon_pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&qr4.cert.pk) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "anon_pubkey format invalid"),
    };
    let archive_sig_bytes = match hex::decode(qr4.sig.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "archive_sig hex decode failed"),
    };
    if !verify_sr25519_signature(&anon_pubkey_bytes, &archive_sign_source, &archive_sig_bytes) {
        return api_error(StatusCode::UNAUTHORIZED, 2004, "archive_sig invalid");
    }

    // 4. 去重 + 录入
    let anon_cert_json = match serde_json::to_string(&qr4.cert) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "serialize anon_cert failed"),
    };
    let anon_cert_hash = hex::encode(Blake2b256::digest(anon_cert_json.as_bytes()));
    // 以 anon_cert.province_code 为准落库
    let province_code = qr4.cert.prov.clone();

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store.imported_archives.contains_key(qr4.ano.as_str()) {
        return api_error(StatusCode::CONFLICT, 1007, "archive_no already imported");
    }
    store.imported_archives.insert(
        qr4.ano.clone(),
        ImportedArchive {
            archive_no: qr4.ano.clone(),
            province_code: province_code.clone(),
            anon_cert_hash,
            imported_at: Utc::now(),
            status: ArchiveImportStatus::Active,
        },
    );
    append_audit_log(
        &mut store,
        "CPMS_ARCHIVE_IMPORT",
        &ctx.admin_pubkey,
        Some(qr4.ano.clone()),
        None,
        "SUCCESS",
        format!(
            "archive_no={} province_code={} citizen_status={} voting_eligible={}",
            qr4.ano, province_code, qr4.cs, qr4.ve
        ),
    );
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CpmsArchiveImportOutput {
            archive_no: qr4.ano,
            province_code,
            status: "ACTIVE",
        },
    })
    .into_response()
}

/// 作废安装令牌。
pub(crate) async fn revoke_install_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site = match store.cpms_site_keys.get_mut(site_sfid.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "site_sfid not found"),
    };
    site.install_token_status = InstallTokenStatus::Revoked;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    append_audit_log(
        &mut store,
        "CPMS_INSTALL_TOKEN_REVOKE",
        &ctx.admin_pubkey,
        Some(site_sfid.to_string()),
        None,
        "SUCCESS",
        format!("site_sfid={}", site_sfid),
    );
    drop(store);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "revoked",
    })
    .into_response()
}

/// 重新签发安装令牌（QR1）。
pub(crate) async fn reissue_install_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site_sfid_validated = match validate_sfid_id_format(site_sfid.trim()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let new_token = Uuid::new_v4().to_string().replace('-', "");
    let sign_source = format!("sfid-cpms-v1|install|{}|{}", site_sfid_validated, new_token);
    let signature = match sign_with_main_key(&state, &sign_source) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "sign QR1 failed"),
    };
    let rsa_pubkey_pem = match key_admins::rsa_blind::get_public_key_pem() {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "RSA public key not available"),
    };
    let rsa_raw = strip_pem_envelope(&rsa_pubkey_pem);
    let qr1 = serde_json::json!({
        "proto": "SFID_CPMS_V1",
        "type": "INSTALL",
        "sfid": site_sfid_validated,
        "token": new_token,
        "rsa": rsa_raw,
        "sig": signature,
    });
    let qr1_payload = match serde_json::to_string(&qr1) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "serialize QR1 failed"),
    };

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site = match store.cpms_site_keys.get_mut(site_sfid_validated.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "site_sfid not found"),
    };
    if site.install_token_status == InstallTokenStatus::Pending {
        return api_error(StatusCode::CONFLICT, 1007, "install_token is still pending, cannot reissue");
    }
    site.install_token = new_token;
    site.install_token_status = InstallTokenStatus::Pending;
    site.status = CpmsSiteStatus::Pending;
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    append_audit_log(
        &mut store,
        "CPMS_INSTALL_TOKEN_REISSUE",
        &ctx.admin_pubkey,
        Some(site_sfid_validated.clone()),
        None,
        "SUCCESS",
        format!("site_sfid={}", site_sfid_validated),
    );
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: GenerateCpmsInstallOutput {
            site_sfid: site_sfid_validated,
            qr1_payload,
        },
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
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
        .map(|site| cpms_site_keys_to_list_row(site, &store))
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
    let ctx = match require_institution_or_key_admin(&state, &headers) {
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
        let created_by_name = resolve_admin_display_name(&store, &output.created_by);
    let response_row = cpms_site_keys_to_list_row_simple(&output, created_by_name);
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
    let created_by_name = resolve_admin_display_name(&store, &output.created_by);
    let response_row = cpms_site_keys_to_list_row_simple(&output, created_by_name);
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
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: response_row,
    })
    .into_response()
}

fn cpms_site_keys_to_list_row(site: &CpmsSiteKeys, store: &Store) -> CpmsSiteKeysListRow {
    let created_by_name = resolve_admin_display_name(store, &site.created_by);
    CpmsSiteKeysListRow {
        site_sfid: site.site_sfid.clone(),
        install_token_status: site.install_token_status.clone(),
        status: site.status.clone(),
        version: site.version,
        province_code: site.province_code.clone(),
        admin_province: site.admin_province.clone(),
        city_name: site.city_name.clone(),
        institution_code: site.institution_code.clone(),
        institution_name: site.institution_name.clone(),
        qr1_payload: site.qr1_payload.clone(),
        created_by: site.created_by.clone(),
        created_by_name,
        created_at: site.created_at,
        updated_by: site.updated_by.clone(),
        updated_at: site.updated_at,
    }
}

fn cpms_site_keys_to_list_row_simple(site: &CpmsSiteKeys, created_by_name: String) -> CpmsSiteKeysListRow {
    CpmsSiteKeysListRow {
        site_sfid: site.site_sfid.clone(),
        install_token_status: site.install_token_status.clone(),
        status: site.status.clone(),
        version: site.version,
        province_code: site.province_code.clone(),
        admin_province: site.admin_province.clone(),
        city_name: site.city_name.clone(),
        institution_code: site.institution_code.clone(),
        institution_name: site.institution_name.clone(),
        qr1_payload: site.qr1_payload.clone(),
        created_by: site.created_by.clone(),
        created_by_name,
        created_at: site.created_at,
        updated_by: site.updated_by.clone(),
        updated_at: site.updated_at,
    }
}

fn strip_pem_envelope(pem: &str) -> String {
    pem.lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("")
}

fn resolve_admin_display_name(store: &Store, pubkey: &str) -> String {
    if let Some(admin) = store.admin_users_by_pubkey.get(pubkey) {
        let role_label = match admin.role {
            AdminRole::KeyAdmin => "密钥管理员",
            AdminRole::InstitutionAdmin => "机构管理员",
            AdminRole::SystemAdmin => "系统管理员",
        };
        if let Some(province) = store.super_admin_province_by_pubkey.get(pubkey) {
            format!("{}{}", province, role_label)
        } else if !admin.admin_name.is_empty() {
            format!("{} ({})", admin.admin_name, role_label)
        } else {
            role_label.to_string()
        }
    } else {
        "未知".to_string()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ChainInstitutionRegisterReceipt {
    pub(super) genesis_hash: String,
    pub(super) sfid_id: String,
    pub(super) register_nonce: String,
    pub(super) signature: String,
    pub(super) tx_hash: String,
    pub(super) block_number: u64,
}

pub(super) fn validate_sfid_id_format(raw: &str) -> Result<String, &'static str> {
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
        || !segments[1]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
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
    // 中文注释：机构 sfid_id 登记统一要求当前服务端 signer 必须就是链上 MAIN。
    key_admins::validate_active_main_signer_with_keyring(state)
        .map_err(|e| format!("current signer is not chain main: {e}"))?;
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

pub(super) async fn submit_register_sfid_institution_extrinsic(
    state: &AppState,
    site_sfid: &str,
    institution_name: &str,
) -> Result<ChainInstitutionRegisterReceipt, String> {
    let sfid_id = validate_sfid_id_format(site_sfid)
        .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let register_nonce = Uuid::new_v4().to_string();
    let credential =
        build_institution_credential(state, sfid_id.as_str(), institution_name, register_nonce)
            .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let ws_url = resolve_chain_ws_url()
        .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    // 中文注释：
    // citizenchain 是 PoW 链，GRANDPA finality 显著落后 best block，subxt 0.43 默认行为
    // (从 finalized 块读 nonce / 取 era birth block / 等 finalize) 在这里全部踩坑。
    // 必须做三件事：
    //   ① 用 legacy RPC system_accountNextIndex 取 best+pool 视图的 nonce，避免 Stale
    //   ② extrinsic 强制 immortal，避免 mortal era 的 birth-block-hash 在 best 视图里查不到
    //      被链端判定为 AncientBirthBlock
    //   ③ submit_and_watch 后只等 InBestBlock 而非 wait_for_finalized，避免 120s 超时
    // 详见 ADR `04-decisions/sfid/2026-04-07-subxt-0.43-pow-chain-quirks.md`。
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.clone())
        .await
        .map_err(|e| {
            format!("register_sfid_institution submit failed: chain websocket connect failed: {e}")
        })?;
    // ① legacy RPC client，用于显式取 nonce
    let rpc_client = subxt::backend::rpc::RpcClient::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| {
            format!("register_sfid_institution submit failed: legacy rpc connect failed: {e}")
        })?;
    let legacy_rpc =
        subxt::backend::legacy::LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let (signer_pubkey, signer_seed) = resolve_chain_signer_material(state)
        .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let signer_account = AccountId32(
        parse_account_id32(signer_pubkey.as_str())
            .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?,
    );
    let chain_nonce = legacy_rpc
        .system_account_next_index(&signer_account)
        .await
        .map_err(|e| {
            format!("register_sfid_institution submit failed: fetch account nonce failed: {e}")
        })?;
    let payload = tx(
        "DuoqianManagePow",
        "register_sfid_institution",
        vec![
            Value::from_bytes(sfid_id.as_bytes().to_vec()),
            Value::from_bytes(credential.name.as_bytes().to_vec()),
            Value::from_bytes(credential.register_nonce.as_bytes().to_vec()),
            Value::from_bytes(hex::decode(credential.signature.as_str()).map_err(|e| {
                format!("register_sfid_institution submit failed: signature hex decode failed: {e}")
            })?),
        ],
    );
    // ② immortal + 显式 nonce(对应注释 ①/②)
    let params = subxt::config::DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
        .immortal()
        .nonce(chain_nonce)
        .build();
    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, params)
        .await
        .map_err(|e| {
            format!("register_sfid_institution submit failed: build extrinsic failed: {e}")
        })?;
    let signing_key =
        key_admins::chain_keyring::try_load_signing_key_from_seed(signer_seed.expose_secret())
            .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let signature = signing_key.sign(&partial_tx.signer_payload()).0;
    let extrinsic = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(extrinsic.hash().as_ref()));

    let mut submitted = extrinsic.submit_and_watch().await.map_err(|e| {
        format!("register_sfid_institution submit failed: submit_and_watch failed: {e}")
    })?;
    // ③ 只等 InBestBlock（对应上方注释 ③）。dispatch 已发生、事件可读，无需等 finalize。
    let in_block = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        async {
            use subxt::tx::TxStatus;
            loop {
                match submitted.next().await {
                    Some(Ok(TxStatus::InBestBlock(b))) => return Ok::<_, String>(b),
                    Some(Ok(TxStatus::InFinalizedBlock(b))) => return Ok(b),
                    Some(Ok(TxStatus::Error { message }))
                    | Some(Ok(TxStatus::Invalid { message }))
                    | Some(Ok(TxStatus::Dropped { message })) => {
                        return Err(format!("tx pool reported: {message}"));
                    }
                    Some(Ok(_)) => continue,
                    Some(Err(e)) => return Err(format!("tx watch stream error: {e}")),
                    None => return Err("tx watch stream closed unexpectedly".to_string()),
                }
            }
        },
    )
    .await
    .map_err(|_| {
        "register_sfid_institution submit failed: timed out waiting for in-block inclusion".to_string()
    })?
    .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
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
        genesis_hash: credential.genesis_hash,
        sfid_id,
        register_nonce: credential.register_nonce,
        signature: credential.signature,
        tx_hash,
        block_number,
    })
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

fn is_trusted_attestor_pubkey(state: &AppState, public_key: &str) -> bool {
    let Some(candidate) = parse_sr25519_pubkey(public_key) else {
        return false;
    };
    if key_admins::validate_active_main_signer_with_keyring(state).is_err() {
        return false;
    }
    let current = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => return false,
    };
    parse_sr25519_pubkey(current.as_str())
        .map(|v| v == candidate)
        .unwrap_or(false)
}

/// 从 site_sfid 的 r5 段提取两位字母省代码。
///
/// site_sfid 格式：`{a3}-{r5}-{t2p1c1}-{n9}-{d8}`
/// r5 段 = 2 位字母省码 + 3 位数字城市码。
pub(super) fn extract_province_code_from_sfid(site_sfid: &str) -> String {
    let segments: Vec<&str> = site_sfid.split('-').collect();
    if segments.len() >= 2 && segments[1].len() >= 2 {
        segments[1][..2].to_string()
    } else {
        String::new()
    }
}

/// 用 SFID 主密钥（sr25519）对消息签名，返回 hex 编码签名。
fn sign_with_main_key(state: &AppState, message: &str) -> Result<String, String> {
    let seed = state
        .signing_seed_hex
        .read()
        .map_err(|_| "seed lock poisoned".to_string())?;
    let keypair =
        key_admins::chain_keyring::try_load_signing_key_from_seed(seed.expose_secret())
            .map_err(|e| format!("load signing key failed: {e}"))?;
    let sig = keypair.sign(message.as_bytes());
    Ok(format!("0x{}", hex::encode(sig.0)))
}

/// 验证 sr25519 签名。
pub(crate) fn verify_sr25519_signature(pubkey_bytes: &[u8; 32], message: &str, signature: &[u8]) -> bool {
    use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
    let pk = match Sr25519PublicKey::from_bytes(pubkey_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let sig = match Sr25519Signature::from_bytes(signature) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let ctx = signing_context(b"substrate");
    pk.verify(ctx.bytes(message.as_bytes()), &sig).is_ok()
}
