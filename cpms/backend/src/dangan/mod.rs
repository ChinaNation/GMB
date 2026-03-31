//! # 档案管理模块 (dangan)
//!
//! 档案号生成（V3 格式）、QR 载荷构造与签名、公民状态校验、站点密钥注册载荷构造。

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use axum::{http::StatusCode, Json};
use chrono::Utc;
use rand::rngs::OsRng;
use schnorrkel::{signing_context, MiniSecretKey};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{err, initialize::QrSignKeyRuntime, ApiError, AppState, Archive};

pub(crate) mod province_codes;

const ARCHIVE_NO_MAX_RETRY: u32 = 20;
const QR_EXPIRES_SECONDS: i64 = 24 * 60 * 60;

#[derive(Serialize)]
pub(crate) struct QrPayload {
    pub(crate) ver: String,
    pub(crate) issuer_id: String,
    pub(crate) site_sfid: String,
    pub(crate) sign_key_id: String,
    pub(crate) archive_no: String,
    pub(crate) citizen_status: String,
    pub(crate) voting_eligible: bool,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
    pub(crate) qr_id: String,
    pub(crate) sig_alg: String,
    pub(crate) signature: String,
}

#[derive(Serialize)]
pub(crate) struct SiteKeyRegistrationPayload {
    pub(crate) ver: String,
    pub(crate) qr_type: String,
    pub(crate) issuer_id: String,
    pub(crate) site_sfid: String,
    pub(crate) keys: Vec<SiteKeyPublicItem>,
    pub(crate) issued_at: i64,
    pub(crate) qr_id: String,
    pub(crate) sig_alg: String,
    pub(crate) sign_key_id: String,
    pub(crate) signature: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SiteKeyPublicItem {
    pub(crate) key_id: String,
    pub(crate) purpose: String,
    pub(crate) status: String,
    pub(crate) pubkey: String,
}

pub(crate) async fn generate_archive_no_with_retry(
    state: &AppState,
    province_code: &str,
    city_code: &str,
    created_date_yyyymmdd: &str,
    terminal_id: &str,
    admin_pubkey: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let seq_key = format!("{}|{}|{}", province_code, city_code, created_date_yyyymmdd);

    let mut nonce: i64 = sqlx::query_scalar(
        "INSERT INTO sequence_counters (seq_key, next_seq)
         VALUES ($1, 2)
         ON CONFLICT (seq_key) DO UPDATE SET next_seq = sequence_counters.next_seq + 1
         RETURNING next_seq - 1",
    )
    .bind(&seq_key)
    .fetch_one(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "sequence alloc failed",
        )
    })?;

    for _ in 0..ARCHIVE_NO_MAX_RETRY {
        let random9 = generate_random9(terminal_id, admin_pubkey, nonce as u32);
        let check_digit =
            archive_checksum_digit(province_code, city_code, &random9, created_date_yyyymmdd);
        let archive_no = format!(
            "{}{}{}{}{}",
            province_code, city_code, check_digit, random9, created_date_yyyymmdd
        );

        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM archives WHERE archive_no = $1)")
                .bind(&archive_no)
                .fetch_one(&state.db)
                .await
                .map_err(|_| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "archive lookup failed",
                    )
                })?;

        if !exists {
            return Ok(archive_no);
        }
        nonce += 1;
    }

    Err(err(
        StatusCode::CONFLICT,
        3005,
        "archive_no conflict, retry exhausted",
    ))
}

fn generate_random9(terminal_id: &str, admin_pubkey: &str, nonce: u32) -> String {
    let ts = Utc::now().timestamp_millis();
    let source = format!("{}|{}|{}|{}", ts, terminal_id, admin_pubkey, nonce);
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let n = hasher.finish() % 1_000_000_000;
    format!("{:09}", n)
}

// ── SFID-CPMS QR v1: AR4 不透明档案号 ─────────────────────────────────

const AR4_RANDOM_LEN: usize = 26;
const AR4_CHECK_LEN: usize = 2;
const AR4_MAX_RETRY: u32 = 20;
/// Base32 字母表（RFC 4648 无 padding）
const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// 生成 AR4 格式不透明档案号：`AR4-<26位Base32随机>-<2位校验>`
///
/// 随机体用安全随机数，不编码省、市、站点、日期。
pub(crate) async fn generate_ar4_archive_no(
    state: &AppState,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    use rand::RngCore;

    for _ in 0..AR4_MAX_RETRY {
        // 生成 26 位 Base32 随机体
        let mut random_bytes = [0u8; 26];
        OsRng.fill_bytes(&mut random_bytes);
        let random_body: String = random_bytes
            .iter()
            .map(|b| BASE32_ALPHABET[(*b as usize) % BASE32_ALPHABET.len()] as char)
            .collect();

        // 计算 2 位校验
        let check = ar4_checksum(&random_body);
        let archive_no = format!("AR4-{}-{}", random_body, check);

        // 碰撞检测
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM archives WHERE archive_no = $1)")
                .bind(&archive_no)
                .fetch_one(&state.db)
                .await
                .map_err(|_| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "archive lookup failed",
                    )
                })?;

        if !exists {
            return Ok(archive_no);
        }
    }

    Err(err(
        StatusCode::CONFLICT,
        3005,
        "ar4 archive_no conflict, retry exhausted",
    ))
}

