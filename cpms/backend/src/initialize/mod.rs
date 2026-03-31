use std::{collections::HashSet, env};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use rand::rngs::OsRng;
use schnorrkel::{signing_context, MiniSecretKey, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{err, ok, write_audit, ApiError, ApiResponse, AppState};

// ── 密钥加密存储 ──────────────────────────────────────────────────────────
// 中文注释：使用环境变量 CPMS_KEY_ENCRYPT_SECRET（32 字节 hex）作为主密钥，
// 对 QR 签名私钥做 XOR 加密后存入 DB。环境变量不存在时回退到明文（日志警告）。

fn master_encrypt_key() -> Option<[u8; 32]> {
    let hex_str = env::var("CPMS_KEY_ENCRYPT_SECRET").ok()?;
    let bytes = hex::decode(hex_str.trim()).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}

/// 用主密钥 + key_id 派生的流密钥对 secret 做 XOR 加密/解密（对称操作）。
fn xor_with_derived_key(master: &[u8; 32], key_id: &str, data: &[u8; 32]) -> [u8; 32] {
    use blake2::digest::consts::U32;
    use blake2::{Blake2b, Digest};
    type Blake2b256 = Blake2b<U32>;
    let mut hasher = Blake2b256::new();
    hasher.update(master);
    hasher.update(key_id.as_bytes());
    let derived = hasher.finalize();
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = data[i] ^ derived[i];
    }
    out
}

/// 加密 secret 后返回 hex 字符串（用于存入 DB）。
fn encrypt_secret(key_id: &str, secret_bytes: &[u8; 32]) -> String {
    match master_encrypt_key() {
        Some(master) => {
            let encrypted = xor_with_derived_key(&master, key_id, secret_bytes);
            format!("enc:{}", hex::encode(encrypted))
        }
        None => {
            eprintln!("WARNING: CPMS_KEY_ENCRYPT_SECRET not set, storing QR sign key in plaintext");
            hex::encode(secret_bytes)
        }
    }
}

/// 从 DB 读取的 secret 字符串解密为 32 字节。
fn decrypt_secret(key_id: &str, stored: &str) -> Option<Vec<u8>> {
    if let Some(enc_hex) = stored.strip_prefix("enc:") {
        let master = master_encrypt_key()?;
        let encrypted = hex::decode(enc_hex).ok()?;
        if encrypted.len() != 32 {
            return None;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&encrypted);
        let decrypted = xor_with_derived_key(&master, key_id, &arr);
        Some(decrypted.to_vec())
    } else {
        // 明文回退（兼容旧数据）
        crate::decode_bytes(stored)
    }
}

const FIXED_SUPER_ADMIN_COUNT: usize = 1;
const FIXED_QR_SIGN_KEY_COUNT: usize = 1;

#[derive(Clone)]
pub struct QrSignKeyRuntime {
    pub key_id: String,
    pub purpose: String,
    pub status: String,
    pub pubkey: String,
    pub secret_bytes: Vec<u8>,
}

/// SFID-CPMS QR v1 协议 QR1 载荷。
#[derive(Deserialize)]
struct SfidInstallQrPayload {
    ver: u32,
    qr_type: String,
    site_sfid: String,
    install_token: String,
    /// SFID RSA 公钥 PEM，CPMS 从 QR1 自动获取。
    rsa_public_key: String,
    signature: String,
}

/// SFID-CPMS QR v1 协议 QR3 载荷。
#[derive(Deserialize)]
struct SfidAnonCertQrPayload {
    ver: u32,
    qr_type: String,
    province_code: String,
    blind_anon_sig: String,
}

/// QR2 注册请求载荷（本机构造，展示给 SFID 扫描）。
#[derive(Serialize)]
struct CpmsRegisterReqPayload {
    ver: u32,
    qr_type: String,
    site_sfid: String,
    install_token: String,
    blind_anon_req: String,
}

/// 匿名证书（解盲后持久化到 DB）。
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AnonCert {
    pub(crate) province_code: String,
    pub(crate) anon_pubkey: String,
    pub(crate) sfid_sig: String,
    #[serde(default)]
    pub(crate) msg_randomizer: Option<String>,
}

#[derive(Deserialize)]
struct InstallInitializeRequest {
    sfid_init_qr_content: String,
}

