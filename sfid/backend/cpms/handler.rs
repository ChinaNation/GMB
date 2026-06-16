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
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use chrono::{NaiveDate, Utc};

use crate::admins::actions::require_admin_security_grant;
use crate::admins::operation_auth::AdminActionType;
use crate::cpms::CpmsRegionClaims;
use crate::number::validate_sfid_number_format;
use crate::*;

type Blake2b256 = Blake2b<U32>;

const MAX_STATUS_REASON_CHARS: usize = 500;
const GEO_SEAL_PREFIX: &str = "g1";
const ELECTION_SCOPE_PROVINCE: &str = "PROVINCE";
const ELECTION_SCOPE_CITY: &str = "CITY";
const ELECTION_SCOPE_TOWN: &str = "TOWN";

struct NormalizedRegionClaims {
    province_code: String,
    city_code: Option<String>,
    town_code: Option<String>,
}

fn cpms_status_text(status: &CpmsSiteStatus) -> &'static str {
    match status {
        CpmsSiteStatus::Pending => "PENDING",
        CpmsSiteStatus::Active => "ACTIVE",
        CpmsSiteStatus::Disabled => "DISABLED",
        CpmsSiteStatus::Revoked => "REVOKED",
    }
}

fn install_token_status_text(status: &InstallTokenStatus) -> &'static str {
    match status {
        InstallTokenStatus::Pending => "PENDING",
        InstallTokenStatus::Used => "USED",
        InstallTokenStatus::Revoked => "REVOKED",
    }
}

fn cpms_site_from_row(row: &postgres::Row) -> Result<CpmsSiteKeys, String> {
    let payload: serde_json::Value = row.get(7);
    let mut site: CpmsSiteKeys = serde_json::from_value(payload)
        .map_err(|e| format!("deserialize cpms site payload failed: {e}"))?;
    site.sfid_number = row.get(0);
    site.province_code = row.get(1);
    site.city_code = row.get(2);
    site.cpms_pubkey_hash = row.get(5);
    site.created_by = row.get(6);
    Ok(site)
}

impl Db {
    pub(crate) fn get_cpms_site(&self, sfid_number: &str) -> Result<Option<CpmsSiteKeys>, String> {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT sfid_number, p_code, c_code, status, install_token_status,
                            cpms_pubkey_hash, created_by, payload
                     FROM cpms_sites
                     WHERE sfid_number = $1",
                    &[&sfid_number],
                )
                .map_err(|e| format!("query cpms site failed: {e}"))?;
            row.as_ref().map(cpms_site_from_row).transpose()
        })
    }

    fn upsert_cpms_site(&self, site: &CpmsSiteKeys) -> Result<(), String> {
        let site = site.clone();
        self.with_client(move |conn| {
            let payload = serde_json::to_value(&site)
                .map_err(|e| format!("serialize cpms site failed: {e}"))?;
            conn.execute(
                "INSERT INTO cpms_sites (
                    sfid_number, p_code, c_code, status, install_token_status,
                    cpms_pubkey_hash, created_by, created_at, updated_at, payload
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                 ON CONFLICT (sfid_number) DO UPDATE SET
                    p_code = EXCLUDED.p_code,
                    c_code = EXCLUDED.c_code,
                    status = EXCLUDED.status,
                    install_token_status = EXCLUDED.install_token_status,
                    cpms_pubkey_hash = EXCLUDED.cpms_pubkey_hash,
                    updated_at = EXCLUDED.updated_at,
                    payload = EXCLUDED.payload",
                &[
                    &site.sfid_number,
                    &site.province_code,
                    &site.city_code,
                    &cpms_status_text(&site.status),
                    &install_token_status_text(&site.install_token_status),
                    &site.cpms_pubkey_hash,
                    &site.created_by,
                    &site.created_at,
                    &site.updated_at,
                    &payload,
                ],
            )
            .map_err(|e| format!("upsert cpms site failed: {e}"))?;
            Ok(())
        })
    }

    fn delete_cpms_site(&self, sfid_number: &str) -> Result<bool, String> {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let affected = conn
                .execute(
                    "DELETE FROM cpms_sites WHERE sfid_number = $1",
                    &[&sfid_number],
                )
                .map_err(|e| format!("delete cpms site failed: {e}"))?;
            Ok(affected > 0)
        })
    }

    fn list_cpms_sites(&self, province_name: Option<&str>) -> Result<Vec<CpmsSiteKeys>, String> {
        let p_code = province_name
            .and_then(crate::china::province_code_by_name)
            .map(str::to_string);
        self.with_client(move |conn| {
            let rows = conn
                .query(
                    "SELECT sfid_number, p_code, c_code, status, install_token_status,
                            cpms_pubkey_hash, created_by, payload
                     FROM cpms_sites
                     WHERE ($1::text IS NULL OR p_code = $1)
                     ORDER BY p_code ASC, c_code ASC, sfid_number ASC",
                    &[&p_code],
                )
                .map_err(|e| format!("list cpms sites failed: {e}"))?;
            rows.iter().map(cpms_site_from_row).collect()
        })
    }

    /// 按机构自身 sfid_number 反查机构真源(subjects 主键含 sfid_number,全局唯一)。
    /// 返回 (province_name, city_name, province_code, city_code, institution_code, institution_name, category)。
    fn find_cpms_target_institution_by_sfid(
        &self,
        sfid_number: &str,
    ) -> Result<Option<(String, String, String, String, String, String, String)>, String> {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT COALESCE(province, ''), COALESCE(city, ''),
                            COALESCE(province_code, ''), COALESCE(city_code, ''),
                            COALESCE(institution_code, ''), COALESCE(name, ''),
                            COALESCE(category, '')
                     FROM subjects
                     WHERE kind = 'PUBLIC'
                       AND status = 'ACTIVE'
                       AND sfid_number = $1
                     LIMIT 1",
                    &[&sfid_number],
                )
                .map_err(|e| format!("query cpms target institution by sfid failed: {e}"))?;
            Ok(row.map(|row| {
                (
                    row.get(0),
                    row.get(1),
                    row.get(2),
                    row.get(3),
                    row.get(4),
                    row.get(5),
                    row.get(6),
                )
            }))
        })
    }
}

