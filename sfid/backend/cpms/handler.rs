use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};

// 中文注释:SFID 工具统一从 crate::sfid 拿,见 feedback_sfid_module_is_single_entry.md
use crate::sfid::validate_sfid_number_format;
use crate::*;

type Blake2b256 = Blake2b<U32>;

const MAX_CITY_CHARS: usize = 100;
const MAX_INSTITUTION_CHARS: usize = 100;
const MAX_PROVINCE_CHARS: usize = 100;
const MAX_STATUS_REASON_CHARS: usize = 500;
const GEO_SEAL_PREFIX: &str = "g1";

// 中文注释:Phase 2 Day 3 Round 2 迁移 cpms_site_keys 到 sharded_store。
// 根据 site_sfid 定位所在省份分片:
//   - sheng-admin 已经锁定 province scope,直接用 scope 省即可;
//   - 无省域 scope 的内部调用才跨省扫描定位,开销可接受(admin 低频操作)。
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

/// 生成 `SFID_CPMS_V1 / INSTALL` 安装授权二维码。
///
/// 中文注释:两码方案下 INSTALL 只携带 `sfid_number / province_name / city_name / install_secret / sig`。
/// 省市代码由 SFID 从 sfid_number 解码,不作为二维码字段重复携带。
pub(crate) async fn generate_cpms_install_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GenerateCpmsInstallInput>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
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
    // ── 查找已有机构,直接读机构当前 sfid_number；二维码不重复携带省市码。 ──
    let site_sfid = match state.store.read() {
        Ok(store) => {
            let found = store.multisig_institutions.values().find(|i| {
                i.province == province && i.city == city && i.institution_code == institution
            });
            match found {
                Some(inst) => inst.sfid_number.clone(),
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
    let site_sfid = match validate_sfid_number_format(site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
    };
    let province_code = extract_province_code_from_sfid(&site_sfid);
    let city_code = extract_city_code_from_sfid(&site_sfid);

    let install_secret = match generate_install_secret() {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "generate install_secret failed",
            )
        }
    };
    let install_secret_hash = install_secret_hash(install_secret.as_str());

    // 用 SFID 主密钥签名 INSTALL 精简字段。
    let sign_source = build_install_sign_source(&site_sfid, &province, &city, &install_secret_hash);
    let signature = match sign_with_main_key(&state, &sign_source) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "sign QR1 failed"),
    };

    // 构造 QR1 payload
    let qr1 = serde_json::json!({
        "proto": "SFID_CPMS_V1",
        "type": "INSTALL",
        "sfid_number": site_sfid,
        "province_name": province,
        "city_name": city,
        "install_secret": install_secret,
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
    // 写 cpms_site_keys 到 sharded_store
    let site_sfid_key = site_sfid.clone();
    let new_site = CpmsSiteKeys {
        site_sfid: site_sfid.clone(),
        install_token: String::new(),
        install_secret: install_secret.clone(),
        install_secret_hash: install_secret_hash.clone(),
        install_token_status: InstallTokenStatus::Pending,
        status: CpmsSiteStatus::Pending,
        version: 1,
        province_code: province_code.clone(),
        admin_province: province.clone(),
        city_name: city.clone(),
        city_code: city_code.clone(),
        institution_code: institution.clone(),
        institution_name: String::new(),
        qr1_payload: qr1_payload.clone(),
        cpms_pubkey_hash: None,
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
                store
                    .cpms_site_keys
                    .insert(site_sfid.clone(), new_site_for_legacy);
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
            "sfid_number={} province={} city={} city_code={} institution={}",
            site_sfid, province, city, city_code, institution,
        ),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: GenerateCpmsInstallOutput {
            sfid_number: site_sfid,
            qr1_payload,
        },
    })
    .into_response()
}

