use axum::{http::StatusCode, Json};

use crate::{err, ApiError};

/// 读取当前 CPMS 实例的有效档案总量。
///
/// 中文注释：列表页不再实时 `COUNT(*)`，总量由档案创建/注销事务同步维护。
pub(super) async fn load_active_archive_count(
    db: &sqlx::PgPool,
) -> Result<i64, (StatusCode, Json<ApiError>)> {
    sqlx::query_scalar("SELECT active_count FROM archive_stats WHERE id = 1")
        .fetch_one(db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query archive stats failed",
            )
        })
}

/// 在档案创建/注销事务中同步调整统计表。
pub(crate) async fn adjust_archive_stats(
    conn: &mut sqlx::PgConnection,
    active_delta: i64,
    deleted_delta: i64,
    updated_at: i64,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    sqlx::query(
        "INSERT INTO archive_stats (id, active_count, deleted_count, updated_at)
         VALUES (1, 0, 0, $1)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(updated_at)
    .execute(&mut *conn)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "init archive stats failed",
        )
    })?;

    sqlx::query(
        "UPDATE archive_stats
         SET active_count = active_count + $1,
             deleted_count = deleted_count + $2,
             updated_at = $3
         WHERE id = 1",
    )
    .bind(active_delta)
    .bind(deleted_delta)
    .bind(updated_at)
    .execute(conn)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "update archive stats failed",
        )
    })?;
    Ok(())
}
