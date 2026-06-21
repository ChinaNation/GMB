use axum::{http::StatusCode, Json};
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

use crate::{
    common::{err, ApiError},
    initialize, AppState,
};

use super::{
    active_archive_sign_key, effective_voting_eligible, sign_archive_payload_with_secret,
    CITIZEN_STATUS_REVOKED,
};

type Blake2b256 = Blake2b<U32>;

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct CpmsStatusExportFile {
    proto: String,
    r#type: String,
    version: u32,
    export_year: i32,
    sfid_number: String,
    cpms_pubkey: String,
    export_batch_id: String,
    pub(crate) exported_at: i64,
    citizen_binding_records_count: usize,
    binding_release_records_count: usize,
    records_hash: String,
    citizen_binding_records: Vec<CpmsCitizenBindingRecord>,
    binding_release_records: Vec<CpmsBindingReleaseRecord>,
    sig: String,
}

#[derive(Clone, Serialize)]
pub(crate) struct CpmsStatusExportState {
    now_utc: i64,
    pending_export_year: Option<i32>,
    can_export: bool,
    reminder_active: bool,
    operator_lock_active: bool,
    exported: bool,
    next_export_available_at: Option<i64>,
    disabled_reason: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct CpmsCitizenBindingRecord {
    archive_no: String,
    wallet_address: String,
    wallet_pubkey: String,
    wallet_sig_alg: String,
    wallet_bound_at: i64,
    citizen_status: String,
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

/// 构造 CPMS 离线年度状态导出文件。
///
/// 中文注释：导出文件只给 SFID 更新档案号、钱包、公民状态和投票资格绑定事实。
/// 姓名、出生日期、地址、护照号等 CPMS 内部实名资料不进入年度报告。
pub(crate) async fn build_and_record_cpms_status_export(
    state: &AppState,
) -> Result<CpmsStatusExportFile, (StatusCode, Json<ApiError>)> {
    let now = Utc::now();
    let export_year = resolve_export_year(state, now).await?.ok_or_else(|| {
        err(
            StatusCode::CONFLICT,
            3018,
            "annual status export not required",
        )
    })?;
    let export_file = build_cpms_status_export(state, export_year, now).await?;

    sqlx::query(
        "INSERT INTO cpms_status_exports
         (export_year, export_batch_id, exported_at, records_hash, citizen_binding_records_count, binding_release_records_count, export_file)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (export_year) DO UPDATE SET
           export_batch_id = EXCLUDED.export_batch_id,
           exported_at = EXCLUDED.exported_at,
           records_hash = EXCLUDED.records_hash,
           citizen_binding_records_count = EXCLUDED.citizen_binding_records_count,
           binding_release_records_count = EXCLUDED.binding_release_records_count,
           export_file = EXCLUDED.export_file",
    )
    .bind(export_year)
    .bind(&export_file.export_batch_id)
    .bind(export_file.exported_at)
    .bind(&export_file.records_hash)
    .bind(export_file.citizen_binding_records_count as i64)
    .bind(export_file.binding_release_records_count as i64)
    .bind(sqlx::types::Json(&export_file))
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "record annual status export failed"))?;

    Ok(export_file)
}

pub(crate) async fn status_export_state(
    state: &AppState,
) -> Result<CpmsStatusExportState, (StatusCode, Json<ApiError>)> {
    let now = Utc::now();
    resolve_status_export_state(state, now).await
}

pub(crate) async fn ensure_operator_annual_export_unlocked(
    state: &AppState,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let export_state = resolve_status_export_state(state, Utc::now()).await?;
    if !export_state.operator_lock_active {
        return Ok(());
    }
    Err(err(
        StatusCode::LOCKED,
        2010,
        "annual status export required",
    ))
}