#[derive(Serialize)]
struct InstallInitializeData {
    site_sfid: String,
    super_admin_bind_qrs: Vec<SuperAdminBindQrData>,
}

#[derive(Serialize)]
struct GenerateQr2Data {
    qr2_payload: String,
}

/// QR3 处理请求。
#[derive(Deserialize)]
struct ProcessAnonCertRequest {
    sfid_anon_cert_qr_content: String,
}

#[derive(Serialize)]
struct InstallStatusData {
    initialized: bool,
    site_sfid: Option<String>,
    super_admin_bound_count: usize,
    super_admin_bind_qrs: Vec<SuperAdminBindQrData>,
    qr2_ready: bool,
    qr2_payload: Option<String>,
    anon_cert_done: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct SuperAdminBindQrData {
    key_id: String,
    bound: bool,
    qr_payload: SuperAdminBindQrPayload,
    qr_content: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SuperAdminBindQrPayload {
    ver: String,
    qr_type: String,
    issuer_id: String,
    site_sfid: String,
    sign_key_id: String,
    sign_key_pubkey: String,
    bind_nonce: String,
    issued_at: i64,
}

#[derive(Deserialize)]
struct BindSuperAdminRequest {
    key_id: String,
    admin_pubkey: String,
    bind_nonce: String,
    signature: String,
}

#[derive(Serialize)]
struct BindSuperAdminData {
    user_id: String,
    admin_pubkey: String,
    role: String,
    status: String,
    managed_key_id: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/install/status", get(install_status))
        .route("/api/v1/install/initialize", post(initialize_install))
        .route("/api/v1/install/generate-qr2", post(generate_qr2))
        .route(
            "/api/v1/install/super-admin/bind",
            post(bind_super_admin_from_wuminapp),
        )
        // QR3 处理：需要 SUPER_ADMIN 认证，登录后调用
        .route("/api/v1/admin/anon-cert", post(process_anon_cert))
}


async fn install_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<InstallStatusData>>, (StatusCode, Json<ApiError>)> {
    let install_row = sqlx::query("SELECT site_sfid, anon_cert, anon_pubkey, blinding_factor, rsa_public_key_pem FROM system_install WHERE id = 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query install failed",
            )
        })?;

