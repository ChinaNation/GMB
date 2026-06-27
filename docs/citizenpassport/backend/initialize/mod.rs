use std::{env, net::SocketAddr};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
use schnorrkel::MiniSecretKey;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{
    common::{err, ok, rate_limit, write_audit, ApiError, ApiResponse},
    AppState,
};

type Blake2b256 = Blake2b<U32>;

const FIXED_ADMIN_COUNT: usize = 1;
const ARCHIVE_SIGN_KEY_ID: &str = "ARCHIVE";
const INSTALL_SECRET_KEY_ID: &str = "INSTALL_SECRET";

// ── 本机密钥加密存储 ─────────────────────────────────────────────────────
// 中文注释：使用环境变量 CPMS_KEY_ENCRYPT_SECRET（32 字节 hex）作为主密钥，
// 对 ARCHIVE 签名私钥和 install_secret 做 AES-GCM 加密后存入 DB；缺失主密钥时拒绝初始化。

fn master_encrypt_key() -> Result<[u8; 32], String> {
    let hex_str = env::var("CPMS_KEY_ENCRYPT_SECRET")
        .map_err(|_| "CPMS_KEY_ENCRYPT_SECRET not set".to_string())?;
    let bytes = hex::decode(hex_str.trim().trim_start_matches("0x"))
        .map_err(|_| "CPMS_KEY_ENCRYPT_SECRET must be 32-byte hex".to_string())?;
    if bytes.len() != 32 {
        return Err("CPMS_KEY_ENCRYPT_SECRET must be 32-byte hex".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn secret_cipher(key_id: &str) -> Result<Aes256Gcm, String> {
    let master = master_encrypt_key()?;
    let mut hasher = Blake2b256::new();
    hasher.update(master);
    hasher.update(key_id.as_bytes());
    let key = hasher.finalize();
    Aes256Gcm::new_from_slice(&key).map_err(|_| "invalid derived secret key".to_string())
}

fn encrypt_secret(key_id: &str, secret_bytes: &[u8; 32]) -> Result<String, String> {
    let cipher = secret_cipher(key_id)?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), secret_bytes.as_slice())
        .map_err(|_| "encrypt CPMS secret failed".to_string())?;
    Ok(format!(
        "enc:gcm:{}:{}",
        hex::encode(nonce_bytes),
        hex::encode(ciphertext)
    ))
}

fn decrypt_secret(key_id: &str, stored: &str) -> Option<Vec<u8>> {
    let rest = stored.strip_prefix("enc:gcm:")?;
    let (nonce_hex, cipher_hex) = rest.split_once(':')?;
    let nonce = hex::decode(nonce_hex).ok()?;
    if nonce.len() != 12 {
        return None;
    }
    let ciphertext = hex::decode(cipher_hex).ok()?;
    let cipher = secret_cipher(key_id).ok()?;
    cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_slice())
        .ok()
}

fn encrypt_install_secret(install_secret: &str) -> Result<String, String> {
    let bytes = decode_32_byte_hex(install_secret)?;
    encrypt_secret(INSTALL_SECRET_KEY_ID, &bytes)
}

fn decrypt_install_secret(stored: &str) -> Option<String> {
    let bytes = decrypt_secret(INSTALL_SECRET_KEY_ID, stored)?;
    if bytes.len() != 32 {
        return None;
    }
    Some(format!("0x{}", hex::encode(bytes)))
}

