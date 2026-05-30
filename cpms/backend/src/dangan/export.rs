use axum::{http::StatusCode, Json};
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::Serialize;
use sqlx::Row;
use uuid::Uuid;

use crate::{err, initialize, ApiError, AppState};

use super::{
    active_archive_sign_key, normalize_voting_eligible, sign_archive_payload_with_secret,
    CITIZEN_STATUS_REVOKED,
};

type Blake2b256 = Blake2b<U32>;

#[derive(Clone, Serialize)]
pub(crate) struct CpmsStatusExportFile {
    proto: String,
    r#type: String,
    version: u32,
    export_year: i32,
    sfid_number: String,
    cpms_pubkey: String,
    export_batch_id: String,
    pub(crate) exported_at: i64,
    status_records_count: usize,
    number_release_records_count: usize,
    records_hash: String,
    status_records: Vec<CpmsStatusRecord>,
    number_release_records: Vec<CpmsNumberReleaseRecord>,
    sig: String,
}

#[derive(Clone, Serialize)]
struct CpmsStatusRecord {
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
    status_updated_at: i64,
}

#[derive(Clone, Serialize)]
struct CpmsNumberReleaseRecord {
    archive_no: String,
    passport_no: String,
    hard_deleted_at: i64,
}

#[derive(Serialize)]
struct ExportRecordsForHash<'a> {
    status_records: &'a [CpmsStatusRecord],
    number_release_records: &'a [CpmsNumberReleaseRecord],
}

/// 构造 CPMS 离线年度状态导出文件。
///
/// 中文注释：导出文件只给 SFID 更新状态和号码释放事实，不包含姓名、出生日期、地址、
/// 钱包地址等实名或绑定细节；CPMS 仍保持永不联网，由管理员手工导出文件。
pub(crate) async fn build_and_record_cpms_status_export(
    state: &AppState,
) -> Result<CpmsStatusExportFile, (StatusCode, Json<ApiError>)> {
    let now = Utc::now();
    let export_year = current_export_year(now)?;
    let export_file = build_cpms_status_export(state, export_year, now).await?;

    sqlx::query(
        "INSERT INTO cpms_status_exports
         (export_year, export_batch_id, exported_at, records_hash, status_records_count, number_release_records_count)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (export_year) DO UPDATE SET
           export_batch_id = EXCLUDED.export_batch_id,
           exported_at = EXCLUDED.exported_at,
           records_hash = EXCLUDED.records_hash,
           status_records_count = EXCLUDED.status_records_count,
           number_release_records_count = EXCLUDED.number_release_records_count",
    )
    .bind(export_year)
    .bind(&export_file.export_batch_id)
    .bind(export_file.exported_at)
    .bind(&export_file.records_hash)
    .bind(export_file.status_records_count as i64)
    .bind(export_file.number_release_records_count as i64)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "record annual status export failed"))?;

    Ok(export_file)
}

pub(crate) async fn ensure_operator_annual_export_unlocked(
    state: &AppState,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let Some(export_year) = operator_lock_export_year(Utc::now()) else {
        return Ok(());
    };
    if annual_export_exists(state, export_year).await? {
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
    let status_records = load_status_records(state, start_ts, end_ts).await?;
    let number_release_records = load_number_release_records(state, start_ts, end_ts).await?;
    let exported_at = now.timestamp();
    let export_batch_id = format!("cse_{}", Uuid::new_v4().simple());
    let records_hash = records_hash(&status_records, &number_release_records)?;
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
        status_records_count: status_records.len(),
        number_release_records_count: number_release_records.len(),
        records_hash,
        status_records,
        number_release_records,
        sig,
    })
}

async fn load_status_records(
    state: &AppState,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<CpmsStatusRecord>, (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT archive_no, status, citizen_status, voting_eligible, citizen_status_updated_at, updated_at
         FROM archives
         WHERE citizen_status_updated_at >= $1
           AND citizen_status_updated_at < $2
         ORDER BY archive_no",
    )
    .bind(start_ts)
    .bind(end_ts)
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query status export records failed",
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
            let requested_voting = row.try_get::<bool, _>("voting_eligible").ok();
            Ok(CpmsStatusRecord {
                archive_no: row.get("archive_no"),
                citizen_status: citizen_status.clone(),
                voting_eligible: normalize_voting_eligible(&citizen_status, requested_voting),
                status_updated_at: row
                    .try_get("citizen_status_updated_at")
                    .unwrap_or_else(|_| row.get("updated_at")),
            })
        })
        .collect()
}

