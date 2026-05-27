//! # 操作员管理模块 (operator_admin)
//!
//! 档案创建/查询、ARCHIVE 二维码生成/打印。仅 OPERATOR_ADMIN 角色可访问。

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
    authz, dangan, err, find_admin_by_user_id, initialize, ok, ss58, write_audit, ApiError,
    ApiResponse, AppState, Archive,
};

#[derive(Deserialize)]
struct CreateArchiveRequest {
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
    qr_payload: crate::dangan::ArchiveQrPayload,
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

#[derive(Deserialize)]
struct WalletBindRequest {
    wallet_address: String,
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
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/archives", post(create_archive).get(list_archives))
        .route(
            "/api/v1/archives/:archive_id",
            get(get_archive).put(update_archive),
        )
        .route(
            "/api/v1/archives/:archive_id/qr/generate",
            post(generate_archive_qr),
        )
        .route(
            "/api/v1/archives/:archive_id/wallet",
            post(bind_archive_wallet),
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
    let install = initialize::load_cpms_install_runtime(&state).await?;

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
        install.install_secret.as_str(),
        terminal_id,
        &admin.admin_pubkey,
    )
    .await?;

    let addr = req.address.unwrap_or_default();
    if addr.chars().count() > 100 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "address too long (max 100)",
        ));
    }

    let voting = citizen_status == "NORMAL";

    let now_ts = Utc::now().timestamp();
    let valid_from = dangan::archive_valid_from(now_ts);
    let valid_until = dangan::archive_valid_until(now_ts);
    let archive = Archive {
        archive_id: format!("ar_{}", Uuid::new_v4().simple()),
        archive_no: archive_no.clone(),
        province_code: install.province_code,
        city_code: install.city_code,
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
        valid_from,
        valid_until,
        citizen_status_updated_at: now_ts,
        wallet_address: None,
        wallet_pubkey: None,
        wallet_sig_alg: "sr25519".to_string(),
        wallet_bound_at: None,
        wallet_bound_by: None,
        archive_qr_payload: String::new(),
        created_at: now_ts,
        updated_at: now_ts,
    };

    sqlx::query(
        "INSERT INTO archives (archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, town_code, village_id, address, status, citizen_status, voting_eligible, valid_from, valid_until, citizen_status_updated_at, archive_qr_payload, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)",
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
    .bind(&archive.valid_from)
    .bind(&archive.valid_until)
    .bind(archive.citizen_status_updated_at)
    .bind(&archive.archive_qr_payload)
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
            "SELECT archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, COALESCE(valid_from,'') AS valid_from, COALESCE(valid_until,'') AS valid_until, COALESCE(citizen_status_updated_at, updated_at) AS citizen_status_updated_at, COALESCE(archive_qr_payload,'') AS archive_qr_payload, created_at, updated_at
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
            "SELECT archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, COALESCE(valid_from,'') AS valid_from, COALESCE(valid_until,'') AS valid_until, COALESCE(citizen_status_updated_at, updated_at) AS citizen_status_updated_at, COALESCE(archive_qr_payload,'') AS archive_qr_payload, created_at, updated_at
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

async fn bind_archive_wallet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<WalletBindRequest>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_role(&state, &headers, "OPERATOR_ADMIN").await?;
    let now = Utc::now().timestamp();
    let wallet_address = req.wallet_address.trim();
    let wallet_pubkey = ss58::ss58_to_pubkey_hex(wallet_address)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "invalid wallet_address"))?;

    // 中文注释:CPMS 线下只确认并保存钱包地址;钱包签名验证统一放到 SFID 绑定阶段。
    let result = sqlx::query(
        "UPDATE archives
         SET wallet_address=$1, wallet_pubkey=$2, wallet_sig_alg='sr25519',
             wallet_bound_at=$3, wallet_bound_by=$4, archive_qr_payload='', updated_at=$5
         WHERE archive_id=$6",
    )
    .bind(wallet_address)
    .bind(&wallet_pubkey)
    .bind(now)
    .bind(&ctx.user_id)
    .bind(now)
    .bind(&archive_id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "update archive wallet failed",
        )
    })?;
    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, 3004, "archive not found"));
    }

    write_audit(
        &state,
        Some(ctx.user_id),
        "BIND_ARCHIVE_WALLET",
        "CITIZEN_ARCHIVE",
        Some(archive_id.clone()),
        "SUCCESS",
        serde_json::json!({ "wallet_pubkey": wallet_pubkey, "wallet_address": wallet_address }),
    )
    .await?;

    let archive = fetch_archive_by_id(&state, &archive_id).await?;
    Ok(Json(ok(archive)))
}

