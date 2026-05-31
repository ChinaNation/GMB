//! # 公民档案业务路由
//!
//! 档案创建/查询、投票账户绑定、软删除、ARCHIVE 二维码更新/打印。
//! 中文注释：档案业务允许 SUPER_ADMIN 与 OPERATOR_ADMIN；系统管理才仅限 SUPER_ADMIN。

use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Error as SqlxError, Postgres, QueryBuilder, Row};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    address, authz, dangan, err, find_admin_by_user_id, initialize, ok, rate_limit, ss58,
    write_audit, ApiError, ApiResponse, AppState, Archive,
};

#[derive(Deserialize)]
struct CreateArchiveRequest {
    last_name: String,
    first_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    #[serde(default)]
    town_code: Option<String>,
    #[serde(default)]
    village_id: Option<String>,
    #[serde(default)]
    address: Option<String>,
    citizen_status: Option<String>,
    voting_eligible: Option<bool>,
}

#[derive(Serialize)]
struct CreateArchiveData {
    archive_id: String,
    archive_no: String,
    passport_no: String,
    status: String,
    citizen_status: String,
}

#[derive(Deserialize)]
// 中文注释：档案列表只接受游标分页和索引化精确检索参数；旧的 page/page_size/q 和选择器式字段参数会被拒绝。
#[serde(deny_unknown_fields)]
struct ArchiveListQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    search: Option<String>,
    birth_date: Option<String>,
    town_code: Option<String>,
    village_id: Option<String>,
    citizen_status: Option<String>,
}

#[derive(Serialize)]
struct ArchiveListData {
    items: Vec<Archive>,
    limit: usize,
    next_cursor: Option<String>,
    has_next: bool,
    total_active: i64,
}

#[derive(Serialize, Deserialize)]
// 中文注释：游标绑定稳定排序键，避免百万级档案使用 OFFSET 扫描。
struct ArchiveListCursor {
    created_at: i64,
    archive_id: String,
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

#[derive(Serialize)]
struct ArchiveDeleteChallengeData {
    challenge_id: String,
    sign_request: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct ArchiveDeleteCompleteRequest {
    challenge_id: String,
    pubkey: String,
    sig_alg: String,
    signature: String,
    payload_hash: String,
    signed_at: i64,
}

#[derive(Serialize)]
struct ArchiveDeleteCompleteData {
    archive_id: String,
    deleted_at: i64,
    deleted_by: String,
}

#[derive(Serialize)]
struct StatusExportData {
    file_name: String,
    export_file: dangan::CpmsStatusExportFile,
}

#[derive(Serialize)]
struct StatusExportStateData {
    state: dangan::CpmsStatusExportState,
}

// 编辑档案请求
#[derive(Deserialize)]
struct UpdateArchiveRequest {
    last_name: Option<String>,
    first_name: Option<String>,
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
            post(save_archive_wallet),
        )
        .route(
            "/api/v1/archives/:archive_id/qr/print",
            post(print_archive_qr),
        )
        .route(
            "/api/v1/archives/:archive_id/delete/challenge",
            post(create_archive_delete_challenge),
        )
        .route(
            "/api/v1/archives/:archive_id/delete/complete",
            post(complete_archive_delete),
        )
        .route(
            "/api/v1/archives/status-export/state",
            get(status_export_state),
        )
        .route("/api/v1/archives/status-export", get(export_status_file))
}

async fn create_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateArchiveRequest>,
) -> Result<Json<ApiResponse<CreateArchiveData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    let admin = find_admin_by_user_id(&state, &ctx.user_id).await?;
    let install = initialize::load_cpms_install_runtime(&state).await?;

    if req.last_name.trim().is_empty() || req.first_name.trim().is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "last_name and first_name are required",
        ));
    }
    let birth_date = validate_birth_date(&req.birth_date)?;
    if req.gender_code != "M" && req.gender_code != "W" {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid gender_code"));
    }
    let height_cm = req
        .height_cm
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "height_cm is required"))?;
    validate_height_cm(height_cm)?;
    let town_code = req.town_code.unwrap_or_default().trim().to_string();
    let village_id = req.village_id.unwrap_or_default().trim().to_string();
    if town_code.trim().is_empty() || village_id.trim().is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "town_code and village_id are required",
        ));
    }
    address::validate_town_village(&state, &town_code, &village_id).await?;
    let terminal_id = headers
        .get("x-terminal-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("terminal-000");
    let citizen_status = req
        .citizen_status
        .unwrap_or_else(|| dangan::CITIZEN_STATUS_NORMAL.to_string());
    dangan::validate_citizen_status(&citizen_status)?;
    let archive_id = format!("ar_{}", Uuid::new_v4().simple());
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;
    let numbers = crate::number::generate_archive_numbers_with_retry(
        tx.as_mut(),
        &archive_id,
        install.install_secret.as_str(),
        install.province_code.as_str(),
        install.city_code.as_str(),
        terminal_id,
        &admin.admin_pubkey,
    )
    .await?;

    let addr = req.address.unwrap_or_default().trim().to_string();
    if addr.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "address is required"));
    }
    if addr.chars().count() > 100 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "address too long (max 100)",
        ));
    }

    let now_ts = Utc::now().timestamp();
    let voting = dangan::resolve_voting_eligible(
        &citizen_status,
        birth_date,
        req.voting_eligible,
        true,
        now_ts,
    )?;
    let valid_from = dangan::archive_valid_from(now_ts);
    let valid_until =
        dangan::archive_valid_until(now_ts, dangan::archive_validity_years(now_ts, birth_date));
    let archive = Archive {
        archive_id,
        archive_no: numbers.archive_no.clone(),
        province_code: install.province_code,
        city_code: install.city_code,
        last_name: req.last_name.trim().to_string(),
        first_name: req.first_name.trim().to_string(),
        birth_date: req.birth_date.trim().to_string(),
        gender_code: req.gender_code,
        height_cm: Some(height_cm),
        passport_no: numbers.passport_no.clone(),
        town_code,
        village_id,
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
        deleted_at: None,
        deleted_by: None,
        delete_reason: None,
        created_at: now_ts,
        updated_at: now_ts,
    };

    // 中文注释：号码池领取与档案写入必须同事务完成，避免回收号码被半消费。
    sqlx::query(
        "INSERT INTO archives (archive_id, archive_no, province_code, city_code, last_name, first_name, birth_date, gender_code, height_cm, passport_no, town_code, village_id, address, status, citizen_status, voting_eligible, valid_from, valid_until, citizen_status_updated_at, archive_qr_payload, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7::DATE, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17::DATE, $18::DATE, $19, $20, $21, $22)",
    )
    .bind(&archive.archive_id)
    .bind(&archive.archive_no)
    .bind(&archive.province_code)
    .bind(&archive.city_code)
    .bind(&archive.last_name)
    .bind(&archive.first_name)
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
    .execute(tx.as_mut())
    .await
    .map_err(|_| err(StatusCode::CONFLICT, 3005, "archive_no conflict, retry exhausted"))?;

    // 中文注释：档案列表总数来自统计表；创建档案时和档案写入同事务递增。
    dangan::adjust_archive_stats(tx.as_mut(), 1, 0, now_ts).await?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

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
        archive_no: numbers.archive_no,
        passport_no: numbers.passport_no,
        status: archive.status,
        citizen_status: archive.citizen_status,
    })))
}