/// 处理 QR4 档案业务二维码，验证并录入档案。
///
/// SFID_CPMS_V1 两码方案：解 `geo_seal` 得到省市归属,再验证 CPMS 本机签名,
/// 去重后录入 imported_archives。
pub(crate) async fn archive_import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsArchiveImportInput>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // 解析 QR4
    let qr4: CpmsArchiveQrPayload = match serde_json::from_str(input.qr_payload.trim()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid QR4 payload"),
    };
    if qr4.r#type != "ARCHIVE" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "type must be ARCHIVE");
    }

    let verified = match verify_cpms_archive_qr(&state, &qr4, ctx.admin_province.as_deref()).await {
        Ok(v) => v,
        Err((status, code, msg)) => return api_error(status, code, msg.as_str()),
    };

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store
        .imported_archives
        .contains_key(verified.archive_no.as_str())
    {
        return api_error(StatusCode::CONFLICT, 1007, "archive_no already imported");
    }
    store.imported_archives.insert(
        verified.archive_no.clone(),
        ImportedArchive {
            archive_no: verified.archive_no.clone(),
            province_code: verified.province_code.clone(),
            city_code: verified.city_code.clone(),
            sfid_number: verified.sfid_number.clone(),
            cpms_pubkey_hash: verified.cpms_pubkey_hash.clone(),
            geo_seal_hash: verified.geo_seal_hash.clone(),
            imported_at: Utc::now(),
            status: ArchiveImportStatus::Active,
        },
    );
    append_audit_log(
        &mut store,
        "CPMS_ARCHIVE_IMPORT",
        &ctx.admin_pubkey,
        Some(verified.archive_no.clone()),
        None,
        "SUCCESS",
        format!(
            "archive_no={} province_code={} city_code={} sfid_number={} citizen_status={} voting_eligible={}",
            verified.archive_no,
            verified.province_code,
            verified.city_code,
            verified.sfid_number,
            qr4.cs,
            qr4.ve
        ),
    );
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CpmsArchiveImportOutput {
            archive_no: verified.archive_no,
            province_code: verified.province_code,
            city_code: verified.city_code,
            sfid_number: verified.sfid_number,
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
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_trimmed = site_sfid.trim().to_string();
    let province =
        match resolve_site_province_via_shard(&state, &sfid_trimmed, ctx.admin_province.as_deref())
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
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let site_sfid_validated = match validate_sfid_number_format(site_sfid.trim()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
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
                (
                    s.admin_province.clone(),
                    s.city_name.clone(),
                    s.city_code.clone(),
                    s.institution_name.clone(),
                    s.province_code.clone(),
                )
            })
        })
        .await
        .ok()
        .flatten();
    let (prov_name, city_name, _city_code, _inst_name, _province_code) =
        site_meta.unwrap_or_else(|| {
            (
                province.clone(),
                String::new(),
                extract_city_code_from_sfid(&site_sfid_validated),
                String::new(),
                extract_province_code_from_sfid(&site_sfid_validated),
            )
        });
    let install_secret = match generate_install_secret() {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "generate install_secret failed",
            )
        }
    };
    let install_secret_hash = install_secret_hash(install_secret.as_str());
    let sign_source = build_install_sign_source(
        &site_sfid_validated,
        &prov_name,
        &city_name,
        &install_secret_hash,
    );
    let signature = match sign_with_main_key(&state, &sign_source) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "sign QR1 failed"),
    };

    let qr1 = serde_json::json!({
        "proto": "SFID_CPMS_V1",
        "type": "INSTALL",
        "sfid_number": site_sfid_validated,
        "province_name": prov_name,
        "city_name": city_name,
        "install_secret": install_secret,
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
    let actor_pubkey = ctx.admin_pubkey.clone();
    let sfid_for_closure = site_sfid_validated.clone();
    let install_secret_clone = install_secret.clone();
    let install_secret_hash_clone = install_secret_hash.clone();
    let qr1_payload_clone = qr1_payload.clone();
    let write_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let site = match shard.cpms_site_keys.get_mut(sfid_for_closure.as_str()) {
                Some(v) => v,
                None => return Err("site_sfid not found"),
            };
            if site.status != CpmsSiteStatus::Revoked
                && site.install_token_status != InstallTokenStatus::Revoked
            {
                return Err("only revoked cpms install authorization can be reissued");
            }
            site.install_token = String::new();
            site.install_secret = install_secret_clone;
            site.install_secret_hash = install_secret_hash_clone;
            site.install_token_status = InstallTokenStatus::Pending;
            site.status = CpmsSiteStatus::Pending;
            site.qr1_payload = qr1_payload_clone;
            site.cpms_pubkey_hash = None;
            site.version += 1;
            site.updated_by = Some(actor_pubkey.clone());
            site.updated_at = Some(Utc::now());
            Ok(())
        })
        .await;
    match write_result {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => {
            let code = if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::CONFLICT
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
                    site.install_token = String::new();
                    site.install_secret = install_secret.clone();
                    site.install_secret_hash = install_secret_hash.clone();
                    site.install_token_status = InstallTokenStatus::Pending;
                    site.status = CpmsSiteStatus::Pending;
                    site.qr1_payload = qr1_payload.clone();
                    site.cpms_pubkey_hash = None;
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
        format!("sfid_number={}", site_sfid_validated),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: GenerateCpmsInstallOutput {
            sfid_number: site_sfid_validated,
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
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if site_sfid.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "site_sfid is required");
    }
    let site_sfid = match validate_sfid_number_format(site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    // Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store
    let province =
        match resolve_site_province_via_shard(&state, &site_sfid, ctx.admin_province.as_deref())
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
            let info = (
                existing.site_sfid.clone(),
                existing.status.clone(),
                existing.version,
            );
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
    let ctx = match require_sheng_admin(&state, &headers) {
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
        // 中文注释:无省域 scope 的历史兜底路径;正常 SHENG/SHI 登录应带省域。
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
    rows.sort_by(|a, b| a.sfid_number.cmp(&b.sfid_number));
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
/// 按市公安局机构 sfid_number 反查其 CPMS 站点(`cpms_site_keys`)。
///
/// 中文注释:对外一律使用公安局机构 `sfid_number`。`cpms_site_keys` 的内部
/// 历史字段保存同一个值,这里仍用 `(admin_province, city_name, institution_code)`
/// 做一次机构详情页反查,确保每市公安局只挂自己的 CPMS 授权。
pub(crate) async fn get_cpms_site_by_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_number = sfid_number.trim().to_string();
    if sfid_number.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "sfid_number is required");
    }

    // Phase 2 Day 3 Round 2:机构也从 sharded_store 读
    let province_code = extract_province_code_from_sfid(&sfid_number);
    let province_name = match crate::sfid::province::province_name_by_code(&province_code) {
        Some(n) => n.to_string(),
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_number",
            )
        }
    };
    let sfid_number_r = sfid_number.clone();
    let inst_result = state
        .sharded_store
        .read_province(&province_name, move |shard| {
            shard.multisig_institutions.get(&sfid_number_r).cloned()
        })
        .await;
    let inst = match inst_result {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
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
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if site_sfid.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "site_sfid is required");
    }
    let site_sfid = match validate_sfid_number_format(site_sfid.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let reason_text = reason.unwrap_or_default().trim().to_string();
    if reason_text.chars().count() > MAX_STATUS_REASON_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "reason too long");
    }
    // Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store
    let province =
        match resolve_site_province_via_shard(&state, &site_sfid, ctx.admin_province.as_deref())
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
        sfid_number: site.site_sfid.clone(),
        install_token_status: site.install_token_status.clone(),
        status: site.status.clone(),
        version: site.version,
        province_code: site.province_code.clone(),
        admin_province: site.admin_province.clone(),
        city_name: site.city_name.clone(),
        city_code: site.city_code.clone(),
        institution_code: site.institution_code.clone(),
        institution_name: site.institution_name.clone(),
        qr1_payload: site.qr1_payload.clone(),
        cpms_pubkey_bound: site.cpms_pubkey_hash.is_some(),
        created_by: site.created_by.clone(),
        created_by_name,
        created_at: site.created_at,
        updated_by: site.updated_by.clone(),
        updated_at: site.updated_at,
    }
}

