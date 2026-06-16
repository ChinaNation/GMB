//! CPMS 年度报告导入。
//!
//! 中文注释：CPMS 是档案号、钱包、公民状态和投票资格的真源；SFID 是身份 ID
//! 的真源。本文件只按档案号覆盖已有 SFID 绑定，不自动生成新的身份 ID。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::login::AdminAuthContext;
use crate::admins::operation_auth::AdminActionType;
use crate::citizens::binding::{pubkey_hex_to_ss58, ss58_to_pubkey_hex};
use crate::*;

type Blake2b256 = Blake2b<U32>;

const EXPORT_PROTO: &str = "SFID_CPMS_V1";
const EXPORT_TYPE: &str = "CPMS_STATUS_EXPORT";
const EXPORT_VERSION: u32 = 1;
const WALLET_SIG_ALG_SR25519: &str = "sr25519";
const RELEASE_REASON_AFTER_100_YEARS: &str = "ARCHIVE_HARD_DELETED_AFTER_100_YEARS";

#[derive(Deserialize)]
pub(crate) struct CpmsStatusExportImportInput {
    export_file: CpmsStatusExportFile,
}

#[derive(Clone, Deserialize, Serialize)]
struct CpmsStatusExportFile {
    proto: String,
    r#type: String,
    version: u32,
    export_year: i32,
    sfid_number: String,
    cpms_pubkey: String,
    export_batch_id: String,
    exported_at: i64,
    citizen_binding_records_count: usize,
    binding_release_records_count: usize,
    records_hash: String,
    citizen_binding_records: Vec<CpmsCitizenBindingRecord>,
    binding_release_records: Vec<CpmsBindingReleaseRecord>,
    sig: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct CpmsCitizenBindingRecord {
    archive_no: String,
    wallet_address: String,
    wallet_pubkey: String,
    wallet_sig_alg: String,
    wallet_bound_at: i64,
    citizen_status: CitizenStatus,
    voting_eligible: bool,
    status_updated_at: i64,
}

#[derive(Clone, Deserialize, Serialize)]
struct CpmsBindingReleaseRecord {
    archive_no: String,
    released_at: i64,
    release_reason: String,
}

#[derive(Serialize)]
struct ExportRecordsForHash<'a> {
    citizen_binding_records: &'a [CpmsCitizenBindingRecord],
    binding_release_records: &'a [CpmsBindingReleaseRecord],
}

#[derive(Serialize)]
pub(crate) struct CpmsStatusExportImportOutput {
    sfid_number: String,
    export_year: i32,
    export_batch_id: String,
    already_imported: bool,
    imported_binding_records: usize,
    updated_binding_records: usize,
    wallet_replaced_records: usize,
    deleted_ineligible_records: usize,
    released_binding_records: usize,
    unmatched_binding_records: Vec<String>,
    unmatched_release_records: Vec<String>,
}

