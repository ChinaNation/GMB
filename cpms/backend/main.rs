use std::{env, net::SocketAddr, path::Path, sync::Arc};

use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

mod address;
mod authz;
// 中文注释：行政区唯一源是 SFID 维护的 china.sqlite；CPMS 安装包随附其只读拷贝，
// 运行时用 rusqlite 按安装码所属市窄查询镇/村，不在 CPMS 侧维护第二套行政区源。
mod china;
mod dangan;
mod initialize;
mod login;
mod number;
mod qr;
mod rate_limit;
mod ss58;
mod store;
mod super_admin;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    // 登录和二维码场景需要快速本地互斥逻辑，仍保留轻量进程内锁用于并发窗口控制。
    qr_result_gc_lock: Arc<RwLock<()>>,
    rate_limiter: Arc<rate_limit::RateLimiter>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AdminUser {
    user_id: String,
    admin_pubkey: String,
    admin_name: String,
    role: String,
    immutable: bool,
    managed_key_id: Option<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Archive {
    archive_id: String,
    archive_no: String,
    province_code: String,
    city_code: String,
    last_name: String,
    first_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    passport_no: String,
    town_code: String,
    village_id: String,
    address: String,
    status: String,
    citizen_status: String,
    voting_eligible: bool,
    valid_from: String,
    valid_until: String,
    citizen_status_updated_at: i64,
    wallet_address: Option<String>,
    wallet_pubkey: Option<String>,
    wallet_sig_alg: String,
    wallet_bound_at: Option<i64>,
    wallet_bound_by: Option<String>,
    archive_qr_payload: String,
    deleted_at: Option<i64>,
    deleted_by: Option<String>,
    delete_reason: Option<String>,
    created_at: i64,
    updated_at: i64,
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
    /// 中文注释:稳定业务错误码供前端判断;数字 code 只保留为内部分类编号。
    error_code: &'static str,
    message: String,
    trace_id: String,
}

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("db/migrations");

#[tokio::main]
async fn main() {
    let database_url = env::var("CPMS_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://cpms:cpms@127.0.0.1:5433/cpms_dev".to_string());

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("connect postgres failed");

    MIGRATOR.run(&db).await.expect("run migrations failed");
    initialize::ensure_secret_config(&db)
        .await
        .expect("CPMS secret encryption config invalid");

    // 中文注释：已初始化实例启动时按安装码所属市重建地址表，避免旧硬编码镇村残留。
    address::sync_installed_city_address(&db)
        .await
        .expect("sync installed city address failed");
    // 中文注释：启动时先执行一次到期档案硬删除；软删除未满 100 年的号码不会进入回收池。
    dangan::run_due_archive_hard_delete(&db)
        .await
        .expect("run archive hard delete failed");

    let state = AppState {
        db,
        qr_result_gc_lock: Arc::new(RwLock::new(())),
        rate_limiter: Arc::new(rate_limit::RateLimiter::new()),
    };

    let cleanup_store = store::StoreDb::new(state.db.clone());

    // 前端静态文件目录：优先 CPMS_FRONTEND_DIR 环境变量，默认 ./frontend
    let frontend_dir = env::var("CPMS_FRONTEND_DIR").unwrap_or_else(|_| "./frontend".to_string());
    validate_frontend_dir(&frontend_dir);
    let serve_frontend = tower_http::services::ServeDir::new(&frontend_dir).fallback(
        tower_http::services::ServeFile::new(format!("{}/index.html", frontend_dir)),
    );

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .merge(initialize::router())
        .merge(login::router())
        .merge(super_admin::router())
        .merge(dangan::router())
        .merge(address::router())
        .with_state(state.clone())
        .fallback_service(serve_frontend)
        .layer(middleware::from_fn(security_headers));

    let addr: SocketAddr = env::var("CPMS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()
        .expect("invalid CPMS_BIND");

    // 中文注释：后台定时清理过期 session、challenge 和 QR 登录结果，避免 DB 无限膨胀。
    {
        let store = cleanup_store;
        tokio::spawn(async move {
            let interval = tokio::time::Duration::from_secs(300); // 每 5 分钟
            loop {
                tokio::time::sleep(interval).await;
                let now = Utc::now().timestamp();
                store.cleanup_auth_runtime(now).await;
            }
        });
    }
    {
        let db = state.db.clone();
        tokio::spawn(async move {
            let interval = tokio::time::Duration::from_secs(24 * 3600);
            loop {
                tokio::time::sleep(interval).await;
                if let Err(e) = dangan::run_due_archive_hard_delete(&db).await {
                    eprintln!("archive hard delete failed: {e}");
                }
            }
        });
    }

    println!("cpms-backend listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("server failed");
}

async fn health() -> Json<ApiResponse<serde_json::Value>> {
    Json(ok(serde_json::json!({"status": "ok"})))
}

fn validate_frontend_dir(frontend_dir: &str) {
    let index_path = Path::new(frontend_dir).join("index.html");
    if env::var("CPMS_FRONTEND_DIR").is_ok() && !index_path.is_file() {
        panic!(
            "CPMS_FRONTEND_DIR is set but index.html is missing: {}",
            index_path.display()
        );
    }
}

async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "camera=(self), microphone=(), geolocation=(), payment=(), usb=()",
        ),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' blob:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'; form-action 'self'",
        ),
    );
    response
}