fn cpms_site_keys_to_list_row_simple(
    site: &CpmsSiteKeys,
    created_by_name: String,
) -> CpmsSiteKeysListRow {
    CpmsSiteKeysListRow {
        sfid_number: site.site_sfid.clone(),
        install_token_status: site.install_token_status.clone(),
        status: site.status.clone(),
        version: site.version,
        province_code: site.province_code.clone(),
        admin_province: site.admin_province.clone(),
        city_name: site.city_name.clone(),
        city_code: site.city_code.clone(),
        institution_code: site.institution_code.clone(),
        institution_name: site.institution_name.clone(),
        qr1_payload: site.qr1_payload.clone(),
        cpms_pubkey_bound: site.cpms_pubkey_hash.is_some(),
        created_by: site.created_by.clone(),
        created_by_name,
        created_at: site.created_at,
        updated_by: site.updated_by.clone(),
        updated_at: site.updated_at,
    }
}

/// 验证 `SFID_CPMS_V1 / ARCHIVE` 档案二维码。
///
/// 中文注释:本函数是两码方案的 SFID 侧核心验收点。它不会信任二维码明文省市,
/// 只用安装授权中保存的 `install_secret` 尝试解开 `geo_seal`,再校验 CPMS 本机签名。
pub(crate) async fn verify_cpms_archive_qr(
    state: &AppState,
    qr4: &CpmsArchiveQrPayload,
    admin_province_scope: Option<&str>,
) -> Result<VerifiedCpmsArchive, (StatusCode, u32, String)> {
    if qr4.proto != "SFID_CPMS_V1" {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "proto must be SFID_CPMS_V1".to_string(),
        ));
    }
    if qr4.r#type != "ARCHIVE" {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "type must be ARCHIVE".to_string(),
        ));
    }
    if qr4.ano.trim().is_empty()
        || qr4.cs.trim().is_empty()
        || qr4.cpms_pubkey.trim().is_empty()
        || qr4.geo_seal.trim().is_empty()
        || qr4.sig.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "archive QR fields are required".to_string(),
        ));
    }
    let (province_name, site, seal) = find_site_by_geo_seal(
        state,
        qr4.geo_seal.as_str(),
        qr4.ano.as_str(),
        qr4.cpms_pubkey.as_str(),
        admin_province_scope,
    )
    .await?;

    validate_geo_seal_against_site(&site, &seal)?;

    if matches!(
        site.status,
        CpmsSiteStatus::Disabled | CpmsSiteStatus::Revoked
    ) {
        return Err((
            StatusCode::FORBIDDEN,
            1003,
            "CPMS install authorization is not active".to_string(),
        ));
    }
    if site.install_token_status == InstallTokenStatus::Revoked {
        return Err((
            StatusCode::FORBIDDEN,
            1003,
            "CPMS install authorization is revoked".to_string(),
        ));
    }

    let cpms_pubkey =
        crate::login::parse_sr25519_pubkey_bytes(qr4.cpms_pubkey.as_str()).ok_or((
            StatusCode::BAD_REQUEST,
            1001,
            "cpms_pubkey format invalid".to_string(),
        ))?;
    let archive_sig = hex::decode(qr4.sig.trim().trim_start_matches("0x")).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            1001,
            "archive sig hex decode failed".to_string(),
        )
    })?;
    let geo_seal_hash = hash_hex(qr4.geo_seal.as_bytes());
    let archive_sign_source = build_archive_sign_source(
        qr4.ano.as_str(),
        qr4.cs.as_str(),
        qr4.ve,
        qr4.cpms_pubkey.as_str(),
        geo_seal_hash.as_str(),
    );
    if !verify_sr25519_signature(&cpms_pubkey, &archive_sign_source, &archive_sig) {
        return Err((
            StatusCode::UNAUTHORIZED,
            2004,
            "archive signature invalid".to_string(),
        ));
    }
    let cpms_pubkey_hash = hash_hex(qr4.cpms_pubkey.as_bytes());
    bind_cpms_pubkey_if_needed(
        state,
        &province_name,
        site.site_sfid.as_str(),
        cpms_pubkey_hash.as_str(),
    )
    .await?;

    Ok(VerifiedCpmsArchive {
        archive_no: qr4.ano.clone(),
        province_code: extract_province_code_from_sfid(seal.sfid_number.as_str()),
        city_code: extract_city_code_from_sfid(seal.sfid_number.as_str()),
        sfid_number: seal.sfid_number,
        cpms_pubkey_hash,
        geo_seal_hash,
    })
}