    let site_sfid = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("site_sfid").ok().flatten());
    let anon_cert_stored: Option<String> = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("anon_cert").ok().flatten());
    let anon_cert_done = anon_cert_stored.as_ref().map(|s| !s.is_empty()).unwrap_or(false);

    let keys = load_qr_sign_keys(&state).await?;

    let super_admin_bound_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_users WHERE role = 'SUPER_ADMIN' AND status = 'ACTIVE'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query super admin failed",
        )
    })?;

    let bound_key_rows = sqlx::query(
        "SELECT managed_key_id FROM admin_users WHERE role = 'SUPER_ADMIN' AND managed_key_id IS NOT NULL",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query bound keys failed"))?;

    let mut bound_keys = HashSet::new();
    for row in bound_key_rows {
        let key_id: Option<String> = row.get("managed_key_id");
        if let Some(v) = key_id {
            bound_keys.insert(v);
        }
    }

    let bind_qrs = build_super_admin_bind_qrs(site_sfid.clone(), &keys, &bound_keys)?;

    // QR2 在超级管理员绑定后才可生成
    let admin_bound = super_admin_bound_count as usize >= FIXED_SUPER_ADMIN_COUNT;
    let has_anon_pubkey = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("anon_pubkey").ok().flatten())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let qr2_ready = admin_bound && has_anon_pubkey;

    // 如果 QR2 就绪，构造 QR2 payload
    let qr2_payload = if qr2_ready {
        if let (Some(ref sfid), Some(ref row)) = (&site_sfid, &install_row) {
            let install_token: Option<String> = row.try_get::<Option<String>, _>("install_token").ok().flatten();
            let anon_pubkey: Option<String> = row.try_get::<Option<String>, _>("anon_pubkey").ok().flatten();
            let blinding_factor: Option<String> = row.try_get::<Option<String>, _>("blinding_factor").ok().flatten();
            let rsa_pem: Option<String> = row.try_get::<Option<String>, _>("rsa_public_key_pem").ok().flatten();

            if let (Some(token), Some(_anon_pk), Some(bf), Some(rsa)) = (install_token, anon_pubkey, blinding_factor, rsa_pem) {
                // blinding 已在 initialize 阶段完成，blind_msg 存在 blinding_factor 旁边
                // 实际上 blind_msg_hex 需要重新计算或从 DB 存取
                // 但当前设计中 initialize 已经做了盲化，我们需要把 QR2 的构造也存入 DB
                // 暂时返回 None，由 generate_qr2 端点负责
                None
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(Json(ok(InstallStatusData {
        initialized: site_sfid.is_some() && !keys.is_empty(),
        site_sfid,
        super_admin_bound_count: super_admin_bound_count as usize,
        super_admin_bind_qrs: bind_qrs,
        qr2_ready,
        qr2_payload,
        anon_cert_done,
    })))
}

async fn initialize_install(
    State(state): State<AppState>,
    Json(req): Json<InstallInitializeRequest>,
) -> Result<Json<ApiResponse<InstallInitializeData>>, (StatusCode, Json<ApiError>)> {
    if req.sfid_init_qr_content.trim().is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "invalid sfid_init_qr_content",
        ));
    }

    let qr_payload = parse_sfid_install_qr_content(&req.sfid_init_qr_content)
        .map_err(|reason| err(StatusCode::BAD_REQUEST, 4002, &reason))?;
    validate_sfid_install_qr(&qr_payload)
        .map_err(|reason| err(StatusCode::BAD_REQUEST, 4002, &reason))?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let current_site = sqlx::query("SELECT site_sfid FROM system_install WHERE id = 1 FOR UPDATE")
        .fetch_optional(tx.as_mut())
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "lock install failed",
            )
        })?
        .and_then(|r| r.try_get::<Option<String>, _>("site_sfid").ok().flatten());

    if current_site.is_some() {
        return Err(err(
            StatusCode::CONFLICT,
            4001,
            "cpms is already initialized",
        ));
    }

    let now_ts = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO system_install (id, site_sfid, install_token, rsa_public_key_pem, initialized_at)
         VALUES (1, $1, $2, $3, $4)
         ON CONFLICT (id) DO UPDATE SET site_sfid = EXCLUDED.site_sfid, install_token = EXCLUDED.install_token, rsa_public_key_pem = EXCLUDED.rsa_public_key_pem, initialized_at = EXCLUDED.initialized_at",
    )
    .bind(qr_payload.site_sfid.trim())
    .bind(qr_payload.install_token.trim())
    .bind(qr_payload.rsa_public_key.trim())
    .bind(now_ts)
    .execute(tx.as_mut())
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save install state failed"))?;

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

    let key_meta = [
        ("K1", "PRIMARY", "ACTIVE"),
    ];

    let mut keys_runtime = Vec::with_capacity(FIXED_QR_SIGN_KEY_COUNT);
    for (key_id, purpose, status) in key_meta {
        let (pubkey, secret_raw) = generate_sr25519_keypair_raw();
        let secret_stored = encrypt_secret(key_id, &secret_raw);
        sqlx::query(
            "INSERT INTO qr_sign_keys (key_id, purpose, status, pubkey, secret, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(key_id)
        .bind(purpose)
        .bind(status)
        .bind(&pubkey)
        .bind(&secret_stored)
        .bind(now_ts)
        .bind(now_ts)
        .execute(tx.as_mut())
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "insert qr key failed"))?;

        let secret_bytes = secret_raw.to_vec();
        keys_runtime.push(QrSignKeyRuntime {
            key_id: key_id.to_string(),
            purpose: purpose.to_string(),
            status: status.to_string(),
            pubkey,
            secret_bytes,
        });
    }

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    write_audit(
        &state,
        None,
        "INSTALL_INITIALIZE",
        "CPMS_INSTALL",
        Some(qr_payload.site_sfid.clone()),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    let bind_qrs = build_super_admin_bind_qrs(
        Some(qr_payload.site_sfid.clone()),
        &keys_runtime,
        &HashSet::new(),
    )?;

    Ok(Json(ok(InstallInitializeData {
        site_sfid: qr_payload.site_sfid,
        super_admin_bind_qrs: bind_qrs,
    })))
}