async fn list_archives(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ArchiveListQuery>,
) -> Result<Json<ApiResponse<ArchiveListData>>, (StatusCode, Json<ApiError>)> {
    authz::require_archive_admin(&state, &headers).await?;

    // 中文注释：单页上限固定为 100，防止一次请求拖垮市级百万档案列表。
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let cursor = decode_archive_list_cursor(query.cursor.as_deref())?;
    validate_archive_list_query(&query)?;

    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT archive_id, archive_no, province_code, city_code, last_name, first_name, birth_date::TEXT AS birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, valid_from::TEXT AS valid_from, valid_until::TEXT AS valid_until, COALESCE(citizen_status_updated_at, updated_at) AS citizen_status_updated_at, wallet_address, wallet_pubkey, COALESCE(wallet_sig_alg,'sr25519') AS wallet_sig_alg, wallet_bound_at, wallet_bound_by, COALESCE(archive_qr_payload,'') AS archive_qr_payload, deleted_at, deleted_by, delete_reason, created_at, updated_at
         FROM archives
         WHERE status = 'ACTIVE'",
    );
    push_archive_list_filters(&mut qb, &query);
    if let Some(cursor) = cursor {
        qb.push(" AND (created_at, archive_id) < (");
        qb.push_bind(cursor.created_at);
        qb.push(", ");
        qb.push_bind(cursor.archive_id);
        qb.push(")");
    }
    qb.push(" ORDER BY created_at DESC, archive_id DESC LIMIT ");
    qb.push_bind((limit + 1) as i64);

    let rows = qb.build().fetch_all(&state.db).await.map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query archives failed",
        )
    })?;

    let has_next = rows.len() > limit;
    let mut items: Vec<Archive> = rows.into_iter().take(limit).map(row_to_archive).collect();
    let next_cursor = if has_next {
        items
            .last()
            .map(|archive| encode_archive_list_cursor(archive.created_at, &archive.archive_id))
            .transpose()?
    } else {
        None
    };
    items.shrink_to_fit();
    let total_active = super::stats::load_active_archive_count(&state.db).await?;

    Ok(Json(ok(ArchiveListData {
        items,
        limit,
        next_cursor,
        has_next,
        total_active,
    })))
}

