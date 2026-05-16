//! # 档案管理模块 (dangan)
//!
//! 档案号生成、ARCHIVE 二维码载荷构造与签名、公民状态校验。

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use axum::{http::StatusCode, Json};
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use rand::{rngs::OsRng, RngCore};
use schnorrkel::{signing_context, MiniSecretKey};
use serde::Serialize;

use crate::{err, initialize::QrSignKeyRuntime, ApiError, AppState, Archive};

type Blake2b256 = Blake2b<U32>;

const ARCHIVE_NO_MAX_RETRY: u32 = 20;
const ARCHIVE_SIGN_KEY_ID: &str = "ARCHIVE";
const ARCHIVE_NO_BODY_BYTES: usize = 16;
const GEO_SEAL_PREFIX: &str = "g1";
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// SFID_CPMS_V1 / ARCHIVE 档案二维码载荷。
#[derive(Clone, Serialize)]
pub(crate) struct ArchiveQrPayload {
    pub(crate) proto: String,
    pub(crate) r#type: String,
    pub(crate) ano: String,
    pub(crate) cs: String,
    pub(crate) ve: bool,
    pub(crate) cpms_pubkey: String,
    pub(crate) geo_seal: String,
    pub(crate) sig: String,
}

#[derive(Serialize)]
struct GeoSealClaims {
    /// 中文注释：归属密文只放机构 SFID 号；省市由 SFID 从 sfid_number 解码。
    sfid_number: String,
}

/// 生成不暴露省市和机构号的档案号。
///
/// 中文注释：`install_secret` 参与哈希域隔离，安全随机数提供全局碰撞强度，
/// DB 唯一索引负责兜底拒绝本机重复；SFID 录入时再做全局唯一最终校验。
pub(crate) async fn generate_archive_no_with_retry(
    state: &AppState,
    install_secret: &str,
    terminal_id: &str,
    admin_pubkey: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let mut counter: i64 = sqlx::query_scalar(
        "INSERT INTO sequence_counters (seq_key, next_seq)
         VALUES ('archive_no', 2)
         ON CONFLICT (seq_key) DO UPDATE SET next_seq = sequence_counters.next_seq + 1
         RETURNING next_seq - 1",
    )
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
        let mut random = [0u8; 32];
        OsRng.fill_bytes(&mut random);
        let body = archive_no_body(
            install_secret,
            terminal_id,
            admin_pubkey,
            counter,
            random.as_slice(),
        );
        let checksum = archive_no_checksum(&body);
        // 中文注释：档案号不携带协议前缀，避免把示例前缀固化成业务含义。
        let archive_no = format!("{}-{}", body, checksum);

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
        counter += 1;
    }

    Err(err(
        StatusCode::CONFLICT,
        3005,
        "archive_no conflict, retry exhausted",
    ))
}

fn archive_no_body(
    install_secret: &str,
    terminal_id: &str,
    admin_pubkey: &str,
    counter: i64,
    random: &[u8],
) -> String {
    let mut hasher = Blake2b256::new();
    hasher.update(b"sfid-cpms-v1|archive-no|");
    hasher.update(install_secret.as_bytes());
    hasher.update(b"|");
    hasher.update(terminal_id.as_bytes());
    hasher.update(b"|");
    hasher.update(admin_pubkey.as_bytes());
    hasher.update(b"|");
    hasher.update(counter.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(random);
    let digest = hasher.finalize();
    base32_no_padding(&digest[..ARCHIVE_NO_BODY_BYTES])
}

pub(crate) fn archive_no_checksum(body: &str) -> String {
    let mut hasher = Blake2b256::new();
    hasher.update(b"sfid-cpms-v1|ano-check|");
    hasher.update(body.as_bytes());
    let digest = hasher.finalize();
    base32_no_padding(&digest[..4]).chars().take(2).collect()
}

fn base32_no_padding(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut buffer: u32 = 0;
    let mut bits_left: u8 = 0;
    for byte in bytes {
        buffer = (buffer << 8) | (*byte as u32);
        bits_left += 8;
        while bits_left >= 5 {
            let idx = ((buffer >> (bits_left - 5)) & 0x1f) as usize;
            out.push(BASE32_ALPHABET[idx] as char);
            bits_left -= 5;
        }
    }
    if bits_left > 0 {
        let idx = ((buffer << (5 - bits_left)) & 0x1f) as usize;
        out.push(BASE32_ALPHABET[idx] as char);
    }
    out
}

/// 构造 ARCHIVE 载荷（SFID_CPMS_V1）。
///
/// 中文注释：二维码明文字段不放省、市、CPMS 机构号；归属只放入 `geo_seal`，
/// SFID 使用安装授权中的 install_secret 才能解开。
pub(crate) async fn build_archive_qr_payload(
    state: &AppState,
    archive: &Archive,
) -> Result<ArchiveQrPayload, (StatusCode, Json<ApiError>)> {
    let install = crate::initialize::load_cpms_install_runtime(state).await?;
    let sign_key = active_archive_sign_key(state).await?;
    if sign_key.pubkey != install.cpms_pubkey {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "archive sign key does not match install cpms_pubkey",
        ));
    }

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let claims = GeoSealClaims {
        sfid_number: install.sfid_number,
    };
    let geo_seal = encrypt_geo_seal(
        install.install_secret.as_str(),
        &nonce_bytes,
        &claims,
        archive.archive_no.as_str(),
        install.cpms_pubkey.as_str(),
    )?;
    let geo_seal_hash = hash_hex(geo_seal.as_bytes());
    let sign_source = build_archive_sign_source(
        archive.archive_no.as_str(),
        archive.citizen_status.as_str(),
        archive.voting_eligible,
        sign_key.pubkey.as_str(),
        geo_seal_hash.as_str(),
    );
    let sig = sign_archive_payload_with_secret(&sign_key.secret_bytes, &sign_source)?;

    Ok(ArchiveQrPayload {
        proto: "SFID_CPMS_V1".to_string(),
        r#type: "ARCHIVE".to_string(),
        ano: archive.archive_no.clone(),
        cs: archive.citizen_status.clone(),
        ve: archive.voting_eligible,
        cpms_pubkey: sign_key.pubkey,
        geo_seal,
        sig,
    })
}