/// 绑定超级管理员后调用，生成匿名密钥对 + 盲化 + 返回 QR2。
async fn generate_qr2(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<GenerateQr2Data>>, (StatusCode, Json<ApiError>)> {
    // 检查超级管理员已绑定
    let admin_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_users WHERE role = 'SUPER_ADMIN' AND status = 'ACTIVE'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "count super admin failed"))?;
    if (admin_count as usize) < FIXED_SUPER_ADMIN_COUNT {
        return Err(err(StatusCode::CONFLICT, 4003, "super admin not bound yet"));
    }

    // 读取 install 信息
    let row = sqlx::query("SELECT site_sfid, install_token, rsa_public_key_pem, anon_pubkey FROM system_install WHERE id = 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query install failed"))?
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "not initialized"))?;

    let site_sfid: String = row.try_get::<Option<String>, _>("site_sfid")
        .ok().flatten()
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "site_sfid not found"))?;
    let install_token: String = row.try_get::<Option<String>, _>("install_token")
        .ok().flatten()
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "install_token not found"))?;
    let rsa_pubkey_pem: String = row.try_get::<Option<String>, _>("rsa_public_key_pem")
        .ok().flatten()
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "rsa_public_key_pem not found"))?;

    // 如果已经生成过匿名密钥，直接返回 QR2（幂等）
    let existing_anon: Option<String> = row.try_get::<Option<String>, _>("anon_pubkey")
        .ok().flatten().filter(|s| !s.is_empty());
    if existing_anon.is_some() {
        // 需要从 DB 重建 QR2，但 blind_msg 没有单独存储
        // 重新盲化（幂等生成新 QR2）
    }

    // 生成匿名签发密钥对
    let (anon_pubkey_hex, anon_secret_raw) = generate_sr25519_keypair_raw();
    let anon_secret_stored = encrypt_secret("ANON", &anon_secret_raw);

    let province_code = extract_province_code(&site_sfid);

    // RSABSSA 盲化
    let blinding_output = crate::rsa_blind_client::blind_message(
        rsa_pubkey_pem.trim(),
        &anon_pubkey_hex,
        &province_code,
    )
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &format!("blind failed: {e}")))?;

    let blinding_secret_hex = hex::encode(&blinding_output.blinding_secret);
    sqlx::query(
        "UPDATE system_install SET anon_pubkey = $1, anon_key_encrypted = $2, blinding_factor = $3 WHERE id = 1",
    )
    .bind(&anon_pubkey_hex)
    .bind(&anon_secret_stored)
    .bind(&blinding_secret_hex)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save anon key failed"))?;

    // 构造 QR2
    let qr2 = CpmsRegisterReqPayload {
        ver: 1,
        qr_type: "CPMS_REGISTER_REQ".to_string(),
        site_sfid: site_sfid.clone(),
        install_token,
        blind_anon_req: format!("0x{}", blinding_output.blind_msg_hex),
    };
    let qr2_payload = serde_json::to_string(&qr2)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "serialize QR2 failed"))?;

    write_audit(
        &state,
        None,
        "GENERATE_QR2",
        "CPMS_INSTALL",
        Some(site_sfid),
        "SUCCESS",
        serde_json::json!({ "anon_pubkey": anon_pubkey_hex }),
    )
    .await?;

    Ok(Json(ok(GenerateQr2Data { qr2_payload })))
}

