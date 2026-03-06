use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    authz, dangan, err, find_admin_by_pubkey, ok, validate_admin_status, write_audit, ApiError,
    ApiResponse, AppState,
};

#[derive(Deserialize)]
struct CreateOperatorRequest {
    admin_pubkey: String,
}

#[derive(Deserialize)]
struct UpdateOperatorRequest {
    admin_pubkey: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct UpdateOperatorStatusRequest {
    status: String,
}

#[derive(Serialize)]
struct OperatorData {
    user_id: String,
    admin_pubkey: String,
    role: String,
    status: String,
}

#[derive(Serialize)]
struct SiteKeyRegistrationData {
    qr_payload: crate::dangan::SiteKeyRegistrationPayload,
    qr_content: String,
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
        .route(
            "/api/v1/admin/operators/:id",
            put(update_operator).delete(delete_operator),
        )
        .route(
            "/api/v1/admin/operators/:id/status",
            put(update_operator_status),
        )
        .route(
            "/api/v1/admin/site-keys/registration-qr",
            post(generate_site_key_registration_qr),
        )
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
        "SELECT user_id, admin_pubkey, role, status
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
            role: r.get("role"),
            status: r.get("status"),
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
    if req.admin_pubkey.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid admin_pubkey"));
    }

    if find_admin_by_pubkey(&state, req.admin_pubkey.trim())
        .await
        .is_ok()
    {
        return Err(err(
            StatusCode::CONFLICT,
            3001,
            "admin_pubkey already exists",
        ));
    }

    let now_ts = Utc::now().timestamp();
    let operator = OperatorData {
        user_id: format!("u_operator_{}", Uuid::new_v4().simple()),
        admin_pubkey: req.admin_pubkey.trim().to_string(),
        role: "OPERATOR_ADMIN".to_string(),
        status: "ACTIVE".to_string(),
    };

    sqlx::query(
        "INSERT INTO admin_users (user_id, admin_pubkey, role, status, immutable, managed_key_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, FALSE, NULL, $5, $6)",
    )
    .bind(&operator.user_id)
    .bind(&operator.admin_pubkey)
    .bind(&operator.role)
    .bind(&operator.status)
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

async fn update_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateOperatorRequest>,
) -> Result<Json<ApiResponse<OperatorData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;

    let row = sqlx::query(
        "SELECT user_id, admin_pubkey, role, status
         FROM admin_users
         WHERE user_id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query operator failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;

    let role: String = row.get("role");
    if role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }

    if let Some(ref admin_pubkey) = req.admin_pubkey {
        if admin_pubkey.trim().is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid admin_pubkey"));
        }
        let dup: Option<String> = sqlx::query_scalar(
            "SELECT user_id
             FROM admin_users
             WHERE admin_pubkey = $1 AND user_id <> $2
             LIMIT 1",
        )
        .bind(admin_pubkey.trim())
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "check pubkey failed",
            )
        })?;
        if dup.is_some() {
            return Err(err(
                StatusCode::CONFLICT,
                3001,
                "admin_pubkey already exists",
            ));
        }
    }

    let mut admin_pubkey: String = row.get("admin_pubkey");
    let mut status: String = row.get("status");

    if let Some(v) = req.admin_pubkey {
        admin_pubkey = v.trim().to_string();
    }
    if let Some(v) = req.status {
        validate_admin_status(&v)?;
        status = v;
    }

    sqlx::query(
        "UPDATE admin_users
         SET admin_pubkey = $1, status = $2, updated_at = $3
         WHERE user_id = $4",
    )
    .bind(&admin_pubkey)
    .bind(&status)
    .bind(Utc::now().timestamp())
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "update operator failed",
        )
    })?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_OPERATOR",
        "ADMIN_USER",
        Some(id.clone()),
        "SUCCESS",
        serde_json::json!({"status": status}),
    )
    .await?;

    Ok(Json(ok(OperatorData {
        user_id: id,
        admin_pubkey,
        role,
        status,
    })))
}

async fn delete_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;

    let role: Option<String> =
        sqlx::query_scalar("SELECT role FROM admin_users WHERE user_id = $1 LIMIT 1")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "query operator failed",
                )
            })?;

    let Some(role) = role else {
        return Err(err(StatusCode::NOT_FOUND, 3002, "operator not found"));
    };
    if role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }

    sqlx::query("DELETE FROM admin_users WHERE user_id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "delete operator failed",
            )
        })?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "DELETE_OPERATOR",
        "ADMIN_USER",
        Some(id),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    Ok(Json(ok(serde_json::json!({}))))
}

async fn update_operator_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateOperatorStatusRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    validate_admin_status(&req.status)?;

    let role: Option<String> =
        sqlx::query_scalar("SELECT role FROM admin_users WHERE user_id = $1 LIMIT 1")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "query operator failed",
                )
            })?;

    let Some(role) = role else {
        return Err(err(StatusCode::NOT_FOUND, 3002, "operator not found"));
    };
    if role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }

    sqlx::query("UPDATE admin_users SET status = $1, updated_at = $2 WHERE user_id = $3")
        .bind(&req.status)
        .bind(Utc::now().timestamp())
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "update status failed",
            )
        })?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_OPERATOR_STATUS",
        "ADMIN_USER",
        Some(id),
        "SUCCESS",
        serde_json::json!({"status": req.status}),
    )
    .await?;

    Ok(Json(ok(serde_json::json!({}))))
}

async fn generate_site_key_registration_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SiteKeyRegistrationData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let payload = dangan::build_site_key_registration_payload(&state).await?;
    let qr_content = serde_json::to_string(&payload)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "GENERATE_SITE_KEY_REGISTRATION_QR",
        "SITE_KEY_QR",
        Some(payload.qr_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "site_sfid": payload.site_sfid,
            "sign_key_id": payload.sign_key_id
        }),
    )
    .await?;

    Ok(Json(ok(SiteKeyRegistrationData {
        qr_payload: payload,
        qr_content,
    })))
}

async fn update_archive_citizen_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<UpdateCitizenStatusRequest>,
) -> Result<Json<ApiResponse<UpdateCitizenStatusData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    dangan::validate_citizen_status(&req.citizen_status)?;

    let row = sqlx::query(
        "SELECT archive_id, archive_no, citizen_status
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

    let before: String = row.get("citizen_status");
    let archive_no: String = row.get("archive_no");

    sqlx::query("UPDATE archives SET citizen_status = $1, updated_at = $2 WHERE archive_id = $3")
        .bind(req.citizen_status.trim())
        .bind(Utc::now().timestamp())
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
            "archive_no": archive_no,
            "before_citizen_status": before,
            "after_citizen_status": req.citizen_status,
            "voting_eligible": req.citizen_status == "NORMAL"
        }),
    )
    .await?;

    Ok(Json(ok(UpdateCitizenStatusData {
        archive_id,
        archive_no,
        citizen_status: req.citizen_status.clone(),
        voting_eligible: req.citizen_status == "NORMAL",
    })))
}
