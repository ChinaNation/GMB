use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use axum::{http::StatusCode, Json};
use chrono::Utc;
use schnorrkel::{signing_context, MiniSecretKey};
use serde::{Deserialize, Serialize};
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
    let mut nonce = {
        let mut seq = state.sequence.write().await;
        let current = seq.entry(seq_key.clone()).or_insert(1);
        let value = *current;
        *current += 1;
        value
    };

    for _ in 0..ARCHIVE_NO_MAX_RETRY {
        let random9 = generate_random9(terminal_id, admin_pubkey, nonce);
        let check_digit =
            archive_checksum_digit(province_code, city_code, &random9, created_date_yyyymmdd);
        let archive_no = format!(
            "{}{}{}{}{}",
            province_code, city_code, check_digit, random9, created_date_yyyymmdd
        );

        let exists = {
            let archives = state.archives.read().await;
            archives.values().any(|a| a.archive_no == archive_no)
        };
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
    let digest = blake3::hash(payload.as_bytes());
    let sum: u32 = digest.as_bytes().iter().map(|&b| b as u32).sum();
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
    let install = state.install.read().await;
    let site_sfid = install
        .site_sfid
        .clone()
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "cpms not initialized"))?;
    if install.qr_sign_keys.is_empty() {
        return Err(err(
            StatusCode::CONFLICT,
            4005,
            "missing qr sign keys after initialization",
        ));
    }
    Ok((site_sfid, install.qr_sign_keys.clone()))
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

    let mini = MiniSecretKey::from_bytes(&secret_bytes).map_err(|_| {
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