/// 导入 CPMS 年度报告。
///
/// 中文注释：接口开放给所有管理员；导入前先校验 CPMS 安装授权、公钥绑定、
/// records_hash 和 CPMS 签名，之后才进入一次性本地覆盖。
pub(crate) async fn admin_import_cpms_status_export(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsStatusExportImportInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let file = input.export_file;
    let grant_payload = serde_json::json!({
        "target": file.sfid_number.clone(),
        "sfid_number": file.sfid_number.clone(),
        "export_year": file.export_year,
        "export_batch_id": file.export_batch_id.clone(),
        "records_hash": file.records_hash.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CpmsStatusImportConfirm,
        file.sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    if let Err((status, code, message)) = validate_cpms_status_export(&state, &file, &ctx).await {
        return api_error(status, code, message.as_str());
    }

    if let Some(existing_hash) = match state
        .db
        .get_cpms_status_import_hash(&file.sfid_number, file.export_year)
    {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query cpms status import failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "import query failed",
            );
        }
    } {
        if existing_hash != file.records_hash {
            append_import_audit(
                &state,
                &ctx,
                &headers,
                &file,
                "FAILED",
                serde_json::json!({
                    "message": "annual export already imported with different records_hash",
                }),
            );
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "annual export already imported with different records_hash",
            );
        }
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: CpmsStatusExportImportOutput {
                sfid_number: file.sfid_number,
                export_year: file.export_year,
                export_batch_id: file.export_batch_id,
                already_imported: true,
                imported_binding_records: 0,
                updated_binding_records: 0,
                wallet_replaced_records: 0,
                deleted_ineligible_records: 0,
                released_binding_records: 0,
                unmatched_binding_records: Vec::new(),
                unmatched_release_records: Vec::new(),
            },
        })
        .into_response();
    }

    let output = match apply_status_export_to_db(&state, &ctx, &file) {
        Ok(v) => v,
        Err(message) => {
            append_import_audit(
                &state,
                &ctx,
                &headers,
                &file,
                "FAILED",
                serde_json::json!({ "message": message.clone() }),
            );
            return api_error(StatusCode::CONFLICT, 1005, message.as_str());
        }
    };
    if let Err(err) = state.db.insert_cpms_status_import(
        &file.sfid_number,
        file.export_year,
        &file.export_batch_id,
        &file.records_hash,
        &ctx.admin_pubkey,
        &file,
    ) {
        tracing::error!(error = %err, "insert cpms status import failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "import record write failed",
        );
    }
    append_import_audit(
        &state,
        &ctx,
        &headers,
        &file,
        "SUCCESS",
        serde_json::json!({
            "updates": output.updated_binding_records,
            "wallet_replaced": output.wallet_replaced_records,
            "deleted_ineligible": output.deleted_ineligible_records,
            "releases": output.released_binding_records,
            "unmatched_bindings": output.unmatched_binding_records.len(),
            "unmatched_releases": output.unmatched_release_records.len(),
        }),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

async fn validate_cpms_status_export(
    state: &AppState,
    file: &CpmsStatusExportFile,
    ctx: &AdminAuthContext,
) -> Result<(), (StatusCode, u32, String)> {
    validate_export_header(file)?;
    validate_export_records(file)?;

    let site = state
        .db
        .get_cpms_site(file.sfid_number.as_str())
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, 1004, err))?
        .ok_or((
            StatusCode::NOT_FOUND,
            1004,
            "cpms install authorization not found".to_string(),
        ))?;
    if !in_scope_cpms_site(&site, ctx.admin_province.as_deref()) {
        return Err((
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province institutions".to_string(),
        ));
    }
    if let Some(city) = ctx.admin_city.as_deref() {
        if site.city_name != city {
            return Err((
                StatusCode::FORBIDDEN,
                1003,
                "cannot manage other city institutions".to_string(),
            ));
        }
    }
    if site.status != CpmsSiteStatus::Active
        || site.install_token_status == InstallTokenStatus::Revoked
    {
        return Err((
            StatusCode::FORBIDDEN,
            1003,
            "CPMS install authorization is not active".to_string(),
        ));
    }
    let cpms_pubkey_hash = hash_hex(file.cpms_pubkey.as_bytes());
    match site.cpms_pubkey_hash.as_deref() {
        Some(expected) if expected == cpms_pubkey_hash => {}
        Some(_) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "cpms_pubkey does not match installed CPMS".to_string(),
            ))
        }
        None => {
            return Err((
                StatusCode::CONFLICT,
                1005,
                "CPMS public key is not bound by archive verification".to_string(),
            ))
        }
    }

    let cpms_pubkey = parse_sr25519_pubkey_bytes(file.cpms_pubkey.as_str()).ok_or((
        StatusCode::BAD_REQUEST,
        1001,
        "cpms_pubkey format invalid".to_string(),
    ))?;
    let signature = hex::decode(file.sig.trim().trim_start_matches("0x")).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            1001,
            "status export sig hex decode failed".to_string(),
        )
    })?;
    let sign_source = build_status_export_sign_source(
        &file.sfid_number,
        &file.cpms_pubkey,
        &file.export_batch_id,
        file.exported_at,
        &file.records_hash,
    );
    if !crate::cpms::handler::verify_sr25519_signature(&cpms_pubkey, &sign_source, &signature) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "status export signature invalid".to_string(),
        ));
    }
    Ok(())
}

fn validate_export_header(file: &CpmsStatusExportFile) -> Result<(), (StatusCode, u32, String)> {
    if file.proto != EXPORT_PROTO {
        return Err((StatusCode::BAD_REQUEST, 1001, "proto invalid".to_string()));
    }
    if file.r#type != EXPORT_TYPE {
        return Err((StatusCode::BAD_REQUEST, 1001, "type invalid".to_string()));
    }
    if file.version != EXPORT_VERSION {
        return Err((StatusCode::BAD_REQUEST, 1001, "version invalid".to_string()));
    }
    if file.export_year <= 0
        || file.sfid_number.trim().is_empty()
        || file.cpms_pubkey.trim().is_empty()
        || file.export_batch_id.trim().is_empty()
        || file.exported_at <= 0
        || file.records_hash.trim().is_empty()
        || file.sig.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "status export header fields are required".to_string(),
        ));
    }
    Ok(())
}

