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

use uuid::Uuid;
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

/// SFID_CPMS_V1 协议 QR1 载荷。
#[derive(Deserialize)]
struct SfidInstallQrPayload {
    #[serde(default)]
    proto: String,
    #[serde(alias = "qr_type")]
    r#type: String,
    sfid: String,
    token: String,
    /// SFID RSA 公钥（base64 裸数据，无 PEM 头尾）
    rsa: String,
    sig: String,
    /// 协议扩展：省/市/机构名称（SFID 后端生成 QR1 时写入）
    #[serde(default)]
    province_name: Option<String>,
    #[serde(default)]
    city_name: Option<String>,
    #[serde(default)]
    institution_name: Option<String>,
}

/// SFID_CPMS_V1 协议 QR3 载荷。
#[derive(Deserialize)]
struct SfidAnonCertQrPayload {
    #[serde(default)]
    proto: String,
    #[serde(alias = "qr_type")]
    r#type: String,
    prov: String,
    bsig: String,
}

/// QR2 注册请求载荷（本机构造，展示给 SFID 扫描）。
#[derive(Serialize)]
struct CpmsRegisterReqPayload {
    proto: String,
    r#type: String,
    sfid: String,
    token: String,
    blind: String,
}

/// 匿名证书（解盲后持久化到 DB）。
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AnonCert {
    pub(crate) prov: String,
    pub(crate) pk: String,
    pub(crate) sig: String,
    #[serde(default)]
    pub(crate) mr: Option<String>,
}

#[derive(Deserialize)]
struct InstallInitializeRequest {
    sfid_init_qr_content: String,
}

#[derive(Serialize)]
struct InstallInitializeData {
    site_sfid: String,
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
    province_name: Option<String>,
    city_name: Option<String>,
    institution_name: Option<String>,
    super_admin_bound_count: usize,
    qr2_ready: bool,
    qr2_payload: Option<String>,
    anon_cert_done: bool,
}

#[derive(Deserialize)]
struct BindSuperAdminRequest {
    admin_pubkey: String,
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
        .route(
            "/api/v1/install/super-admin/bind",
            post(bind_super_admin_from_wuminapp),
        )
        // 以下需要 SUPER_ADMIN 认证，登录后调用
        .route("/api/v1/admin/generate-qr2", post(generate_qr2))
        .route("/api/v1/admin/anon-cert", post(process_anon_cert))
}


async fn install_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<InstallStatusData>>, (StatusCode, Json<ApiError>)> {
    let install_row = sqlx::query("SELECT site_sfid, anon_cert, anon_pubkey, blinding_factor, rsa_public_key_pem, province_name, city_name, institution_name, qr2_payload FROM system_install WHERE id = 1")
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
    let province_name: Option<String> = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("province_name").ok().flatten());
    let city_name: Option<String> = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("city_name").ok().flatten());
    let institution_name_val: Option<String> = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("institution_name").ok().flatten());
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

    // QR2 在超级管理员绑定后才可生成
    let admin_bound = super_admin_bound_count as usize >= FIXED_SUPER_ADMIN_COUNT;
    let has_anon_pubkey = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("anon_pubkey").ok().flatten())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let qr2_ready = admin_bound && has_anon_pubkey;

    // 从 DB 读取已持久化的 qr2_payload
    let qr2_payload: Option<String> = install_row.as_ref()
        .and_then(|r| r.try_get::<Option<String>, _>("qr2_payload").ok().flatten())
        .filter(|s| !s.is_empty());

    let initialized = site_sfid.is_some() && !keys.is_empty();

    Ok(Json(ok(InstallStatusData {
        initialized,
        site_sfid,
        province_name,
        city_name,
        institution_name: institution_name_val,
        super_admin_bound_count: super_admin_bound_count as usize,
        qr2_ready,
        qr2_payload,
        anon_cert_done,
    })))
}

