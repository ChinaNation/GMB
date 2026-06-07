use chrono::{DateTime, Months, NaiveTime, Utc};
use sqlx::Row;
use uuid::Uuid;

const HARD_DELETE_MONTHS: u32 = 100 * 12;

/// 执行已到期注销档案的硬删除。
///
/// 中文注释：详情页删除按钮对应“注销软删除”，`deleted_at` 是 100 年计时起点。
/// 到期后只释放档案号与护照号这一对号码；实名档案行物理删除，最小审计记录不保存实名原文。
pub(crate) async fn run_due_archive_hard_delete(db: &sqlx::PgPool) -> Result<u64, String> {
    let now_ts = Utc::now().timestamp();
    let cutoff_ts = hard_delete_cutoff_ts(now_ts)?;
    let mut deleted_count = 0;

    loop {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| format!("begin archive hard delete tx failed: {e}"))?;

        let row = sqlx::query(
            "SELECT archive_id, archive_no, passport_no, deleted_at
             FROM archives
             WHERE status = 'DELETED'
               AND deleted_at IS NOT NULL
               AND deleted_at <= $1
             ORDER BY deleted_at, archive_id
             LIMIT 1
             FOR UPDATE SKIP LOCKED",
        )
        .bind(cutoff_ts)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(|e| format!("query due archive failed: {e}"))?;

        let Some(row) = row else {
            tx.commit()
                .await
                .map_err(|e| format!("commit empty archive hard delete tx failed: {e}"))?;
            break;
        };

        let archive_id: String = row.get("archive_id");
        let archive_no: String = row.get("archive_no");
        let passport_no: String = row.get("passport_no");
        let deleted_at: i64 = row.get("deleted_at");

        let pool_result = sqlx::query(
            "INSERT INTO archive_number_recycle_pool
             (pool_id, archive_no, passport_no, source_archive_id, deleted_at, released_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (source_archive_id) DO NOTHING",
        )
        .bind(format!("anp_{}", Uuid::new_v4().simple()))
        .bind(&archive_no)
        .bind(&passport_no)
        .bind(&archive_id)
        .bind(deleted_at)
        .bind(now_ts)
        .execute(tx.as_mut())
        .await
        .map_err(|e| format!("insert archive number recycle pool failed: {e}"))?;
        if pool_result.rows_affected() != 1 {
            return Err(format!(
                "archive number recycle pool already exists for archive {archive_id}"
            ));
        }

        let log_result = sqlx::query(
            "INSERT INTO archive_hard_delete_logs
             (hard_delete_id, source_archive_id, archive_no, passport_no, deleted_at, hard_deleted_at, reason)
             VALUES ($1, $2, $3, $4, $5, $6, 'deleted archive reached 100 years')
             ON CONFLICT (source_archive_id) DO NOTHING",
        )
        .bind(format!("ahd_{}", Uuid::new_v4().simple()))
        .bind(&archive_id)
        .bind(&archive_no)
        .bind(&passport_no)
        .bind(deleted_at)
        .bind(now_ts)
        .execute(tx.as_mut())
        .await
        .map_err(|e| format!("insert archive hard delete log failed: {e}"))?;
        if log_result.rows_affected() != 1 {
            return Err(format!(
                "archive hard delete log already exists for archive {archive_id}"
            ));
        }

        sqlx::query("DELETE FROM archive_delete_challenges WHERE archive_id = $1")
            .bind(&archive_id)
            .execute(tx.as_mut())
            .await
            .map_err(|e| format!("delete archive challenge residue failed: {e}"))?;

        sqlx::query("DELETE FROM qr_print_records WHERE archive_id = $1")
            .bind(&archive_id)
            .execute(tx.as_mut())
            .await
            .map_err(|e| format!("delete archive print residue failed: {e}"))?;

        sqlx::query("DELETE FROM archives WHERE archive_id = $1 AND status = 'DELETED'")
            .bind(&archive_id)
            .execute(tx.as_mut())
            .await
            .map_err(|e| format!("hard delete archive failed: {e}"))?;

        super::remove_archive_material_files(&archive_id).await?;

        tx.commit()
            .await
            .map_err(|e| format!("commit archive hard delete tx failed: {e}"))?;
        deleted_count += 1;
    }

    Ok(deleted_count)
}

fn hard_delete_cutoff_ts(now_ts: i64) -> Result<i64, String> {
    let now = DateTime::<Utc>::from_timestamp(now_ts, 0)
        .ok_or_else(|| "invalid current timestamp".to_string())?;
    let cutoff_date = now
        .date_naive()
        .checked_sub_months(Months::new(HARD_DELETE_MONTHS))
        .ok_or_else(|| "archive hard delete cutoff overflow".to_string())?;
    let end_of_day =
        NaiveTime::from_hms_opt(23, 59, 59).ok_or_else(|| "invalid cutoff time".to_string())?;
    Ok(cutoff_date.and_time(end_of_day).and_utc().timestamp())
}

#[cfg(test)]
mod tests {
    use super::{hard_delete_cutoff_ts, run_due_archive_hard_delete};
    use chrono::{Months, Utc};
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[test]
    fn hard_delete_cutoff_uses_deleted_calendar_date_after_100_years() {
        let now = chrono::NaiveDate::from_ymd_opt(2126, 5, 30)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let cutoff = hard_delete_cutoff_ts(now).expect("cutoff");
        let cutoff_date = chrono::DateTime::from_timestamp(cutoff, 0)
            .unwrap()
            .date_naive();

        assert_eq!(
            cutoff_date,
            chrono::NaiveDate::from_ymd_opt(2026, 5, 30).unwrap()
        );
    }