/// 处理 QR3 匿名证书二维码（需要 SUPER_ADMIN 认证）。
///
/// CPMS 超级管理员登录后扫描 SFID 返回的 QR3 后调用此端点。
/// 本机解盲 blind_anon_sig，验证最终签名，持久化匿名证书。
async fn process_anon_cert(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ProcessAnonCertRequest>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiError>)> {
    crate::authz::require_role(&state, &headers, "SUPER_ADMIN").await?;

    if req.sfid_anon_cert_qr_content.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "empty qr content"));
    }

    let qr3: SfidAnonCertQrPayload =
        serde_json::from_str(req.sfid_anon_cert_qr_content.trim())
            .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid QR3 payload"))?;

    if qr3.qr_type != "SFID_ANON_CERT" {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "qr_type must be SFID_ANON_CERT"));
    }
    if qr3.province_code.trim().is_empty() || qr3.blind_anon_sig.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "province_code and blind_anon_sig are required"));
    }

    // 读取本机 anon_pubkey、blinding_factor 和 RSA 公钥
    let row = sqlx::query("SELECT anon_pubkey, blinding_factor, rsa_public_key_pem FROM system_install WHERE id = 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query anon_pubkey failed"))?
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "not initialized"))?;

    let anon_pubkey: Option<String> = row.get("anon_pubkey");
    let anon_pubkey = anon_pubkey.ok_or_else(|| {
        err(StatusCode::CONFLICT, 4003, "anon_pubkey not found, run initialize first")
    })?;

    let blinding_factor_hex: Option<String> = row.get("blinding_factor");
    let blinding_factor_hex = blinding_factor_hex.ok_or_else(|| {
        err(StatusCode::CONFLICT, 4003, "blinding_factor not found, run initialize first")
    })?;
    let blinding_secret = hex::decode(blinding_factor_hex.trim())
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "blinding_factor hex decode failed"))?;

    let rsa_pubkey_pem: Option<String> = row.get("rsa_public_key_pem");
    let rsa_pubkey_pem = rsa_pubkey_pem.ok_or_else(|| {
        err(StatusCode::CONFLICT, 4003, "rsa_public_key_pem not found")
    })?;

    // 解盲 blind_anon_sig
    let finalized = crate::rsa_blind_client::finalize_signature(
        &rsa_pubkey_pem,
        &qr3.blind_anon_sig,
        &blinding_secret,
        &anon_pubkey,
        &qr3.province_code,
    )
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &format!("finalize failed: {e}")))?;

    let anon_cert = AnonCert {
        province_code: qr3.province_code.clone(),
        anon_pubkey: anon_pubkey.clone(),
        sfid_sig: finalized.signature_hex,
        msg_randomizer: finalized.msg_randomizer_hex,
    };

    let anon_cert_json = serde_json::to_string(&anon_cert)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "serialize anon_cert failed"))?;

    // 持久化匿名证书
    sqlx::query("UPDATE system_install SET anon_cert = $1 WHERE id = 1")
        .bind(&anon_cert_json)
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save anon_cert failed"))?;

    write_audit(
        &state,
        None,
        "INSTALL_ANON_CERT",
        "CPMS_INSTALL",
        None,
        "SUCCESS",
        serde_json::json!({
            "province_code": qr3.province_code,
        }),
    )
    .await?;

    Ok(Json(ok("anon_cert saved")))
}