async fn active_archive_sign_key(
    state: &AppState,
) -> Result<QrSignKeyRuntime, (StatusCode, Json<ApiError>)> {
    let keys = crate::initialize::load_qr_sign_keys(state).await?;
    keys.into_iter()
        .find(|k| k.key_id == ARCHIVE_SIGN_KEY_ID && k.status == "ACTIVE")
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                "missing active archive sign key",
            )
        })
}

fn encrypt_geo_seal(
    install_secret: &str,
    nonce_bytes: &[u8; 12],
    claims: &GeoSealClaims,
    archive_no: &str,
    cpms_pubkey: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let key = derive_geo_seal_key(install_secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "geo_seal key invalid",
        )
    })?;
    let plain = serde_json::to_vec(claims).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "geo_seal json failed",
        )
    })?;
    let cipher_text = cipher
        .encrypt(
            Nonce::from_slice(nonce_bytes),
            Payload {
                msg: plain.as_ref(),
                aad: geo_seal_aad(archive_no, cpms_pubkey).as_bytes(),
            },
        )
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5003,
                "geo_seal encrypt failed",
            )
        })?;
    Ok(format!(
        "{}.{}.{}",
        GEO_SEAL_PREFIX,
        hex::encode(nonce_bytes),
        hex::encode(cipher_text)
    ))
}

fn derive_geo_seal_key(install_secret: &str) -> [u8; 32] {
    let digest = Blake2b256::digest(install_secret.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest[..32]);
    key
}

fn build_archive_sign_source(
    archive_no: &str,
    citizen_status: &str,
    voting_eligible: bool,
    cpms_pubkey: &str,
    geo_seal_hash: &str,
) -> String {
    format!(
        "sfid-cpms-v1|archive|{}|{}|{}|{}|{}",
        archive_no, citizen_status, voting_eligible, cpms_pubkey, geo_seal_hash
    )
}

fn geo_seal_aad(archive_no: &str, cpms_pubkey: &str) -> String {
    format!("sfid-cpms-v1|geo-seal|{}|{}", archive_no, cpms_pubkey)
}

pub(crate) fn validate_citizen_status(status: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    match status {
        "NORMAL" | "ABNORMAL" => Ok(()),
        _ => Err(err(StatusCode::BAD_REQUEST, 1001, "invalid citizen_status")),
    }
}

pub(crate) fn sign_archive_payload_with_secret(
    secret_bytes: &[u8],
    payload: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    if secret_bytes.len() != 32 {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid archive sign secret length",
        ));
    }

    let mini = MiniSecretKey::from_bytes(secret_bytes).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid archive sign secret key",
        )
    })?;
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    let sig = keypair.sign(signing_context(b"substrate").bytes(payload.as_bytes()));
    Ok(format!("0x{}", hex::encode(sig.to_bytes())))
}

fn hash_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(Blake2b256::digest(bytes)))
}