fn append_cpms_audit_log_best_effort(
    state: &AppState,
    action: &'static str,
    actor_pubkey: &str,
    target_sfid: Option<String>,
    detail: serde_json::Value,
) {
    crate::core::runtime_ops::append_audit_log(state, action, actor_pubkey, target_sfid, detail);
}

pub(crate) async fn generate_cpms_install_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GenerateCpmsInstallInput>,
) -> impl IntoResponse {
    let ctx = match require_federal_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let grant_payload = serde_json::json!({
        "province": input.province.clone(),
        "city": input.city.clone(),
        "institution": input.institution.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CpmsIssueInstallCode,
        "*",
        Some(&grant_payload),
    ) {
        return resp;
    }

    let sfid_number = match validate_sfid_number_format(input.sfid_number.trim()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    // 中文注释:安装码以机构自身 sfid_number 为唯一键(写/读同键);按 sfid 反查机构真源,
    // 不再用 (province,city,institution) 三元组重解析(institution_code 如 ZF 是类别码,
    // 同市可命中数十个机构,曾导致公安局页面生成的安装码错落到农业局名下)。
    let Some((
        province,
        city,
        province_code,
        city_code,
        institution_code,
        institution_name,
        category,
    )) = (match state.db.find_cpms_target_institution_by_sfid(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query cpms target institution by sfid failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    })
    else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    // 中文注释:铁律——只有公安局(category=PUBLIC_SECURITY)才能签发 CPMS 安装码。
    // 前端按钮只是展示层,服务端必须独立强制,否则任何直调 API 都能给非公安机构发码。
    if category != "PUBLIC_SECURITY" {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "install code is only available for PUBLIC_SECURITY institutions",
        );
    }
    // federal admin 只能给本省机构发码;federal(无 scope)放行。
    if let Some(scope) = ctx.admin_province.as_deref() {
        if province != scope {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "province out of current admin scope",
            );
        }
    }
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
    let sign_source =
        build_install_sign_source(&sfid_number, &province, &city, &install_secret_hash);
    let signature = match sign_with_main_key(&state, &sign_source) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "sign QR1 failed"),
    };
    let qr1 = serde_json::json!({
        "proto": "SFID_CPMS_V1",
        "type": "INSTALL",
        "sfid_number": sfid_number,
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
    let site = CpmsSiteKeys {
        sfid_number: sfid_number.clone(),
        install_token: String::new(),
        install_secret,
        install_secret_hash,
        install_token_status: InstallTokenStatus::Pending,
        status: CpmsSiteStatus::Pending,
        version: 1,
        province_code,
        admin_province: province,
        city_name: city,
        city_code,
        institution_code,
        institution_name,
        qr1_payload: qr1_payload.clone(),
        cpms_pubkey_hash: None,
        created_by: ctx.admin_pubkey.clone(),
        created_at,
        updated_by: None,
        updated_at: None,
    };
    if let Err(err) = state.db.upsert_cpms_site(&site) {
        tracing::error!(error = %err, "write cpms site failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "cpms site write failed",
        );
    }
    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_INSTALL_QR_GENERATE",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        serde_json::json!({
            "city": site.city_name.clone(),
            "institution": site.institution_code.clone(),
        }),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: GenerateCpmsInstallOutput {
            sfid_number: sfid_number,
            qr1_payload,
        },
    })
    .into_response()
}