async fn find_site_by_geo_seal(
    state: &AppState,
    geo_seal: &str,
    archive_no: &str,
    cpms_pubkey: &str,
    admin_province_scope: Option<&str>,
) -> Result<(String, CpmsSiteKeys, CpmsGeoSealClaims), (StatusCode, u32, String)> {
    if let Some(province) = admin_province_scope {
        let found = state
            .sharded_store
            .read_province(province, |shard| {
                find_site_in_shard_by_geo_seal(geo_seal, archive_no, cpms_pubkey, shard)
            })
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "shard read failed".to_string(),
                )
            })?;
        return found
            .map(|(site, seal)| (province.to_string(), site, seal))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                2004,
                "geo_seal cannot be decrypted".to_string(),
            ));
    }

    let mut found: Option<(String, CpmsSiteKeys, CpmsGeoSealClaims)> = None;
    state
        .sharded_store
        .for_each_province(|province, shard| {
            if found.is_some() {
                return;
            }
            if let Some((site, seal)) =
                find_site_in_shard_by_geo_seal(geo_seal, archive_no, cpms_pubkey, shard)
            {
                found = Some((province.to_string(), site, seal));
            }
        })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "shard scan failed".to_string(),
            )
        })?;
    found.ok_or((
        StatusCode::UNAUTHORIZED,
        2004,
        "geo_seal cannot be decrypted".to_string(),
    ))
}

