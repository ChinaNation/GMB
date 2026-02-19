use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

mod province_codes;

#[derive(Clone)]
struct AppState {
    users: Arc<RwLock<HashMap<String, User>>>,
    tokens: Arc<RwLock<HashMap<String, String>>>,
    refresh_tokens: Arc<RwLock<HashMap<String, String>>>,
    archives: Arc<RwLock<HashMap<String, Archive>>>,
    sequence: Arc<RwLock<HashMap<String, u32>>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct User {
    user_id: String,
    username: String,
    password: String,
    role: String,
    status: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Archive {
    archive_id: String,
    archive_index_no: String,
    province_code: String,
    full_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    passport_no: String,
    status: String,
}

#[derive(Serialize)]
struct ApiResponse<T>
where
    T: Serialize,
{
    code: i32,
    message: String,
    data: Option<T>,
}

#[derive(Serialize)]
struct ApiError {
    code: i32,
    message: String,
    trace_id: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginData {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    user: LoginUser,
}

#[derive(Serialize)]
struct LoginUser {
    user_id: String,
    role: String,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize)]
struct RefreshData {
    access_token: String,
}

#[derive(Deserialize)]
struct CreateArchiveRequest {
    province_code: String,
    full_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    passport_no: String,
}

#[derive(Serialize)]
struct CreateArchiveData {
    archive_id: String,
    archive_index_no: String,
    status: String,
}

#[derive(Deserialize)]
struct ListQuery {
    full_name: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[tokio::main]
async fn main() {
    let mut users = HashMap::new();
    users.insert(
        "superadmin".to_string(),
        User {
            user_id: "u_super_admin".to_string(),
            username: "superadmin".to_string(),
            password: std::env::var("CPMS_SUPERADMIN_PASSWORD")
                .unwrap_or_else(|_| "change-me".to_string()),
            role: "SUPER_ADMIN".to_string(),
            status: "ACTIVE".to_string(),
        },
    );

    let state = AppState {
        users: Arc::new(RwLock::new(users)),
        tokens: Arc::new(RwLock::new(HashMap::new())),
        refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
        archives: Arc::new(RwLock::new(HashMap::new())),
        sequence: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh))
        .route("/api/v1/archives", post(create_archive).get(list_archives))
        .route("/api/v1/archives/:archive_id", get(get_archive))
        .with_state(state);

    let addr: SocketAddr = std::env::var("CPMS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()
        .expect("invalid CPMS_BIND");

    println!("cpms-host-program listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}

async fn health() -> Json<ApiResponse<serde_json::Value>> {
    Json(ok(serde_json::json!({"status": "ok"})))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginData>>, (StatusCode, Json<ApiError>)> {
    let users = state.users.read().await;
    let user = users
        .get(&req.username)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "invalid username or password"))?;

    if user.password != req.password || user.status != "ACTIVE" {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            2001,
            "invalid username or password",
        ));
    }

    let access_token = format!("atk_{}", Uuid::new_v4());
    let refresh_token = format!("rtk_{}", Uuid::new_v4());

    state
        .tokens
        .write()
        .await
        .insert(access_token.clone(), user.user_id.clone());
    state
        .refresh_tokens
        .write()
        .await
        .insert(refresh_token.clone(), user.user_id.clone());

    Ok(Json(ok(LoginData {
        access_token,
        refresh_token,
        expires_in: 1800,
        user: LoginUser {
            user_id: user.user_id.clone(),
            role: user.role.clone(),
        },
    })))
}

async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<RefreshData>>, (StatusCode, Json<ApiError>)> {
    let refresh_tokens = state.refresh_tokens.read().await;
    let user_id = refresh_tokens
        .get(&req.refresh_token)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "invalid refresh token"))?
        .clone();
    drop(refresh_tokens);

    let access_token = format!("atk_{}", Uuid::new_v4());
    state.tokens.write().await.insert(access_token.clone(), user_id);

    Ok(Json(ok(RefreshData { access_token })))
}

async fn create_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateArchiveRequest>,
) -> Result<Json<ApiResponse<CreateArchiveData>>, (StatusCode, Json<ApiError>)> {
    let _user_id = require_auth(&state, &headers).await?;

    if !province_codes::is_valid_province_code(&req.province_code) {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid province_code"));
    }
    if req.gender_code != "M" && req.gender_code != "W" {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid gender_code"));
    }
    let birth_date = NaiveDate::parse_from_str(&req.birth_date, "%Y-%m-%d")
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"))?;

    let birth_yyyymmdd = birth_date.format("%Y%m%d").to_string();
    let seq_key = format!("{}|{}|{}", req.province_code, req.gender_code, birth_yyyymmdd);

    let next_seq = {
        let mut seq = state.sequence.write().await;
        let current = seq.entry(seq_key).or_insert(1);
        let value = *current;
        *current += 1;
        value
    };

    let archive_index_no = format!(
        "{}{}{}{:06}",
        req.province_code, req.gender_code, birth_yyyymmdd, next_seq
    );

    let archive = Archive {
        archive_id: format!("ar_{}", Uuid::new_v4().simple()),
        archive_index_no: archive_index_no.clone(),
        province_code: req.province_code,
        full_name: req.full_name,
        birth_date: req.birth_date,
        gender_code: req.gender_code,
        height_cm: req.height_cm,
        passport_no: req.passport_no,
        status: "ACTIVE".to_string(),
    };

    state
        .archives
        .write()
        .await
        .insert(archive.archive_id.clone(), archive.clone());

    Ok(Json(ok(CreateArchiveData {
        archive_id: archive.archive_id,
        archive_index_no,
        status: archive.status,
    })))
}

async fn list_archives(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let _user_id = require_auth(&state, &headers).await?;

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
    let _user_id = require_auth(&state, &headers).await?;

    let archives = state.archives.read().await;
    let archive = archives
        .get(&archive_id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?
        .clone();

    Ok(Json(ok(archive)))
}

async fn require_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "missing bearer token"))?;

    let tokens = state.tokens.read().await;
    let user_id = tokens
        .get(token)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "invalid token"))?
        .clone();

    Ok(user_id)
}

fn ok<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: Some(data),
    }
}

fn err(status: StatusCode, code: i32, message: &str) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            code,
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
}
