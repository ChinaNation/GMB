use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};

use axum::{http::StatusCode, routing::get, Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use schnorrkel::{signing_context, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

mod authz;
mod dangan;
mod initialize;
mod login;
mod operator_admin;
mod super_admin;
use initialize::{init_super_admin_users, load_or_init_install_data, QrSignKeyRuntime};

#[derive(Clone)]
struct AppState {
    runtime_store_path: PathBuf,
    install: Arc<RwLock<InstallRuntime>>,
    admin_users: Arc<RwLock<HashMap<String, AdminUser>>>,
    sessions: Arc<RwLock<HashMap<String, login::Session>>>,
    login_challenges: Arc<RwLock<HashMap<String, login::LoginChallenge>>>,
    qr_login_results: Arc<RwLock<HashMap<String, login::QrLoginResult>>>,
    archives: Arc<RwLock<HashMap<String, Archive>>>,
    sequence: Arc<RwLock<HashMap<String, u32>>>,
    qr_print_records: Arc<RwLock<Vec<QrPrintRecord>>>,
    audit_logs: Arc<RwLock<Vec<AuditLog>>>,
}

#[derive(Clone)]
struct InstallRuntime {
    file_path: PathBuf,
    site_sfid: Option<String>,
    qr_sign_keys: Vec<QrSignKeyRuntime>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AdminUser {
    user_id: String,
    admin_pubkey: String,
    role: String,
    status: String,
    immutable: bool,
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
}

#[derive(Clone, Serialize, Deserialize)]
struct AuditLog {
    log_id: String,
    operator_user_id: Option<String>,
    action: String,
    target_type: String,
    target_id: Option<String>,
    result: String,
    detail: serde_json::Value,
    created_at: i64,
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

#[derive(Clone, Serialize, Deserialize)]
struct QrPrintRecord {
    print_id: String,
    archive_id: String,
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
    printed_at: i64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct RuntimeStore {
    admin_users: HashMap<String, AdminUser>,
    sessions: HashMap<String, login::Session>,
    login_challenges: HashMap<String, login::LoginChallenge>,
    qr_login_results: HashMap<String, login::QrLoginResult>,
    archives: HashMap<String, Archive>,
    sequence: HashMap<String, u32>,
    qr_print_records: Vec<QrPrintRecord>,
    audit_logs: Vec<AuditLog>,
}

#[tokio::main]
async fn main() {
    let install = load_or_init_install_data().unwrap_or_else(|reason| panic!("{reason}"));
    if install.was_created {
        println!(
            "cpms-backend install bootstrap initialized at {}",
            install.file_path.display()
        );
    }
    if install.data.is_none() {
        println!("cpms-backend waiting for SFID install qr initialization");
    }

    let super_admins = install
        .data
        .as_ref()
        .map(|d| d.super_admins.clone())
        .unwrap_or_default();
    let site_sfid = install.data.as_ref().map(|d| d.site_sfid.clone());
    let runtime_store_path = PathBuf::from(
        std::env::var("CPMS_RUNTIME_STORE_FILE")
            .unwrap_or_else(|_| "runtime/cpms_runtime_store.json".to_string()),
    );
    let mut runtime_store = match load_runtime_store(&runtime_store_path) {
        Ok(store) => store,
        Err(reason) => {
            eprintln!("failed to load runtime store: {reason}");
            RuntimeStore::default()
        }
    };
    if runtime_store.admin_users.is_empty() {
        runtime_store.admin_users = init_super_admin_users(&super_admins);
    } else {
        for (k, v) in init_super_admin_users(&super_admins) {
            runtime_store.admin_users.entry(k).or_insert(v);
        }
    }

    let state = AppState {
        runtime_store_path,
        install: Arc::new(RwLock::new(InstallRuntime {
            file_path: install.file_path,
            site_sfid,
            qr_sign_keys: install.qr_sign_keys,
        })),
        admin_users: Arc::new(RwLock::new(runtime_store.admin_users)),
        sessions: Arc::new(RwLock::new(runtime_store.sessions)),
        login_challenges: Arc::new(RwLock::new(runtime_store.login_challenges)),
        qr_login_results: Arc::new(RwLock::new(runtime_store.qr_login_results)),
        archives: Arc::new(RwLock::new(runtime_store.archives)),
        sequence: Arc::new(RwLock::new(runtime_store.sequence)),
        qr_print_records: Arc::new(RwLock::new(runtime_store.qr_print_records)),
        audit_logs: Arc::new(RwLock::new(runtime_store.audit_logs)),
    };
    if let Err(reason) = persist_runtime_store(&state).await {
        eprintln!("failed to persist runtime store on startup: {reason}");
    }

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .merge(initialize::router())
        .merge(login::router())
        .merge(super_admin::router())
        .merge(operator_admin::router())
        .with_state(state);

    let addr: SocketAddr = std::env::var("CPMS_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("invalid CPMS_BIND");

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
    let users = state.admin_users.read().await;
    users
        .values()
        .find(|u| u.admin_pubkey == admin_pubkey)
        .cloned()
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2002, "admin_pubkey not found"))
}

async fn find_admin_by_user_id(
    state: &AppState,
    user_id: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    let users = state.admin_users.read().await;
    users
        .get(user_id)
        .cloned()
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2002, "admin user not found"))
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

fn load_runtime_store(path: &FsPath) -> Result<RuntimeStore, String> {
    if !path.exists() {
        return Ok(RuntimeStore::default());
    }
    let raw =
        fs::read(path).map_err(|e| format!("read runtime store {} failed: {e}", path.display()))?;
    if raw.is_empty() {
        return Ok(RuntimeStore::default());
    }
    serde_json::from_slice::<RuntimeStore>(&raw)
        .map_err(|e| format!("parse runtime store {} failed: {e}", path.display()))
}

async fn snapshot_runtime_store(state: &AppState) -> RuntimeStore {
    RuntimeStore {
        admin_users: state.admin_users.read().await.clone(),
        sessions: state.sessions.read().await.clone(),
        login_challenges: state.login_challenges.read().await.clone(),
        qr_login_results: state.qr_login_results.read().await.clone(),
        archives: state.archives.read().await.clone(),
        sequence: state.sequence.read().await.clone(),
        qr_print_records: state.qr_print_records.read().await.clone(),
        audit_logs: state.audit_logs.read().await.clone(),
    }
}

fn persist_runtime_store_to_path(path: &FsPath, store: &RuntimeStore) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("runtime store path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("create runtime store dir {} failed: {e}", parent.display()))?;
    let bytes = serde_json::to_vec_pretty(store)
        .map_err(|e| format!("serialize runtime store failed: {e}"))?;
    let tmp_path = parent.join(format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("cpms_runtime_store.json")
    ));
    fs::write(&tmp_path, bytes)
        .map_err(|e| format!("write runtime store tmp {} failed: {e}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).map_err(|e| {
        format!(
            "rename runtime store {} -> {} failed: {e}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

async fn persist_runtime_store(state: &AppState) -> Result<(), String> {
    let snapshot = snapshot_runtime_store(state).await;
    persist_runtime_store_to_path(&state.runtime_store_path, &snapshot)
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
    let log = AuditLog {
        log_id: format!("log_{}", Uuid::new_v4().simple()),
        operator_user_id,
        action: action.to_string(),
        target_type: target_type.to_string(),
        target_id,
        result: result.to_string(),
        detail,
        created_at: Utc::now().timestamp(),
    };
    state.audit_logs.write().await.push(log);
    persist_runtime_store(state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;
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