/// 处理 QR1 安装授权二维码。
///
/// 两种路径：
/// - 首次初始化：写入 site_sfid + token + RSA 公钥，生成 QR 签名密钥。
/// - 重新授权（site_sfid 一致）：只更新 token + RSA 公钥，清空匿名证书相关字段，
///   保留全部业务数据（admin_users / archives / qr_sign_keys）。
///   CPMS 不判断授权是否失效——只有 SFID 侧能判断。
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

    let is_reauth = if let Some(ref existing_sfid) = current_site {
        // 已初始化：校验 sfid 一致才允许重新授权
        if existing_sfid.trim() != qr_payload.sfid.trim() {
            return Err(err(
                StatusCode::CONFLICT,
                4001,
                "site_sfid mismatch, cannot reauthorize with a different sfid",
            ));
        }
        true
    } else {
        false
    };

    let now_ts = Utc::now().timestamp();

    if is_reauth {
        // ── 重新授权路径：只刷新凭证，清空匿名证书，更新名称，保留全部业务数据 ──
        sqlx::query(
            "UPDATE system_install
             SET install_token = $1,
                 rsa_public_key_pem = $2,
                 initialized_at = $3,
                 anon_pubkey = NULL,
                 anon_key_encrypted = NULL,
                 blinding_factor = NULL,
                 anon_cert = NULL,
                 province_name = COALESCE($4, province_name),
                 city_name = COALESCE($5, city_name),
                 institution_name = COALESCE($6, institution_name)
             WHERE id = 1",
        )
        .bind(qr_payload.token.trim())
        .bind(&rebuild_pem_envelope(qr_payload.rsa.trim()))
        .bind(now_ts)
        .bind(qr_payload.province_name.as_deref())
        .bind(qr_payload.city_name.as_deref())
        .bind(qr_payload.institution_name.as_deref())
        .execute(tx.as_mut())
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "reauthorize update failed"))?;

        // qr_sign_keys / admin_users / archives 全部保留，不动

        tx.commit()
            .await
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

        write_audit(
            &state,
            None,
            "INSTALL_REAUTHORIZE",
            "CPMS_INSTALL",
            Some(qr_payload.sfid.clone()),
            "SUCCESS",
            serde_json::json!({}),
        )
        .await?;
    } else {
        // ── 首次初始化路径（原有逻辑） ──
        sqlx::query(
            "INSERT INTO system_install (id, site_sfid, install_token, rsa_public_key_pem, initialized_at, province_name, city_name, institution_name)
             VALUES (1, $1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO UPDATE SET site_sfid = EXCLUDED.site_sfid, install_token = EXCLUDED.install_token, rsa_public_key_pem = EXCLUDED.rsa_public_key_pem, initialized_at = EXCLUDED.initialized_at, province_name = EXCLUDED.province_name, city_name = EXCLUDED.city_name, institution_name = EXCLUDED.institution_name",
        )
        .bind(qr_payload.sfid.trim())
        .bind(qr_payload.token.trim())
        .bind(&rebuild_pem_envelope(qr_payload.rsa.trim()))
        .bind(now_ts)
        .bind(qr_payload.province_name.as_deref())
        .bind(qr_payload.city_name.as_deref())
        .bind(qr_payload.institution_name.as_deref())
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
        }

        tx.commit()
            .await
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

        write_audit(
            &state,
            None,
            "INSTALL_INITIALIZE",
            "CPMS_INSTALL",
            Some(qr_payload.sfid.clone()),
            "SUCCESS",
            serde_json::json!({}),
        )
        .await?;
    }

    Ok(Json(ok(InstallInitializeData {
        site_sfid: qr_payload.sfid,
    })))
}

/// 登录后调用，生成匿名密钥对 + 盲化 + 返回 QR2。需要 SUPER_ADMIN 认证。
async fn generate_qr2(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<GenerateQr2Data>>, (StatusCode, Json<ApiError>)> {
    crate::authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
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
        proto: "SFID_CPMS_V1".to_string(),
        r#type: "REGISTER".to_string(),
        sfid: site_sfid.clone(),
        token: install_token,
        blind: format!("0x{}", blinding_output.blind_msg_hex),
    };
    let qr2_payload = serde_json::to_string(&qr2)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "serialize QR2 failed"))?;

    // 持久化 qr2_payload，重新生成时覆盖旧值
    sqlx::query("UPDATE system_install SET qr2_payload = $1 WHERE id = 1")
        .bind(&qr2_payload)
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save qr2_payload failed"))?;

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

    if qr3.r#type != "CERT" {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "type must be CERT"));
    }
    if qr3.prov.trim().is_empty() || qr3.bsig.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "prov and bsig are required"));
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
        &qr3.bsig,
        &blinding_secret,
        &anon_pubkey,
        &qr3.prov,
    )
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &format!("finalize failed: {e}")))?;

    let anon_cert = AnonCert {
        prov: qr3.prov.clone(),
        pk: anon_pubkey.clone(),
        sig: finalized.signature_hex,
        mr: finalized.msg_randomizer_hex,
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
            "province_code": qr3.prov,
        }),
    )
    .await?;

    Ok(Json(ok("anon_cert saved")))
}