fn push_archive_list_filters(qb: &mut QueryBuilder<'_, Postgres>, query: &ArchiveListQuery) {
    if let Some(search) = trimmed_opt(query.search.as_deref()) {
        // 中文注释：统一输入框只做精确匹配，不做 LIKE；由 PostgreSQL 用各字段索引组合执行。
        qb.push(" AND (archive_no = ");
        qb.push_bind(search.to_string());
        qb.push(" OR passport_no = ");
        qb.push_bind(search.to_string());
        qb.push(" OR (last_name || first_name) = ");
        qb.push_bind(search.to_string());
        qb.push(")");
    }
    if let Some(birth_date) = trimmed_opt(query.birth_date.as_deref()) {
        qb.push(" AND birth_date = ");
        qb.push_bind(birth_date.to_string());
        qb.push("::DATE");
    }
    if let Some(town_code) = trimmed_opt(query.town_code.as_deref()) {
        qb.push(" AND town_code = ");
        qb.push_bind(town_code.to_string());
    }
    if let Some(village_id) = trimmed_opt(query.village_id.as_deref()) {
        qb.push(" AND village_id = ");
        qb.push_bind(village_id.to_string());
    }
    if let Some(citizen_status) = trimmed_opt(query.citizen_status.as_deref()) {
        qb.push(" AND citizen_status = ");
        qb.push_bind(citizen_status.to_string());
    }
}

fn validate_archive_list_query(
    query: &ArchiveListQuery,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    validate_short_filter(query.search.as_deref(), "search", 64)?;
    validate_short_filter(query.town_code.as_deref(), "town_code", 32)?;
    validate_short_filter(query.village_id.as_deref(), "village_id", 64)?;
    if let Some(birth_date) = trimmed_opt(query.birth_date.as_deref()) {
        NaiveDate::parse_from_str(birth_date, "%Y-%m-%d")
            .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"))?;
    }
    if let Some(citizen_status) = trimmed_opt(query.citizen_status.as_deref()) {
        dangan::validate_citizen_status(citizen_status)?;
    }
    Ok(())
}

fn validate_short_filter(
    value: Option<&str>,
    field: &str,
    max_chars: usize,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    if let Some(value) = value {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.chars().count() > max_chars {
            return Err(err(
                StatusCode::BAD_REQUEST,
                1001,
                &format!("invalid {field}"),
            ));
        }
    }
    Ok(())
}

fn trimmed_opt(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

fn encode_archive_list_cursor(
    created_at: i64,
    archive_id: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let cursor = ArchiveListCursor {
        created_at,
        archive_id: archive_id.to_string(),
    };
    let json = serde_json::to_vec(&cursor).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "encode cursor failed",
        )
    })?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

fn decode_archive_list_cursor(
    raw: Option<&str>,
) -> Result<Option<ArchiveListCursor>, (StatusCode, Json<ApiError>)> {
    let Some(raw) = trimmed_opt(raw) else {
        return Ok(None);
    };
    let bytes = URL_SAFE_NO_PAD
        .decode(raw.as_bytes())
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid cursor"))?;
    let cursor: ArchiveListCursor = serde_json::from_slice(&bytes)
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid cursor"))?;
    if cursor.archive_id.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid cursor"));
    }
    Ok(Some(cursor))
}

async fn get_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    authz::require_archive_admin(&state, &headers).await?;

    let archive = fetch_archive_by_id(&state, &archive_id).await?;
    Ok(Json(ok(archive)))
}

async fn save_archive_wallet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<WalletBindRequest>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    let now = Utc::now().timestamp();
    let wallet_address = req.wallet_address.trim();
    let wallet_pubkey = ss58::ss58_to_pubkey_hex(wallet_address)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "invalid wallet_address"))?;
    let existing_archive: Option<String> = sqlx::query_scalar(
        "SELECT archive_id
         FROM archives
         WHERE wallet_pubkey = $1 AND archive_id <> $2
         LIMIT 1",
    )
    .bind(&wallet_pubkey)
    .bind(&archive_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query archive wallet failed",
        )
    })?;
    if existing_archive.is_some() {
        return Err(err(StatusCode::CONFLICT, 3009, "wallet already bound"));
    }

    // 中文注释:CPMS 线下只确认并保存钱包地址;钱包签名验证统一放到 SFID 绑定阶段。
    let result = sqlx::query(
        "UPDATE archives
         SET wallet_address=$1, wallet_pubkey=$2, wallet_sig_alg='sr25519',
             wallet_bound_at=$3, wallet_bound_by=$4, updated_at=$5
         WHERE archive_id=$6 AND status <> 'DELETED'",
    )
    .bind(wallet_address)
    .bind(&wallet_pubkey)
    .bind(now)
    .bind(&ctx.user_id)
    .bind(now)
    .bind(&archive_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        if is_unique_constraint(&e, "uq_archives_wallet_pubkey_lifetime") {
            err(StatusCode::CONFLICT, 3009, "wallet already bound")
        } else {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "update archive wallet failed",
            )
        }
    })?;
    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, 3004, "archive not found"));
    }
    dangan::clear_archive_qr_payload(&state, &archive_id, now).await?;

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