async fn build_cpms_status_export(
    state: &AppState,
    export_year: i32,
    now: DateTime<Utc>,
) -> Result<CpmsStatusExportFile, (StatusCode, Json<ApiError>)> {
    let install = initialize::load_cpms_install_runtime(state).await?;
    let sign_key = active_archive_sign_key(state).await?;
    if sign_key.pubkey != install.cpms_pubkey {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "archive sign key does not match install cpms_pubkey",
        ));
    }

    let (start_ts, end_ts) = export_year_bounds(export_year)?;
    let citizen_binding_records = load_citizen_binding_records(state, now.timestamp()).await?;
    let binding_release_records = load_binding_release_records(state, start_ts, end_ts).await?;
    let exported_at = now.timestamp();
    let export_batch_id = format!("cse_{}", Uuid::new_v4().simple());
    let records_hash = records_hash(&citizen_binding_records, &binding_release_records)?;
    let sign_source = build_status_export_sign_source(
        &install.sfid_number,
        &sign_key.pubkey,
        &export_batch_id,
        exported_at,
        &records_hash,
    );
    let sig = sign_archive_payload_with_secret(&sign_key.secret_bytes, &sign_source)?;

    Ok(CpmsStatusExportFile {
        proto: "SFID_CPMS_V1".to_string(),
        r#type: "CPMS_STATUS_EXPORT".to_string(),
        version: 1,
        export_year,
        sfid_number: install.sfid_number,
        cpms_pubkey: sign_key.pubkey,
        export_batch_id,
        exported_at,
        citizen_binding_records_count: citizen_binding_records.len(),
        binding_release_records_count: binding_release_records.len(),
        records_hash,
        citizen_binding_records,
        binding_release_records,
        sig,
    })
}

async fn load_citizen_binding_records(
    state: &AppState,
    checked_at: i64,
) -> Result<Vec<CpmsCitizenBindingRecord>, (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT archive_no, status, birth_date::TEXT AS birth_date, citizen_status, voting_eligible,
                COALESCE(citizen_status_updated_at, updated_at) AS status_updated_at,
                wallet_address, wallet_pubkey, COALESCE(wallet_sig_alg, 'sr25519') AS wallet_sig_alg,
                COALESCE(wallet_bound_at, updated_at) AS wallet_bound_at
         FROM archives
         WHERE COALESCE(wallet_address, '') <> ''
           AND COALESCE(wallet_pubkey, '') <> ''
         ORDER BY archive_no",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query citizen binding export records failed",
        )
    })?;

    rows.into_iter()
        .map(|row| {
            let archive_status: String = row.get("status");
            let raw_citizen_status: String = row.get("citizen_status");
            let citizen_status = if archive_status == "DELETED" {
                CITIZEN_STATUS_REVOKED.to_string()
            } else {
                raw_citizen_status
            };
            super::validate_citizen_status(&citizen_status)?;
            let birth_date_text: String = row.get("birth_date");
            let birth_date =
                NaiveDate::parse_from_str(&birth_date_text, "%Y-%m-%d").map_err(|_| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "invalid birth_date",
                    )
                })?;
            let requested_voting = row.try_get::<bool, _>("voting_eligible").ok();
            let wallet_sig_alg: String = row.get("wallet_sig_alg");
            if wallet_sig_alg != "sr25519" {
                return Err(err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "archive wallet signature algorithm invalid",
                ));
            }
            Ok(CpmsCitizenBindingRecord {
                archive_no: row.get("archive_no"),
                wallet_address: row.get("wallet_address"),
                wallet_pubkey: row.get("wallet_pubkey"),
                wallet_sig_alg,
                wallet_bound_at: row.get("wallet_bound_at"),
                citizen_status: citizen_status.clone(),
                voting_eligible: effective_voting_eligible(
                    &citizen_status,
                    birth_date,
                    requested_voting,
                    true,
                    checked_at,
                ),
                status_updated_at: row.get("status_updated_at"),
            })
        })
        .collect()
}

async fn load_binding_release_records(
    state: &AppState,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<CpmsBindingReleaseRecord>, (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT archive_no, hard_deleted_at
         FROM archive_hard_delete_logs
         WHERE hard_deleted_at >= $1
           AND hard_deleted_at < $2
         ORDER BY hard_deleted_at, archive_no",
    )
    .bind(start_ts)
    .bind(end_ts)
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query binding release records failed",
        )
    })?;

    Ok(rows
        .into_iter()
        .map(|row| CpmsBindingReleaseRecord {
            archive_no: row.get("archive_no"),
            released_at: row.get("hard_deleted_at"),
            release_reason: "ARCHIVE_HARD_DELETED_AFTER_100_YEARS".to_string(),
        })
        .collect())
}

fn records_hash(
    citizen_binding_records: &[CpmsCitizenBindingRecord],
    binding_release_records: &[CpmsBindingReleaseRecord],
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let json = serde_json::to_vec(&ExportRecordsForHash {
        citizen_binding_records,
        binding_release_records,
    })
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "status export json failed",
        )
    })?;
    let digest = Blake2b256::digest(&json);
    Ok(format!("0x{}", hex::encode(digest)))
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

