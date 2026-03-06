use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    authz, dangan, err, find_admin_by_user_id, ok, write_audit, ApiError, ApiResponse, AppState,
    Archive, QrPrintRecord,
};

#[derive(Deserialize)]
struct CreateArchiveRequest {
    province_code: String,
    city_code: String,
    full_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    passport_no: String,
    citizen_status: Option<String>,
}

#[derive(Serialize)]
struct CreateArchiveData {
    archive_id: String,
    archive_no: String,
    status: String,
    citizen_status: String,
}

#[derive(Deserialize)]
struct ListQuery {
    full_name: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Serialize)]
struct QrGenerateData {
    qr_payload: crate::dangan::QrPayload,
    qr_content: String,
}

#[derive(Serialize)]
struct QrPrintData {
    print_id: String,
    archive_id: String,
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
    printed_at: i64,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/archives", post(create_archive).get(list_archives))
        .route("/api/v1/archives/:archive_id", get(get_archive))
        .route(
            "/api/v1/archives/:archive_id/qr/generate",
            post(generate_archive_qr),
        )
        .route(
            "/api/v1/archives/:archive_id/qr/print",
            post(print_archive_qr),
        )
}

async fn create_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateArchiveRequest>,
) -> Result<Json<ApiResponse<CreateArchiveData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "OPERATOR_ADMIN").await?;
    let admin = find_admin_by_user_id(&state, &ctx.user_id).await?;

    if !crate::dangan::province_codes::is_valid_province_code(&req.province_code) {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid province_code"));
    }
    if !crate::dangan::province_codes::is_valid_city_code_for_province(
        &req.province_code,
        &req.city_code,
    ) {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid city_code"));
    }
    if req.gender_code != "M" && req.gender_code != "W" {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid gender_code"));
    }
    let _birth_date = NaiveDate::parse_from_str(&req.birth_date, "%Y-%m-%d")
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"))?;
    let terminal_id = headers
        .get("x-terminal-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("terminal-000");
    let citizen_status = req.citizen_status.unwrap_or_else(|| "NORMAL".to_string());
    dangan::validate_citizen_status(&citizen_status)?;
    let created_date_yyyymmdd = Utc::now().format("%Y%m%d").to_string();

    let archive_no = dangan::generate_archive_no_with_retry(
        &state,
        &req.province_code,
        &req.city_code,
        &created_date_yyyymmdd,
        terminal_id,
        &admin.admin_pubkey,
    )
    .await?;

    let archive = Archive {
        archive_id: format!("ar_{}", Uuid::new_v4().simple()),
        archive_no: archive_no.clone(),
        province_code: req.province_code,
        city_code: req.city_code,
        full_name: req.full_name,
        birth_date: req.birth_date,
        gender_code: req.gender_code,
        height_cm: req.height_cm,
        passport_no: req.passport_no,
        status: "ACTIVE".to_string(),
        citizen_status,
    };

    state
        .archives
        .write()
        .await
        .insert(archive.archive_id.clone(), archive.clone());

    write_audit(
        &state,
        Some(ctx.user_id),
        "CREATE_ARCHIVE",
        "CITIZEN_ARCHIVE",
        Some(archive.archive_id.clone()),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    Ok(Json(ok(CreateArchiveData {
        archive_id: archive.archive_id,
        archive_no,
        status: archive.status,
        citizen_status: archive.citizen_status,
    })))
}

async fn list_archives(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "OPERATOR_ADMIN").await?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let archives = state.archives.read().await;
    let mut items: Vec<Archive> = archives.values().cloned().collect();

    if let Some(name) = query.full_name {
        items.retain(|a| a.full_name.contains(&name));
    }

    items.sort_by(|a, b| a.archive_id.cmp(&b.archive_id));
    let total = items.len();
    let start = (page - 1) * page_size;
    let end = (start + page_size).min(total);
    let page_items = if start >= total {
        vec![]
    } else {
        items[start..end].to_vec()
    };

    Ok(Json(ok(serde_json::json!({
        "items": page_items,
        "page": page,
        "page_size": page_size,
        "total": total
    }))))
}

async fn get_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "OPERATOR_ADMIN").await?;

    let archives = state.archives.read().await;
    let archive = archives
        .get(&archive_id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?
        .clone();

    Ok(Json(ok(archive)))
}

async fn generate_archive_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<QrGenerateData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "OPERATOR_ADMIN").await?;
    let archive = {
        let archives = state.archives.read().await;
        archives
            .get(&archive_id)
            .cloned()
            .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?
    };
    let qr_payload = dangan::build_qr_payload(&state, &archive).await?;
    let qr_content = serde_json::to_string(&qr_payload)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "GENERATE_ARCHIVE_QR",
        "QR",
        Some(qr_payload.qr_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_id": archive_id,
            "archive_no": qr_payload.archive_no,
            "citizen_status": qr_payload.citizen_status,
            "voting_eligible": qr_payload.voting_eligible,
            "sign_key_id": qr_payload.sign_key_id
        }),
    )
    .await?;

    Ok(Json(ok(QrGenerateData {
        qr_payload,
        qr_content,
    })))
}

async fn print_archive_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<QrPrintData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "OPERATOR_ADMIN").await?;
    let archive = {
        let archives = state.archives.read().await;
        archives
            .get(&archive_id)
            .cloned()
            .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?
    };
    let qr_payload = dangan::build_qr_payload(&state, &archive).await?;

    let record = QrPrintRecord {
        print_id: format!("qpr_{}", Uuid::new_v4().simple()),
        archive_id: archive.archive_id,
        archive_no: qr_payload.archive_no.clone(),
        citizen_status: qr_payload.citizen_status.clone(),
        voting_eligible: qr_payload.voting_eligible,
        printed_at: Utc::now().timestamp(),
    };
    state.qr_print_records.write().await.push(record.clone());

    write_audit(
        &state,
        Some(ctx.user_id),
        "PRINT_ARCHIVE_QR",
        "QR_PRINT_RECORD",
        Some(record.print_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_id": record.archive_id,
            "archive_no": record.archive_no,
            "citizen_status": record.citizen_status,
            "voting_eligible": record.voting_eligible,
            "sign_key_id": qr_payload.sign_key_id
        }),
    )
    .await?;

    Ok(Json(ok(QrPrintData {
        print_id: record.print_id,
        archive_id: record.archive_id,
        archive_no: record.archive_no,
        citizen_status: record.citizen_status,
        voting_eligible: record.voting_eligible,
        printed_at: record.printed_at,
    })))
}