pub(crate) async fn archive_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsArchiveVerifyInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let archive_code: CpmsArchiveCodePayload = match serde_json::from_str(input.qr_payload.trim()) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid archive QR payload"),
    };
    let verified =
        match verify_cpms_archive_qr(&state, &archive_code, ctx.admin_province.as_deref()).await {
            Ok(v) => v,
            Err((status, code, msg)) => return api_error(status, code, msg.as_str()),
        };
    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_ARCHIVE_VERIFY",
        &ctx.admin_pubkey,
        Some(verified.sfid_number.clone()),
        serde_json::json!({ "archive_no": verified.archive_no.clone() }),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CpmsArchiveVerifyOutput {
            archive_no: verified.archive_no,
            citizen_status: verified.citizen_status,
            voting_eligible: verified.voting_eligible,
            valid_from: verified.valid_from,
            valid_until: verified.valid_until,
            status_updated_at: verified.status_updated_at,
            province_code: verified.province_code,
            city_code: verified.city_code,
            residence_province_code: verified.residence_province_code,
            residence_city_code: verified.residence_city_code,
            residence_town_code: verified.residence_town_code,
            birth_province_code: verified.birth_province_code,
            birth_city_code: verified.birth_city_code,
            birth_town_code: verified.birth_town_code,
            election_scope_level: verified.election_scope_level,
            sfid_number: verified.sfid_number,
            status: "verified",
        },
    })
    .into_response()
}

pub(crate) async fn revoke_install_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    update_cpms_site_token_status(state, headers, sfid_number, InstallTokenStatus::Revoked).await
}

pub(crate) async fn reissue_install_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_federal_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_number = match validate_sfid_number_format(sfid_number.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let grant_payload = serde_json::json!({ "target": sfid_number.clone() });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CpmsIssueInstallCode,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let mut site = match load_scoped_site(&state, &ctx, &sfid_number) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if matches!(site.status, CpmsSiteStatus::Revoked) {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "revoked cpms site cannot be reissued",
        );
    }
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
    site.install_secret = install_secret;
    site.install_secret_hash = install_secret_hash(site.install_secret.as_str());
    site.install_token_status = InstallTokenStatus::Pending;
    site.cpms_pubkey_hash = None;
    site.status = CpmsSiteStatus::Pending;
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    let sign_source = build_install_sign_source(
        site.sfid_number.as_str(),
        site.admin_province.as_str(),
        site.city_name.as_str(),
        site.install_secret_hash.as_str(),
    );
    let signature = match sign_with_main_key(&state, &sign_source) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "sign QR1 failed"),
    };
    let qr1 = serde_json::json!({
        "proto": "SFID_CPMS_V1",
        "type": "INSTALL",
        "sfid_number": site.sfid_number,
        "province_name": site.admin_province,
        "city_name": site.city_name,
        "install_secret": site.install_secret,
        "sig": signature,
    });
    site.qr1_payload = match serde_json::to_string(&qr1) {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "serialize QR1 failed",
            )
        }
    };
    if let Err(err) = state.db.upsert_cpms_site(&site) {
        tracing::error!(error = %err, "reissue cpms site failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "cpms site write failed",
        );
    }
    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_INSTALL_QR_REISSUE",
        &ctx.admin_pubkey,
        Some(site.sfid_number.clone()),
        // 操作语义已由 action 表达,无额外事实字段
        serde_json::json!({}),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: cpms_site_keys_to_list_row_simple(&site, ctx.admin_name),
    })
    .into_response()
}