async fn bind_super_admin_from_wuminapp(
    State(state): State<AppState>,
    Json(req): Json<BindSuperAdminRequest>,
) -> Result<Json<ApiResponse<BindSuperAdminData>>, (StatusCode, Json<ApiError>)> {
    if req.key_id.trim().is_empty()
        || req.admin_pubkey.trim().is_empty()
        || req.bind_nonce.trim().is_empty()
        || req.signature.trim().is_empty()
    {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid bind request"));
    }

    let site_sfid = sqlx::query("SELECT site_sfid FROM system_install WHERE id = 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query install failed",
            )
        })?
        .and_then(|r| r.try_get::<Option<String>, _>("site_sfid").ok().flatten())
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "cpms not initialized"))?;

    let key_row = sqlx::query("SELECT pubkey FROM qr_sign_keys WHERE key_id = $1")
        .bind(req.key_id.trim())
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query key failed"))?
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "invalid key_id"))?;
    let sign_key_pubkey: String = key_row.get("pubkey");

    let expected_nonce = super_admin_bind_nonce(&site_sfid, req.key_id.trim(), &sign_key_pubkey);
    if req.bind_nonce.trim() != expected_nonce {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid bind_nonce"));
    }

    let bind_sign_source = super_admin_bind_sign_source(
        &site_sfid,
        req.key_id.trim(),
        req.admin_pubkey.trim(),
        req.bind_nonce.trim(),
    );
    crate::verify_signature_with_context(
        req.admin_pubkey.trim(),
        &bind_sign_source,
        req.signature.trim(),
        b"CPMS-SUPER-ADMIN-BIND-V1",
    )
    .map_err(|reason| err(StatusCode::UNAUTHORIZED, 2002, reason))?;

    let user_id = super_admin_user_id_for_key_id(req.key_id.trim())
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "invalid key_id"))?;
    let now_ts = Utc::now().timestamp();

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'SUPER_ADMIN'")
            .fetch_one(tx.as_mut())
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "count super admin failed",
                )
            })?;
    if count as usize >= FIXED_SUPER_ADMIN_COUNT {
        return Err(err(
            StatusCode::CONFLICT,
            4004,
            "super admin already bound",
        ));
    }

    let key_occupied: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM admin_users WHERE managed_key_id = $1 LIMIT 1")
            .bind(req.key_id.trim())
            .fetch_optional(tx.as_mut())
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "check key binding failed",
                )
            })?;
    if key_occupied.is_some() {
        return Err(err(
            StatusCode::CONFLICT,
            4004,
            "sign key already bound to super admin",
        ));
    }

    let pubkey_exists: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM admin_users WHERE admin_pubkey = $1 LIMIT 1")
            .bind(req.admin_pubkey.trim())
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
            "admin_pubkey already bound",
        ));
    }

    sqlx::query(
        "INSERT INTO admin_users (user_id, admin_pubkey, role, status, immutable, managed_key_id, created_at, updated_at)
         VALUES ($1, $2, 'SUPER_ADMIN', 'ACTIVE', TRUE, $3, $4, $5)",
    )
    .bind(&user_id)
    .bind(req.admin_pubkey.trim())
    .bind(req.key_id.trim())
    .bind(now_ts)
    .bind(now_ts)
    .execute(tx.as_mut())
    .await
    .map_err(|_| err(StatusCode::CONFLICT, 4004, "bind super admin failed"))?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    write_audit(
        &state,
        Some(user_id.clone()),
        "BIND_SUPER_ADMIN",
        "ADMIN_USER",
        Some(user_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "managed_key_id": req.key_id.trim(),
        }),
    )
    .await?;

    Ok(Json(ok(BindSuperAdminData {
        user_id,
        admin_pubkey: req.admin_pubkey.trim().to_string(),
        role: "SUPER_ADMIN".to_string(),
        status: "ACTIVE".to_string(),
        managed_key_id: req.key_id.trim().to_string(),
    })))
}

/// 公开解密接口，供 dangan 模块解密匿名私钥。
pub(crate) fn decrypt_secret_public(key_id: &str, stored: &str) -> Option<Vec<u8>> {
    decrypt_secret(key_id, stored)
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
                "invalid qr sign secret encoding or decryption failed",
            )
        })?;
        if secret_bytes.len() != 32 {
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5003,
                "invalid qr sign secret length",
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

fn build_super_admin_bind_qrs(
    site_sfid: Option<String>,
    keys: &[QrSignKeyRuntime],
    bound_keys: &HashSet<String>,
) -> Result<Vec<SuperAdminBindQrData>, (StatusCode, Json<ApiError>)> {
    let Some(site_sfid) = site_sfid else {
        return Ok(Vec::new());
    };

    keys.iter()
        .map(|key| {
            let bind_nonce = super_admin_bind_nonce(&site_sfid, &key.key_id, &key.pubkey);
            let qr_payload = SuperAdminBindQrPayload {
                ver: "1".to_string(),
                qr_type: "CPMS_SUPER_ADMIN_BIND".to_string(),
                issuer_id: "cpms".to_string(),
                site_sfid: site_sfid.clone(),
                sign_key_id: key.key_id.clone(),
                sign_key_pubkey: key.pubkey.clone(),
                bind_nonce,
                issued_at: Utc::now().timestamp(),
            };
            let qr_content = serde_json::to_string(&qr_payload)
                .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))?;
            Ok(SuperAdminBindQrData {
                key_id: key.key_id.clone(),
                bound: bound_keys.contains(&key.key_id),
                qr_payload,
                qr_content,
            })
        })
        .collect()
}

fn super_admin_bind_nonce(site_sfid: &str, key_id: &str, sign_key_pubkey: &str) -> String {
    let source = format!(
        "cpms-super-admin-bind-nonce-v1|{}|{}|{}",
        site_sfid, key_id, sign_key_pubkey
    );
    use blake2::digest::consts::U32;
    use blake2::{Blake2b, Digest};
    type Blake2b256 = Blake2b<U32>;
    let digest = Blake2b256::digest(source.as_bytes());
    hex::encode(&digest[..16])
}

