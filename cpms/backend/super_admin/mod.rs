//! # 超级管理员模块 (super_admin)
//!
//! 管理员管理仅 SUPER_ADMIN 角色可访问。
//! 中文注释：公民状态变更属于档案业务，允许 SUPER_ADMIN 与 OPERATOR_ADMIN。

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, put},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    authz,
    common::{err, ok, ss58, write_audit, ApiError, ApiResponse, Archive},
    dangan, AppState,
};

#[derive(Deserialize)]
struct CreateAdminRequest {
    role: String,
    admin_pubkey: String,
    admin_name: String,
}

#[derive(Serialize)]
struct AdminData {
    user_id: String,
    admin_pubkey: String,
    admin_address: String,
    admin_name: String,
    role: String,
    immutable: bool,
    can_edit_name: bool,
    can_delete: bool,
}

#[derive(Deserialize)]
struct UpdateAdminNameRequest {
    admin_name: String,
}

#[derive(Deserialize)]
struct UpdateCitizenStatusRequest {
    citizen_status: String,
}

#[derive(Serialize)]
struct UpdateCitizenStatusData {
    archive_id: String,
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/admins", get(list_admins).post(create_admin))
        .route(
            "/api/v1/admin/admins/:id",
            put(update_admin_name).delete(delete_admin),
        )
        .route(
            "/api/v1/archives/:archive_id/citizen-status",
            put(update_archive_citizen_status),
        )
}

async fn list_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<AdminData>>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;

    let rows = sqlx::query(
        "SELECT user_id, admin_pubkey, COALESCE(admin_name, '') AS admin_name,
                role, immutable
         FROM admin_users
         WHERE role IN ('SUPER_ADMIN', 'OPERATOR_ADMIN')
         ORDER BY
           CASE
             WHEN role = 'SUPER_ADMIN' AND immutable = TRUE THEN 0
             WHEN role = 'SUPER_ADMIN' THEN 1
             ELSE 2
           END,
           created_at,
           user_id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query admins failed",
        )
    })?;

    let admins = rows.into_iter().map(row_to_admin_data).collect::<Vec<_>>();

    Ok(Json(ok(admins)))
}

async fn create_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateAdminRequest>,
) -> Result<Json<ApiResponse<AdminData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let role = req.role.trim();
    if role != "SUPER_ADMIN" && role != "OPERATOR_ADMIN" {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid admin role"));
    }
    let admin_name = validate_admin_name(req.admin_name.as_str())?;
    let admin_pubkey = normalize_admin_pubkey(req.admin_pubkey.as_str())?;
    let now_ts = Utc::now().timestamp();
    let user_id = if role == "SUPER_ADMIN" {
        format!("u_super_{}", Uuid::new_v4().simple())
    } else {
        format!("u_operator_{}", Uuid::new_v4().simple())
    };

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    // 中文注释：超级管理员总数上限必须和插入共用锁，避免并发新增突破 5 个。
    sqlx::query("LOCK TABLE admin_users IN SHARE ROW EXCLUSIVE MODE")
        .execute(tx.as_mut())
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "lock admins failed",
            )
        })?;

    if role == "SUPER_ADMIN" {
        let super_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'SUPER_ADMIN'")
                .fetch_one(tx.as_mut())
                .await
                .map_err(|_| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "count super admins failed",
                    )
                })?;
        if super_count >= 5 {
            return Err(err(StatusCode::CONFLICT, 3001, "super admin limit reached"));
        }
    }

    let insert_result = sqlx::query(
        "INSERT INTO admin_users (user_id, admin_pubkey, admin_name, role, immutable, managed_key_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, FALSE, NULL, $5, $6)",
    )
    .bind(&user_id)
    .bind(&admin_pubkey)
    .bind(&admin_name)
    .bind(role)
    .bind(now_ts)
    .bind(now_ts)
    .execute(tx.as_mut())
    .await;
    if insert_result.is_err() {
        return Err(err(
            StatusCode::CONFLICT,
            3001,
            "admin_pubkey already exists",
        ));
    }

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    let admin = AdminData {
        user_id,
        admin_address: admin_address_for_pubkey(&admin_pubkey),
        admin_pubkey,
        admin_name,
        role: role.to_string(),
        immutable: false,
        can_edit_name: true,
        can_delete: true,
    };

    write_audit(
        &state,
        Some(ctx.user_id),
        "CREATE_ADMIN",
        "ADMIN_USER",
        Some(admin.user_id.clone()),
        "SUCCESS",
        serde_json::json!({"role": admin.role, "admin_name": admin.admin_name}),
    )
    .await?;

    Ok(Json(ok(admin)))
}

async fn update_admin_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateAdminNameRequest>,
) -> Result<Json<ApiResponse<AdminData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let admin_name = validate_admin_name(req.admin_name.as_str())?;
    let now = Utc::now().timestamp();
    let row = sqlx::query(
        "UPDATE admin_users
         SET admin_name = $1, updated_at = $2
         WHERE user_id = $3 AND role IN ('SUPER_ADMIN', 'OPERATOR_ADMIN')
         RETURNING user_id, admin_pubkey, COALESCE(admin_name, '') AS admin_name, role, immutable",
    )
    .bind(&admin_name)
    .bind(now)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "update admin failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "admin not found"))?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_ADMIN_NAME",
        "ADMIN_USER",
        Some(id),
        "SUCCESS",
        serde_json::json!({"admin_name": admin_name}),
    )
    .await?;

    Ok(Json(ok(row_to_admin_data(row))))
}