async fn update_cpms_site_token_status(
    state: AppState,
    headers: HeaderMap,
    sfid_number: String,
    target: InstallTokenStatus,
) -> axum::response::Response {
    let ctx = match require_federal_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_number = match validate_sfid_number_format(sfid_number.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let grant_payload = serde_json::json!({ "target": sfid_number.clone() });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CpmsRevokeInstallToken,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let mut site = match load_scoped_site(&state, &ctx, &sfid_number) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    site.install_token_status = target;
    site.version += 1;
    site.updated_by = Some(ctx.admin_pubkey.clone());
    site.updated_at = Some(Utc::now());
    if let Err(err) = state.db.upsert_cpms_site(&site) {
        tracing::error!(error = %err, "update cpms token status failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "cpms site write failed",
        );
    }
    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_INSTALL_TOKEN_REVOKE",
        &ctx.admin_pubkey,
        Some(site.sfid_number.clone()),
        serde_json::json!({ "status": site.install_token_status.clone() }),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: cpms_site_keys_to_list_row_simple(&site, ctx.admin_name),
    })
    .into_response()
}

pub(crate) async fn disable_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
    Json(input): Json<UpdateCpmsSiteStatusInput>,
) -> impl IntoResponse {
    update_cpms_site_status(
        state,
        headers,
        sfid_number,
        CpmsSiteStatus::Disabled,
        input.reason,
    )
    .await
}

pub(crate) async fn enable_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
    Json(input): Json<UpdateCpmsSiteStatusInput>,
) -> impl IntoResponse {
    update_cpms_site_status(
        state,
        headers,
        sfid_number,
        CpmsSiteStatus::Active,
        input.reason,
    )
    .await
}

pub(crate) async fn revoke_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
    Json(input): Json<UpdateCpmsSiteStatusInput>,
) -> impl IntoResponse {
    update_cpms_site_status(
        state,
        headers,
        sfid_number,
        CpmsSiteStatus::Revoked,
        input.reason,
    )
    .await
}

pub(crate) async fn delete_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_federal_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_number = match validate_sfid_number_format(sfid_number.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let grant_payload = serde_json::json!({ "target": sfid_number.clone() });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CpmsDeleteKeys,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let site = match load_scoped_site(&state, &ctx, &sfid_number) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if site.status != CpmsSiteStatus::Pending {
        return api_error(
            StatusCode::CONFLICT,
            1007,
            "only pending cpms site can be deleted",
        );
    }
    if let Err(err) = state.db.delete_cpms_site(&sfid_number) {
        tracing::error!(error = %err, "delete cpms site failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "cpms site delete failed",
        );
    }
    append_cpms_audit_log_best_effort(
        &state,
        "CPMS_KEYS_DELETE",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        // 只有待安装(PENDING)站点允许删除,记录删除时的站点状态
        serde_json::json!({ "status": "PENDING" }),
    );
    #[derive(serde::Serialize)]
    struct DeleteOutput {
        deleted: bool,
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: DeleteOutput { deleted: true },
    })
    .into_response()
}