async fn load_number_release_records(
    state: &AppState,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<CpmsNumberReleaseRecord>, (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT archive_no, passport_no, hard_deleted_at
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
            "query number release records failed",
        )
    })?;

    Ok(rows
        .into_iter()
        .map(|row| CpmsNumberReleaseRecord {
            archive_no: row.get("archive_no"),
            passport_no: row.get("passport_no"),
            hard_deleted_at: row.get("hard_deleted_at"),
        })
        .collect())
}

async fn annual_export_exists(
    state: &AppState,
    export_year: i32,
) -> Result<bool, (StatusCode, Json<ApiError>)> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM cpms_status_exports WHERE export_year = $1)")
        .bind(export_year)
        .fetch_one(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query annual status export failed",
            )
        })
}

fn records_hash(
    status_records: &[CpmsStatusRecord],
    number_release_records: &[CpmsNumberReleaseRecord],
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let json = serde_json::to_vec(&ExportRecordsForHash {
        status_records,
        number_release_records,
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

fn current_export_year(now: DateTime<Utc>) -> Result<i32, (StatusCode, Json<ApiError>)> {
    if now.month() == 1 && (1..=10).contains(&now.day()) {
        Ok(now.year() - 1)
    } else {
        Err(err(
            StatusCode::CONFLICT,
            3017,
            "annual status export window closed",
        ))
    }
}

fn operator_lock_export_year(now: DateTime<Utc>) -> Option<i32> {
    // 中文注释：1 月 6 日到 1 月 10 日仍未导出上一年度报告时，只锁操作管理员。
    (now.month() == 1 && (6..=10).contains(&now.day())).then(|| now.year() - 1)
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
        build_status_export_sign_source, current_export_year, export_year_bounds,
        operator_lock_export_year, records_hash, CpmsNumberReleaseRecord, CpmsStatusRecord,
    };
    use chrono::{NaiveDate, Utc};

    #[test]
    fn revoked_status_record_never_exports_voting_eligible() {
        let record = CpmsStatusRecord {
            archive_no: "ARCHIVE-1".to_string(),
            citizen_status: "REVOKED".to_string(),
            voting_eligible: false,
            status_updated_at: 1,
        };

        assert!(!record.voting_eligible);
    }

    #[test]
    fn export_records_hash_is_stable_for_same_ordered_records() {
        let status_records = vec![CpmsStatusRecord {
            archive_no: "ARCHIVE-1".to_string(),
            citizen_status: "NORMAL".to_string(),
            voting_eligible: true,
            status_updated_at: 1,
        }];
        let release_records = vec![CpmsNumberReleaseRecord {
            archive_no: "ARCHIVE-OLD".to_string(),
            passport_no: "GD000000001".to_string(),
            hard_deleted_at: 2,
        }];

        let first_hash = match records_hash(&status_records, &release_records) {
            Ok(hash) => hash,
            Err(_) => panic!("records hash"),
        };
        let second_hash = match records_hash(&status_records, &release_records) {
            Ok(hash) => hash,
            Err(_) => panic!("records hash"),
        };
        assert_eq!(first_hash, second_hash);
    }

    #[test]
    fn status_export_sign_source_is_canonical() {
        assert_eq!(
            build_status_export_sign_source(
                "GFR-GD001-ZG0X-123456789-2026",
                "0xpub",
                "cse_1",
                9,
                "0xhash"
            ),
            "sfid-cpms-v1|cpms-status-export|GFR-GD001-ZG0X-123456789-2026|0xpub|cse_1|9|0xhash"
        );
    }

    #[test]
    fn annual_export_window_uses_previous_year() {
        let now = NaiveDate::from_ymd_opt(2027, 1, 10)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let year = match current_export_year(now) {
            Ok(year) => year,
            Err(_) => panic!("export year"),
        };
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
    fn operator_lock_starts_after_january_fifth() {
        let jan_5 = NaiveDate::from_ymd_opt(2027, 1, 5)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();
        let jan_6 = NaiveDate::from_ymd_opt(2027, 1, 6)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        assert_eq!(operator_lock_export_year(jan_5), None);
        assert_eq!(operator_lock_export_year(jan_6), Some(2026));
    }
}
