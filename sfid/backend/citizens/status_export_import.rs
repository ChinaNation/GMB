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
    released_binding_records: usize,
    unmatched_binding_records: Vec<String>,
    unmatched_release_records: Vec<String>,
}

struct ImportPlan {
    updates: Vec<BindingUpdatePlan>,
    releases: Vec<BindingReleasePlan>,
    unmatched_binding_records: Vec<String>,
    unmatched_release_records: Vec<String>,
}

struct BindingUpdatePlan {
    citizen_id: u64,
    record: CpmsCitizenBindingRecord,
    normalized_wallet_pubkey: String,
    canonical_wallet_address: String,
    wallet_changed: bool,
}

struct BindingReleasePlan {
    citizen_id: u64,
    released_at: i64,
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

    let import_key = import_record_key(&file.sfid_number, file.export_year);
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Some(existing) = store.cpms_status_export_imports.get(import_key.as_str()) {
        if existing.records_hash != file.records_hash {
            append_import_audit(
                &mut store,
                &ctx,
                &headers,
                &file,
                "FAILED",
                "annual export already imported with different records_hash".to_string(),
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
                released_binding_records: 0,
                unmatched_binding_records: Vec::new(),
                unmatched_release_records: Vec::new(),
            },
        })
        .into_response();
    }

    let plan = match build_import_plan(&store, &file) {
        Ok(v) => v,
        Err(message) => {
            append_import_audit(&mut store, &ctx, &headers, &file, "FAILED", message.clone());
            return api_error(StatusCode::CONFLICT, 1005, message.as_str());
        }
    };
    let affected_citizen_ids: Vec<u64> = plan
        .updates
        .iter()
        .map(|item| item.citizen_id)
        .chain(plan.releases.iter().map(|item| item.citizen_id))
        .collect();
    let output = apply_import_plan(&mut store, &ctx, &file, plan);
    // 中文注释:管理员公民列表读取 citizens 分区表;CPMS 年度导入改动 Store 后,
    // 必须把受影响记录同步写入目标表,否则精确检索会读到过期状态或空结果。
    let changed_records: Vec<_> = affected_citizen_ids
        .iter()
        .filter_map(|id| store.citizen_records.get(id).cloned())
        .collect();
    store.cpms_status_export_imports.insert(
        import_key,
        CpmsStatusExportImportRecord {
            sfid_number: file.sfid_number.clone(),
            export_year: file.export_year,
            export_batch_id: file.export_batch_id.clone(),
            records_hash: file.records_hash.clone(),
            imported_at: Utc::now(),
            imported_by: ctx.admin_pubkey.clone(),
        },
    );
    append_import_audit(
        &mut store,
        &ctx,
        &headers,
        &file,
        "SUCCESS",
        format!(
            "updates={} wallet_replaced={} releases={} unmatched_bindings={} unmatched_releases={}",
            output.updated_binding_records,
            output.wallet_replaced_records,
            output.released_binding_records,
            output.unmatched_binding_records.len(),
            output.unmatched_release_records.len()
        ),
    );
    drop(store);

    for record in changed_records {
        if let Err(e) = state.store.upsert_citizen_row(&record) {
            tracing::error!(citizen_id = record.id, error = %e, "citizen row upsert failed after CPMS status import");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "citizen row write failed",
            );
        }
    }

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

    let site = {
        let store = state.store.read().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "store read failed".to_string(),
            )
        })?;
        store
            .cpms_site_keys
            .get(file.sfid_number.as_str())
            .cloned()
            .ok_or((
                StatusCode::NOT_FOUND,
                1004,
                "cpms install authorization not found".to_string(),
            ))?
    };
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
        if record.archive_no.trim().is_empty()
            || record.wallet_address.trim().is_empty()
            || record.wallet_pubkey.trim().is_empty()
            || record.wallet_bound_at <= 0
            || record.status_updated_at <= 0
        {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "citizen binding record fields are required".to_string(),
            ));
        }
        if !binding_archives.insert(record.archive_no.clone()) {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "duplicate citizen binding archive_no".to_string(),
            ));
        }
        if record.wallet_sig_alg != WALLET_SIG_ALG_SR25519 {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "wallet_sig_alg must be sr25519".to_string(),
            ));
        }
        if record.citizen_status == CitizenStatus::Revoked && record.voting_eligible {
            return Err((
                StatusCode::BAD_REQUEST,
                1001,
                "revoked citizen must not have voting eligibility".to_string(),
            ));
        }
        validate_wallet_pair(
            record.wallet_address.as_str(),
            record.wallet_pubkey.as_str(),
        )?;
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