async fn resolve_status_export_state(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<CpmsStatusExportState, (StatusCode, Json<ApiError>)> {
    let first_export_year = load_first_export_year(state).await?;
    let latest_exportable_year = latest_exportable_year(now);
    let missing_export_year = match first_export_year {
        Some(first_year) if first_year <= latest_exportable_year => {
            let exported_years =
                load_exported_years(state, first_year, latest_exportable_year).await?;
            first_missing_export_year(first_year, latest_exportable_year, &exported_years)
        }
        _ => None,
    };
    let operator_lock_active = missing_export_year
        .map(|year| is_operator_lock_active(now, year))
        .unwrap_or(false);
    let exported = match first_export_year {
        Some(first_year) if first_year <= latest_exportable_year => missing_export_year.is_none(),
        _ => false,
    };
    let export_year = match (missing_export_year, exported, first_export_year) {
        (Some(year), _, _) => Some(year),
        (None, true, _) => Some(latest_exportable_year),
        (None, false, Some(first_year)) if first_year <= latest_exportable_year => {
            Some(latest_exportable_year)
        }
        _ => None,
    };
    let disabled_reason = if export_year.is_some() {
        None
    } else if first_export_year.is_none() {
        Some("system not initialized".to_string())
    } else {
        Some("annual status export not required".to_string())
    };

    Ok(CpmsStatusExportState {
        now_utc: now.timestamp(),
        pending_export_year: export_year,
        can_export: export_year.is_some(),
        reminder_active: missing_export_year.is_some(),
        operator_lock_active,
        exported,
        next_export_available_at: missing_export_year
            .is_none()
            .then(|| next_export_available_at(now))
            .transpose()?,
        disabled_reason,
    })
}

async fn resolve_export_year(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Option<i32>, (StatusCode, Json<ApiError>)> {
    Ok(resolve_status_export_state(state, now)
        .await?
        .pending_export_year)
}

async fn load_first_export_year(
    state: &AppState,
) -> Result<Option<i32>, (StatusCode, Json<ApiError>)> {
    let initialized_at: Option<i64> =
        sqlx::query_scalar("SELECT initialized_at FROM system_install WHERE id = 1")
            .fetch_one(&state.db)
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "query system install failed",
                )
            })?;

    initialized_at
        .map(|ts| {
            chrono::DateTime::<Utc>::from_timestamp(ts, 0)
                .map(|dt| dt.year())
                .ok_or_else(|| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "system initialized_at invalid",
                    )
                })
        })
        .transpose()
}

async fn load_exported_years(
    state: &AppState,
    start_year: i32,
    end_year: i32,
) -> Result<HashSet<i32>, (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT export_year FROM cpms_status_exports
         WHERE export_year >= $1 AND export_year <= $2
         ORDER BY export_year",
    )
    .bind(start_year)
    .bind(end_year)
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query annual status export failed",
        )
    })?;

    Ok(rows
        .into_iter()
        .map(|row| row.get::<i32, _>("export_year"))
        .collect())
}

fn latest_exportable_year(now: DateTime<Utc>) -> i32 {
    now.year() - 1
}

fn first_missing_export_year(
    first_year: i32,
    latest_year: i32,
    exported_years: &HashSet<i32>,
) -> Option<i32> {
    (first_year..=latest_year).find(|year| !exported_years.contains(year))
}

fn is_operator_lock_active(now: DateTime<Utc>, pending_export_year: i32) -> bool {
    // 中文注释：某年度报告在下一年 1 月 10 日后仍未导出时，操作员持续锁定直到补导完成。
    annual_export_lock_start_at(pending_export_year)
        .map(|lock_start| now.timestamp() >= lock_start)
        .unwrap_or(false)
}

fn annual_export_lock_start_at(export_year: i32) -> Result<i64, (StatusCode, Json<ApiError>)> {
    NaiveDate::from_ymd_opt(export_year + 1, 1, 11)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "annual export year invalid",
            )
        })
}

fn next_export_available_at(now: DateTime<Utc>) -> Result<i64, (StatusCode, Json<ApiError>)> {
    NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "annual export year invalid",
            )
        })
}