fn validate_export_records(file: &CpmsStatusExportFile) -> Result<(), (StatusCode, u32, String)> {
    if file.citizen_binding_records_count != file.citizen_binding_records.len()
        || file.binding_release_records_count != file.binding_release_records.len()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            1001,
            "status export record count mismatch".to_string(),
        ));
    }
    let computed_hash = records_hash(&file.citizen_binding_records, &file.binding_release_records)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                1001,
                "status export records hash failed".to_string(),
            )
        })?;
    if computed_hash != file.records_hash {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "status export records_hash mismatch".to_string(),
        ));
    }

    let mut binding_archives = HashSet::new();
    for record in &file.citizen_binding_records {
        if record.archive_no.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "citizen binding archive_no is required".to_string(),
            ));
        }
        if !binding_archives.insert(record.archive_no.clone()) {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "duplicate citizen binding archive_no".to_string(),
            ));
        }
    }

    let mut release_archives = HashSet::new();
    for record in &file.binding_release_records {
        if record.archive_no.trim().is_empty() || record.released_at <= 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "binding release record fields are required".to_string(),
            ));
        }
        if !release_archives.insert(record.archive_no.clone()) {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "duplicate binding release archive_no".to_string(),
            ));
        }
        if binding_archives.contains(record.archive_no.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "archive_no cannot appear in both binding and release records".to_string(),
            ));
        }
        if record.release_reason != RELEASE_REASON_AFTER_100_YEARS {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "binding release reason invalid".to_string(),
            ));
        }
    }
    Ok(())
}

fn apply_status_export_to_db(
    state: &AppState,
    ctx: &AdminAuthContext,
    file: &CpmsStatusExportFile,
) -> Result<CpmsStatusExportImportOutput, String> {
    let imported_binding_records = file.citizen_binding_records.len();
    let mut updated_binding_records = 0usize;
    let mut wallet_replaced_records = 0usize;
    let mut deleted_ineligible_records = 0usize;
    let mut released_binding_records = 0usize;
    let mut unmatched_binding_records = Vec::new();
    let mut unmatched_release_records = Vec::new();
    let release_archive_nos: HashSet<String> = file
        .binding_release_records
        .iter()
        .map(|record| record.archive_no.clone())
        .collect();

    for record in &file.citizen_binding_records {
        let Some(mut existing) = state
            .db
            .find_bound_citizen_by_archive(record.archive_no.as_str())?
        else {
            unmatched_binding_records.push(record.archive_no.clone());
            continue;
        };
        if existing.bind_status() != CitizenBindStatus::Bound {
            unmatched_binding_records.push(record.archive_no.clone());
            continue;
        }
        if !status_export_record_has_required_voter_fields(record) {
            state.db.delete_citizen_binding_record(&existing)?;
            deleted_ineligible_records += 1;
            continue;
        }
        let Some((normalized_wallet_pubkey, canonical_wallet_address)) =
            normalized_status_export_wallet(record)
        else {
            state.db.delete_citizen_binding_record(&existing)?;
            deleted_ineligible_records += 1;
            continue;
        };
        if let Some(owner) = state
            .db
            .find_bound_citizen_by_wallet(normalized_wallet_pubkey.as_str())?
        {
            if owner.id != existing.id
                && !owner
                    .archive_no
                    .as_ref()
                    .is_some_and(|archive_no| release_archive_nos.contains(archive_no))
            {
                return Err("wallet_pubkey already bound to another archive_no".to_string());
            }
        }
        if existing.wallet_pubkey.as_deref() != Some(normalized_wallet_pubkey.as_str()) {
            wallet_replaced_records += 1;
        }
        existing.wallet_pubkey = Some(normalized_wallet_pubkey);
        existing.wallet_address = Some(canonical_wallet_address);
        existing.citizen_status = Some(record.citizen_status.clone());
        existing.voting_eligible =
            record.voting_eligible && record.citizen_status == CitizenStatus::Normal;
        existing.status_updated_at = Some(record.status_updated_at);
        existing.bound_at = chrono::DateTime::<Utc>::from_timestamp(record.wallet_bound_at, 0);
        existing.bound_by = Some(ctx.admin_pubkey.clone());
        state.db.upsert_citizen_row(&existing)?;
        updated_binding_records += 1;
    }

    for release in &file.binding_release_records {
        let Some(existing) = state
            .db
            .find_bound_citizen_by_archive(release.archive_no.as_str())?
        else {
            unmatched_release_records.push(release.archive_no.clone());
            continue;
        };
        if existing.bind_status() != CitizenBindStatus::Bound {
            unmatched_release_records.push(release.archive_no.clone());
            continue;
        }
        state.db.delete_citizen_binding_record(&existing)?;
        released_binding_records += 1;
    }

    Ok(CpmsStatusExportImportOutput {
        sfid_number: file.sfid_number.clone(),
        export_year: file.export_year,
        export_batch_id: file.export_batch_id.clone(),
        already_imported: false,
        imported_binding_records,
        updated_binding_records,
        wallet_replaced_records,
        deleted_ineligible_records,
        released_binding_records,
        unmatched_binding_records,
        unmatched_release_records,
    })
}

