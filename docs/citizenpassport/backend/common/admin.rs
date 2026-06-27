//! 管理员记录查询（跨模块共享 DB helper）。

use axum::{http::StatusCode, Json};
use sqlx::Row;

use super::response::{err, ApiError};
use super::types::AdminUser;
use crate::AppState;

pub(crate) async fn find_admin_by_pubkey(
    state: &AppState,
    admin_account: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    // 归一化：去 0x 前缀，小写
    let normalized = admin_account
        .trim()
        .strip_prefix("0x")
        .or_else(|| admin_account.trim().strip_prefix("0X"))
        .unwrap_or(admin_account.trim())
        .to_lowercase();
    let row = sqlx::query(
        "SELECT user_id, admin_account, COALESCE(admin_display_name, '') AS admin_display_name, user_group, immutable, managed_key_id, created_at, updated_at
         FROM admin_users
         WHERE admin_account = $1",
    )
    .bind(&normalized)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query admin failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 2002, "admin_account not found"))?;

    Ok(AdminUser {
        user_id: row.get("user_id"),
        admin_account: row.get("admin_account"),
        admin_display_name: row.get("admin_display_name"),
        user_group: row.get("user_group"),
        immutable: row.get("immutable"),
        managed_key_id: row.get("managed_key_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub(crate) async fn find_admin_by_user_id(
    state: &AppState,
    user_id: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT user_id, admin_account, COALESCE(admin_display_name, '') AS admin_display_name, user_group, immutable, managed_key_id, created_at, updated_at
         FROM admin_users
         WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query admin failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 2002, "admin user not found"))?;

    Ok(AdminUser {
        user_id: row.get("user_id"),
        admin_account: row.get("admin_account"),
        admin_display_name: row.get("admin_display_name"),
        user_group: row.get("user_group"),
        immutable: row.get("immutable"),
        managed_key_id: row.get("managed_key_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
