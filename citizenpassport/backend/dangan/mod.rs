//! # 档案管理模块 (dangan)
//!
//! ARCHIVE 二维码载荷构造与签名、公民状态校验、有效期计算、年度状态导出。

mod export;
mod lifecycle;
mod materials;
mod routes;
mod stats;

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use axum::{http::StatusCode, Json, Router};
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use rand::{rngs::OsRng, RngCore};
use schnorrkel::{signing_context, MiniSecretKey};
use serde::Serialize;

use crate::{
    common::{err, ApiError, Archive},
    initialize::QrSignKeyRuntime,
    AppState,
};

pub(crate) use lifecycle::run_due_archive_hard_delete;
pub(crate) use materials::remove_archive_material_files;
pub(crate) use stats::adjust_archive_stats;

type Blake2b256 = Blake2b<U32>;

pub(crate) const CITIZEN_STATUS_NORMAL: &str = "NORMAL";
pub(crate) const CITIZEN_STATUS_REVOKED: &str = "REVOKED";
pub(crate) const ELECTION_SCOPE_PROVINCE: &str = "PROVINCE";
pub(crate) const ELECTION_SCOPE_CITY: &str = "CITY";
pub(crate) const ELECTION_SCOPE_TOWN: &str = "TOWN";
const ARCHIVE_SIGN_KEY_ID: &str = "ARCHIVE";
const GEO_SEAL_PREFIX: &str = "g1";

pub(crate) use export::{
    build_and_record_cpms_status_export, ensure_operator_annual_export_unlocked,
    status_export_state, CpmsStatusExportFile, CpmsStatusExportState,
};

pub(crate) fn router() -> Router<AppState> {
    routes::router().merge(materials::router())
}

/// CID_CPMS_V1 / ARCHIVE 档案二维码载荷。
#[derive(Clone, Serialize)]
pub(crate) struct ArchiveQrPayload {
    pub(crate) proto: String,
    pub(crate) r#type: String,
    pub(crate) archive_no: String,
    pub(crate) citizen_status: String,
    pub(crate) voting_eligible: bool,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) status_updated_at: i64,
    pub(crate) cpms_pubkey: String,
    pub(crate) geo_seal: String,
    pub(crate) wallet_address: String,
    pub(crate) wallet_pubkey: String,
    pub(crate) wallet_sig_alg: String,
    pub(crate) sig: String,
}

#[derive(Serialize)]
struct GeoSealClaims {
    /// 中文注释：机构 CID 号只证明本 CPMS 属于哪个市公安局授权安装。
    cid_number: String,
    /// 中文注释：居住地/出生地只放行政区代码，不放中文地名，且只在 geo_seal 密文中出现。
    residence: GeoSealRegionClaims,
    birthplace: GeoSealRegionClaims,
    election_scope_level: String,
}

#[derive(Serialize)]
struct GeoSealRegionClaims {
    province_code: String,
    city_code: Option<String>,
    town_code: Option<String>,
}

/// 构造 ARCHIVE 载荷（CID_CPMS_V1）。
///
/// 中文注释：二维码明文字段不放省、市、CPMS 机构号；归属只放入 `geo_seal`，
/// CID 使用安装授权中的 install_secret 才能解开。
pub(crate) async fn build_archive_qr_payload(
    state: &AppState,
    archive: &Archive,
) -> Result<ArchiveQrPayload, (StatusCode, Json<ApiError>)> {
    let wallet_address = archive
        .wallet_address
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "archive wallet required"))?;
    let wallet_pubkey = archive
        .wallet_pubkey
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "archive wallet required"))?;
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
    let election_scope_level = validate_election_scope_level(&archive.election_scope_level)?;
    let claims = GeoSealClaims {
        cid_number: install.cid_number,
        residence: scoped_region(
            &archive.province_code,
            &archive.city_code,
            &archive.town_code,
            &election_scope_level,
        ),
        birthplace: scoped_region(
            &archive.birth_province_code,
            &archive.birth_city_code,
            &archive.birth_town_code,
            &election_scope_level,
        ),
        election_scope_level,
    };
    let geo_seal = encrypt_geo_seal(
        install.install_secret.as_str(),
        &nonce_bytes,
        &claims,
        archive.archive_no.as_str(),
        install.cpms_pubkey.as_str(),
    )?;
    let geo_seal_hash = hash_hex(geo_seal.as_bytes());
    let sign_source = build_archive_sign_source(ArchiveSignSourceParts {
        archive_no: archive.archive_no.as_str(),
        citizen_status: archive.citizen_status.as_str(),
        voting_eligible: archive.voting_eligible,
        valid_from: archive.valid_from.as_str(),
        valid_until: archive.valid_until.as_str(),
        status_updated_at: archive.citizen_status_updated_at,
        cpms_pubkey: sign_key.pubkey.as_str(),
        geo_seal_hash: geo_seal_hash.as_str(),
        wallet_address,
        wallet_pubkey,
    });
    let sig = sign_archive_payload_with_secret(&sign_key.secret_bytes, &sign_source)?;

    Ok(ArchiveQrPayload {
        proto: "CID_CPMS_V1".to_string(),
        r#type: "ARCHIVE".to_string(),
        archive_no: archive.archive_no.clone(),
        citizen_status: archive.citizen_status.clone(),
        voting_eligible: archive.voting_eligible,
        valid_from: archive.valid_from.clone(),
        valid_until: archive.valid_until.clone(),
        status_updated_at: archive.citizen_status_updated_at,
        cpms_pubkey: sign_key.pubkey,
        geo_seal,
        wallet_address: wallet_address.to_string(),
        wallet_pubkey: wallet_pubkey.to_string(),
        wallet_sig_alg: archive.wallet_sig_alg.clone(),
        sig,
    })
}