fn decode_32_byte_hex(input: &str) -> Result<[u8; 32], String> {
    let raw = input
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    let bytes = hex::decode(raw).map_err(|_| "install_secret must be hex".to_string())?;
    if bytes.len() != 32 {
        return Err("install_secret must be 32 bytes".to_string());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[derive(Clone)]
pub(crate) struct QrSignKeyRuntime {
    pub(crate) key_id: String,
    // 中文注释：purpose 是数据库审计字段；运行时只取 ARCHIVE + ACTIVE 的密钥。
    #[allow(dead_code)]
    pub(crate) purpose: String,
    pub(crate) status: String,
    pub(crate) pubkey: String,
    pub(crate) secret_bytes: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct CpmsInstallRuntime {
    pub(crate) cid_number: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) install_secret: String,
    pub(crate) cpms_pubkey: String,
}

/// CID_CPMS_V1 / INSTALL 安装码载荷。
#[derive(Deserialize)]
struct CidInstallQrPayload {
    proto: String,
    r#type: String,
    cid_number: String,
    province_name: String,
    city_name: String,
    install_secret: String,
    sig: String,
}

#[derive(Deserialize)]
struct InstallInitializeRequest {
    cid_init_qr_content: String,
}

#[derive(Serialize)]
struct InstallInitializeData {
    cid_number: String,
}

#[derive(Serialize)]
struct InstallStatusData {
    initialized: bool,
    cid_number: Option<String>,
    province_code: Option<String>,
    city_code: Option<String>,
    province_name: Option<String>,
    city_name: Option<String>,
    admins_bound_count: usize,
    archive_signing_ready: bool,
    cpms_pubkey: Option<String>,
}

#[derive(Deserialize)]
struct BindAdminRequest {
    admin_account: String,
}

#[derive(Serialize)]
struct BindAdminData {
    user_id: String,
    admin_account: String,
    user_group: String,
    managed_key_id: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/install/status", get(install_status))
        .route("/api/v1/install/initialize", post(initialize_install))
        .route(
            "/api/v1/install/admins/bind",
            post(bind_admins_from_citizenwallet),
        )
}

pub(crate) async fn ensure_secret_config(db: &sqlx::PgPool) -> Result<(), String> {
    let has_stored_secret: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM system_install
            WHERE id = 1 AND install_secret IS NOT NULL AND install_secret <> ''
            UNION ALL
            SELECT 1 FROM qr_sign_keys
            LIMIT 1
         )",
    )
    .fetch_one(db)
    .await
    .map_err(|e| format!("query CPMS secret state failed: {e}"))?;
    if has_stored_secret {
        master_encrypt_key()?;
        verify_stored_secret_materials(db).await?;
    }
    Ok(())
}

async fn verify_stored_secret_materials(db: &sqlx::PgPool) -> Result<(), String> {
    if let Some(stored) = sqlx::query_scalar::<_, String>(
        "SELECT install_secret
         FROM system_install
         WHERE id = 1 AND install_secret IS NOT NULL AND install_secret <> ''",
    )
    .fetch_optional(db)
    .await
    .map_err(|e| format!("query install_secret failed: {e}"))?
    {
        decrypt_install_secret(&stored)
            .ok_or_else(|| "decrypt install_secret failed".to_string())?;
    }

    let rows = sqlx::query("SELECT key_id, secret FROM qr_sign_keys")
        .fetch_all(db)
        .await
        .map_err(|e| format!("query qr_sign_keys failed: {e}"))?;
    for row in rows {
        let key_id: String = row.get("key_id");
        let secret: String = row.get("secret");
        let bytes = decrypt_secret(&key_id, &secret)
            .ok_or_else(|| format!("decrypt {key_id} key failed"))?;
        if bytes.len() != 32 {
            return Err(format!("{key_id} key secret length invalid"));
        }
    }
    Ok(())
}

async fn install_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<InstallStatusData>>, (StatusCode, Json<ApiError>)> {
    let install_row = sqlx::query(
        "SELECT cid_number, province_code, city_code, province_name, city_name, cpms_pubkey
         FROM system_install
         WHERE id = 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query install failed",
        )
    })?;

    let cid_number = install_row
        .as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("cid_number").ok().flatten());
    let province_code = install_row.as_ref().and_then(|r| {
        r.try_get::<Option<String>, _>("province_code")
            .ok()
            .flatten()
    });
    let city_code = install_row
        .as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("city_code").ok().flatten());
    let province_name = install_row.as_ref().and_then(|r| {
        r.try_get::<Option<String>, _>("province_name")
            .ok()
            .flatten()
    });
    let city_name = install_row
        .as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("city_name").ok().flatten());
    let cpms_pubkey = install_row
        .as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("cpms_pubkey").ok().flatten());

    let keys = load_qr_sign_keys(&state).await?;
    let archive_signing_ready = keys
        .iter()
        .any(|k| k.key_id == ARCHIVE_SIGN_KEY_ID && k.status == "ACTIVE");

    let admins_bound_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE user_group = 'admins'")
            .fetch_one(&state.db)
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "query admins failed",
                )
            })?;

    Ok(Json(ok(InstallStatusData {
        initialized: cid_number.is_some() && archive_signing_ready,
        cid_number,
        province_code,
        city_code,
        province_name,
        city_name,
        admins_bound_count: admins_bound_count as usize,
        archive_signing_ready,
        cpms_pubkey,
    })))
}