/// 编辑公民档案。实名字段变更后清空旧 ARCHIVE 二维码，等待更新按钮重新签发。
async fn update_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<UpdateArchiveRequest>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    authz::require_archive_admin(&state, &headers).await?;
    let mut archive = fetch_archive_by_id(&state, &archive_id).await?;
    ensure_archive_not_deleted(&archive)?;

    if let Some(v) = req.last_name {
        if v.trim().is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, 1001, "last_name is required"));
        }
        archive.last_name = v.trim().to_string();
    }
    if let Some(v) = req.first_name {
        if v.trim().is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, 1001, "first_name is required"));
        }
        archive.first_name = v.trim().to_string();
    }
    if let Some(v) = req.birth_date {
        validate_birth_date(&v)?;
        archive.birth_date = v.trim().to_string();
    }
    if let Some(v) = req.gender_code {
        if v != "M" && v != "W" {
            return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid gender_code"));
        }
        archive.gender_code = v;
    }
    if let Some(v) = req.height_cm {
        validate_height_cm(v)?;
        archive.height_cm = Some(v);
    }
    if let Some(v) = req.town_code {
        archive.town_code = v.trim().to_string();
    }
    if let Some(v) = req.village_id {
        archive.village_id = v.trim().to_string();
    }
    address::validate_town_village(&state, &archive.town_code, &archive.village_id).await?;
    if let Some(v) = req.address {
        let address = v.trim().to_string();
        if address.is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, 1001, "address is required"));
        }
        if address.chars().count() > 100 {
            return Err(err(
                StatusCode::BAD_REQUEST,
                1001,
                "address too long (max 100)",
            ));
        }
        archive.address = address;
    }
    if let Some(v) = req.citizen_status {
        dangan::validate_citizen_status(&v)?;
        archive.citizen_status = v;
        archive.citizen_status_updated_at = Utc::now().timestamp();
    }
    let requested_voting = req.voting_eligible;
    if let Some(v) = requested_voting {
        archive.voting_eligible = v;
    }
    let birth_date = validate_birth_date(&archive.birth_date)?;
    archive.voting_eligible = dangan::resolve_voting_eligible(
        &archive.citizen_status,
        birth_date,
        requested_voting,
        archive.voting_eligible,
        Utc::now().timestamp(),
    )?;

    archive.updated_at = Utc::now().timestamp();

    archive.archive_qr_payload = String::new();

    sqlx::query(
        "UPDATE archives SET last_name=$1, first_name=$2, birth_date=$3::DATE, gender_code=$4, height_cm=$5, town_code=$6, village_id=$7, address=$8, citizen_status=$9, voting_eligible=$10, citizen_status_updated_at=$11, updated_at=$12 WHERE archive_id=$13",
    )
    .bind(&archive.last_name)
    .bind(&archive.first_name)
    .bind(&archive.birth_date)
    .bind(&archive.gender_code)
    .bind(archive.height_cm)
    .bind(&archive.town_code)
    .bind(&archive.village_id)
    .bind(&archive.address)
    .bind(&archive.citizen_status)
    .bind(archive.voting_eligible)
    .bind(archive.citizen_status_updated_at)
    .bind(archive.updated_at)
    .bind(&archive_id)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "update archive failed"))?;
    dangan::clear_archive_qr_payload(&state, &archive_id, archive.updated_at).await?;

    Ok(Json(ok(archive)))
}

async fn generate_archive_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<QrGenerateData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    let archive = fetch_archive_by_id(&state, &archive_id).await?;
    ensure_archive_not_deleted(&archive)?;
    ensure_archive_qr_ready(&state, &archive).await?;

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
        Some(qr_payload.archive_no.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_id": archive_id,
            "archive_no": qr_payload.archive_no,
            "citizen_status": qr_payload.citizen_status,
            "voting_eligible": qr_payload.voting_eligible,
            "valid_from": qr_payload.valid_from,
            "valid_until": qr_payload.valid_until,
            "status_updated_at": qr_payload.status_updated_at,
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
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    let archive = fetch_archive_by_id(&state, &archive_id).await?;
    ensure_archive_not_deleted(&archive)?;
    ensure_archive_qr_ready(&state, &archive).await?;

    let qr_payload = dangan::build_archive_qr_payload(&state, &archive).await?;

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
            "valid_from": qr_payload.valid_from,
            "valid_until": qr_payload.valid_until,
            "cpms_pubkey": qr_payload.cpms_pubkey
        }),
    )
    .await?;

    Ok(Json(ok(record)))
}