/// AR4 档案号 2 位校验码。
fn ar4_checksum(random_body: &str) -> String {
    use blake2::digest::consts::U32;
    use blake2::{Blake2b, Digest};
    type Blake2b256 = Blake2b<U32>;
    let payload = format!("cpms-ar4-checksum-v1|{}", random_body);
    let digest = Blake2b256::digest(payload.as_bytes());
    // 取前两字节映射到 Base32
    let c1 = BASE32_ALPHABET[(digest[0] as usize) % BASE32_ALPHABET.len()] as char;
    let c2 = BASE32_ALPHABET[(digest[1] as usize) % BASE32_ALPHABET.len()] as char;
    format!("{}{}", c1, c2)
}

// ── SFID-CPMS QR v1: QR4 档案业务二维码构造 ──────────────────────────

/// QR4 档案业务二维码载荷。
#[derive(Serialize)]
pub(crate) struct ArchiveQr4Payload {
    pub(crate) ver: u32,
    pub(crate) qr_type: String,
    pub(crate) province_code: String,
    pub(crate) archive_no: String,
    pub(crate) citizen_status: String,
    pub(crate) voting_eligible: bool,
    pub(crate) anon_cert: serde_json::Value,
    pub(crate) archive_sig: String,
}

/// 构造 QR4 载荷（SFID-CPMS QR v1 协议）。
///
/// 使用本机匿名私钥签名，嵌入匿名证书。
pub(crate) async fn build_qr4_payload(
    state: &AppState,
    archive: &crate::Archive,
) -> Result<ArchiveQr4Payload, (StatusCode, Json<ApiError>)> {
    // 读取匿名证书和匿名私钥
    let row = sqlx::query("SELECT anon_cert, anon_key_encrypted, anon_pubkey FROM system_install WHERE id = 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query anon_cert failed"))?
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "cpms not initialized"))?;

    let anon_cert_json: Option<String> = row.get("anon_cert");
    let anon_cert_json = anon_cert_json
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "anon_cert not found, complete QR3 first"))?;
    let anon_cert: serde_json::Value = serde_json::from_str(&anon_cert_json)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "parse anon_cert failed"))?;

    let province_code = anon_cert
        .get("province_code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "anon_cert missing province_code"))?
        .to_string();

    // 解密匿名私钥
    let anon_secret_stored: Option<String> = row.get("anon_key_encrypted");
    let anon_secret_stored = anon_secret_stored
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "anon_key not found"))?;
    let anon_secret_bytes = crate::initialize::decrypt_secret_public("ANON", &anon_secret_stored)
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, 5003, "decrypt anon key failed"))?;

    let voting_eligible = archive.citizen_status == "NORMAL";

    // 签名原文：cpms-archive-qr-v1|{province_code}|{archive_no}|{citizen_status}|{voting_eligible}
    let sign_source = format!(
        "cpms-archive-qr-v1|{}|{}|{}|{}",
        province_code, archive.archive_no, archive.citizen_status, voting_eligible
    );
    let archive_sig = sign_qr_payload_with_secret(&anon_secret_bytes, &sign_source)?;

    Ok(ArchiveQr4Payload {
        ver: 1,
        qr_type: "CPMS_ARCHIVE_QR".to_string(),
        province_code,
        archive_no: archive.archive_no.clone(),
        citizen_status: archive.citizen_status.clone(),
        voting_eligible,
        anon_cert,
        archive_sig,
    })
}

pub(crate) fn archive_checksum_digit(
    province_code: &str,
    city_code: &str,
    random9: &str,
    created_date_yyyymmdd: &str,
) -> char {
    let payload = format!(
        "cpms-archive-v3|{}{}{}{}",
        province_code, city_code, random9, created_date_yyyymmdd
    );
    use blake2::digest::consts::U32;
    use blake2::{Blake2b, Digest};
    type Blake2b256 = Blake2b<U32>;
    let digest = Blake2b256::digest(payload.as_bytes());
    let sum: u32 = digest.iter().map(|&b| b as u32).sum();
    let n = (sum % 10) as u8;
    char::from(b'0' + n)
}