async fn bind_super_admin_from_wuminapp(
    State(state): State<AppState>,
    Json(req): Json<BindSuperAdminRequest>,
) -> Result<Json<ApiResponse<BindSuperAdminData>>, (StatusCode, Json<ApiError>)> {
    let raw_input = req.admin_pubkey.trim().to_string();
    if raw_input.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "admin_pubkey is required"));
    }
    // 归一化公钥：支持 SS58 地址或 0x hex 公钥
    let admin_pubkey = {
        // 先尝试 0x hex
        let stripped = raw_input.strip_prefix("0x").or_else(|| raw_input.strip_prefix("0X")).unwrap_or(&raw_input);
        if stripped.len() == 64 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
            stripped.to_lowercase()
        } else if let Some(hex_with_prefix) = crate::ss58::ss58_to_pubkey_hex(&raw_input) {
            // SS58 地址 → 解码为 0x hex → 去掉 0x 前缀
            hex_with_prefix.strip_prefix("0x").unwrap_or(&hex_with_prefix).to_lowercase()
        } else {
            return Err(err(StatusCode::BAD_REQUEST, 1001,
                "admin_pubkey must be SS58 address or 32-byte hex (64 hex chars)"));
        }
    };

    // 使用固定的 K1 作为 key_id
    let user_id = "u_super_admin_01".to_string();
    let key_id = "K1";
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

    let pubkey_exists: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM admin_users WHERE admin_pubkey = $1 LIMIT 1")
            .bind(&admin_pubkey)
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
    .bind(&admin_pubkey)
    .bind(key_id)
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
            "managed_key_id": key_id,
        }),
    )
    .await?;

    Ok(Json(ok(BindSuperAdminData {
        user_id,
        admin_pubkey: admin_pubkey.clone(),
        role: "SUPER_ADMIN".to_string(),
        status: "ACTIVE".to_string(),
        managed_key_id: key_id.to_string(),
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
    if payload.r#type.trim().is_empty()
        || payload.sfid.trim().is_empty()
        || payload.token.trim().is_empty()
        || payload.rsa.trim().is_empty()
    {
        return Err("invalid sfid install qr payload".to_string());
    }

    if payload.r#type != "INSTALL" {
        return Err(format!(
            "invalid sfid install type '{}', expected INSTALL",
            payload.r#type
        ));
    }

    // CPMS 是离线系统，初始化时没有 SFID 公钥，不验 QR1 签名。
    // 安全性靠 install_token 一次性消费保证。
    Ok(())
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

/// 生成 sr25519 密钥对，返回 (0x hex pubkey, secret_raw_32bytes)。
/// pubkey 统一 0x 小写 hex 格式（feedback_pubkey_format_rule.md）。
fn generate_sr25519_keypair_raw() -> (String, [u8; 32]) {
    let mini = MiniSecretKey::generate_with(OsRng);
    let secret = mini.to_bytes();
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    (format!("0x{}", hex::encode(keypair.public.to_bytes())), secret)
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

/// 从裸 base64 重建 PEM 信封。
fn rebuild_pem_envelope(raw_b64: &str) -> String {
    let mut pem = String::from("-----BEGIN PUBLIC KEY-----\n");
    for (i, ch) in raw_b64.chars().enumerate() {
        pem.push(ch);
        if (i + 1) % 64 == 0 {
            pem.push('\n');
        }
    }
    if !pem.ends_with('\n') {
        pem.push('\n');
    }
    pem.push_str("-----END PUBLIC KEY-----");
    pem
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
