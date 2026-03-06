use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    authz, dangan, err, find_admin_by_pubkey, ok, validate_admin_status, write_audit, AdminUser,
    ApiError, ApiResponse, AppState,
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

    let users = state.admin_users.read().await;
    let operators = users
        .values()
        .filter(|u| u.role == "OPERATOR_ADMIN")
        .map(|u| OperatorData {
            user_id: u.user_id.clone(),
            admin_pubkey: u.admin_pubkey.clone(),
            role: u.role.clone(),
            status: u.status.clone(),
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

    if find_admin_by_pubkey(&state, &req.admin_pubkey)
        .await
        .is_ok()
    {
        return Err(err(
            StatusCode::CONFLICT,
            3001,
            "admin_pubkey already exists",
        ));
    }

    let operator = AdminUser {
        user_id: format!("u_operator_{}", Uuid::new_v4().simple()),
        admin_pubkey: req.admin_pubkey,
        role: "OPERATOR_ADMIN".to_string(),
        status: "ACTIVE".to_string(),
        immutable: false,
    };
    state
        .admin_users
        .write()
        .await
        .insert(operator.user_id.clone(), operator.clone());

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

    Ok(Json(ok(OperatorData {
        user_id: operator.user_id,
        admin_pubkey: operator.admin_pubkey,
        role: operator.role,
        status: operator.status,
    })))
}

async fn update_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateOperatorRequest>,
) -> Result<Json<ApiResponse<OperatorData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let mut users = state.admin_users.write().await;
    let current = users
        .get(&id)
        .cloned()
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;
    if current.role != "OPERATOR_ADMIN" {
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
        let duplicated = users
            .values()
            .any(|u| u.user_id != current.user_id && u.admin_pubkey == *admin_pubkey);
        if duplicated {
            return Err(err(
                StatusCode::CONFLICT,
                3001,
                "admin_pubkey already exists",
            ));
        }
    }

    let operator = users
        .get_mut(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;

    if let Some(admin_pubkey) = req.admin_pubkey {
        operator.admin_pubkey = admin_pubkey;
    }

    if let Some(status) = req.status {
        validate_admin_status(&status)?;
        operator.status = status;
    }

    let updated = operator.clone();
    drop(users);

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_OPERATOR",
        "ADMIN_USER",
        Some(updated.user_id.clone()),
        "SUCCESS",
        serde_json::json!({"status": updated.status}),
    )
    .await?;

    Ok(Json(ok(OperatorData {
        user_id: updated.user_id,
        admin_pubkey: updated.admin_pubkey,
        role: updated.role,
        status: updated.status,
    })))
}

async fn delete_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let mut users = state.admin_users.write().await;
    let user = users
        .get(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;
    if user.role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }
    users.remove(&id);
    drop(users);

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

    let mut users = state.admin_users.write().await;
    let operator = users
        .get_mut(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;
    if operator.role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }

    operator.status = req.status.clone();
    drop(users);

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

    let mut archives = state.archives.write().await;
    let archive = archives
        .get_mut(&archive_id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?;
    let before = archive.citizen_status.clone();
    archive.citizen_status = req.citizen_status.clone();
    let updated = archive.clone();
    drop(archives);

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_ARCHIVE_CITIZEN_STATUS",
        "CITIZEN_ARCHIVE",
        Some(updated.archive_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_no": updated.archive_no,
            "before_citizen_status": before,
            "after_citizen_status": updated.citizen_status,
            "voting_eligible": updated.citizen_status == "NORMAL"
        }),
    )
    .await?;

    Ok(Json(ok(UpdateCitizenStatusData {
        archive_id: updated.archive_id,
        archive_no: updated.archive_no,
        citizen_status: updated.citizen_status.clone(),
        voting_eligible: updated.citizen_status == "NORMAL",
    })))
}
