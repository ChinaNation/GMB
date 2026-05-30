//! # 超级管理员模块 (super_admin)
//!
//! 操作员 CRUD 仅 SUPER_ADMIN 角色可访问。
//! 中文注释：公民状态变更属于档案业务，允许 SUPER_ADMIN 与 OPERATOR_ADMIN。

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    authz, dangan, err, find_admin_by_pubkey, ok, write_audit, ApiError, ApiResponse, AppState,
    Archive,
};

#[derive(Deserialize)]
struct CreateOperatorRequest {
    admin_pubkey: String,
    #[serde(default)]
    admin_name: Option<String>,
}

#[derive(Serialize)]
struct OperatorData {
    user_id: String,
    admin_pubkey: String,
    admin_name: String,
    role: String,
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
        .route(
            "/api/v1/admin/operators",
            get(list_operators).post(create_operator),
        )
        .route("/api/v1/admin/operators/:id", delete(delete_operator))
        .route(
            "/api/v1/archives/:archive_id/citizen-status",
            put(update_archive_citizen_status),
        )
}

async fn list_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<OperatorData>>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;

    let rows = sqlx::query(
        "SELECT user_id, admin_pubkey, COALESCE(admin_name, '') AS admin_name, role
         FROM admin_users
         WHERE role = 'OPERATOR_ADMIN'
         ORDER BY user_id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query operators failed",
        )
    })?;

    let operators = rows
        .into_iter()
        .map(|r| OperatorData {
            user_id: r.get("user_id"),
            admin_pubkey: r.get("admin_pubkey"),
            admin_name: r.get("admin_name"),
            role: r.get("role"),
        })
        .collect::<Vec<OperatorData>>();

    Ok(Json(ok(operators)))
}

async fn create_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateOperatorRequest>,
) -> Result<Json<ApiResponse<OperatorData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let raw_input = req.admin_pubkey.trim().to_string();
    if raw_input.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid admin_pubkey"));
    }
    // 归一化公钥：支持 SS58 地址或 0x hex
    let admin_pubkey = {
        let stripped = raw_input
            .strip_prefix("0x")
            .or_else(|| raw_input.strip_prefix("0X"))
            .unwrap_or(&raw_input);
        if stripped.len() == 64 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
            stripped.to_lowercase()
        } else if let Some(hex_with_prefix) = crate::ss58::ss58_to_pubkey_hex(&raw_input) {
            hex_with_prefix
                .strip_prefix("0x")
                .unwrap_or(&hex_with_prefix)
                .to_lowercase()
        } else {
            return Err(err(
                StatusCode::BAD_REQUEST,
                1001,
                "admin_pubkey must be SS58 address or 32-byte hex",
            ));
        }
    };
    let admin_name = req.admin_name.as_deref().unwrap_or("").trim().to_string();

    if find_admin_by_pubkey(&state, &admin_pubkey).await.is_ok() {
        return Err(err(
            StatusCode::CONFLICT,
            3001,
            "admin_pubkey already exists",
        ));
    }

    let now_ts = Utc::now().timestamp();
    let operator = OperatorData {
        user_id: format!("u_operator_{}", Uuid::new_v4().simple()),
        admin_pubkey,
        admin_name,
        role: "OPERATOR_ADMIN".to_string(),
    };

    sqlx::query(
        "INSERT INTO admin_users (user_id, admin_pubkey, admin_name, role, status, immutable, managed_key_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, FALSE, NULL, $6, $7)",
    )
    .bind(&operator.user_id)
    .bind(&operator.admin_pubkey)
    .bind(&operator.admin_name)
    .bind(&operator.role)
    .bind("ACTIVE")
    .bind(now_ts)
    .bind(now_ts)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::CONFLICT, 3001, "admin_pubkey already exists"))?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "CREATE_OPERATOR",
        "ADMIN_USER",
        Some(operator.user_id.clone()),
        "SUCCESS",
        serde_json::json!({"role": operator.role}),
    )
    .await?;

    Ok(Json(ok(operator)))
}

async fn delete_operator(
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
        "SELECT user_id, admin_pubkey, COALESCE(admin_name, '') AS admin_name, role
         FROM admin_users
         WHERE user_id = $1
         FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query operator failed",
        )
    })?;

    let Some(row) = row else {
        return Err(err(StatusCode::NOT_FOUND, 3002, "operator not found"));
    };
    let role: String = row.get("role");
    if role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }
    let admin_pubkey: String = row.get("admin_pubkey");
    let admin_name: String = row.get("admin_name");

    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(&id)
        .execute(tx.as_mut())
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "delete operator sessions failed",
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
                "delete operator failed",
            )
        })?;

    // 中文注释：操作管理员物理删除后只靠审计快照追溯，不保留旧状态分支。
    sqlx::query(
        "INSERT INTO audit_logs (log_id, operator_user_id, action, target_type, target_id, result, detail, created_at)
         VALUES ($1, $2, 'DELETE_OPERATOR', 'ADMIN_USER', $3, 'SUCCESS', $4, $5)",
    )
    .bind(format!("log_{}", Uuid::new_v4().simple()))
    .bind(&ctx.user_id)
    .bind(&id)
    .bind(sqlx::types::Json(serde_json::json!({
        "deleted_user_id": id,
        "admin_pubkey": admin_pubkey,
        "admin_name": admin_name,
        "role": role
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

async fn update_archive_citizen_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<UpdateCitizenStatusRequest>,
) -> Result<Json<ApiResponse<UpdateCitizenStatusData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    dangan::validate_citizen_status(&req.citizen_status)?;

    let row = sqlx::query(
        "SELECT archive_id, archive_no, province_code, city_code, last_name, first_name, birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, valid_from, valid_until, citizen_status_updated_at, wallet_address, wallet_pubkey, COALESCE(wallet_sig_alg,'sr25519') AS wallet_sig_alg, wallet_bound_at, wallet_bound_by, COALESCE(archive_qr_payload,'') AS archive_qr_payload, deleted_at, deleted_by, delete_reason, created_at, updated_at
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
    archive.voting_eligible = dangan::normalize_voting_eligible(&archive.citizen_status, None);
    archive.citizen_status_updated_at = Utc::now().timestamp();
    archive.updated_at = archive.citizen_status_updated_at;
    archive.archive_qr_payload = if archive
        .wallet_address
        .as_deref()
        .is_some_and(|v| !v.is_empty())
        && archive
            .wallet_pubkey
            .as_deref()
            .is_some_and(|v| !v.is_empty())
    {
        let archive_qr = dangan::build_archive_qr_payload(&state, &archive).await?;
        serde_json::to_string(&archive_qr).map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "archive code encode failed",
            )
        })?
    } else {
        String::new()
    };

    sqlx::query("UPDATE archives SET citizen_status = $1, voting_eligible = $2, citizen_status_updated_at = $3, archive_qr_payload = $4, updated_at = $5 WHERE archive_id = $6")
        .bind(&archive.citizen_status)
        .bind(archive.voting_eligible)
        .bind(archive.citizen_status_updated_at)
        .bind(&archive.archive_qr_payload)
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
