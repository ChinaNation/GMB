use std::{env, net::SocketAddr, sync::Arc};

use axum::{http::StatusCode, routing::get, Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use schnorrkel::{signing_context, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

mod authz;
mod dangan;
mod initialize;
mod login;
mod operator_admin;
mod rsa_blind_client;
mod super_admin;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    // 登录和二维码场景需要快速本地互斥逻辑，仍保留轻量进程内锁用于并发窗口控制。
    qr_result_gc_lock: Arc<RwLock<()>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AdminUser {
    user_id: String,
    admin_pubkey: String,
    role: String,
    status: String,
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
    full_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    passport_no: String,
    status: String,
    citizen_status: String,
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

    let state = AppState {
        db,
        qr_result_gc_lock: Arc::new(RwLock::new(())),
    };

    let cleanup_db = state.db.clone();
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .merge(initialize::router())
        .merge(login::router())
        .merge(super_admin::router())
        .merge(operator_admin::router())
        .with_state(state);

    let addr: SocketAddr = env::var("CPMS_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("invalid CPMS_BIND");

    // 中文注释：后台定时清理过期 session、challenge 和 QR 登录结果，避免 DB 无限膨胀。
    {
        let db = cleanup_db;
        tokio::spawn(async move {
            let interval = tokio::time::Duration::from_secs(300); // 每 5 分钟
            loop {
                tokio::time::sleep(interval).await;
                let now = Utc::now().timestamp();
                let _ = sqlx::query("DELETE FROM sessions WHERE expires_at < $1")
                    .bind(now)
                    .execute(&db)
                    .await;
                let _ = sqlx::query("DELETE FROM login_challenges WHERE expire_at < $1")
                    .bind(now)
                    .execute(&db)
                    .await;
                // qr_login_results 保留 10 分钟（供轮询查询）
                let cutoff = now - 600;
                let _ = sqlx::query("DELETE FROM qr_login_results WHERE created_at < $1")
                    .bind(cutoff)
                    .execute(&db)
                    .await;
            }
        });
    }

    println!("cpms-backend listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}

async fn health() -> Json<ApiResponse<serde_json::Value>> {
    Json(ok(serde_json::json!({"status": "ok"})))
}

async fn find_admin_by_pubkey(
    state: &AppState,
    admin_pubkey: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT user_id, admin_pubkey, role, status, immutable, managed_key_id, created_at, updated_at
         FROM admin_users
         WHERE admin_pubkey = $1",
    )
    .bind(admin_pubkey)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query admin failed"))?
    .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2002, "admin_pubkey not found"))?;

    Ok(AdminUser {
        user_id: row.get("user_id"),
        admin_pubkey: row.get("admin_pubkey"),
        role: row.get("role"),
        status: row.get("status"),
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
        "SELECT user_id, admin_pubkey, role, status, immutable, managed_key_id, created_at, updated_at
         FROM admin_users
         WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query admin failed"))?
    .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2002, "admin user not found"))?;

    Ok(AdminUser {
        user_id: row.get("user_id"),
        admin_pubkey: row.get("admin_pubkey"),
        role: row.get("role"),
        status: row.get("status"),
        immutable: row.get("immutable"),
        managed_key_id: row.get("managed_key_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn validate_admin_status(status: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    match status {
        "ACTIVE" | "DISABLED" => Ok(()),
        _ => Err(err(StatusCode::BAD_REQUEST, 1001, "invalid status")),
    }
}

fn verify_signature_with_context(
    admin_pubkey: &str,
    payload: &str,
    signature: &str,
    context: &[u8],
) -> Result<(), &'static str> {
    let pubkey_bytes = decode_bytes(admin_pubkey).ok_or("invalid admin_pubkey encoding")?;
    if pubkey_bytes.len() != 32 {
        return Err("invalid admin_pubkey length");
    }
    let sig_bytes = decode_bytes(signature).ok_or("invalid signature encoding")?;
    if sig_bytes.len() != 64 {
        return Err("invalid signature length");
    }

    let pk = PublicKey::from_bytes(&pubkey_bytes).map_err(|_| "invalid sr25519 public key")?;
    let sig = Signature::from_bytes(&sig_bytes).map_err(|_| "invalid sr25519 signature")?;

    let ctx = signing_context(context);
    pk.verify(ctx.bytes(payload.as_bytes()), &sig)
        .map_err(|_| "sr25519 verify failed")
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
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        dangan::{archive_checksum_digit, sign_qr_payload_with_secret, validate_citizen_status},
        login::verify_challenge_signature,
    };
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use schnorrkel::{signing_context, ExpansionMode, MiniSecretKey, PublicKey, Signature};

    #[test]
    fn verify_signature_accepts_hex_inputs() {
        let payload = "cpms-admin-auth-v1|chl_x|pub_x|nonce_x|1234567890";
        let (pubkey_hex, sig_hex) = build_signed_payload(payload);
        assert!(verify_challenge_signature(&pubkey_hex, payload, &sig_hex).is_ok());
    }

    #[test]
    fn verify_signature_accepts_base64_inputs() {
        let payload = "cpms-admin-auth-v1|chl_y|pub_y|nonce_y|1234567890";
        let (pubkey_hex, sig_hex) = build_signed_payload(payload);
        let pubkey_raw = hex::decode(pubkey_hex).expect("hex pubkey decode");
        let sig_raw = hex::decode(sig_hex).expect("hex signature decode");
        let pubkey_b64 = STANDARD.encode(pubkey_raw);
        let sig_b64 = STANDARD.encode(sig_raw);
        assert!(verify_challenge_signature(&pubkey_b64, payload, &sig_b64).is_ok());
    }

    #[test]
    fn verify_signature_rejects_tampered_payload() {
        let payload = "cpms-admin-auth-v1|chl_z|pub_z|nonce_z|1234567890";
        let (pubkey_hex, sig_hex) = build_signed_payload(payload);
        let tampered = "cpms-admin-auth-v1|chl_z|pub_z|nonce_z|1234567891";
        let result = verify_challenge_signature(&pubkey_hex, tampered, &sig_hex);
        assert!(result.is_err());
    }

    #[test]
    fn verify_signature_rejects_invalid_encoding() {
        let payload = "cpms-admin-auth-v1|chl_w|pub_w|nonce_w|1234567890";
        let result = verify_challenge_signature("not-a-key", payload, "not-a-signature");
        assert!(result.is_err());
    }

    #[test]
    fn citizen_status_validation_works() {
        assert!(validate_citizen_status("NORMAL").is_ok());
        assert!(validate_citizen_status("ABNORMAL").is_ok());
        assert!(validate_citizen_status("UNKNOWN").is_err());
    }

    #[test]
    fn qr_signature_can_be_verified() {
        let payload = "cpms-qr-v1|site|archive_no|NORMAL|true|1700000000|qr_1";
        let secret = [9u8; 32];
        let sig_hex = match sign_qr_payload_with_secret(&secret, payload) {
            Ok(v) => v,
            Err(_) => panic!("sign failed"),
        };
        let sig_bytes = hex::decode(sig_hex).expect("decode signature");
        let sig = Signature::from_bytes(&sig_bytes).expect("signature bytes");

        let mini = MiniSecretKey::from_bytes(&secret).expect("mini secret key");
        let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
        let pk = PublicKey::from_bytes(&keypair.public.to_bytes()).expect("public key bytes");
        let verify_result = pk.verify(
            signing_context(b"CPMS-QR-SIGN-V1").bytes(payload.as_bytes()),
            &sig,
        );
        assert!(verify_result.is_ok());
    }

    #[test]
    fn archive_no_v3_format_is_stable() {
        let province = "GD";
        let city = "001";
        let random9 = "123456789";
        let created_date = "20260227";
        let check = archive_checksum_digit(province, city, random9, created_date);
        let archive_no = format!("{}{}{}{}{}", province, city, check, random9, created_date);
        assert_eq!(archive_no.len(), 23);
        assert!(archive_no.starts_with("GD001"));
        assert!(archive_no.ends_with("20260227"));
    }

    fn build_signed_payload(payload: &str) -> (String, String) {
        let mini = MiniSecretKey::from_bytes(&[7u8; 32]).expect("mini secret key");
        let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
        let sig = keypair.sign(signing_context(b"CPMS-ADMIN-AUTH-V1").bytes(payload.as_bytes()));
        (
            hex::encode(keypair.public.to_bytes()),
            hex::encode(sig.to_bytes()),
        )
    }
}