async fn create_archive_delete_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<ArchiveDeleteChallengeData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    let admin = find_admin_by_user_id(&state, &ctx.user_id).await?;
    let archive = fetch_archive_by_id(&state, &archive_id).await?;
    ensure_archive_not_deleted(&archive)?;

    let issued_at = Utc::now().timestamp();
    let expire_at = issued_at + 120;
    let challenge_id = format!("adc_{}", Uuid::new_v4().simple());
    let admin_address = ss58::pubkey_hex_to_ss58(&admin.admin_pubkey).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid admin pubkey",
        )
    })?;
    let admin_pubkey_hex = normalize_pubkey_hex(&admin.admin_pubkey).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid admin pubkey",
        )
    })?;
    let delete_payload = build_archive_delete_payload(
        &challenge_id,
        &archive.archive_id,
        &archive.archive_no,
        &admin_pubkey_hex,
        expire_at,
    )?;
    let payload_hex = format!("0x{}", hex::encode(delete_payload.as_bytes()));
    let sign_request = build_archive_delete_sign_request(
        &challenge_id,
        issued_at,
        expire_at,
        &admin_address,
        &admin_pubkey_hex,
        &payload_hex,
        &archive,
    )?;

    sqlx::query(
        "INSERT INTO archive_delete_challenges
         (challenge_id, archive_id, archive_no, admin_id, admin_pubkey, delete_payload, expire_at, consumed, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8)",
    )
    .bind(&challenge_id)
    .bind(&archive.archive_id)
    .bind(&archive.archive_no)
    .bind(&ctx.user_id)
    .bind(&admin_pubkey_hex)
    .bind(&delete_payload)
    .bind(expire_at)
    .bind(issued_at)
    .execute(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "create delete challenge failed",
        )
    })?;

    Ok(Json(ok(ArchiveDeleteChallengeData {
        challenge_id,
        sign_request,
        expire_at,
    })))
}

