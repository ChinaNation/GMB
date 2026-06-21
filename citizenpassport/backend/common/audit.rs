//! 审计日志写入（跨模块共享 helper）。

use axum::{http::StatusCode, Json};
use chrono::Utc;
use uuid::Uuid;

use super::response::{err, ApiError};
use crate::AppState;

pub(crate) async fn write_audit(
    state: &AppState,
    operator_user_id: Option<String>,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    result: &str,
    detail: serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    sqlx::query(
        "INSERT INTO audit_logs (log_id, operator_user_id, action, target_type, target_id, result, detail, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(format!("log_{}", Uuid::new_v4().simple()))
    .bind(operator_user_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(result)
    .bind(sqlx::types::Json(detail))
    .bind(Utc::now().timestamp())
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "write audit failed"))?;
    Ok(())
}