async fn find_admin_by_pubkey(
    state: &AppState,
    admin_pubkey: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    // 归一化：去 0x 前缀，小写
    let normalized = admin_pubkey
        .trim()
        .strip_prefix("0x")
        .or_else(|| admin_pubkey.trim().strip_prefix("0X"))
        .unwrap_or(admin_pubkey.trim())
        .to_lowercase();
    let row = sqlx::query(
        "SELECT user_id, admin_pubkey, COALESCE(admin_name, '') AS admin_name, role, immutable, managed_key_id, created_at, updated_at
         FROM admin_users
         WHERE admin_pubkey = $1",
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
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 2002, "admin_pubkey not found"))?;

    Ok(AdminUser {
        user_id: row.get("user_id"),
        admin_pubkey: row.get("admin_pubkey"),
        admin_name: row.get("admin_name"),
        role: row.get("role"),
        immutable: row.get("immutable"),
        managed_key_id: row.get("managed_key_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn find_admin_by_user_id(
    state: &AppState,
    user_id: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT user_id, admin_pubkey, COALESCE(admin_name, '') AS admin_name, role, immutable, managed_key_id, created_at, updated_at
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
        admin_pubkey: row.get("admin_pubkey"),
        admin_name: row.get("admin_name"),
        role: row.get("role"),
        immutable: row.get("immutable"),
        managed_key_id: row.get("managed_key_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn decode_bytes(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();

    let hex_raw = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if let Ok(v) = hex::decode(hex_raw) {
        return Some(v);
    }

    if let Ok(v) = STANDARD.decode(trimmed) {
        return Some(v);
    }

    None
}

async fn write_audit(
    state: &AppState,
    operator_user_id: Option<String>,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    result: &str,
    detail: serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    sqlx::query(
        "INSERT INTO audit_logs (log_id, operator_user_id, action, target_type, target_id, result, detail, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(format!("log_{}", Uuid::new_v4().simple()))
    .bind(operator_user_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(result)
    .bind(sqlx::types::Json(detail))
    .bind(Utc::now().timestamp())
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "write audit failed"))?;
    Ok(())
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
            error_code: cpms_error_code(status, message),
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
}

fn cpms_error_code(status: StatusCode, message: &str) -> &'static str {
    // 中文注释:CPMS 是离线实名系统,错误码只描述本机认证、档案、签发和审计状态。
    match message {
        "missing session cookie" => "CPMS_AUTH_MISSING_SESSION",
        "invalid session" => "CPMS_AUTH_INVALID_SESSION",
        "session expired" => "CPMS_AUTH_SESSION_EXPIRED",
        "permission denied" => "CPMS_AUTH_PERMISSION_DENIED",
        "admin_pubkey not found" => "CPMS_AUTH_ADMIN_NOT_FOUND",
        "admin user not found" => "CPMS_AUTH_ADMIN_NOT_FOUND",
        "challenge not found" => "CPMS_AUTH_CHALLENGE_NOT_FOUND",
        "challenge already consumed" => "CPMS_AUTH_CHALLENGE_CONSUMED",
        "challenge expired" => "CPMS_AUTH_CHALLENGE_EXPIRED",
        "challenge pubkey mismatch" | "challenge session mismatch" => {
            "CPMS_AUTH_CHALLENGE_MISMATCH"
        }
        "signature verify failed" => "CPMS_AUTH_SIGNATURE_VERIFY_FAILED",
        "archive not found" => "CPMS_INTAKE_ARCHIVE_NOT_FOUND",
        "address area not found" => "CPMS_INTAKE_ADDRESS_AREA_NOT_FOUND",
        "archive_no conflict, retry exhausted" => "CPMS_INTAKE_ARCHIVE_DUPLICATED",
        "passport_no conflict, retry exhausted" => "CPMS_INTAKE_PASSPORT_DUPLICATED",
        "passport_no capacity exhausted" => "CPMS_INTAKE_PASSPORT_CAPACITY_EXHAUSTED",
        "invalid passport province_code" | "invalid passport city_code" => {
            "CPMS_INTAKE_PASSPORT_AREA_INVALID"
        }
        "invalid citizen_status" => "CPMS_INTAKE_CITIZEN_STATUS_INVALID",
        "annual status export required" => "CPMS_ANNUAL_STATUS_EXPORT_REQUIRED",
        "annual status export not required" => "CPMS_ANNUAL_STATUS_EXPORT_NOT_REQUIRED",
        "qr encode failed" => "CPMS_ISSUE_QR_GENERATE_FAILED",
        "save print record failed" => "CPMS_AUDIT_WRITE_FAILED",
        "archive wallet required" => "CPMS_ARCHIVE_WALLET_REQUIRED",
        "invalid wallet_address" => "CPMS_ARCHIVE_WALLET_ADDRESS_INVALID",
        "wallet already bound" => "CPMS_ARCHIVE_WALLET_ALREADY_BOUND",
        "archive already deleted" => "CPMS_ARCHIVE_ALREADY_DELETED",
        "delete challenge not found" => "CPMS_ARCHIVE_DELETE_CHALLENGE_NOT_FOUND",
        "delete challenge already consumed" => "CPMS_ARCHIVE_DELETE_CHALLENGE_CONSUMED",
        "delete challenge expired" => "CPMS_ARCHIVE_DELETE_CHALLENGE_EXPIRED",
        "delete challenge mismatch" => "CPMS_ARCHIVE_DELETE_CHALLENGE_MISMATCH",
        "delete signer mismatch" => "CPMS_ARCHIVE_DELETE_SIGNER_MISMATCH",
        "delete signature verify failed" => "CPMS_ARCHIVE_DELETE_SIGNATURE_INVALID",
        "delete payload hash mismatch" => "CPMS_ARCHIVE_DELETE_PAYLOAD_HASH_MISMATCH",
        "material not found" => "CPMS_ARCHIVE_MATERIAL_NOT_FOUND",
        "material file not found" => "CPMS_ARCHIVE_MATERIAL_FILE_NOT_FOUND",
        "material type invalid" => "CPMS_ARCHIVE_MATERIAL_TYPE_INVALID",
        "material mime invalid" => "CPMS_ARCHIVE_MATERIAL_MIME_INVALID",
        "material file required" => "CPMS_ARCHIVE_MATERIAL_FILE_REQUIRED",
        "material file too large" => "CPMS_ARCHIVE_MATERIAL_FILE_TOO_LARGE",
        "material file empty" => "CPMS_ARCHIVE_MATERIAL_FILE_EMPTY",
        "invalid admin role" => "CPMS_ADMIN_ROLE_INVALID",
        "admin name required" => "CPMS_ADMIN_NAME_REQUIRED",
        "admin name too long" => "CPMS_ADMIN_NAME_TOO_LONG",
        "super admin limit reached" => "CPMS_ADMIN_SUPER_ADMIN_LIMIT_REACHED",
        "initial super admin cannot be deleted" => "CPMS_ADMIN_INITIAL_SUPER_ADMIN_IMMUTABLE",
        "admin not found" => "CPMS_ADMIN_NOT_FOUND",
        "admin_pubkey already exists" => "CPMS_ADMIN_PUBKEY_DUPLICATED",
        "too many requests" => "CPMS_RATE_LIMITED",
        _ if status == StatusCode::UNAUTHORIZED => "CPMS_AUTH_UNAUTHORIZED",
        _ if status == StatusCode::FORBIDDEN => "CPMS_AUTH_FORBIDDEN",
        _ if status == StatusCode::BAD_REQUEST => "CPMS_REQUEST_INVALID",
        _ if status == StatusCode::NOT_FOUND => "CPMS_RESOURCE_NOT_FOUND",
        _ if status == StatusCode::CONFLICT => "CPMS_RESOURCE_CONFLICT",
        _ if status == StatusCode::GONE => "CPMS_RESOURCE_EXPIRED",
        _ if status == StatusCode::UNPROCESSABLE_ENTITY => "CPMS_BUSINESS_VALIDATION_FAILED",
        _ if status == StatusCode::SERVICE_UNAVAILABLE => "CPMS_SERVICE_UNAVAILABLE",
        _ if status == StatusCode::LOCKED => "CPMS_RESOURCE_LOCKED",
        _ if status == StatusCode::TOO_MANY_REQUESTS => "CPMS_RATE_LIMITED",
        _ => "CPMS_INTERNAL_ERROR",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        dangan::{sign_archive_payload_with_secret, validate_citizen_status},
        number::archive_no_checksum,
    };
    use schnorrkel::{signing_context, ExpansionMode, MiniSecretKey, PublicKey, Signature};

    #[test]
    fn citizen_status_validation_works() {
        assert!(validate_citizen_status("NORMAL").is_ok());
        assert!(validate_citizen_status("REVOKED").is_ok());
        assert!(validate_citizen_status("DELETED").is_err());
        assert!(validate_citizen_status("UNKNOWN").is_err());
    }

    #[test]
    fn qr_signature_can_be_verified() {
        let payload =
            "sfid-cpms-v1|archive|ABCDEFGHIJKLMNOPQRSTUVWXY2-Z7|NORMAL|true|2026-05-24|2036-05-23|0x1234|0xabcd";
        let secret = [9u8; 32];
        let sig_hex = match sign_archive_payload_with_secret(&secret, payload) {
            Ok(v) => v,
            Err(_) => panic!("sign failed"),
        };
        let sig_bytes = hex::decode(sig_hex.trim_start_matches("0x")).expect("decode signature");
        let sig = Signature::from_bytes(&sig_bytes).expect("signature bytes");

        let mini = MiniSecretKey::from_bytes(&secret).expect("mini secret key");
        let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
        let pk = PublicKey::from_bytes(&keypair.public.to_bytes()).expect("public key bytes");
        let verify_result = pk.verify(
            signing_context(b"substrate").bytes(payload.as_bytes()),
            &sig,
        );
        assert!(verify_result.is_ok());
    }

    #[test]
    fn archive_no_checksum_uses_public_base32_chars() {
        let body = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let check = archive_no_checksum(body);
        let archive_no = format!("{}-{}", body, check);
        assert_eq!(check.len(), 2);
        assert_eq!(archive_no.len(), 29);
        assert_eq!(archive_no.split('-').count(), 2);
    }
}