fn build_import_plan(store: &Store, file: &CpmsStatusExportFile) -> Result<ImportPlan, String> {
    let mut updates = Vec::new();
    let mut releases = Vec::new();
    let mut unmatched_binding_records = Vec::new();
    let mut unmatched_release_records = Vec::new();
    let release_archive_nos: HashSet<String> = file
        .binding_release_records
        .iter()
        .map(|record| record.archive_no.clone())
        .collect();

    for record in &file.citizen_binding_records {
        let normalized_wallet_pubkey = normalize_pubkey_hex(record.wallet_pubkey.as_str())
            .ok_or_else(|| "invalid wallet_pubkey".to_string())?;
        let canonical_wallet_address = pubkey_hex_to_ss58(normalized_wallet_pubkey.as_str())
            .ok_or_else(|| "invalid wallet_pubkey".to_string())?;
        let wallet_owner = store
            .citizen_id_by_wallet_pubkey
            .get(normalized_wallet_pubkey.as_str())
            .copied();
        let citizen_id = match store
            .citizen_id_by_archive_no
            .get(record.archive_no.as_str())
        {
            Some(cid) => *cid,
            None => {
                if let Some(owner) = wallet_owner {
                    if is_bound_citizen_record(store, owner)
                        && !owner_archive_is_released(store, owner, &release_archive_nos)
                    {
                        return Err("wallet_pubkey already bound to another archive_no".to_string());
                    }
                }
                unmatched_binding_records.push(record.archive_no.clone());
                continue;
            }
        };
        let Some(existing) = store.citizen_records.get(&citizen_id) else {
            unmatched_binding_records.push(record.archive_no.clone());
            continue;
        };
        if existing.bind_status() != CitizenBindStatus::Bound {
            unmatched_binding_records.push(record.archive_no.clone());
            continue;
        }
        if let Some(owner) = wallet_owner {
            if owner != citizen_id
                && is_bound_citizen_record(store, owner)
                && !owner_archive_is_released(store, owner, &release_archive_nos)
            {
                return Err("wallet_pubkey already bound to another archive_no".to_string());
            }
        }
        let wallet_changed =
            existing.wallet_pubkey.as_deref() != Some(normalized_wallet_pubkey.as_str());
        updates.push(BindingUpdatePlan {
            citizen_id,
            record: record.clone(),
            normalized_wallet_pubkey,
            canonical_wallet_address,
            wallet_changed,
        });
    }

    for record in &file.binding_release_records {
        let Some(citizen_id) = store
            .citizen_id_by_archive_no
            .get(record.archive_no.as_str())
            .copied()
        else {
            unmatched_release_records.push(record.archive_no.clone());
            continue;
        };
        let Some(existing) = store.citizen_records.get(&citizen_id) else {
            unmatched_release_records.push(record.archive_no.clone());
            continue;
        };
        if existing.bind_status() != CitizenBindStatus::Bound {
            unmatched_release_records.push(record.archive_no.clone());
            continue;
        }
        releases.push(BindingReleasePlan {
            citizen_id,
            released_at: record.released_at,
        });
    }

    Ok(ImportPlan {
        updates,
        releases,
        unmatched_binding_records,
        unmatched_release_records,
    })
}