/// 编辑公民档案。修改后自动重新生成 ARCHIVE 二维码。
async fn update_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<UpdateArchiveRequest>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let mut archive = fetch_archive_by_id(&state, &archive_id).await?;

    if let Some(v) = req.full_name {
        archive.full_name = v;
    }
    if let Some(v) = req.birth_date {
        archive.birth_date = v;
    }
    if let Some(v) = req.gender_code {
        archive.gender_code = v;
    }
    if let Some(v) = req.height_cm {
        archive.height_cm = Some(v);
    }
    if let Some(v) = req.town_code {
        archive.town_code = v;
    }
    if let Some(v) = req.village_id {
        archive.village_id = v;
    }
    if let Some(v) = req.address {
        if v.chars().count() > 100 {
            return Err(err(
                StatusCode::BAD_REQUEST,
                1001,
                "address too long (max 100)",
            ));
        }
        archive.address = v;
    }
    if let Some(v) = req.citizen_status {
        dangan::validate_citizen_status(&v)?;
        archive.citizen_status = v;
        archive.citizen_status_updated_at = Utc::now().timestamp();
    }
    archive.voting_eligible = archive.citizen_status == "NORMAL";

    archive.updated_at = Utc::now().timestamp();

    // 中文注释:只有已保存钱包地址的档案才允许签出 ARCHIVE;编辑实名字段时保留该硬门槛。
    archive.archive_qr_payload = if archive_has_wallet(&archive) {
        let archive_qr = dangan::build_archive_qr_payload(&state, &archive).await?;
        serde_json::to_string(&archive_qr).map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "archive qr encode failed",
            )
        })?
    } else {
        String::new()
    };

    sqlx::query(
        "UPDATE archives SET full_name=$1, birth_date=$2, gender_code=$3, height_cm=$4, town_code=$5, village_id=$6, address=$7, citizen_status=$8, voting_eligible=$9, citizen_status_updated_at=$10, archive_qr_payload=$11, updated_at=$12 WHERE archive_id=$13",
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
    .bind(archive.citizen_status_updated_at)
    .bind(&archive.archive_qr_payload)
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

    let qr_payload = dangan::build_archive_qr_payload(&state, &archive).await?;
    let qr_content = serde_json::to_string(&qr_payload)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))?;
    sqlx::query("UPDATE archives SET archive_qr_payload=$1, updated_at=$2 WHERE archive_id=$3")
        .bind(&qr_content)
        .bind(Utc::now().timestamp())
        .bind(&archive_id)
        .execute(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "save archive qr failed",
            )
        })?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "GENERATE_ARCHIVE_QR",
        "QR",
        Some(qr_payload.ano.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_id": archive_id,
            "archive_no": qr_payload.ano,
            "citizen_status": qr_payload.cs,
            "valid_from": qr_payload.valid_from,
            "valid_until": qr_payload.valid_until,
            "cpms_pubkey": qr_payload.cpms_pubkey
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

    let qr_payload = dangan::build_archive_qr_payload(&state, &archive).await?;

    let record = QrPrintData {
        print_id: format!("qpr_{}", Uuid::new_v4().simple()),
        archive_id: archive.archive_id,
        archive_no: qr_payload.ano.clone(),
        citizen_status: qr_payload.cs.clone(),
        voting_eligible: archive.voting_eligible,
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
            "valid_from": qr_payload.valid_from,
            "valid_until": qr_payload.valid_until,
            "cpms_pubkey": qr_payload.cpms_pubkey
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
        "SELECT archive_id, archive_no, province_code, city_code, full_name, birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, COALESCE(valid_from,'') AS valid_from, COALESCE(valid_until,'') AS valid_until, COALESCE(citizen_status_updated_at, updated_at) AS citizen_status_updated_at, wallet_address, wallet_pubkey, COALESCE(wallet_sig_alg,'sr25519') AS wallet_sig_alg, wallet_bound_at, wallet_bound_by, COALESCE(archive_qr_payload,'') AS archive_qr_payload, created_at, updated_at
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
        valid_from: row.try_get("valid_from").unwrap_or_default(),
        valid_until: row.try_get("valid_until").unwrap_or_default(),
        citizen_status_updated_at: row
            .try_get("citizen_status_updated_at")
            .unwrap_or_else(|_| {
                row.try_get("updated_at")
                    .unwrap_or_else(|_| Utc::now().timestamp())
            }),
        wallet_address: row.try_get("wallet_address").ok(),
        wallet_pubkey: row.try_get("wallet_pubkey").ok(),
        wallet_sig_alg: row
            .try_get("wallet_sig_alg")
            .unwrap_or_else(|_| "sr25519".to_string()),
        wallet_bound_at: row.try_get("wallet_bound_at").ok(),
        wallet_bound_by: row.try_get("wallet_bound_by").ok(),
        archive_qr_payload: row.try_get("archive_qr_payload").unwrap_or_default(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn archive_has_wallet(archive: &Archive) -> bool {
    archive
        .wallet_address
        .as_deref()
        .is_some_and(|v| !v.trim().is_empty())
        && archive
            .wallet_pubkey
            .as_deref()
            .is_some_and(|v| !v.trim().is_empty())
}