pub(crate) async fn list_cpms_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_federal_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows = match state.db.list_cpms_sites(ctx.admin_province.as_deref()) {
        Ok(sites) => sites
            .iter()
            .map(|site| cpms_site_keys_to_list_row_simple(site, site.created_by.clone()))
            .collect::<Vec<_>>(),
        Err(err) => {
            tracing::error!(error = %err, "list cpms sites failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "cpms list failed");
        }
    };
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

pub(crate) async fn get_cpms_site_by_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_federal_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_number = match validate_sfid_number_format(sfid_number.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let site = match state.db.get_cpms_site(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query cpms site failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "cpms query failed");
        }
    };
    let row = match site {
        Some(site) => {
            if let Some(scope) = ctx.admin_province.as_deref() {
                if site.admin_province != scope {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        1003,
                        "cannot view other province institutions",
                    );
                }
            }
            Some(cpms_site_keys_to_list_row_simple(
                &site,
                site.created_by.clone(),
            ))
        }
        None => None,
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: row,
    })
    .into_response()
}

async fn update_cpms_site_status(
    state: AppState,
    headers: HeaderMap,
    sfid_number: String,
    target_status: CpmsSiteStatus,
    reason: Option<String>,
) -> axum::response::Response {
    let ctx = match require_federal_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sfid_number = match validate_sfid_number_format(sfid_number.as_str()) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };
    let reason_text = reason.unwrap_or_default().trim().to_string();
    if reason_text.chars().count() > MAX_STATUS_REASON_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "reason too long");
    }
    let action_type = match target_status {
        CpmsSiteStatus::Active => AdminActionType::CpmsEnableKeys,
        CpmsSiteStatus::Disabled => AdminActionType::CpmsDisableKeys,
        CpmsSiteStatus::Revoked => AdminActionType::CpmsRevokeKeys,
        CpmsSiteStatus::Pending => {
            return api_error(StatusCode::BAD_REQUEST, 1001, "invalid target status")
        }
    };
    let grant_payload = serde_json::json!({ "target": sfid_number.clone(), "reason": reason_text });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        action_type,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let mut site = match load_scoped_site(&state, &ctx, &sfid_number) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let changed = site.status != target_status;
    if changed && !can_transition_cpms_site_status(&site.status, &target_status) {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "invalid cpms site status transition",
        );
    }
    if changed {
        site.status = target_status;
        site.version += 1;
        site.updated_by = Some(ctx.admin_pubkey.clone());
        site.updated_at = Some(Utc::now());
        if let Err(err) = state.db.upsert_cpms_site(&site) {
            tracing::error!(error = %err, "update cpms site status failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "cpms site write failed",
            );
        }
        append_cpms_audit_log_best_effort(
            &state,
            "CPMS_KEYS_STATUS_UPDATE",
            &ctx.admin_pubkey,
            Some(site.sfid_number.clone()),
            serde_json::json!({
                "status": site.status.clone(),
                "reason": reason_text.clone(),
            }),
        );
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: cpms_site_keys_to_list_row_simple(&site, ctx.admin_name),
    })
    .into_response()
}

fn load_scoped_site(
    state: &AppState,
    ctx: &crate::admins::login::AdminAuthContext,
    sfid_number: &str,
) -> Result<CpmsSiteKeys, axum::response::Response> {
    let site = state
        .db
        .get_cpms_site(sfid_number)
        .map_err(|err| {
            tracing::error!(error = %err, "query cpms site failed");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "cpms query failed")
        })?
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, 1004, "cpms site not found"))?;
    if let Some(scope) = ctx.admin_province.as_deref() {
        if site.admin_province != scope {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                1003,
                "cannot manage other province institutions",
            ));
        }
    }
    Ok(site)
}