/// 处理 CID 签发的 INSTALL 安装码。
///
/// 中文注释：第 2 步不再生成中间注册码。CPMS 安装时一次性写入
/// `cid_number / province_name / city_name / install_secret / sig`，同时生成本机 ARCHIVE 签名密钥。
/// 已初始化实例如需换绑，按当前任务口径直接清库重装，不走旧数据兼容分支。
async fn initialize_install(
    State(state): State<AppState>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<InstallInitializeRequest>,
) -> Result<Json<ApiResponse<InstallInitializeData>>, (StatusCode, Json<ApiError>)> {
    rate_limit::check(&state, client_addr, &headers, "install_initialize", 5, 60).await?;

    if req.cid_init_qr_content.trim().is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "invalid cid_init_qr_content",
        ));
    }

    let qr_payload = parse_cid_install_qr_content(&req.cid_init_qr_content)
        .map_err(|reason| err(StatusCode::BAD_REQUEST, 4002, &reason))?;
    validate_cid_install_qr(&qr_payload)
        .map_err(|reason| err(StatusCode::BAD_REQUEST, 4002, &reason))?;
    let (province_code, city_code) = parse_cid_area_codes(qr_payload.cid_number.as_str());
    let province_code = province_code
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 4002, "invalid install cid_number"))?;
    let city_code = city_code
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 4002, "invalid install cid_number"))?;

    let install_secret_stored = encrypt_install_secret(qr_payload.install_secret.as_str())
        .map_err(|reason| err(StatusCode::SERVICE_UNAVAILABLE, 5003, &reason))?;
    let (cpms_pubkey, secret_raw) = generate_sr25519_keypair_raw();
    let secret_stored = encrypt_secret(ARCHIVE_SIGN_KEY_ID, &secret_raw)
        .map_err(|reason| err(StatusCode::SERVICE_UNAVAILABLE, 5003, &reason))?;
    let now_ts = Utc::now().timestamp();

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let existing_cid: Option<String> =
        sqlx::query_scalar("SELECT cid_number FROM system_install WHERE id = 1 FOR UPDATE")
            .fetch_optional(tx.as_mut())
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "lock install failed",
                )
            })?
            .flatten();

    if existing_cid
        .as_ref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
    {
        return Err(err(
            StatusCode::CONFLICT,
            4001,
            "cpms already initialized, reset database before reinstall",
        ));
    }

    sqlx::query(
        "INSERT INTO system_install (
             id, cid_number, install_secret, install_secret_hash, install_sig,
             province_code, city_code, province_name, city_name, cpms_pubkey, initialized_at
         )
         VALUES (1, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (id) DO UPDATE SET
             cid_number = EXCLUDED.cid_number,
             install_secret = EXCLUDED.install_secret,
             install_secret_hash = EXCLUDED.install_secret_hash,
             install_sig = EXCLUDED.install_sig,
             province_code = EXCLUDED.province_code,
             city_code = EXCLUDED.city_code,
             province_name = EXCLUDED.province_name,
             city_name = EXCLUDED.city_name,
             cpms_pubkey = EXCLUDED.cpms_pubkey,
             initialized_at = EXCLUDED.initialized_at",
    )
    .bind(qr_payload.cid_number.trim())
    .bind(&install_secret_stored)
    .bind(install_secret_hash(qr_payload.install_secret.as_str()))
    .bind(qr_payload.sig.trim())
    .bind(&province_code)
    .bind(&city_code)
    .bind(qr_payload.province_name.trim())
    .bind(qr_payload.city_name.trim())
    .bind(&cpms_pubkey)
    .bind(now_ts)
    .execute(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "save install state failed",
        )
    })?;

    sqlx::query("DELETE FROM qr_sign_keys")
        .execute(tx.as_mut())
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "clear qr keys failed",
            )
        })?;

    sqlx::query(
        "INSERT INTO qr_sign_keys (key_id, purpose, status, pubkey, secret, created_at, updated_at)
         VALUES ($1, 'ARCHIVE', 'ACTIVE', $2, $3, $4, $5)",
    )
    .bind(ARCHIVE_SIGN_KEY_ID)
    .bind(&cpms_pubkey)
    .bind(&secret_stored)
    .bind(now_ts)
    .bind(now_ts)
    .execute(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "insert archive key failed",
        )
    })?;

    crate::address::sync_city_address_by_cid_in_tx(tx.as_mut(), qr_payload.cid_number.as_str())
        .await
        .map_err(|reason| err(StatusCode::BAD_REQUEST, 4002, &reason))?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    write_audit(
        &state,
        None,
        "INSTALL_INITIALIZE",
        "CPMS_INSTALL",
        Some(qr_payload.cid_number.clone()),
        "SUCCESS",
        serde_json::json!({
            "cid_number": qr_payload.cid_number,
            "cpms_pubkey": cpms_pubkey,
        }),
    )
    .await?;

    Ok(Json(ok(InstallInitializeData {
        cid_number: qr_payload.cid_number,
    })))
}

