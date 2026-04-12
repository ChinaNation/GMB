//! # 操作员管理模块 (operator_admin)
//!
//! 档案创建/查询、QR 码生成/打印。仅 OPERATOR_ADMIN 角色可访问。

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    authz, dangan, err, find_admin_by_user_id, ok, write_audit, ApiError, ApiResponse, AppState,
    Archive,
};

#[derive(Deserialize)]
struct CreateArchiveRequest {
    province_code: String,
    city_code: String,
    full_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    #[serde(default)]
    passport_no: Option<String>,
    #[serde(default)]
    town_code: Option<String>,
    #[serde(default)]
    village_id: Option<String>,
    #[serde(default)]
    address: Option<String>,
    citizen_status: Option<String>,
    #[serde(default)]
    voting_eligible: Option<bool>,
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

// 编辑档案请求
#[derive(Deserialize)]
struct UpdateArchiveRequest {
    full_name: Option<String>,
    birth_date: Option<String>,
    gender_code: Option<String>,
    height_cm: Option<f32>,
    town_code: Option<String>,
    village_id: Option<String>,
    address: Option<String>,
    citizen_status: Option<String>,
    voting_eligible: Option<bool>,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/archives", post(create_archive).get(list_archives))
        .route("/api/v1/archives/:archive_id", get(get_archive).put(update_archive))
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
    let ctx = authz::require_auth(&state, &headers).await?;
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
    let archive_no = dangan::generate_archive_no_with_retry(
        &state,
        &req.province_code,
        terminal_id,
        &admin.admin_pubkey,
    )
    .await?;