fn cpms_site_keys_to_list_row_simple(
    site: &CpmsSiteKeys,
    created_by_name: String,
) -> CpmsSiteKeysListRow {
    CpmsSiteKeysListRow {
        sfid_number: site.sfid_number.clone(),
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

pub(crate) async fn verify_cpms_archive_qr(
    state: &AppState,
    archive_code: &CpmsArchiveCodePayload,
    admin_province_scope: Option<&str>,
) -> Result<VerifiedCpmsArchive, (StatusCode, u32, String)> {
    if archive_code.proto != "SFID_CPMS_V1" {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "proto must be SFID_CPMS_V1".to_string(),
        ));
    }
    if archive_code.r#type != "ARCHIVE" {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "type must be ARCHIVE".to_string(),
        ));
    }
    if archive_code.archive_no.trim().is_empty()
        || archive_code.citizen_status.trim().is_empty()
        || archive_code.valid_from.trim().is_empty()
        || archive_code.valid_until.trim().is_empty()
        || archive_code.status_updated_at <= 0
        || archive_code.cpms_pubkey.trim().is_empty()
        || archive_code.geo_seal.trim().is_empty()
        || archive_code.wallet_address.trim().is_empty()
        || archive_code.wallet_pubkey.trim().is_empty()
        || archive_code.sig.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "archive code fields are required".to_string(),
        ));
    }
    validate_archive_validity(
        archive_code.valid_from.as_str(),
        archive_code.valid_until.as_str(),
    )?;
    let (site, seal) = find_site_by_geo_seal(
        state,
        archive_code.geo_seal.as_str(),
        archive_code.archive_no.as_str(),
        archive_code.cpms_pubkey.as_str(),
        admin_province_scope,
    )
    .await?;
    validate_geo_seal_against_site(&site, &seal)?;
    let election_scope_level = validate_election_scope_level(seal.election_scope_level.as_str())?;
    let residence_region =
        validate_region_claims(&seal.residence, "residence", election_scope_level.as_str())?;
    let birthplace_region = validate_region_claims(
        &seal.birthplace,
        "birthplace",
        election_scope_level.as_str(),
    )?;
    validate_residence_region_against_site(
        &site,
        &residence_region,
        election_scope_level.as_str(),
    )?;
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
    let cpms_pubkey = crate::admins::login::parse_sr25519_pubkey_bytes(
        archive_code.cpms_pubkey.as_str(),
    )
    .ok_or((
        StatusCode::BAD_REQUEST,
        1001,
        "cpms_pubkey format invalid".to_string(),
    ))?;
    let archive_sig =
        hex::decode(archive_code.sig.trim().trim_start_matches("0x")).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                1001,
                "archive sig hex decode failed".to_string(),
            )
        })?;
    let geo_seal_hash = hash_hex(archive_code.geo_seal.as_bytes());
    let archive_sign_source = build_archive_sign_source(
        archive_code.archive_no.as_str(),
        archive_code.citizen_status.as_str(),
        archive_code.voting_eligible,
        archive_code.valid_from.as_str(),
        archive_code.valid_until.as_str(),
        archive_code.status_updated_at,
        archive_code.cpms_pubkey.as_str(),
        geo_seal_hash.as_str(),
        archive_code.wallet_address.as_str(),
        archive_code.wallet_pubkey.as_str(),
    );
    if !verify_sr25519_signature(&cpms_pubkey, &archive_sign_source, &archive_sig) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "archive signature invalid".to_string(),
        ));
    }
    let cpms_pubkey_hash = hash_hex(archive_code.cpms_pubkey.as_bytes());
    let citizen_status = citizen_status_from_cpms(archive_code.citizen_status.as_str())?;
    bind_cpms_pubkey_if_needed(state, site.sfid_number.as_str(), cpms_pubkey_hash.as_str()).await?;
    Ok(VerifiedCpmsArchive {
        archive_no: archive_code.archive_no.clone(),
        citizen_status,
        voting_eligible: archive_code.voting_eligible,
        valid_from: archive_code.valid_from.clone(),
        valid_until: archive_code.valid_until.clone(),
        status_updated_at: archive_code.status_updated_at,
        province_code: extract_province_code_from_sfid(seal.sfid_number.as_str()),
        city_code: extract_city_code_from_sfid(seal.sfid_number.as_str()),
        residence_province_code: residence_region.province_code,
        residence_city_code: residence_region.city_code,
        residence_town_code: residence_region.town_code,
        birth_province_code: birthplace_region.province_code,
        birth_city_code: birthplace_region.city_code,
        birth_town_code: birthplace_region.town_code,
        election_scope_level,
        sfid_number: seal.sfid_number,
        wallet_address: archive_code.wallet_address.clone(),
        wallet_pubkey: archive_code.wallet_pubkey.clone(),
        wallet_sig_alg: archive_code.wallet_sig_alg.clone(),
    })
}