async fn complete_archive_delete(
    State(state): State<AppState>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<ArchiveDeleteCompleteRequest>,
) -> Result<Json<ApiResponse<ArchiveDeleteCompleteData>>, (StatusCode, Json<ApiError>)> {
    rate_limit::check(
        &state,
        client_addr,
        &headers,
        "archive_delete_complete",
        20,
        60,
    )
    .await?;

    let ctx = authz::require_archive_admin(&state, &headers).await?;
    let admin = find_admin_by_user_id(&state, &ctx.user_id).await?;

    if req.sig_alg != "sr25519" {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete signature algorithm invalid",
        )
        .await;
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            3014,
            "delete signature verify failed",
        ));
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let row = match sqlx::query(
        "SELECT challenge_id, archive_id, archive_no, admin_id, admin_pubkey, delete_payload, expire_at, consumed
         FROM archive_delete_challenges
         WHERE challenge_id = $1
         FOR UPDATE",
    )
    .bind(req.challenge_id.trim())
    .fetch_optional(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query delete challenge failed",
        )
    })? {
        Some(row) => row,
        None => {
            audit_archive_delete_failure(
                &state,
                &ctx.user_id,
                &archive_id,
                req.challenge_id.trim(),
                "delete challenge not found",
            )
            .await;
            return Err(err(
                StatusCode::NOT_FOUND,
                3004,
                "delete challenge not found",
            ));
        }
    };

    let challenge_archive_id: String = row.get("archive_id");
    let challenge_admin_id: String = row.get("admin_id");
    let challenge_admin_pubkey: String = row.get("admin_pubkey");
    let delete_payload: String = row.get("delete_payload");
    let expire_at: i64 = row.get("expire_at");
    let consumed: bool = row.get("consumed");

    if consumed {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete challenge already consumed",
        )
        .await;
        return Err(err(
            StatusCode::CONFLICT,
            3011,
            "delete challenge already consumed",
        ));
    }
    if expire_at < Utc::now().timestamp() {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete challenge expired",
        )
        .await;
        return Err(err(StatusCode::GONE, 3012, "delete challenge expired"));
    }
    if challenge_archive_id != archive_id || challenge_admin_id != ctx.user_id {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete challenge mismatch",
        )
        .await;
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            3013,
            "delete challenge mismatch",
        ));
    }

    let archive_row = match sqlx::query(
        "SELECT archive_id, archive_no, province_code, city_code, last_name, first_name, birth_date::TEXT AS birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, valid_from::TEXT AS valid_from, valid_until::TEXT AS valid_until, COALESCE(citizen_status_updated_at, updated_at) AS citizen_status_updated_at, wallet_address, wallet_pubkey, COALESCE(wallet_sig_alg,'sr25519') AS wallet_sig_alg, wallet_bound_at, wallet_bound_by, COALESCE(archive_qr_payload,'') AS archive_qr_payload, deleted_at, deleted_by, delete_reason, created_at, updated_at
         FROM archives
         WHERE archive_id = $1
         FOR UPDATE",
    )
    .bind(&archive_id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query archive failed"))? {
        Some(row) => row,
        None => {
            audit_archive_delete_failure(
                &state,
                &ctx.user_id,
                &archive_id,
                req.challenge_id.trim(),
                "archive not found",
            )
            .await;
            return Err(err(StatusCode::NOT_FOUND, 3004, "archive not found"));
        }
    };
    let archive = row_to_archive(archive_row);
    if archive.status == "DELETED" || archive.deleted_at.is_some() {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "archive already deleted",
        )
        .await;
        return Err(err(StatusCode::CONFLICT, 3008, "archive already deleted"));
    }
    if archive.archive_no != row.get::<String, _>("archive_no") {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete challenge mismatch",
        )
        .await;
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            3013,
            "delete challenge mismatch",
        ));
    }

    let Some(signed_pubkey) = normalize_pubkey_hex(&req.pubkey) else {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete signer mismatch",
        )
        .await;
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            3014,
            "delete signer mismatch",
        ));
    };
    let expected_pubkey = normalize_pubkey_hex(&challenge_admin_pubkey).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid admin pubkey",
        )
    })?;
    let current_admin_pubkey = normalize_pubkey_hex(&admin.admin_pubkey).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid admin pubkey",
        )
    })?;
    if signed_pubkey != expected_pubkey || signed_pubkey != current_admin_pubkey {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete signer mismatch",
        )
        .await;
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            3014,
            "delete signer mismatch",
        ));
    }

    let expected_hash = payload_sha256_hex(delete_payload.as_bytes());
    if req.payload_hash.to_lowercase() != expected_hash {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete payload hash mismatch",
        )
        .await;
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            3015,
            "delete payload hash mismatch",
        ));
    }
    if req.signed_at > expire_at + 30 {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "delete challenge expired",
        )
        .await;
        return Err(err(StatusCode::GONE, 3012, "delete challenge expired"));
    }

    if let Err(reason) = crate::login::verify_wumin_login_signature(
        &signed_pubkey,
        &delete_payload,
        req.signature.trim(),
    ) {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            reason,
        )
        .await;
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            3016,
            "delete signature verify failed",
        ));
    }

    let deleted_at = Utc::now().timestamp();
    let delete_result = sqlx::query(
        "UPDATE archives
         SET status='DELETED', citizen_status='REVOKED', voting_eligible=false,
             citizen_status_updated_at=$1, deleted_at=$1, deleted_by=$2, delete_reason=$3, updated_at=$1
         WHERE archive_id=$4 AND status <> 'DELETED'",
    )
    .bind(deleted_at)
    .bind(&ctx.user_id)
    .bind("wumin signed archive delete")
    .bind(&archive_id)
    .execute(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "delete archive failed",
        )
    })?;
    if delete_result.rows_affected() == 0 {
        audit_archive_delete_failure(
            &state,
            &ctx.user_id,
            &archive_id,
            req.challenge_id.trim(),
            "archive already deleted",
        )
        .await;
        return Err(err(StatusCode::CONFLICT, 3008, "archive already deleted"));
    }

    // 中文注释：注销软删除会从有效档案总数扣减，避免列表页实时 COUNT 百万级档案。
    dangan::adjust_archive_stats(tx.as_mut(), -1, 1, deleted_at).await?;

    sqlx::query(
        "UPDATE archive_delete_challenges SET consumed=true, consumed_at=$1 WHERE challenge_id=$2 AND consumed=false",
    )
    .bind(deleted_at)
    .bind(req.challenge_id.trim())
    .execute(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "consume delete challenge failed",
        )
    })?;

    sqlx::query(
        "INSERT INTO audit_logs (log_id, operator_user_id, action, target_type, target_id, result, detail, created_at)
         VALUES ($1, $2, 'DELETE_ARCHIVE', 'CITIZEN_ARCHIVE', $3, 'SUCCESS', $4, $5)",
    )
    .bind(format!("log_{}", Uuid::new_v4().simple()))
    .bind(&ctx.user_id)
    .bind(&archive_id)
    .bind(sqlx::types::Json(serde_json::json!({
        "archive_no": archive.archive_no,
        "deleted_at": deleted_at,
        "signer_pubkey": signed_pubkey
    })))
    .bind(deleted_at)
    .execute(tx.as_mut())
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "write audit failed"))?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    Ok(Json(ok(ArchiveDeleteCompleteData {
        archive_id,
        deleted_at,
        deleted_by: ctx.user_id,
    })))
}

async fn audit_archive_delete_failure(
    state: &AppState,
    operator_user_id: &str,
    archive_id: &str,
    challenge_id: &str,
    reason: &str,
) {
    let _ = write_audit(
        state,
        Some(operator_user_id.to_string()),
        "DELETE_ARCHIVE",
        "CITIZEN_ARCHIVE",
        Some(archive_id.to_string()),
        "FAILED",
        serde_json::json!({
            "archive_id": archive_id,
            "challenge_id": challenge_id,
            "reason": reason,
        }),
    )
    .await;
}

async fn export_status_file(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StatusExportData>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let export_file = dangan::build_and_record_cpms_status_export(&state).await?;
    let file_name = format!("cpms-annual-status-report-{}.json", export_file.exported_at);
    Ok(Json(ok(StatusExportData {
        file_name,
        export_file,
    })))
}

async fn status_export_state(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StatusExportStateData>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let export_state = dangan::status_export_state(&state).await?;
    Ok(Json(ok(StatusExportStateData {
        state: export_state,
    })))
}