fn status_export_record_has_required_voter_fields(record: &CpmsCitizenBindingRecord) -> bool {
    // 中文注释:先筛状态、资格、算法、时间和钱包字段完整性;地址与公钥一致性由下一步规范化校验决定。
    record.citizen_status == CitizenStatus::Normal
        && record.voting_eligible
        && record.wallet_sig_alg == WALLET_SIG_ALG_SR25519
        && record.wallet_bound_at > 0
        && record.status_updated_at > 0
        && !record.wallet_address.trim().is_empty()
        && !record.wallet_pubkey.trim().is_empty()
}

/// 中文注释:导入审计 = 基础事实(年度/批次/结果/请求来源) + 调用方扩展字段(extra,
/// 必须是 JSON 对象:失败传 {"message": …},成功传各计数字段),合并后整体入审计表。
fn append_import_audit(
    state: &AppState,
    ctx: &AdminAuthContext,
    headers: &HeaderMap,
    file: &CpmsStatusExportFile,
    result: &'static str,
    extra: serde_json::Value,
) {
    let mut detail = serde_json::json!({
        "year": file.export_year,
        "batch": file.export_batch_id.clone(),
        "result": result,
        "request_id": request_id_from_headers(headers),
        "actor_ip": actor_ip_from_headers(headers),
    });
    if let (Some(base), Some(extra_map)) = (detail.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_map {
            base.insert(key.clone(), value.clone());
        }
    }
    crate::core::runtime_ops::append_audit_log(
        state,
        "CPMS_STATUS_EXPORT_IMPORT",
        &ctx.admin_pubkey,
        Some(file.sfid_number.clone()),
        detail,
    );
}

fn records_hash(
    citizen_binding_records: &[CpmsCitizenBindingRecord],
    binding_release_records: &[CpmsBindingReleaseRecord],
) -> Result<String, serde_json::Error> {
    let json = serde_json::to_vec(&ExportRecordsForHash {
        citizen_binding_records,
        binding_release_records,
    })?;
    Ok(hash_hex(&json))
}

fn build_status_export_sign_source(
    sfid_number: &str,
    cpms_pubkey: &str,
    export_batch_id: &str,
    exported_at: i64,
    records_hash: &str,
) -> String {
    format!(
        "sfid-cpms-v1|cpms-status-export|{}|{}|{}|{}|{}",
        sfid_number, cpms_pubkey, export_batch_id, exported_at, records_hash
    )
}

fn normalized_status_export_wallet(record: &CpmsCitizenBindingRecord) -> Option<(String, String)> {
    let normalized_wallet_pubkey = normalize_pubkey_hex(record.wallet_pubkey.as_str())?;
    let wallet_address = record.wallet_address.trim();
    let decoded_pubkey = ss58_to_pubkey_hex(wallet_address)?;
    if decoded_pubkey != normalized_wallet_pubkey {
        return None;
    }
    let canonical_address = pubkey_hex_to_ss58(normalized_wallet_pubkey.as_str())?;
    if canonical_address != wallet_address {
        return None;
    }
    Some((normalized_wallet_pubkey, canonical_address))
}

fn normalize_pubkey_hex(pubkey: &str) -> Option<String> {
    let bytes = parse_sr25519_pubkey_bytes(pubkey)?;
    Some(format!("0x{}", hex::encode(bytes)))
}

fn hash_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(Blake2b256::digest(bytes)))
}