async fn bind_admins_from_citizenwallet(
    State(state): State<AppState>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<BindAdminRequest>,
) -> Result<Json<ApiResponse<BindAdminData>>, (StatusCode, Json<ApiError>)> {
    rate_limit::check(&state, client_addr, &headers, "install_admins_bind", 5, 60).await?;

    let _install = load_cpms_install_runtime(&state).await?;
    let raw_input = req.admin_account.trim().to_string();
    if raw_input.is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_account is required",
        ));
    }
    let admin_account = normalize_admin_account(&raw_input)?;
    let user_id = "u_admins_01".to_string();
    let now_ts = Utc::now().timestamp();

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE user_group = 'admins'")
            .fetch_one(tx.as_mut())
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "count admins failed",
                )
            })?;
    if count as usize >= FIXED_ADMIN_COUNT {
        return Err(err(StatusCode::CONFLICT, 4004, "admin already bound"));
    }

    let pubkey_exists: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM admin_users WHERE admin_account = $1 LIMIT 1")
            .bind(&admin_account)
            .fetch_optional(tx.as_mut())
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "check pubkey failed",
                )
            })?;
    if pubkey_exists.is_some() {
        return Err(err(
            StatusCode::CONFLICT,
            4004,
            "admin_account already bound",
        ));
    }

    sqlx::query(
        "INSERT INTO admin_users (user_id, admin_account, user_group, immutable, managed_key_id, created_at, updated_at)
         VALUES ($1, $2, 'admins', TRUE, $3, $4, $5)",
    )
    .bind(&user_id)
    .bind(&admin_account)
    .bind(ARCHIVE_SIGN_KEY_ID)
    .bind(now_ts)
    .bind(now_ts)
    .execute(tx.as_mut())
    .await
    .map_err(|_| err(StatusCode::CONFLICT, 4004, "bind admin failed"))?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    write_audit(
        &state,
        Some(user_id.clone()),
        "BIND_ADMIN",
        "ADMIN_USER",
        Some(user_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "managed_key_id": ARCHIVE_SIGN_KEY_ID,
        }),
    )
    .await?;

    Ok(Json(ok(BindAdminData {
        user_id,
        admin_account,
        user_group: "admins".to_string(),
        managed_key_id: ARCHIVE_SIGN_KEY_ID.to_string(),
    })))
}

pub(crate) async fn load_qr_sign_keys(
    state: &AppState,
) -> Result<Vec<QrSignKeyRuntime>, (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT key_id, purpose, status, pubkey, secret
         FROM qr_sign_keys
         ORDER BY key_id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query qr keys failed",
        )
    })?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let key_id: String = row.get("key_id");
        let secret_stored: String = row.get("secret");
        let secret_bytes = decrypt_secret(&key_id, &secret_stored).ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5003,
                "invalid archive sign secret encoding or decryption failed",
            )
        })?;
        if secret_bytes.len() != 32 {
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5003,
                "invalid archive sign secret length",
            ));
        }
        out.push(QrSignKeyRuntime {
            key_id: row.get("key_id"),
            purpose: row.get("purpose"),
            status: row.get("status"),
            pubkey: row.get("pubkey"),
            secret_bytes,
        });
    }
    Ok(out)
}

pub(crate) async fn load_cpms_install_runtime(
    state: &AppState,
) -> Result<CpmsInstallRuntime, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT cid_number, province_code, city_code, install_secret, cpms_pubkey
         FROM system_install
         WHERE id = 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query install failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "cpms not initialized"))?;

    let install_secret_stored: Option<String> = row.get("install_secret");
    let install_secret = install_secret_stored
        .as_deref()
        .and_then(decrypt_install_secret)
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "install_secret not found"))?;

    let cid_number = read_required_text(&row, "cid_number")?;

    Ok(CpmsInstallRuntime {
        cid_number,
        province_code: read_required_text(&row, "province_code")?,
        city_code: read_required_text(&row, "city_code")?,
        install_secret,
        cpms_pubkey: read_required_text(&row, "cpms_pubkey")?,
    })
}

fn read_required_text(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, &format!("{column} not found")))
}

fn parse_cid_install_qr_content(content: &str) -> Result<CidInstallQrPayload, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("cid_init_qr_content is empty".to_string());
    }

    if let Ok(payload) = serde_json::from_str::<CidInstallQrPayload>(trimmed) {
        return Ok(payload);
    }

    if let Ok(decoded) = STANDARD.decode(trimmed) {
        if let Ok(decoded_text) = String::from_utf8(decoded) {
            if let Ok(payload) = serde_json::from_str::<CidInstallQrPayload>(&decoded_text) {
                return Ok(payload);
            }
        }
    }

    Err("invalid cid_init_qr_content, expected json or base64(json)".to_string())
}