fn export_year_bounds(export_year: i32) -> Result<(i64, i64), (StatusCode, Json<ApiError>)> {
    let start = NaiveDate::from_ymd_opt(export_year, 1, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "annual export year invalid",
            )
        })?
        .and_utc()
        .timestamp();
    let end = NaiveDate::from_ymd_opt(export_year + 1, 1, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "annual export year invalid",
            )
        })?
        .and_utc()
        .timestamp();
    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::{
        build_status_export_sign_source, export_year_bounds, first_missing_export_year,
        is_operator_lock_active, latest_exportable_year, records_hash, CpmsBindingReleaseRecord,
        CpmsCitizenBindingRecord,
    };
    use chrono::{NaiveDate, Utc};
    use std::collections::HashSet;

    #[test]
    fn revoked_status_record_never_exports_voting_eligible() {
        let record = CpmsCitizenBindingRecord {
            archive_no: "ARCHIVE-1".to_string(),
            wallet_address: "addr2027".to_string(),
            wallet_pubkey: "0xabc".to_string(),
            wallet_sig_alg: "sr25519".to_string(),
            wallet_bound_at: 1,
            citizen_status: "REVOKED".to_string(),
            voting_eligible: false,
            status_updated_at: 1,
        };

        assert!(!record.voting_eligible);
    }

    #[test]
    fn export_records_hash_is_stable_for_same_ordered_records() {
        let binding_records = vec![CpmsCitizenBindingRecord {
            archive_no: "ARCHIVE-1".to_string(),
            wallet_address: "addr2027".to_string(),
            wallet_pubkey: "0xabc".to_string(),
            wallet_sig_alg: "sr25519".to_string(),
            wallet_bound_at: 1,
            citizen_status: "NORMAL".to_string(),
            voting_eligible: true,
            status_updated_at: 1,
        }];
        let release_records = vec![CpmsBindingReleaseRecord {
            archive_no: "ARCHIVE-OLD".to_string(),
            released_at: 2,
            release_reason: "ARCHIVE_HARD_DELETED_AFTER_100_YEARS".to_string(),
        }];

        let first_hash = match records_hash(&binding_records, &release_records) {
            Ok(hash) => hash,
            Err(_) => panic!("records hash"),
        };
        let second_hash = match records_hash(&binding_records, &release_records) {
            Ok(hash) => hash,
            Err(_) => panic!("records hash"),
        };
        assert_eq!(first_hash, second_hash);
    }

    #[test]
    fn status_export_sign_source_is_canonical() {
        assert_eq!(
            build_status_export_sign_source(
                "GD001-GZG0E-123456789-2026",
                "0xpub",
                "cse_1",
                9,
                "0xhash"
            ),
            "sfid-cpms-v1|cpms-status-export|GD001-GZG0E-123456789-2026|0xpub|cse_1|9|0xhash"
        );
    }

    #[test]
    fn annual_export_uses_previous_year_after_january_first() {
        let now = NaiveDate::from_ymd_opt(2027, 1, 10)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let year = latest_exportable_year(now);
        assert_eq!(year, 2026);
        let (start, end) = match export_year_bounds(year) {
            Ok(bounds) => bounds,
            Err(_) => panic!("bounds"),
        };
        assert_eq!(
            chrono::DateTime::<Utc>::from_timestamp(start, 0)
                .unwrap()
                .date_naive(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(
            chrono::DateTime::<Utc>::from_timestamp(end, 0)
                .unwrap()
                .date_naive(),
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
        );
    }

    #[test]
    fn first_missing_export_year_picks_earliest_gap() {
        let exported_years = HashSet::from([2025, 2027]);
        assert_eq!(
            first_missing_export_year(2025, 2028, &exported_years),
            Some(2026)
        );
        let all_exported = HashSet::from([2025, 2026]);
        assert_eq!(first_missing_export_year(2025, 2026, &all_exported), None);
    }

    #[test]
    fn operator_lock_starts_after_january_tenth_and_persists() {
        let jan_10 = NaiveDate::from_ymd_opt(2027, 1, 10)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();
        let jan_11 = NaiveDate::from_ymd_opt(2027, 1, 11)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let may_30 = NaiveDate::from_ymd_opt(2027, 5, 30)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        assert!(!is_operator_lock_active(jan_10, 2026));
        assert!(is_operator_lock_active(jan_11, 2026));
        assert!(is_operator_lock_active(may_30, 2026));
    }
}
