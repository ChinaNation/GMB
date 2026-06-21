use std::{env, net::SocketAddr, path::Path, sync::Arc};

use axum::{
    body::Body,
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::RwLock;

mod address;
mod admins;
mod authz;
mod common;
mod dangan;
mod initialize;
mod login;
mod number;
mod qr;
mod store;

// 中文注释：跨模块共享的响应封装、DTO、helper 统一在 common/（与前端 common/ 对齐）。
use common::{ok, ApiResponse};

#[derive(Clone)]
struct AppState {
    db: PgPool,
    // 登录和二维码场景需要快速本地互斥逻辑，仍保留轻量进程内锁用于并发窗口控制。
    qr_result_gc_lock: Arc<RwLock<()>>,
    rate_limiter: Arc<common::rate_limit::RateLimiter>,
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

    // 中文注释：已初始化实例启动时按安装码所属市重建地址表，避免旧硬编码地址数据残留。
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
        rate_limiter: Arc::new(common::rate_limit::RateLimiter::new()),
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
        .merge(admins::router())
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
    let is_api_path = req.uri().path().starts_with("/api/");
    let mut response = next.run(req).await;
    let html_fallback = response.status() == StatusCode::OK
        && response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/html"));
    if is_api_path && html_fallback {
        // 中文注释：API 未命中时不能落到前端 index.html，否则前端会把 HTML 当 JSON 解析。
        response = common::err(StatusCode::NOT_FOUND, 404, "api route not found").into_response();
    }
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