fn validate_cid_install_qr(payload: &CidInstallQrPayload) -> Result<(), String> {
    if payload.proto != "CID_CPMS_V1" {
        return Err(format!(
            "invalid cid install proto '{}', expected CID_CPMS_V1",
            payload.proto
        ));
    }
    if payload.r#type != "INSTALL" {
        return Err(format!(
            "invalid cid install type '{}', expected INSTALL",
            payload.r#type
        ));
    }
    if payload.cid_number.trim().is_empty()
        || payload.province_name.trim().is_empty()
        || payload.city_name.trim().is_empty()
        || payload.sig.trim().is_empty()
    {
        return Err("invalid cid install qr payload".to_string());
    }
    crate::address::validate_install_area(
        payload.cid_number.trim(),
        payload.province_name.trim(),
        payload.city_name.trim(),
    )?;
    decode_32_byte_hex(payload.install_secret.as_str())?;
    Ok(())
}

fn parse_cid_area_codes(cid_number: &str) -> (Option<String>, Option<String>) {
    let r5 = cid_number.split('-').next().unwrap_or("");
    if r5.len() >= 5 {
        (Some(r5[..2].to_string()), Some(r5[2..5].to_string()))
    } else {
        (None, None)
    }
}

fn normalize_admin_account(raw_input: &str) -> Result<String, (StatusCode, Json<ApiError>)> {
    let stripped = raw_input
        .strip_prefix("0x")
        .or_else(|| raw_input.strip_prefix("0X"))
        .unwrap_or(raw_input);
    if stripped.len() == 64 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(stripped.to_lowercase());
    }
    if let Some(hex_with_prefix) = crate::common::ss58::ss58_to_pubkey_hex(raw_input) {
        return Ok(hex_with_prefix
            .strip_prefix("0x")
            .unwrap_or(&hex_with_prefix)
            .to_lowercase());
    }
    Err(err(
        StatusCode::BAD_REQUEST,
        1001,
        "admin_account must be SS58 address or 32-byte hex (64 hex chars)",
    ))
}

/// 生成 sr25519 密钥对，返回 (0x hex pubkey, secret_raw_32bytes)。
fn generate_sr25519_keypair_raw() -> (String, [u8; 32]) {
    let mini = MiniSecretKey::generate_with(OsRng);
    let secret = mini.to_bytes();
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    (
        format!("0x{}", hex::encode(keypair.public.to_bytes())),
        secret,
    )
}

fn install_secret_hash(install_secret: &str) -> String {
    hash_hex(install_secret.as_bytes())
}

fn hash_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(Blake2b256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install_payload() -> CidInstallQrPayload {
        CidInstallQrPayload {
            proto: "CID_CPMS_V1".to_string(),
            r#type: "INSTALL".to_string(),
            cid_number: "GD001-GZG0E-123456789-2026".to_string(),
            province_name: "广东省".to_string(),
            city_name: "荔湾市".to_string(),
            install_secret: format!("0x{}", "11".repeat(32)),
            sig: "0xcid-issued-signature-kept-for-protocol".to_string(),
        }
    }

    /// 中文注释：安装 QR 校验含行政区交叉核对，依赖 CID 维护的 china.sqlite 唯一源。
    /// 测试指向其源文件（与 deploy 随附的是同一份），缺失则跳过，不在 CPMS 侧维护第二套源。
    fn point_to_china_source() -> bool {
        let china_db = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../citizencode/backend/china/china.sqlite"
        );
        if !std::path::Path::new(china_db).exists() {
            eprintln!("skip: china source not found at {china_db}");
            return false;
        }
        std::env::set_var("CPMS_CHINA_DB", china_db);
        true
    }

    #[test]
    fn validate_cid_install_qr_does_not_require_cid_pubkey_env() {
        if !point_to_china_source() {
            return;
        }
        let payload = install_payload();
        assert!(validate_cid_install_qr(&payload).is_ok());
    }

    #[test]
    fn validate_cid_install_qr_rejects_bad_install_secret() {
        if !point_to_china_source() {
            return;
        }
        let mut payload = install_payload();
        payload.install_secret = "0x1234".to_string();
        let result = validate_cid_install_qr(&payload);
        assert!(matches!(result, Err(reason) if reason == "install_secret must be 32 bytes"));
    }
}