fn find_site_in_shard_by_geo_seal(
    geo_seal: &str,
    archive_no: &str,
    cpms_pubkey: &str,
    shard: &crate::store_shards::StoreShard,
) -> Option<(CpmsSiteKeys, CpmsGeoSealClaims)> {
    for site in shard.cpms_site_keys.values() {
        if site.install_secret.trim().is_empty() {
            continue;
        }
        if let Ok(seal) = decrypt_geo_seal(
            site.install_secret.as_str(),
            geo_seal,
            archive_no,
            cpms_pubkey,
        ) {
            return Some((site.clone(), seal));
        }
    }
    None
}

fn validate_geo_seal_against_site(
    site: &CpmsSiteKeys,
    seal: &CpmsGeoSealClaims,
) -> Result<(), (StatusCode, u32, String)> {
    if seal.sfid_number != site.site_sfid {
        return Err((
            StatusCode::UNAUTHORIZED,
            2004,
            "geo_seal install scope mismatch".to_string(),
        ));
    }
    Ok(())
}

async fn bind_cpms_pubkey_if_needed(
    state: &AppState,
    province: &str,
    sfid_number: &str,
    cpms_pubkey_hash: &str,
) -> Result<(), (StatusCode, u32, String)> {
    let sfid_key = sfid_number.to_string();
    let hash = cpms_pubkey_hash.to_string();
    let result = state
        .sharded_store
        .write_province(province, move |shard| {
            let Some(site) = shard.cpms_site_keys.get_mut(sfid_key.as_str()) else {
                return Err((
                    StatusCode::NOT_FOUND,
                    1004,
                    "cpms install authorization not found",
                ));
            };
            if let Some(existing) = site.cpms_pubkey_hash.as_deref() {
                if existing != hash {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        2004,
                        "cpms_pubkey does not match installed CPMS",
                    ));
                }
            } else {
                site.cpms_pubkey_hash = Some(hash.clone());
            }
            if site.status == CpmsSiteStatus::Pending {
                site.status = CpmsSiteStatus::Active;
                site.install_token_status = InstallTokenStatus::Used;
            }
            site.version += 1;
            site.updated_at = Some(Utc::now());
            Ok(())
        })
        .await;
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err((status, code, msg))) => Err((status, code, msg.to_string())),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "shard write failed".to_string(),
        )),
    }
}

fn decrypt_geo_seal(
    install_secret: &str,
    geo_seal: &str,
    archive_no: &str,
    cpms_pubkey: &str,
) -> Result<CpmsGeoSealClaims, String> {
    let parts: Vec<&str> = geo_seal.split('.').collect();
    if parts.len() != 3 || parts[0] != GEO_SEAL_PREFIX {
        return Err("geo_seal format invalid".to_string());
    }
    let nonce_bytes =
        hex::decode(parts[1]).map_err(|_| "geo_seal nonce hex invalid".to_string())?;
    if nonce_bytes.len() != 12 {
        return Err("geo_seal nonce length invalid".to_string());
    }
    let cipher_bytes =
        hex::decode(parts[2]).map_err(|_| "geo_seal cipher hex invalid".to_string())?;
    let key = derive_geo_seal_key(install_secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "geo_seal key invalid".to_string())?;
    let plain = cipher
        .decrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: cipher_bytes.as_ref(),
                aad: geo_seal_aad(archive_no, cpms_pubkey).as_bytes(),
            },
        )
        .map_err(|_| "geo_seal decrypt failed".to_string())?;
    serde_json::from_slice(&plain).map_err(|_| "geo_seal json invalid".to_string())
}

fn derive_geo_seal_key(install_secret: &str) -> [u8; 32] {
    let digest = Blake2b256::digest(install_secret.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest[..32]);
    key
}

fn generate_install_secret() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|e| e.to_string())?;
    Ok(format!("0x{}", hex::encode(bytes)))
}

fn install_secret_hash(install_secret: &str) -> String {
    hash_hex(install_secret.as_bytes())
}

fn hash_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(Blake2b256::digest(bytes)))
}