    let addr = req.address.unwrap_or_default();
    if addr.chars().count() > 100 {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "address too long (max 100)"));
    }

    let voting = req.voting_eligible.unwrap_or(citizen_status == "NORMAL");

    let now_ts = Utc::now().timestamp();
    let mut archive = Archive {
        archive_id: format!("ar_{}", Uuid::new_v4().simple()),
        archive_no: archive_no.clone(),
        province_code: req.province_code,
        city_code: req.city_code,
        full_name: req.full_name,
        birth_date: req.birth_date,
        gender_code: req.gender_code,
        height_cm: req.height_cm,
        passport_no: req.passport_no.unwrap_or_default(),
        town_code: req.town_code.unwrap_or_default(),
        village_id: req.village_id.unwrap_or_default(),
        address: addr,
        status: "ACTIVE".to_string(),
        citizen_status,
        voting_eligible: voting,
        qr4_payload: String::new(),
        created_at: now_ts,
        updated_at: now_ts,
    };

    // 自动生成 QR4 payload
    match dangan::build_qr4_payload(&state, &archive).await {
        Ok(qr4) => {
            if let Ok(json) = serde_json::to_string(&qr4) {
                archive.qr4_payload = json;
            }
        }
        Err(_) => { /* QR3 未完成时无法生成 QR4，留空 */ }
    }

    sqlx::query(
        "INSERT INTO archives (archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, town_code, village_id, address, status, citizen_status, voting_eligible, qr4_payload, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
    )
    .bind(&archive.archive_id)
    .bind(&archive.archive_no)
    .bind(&archive.province_code)
    .bind(&archive.city_code)
    .bind(&archive.full_name)
    .bind(&archive.birth_date)
    .bind(&archive.gender_code)
    .bind(archive.height_cm)
    .bind(&archive.passport_no)
    .bind(&archive.town_code)
    .bind(&archive.village_id)
    .bind(&archive.address)
    .bind(&archive.status)
    .bind(&archive.citizen_status)
    .bind(archive.voting_eligible)
    .bind(&archive.qr4_payload)
    .bind(archive.created_at)
    .bind(archive.updated_at)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::CONFLICT, 3005, "archive_no conflict, retry exhausted"))?;

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
    authz::require_auth(&state, &headers).await?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;

    let (total, rows) = if let Some(name) = query.full_name {
        let pattern = format!("%{}%", name);
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM archives WHERE full_name LIKE $1")
                .bind(&pattern)
                .fetch_one(&state.db)
                .await
                .map_err(|_| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "count archives failed",
                    )
                })?;

        let rows = sqlx::query(
            "SELECT archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, COALESCE(qr4_payload,'') AS qr4_payload, created_at, updated_at
             FROM archives
             WHERE full_name LIKE $1
             ORDER BY archive_id
             LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query archives failed"))?;

        (total, rows)
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM archives")
            .fetch_one(&state.db)
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "count archives failed",
                )
            })?;

        let rows = sqlx::query(
            "SELECT archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, COALESCE(qr4_payload,'') AS qr4_payload, created_at, updated_at
             FROM archives
             ORDER BY archive_id
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query archives failed"))?;

        (total, rows)
    };

    let items: Vec<Archive> = rows.into_iter().map(row_to_archive).collect();

    Ok(Json(ok(serde_json::json!({
        "items": items,
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
    authz::require_auth(&state, &headers).await?;

    let archive = fetch_archive_by_id(&state, &archive_id).await?;
    Ok(Json(ok(archive)))
}

/// 编辑公民档案。修改后自动重新生成 QR4。
async fn update_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<UpdateArchiveRequest>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let mut archive = fetch_archive_by_id(&state, &archive_id).await?;

    if let Some(v) = req.full_name { archive.full_name = v; }
    if let Some(v) = req.birth_date { archive.birth_date = v; }
    if let Some(v) = req.gender_code { archive.gender_code = v; }
    if let Some(v) = req.height_cm { archive.height_cm = Some(v); }
    if let Some(v) = req.town_code { archive.town_code = v; }
    if let Some(v) = req.village_id { archive.village_id = v; }
    if let Some(v) = req.address {
        if v.chars().count() > 100 {
            return Err(err(StatusCode::BAD_REQUEST, 1001, "address too long (max 100)"));
        }
        archive.address = v;
    }
    if let Some(v) = req.citizen_status {
        dangan::validate_citizen_status(&v)?;
        archive.citizen_status = v;
    }
    if let Some(v) = req.voting_eligible { archive.voting_eligible = v; }

    archive.updated_at = Utc::now().timestamp();

    // 自动重新生成 QR4
    match dangan::build_qr4_payload(&state, &archive).await {
        Ok(qr4) => {
            if let Ok(json) = serde_json::to_string(&qr4) {
                archive.qr4_payload = json;
            }
        }
        Err(_) => { /* QR3 未完成时无法生成 */ }
    }

    sqlx::query(
        "UPDATE archives SET full_name=$1, birth_date=$2, gender_code=$3, height_cm=$4, town_code=$5, village_id=$6, address=$7, citizen_status=$8, voting_eligible=$9, qr4_payload=$10, updated_at=$11 WHERE archive_id=$12",
    )
    .bind(&archive.full_name)
    .bind(&archive.birth_date)
    .bind(&archive.gender_code)
    .bind(archive.height_cm)
    .bind(&archive.town_code)
    .bind(&archive.village_id)
    .bind(&archive.address)
    .bind(&archive.citizen_status)
    .bind(archive.voting_eligible)
    .bind(&archive.qr4_payload)
    .bind(archive.updated_at)
    .bind(&archive_id)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "update archive failed"))?;

    Ok(Json(ok(archive)))
}

async fn generate_archive_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<QrGenerateData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "OPERATOR_ADMIN").await?;
    let archive = fetch_archive_by_id(&state, &archive_id).await?;

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
    let archive = fetch_archive_by_id(&state, &archive_id).await?;

    let qr_payload = dangan::build_qr_payload(&state, &archive).await?;

    let record = QrPrintData {
        print_id: format!("qpr_{}", Uuid::new_v4().simple()),
        archive_id: archive.archive_id,
        archive_no: qr_payload.archive_no.clone(),
        citizen_status: qr_payload.citizen_status.clone(),
        voting_eligible: qr_payload.voting_eligible,
        printed_at: Utc::now().timestamp(),
    };

    sqlx::query(
        "INSERT INTO qr_print_records (print_id, archive_id, archive_no, citizen_status, voting_eligible, printed_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&record.print_id)
    .bind(&record.archive_id)
    .bind(&record.archive_no)
    .bind(&record.citizen_status)
    .bind(record.voting_eligible)
    .bind(record.printed_at)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save print record failed"))?;

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

    Ok(Json(ok(record)))
}

async fn fetch_archive_by_id(
    state: &AppState,
    archive_id: &str,
) -> Result<Archive, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, COALESCE(qr4_payload,'') AS qr4_payload, created_at, updated_at
         FROM archives
         WHERE archive_id = $1",
    )
    .bind(archive_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query archive failed"))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?;

    Ok(row_to_archive(row))
}

fn row_to_archive(row: sqlx::postgres::PgRow) -> Archive {
    Archive {
        archive_id: row.get("archive_id"),
        archive_no: row.get("archive_no"),
        province_code: row.get("province_code"),
        city_code: row.get("city_code"),
        full_name: row.get("full_name"),
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
        qr4_payload: row.try_get("qr4_payload").unwrap_or_default(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