pub(crate) async fn clear_archive_qr_payload(
    state: &AppState,
    archive_id: &str,
    updated_at: i64,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    // 中文注释：任何会改变 ARCHIVE 真实性的档案资料变更，都统一删除旧档案码，等待“更新”重新签发。
    sqlx::query(
        "UPDATE archives SET archive_qr_payload = '', updated_at = $1 WHERE archive_id = $2",
    )
    .bind(updated_at)
    .bind(archive_id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "clear archive qr failed",
        )
    })?;
    Ok(())
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

struct ArchiveSignSourceParts<'a> {
    archive_no: &'a str,
    citizen_status: &'a str,
    voting_eligible: bool,
    valid_from: &'a str,
    valid_until: &'a str,
    status_updated_at: i64,
    cpms_pubkey: &'a str,
    geo_seal_hash: &'a str,
    wallet_address: &'a str,
    wallet_pubkey: &'a str,
}

fn build_archive_sign_source(parts: ArchiveSignSourceParts<'_>) -> String {
    format!(
        "cid-cpms-v1|archive|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        parts.archive_no,
        parts.citizen_status,
        parts.voting_eligible,
        parts.valid_from,
        parts.valid_until,
        parts.status_updated_at,
        parts.cpms_pubkey,
        parts.geo_seal_hash,
        parts.wallet_address,
        parts.wallet_pubkey
    )
}

fn geo_seal_aad(archive_no: &str, cpms_pubkey: &str) -> String {
    format!("cid-cpms-v1|geo-seal|{}|{}", archive_no, cpms_pubkey)
}

pub(crate) fn validate_citizen_status(status: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    match status {
        CITIZEN_STATUS_NORMAL | CITIZEN_STATUS_REVOKED => Ok(()),
        _ => Err(err(StatusCode::BAD_REQUEST, 1001, "invalid citizen_status")),
    }
}

pub(crate) fn validate_election_scope_level(
    scope: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let normalized = scope.trim().to_ascii_uppercase();
    match normalized.as_str() {
        ELECTION_SCOPE_PROVINCE | ELECTION_SCOPE_CITY | ELECTION_SCOPE_TOWN => Ok(normalized),
        _ => Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "invalid election_scope_level",
        )),
    }
}

fn scoped_region(
    province_code: &str,
    city_code: &str,
    town_code: &str,
    election_scope_level: &str,
) -> GeoSealRegionClaims {
    let include_city = matches!(
        election_scope_level,
        ELECTION_SCOPE_CITY | ELECTION_SCOPE_TOWN
    );
    let include_town = election_scope_level == ELECTION_SCOPE_TOWN;
    GeoSealRegionClaims {
        province_code: province_code.trim().to_string(),
        city_code: include_city.then(|| city_code.trim().to_string()),
        town_code: include_town.then(|| town_code.trim().to_string()),
    }
}

pub(crate) fn effective_voting_eligible(
    citizen_status: &str,
    birth_date: NaiveDate,
    requested: Option<bool>,
    default_normal: bool,
    checked_at: i64,
) -> bool {
    // 中文注释：投票资格以公民状态和 16 周岁年龄线共同生效，避免未成年人进入 CID 投票账户。
    citizen_status == CITIZEN_STATUS_NORMAL
        && is_voting_age_at(checked_at, birth_date)
        && requested.unwrap_or(default_normal)
}