async fn delete_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let row = sqlx::query(
        "SELECT user_id, admin_pubkey, COALESCE(admin_name, '') AS admin_name, role, immutable
         FROM admin_users
         WHERE user_id = $1 AND role IN ('SUPER_ADMIN', 'OPERATOR_ADMIN')
         FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query admin failed",
        )
    })?;

    let Some(row) = row else {
        return Err(err(StatusCode::NOT_FOUND, 3002, "admin not found"));
    };
    let role: String = row.get("role");
    let immutable: bool = row.get("immutable");
    if immutable {
        return Err(err(
            StatusCode::CONFLICT,
            3003,
            "initial super admin cannot be deleted",
        ));
    }
    let admin_pubkey: String = row.get("admin_pubkey");
    let admin_address = admin_address_for_pubkey(&admin_pubkey);
    let admin_name: String = row.get("admin_name");

    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(&id)
        .execute(tx.as_mut())
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "delete admin sessions failed",
            )
        })?;
    sqlx::query("DELETE FROM admin_users WHERE user_id = $1")
        .bind(&id)
        .execute(tx.as_mut())
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "delete admin failed",
            )
        })?;

    // 中文注释：除初始超级管理员外，管理员物理删除后只靠审计快照追溯。
    sqlx::query(
        "INSERT INTO audit_logs (log_id, operator_user_id, action, target_type, target_id, result, detail, created_at)
         VALUES ($1, $2, 'DELETE_ADMIN', 'ADMIN_USER', $3, 'SUCCESS', $4, $5)",
    )
    .bind(format!("log_{}", Uuid::new_v4().simple()))
    .bind(&ctx.user_id)
    .bind(&id)
    .bind(sqlx::types::Json(serde_json::json!({
        "deleted_user_id": id,
        "admin_pubkey": admin_pubkey,
        "admin_address": admin_address,
        "admin_name": admin_name,
        "role": role,
        "immutable": immutable
    })))
    .bind(Utc::now().timestamp())
    .execute(tx.as_mut())
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "write audit failed"))?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    Ok(Json(ok(serde_json::json!({}))))
}

fn row_to_admin_data(row: sqlx::postgres::PgRow) -> AdminData {
    let admin_pubkey: String = row.get("admin_pubkey");
    let immutable: bool = row.get("immutable");
    AdminData {
        user_id: row.get("user_id"),
        admin_address: admin_address_for_pubkey(&admin_pubkey),
        admin_pubkey,
        admin_name: row.get("admin_name"),
        role: row.get("role"),
        immutable,
        can_edit_name: true,
        can_delete: !immutable,
    }
}

fn validate_admin_name(value: &str) -> Result<String, (StatusCode, Json<ApiError>)> {
    let name = value.trim();
    if name.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "admin name required"));
    }
    if name.chars().count() > 50 {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "admin name too long"));
    }
    Ok(name.to_string())
}

fn normalize_admin_pubkey(value: &str) -> Result<String, (StatusCode, Json<ApiError>)> {
    let raw_input = value.trim();
    if raw_input.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid admin_pubkey"));
    }
    let stripped = raw_input
        .strip_prefix("0x")
        .or_else(|| raw_input.strip_prefix("0X"))
        .unwrap_or(raw_input);
    if stripped.len() == 64 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(stripped.to_lowercase());
    }
    if let Some(hex_with_prefix) = ss58::ss58_to_pubkey_hex(raw_input) {
        return Ok(hex_with_prefix
            .strip_prefix("0x")
            .unwrap_or(&hex_with_prefix)
            .to_lowercase());
    }
    Err(err(
        StatusCode::BAD_REQUEST,
        1001,
        "admin_pubkey must be SS58 address or 32-byte hex",
    ))
}

fn admin_address_for_pubkey(admin_pubkey: &str) -> String {
    ss58::pubkey_hex_to_ss58(admin_pubkey).unwrap_or_else(|| admin_pubkey.to_string())
}