async fn fetch_archive_by_id(
    state: &AppState,
    archive_id: &str,
) -> Result<Archive, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT archive_id, archive_no, province_code, city_code, last_name, first_name, birth_date::TEXT AS birth_date, gender_code, height_cm, passport_no, COALESCE(town_code,'') AS town_code, COALESCE(village_id,'') AS village_id, COALESCE(address,'') AS address, status, citizen_status, COALESCE(voting_eligible,true) AS voting_eligible, valid_from::TEXT AS valid_from, valid_until::TEXT AS valid_until, COALESCE(citizen_status_updated_at, updated_at) AS citizen_status_updated_at, wallet_address, wallet_pubkey, COALESCE(wallet_sig_alg,'sr25519') AS wallet_sig_alg, wallet_bound_at, wallet_bound_by, COALESCE(archive_qr_payload,'') AS archive_qr_payload, deleted_at, deleted_by, delete_reason, created_at, updated_at
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
        deleted_at: row.try_get("deleted_at").ok(),
        deleted_by: row.try_get("deleted_by").ok(),
        delete_reason: row.try_get("delete_reason").ok(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn ensure_archive_not_deleted(archive: &Archive) -> Result<(), (StatusCode, Json<ApiError>)> {
    if archive.status == "DELETED" || archive.deleted_at.is_some() {
        return Err(err(StatusCode::CONFLICT, 3008, "archive already deleted"));
    }
    Ok(())
}

fn build_archive_delete_payload(
    challenge_id: &str,
    archive_id: &str,
    archive_no: &str,
    admin_pubkey: &str,
    expire_at: i64,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    // 中文注释：wumin 冷钱包按 0x 32 字节公钥识别 CPMS 删除 payload，不能输出裸 hex。
    let admin_pubkey_hex = normalize_pubkey_hex(admin_pubkey).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid admin pubkey",
        )
    })?;
    Ok(format!(
        "CPMS_ARCHIVE_DELETE_V1|{}|{}|{}|{}|{}",
        challenge_id, archive_id, archive_no, admin_pubkey_hex, expire_at
    ))
}

fn build_archive_delete_sign_request(
    challenge_id: &str,
    issued_at: i64,
    expire_at: i64,
    admin_address: &str,
    admin_pubkey: &str,
    payload_hex: &str,
    archive: &Archive,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    // 中文注释：payload 保留真实删除原文；展示层只放人工可核对的档案号、管理员 SS58 和过期时间。
    let admin_pubkey_hex = normalize_pubkey_hex(admin_pubkey).ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid admin pubkey",
        )
    })?;
    let sign_request = serde_json::json!({
        "proto": crate::qr::WUMIN_QR_V1,
        "kind": crate::qr::QrKind::SignRequest.wire(),
        "id": challenge_id,
        "issued_at": issued_at,
        "expires_at": expire_at,
        "body": {
            "address": admin_address,
            "pubkey": admin_pubkey_hex,
            "sig_alg": "sr25519",
            "payload_hex": payload_hex,
            "display": {
                "action": "archive_delete",
                "summary": "确认删除 CPMS 公民档案",
                "fields": [
                    { "key": "archive_no", "label": "档案号", "value": archive.archive_no },
                    { "key": "admin_pubkey", "label": "管理员", "value": admin_address },
                    { "key": "expires_at", "label": "过期时间", "value": expire_at.to_string() }
                ]
            }
        }
    });
    serde_json::to_string(&sign_request)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))
}

fn normalize_pubkey_hex(value: &str) -> Option<String> {
    let bytes = crate::decode_bytes(value)?;
    if bytes.len() != 32 {
        return None;
    }
    Some(format!("0x{}", hex::encode(bytes)))
}

fn payload_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("0x{}", hex::encode(digest))
}

async fn ensure_archive_qr_ready(
    state: &AppState,
    archive: &Archive,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    ensure_required_text(&archive.last_name, "archive qr requires last_name")?;
    ensure_required_text(&archive.first_name, "archive qr requires first_name")?;
    if archive.gender_code != "M" && archive.gender_code != "W" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "archive qr requires gender",
        ));
    }
    let height = archive
        .height_cm
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "archive qr requires height"))?;
    validate_height_cm(height)?;
    let birth_date = validate_birth_date(&archive.birth_date)?;
    ensure_required_text(&archive.passport_no, "archive qr requires passport_no")?;
    validate_required_date(&archive.valid_from, "archive qr requires valid_from")?;
    validate_required_date(&archive.valid_until, "archive qr requires valid_until")?;
    ensure_required_text(&archive.province_code, "archive qr requires province")?;
    ensure_required_text(&archive.city_code, "archive qr requires city")?;
    if archive.citizen_status != dangan::CITIZEN_STATUS_NORMAL {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "archive qr requires normal citizen_status",
        ));
    }
    if !archive.voting_eligible {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "archive qr requires voting_eligible",
        ));
    }
    if !dangan::is_voting_age_at(Utc::now().timestamp(), birth_date) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "archive qr requires age 16",
        ));
    }
    ensure_required_text(
        archive.wallet_address.as_deref().unwrap_or_default(),
        "archive qr requires wallet_address",
    )?;
    ensure_required_text(
        archive.wallet_pubkey.as_deref().unwrap_or_default(),
        "archive qr requires wallet_pubkey",
    )?;
    ensure_archive_qr_materials_ready(state, &archive.archive_id).await
}