impl Db {
    fn get_cpms_status_import_hash(
        &self,
        sfid_number: &str,
        export_year: i32,
    ) -> Result<Option<String>, String> {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT records_hash
                     FROM citizen_status_imports
                     WHERE sfid_number = $1 AND export_year = $2",
                    &[&sfid_number, &export_year],
                )
                .map_err(|e| format!("query status import failed: {e}"))?;
            Ok(row.map(|row| row.get(0)))
        })
    }

    fn insert_cpms_status_import(
        &self,
        sfid_number: &str,
        export_year: i32,
        export_batch_id: &str,
        records_hash: &str,
        imported_by: &str,
        file: &CpmsStatusExportFile,
    ) -> Result<(), String> {
        let sfid_number = sfid_number.trim().to_string();
        let export_batch_id = export_batch_id.trim().to_string();
        let records_hash = records_hash.trim().to_string();
        let imported_by = imported_by.trim().to_string();
        let payload = serde_json::to_value(file)
            .map_err(|e| format!("serialize cpms status import failed: {e}"))?;
        let imported_at = Utc::now();
        self.with_client(move |conn| {
            conn.execute(
                "INSERT INTO citizen_status_imports (
                    sfid_number, export_year, export_batch_id, records_hash,
                    imported_at, imported_by, payload
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &sfid_number,
                    &export_year,
                    &export_batch_id,
                    &records_hash,
                    &imported_at,
                    &imported_by,
                    &payload,
                ],
            )
            .map_err(|e| format!("insert status import failed: {e}"))?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file() -> CpmsStatusExportFile {
        let mut file = CpmsStatusExportFile {
            proto: EXPORT_PROTO.to_string(),
            r#type: EXPORT_TYPE.to_string(),
            version: EXPORT_VERSION,
            export_year: 2026,
            sfid_number: "GD001-GZF06-123456789-2026".to_string(),
            cpms_pubkey: "0x11".repeat(32),
            export_batch_id: "cse_test".to_string(),
            exported_at: 1,
            citizen_binding_records_count: 0,
            binding_release_records_count: 1,
            records_hash: String::new(),
            citizen_binding_records: Vec::new(),
            binding_release_records: vec![CpmsBindingReleaseRecord {
                archive_no: "ARCHIVE-OLD".to_string(),
                released_at: 2,
                release_reason: RELEASE_REASON_AFTER_100_YEARS.to_string(),
            }],
            sig: "0xsig".to_string(),
        };
        file.records_hash =
            records_hash(&file.citizen_binding_records, &file.binding_release_records).unwrap();
        file
    }

    #[test]
    fn records_hash_is_stable_for_same_payload() {
        let file = sample_file();
        let first =
            records_hash(&file.citizen_binding_records, &file.binding_release_records).unwrap();
        let second =
            records_hash(&file.citizen_binding_records, &file.binding_release_records).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn release_reason_is_strict() {
        let mut file = sample_file();
        file.binding_release_records[0].release_reason = "OLD".to_string();
        file.records_hash =
            records_hash(&file.citizen_binding_records, &file.binding_release_records).unwrap();
        assert!(validate_export_records(&file).is_err());
    }

    #[test]
    fn ineligible_binding_record_does_not_reject_whole_export() {
        let mut file = sample_file();
        file.binding_release_records.clear();
        file.binding_release_records_count = 0;
        file.citizen_binding_records = vec![CpmsCitizenBindingRecord {
            archive_no: "ARCHIVE-NO-VOTE".to_string(),
            wallet_address: String::new(),
            wallet_pubkey: String::new(),
            wallet_sig_alg: String::new(),
            wallet_bound_at: 0,
            citizen_status: CitizenStatus::Normal,
            voting_eligible: false,
            status_updated_at: 0,
        }];
        file.citizen_binding_records_count = file.citizen_binding_records.len();
        file.records_hash =
            records_hash(&file.citizen_binding_records, &file.binding_release_records).unwrap();
        assert!(validate_export_records(&file).is_ok());
    }

    #[test]
    fn normal_voting_citizen_can_remain_in_sfid() {
        let record = CpmsCitizenBindingRecord {
            archive_no: "ARCHIVE-NORMAL".to_string(),
            wallet_address: "address".to_string(),
            wallet_pubkey: "0x11".to_string(),
            wallet_sig_alg: WALLET_SIG_ALG_SR25519.to_string(),
            wallet_bound_at: 1,
            citizen_status: CitizenStatus::Normal,
            voting_eligible: true,
            status_updated_at: 2,
        };
        assert!(status_export_record_has_required_voter_fields(&record));
    }

    #[test]
    fn ineligible_citizen_must_be_deleted_from_sfid() {
        let record = CpmsCitizenBindingRecord {
            archive_no: "ARCHIVE-INELIGIBLE".to_string(),
            wallet_address: "address".to_string(),
            wallet_pubkey: "0x11".to_string(),
            wallet_sig_alg: WALLET_SIG_ALG_SR25519.to_string(),
            wallet_bound_at: 1,
            citizen_status: CitizenStatus::Normal,
            voting_eligible: false,
            status_updated_at: 2,
        };
        assert!(!status_export_record_has_required_voter_fields(&record));
    }
}