fn geo_seal_aad(archive_no: &str, cpms_pubkey: &str) -> String {
    format!("sfid-cpms-v1|geo-seal|{}|{}", archive_no, cpms_pubkey)
}

fn build_install_sign_source(
    sfid_number: &str,
    province_name: &str,
    city_name: &str,
    install_secret_hash: &str,
) -> String {
    format!(
        "sfid-cpms-v1|install|{}|{}|{}|{}",
        sfid_number, province_name, city_name, install_secret_hash
    )
}

fn build_archive_sign_source(
    archive_no: &str,
    citizen_status: &str,
    voting_eligible: bool,
    cpms_pubkey: &str,
    geo_seal_hash: &str,
) -> String {
    format!(
        "sfid-cpms-v1|archive|{}|{}|{}|{}|{}",
        archive_no, citizen_status, voting_eligible, cpms_pubkey, geo_seal_hash
    )
}

fn resolve_admin_display_name(store: &Store, pubkey: &str) -> String {
    if let Some(admin) = store.admin_users_by_pubkey.get(pubkey) {
        // 中文注释:当前只剩 ShengAdmin / ShiAdmin 两角色。
        let role_label = match admin.role {
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

// 中文注释:`validate_sfid_number_format` 和 SFID_NUMBER_* 常量已搬到
// `crate::sfid::validator`,本文件通过 import 使用。见任务卡 1。

fn can_transition_cpms_site_status(current: &CpmsSiteStatus, target: &CpmsSiteStatus) -> bool {
    matches!(
        (current, target),
        (CpmsSiteStatus::Active, CpmsSiteStatus::Disabled)
            | (CpmsSiteStatus::Active, CpmsSiteStatus::Revoked)
            | (CpmsSiteStatus::Disabled, CpmsSiteStatus::Active)
            | (CpmsSiteStatus::Disabled, CpmsSiteStatus::Revoked)
    )
}

// ADR-008 Phase 23e:`is_trusted_attestor_pubkey` 历史依赖
// `validate_active_main_signer_with_keyring` + AppState.public_key_hex,这两条
// 路径都已下架。该函数当前 dead code(`#[allow(dead_code)]`),保留接口签名供
// 将来 chain pull 凭证回流校验使用,内部退化成"用 SFID main env seed 派生公钥
// 做对比"的 fallback 行为。
#[allow(dead_code)]
fn is_trusted_attestor_pubkey(_state: &AppState, public_key: &str) -> bool {
    let Some(candidate) = parse_sr25519_pubkey(public_key) else {
        return false;
    };
    let Ok(seed_hex) = std::env::var("SFID_SIGNING_SEED_HEX") else {
        return false;
    };
    let Ok(derived) = crate::crypto::sr25519::try_derive_pubkey_hex_from_seed(seed_hex.as_str())
    else {
        return false;
    };
    parse_sr25519_pubkey(derived.as_str())
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

/// 从 site_sfid 的 r5 段提取三位城市代码。
pub(super) fn extract_city_code_from_sfid(site_sfid: &str) -> String {
    let segments: Vec<&str> = site_sfid.split('-').collect();
    if segments.len() >= 2 && segments[1].len() >= 5 {
        segments[1][2..5].to_string()
    } else {
        String::new()
    }
}

/// 用 SFID 主密钥（sr25519）对消息签名，返回 hex 编码签名。
///
/// ADR-008 Phase 23e:AppState 不再持有 signing seed,改从 SFID_SIGNING_SEED_HEX
/// 环境变量按需加载。本函数只在 `generate_cpms_install_qr` 路径上签发
/// 二维码完整性签名,与省管理员 3-tier signing pubkey 无关。
fn sign_with_main_key(_state: &AppState, message: &str) -> Result<String, String> {
    use sp_core::Pair;
    let seed_hex = std::env::var("SFID_SIGNING_SEED_HEX")
        .map_err(|_| "SFID_SIGNING_SEED_HEX not set".to_string())?;
    let keypair = crate::crypto::sr25519::try_load_signing_key_from_seed(seed_hex.as_str())
        .map_err(|e| format!("load signing key failed: {e}"))?;
    let sig = keypair.sign(message.as_bytes());
    Ok(format!("0x{}", hex::encode(sig.0)))
}

/// 验证 sr25519 签名。
pub(crate) fn verify_sr25519_signature(
    pubkey_bytes: &[u8; 32],
    message: &str,
    signature: &[u8],
) -> bool {
    use schnorrkel::{
        signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature,
    };
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