async fn ensure_archive_qr_materials_ready(
    state: &AppState,
    archive_id: &str,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT material_type, COUNT(*) AS count
         FROM archive_materials
         WHERE archive_id = $1
           AND deleted_at IS NULL
           AND material_type IN ('PHOTO', 'BIRTH_CERTIFICATE')
         GROUP BY material_type",
    )
    .bind(archive_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query archive materials failed",
        )
    })?;

    let mut has_photo = false;
    let mut has_birth_certificate = false;
    for row in rows {
        let material_type: String = row.get("material_type");
        let count: i64 = row.get("count");
        if material_type == "PHOTO" && count > 0 {
            has_photo = true;
        }
        if material_type == "BIRTH_CERTIFICATE" && count > 0 {
            has_birth_certificate = true;
        }
    }
    if !has_photo {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "archive qr requires photo",
        ));
    }
    if !has_birth_certificate {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "archive qr requires birth_certificate",
        ));
    }
    Ok(())
}

fn ensure_required_text(value: &str, message: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    if value.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, message));
    }
    Ok(())
}

fn validate_required_date(value: &str, message: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    if value.trim().is_empty() || NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").is_err() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, message));
    }
    Ok(())
}

fn validate_birth_date(value: &str) -> Result<NaiveDate, (StatusCode, Json<ApiError>)> {
    let trimmed = value.trim();
    if trimmed.len() != 10 {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"));
    }
    let birth_date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"))?;
    if birth_date >= Utc::now().date_naive() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"));
    }
    Ok(birth_date)
}

fn is_unique_constraint(error: &SqlxError, constraint: &str) -> bool {
    error
        .as_database_error()
        .and_then(|db_error| db_error.constraint())
        .is_some_and(|name| name == constraint)
}

fn validate_height_cm(value: f32) -> Result<(), (StatusCode, Json<ApiError>)> {
    if !value.is_finite() || !(30.0..=260.0).contains(&value) {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid height_cm"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_archive() -> Archive {
        Archive {
            archive_id: "ar_test".to_string(),
            archive_no: "ARCHIVE123".to_string(),
            province_code: "ZS".to_string(),
            city_code: "001".to_string(),
            last_name: "吴".to_string(),
            first_name: "明".to_string(),
            birth_date: "1988-05-20".to_string(),
            gender_code: "M".to_string(),
            height_cm: Some(175.0),
            passport_no: "ZSABCDEFG1".to_string(),
            town_code: "100001".to_string(),
            village_id: "village-1".to_string(),
            address: "测试地址".to_string(),
            status: "ACTIVE".to_string(),
            citizen_status: "NORMAL".to_string(),
            voting_eligible: true,
            valid_from: "2026-05-29".to_string(),
            valid_until: "2036-05-29".to_string(),
            citizen_status_updated_at: 1_779_984_000,
            wallet_address: None,
            wallet_pubkey: None,
            wallet_sig_alg: "sr25519".to_string(),
            wallet_bound_at: None,
            wallet_bound_by: None,
            archive_qr_payload: String::new(),
            deleted_at: None,
            deleted_by: None,
            delete_reason: None,
            created_at: 1_779_984_000,
            updated_at: 1_779_984_000,
        }
    }

    #[test]
    fn archive_delete_payload_uses_canonical_0x_admin_pubkey() {
        let bare_pubkey = "11".repeat(32);
        let payload = build_archive_delete_payload(
            "adc_test",
            "ar_test",
            "ARCHIVE123",
            &bare_pubkey,
            1_779_984_120,
        )
        .unwrap_or_else(|_| panic!("valid pubkey should build payload"));

        assert_eq!(
            payload,
            format!(
                "CPMS_ARCHIVE_DELETE_V1|adc_test|ar_test|ARCHIVE123|0x{}|1779984120",
                bare_pubkey
            )
        );
    }

    #[test]
    fn archive_delete_sign_request_keeps_pubkey_and_displays_ss58() {
        let bare_pubkey = "22".repeat(32);
        let qr = build_archive_delete_sign_request(
            "adc_test",
            1_779_984_000,
            1_779_984_120,
            "5AdminAddress",
            &bare_pubkey,
            "0x7061796c6f6164",
            &sample_archive(),
        )
        .unwrap_or_else(|_| panic!("valid pubkey should build sign request"));
        let json: serde_json::Value =
            serde_json::from_str(&qr).expect("sign request should be valid json");

        assert_eq!(json["body"]["pubkey"], format!("0x{}", bare_pubkey));
        assert_eq!(
            json["body"]["display"]["fields"][1]["value"],
            "5AdminAddress"
        );
    }
}