    #[tokio::test]
    async fn db_hard_delete_skips_archive_before_100_years() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let case_id = format!("test_not_due_{}", Uuid::new_v4().simple());
        let archive_id = format!("arc_{case_id}");
        let archive_no = format!("AN-{case_id}");
        let passport_no = format!("PP{case_id}");
        let deleted_at = months_ago_ts(99 * 12);

        cleanup_case(&pool, &case_id).await;
        insert_deleted_archive(&pool, &archive_id, &archive_no, &passport_no, deleted_at).await;

        let deleted_count = run_due_archive_hard_delete(&pool)
            .await
            .expect("run hard delete");

        assert_eq!(deleted_count, 0);
        let archive_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM archives WHERE archive_id = $1)")
                .bind(&archive_id)
                .fetch_one(&pool)
                .await
                .expect("archive exists");
        let pool_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM archive_number_recycle_pool WHERE source_archive_id = $1)",
        )
        .bind(&archive_id)
        .fetch_one(&pool)
        .await
        .expect("pool exists");

        assert!(archive_exists);
        assert!(!pool_exists);
        cleanup_case(&pool, &case_id).await;
    }

    #[tokio::test]
    async fn db_hard_delete_recycles_numbers_after_100_years() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let case_id = format!("test_due_{}", Uuid::new_v4().simple());
        let archive_id = format!("arc_{case_id}");
        let archive_no = format!("AN-{case_id}");
        let passport_no = format!("PP{case_id}");
        let deleted_at = months_ago_ts(101 * 12);

        cleanup_case(&pool, &case_id).await;
        insert_deleted_archive(&pool, &archive_id, &archive_no, &passport_no, deleted_at).await;

        let deleted_count = run_due_archive_hard_delete(&pool)
            .await
            .expect("run hard delete");

        assert_eq!(deleted_count, 1);
        let archive_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM archives WHERE archive_id = $1)")
                .bind(&archive_id)
                .fetch_one(&pool)
                .await
                .expect("archive exists");
        let recycled: Option<(String, String)> = sqlx::query_as(
            "SELECT archive_no, passport_no
             FROM archive_number_recycle_pool
             WHERE source_archive_id = $1",
        )
        .bind(&archive_id)
        .fetch_optional(&pool)
        .await
        .expect("recycled row");
        let logged: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM archive_hard_delete_logs WHERE source_archive_id = $1)",
        )
        .bind(&archive_id)
        .fetch_one(&pool)
        .await
        .expect("hard delete log exists");

        assert!(!archive_exists);
        assert_eq!(recycled, Some((archive_no.clone(), passport_no.clone())));
        assert!(logged);
        cleanup_case(&pool, &case_id).await;
    }

    async fn test_pool() -> Option<sqlx::PgPool> {
        let Ok(database_url) = std::env::var("CPMS_TEST_DATABASE_URL") else {
            return None;
        };
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .connect(&database_url)
            .await
            .expect("connect CPMS_TEST_DATABASE_URL");
        sqlx::raw_sql(include_str!("../db/schema.sql"))
            .execute(&pool)
            .await
            .expect("apply schema");
        Some(pool)
    }

    async fn insert_deleted_archive(
        pool: &sqlx::PgPool,
        archive_id: &str,
        archive_no: &str,
        passport_no: &str,
        deleted_at: i64,
    ) {
        sqlx::query(
            "INSERT INTO archives
             (archive_id, archive_no, province_code, city_code, last_name, first_name, birth_date,
              gender_code, height_cm, passport_no, town_code, village_id, address, status,
              citizen_status, voting_eligible, valid_from, valid_until, citizen_status_updated_at,
              archive_qr_payload, deleted_at, deleted_by, delete_reason, created_at, updated_at)
             VALUES
             ($1, $2, 'GD', '001', 'Test', 'Citizen', '2000-01-01',
              'M', 170, $3, 'T001', 'V001', 'test address', 'DELETED',
              'REVOKED', FALSE, '2026-01-01', '2036-01-01', 0,
              '{}', $4, 'admin_test', 'test delete', $5, $5)",
        )
        .bind(archive_id)
        .bind(archive_no)
        .bind(passport_no)
        .bind(deleted_at)
        .bind(Utc::now().timestamp())
        .execute(pool)
        .await
        .expect("insert deleted archive");
    }

    async fn cleanup_case(pool: &sqlx::PgPool, case_id: &str) {
        let archive_id = format!("arc_{case_id}");
        sqlx::query("DELETE FROM archive_number_recycle_pool WHERE source_archive_id = $1 OR archive_no LIKE $2 OR passport_no LIKE $3")
            .bind(&archive_id)
            .bind(format!("%{case_id}%"))
            .bind(format!("%{case_id}%"))
            .execute(pool)
            .await
            .expect("cleanup recycle pool");
        sqlx::query("DELETE FROM archive_hard_delete_logs WHERE source_archive_id = $1 OR archive_no LIKE $2 OR passport_no LIKE $3")
            .bind(&archive_id)
            .bind(format!("%{case_id}%"))
            .bind(format!("%{case_id}%"))
            .execute(pool)
            .await
            .expect("cleanup hard delete logs");
        sqlx::query("DELETE FROM archives WHERE archive_id = $1 OR archive_no LIKE $2 OR passport_no LIKE $3")
            .bind(&archive_id)
            .bind(format!("%{case_id}%"))
            .bind(format!("%{case_id}%"))
            .execute(pool)
            .await
            .expect("cleanup archives");
    }

    fn months_ago_ts(months: u32) -> i64 {
        Utc::now()
            .date_naive()
            .checked_sub_months(Months::new(months))
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    }
}