pub(crate) async fn build_qr_payload(
    state: &AppState,
    archive: &Archive,
) -> Result<QrPayload, (StatusCode, Json<ApiError>)> {
    let (site_sfid, sign_key) = active_qr_sign_key(state).await?;
    let issued_at = Utc::now().timestamp();
    let expire_at = issued_at + QR_EXPIRES_SECONDS;
    let qr_id = format!("qr_{}", Uuid::new_v4().simple());
    let voting_eligible = archive.citizen_status == "NORMAL";
    let sign_source = format!(
        "cpms-qr-v1|{}|{}|{}|{}|{}|{}|{}",
        &site_sfid,
        sign_key.key_id,
        archive.archive_no,
        archive.citizen_status,
        voting_eligible,
        issued_at,
        qr_id
    );
    let signature = sign_qr_payload_with_secret(&sign_key.secret_bytes, &sign_source)?;

    Ok(QrPayload {
        ver: "1".to_string(),
        issuer_id: "cpms".to_string(),
        site_sfid: site_sfid.clone(),
        sign_key_id: sign_key.key_id,
        archive_no: archive.archive_no.clone(),
        citizen_status: archive.citizen_status.clone(),
        voting_eligible,
        issued_at,
        expire_at,
        qr_id,
        sig_alg: "sr25519".to_string(),
        signature,
    })
}

pub(crate) async fn build_site_key_registration_payload(
    state: &AppState,
) -> Result<SiteKeyRegistrationPayload, (StatusCode, Json<ApiError>)> {
    let (site_sfid, keys_runtime) = install_snapshot(state).await?;
    let sign_key = keys_runtime
        .iter()
        .find(|k| k.status == "ACTIVE")
        .cloned()
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                "missing active qr sign key",
            )
        })?;

    let issued_at = Utc::now().timestamp();
    let qr_id = format!("qr_{}", Uuid::new_v4().simple());
    let keys: Vec<SiteKeyPublicItem> = keys_runtime
        .iter()
        .map(|key| SiteKeyPublicItem {
            key_id: key.key_id.clone(),
            purpose: key.purpose.clone(),
            status: key.status.clone(),
            pubkey: key.pubkey.clone(),
        })
        .collect();
    let key_summary = keys
        .iter()
        .map(|k| format!("{}:{}:{}", k.key_id, k.purpose, k.pubkey))
        .collect::<Vec<String>>()
        .join("|");
    let sign_source = format!(
        "cpms-site-key-register-v1|{}|{}|{}|{}",
        &site_sfid, key_summary, issued_at, qr_id
    );
    let signature = sign_qr_payload_with_secret(&sign_key.secret_bytes, &sign_source)?;

    Ok(SiteKeyRegistrationPayload {
        ver: "1".to_string(),
        qr_type: "CPMS_SITE_KEYS_REGISTER".to_string(),
        issuer_id: "cpms".to_string(),
        site_sfid: site_sfid.clone(),
        keys,
        issued_at,
        qr_id,
        sig_alg: "sr25519".to_string(),
        sign_key_id: sign_key.key_id,
        signature,
    })
}

async fn install_snapshot(
    state: &AppState,
) -> Result<(String, Vec<QrSignKeyRuntime>), (StatusCode, Json<ApiError>)> {
    let site_row = sqlx::query("SELECT site_sfid FROM system_install WHERE id = 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query install failed",
            )
        })?;

    let site_sfid = site_row
        .and_then(|r| r.try_get::<Option<String>, _>("site_sfid").ok().flatten())
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "cpms not initialized"))?;

    let keys = crate::initialize::load_qr_sign_keys(state).await?;
    if keys.is_empty() {
        return Err(err(
            StatusCode::CONFLICT,
            4005,
            "missing qr sign keys after initialization",
        ));
    }

    Ok((site_sfid, keys))
}

async fn active_qr_sign_key(
    state: &AppState,
) -> Result<(String, QrSignKeyRuntime), (StatusCode, Json<ApiError>)> {
    let (site_sfid, keys) = install_snapshot(state).await?;
    let sign_key = keys
        .iter()
        .find(|k| k.status == "ACTIVE")
        .cloned()
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                "missing active qr sign key",
            )
        })?;
    Ok((site_sfid, sign_key))
}

pub(crate) fn validate_citizen_status(status: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    match status {
        "NORMAL" | "ABNORMAL" => Ok(()),
        _ => Err(err(StatusCode::BAD_REQUEST, 1001, "invalid citizen_status")),
    }
}

pub(crate) fn sign_qr_payload_with_secret(
    secret_bytes: &[u8],
    payload: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    if secret_bytes.len() != 32 {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid qr sign secret length",
        ));
    }

    let mini = MiniSecretKey::from_bytes(secret_bytes).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid qr sign secret key",
        )
    })?;
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    let sig = keypair.sign(signing_context(b"CPMS-QR-SIGN-V1").bytes(payload.as_bytes()));
    Ok(hex::encode(sig.to_bytes()))
}