async fn find_site_by_geo_seal(
    state: &AppState,
    geo_seal: &str,
    archive_no: &str,
    cpms_pubkey: &str,
    admin_province_scope: Option<&str>,
) -> Result<(CpmsSiteKeys, CpmsGeoSealClaims), (StatusCode, u32, String)> {
    let sites = state
        .db
        .list_cpms_sites(admin_province_scope)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, 1004, err))?;
    for site in sites {
        if site.install_secret.trim().is_empty() {
            continue;
        }
        if let Ok(seal) = decrypt_geo_seal(
            site.install_secret.as_str(),
            geo_seal,
            archive_no,
            cpms_pubkey,
        ) {
            return Ok((site, seal));
        }
    }
    Err((
        StatusCode::UNPROCESSABLE_ENTITY,
        2004,
        "geo_seal cannot be decrypted".to_string(),
    ))
}

fn validate_geo_seal_against_site(
    site: &CpmsSiteKeys,
    seal: &CpmsGeoSealClaims,
) -> Result<(), (StatusCode, u32, String)> {
    if seal.sfid_number != site.sfid_number {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "geo_seal install scope mismatch".to_string(),
        ));
    }
    Ok(())
}

fn validate_election_scope_level(scope: &str) -> Result<String, (StatusCode, u32, String)> {
    let normalized = scope.trim().to_ascii_uppercase();
    match normalized.as_str() {
        ELECTION_SCOPE_PROVINCE | ELECTION_SCOPE_CITY | ELECTION_SCOPE_TOWN => Ok(normalized),
        _ => Err((
            StatusCode::BAD_REQUEST,
            1001,
            "geo_seal election_scope_level invalid".to_string(),
        )),
    }
}

fn validate_region_claims(
    region: &CpmsRegionClaims,
    label: &str,
    election_scope_level: &str,
) -> Result<NormalizedRegionClaims, (StatusCode, u32, String)> {
    let province_code = normalize_region_code(Some(region.province_code.as_str())).ok_or((
        StatusCode::BAD_REQUEST,
        1001,
        format!("geo_seal {label} province_code required"),
    ))?;
    let city_code = normalize_region_code(region.city_code.as_deref());
    let town_code = normalize_region_code(region.town_code.as_deref());

    match election_scope_level {
        ELECTION_SCOPE_PROVINCE => {
            if city_code.is_some() || town_code.is_some() {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    2004,
                    format!("geo_seal {label} must only contain province"),
                ));
            }
        }
        ELECTION_SCOPE_CITY => {
            if city_code.is_none() || town_code.is_some() {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    2004,
                    format!("geo_seal {label} must contain province and city"),
                ));
            }
        }
        ELECTION_SCOPE_TOWN => {
            if city_code.is_none() || town_code.is_none() {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    2004,
                    format!("geo_seal {label} must contain province, city and town"),
                ));
            }
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "geo_seal election_scope_level invalid".to_string(),
            ));
        }
    }

    Ok(NormalizedRegionClaims {
        province_code,
        city_code,
        town_code,
    })
}

fn validate_residence_region_against_site(
    site: &CpmsSiteKeys,
    region: &NormalizedRegionClaims,
    election_scope_level: &str,
) -> Result<(), (StatusCode, u32, String)> {
    if region.province_code != site.province_code {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "geo_seal residence province mismatch".to_string(),
        ));
    }
    if matches!(
        election_scope_level,
        ELECTION_SCOPE_CITY | ELECTION_SCOPE_TOWN
    ) && region.city_code.as_deref() != Some(site.city_code.as_str())
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "geo_seal residence city mismatch".to_string(),
        ));
    }
    Ok(())
}

fn normalize_region_code(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(trimmed.to_string())
}