pub(crate) fn resolve_voting_eligible(
    citizen_status: &str,
    birth_date: NaiveDate,
    requested: Option<bool>,
    default_normal: bool,
    checked_at: i64,
) -> Result<bool, (StatusCode, Json<ApiError>)> {
    if citizen_status == CITIZEN_STATUS_NORMAL
        && requested == Some(true)
        && !is_voting_age_at(checked_at, birth_date)
    {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "voting_eligible requires age 16",
        ));
    }
    Ok(effective_voting_eligible(
        citizen_status,
        birth_date,
        requested,
        default_normal,
        checked_at,
    ))
}

pub(crate) fn is_voting_age_at(timestamp: i64, birth_date: NaiveDate) -> bool {
    age_at(timestamp, birth_date) >= 16
}

pub(crate) fn archive_valid_from(created_at: i64) -> String {
    archive_date(created_at).format("%Y-%m-%d").to_string()
}

pub(crate) fn archive_valid_until(created_at: i64, years: i32) -> String {
    let start = archive_date(created_at);
    let anniversary = NaiveDate::from_ymd_opt(start.year() + years, start.month(), start.day())
        .or_else(|| NaiveDate::from_ymd_opt(start.year() + years, 2, 28))
        .unwrap_or(start + Duration::days(365 * i64::from(years)));
    (anniversary - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string()
}

pub(crate) fn archive_validity_years(created_at: i64, birth_date: NaiveDate) -> i32 {
    // 中文注释：创建档案当天已满 16 周岁签发 10 年有效期，未满 16 周岁签发 5 年。
    if is_voting_age_at(created_at, birth_date) {
        10
    } else {
        5
    }
}

fn age_at(timestamp: i64, birth_date: NaiveDate) -> i32 {
    let today = archive_date(timestamp);
    let birthday_this_year =
        NaiveDate::from_ymd_opt(today.year(), birth_date.month(), birth_date.day())
            .or_else(|| NaiveDate::from_ymd_opt(today.year(), 2, 28))
            .unwrap_or(today);
    let mut age = today.year() - birth_date.year();
    if today < birthday_this_year {
        age -= 1;
    }
    age
}

fn archive_date(timestamp: i64) -> NaiveDate {
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(Utc::now)
        .date_naive()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_sign_source_includes_passport_validity() {
        let source = build_archive_sign_source(ArchiveSignSourceParts {
            archive_no: "ARCHIVE-1",
            citizen_status: "NORMAL",
            voting_eligible: true,
            valid_from: "2026-05-24",
            valid_until: "2036-05-23",
            status_updated_at: 1_779_580_800,
            cpms_pubkey: "0xpub",
            geo_seal_hash: "0xseal",
            wallet_address: "addr2027",
            wallet_pubkey: "0xwallet",
        });

        assert_eq!(
            source,
            "cid-cpms-v1|archive|ARCHIVE-1|NORMAL|true|2026-05-24|2036-05-23|1779580800|0xpub|0xseal|addr2027|0xwallet"
        );
    }

    #[test]
    fn archive_validity_defaults_to_ten_year_passport_window() {
        let start = DateTime::parse_from_rfc3339("2026-05-24T00:00:00Z")
            .unwrap()
            .timestamp();

        assert_eq!(archive_valid_from(start), "2026-05-24");
        assert_eq!(archive_valid_until(start, 10), "2036-05-23");
        assert_eq!(archive_valid_until(start, 5), "2031-05-23");
    }

    #[test]
    fn archive_validity_years_follow_age_boundary() {
        let now = DateTime::parse_from_rfc3339("2026-05-24T00:00:00Z")
            .unwrap()
            .timestamp();
        let under_16 = NaiveDate::from_ymd_opt(2010, 5, 25).unwrap();
        let exactly_16 = NaiveDate::from_ymd_opt(2010, 5, 24).unwrap();

        assert_eq!(archive_validity_years(now, under_16), 5);
        assert_eq!(archive_validity_years(now, exactly_16), 10);
        assert!(!is_voting_age_at(now, under_16));
        assert!(is_voting_age_at(now, exactly_16));
    }

    #[test]
    fn under_16_citizen_cannot_be_voting_eligible() {
        let now = DateTime::parse_from_rfc3339("2026-05-24T00:00:00Z")
            .unwrap()
            .timestamp();
        let under_16 = NaiveDate::from_ymd_opt(2010, 5, 25).unwrap();

        assert!(resolve_voting_eligible("NORMAL", under_16, Some(true), true, now).is_err());
        assert!(matches!(
            resolve_voting_eligible("NORMAL", under_16, None, true, now),
            Ok(false)
        ));
    }
}
