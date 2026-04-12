use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sp_core::Pair;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};

use crate::chain::runtime_align::build_institution_credential_with_province;
use crate::key_admins::signer_router::resolve_business_signer;
use crate::login::AdminAuthContext;
// 中文注释:SFID 工具统一从 crate::sfid 拿,见 feedback_sfid_module_is_single_entry.md
use crate::sfid::{generate_sfid_code, validate_sfid_id_format, GenerateSfidInput};
use crate::*;

type Blake2b256 = Blake2b<U32>;

const MAX_CITY_CHARS: usize = 100;
const MAX_INSTITUTION_CHARS: usize = 100;
const MAX_PROVINCE_CHARS: usize = 100;
const MAX_STATUS_REASON_CHARS: usize = 500;

// 中文注释:Phase 2 Day 3 Round 2 迁移 cpms_site_keys 到 sharded_store。
// 根据 site_sfid 定位所在省份分片:
//   - sheng-admin 已经锁定 province scope,直接用 scope 省即可;
//   - key-admin (scope=None) 需要跨省扫描定位,开销可接受(admin 低频操作)。
pub(crate) async fn resolve_site_province_via_shard(
    state: &AppState,
    site_sfid: &str,
    admin_province_scope: Option<&str>,
) -> Result<String, (StatusCode, &'static str)> {
    if let Some(p) = admin_province_scope {
        return Ok(p.to_string());
    }
    let mut found: Option<String> = None;
    let site_sfid_owned = site_sfid.to_string();
    state
        .sharded_store
        .for_each_province(|province, shard| {
            if found.is_some() {
                return;
            }
            if shard.cpms_site_keys.contains_key(&site_sfid_owned) {
                found = Some(province.to_string());
            }
        })
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "shard scan failed"))?;
    found.ok_or((StatusCode::NOT_FOUND, "site_sfid not found"))
}