fn apply_import_plan(
    store: &mut Store,
    ctx: &AdminAuthContext,
    file: &CpmsStatusExportFile,
    plan: ImportPlan,
) -> CpmsStatusExportImportOutput {
    let imported_binding_records = file.citizen_binding_records.len();
    let updated_binding_records = plan.updates.len();
    let wallet_replaced_records = plan
        .updates
        .iter()
        .filter(|item| item.wallet_changed)
        .count();
    let released_binding_records = plan.releases.len();

    for release in plan.releases {
        let Some(existing) = store.citizen_records.get(&release.citizen_id).cloned() else {
            continue;
        };
        if let Some(archive_no) = existing.archive_no.as_deref() {
            store.citizen_id_by_archive_no.remove(archive_no);
        }
        if let Some(wallet_pubkey) = existing.wallet_pubkey.as_deref() {
            store.citizen_id_by_wallet_pubkey.remove(wallet_pubkey);
            invalidate_vote_cache_for_pubkey(store, wallet_pubkey);
        }
        if let Some(sfid_code) = existing.sfid_code.as_deref() {
            store.citizen_id_by_sfid_code.remove(sfid_code);
        }
        if let Some(record) = store.citizen_records.get_mut(&release.citizen_id) {
            record.wallet_pubkey = None;
            record.wallet_address = None;
            record.archive_no = None;
            record.sfid_code = None;
            record.citizen_status = Some(CitizenStatus::Revoked);
            record.voting_eligible = false;
            record.archive_valid_from = None;
            record.archive_valid_until = None;
            record.status_updated_at = Some(release.released_at);
            record.bound_at = None;
            record.bound_by = Some(ctx.admin_pubkey.clone());
        }
    }

    for update in plan.updates {
        let old_wallet_pubkey = store
            .citizen_records
            .get(&update.citizen_id)
            .and_then(|record| record.wallet_pubkey.clone());
        if let Some(old_pubkey) = old_wallet_pubkey.as_deref() {
            if old_pubkey != update.normalized_wallet_pubkey {
                store.citizen_id_by_wallet_pubkey.remove(old_pubkey);
                invalidate_vote_cache_for_pubkey(store, old_pubkey);
            }
        }
        let Some(record) = store.citizen_records.get_mut(&update.citizen_id) else {
            continue;
        };
        let voting_eligible =
            update.record.voting_eligible && update.record.citizen_status == CitizenStatus::Normal;
        record.wallet_pubkey = Some(update.normalized_wallet_pubkey.clone());
        record.wallet_address = Some(update.canonical_wallet_address);
        record.citizen_status = Some(update.record.citizen_status);
        record.voting_eligible = voting_eligible;
        record.status_updated_at = Some(update.record.status_updated_at);
        record.bound_at = chrono::DateTime::<Utc>::from_timestamp(update.record.wallet_bound_at, 0);
        record.bound_by = Some(ctx.admin_pubkey.clone());
        store
            .citizen_id_by_archive_no
            .insert(update.record.archive_no, update.citizen_id);
        store
            .citizen_id_by_wallet_pubkey
            .insert(update.normalized_wallet_pubkey.clone(), update.citizen_id);
        invalidate_vote_cache_for_pubkey(store, update.normalized_wallet_pubkey.as_str());
    }

    CpmsStatusExportImportOutput {
        sfid_number: file.sfid_number.clone(),
        export_year: file.export_year,
        export_batch_id: file.export_batch_id.clone(),
        already_imported: false,
        imported_binding_records,
        updated_binding_records,
        wallet_replaced_records,
        released_binding_records,
        unmatched_binding_records: plan.unmatched_binding_records,
        unmatched_release_records: plan.unmatched_release_records,
    }
}

fn append_import_audit(
    store: &mut Store,
    ctx: &AdminAuthContext,
    headers: &HeaderMap,
    file: &CpmsStatusExportFile,
    result: &'static str,
    detail: String,
) {
    append_audit_log_with_meta(
        store,
        "CPMS_STATUS_EXPORT_IMPORT",
        &ctx.admin_pubkey,
        None,
        Some(format!("{}:{}", file.sfid_number, file.export_year)),
        request_id_from_headers(headers),
        actor_ip_from_headers(headers),
        result,
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

fn validate_wallet_pair(
    wallet_address: &str,
    wallet_pubkey: &str,
) -> Result<(), (StatusCode, u32, String)> {
    let normalized_wallet_pubkey = normalize_pubkey_hex(wallet_pubkey).ok_or((
        StatusCode::BAD_REQUEST,
        1001,
        "invalid wallet_pubkey".to_string(),
    ))?;
    let decoded_pubkey = ss58_to_pubkey_hex(wallet_address).ok_or((
        StatusCode::BAD_REQUEST,
        1001,
        "invalid wallet_address".to_string(),
    ))?;
    if decoded_pubkey != normalized_wallet_pubkey {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "wallet_address and wallet_pubkey mismatch".to_string(),
        ));
    }
    let canonical_address = pubkey_hex_to_ss58(normalized_wallet_pubkey.as_str()).ok_or((
        StatusCode::BAD_REQUEST,
        1001,
        "invalid wallet_pubkey".to_string(),
    ))?;
    if canonical_address != wallet_address.trim() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "wallet_address is not canonical".to_string(),
        ));
    }
    Ok(())
}

fn normalize_pubkey_hex(pubkey: &str) -> Option<String> {
    let bytes = parse_sr25519_pubkey_bytes(pubkey)?;
    Some(format!("0x{}", hex::encode(bytes)))
}

fn is_bound_citizen_record(store: &Store, citizen_id: u64) -> bool {
    store
        .citizen_records
        .get(&citizen_id)
        .map(|record| record.bind_status() == CitizenBindStatus::Bound)
        .unwrap_or(false)
}

fn owner_archive_is_released(
    store: &Store,
    citizen_id: u64,
    release_archive_nos: &HashSet<String>,
) -> bool {
    store
        .citizen_records
        .get(&citizen_id)
        .and_then(|record| record.archive_no.as_ref())
        .map(|archive_no| release_archive_nos.contains(archive_no))
        .unwrap_or(false)
}