pub(crate) async fn bind_cpms_pubkey_if_needed(
    state: &AppState,
    sfid_number: &str,
    cpms_pubkey_hash: &str,
) -> Result<(), (StatusCode, u32, String)> {
    let mut site = state
        .db
        .get_cpms_site(sfid_number)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, 1004, err))?
        .ok_or((
            StatusCode::NOT_FOUND,
            1004,
            "cpms install authorization not found".to_string(),
        ))?;
    if let Some(existing) = site.cpms_pubkey_hash.as_deref() {
        if existing != cpms_pubkey_hash {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "cpms_pubkey does not match installed CPMS".to_string(),
            ));
        }
    }
    let mut changed = false;
    if site.cpms_pubkey_hash.is_none() {
        site.cpms_pubkey_hash = Some(cpms_pubkey_hash.to_string());
        changed = true;
    }
    if site.status == CpmsSiteStatus::Pending {
        site.status = CpmsSiteStatus::Active;
        changed = true;
    }
    if site.install_token_status == InstallTokenStatus::Pending {
        site.install_token_status = InstallTokenStatus::Used;
        changed = true;
    }
    if changed {
        site.version += 1;
        site.updated_at = Some(Utc::now());
        state
            .db
            .upsert_cpms_site(&site)
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, 1004, err))?;
    }
    Ok(())
}

fn validate_archive_validity(
    valid_from: &str,
    valid_until: &str,
) -> Result<(), (StatusCode, u32, String)> {
    let from = NaiveDate::parse_from_str(valid_from.trim(), "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            1001,
            "valid_from format must be YYYY-MM-DD".to_string(),
        )
    })?;
    let until = NaiveDate::parse_from_str(valid_until.trim(), "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            1001,
            "valid_until format must be YYYY-MM-DD".to_string(),
        )
    })?;
    if from > until {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "valid_from must be before or equal valid_until".to_string(),
        ));
    }
    Ok(())
}

fn citizen_status_from_cpms(
    value: &str,
) -> Result<crate::citizens::model::CitizenStatus, (StatusCode, u32, String)> {
    match value.trim() {
        "NORMAL" => Ok(crate::citizens::model::CitizenStatus::Normal),
        "REVOKED" => Ok(crate::citizens::model::CitizenStatus::Revoked),
        _ => Err((
            StatusCode::BAD_REQUEST,
            1001,
            "citizen_status must be NORMAL or REVOKED".to_string(),
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
    valid_from: &str,
    valid_until: &str,
    status_updated_at: i64,
    cpms_pubkey: &str,
    geo_seal_hash: &str,
    wallet_address: &str,
    wallet_pubkey: &str,
) -> String {
    format!(
        "sfid-cpms-v1|archive|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        archive_no,
        citizen_status,
        voting_eligible,
        valid_from,
        valid_until,
        status_updated_at,
        cpms_pubkey,
        geo_seal_hash,
        wallet_address,
        wallet_pubkey
    )
}

fn can_transition_cpms_site_status(current: &CpmsSiteStatus, target: &CpmsSiteStatus) -> bool {
    matches!(
        (current, target),
        (CpmsSiteStatus::Active, CpmsSiteStatus::Disabled)
            | (CpmsSiteStatus::Active, CpmsSiteStatus::Revoked)
            | (CpmsSiteStatus::Disabled, CpmsSiteStatus::Active)
            | (CpmsSiteStatus::Disabled, CpmsSiteStatus::Revoked)
            | (CpmsSiteStatus::Pending, CpmsSiteStatus::Revoked)
            | (CpmsSiteStatus::Pending, CpmsSiteStatus::Disabled)
    )
}

pub(super) fn extract_province_code_from_sfid(sfid_number: &str) -> String {
    let segments: Vec<&str> = sfid_number.split('-').collect();
    if !segments.is_empty() && segments[0].len() >= 2 {
        segments[0][..2].to_string()
    } else {
        String::new()
    }
}

pub(super) fn extract_city_code_from_sfid(sfid_number: &str) -> String {
    let segments: Vec<&str> = sfid_number.split('-').collect();
    if !segments.is_empty() && segments[0].len() >= 5 {
        segments[0][2..5].to_string()
    } else {
        String::new()
    }
}

fn sign_with_main_key(_state: &AppState, message: &str) -> Result<String, String> {
    use sp_core::Pair;
    let seed_hex = std::env::var("SFID_SIGNING_SEED_HEX")
        .map_err(|_| "SFID_SIGNING_SEED_HEX not set".to_string())?;
    let keypair = crate::crypto::sr25519::try_load_signing_key_from_seed(seed_hex.as_str())
        .map_err(|e| format!("load signing key failed: {e}"))?;
    let sig = keypair.sign(message.as_bytes());
    Ok(format!("0x{}", hex::encode(sig.0)))
}

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