async fn update_archive_citizen_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<UpdateCitizenStatusRequest>,
) -> Result<Json<ApiResponse<UpdateCitizenStatusData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    dangan::validate_citizen_status(&req.citizen_status)?;

    let row = sqlx::query(
        "SELECT archive_id, archive_no, province_code, city_code, last_name, first_name, birth_date::TEXT AS birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, birth_province_code, birth_city_code, birth_town_code, COALESCE(election_scope_level,'PROVINCE') AS election_scope_level, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, valid_from::TEXT AS valid_from, valid_until::TEXT AS valid_until, citizen_status_updated_at, wallet_address, wallet_pubkey, COALESCE(wallet_sig_alg,'sr25519') AS wallet_sig_alg, wallet_bound_at, wallet_bound_by, COALESCE(archive_qr_payload,'') AS archive_qr_payload, deleted_at, deleted_by, delete_reason, created_at, updated_at
         FROM archives
         WHERE archive_id = $1",
    )
    .bind(&archive_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query archive failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?;

    let mut archive = Archive {
        archive_id: row.get("archive_id"),
        archive_no: row.get("archive_no"),
        province_code: row.get("province_code"),
        city_code: row.get("city_code"),
        last_name: row.get("last_name"),
        first_name: row.get("first_name"),
        birth_date: row.get("birth_date"),
        gender_code: row.get("gender_code"),
        height_cm: row.get("height_cm"),
        passport_no: row.get("passport_no"),
        town_code: row.try_get("town_code").unwrap_or_default(),
        village_id: row.try_get("village_id").unwrap_or_default(),
        address: row.try_get("address").unwrap_or_default(),
        birth_province_code: row.try_get("birth_province_code").unwrap_or_default(),
        birth_city_code: row.try_get("birth_city_code").unwrap_or_default(),
        birth_town_code: row.try_get("birth_town_code").unwrap_or_default(),
        election_scope_level: row
            .try_get("election_scope_level")
            .unwrap_or_else(|_| dangan::ELECTION_SCOPE_PROVINCE.to_string()),
        status: row.get("status"),
        citizen_status: row.get("citizen_status"),
        voting_eligible: row.try_get("voting_eligible").unwrap_or(true),
        valid_from: row.get("valid_from"),
        valid_until: row.get("valid_until"),
        citizen_status_updated_at: row.get("citizen_status_updated_at"),
        wallet_address: row.try_get("wallet_address").ok(),
        wallet_pubkey: row.try_get("wallet_pubkey").ok(),
        wallet_sig_alg: row
            .try_get("wallet_sig_alg")
            .unwrap_or_else(|_| "sr25519".to_string()),
        wallet_bound_at: row.try_get("wallet_bound_at").ok(),
        wallet_bound_by: row.try_get("wallet_bound_by").ok(),
        archive_qr_payload: row.try_get("archive_qr_payload").unwrap_or_default(),
        deleted_at: row.try_get("deleted_at").ok(),
        deleted_by: row.try_get("deleted_by").ok(),
        delete_reason: row.try_get("delete_reason").ok(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    if archive.status == "DELETED" || archive.deleted_at.is_some() {
        return Err(err(StatusCode::CONFLICT, 3008, "archive already deleted"));
    }

    let before = archive.citizen_status.clone();
    archive.citizen_status = req.citizen_status.trim().to_string();
    let now = Utc::now().timestamp();
    let birth_date = NaiveDate::parse_from_str(&archive.birth_date, "%Y-%m-%d")
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"))?;
    archive.voting_eligible =
        dangan::resolve_voting_eligible(&archive.citizen_status, birth_date, None, true, now)?;
    archive.citizen_status_updated_at = now;
    archive.updated_at = archive.citizen_status_updated_at;
    archive.archive_qr_payload = String::new();

    sqlx::query("UPDATE archives SET citizen_status = $1, voting_eligible = $2, citizen_status_updated_at = $3, updated_at = $4 WHERE archive_id = $5")
        .bind(&archive.citizen_status)
        .bind(archive.voting_eligible)
        .bind(archive.citizen_status_updated_at)
        .bind(archive.updated_at)
        .bind(&archive_id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "update archive failed",
            )
        })?;
    dangan::clear_archive_qr_payload(&state, &archive_id, archive.updated_at).await?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_ARCHIVE_CITIZEN_STATUS",
        "CITIZEN_ARCHIVE",
        Some(archive_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_no": archive.archive_no.clone(),
            "valid_from": archive.valid_from.clone(),
            "valid_until": archive.valid_until.clone(),
            "before_citizen_status": before,
            "after_citizen_status": archive.citizen_status.clone(),
            "voting_eligible": archive.voting_eligible
        }),
    )
    .await?;

    Ok(Json(ok(UpdateCitizenStatusData {
        archive_id,
        archive_no: archive.archive_no,
        citizen_status: archive.citizen_status,
        voting_eligible: archive.voting_eligible,
    })))
}

#[cfg(test)]
mod tests {
    use super::{normalize_admin_pubkey, validate_admin_name};

    #[test]
    fn validate_admin_name_requires_trimmed_name() {
        assert!(validate_admin_name(" 张三 ").is_ok());
        assert!(validate_admin_name("   ").is_err());
    }

    #[test]
    fn validate_admin_name_rejects_over_50_chars() {
        let long_name = "名".repeat(51);
        assert!(validate_admin_name(&long_name).is_err());
    }

    #[test]
    fn normalize_admin_pubkey_accepts_32_byte_hex() {
        let hex = format!("0x{}", "AB".repeat(32));
        let normalized = match normalize_admin_pubkey(&hex) {
            Ok(value) => value,
            Err(_) => panic!("hex pubkey should normalize"),
        };
        assert_eq!(normalized, "ab".repeat(32));
    }
}