fn super_admin_bind_sign_source(
    site_sfid: &str,
    key_id: &str,
    admin_pubkey: &str,
    bind_nonce: &str,
) -> String {
    format!(
        "cpms-super-admin-bind-v1|{}|{}|{}|{}",
        site_sfid, key_id, admin_pubkey, bind_nonce
    )
}

pub(crate) fn super_admin_user_id_for_key_id(key_id: &str) -> Option<String> {
    match key_id {
        "K1" => Some("u_super_admin_01".to_string()),
        "K2" => Some("u_super_admin_02".to_string()),
        "K3" => Some("u_super_admin_03".to_string()),
        _ => None,
    }
}

fn parse_sfid_install_qr_content(content: &str) -> Result<SfidInstallQrPayload, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("sfid_init_qr_content is empty".to_string());
    }

    if let Ok(payload) = serde_json::from_str::<SfidInstallQrPayload>(trimmed) {
        return Ok(payload);
    }

    if let Ok(decoded) = STANDARD.decode(trimmed) {
        if let Ok(decoded_text) = String::from_utf8(decoded) {
            if let Ok(payload) = serde_json::from_str::<SfidInstallQrPayload>(&decoded_text) {
                return Ok(payload);
            }
        }
    }

    Err("invalid sfid_init_qr_content, expected json or base64(json)".to_string())
}

fn validate_sfid_install_qr(payload: &SfidInstallQrPayload) -> Result<(), String> {
    if payload.qr_type.trim().is_empty()
        || payload.site_sfid.trim().is_empty()
        || payload.install_token.trim().is_empty()
        || payload.rsa_public_key.trim().is_empty()
        || payload.signature.trim().is_empty()
    {
        return Err("invalid sfid install qr payload".to_string());
    }

    if payload.qr_type != "SFID_CPMS_INSTALL" {
        return Err(format!(
            "invalid sfid install qr_type '{}', expected SFID_CPMS_INSTALL",
            payload.qr_type
        ));
    }

    let sfid_pubkey = env::var("SFID_ROOT_PUBKEY")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "SFID_ROOT_PUBKEY is required for install qr verification".to_string())?;

    // SFID-CPMS QR v1 协议签名原文
    let sign_source = format!(
        "sfid-cpms-install-v1|{}|{}",
        payload.site_sfid, payload.install_token
    );

    verify_sr25519_signature(
        &sfid_pubkey,
        &sign_source,
        &payload.signature,
        b"substrate",
    )
}

fn verify_sr25519_signature(
    pubkey: &str,
    payload: &str,
    signature: &str,
    context: &[u8],
) -> Result<(), String> {
    let pubkey_bytes = decode_bytes(pubkey).ok_or_else(|| "invalid pubkey encoding".to_string())?;
    if pubkey_bytes.len() != 32 {
        return Err("invalid pubkey length".to_string());
    }

    let sig_bytes =
        decode_bytes(signature).ok_or_else(|| "invalid signature encoding".to_string())?;
    if sig_bytes.len() != 64 {
        return Err("invalid signature length".to_string());
    }

    let pk = PublicKey::from_bytes(&pubkey_bytes)
        .map_err(|_| "invalid sr25519 public key".to_string())?;
    let sig =
        Signature::from_bytes(&sig_bytes).map_err(|_| "invalid sr25519 signature".to_string())?;
    pk.verify(signing_context(context).bytes(payload.as_bytes()), &sig)
        .map_err(|_| "sr25519 verify failed".to_string())
}

/// 中文注释：生成 sr25519 密钥对，返回 (pubkey_hex, secret_raw_32bytes)。
fn generate_sr25519_keypair_raw() -> (String, [u8; 32]) {
    let mini = MiniSecretKey::generate_with(OsRng);
    let secret = mini.to_bytes();
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    (hex::encode(keypair.public.to_bytes()), secret)
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

/// 从 site_sfid 的 r5 段提取两位字母省代码。
fn extract_province_code(site_sfid: &str) -> String {
    let segments: Vec<&str> = site_sfid.split('-').collect();
    if segments.len() >= 2 && segments[1].len() >= 2 {
        segments[1][..2].to_string()
    } else {
        String::new()
    }
}