fn hash_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(Blake2b256::digest(bytes)))
}

fn import_record_key(sfid_number: &str, export_year: i32) -> String {
    format!("{}|{}", sfid_number.trim(), export_year)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_file() -> CpmsStatusExportFile {
        let mut file = CpmsStatusExportFile {
            proto: EXPORT_PROTO.to_string(),
            r#type: EXPORT_TYPE.to_string(),
            version: EXPORT_VERSION,
            export_year: 2026,
            sfid_number: "GFR-GD001-ZG0X-123456789-2026".to_string(),
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

    fn bound_record(
        id: u64,
        archive_no: &str,
        wallet_pubkey: &str,
        sfid_code: &str,
    ) -> CitizenRecord {
        CitizenRecord {
            id,
            wallet_pubkey: Some(wallet_pubkey.to_string()),
            wallet_address: Some(
                pubkey_hex_to_ss58(wallet_pubkey).unwrap_or_else(|| "5F-test".to_string()),
            ),
            archive_no: Some(archive_no.to_string()),
            sfid_code: Some(sfid_code.to_string()),
            citizen_status: Some(CitizenStatus::Normal),
            voting_eligible: true,
            archive_valid_from: Some("2026-01-01".to_string()),
            archive_valid_until: Some("2035-12-31".to_string()),
            status_updated_at: Some(1),
            sfid_signature: None,
            province_code: Some("GD".to_string()),
            city_code: Some("001".to_string()),
            bound_at: Some(Utc::now()),
            bound_by: Some("admin".to_string()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn release_in_same_report_allows_wallet_to_move_to_another_archive() {
        let old_wallet = format!("0x{}", "11".repeat(32));
        let new_wallet = format!("0x{}", "22".repeat(32));
        let old_wallet_address = pubkey_hex_to_ss58(&old_wallet).unwrap();
        let mut store = Store::default();
        store
            .citizen_records
            .insert(1, bound_record(1, "ARCHIVE-OLD", &old_wallet, "SFID-OLD"));
        store
            .citizen_records
            .insert(2, bound_record(2, "ARCHIVE-NEW", &new_wallet, "SFID-NEW"));
        store
            .citizen_id_by_archive_no
            .insert("ARCHIVE-OLD".to_string(), 1);
        store
            .citizen_id_by_archive_no
            .insert("ARCHIVE-NEW".to_string(), 2);
        store
            .citizen_id_by_wallet_pubkey
            .insert(old_wallet.clone(), 1);
        store
            .citizen_id_by_wallet_pubkey
            .insert(new_wallet.clone(), 2);
        store
            .citizen_id_by_sfid_code
            .insert("SFID-OLD".to_string(), 1);
        store
            .citizen_id_by_sfid_code
            .insert("SFID-NEW".to_string(), 2);

        let mut file = sample_file();
        file.citizen_binding_records = vec![CpmsCitizenBindingRecord {
            archive_no: "ARCHIVE-NEW".to_string(),
            wallet_address: old_wallet_address,
            wallet_pubkey: old_wallet.clone(),
            wallet_sig_alg: WALLET_SIG_ALG_SR25519.to_string(),
            wallet_bound_at: 10,
            citizen_status: CitizenStatus::Normal,
            voting_eligible: true,
            status_updated_at: 10,
        }];
        file.binding_release_records = vec![CpmsBindingReleaseRecord {
            archive_no: "ARCHIVE-OLD".to_string(),
            released_at: 11,
            release_reason: RELEASE_REASON_AFTER_100_YEARS.to_string(),
        }];
        file.citizen_binding_records_count = 1;
        file.binding_release_records_count = 1;
        file.records_hash =
            records_hash(&file.citizen_binding_records, &file.binding_release_records).unwrap();

        let plan = build_import_plan(&store, &file).expect("plan");
        assert_eq!(plan.updates.len(), 1);
        assert_eq!(plan.releases.len(), 1);

        let ctx = AdminAuthContext {
            admin_pubkey: "admin".to_string(),
            role: AdminRole::ShengAdmin,
            admin_name: "管理员".to_string(),
            admin_province: None,
            admin_city: None,
            passkey_bound: false,
        };
        let output = apply_import_plan(&mut store, &ctx, &file, plan);
        assert_eq!(output.updated_binding_records, 1);
        assert_eq!(output.released_binding_records, 1);
        assert_eq!(store.citizen_id_by_wallet_pubkey.get(&old_wallet), Some(&2));
        assert!(store
            .citizen_records
            .get(&1)
            .and_then(|record| record.sfid_code.as_ref())
            .is_none());
    }
}