/// Phase 2 Day 3 Round 2:两段提交 audit_log。
/// 先 cpms 写分片(已经成功),再拿 legacy store 短锁写 audit_log。
/// 审计日志写失败只记 WARN,返回 OK(业务数据不丢)。
#[allow(clippy::too_many_arguments)]
fn append_cpms_audit_log_best_effort(
    state: &AppState,
    action: &'static str,
    actor_pubkey: &str,
    target_pubkey: Option<String>,
    target_archive_no: Option<String>,
    result: &'static str,
    detail: String,
) {
    match state.store.write() {
        Ok(mut store) => {
            append_audit_log(
                &mut store,
                action,
                actor_pubkey,
                target_pubkey,
                target_archive_no,
                result,
                detail,
            );
        }
        Err(e) => {
            tracing::warn!(action, error = %e, "append_audit_log failed (cpms shard write already committed)");
        }
    }
}

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
    // ── 查找已有机构,确定 site_sfid ──
    // 公安局由 reconcile 预创建,sfid_finalized=false 表示尚未固化;
    // 首次生成 QR1 时重新生成 sfid_id 并固化,此后永久复用。
    let (old_sfid_id, already_finalized) = match state.store.read() {
        Ok(store) => {
            let found = store.multisig_institutions.values().find(|i| {
                i.province == province
                    && i.city == city
                    && i.institution_code == institution
            });
            match found {
                Some(inst) => (inst.sfid_id.clone(), inst.sfid_finalized),
                None => {
                    return api_error(
                        StatusCode::NOT_FOUND,
                        1004,
                        "institution not found, reconcile may not have run",
                    );
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "store read failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
        }
    };

    let site_sfid = if already_finalized {
        // sfid 已固化,直接复用
        old_sfid_id.clone()
    } else {
        // 首次生成 QR1:生成新 sfid_id 并替换
        let random_account = Uuid::new_v4().to_string();
        let new_sfid = match generate_sfid_code(GenerateSfidInput {
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
        let new_sfid = match validate_sfid_id_format(new_sfid.as_str()) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
        };

        // 替换 legacy store: remove 旧 key → insert 新 key,设 sfid_finalized=true
        match state.store.write() {
            Ok(mut store) => {
                if let Some(mut inst) = store.multisig_institutions.remove(&old_sfid_id) {
                    inst.sfid_id = new_sfid.clone();
                    inst.sfid_finalized = true;
                    store.multisig_institutions.insert(new_sfid.clone(), inst);
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "store write failed for sfid finalize");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store write failed");
            }
        }

        // 替换 sharded_store 分片
        let old_key = old_sfid_id.clone();
        let new_key = new_sfid.clone();
        let replace_result = state
            .sharded_store
            .write_province(&province, move |shard| {
                if let Some(mut inst) = shard.multisig_institutions.remove(&old_key) {
                    inst.sfid_id = new_key.clone();
                    inst.sfid_finalized = true;
                    shard.multisig_institutions.insert(new_key, inst);
                }
                Ok::<(), &str>(())
            })
            .await;
        if let Err(e) = replace_result {
            tracing::warn!(error = %e, "shard write failed for sfid finalize (legacy already committed)");
        }

        tracing::info!(
            old_sfid = %old_sfid_id,
            new_sfid = %new_sfid,
            province = %province,
            city = %city,
            "sfid_id finalized on first QR1 generation"
        );
        new_sfid
    };

    let site_sfid = match validate_sfid_id_format(site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
    };
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
        "province_name": province,
        "city_name": city,
        "institution_name": institution_name,
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
    // 写 cpms_site_keys 到 sharded_store
    let site_sfid_key = site_sfid.clone();
    let new_site = CpmsSiteKeys {
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
    };
    let new_site_for_legacy = new_site.clone();
    let insert_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            // 已有同 sfid 的站点记录时覆盖(多次生成 QR1 场景)
            shard.cpms_site_keys.insert(site_sfid_key.clone(), new_site);
            Ok(())
        })
        .await;
    match insert_result {
        Ok(Ok(())) => {}
        Ok(Err(())) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "shard write failed",
            )
        }
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "shard write failed",
            )
        }
    }

    // 双写过渡期:sharded_store + legacy store 同步写
    {
        match state.store.write() {
            Ok(mut store) => {
                store.cpms_site_keys.insert(site_sfid.clone(), new_site_for_legacy);
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (cpms generate, shard already committed)");
            }
        }
    }

    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_SFID_GENERATE",
        &ctx.admin_pubkey,
        Some(site_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} province={} city={} institution={} province_code={} finalized={}",
            site_sfid, province, city, institution, province_code, !already_finalized,
        ),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: GenerateCpmsInstallOutput {
            site_sfid,
            qr1_payload,
        },
    })
    .into_response()
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

    // Phase 2 Day 3 Round 2 迁移 cpms_site_keys 到 sharded_store:先定位省,再写分片
    let sfid_trimmed = qr2.sfid.trim().to_string();
    let province = match resolve_site_province_via_shard(
        &state,
        &sfid_trimmed,
        ctx.admin_province.as_deref(),
    )
    .await
    {
        Ok(v) => v,
        Err((code, msg)) => return api_error(code, 1004, msg),
    };
    let token_expected = qr2.token.trim().to_string();
    let actor_pubkey = ctx.admin_pubkey.clone();
    let sfid_trimmed_for_legacy = sfid_trimmed.clone();
    let write_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let site = match shard.cpms_site_keys.get_mut(sfid_trimmed.as_str()) {
                Some(v) => v,
                None => return Err((StatusCode::NOT_FOUND, 1004, "sfid not found")),
            };
            if site.install_token_status != InstallTokenStatus::Pending {
                return Err((StatusCode::CONFLICT, 1007, "token already used or revoked"));
            }
            if site.install_token != token_expected {
                return Err((StatusCode::UNAUTHORIZED, 2004, "token mismatch"));
            }
            site.install_token_status = InstallTokenStatus::Used;
            site.status = CpmsSiteStatus::Active;
            site.version += 1;
            site.updated_by = Some(actor_pubkey.clone());
            site.updated_at = Some(Utc::now());
            Ok(site.province_code.clone())
        })
        .await;
    let province_code = match write_result {
        Ok(Ok(pc)) => pc,
        Ok(Err((code, biz, msg))) => return api_error(code, biz, msg),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "shard write failed",
            )
        }
    };

    // 双写过渡期:sharded_store + legacy store 同步写
    {
        match state.store.write() {
            Ok(mut store) => {
                if let Some(site) = store.cpms_site_keys.get_mut(&sfid_trimmed_for_legacy) {
                    site.install_token_status = InstallTokenStatus::Used;
                    site.status = CpmsSiteStatus::Active;
                    site.version += 1;
                    site.updated_by = Some(ctx.admin_pubkey.clone());
                    site.updated_at = Some(Utc::now());
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (cpms register, shard already committed)");
            }
        }
    }

    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_REGISTER",
        &ctx.admin_pubkey,
        Some(qr2.sfid.clone()),
        None,
        "SUCCESS",
        format!("site_sfid={} province_code={}", qr2.sfid, province_code),
    );

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
/// Phase 2 Day 3:cpms_site_keys 迁移到 sharded_store
pub(crate) async fn revoke_install_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_trimmed = site_sfid.trim().to_string();
    let province = match resolve_site_province_via_shard(
        &state,
        &sfid_trimmed,
        ctx.admin_province.as_deref(),
    )
    .await
    {
        Ok(v) => v,
        Err((code, msg)) => return api_error(code, 1004, msg),
    };
    let actor_pubkey = ctx.admin_pubkey.clone();
    let sfid_for_closure = sfid_trimmed.clone();
    let write_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let site = match shard.cpms_site_keys.get_mut(sfid_for_closure.as_str()) {
                Some(v) => v,
                None => return Err("site_sfid not found"),
            };
            site.install_token_status = InstallTokenStatus::Revoked;
            site.updated_by = Some(actor_pubkey.clone());
            site.updated_at = Some(Utc::now());
            Ok(())
        })
        .await;
    match write_result {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => return api_error(StatusCode::NOT_FOUND, 1004, msg),
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e),
    }

    // 双写过渡期:sharded_store + legacy store 同步写
    {
        match state.store.write() {
            Ok(mut store) => {
                if let Some(site) = store.cpms_site_keys.get_mut(&sfid_trimmed) {
                    site.install_token_status = InstallTokenStatus::Revoked;
                    site.updated_by = Some(ctx.admin_pubkey.clone());
                    site.updated_at = Some(Utc::now());
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (cpms revoke, shard already committed)");
            }
        }
    }

    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_INSTALL_TOKEN_REVOKE",
        &ctx.admin_pubkey,
        Some(sfid_trimmed.clone()),
        None,
        "SUCCESS",
        format!("site_sfid={}", sfid_trimmed),
    );
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

    // 先定位省份，再读站点元数据用于 QR1 名称字段
    let province = match resolve_site_province_via_shard(
        &state,
        &site_sfid_validated,
        ctx.admin_province.as_deref(),
    )
    .await
    {
        Ok(v) => v,
        Err((code, msg)) => return api_error(code, 1004, msg),
    };
    let sfid_for_read = site_sfid_validated.clone();
    let site_meta = state
        .sharded_store
        .read_province(&province, move |shard| {
            shard.cpms_site_keys.get(&sfid_for_read).map(|s| {
                (s.admin_province.clone(), s.city_name.clone(), s.institution_name.clone())
            })
        })
        .await
        .ok()
        .flatten();
    let (prov_name, city_name, inst_name) = site_meta.unwrap_or_else(|| {
        (province.clone(), String::new(), String::new())
    });

    let qr1 = serde_json::json!({
        "proto": "SFID_CPMS_V1",
        "type": "INSTALL",
        "sfid": site_sfid_validated,
        "token": new_token,
        "rsa": rsa_raw,
        "sig": signature,
        "province_name": prov_name,
        "city_name": city_name,
        "institution_name": inst_name,
    });
    let qr1_payload = match serde_json::to_string(&qr1) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "serialize QR1 failed"),
    };
    let actor_pubkey = ctx.admin_pubkey.clone();
    let sfid_for_closure = site_sfid_validated.clone();
    let new_token_clone = new_token.clone();
    let qr1_payload_clone = qr1_payload.clone();
    let write_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let site = match shard.cpms_site_keys.get_mut(sfid_for_closure.as_str()) {
                Some(v) => v,
                None => return Err("site_sfid not found"),
            };
            site.install_token = new_token_clone;
            site.install_token_status = InstallTokenStatus::Pending;
            site.status = CpmsSiteStatus::Pending;
            site.qr1_payload = qr1_payload_clone;
            site.version += 1;
            site.updated_by = Some(actor_pubkey.clone());
            site.updated_at = Some(Utc::now());
            Ok(())
        })
        .await;
    match write_result {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => {
            let code = if msg.contains("pending") {
                StatusCode::CONFLICT
            } else {
                StatusCode::NOT_FOUND
            };
            return api_error(code, 1004, msg);
        }
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e),
    }

    // 双写过渡期:sharded_store + legacy store 同步写
    {
        match state.store.write() {
            Ok(mut store) => {
                if let Some(site) = store.cpms_site_keys.get_mut(&site_sfid_validated) {
                    site.install_token = new_token.clone();
                    site.install_token_status = InstallTokenStatus::Pending;
                    site.status = CpmsSiteStatus::Pending;
                    site.qr1_payload = qr1_payload.clone();
                    site.version += 1;
                    site.updated_by = Some(ctx.admin_pubkey.clone());
                    site.updated_at = Some(Utc::now());
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (cpms reissue, shard already committed)");
            }
        }
    }

    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_INSTALL_TOKEN_REISSUE",
        &ctx.admin_pubkey,
        Some(site_sfid_validated.clone()),
        None,
        "SUCCESS",
        format!("site_sfid={}", site_sfid_validated),
    );

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
    // Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store
    let province = match resolve_site_province_via_shard(
        &state,
        &site_sfid,
        ctx.admin_province.as_deref(),
    )
    .await
    {
        Ok(v) => v,
        Err((code, msg)) => return api_error(code, 1004, msg),
    };
    let sfid_for_closure = site_sfid.clone();
    let scope_province = ctx.admin_province.clone();
    let write_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let existing = match shard.cpms_site_keys.get(sfid_for_closure.as_str()) {
                Some(v) => v.clone(),
                None => return Err((StatusCode::NOT_FOUND, "cpms site not found")),
            };
            if !in_scope_cpms_site(&existing, scope_province.as_deref()) {
                return Err((
                    StatusCode::FORBIDDEN,
                    "cannot manage other province institutions",
                ));
            }
            if existing.status != CpmsSiteStatus::Pending {
                return Err((
                    StatusCode::CONFLICT,
                    "only pending cpms site can be deleted",
                ));
            }
            let info = (existing.site_sfid.clone(), existing.status.clone(), existing.version);
            shard.cpms_site_keys.remove(sfid_for_closure.as_str());
            Ok(info)
        })
        .await;
    let (existing_sfid, existing_status, existing_version) = match write_result {
        Ok(Ok(info)) => info,
        Ok(Err((code, msg))) => return api_error(code, 1004, msg),
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e),
    };

    // 双写过渡期:sharded_store + legacy store 同步写
    {
        match state.store.write() {
            Ok(mut store) => {
                store.cpms_site_keys.remove(&site_sfid);
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (cpms delete, shard already committed)");
            }
        }
    }

    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_KEYS_DELETE",
        &ctx.admin_pubkey,
        Some(existing_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} status={:?} version={}",
            existing_sfid, existing_status, existing_version
        ),
    );
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
    // Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store
    // 按 admin_province scope 决定读一省还是遍历全部
    let scope_province = ctx.admin_province.clone();
    let mut sites: Vec<CpmsSiteKeys> = Vec::new();
    if let Some(ref p) = scope_province {
        let read_result = state
            .sharded_store
            .read_province(p, |shard| {
                shard
                    .cpms_site_keys
                    .values()
                    .filter(|site| in_scope_cpms_site(site, scope_province.as_deref()))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .await;
        match read_result {
            Ok(v) => sites = v,
            Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e),
        }
    } else {
        // KEY_ADMIN：遍历所有省
        let read_result = state
            .sharded_store
            .for_each_province(|_prov, shard| {
                sites.extend(shard.cpms_site_keys.values().cloned());
            })
            .await;
        if let Err(e) = read_result {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e);
        }
    }
    // 拿 legacy store 短锁做 admin 名称解析
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<CpmsSiteKeysListRow> = sites
        .iter()
        .map(|site| cpms_site_keys_to_list_row(site, &store))
        .collect();
    drop(store);
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

/// 任务卡 `20260408-sfid-public-security-cpms-embed`:
/// 按市公安局机构 sfid_id 反查其 CPMS 站点(`cpms_site_keys`)。
///
/// 中文注释:`multisig_institutions.sfid_id` 和 `cpms_site_keys.site_sfid`
/// **不是同一个值**(CPMS 站点的 site_sfid 是生成安装二维码时随机派生的),
/// 所以用 `(admin_province, city_name, institution_code)` 元组匹配——公安局
/// 每市唯一,元组保证一一对应。返回 `null` 表示该公安局尚未生成过 CPMS 站点。
pub(crate) async fn get_cpms_site_by_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_id): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_institution_or_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_id = sfid_id.trim().to_string();
    if sfid_id.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "sfid_id is required");
    }

    // Phase 2 Day 3 Round 2:机构也从 sharded_store 读
    let province_code = extract_province_code_from_sfid(&sfid_id);
    let province_name = match crate::sfid::province::province_name_by_code(&province_code) {
        Some(n) => n.to_string(),
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "cannot resolve province from sfid_id"),
    };
    let sfid_id_r = sfid_id.clone();
    let inst_result = state
        .sharded_store
        .read_province(&province_name, move |shard| {
            shard.multisig_institutions.get(&sfid_id_r).cloned()
        })
        .await;
    let inst = match inst_result {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("shard read: {e}")),
    };
    // 省级管理员只能看本省
    if let Some(scope_province) = ctx.admin_province.as_deref() {
        if inst.province != scope_province {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot view other province institutions",
            );
        }
    }
    // institution 的省已知，读同省分片的 cpms_site_keys
    let inst_province = inst.province.clone();
    let inst_city = inst.city.clone();
    let inst_code = inst.institution_code.clone();
    let province_key = inst_province.clone();
    let matched_site: Option<CpmsSiteKeys> = match state
        .sharded_store
        .read_province(&province_key, move |shard| {
            shard
                .cpms_site_keys
                .values()
                .find(|site| {
                    site.admin_province == inst_province
                        && site.city_name == inst_city
                        && site.institution_code == inst_code
                })
                .cloned()
        })
        .await
    {
        Ok(v) => v,
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e),
    };
    // 审计用的 admin display name 仍走 legacy store 短读
    let matched = match state.store.read() {
        Ok(store) => matched_site.map(|site| cpms_site_keys_to_list_row(&site, &store)),
        Err(_) => matched_site.map(|site| cpms_site_keys_to_list_row_simple(&site, String::new())),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: matched,
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
    // Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store
    let province = match resolve_site_province_via_shard(
        &state,
        &site_sfid,
        ctx.admin_province.as_deref(),
    )
    .await
    {
        Ok(v) => v,
        Err((code, msg)) => return api_error(code, 1004, msg),
    };
    let sfid_for_closure = site_sfid.clone();
    let scope_province = ctx.admin_province.clone();
    let actor_pubkey = ctx.admin_pubkey.clone();
    let target_status_clone = target_status.clone();
    let write_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let site = match shard.cpms_site_keys.get_mut(sfid_for_closure.as_str()) {
                Some(v) => v,
                None => return Err((StatusCode::NOT_FOUND, "cpms site not found")),
            };
            if !in_scope_cpms_site(site, scope_province.as_deref()) {
                return Err((
                    StatusCode::FORBIDDEN,
                    "cannot manage other province institutions",
                ));
            }
            if site.status == target_status_clone {
                // 状态相同，返回当前快照（不改）
                return Ok((site.clone(), false));
            }
            if !can_transition_cpms_site_status(&site.status, &target_status_clone) {
                return Err((StatusCode::CONFLICT, "invalid cpms site status transition"));
            }
            site.status = target_status_clone;
            site.version += 1;
            site.updated_by = Some(actor_pubkey.clone());
            site.updated_at = Some(Utc::now());
            Ok((site.clone(), true))
        })
        .await;
    let (output, changed) = match write_result {
        Ok(Ok(v)) => v,
        Ok(Err((code, msg))) => return api_error(code, 1004, msg),
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e),
    };

    // 双写过渡期:sharded_store + legacy store 同步写(仅状态实际变更时)
    if changed {
        match state.store.write() {
            Ok(mut store) => {
                if let Some(site) = store.cpms_site_keys.get_mut(&site_sfid) {
                    site.status = target_status.clone();
                    site.version += 1;
                    site.updated_by = Some(ctx.admin_pubkey.clone());
                    site.updated_at = Some(Utc::now());
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (cpms status update, shard already committed)");
            }
        }
    }

    // 拿 legacy store 短锁做 admin 名称解析 + 审计日志
    let created_by_name = match state.store.read() {
        Ok(store) => resolve_admin_display_name(&store, &output.created_by),
        Err(_) => output.created_by.clone(),
    };
    let response_row = cpms_site_keys_to_list_row_simple(&output, created_by_name);
    if changed {
        append_cpms_audit_log_best_effort(
            &state,
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
    }
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
            AdminRole::ShengAdmin => "机构管理员",
            AdminRole::ShiAdmin => "系统管理员",
        };
        if let Some(province) = store.sheng_admin_province_by_pubkey.get(pubkey) {
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
#[allow(dead_code)]
pub(crate) struct ChainInstitutionRegisterReceipt {
    pub(crate) genesis_hash: String,
    pub(crate) sfid_id: String,
    pub(crate) register_nonce: String,
    pub(crate) signature: String,
    pub(crate) tx_hash: String,
    pub(crate) block_number: u64,
    /// 链上派生的多签地址(hex, 不含 0x)。注册成功后从 SfidRegisteredAddress 读取。
    pub(crate) duoqian_address: Option<String>,
}

// 中文注释:`validate_sfid_id_format` 和 SFID_ID_* 常量已搬到
// `crate::sfid::validator`,本文件通过 import 使用。见任务卡 1。


pub(crate) async fn submit_register_sfid_institution_extrinsic(
    state: &AppState,
    ctx: &AdminAuthContext,
    site_sfid: &str,
    institution_name: &str,
) -> Result<ChainInstitutionRegisterReceipt, String> {
    let sfid_id = validate_sfid_id_format(site_sfid)
        .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let register_nonce = Uuid::new_v4().to_string();
    // 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 11：
    // 业务 extrinsic 统一由本省 sr25519 Pair 签名并提交，签字段包和 submit 共用
    // 同一把 pair（否则链端 verifier 校验 origin != payload signer 会失败）。
    let (province_pair, province) = resolve_business_signer(state, ctx).map_err(|(_, msg)| {
        format!("register_sfid_institution submit failed: {msg}")
    })?;
    let credential = build_institution_credential_with_province(
        state,
        sfid_id.as_str(),
        institution_name,
        register_nonce,
        province.as_str(),
        &province_pair,
    )
    .map_err(|e| format!("register_sfid_institution submit failed: {e}"))?;
    let ws_url = crate::chain::url::chain_ws_url()
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

    // 用本省 Pair 的公钥作为 signer account（而非 SFID MAIN）。
    let signer_account = AccountId32(province_pair.public().0);
    let chain_nonce = legacy_rpc
        .system_account_next_index(&signer_account)
        .await
        .map_err(|e| {
            format!("register_sfid_institution submit failed: fetch account nonce failed: {e}")
        })?;
    // 任务卡 Phase 1.B 步骤 11：extrinsic 参数末尾追加 `signing_province: Some(province)`。
    let signing_province_val = Value::unnamed_variant(
        "Some",
        vec![Value::from_bytes(province.as_bytes().to_vec())],
    );
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
            signing_province_val,
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
    // 用同一把省 Pair 签 extrinsic signer payload（与字段包签名保持一致）。
    let signature = province_pair.sign(&partial_tx.signer_payload()).0;
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

    let block_hash = in_block.block_hash();
    let block = client
        .blocks()
        .at(block_hash)
        .await
        .map_err(|e| {
            format!("register_sfid_institution included failed: fetch block failed: {e}")
        })?;
    let block_number = block.number().to_string().parse::<u64>().map_err(|e| {
        format!("register_sfid_institution included failed: parse block number failed: {e}")
    })?;

    // 从链上 SfidRegisteredAddress(sfid_id, name) 读取派生的多签地址
    let duoqian_address = {
        let sfid_key = subxt::dynamic::Value::from_bytes(sfid_id.as_bytes());
        let name_key = subxt::dynamic::Value::from_bytes(credential.name.as_bytes());
        let query = subxt::dynamic::storage("DuoqianManagePow", "SfidRegisteredAddress", vec![sfid_key, name_key]);
        match client.storage().at(block_hash).fetch(&query).await {
            Ok(Some(val)) => {
                // AccountId = 32 bytes,编码后取 inner bytes
                let bytes = val.encoded();
                // SCALE 编码的 AccountId 直接就是 32 bytes
                if bytes.len() >= 32 {
                    Some(hex::encode(&bytes[bytes.len() - 32..]))
                } else {
                    tracing::warn!(sfid_id = %sfid_id, "SfidRegisteredAddress returned unexpected length: {}", bytes.len());
                    None
                }
            }
            Ok(None) => {
                tracing::warn!(sfid_id = %sfid_id, name = %credential.name, "SfidRegisteredAddress not found after registration");
                None
            }
            Err(e) => {
                tracing::warn!(sfid_id = %sfid_id, error = %e, "failed to query SfidRegisteredAddress");
                None
            }
        }
    };

    Ok(ChainInstitutionRegisterReceipt {
        genesis_hash: credential.genesis_hash,
        sfid_id,
        register_nonce: credential.register_nonce,
        signature: credential.signature,
        tx_hash,
        block_number,
        duoqian_address,
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

#[allow(dead_code)]
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
