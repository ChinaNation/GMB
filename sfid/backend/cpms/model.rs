//! 中文注释:CPMS 安装授权、INSTALL 安装码、ARCHIVE 档案二维码和省市归属 DTO。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::citizens::model::CitizenStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CpmsSiteStatus {
    Pending,
    Active,
    Disabled,
    Revoked,
}

fn default_cpms_site_status() -> CpmsSiteStatus {
    CpmsSiteStatus::Active
}

fn default_cpms_site_version() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum InstallTokenStatus {
    Pending,
    Used,
    Revoked,
}

fn default_install_token_status() -> InstallTokenStatus {
    InstallTokenStatus::Pending
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CpmsSiteKeys {
    /// CPMS 机构的 SFID 号。历史内部字段仍叫 site_sfid,外部协议统一输出 sfid_number。
    pub(crate) site_sfid: String,
    #[serde(default)]
    pub(crate) install_token: String,
    #[serde(default)]
    pub(crate) install_secret: String,
    #[serde(default)]
    pub(crate) install_secret_hash: String,
    #[serde(default = "default_install_token_status")]
    pub(crate) install_token_status: InstallTokenStatus,
    #[serde(default = "default_cpms_site_status")]
    pub(crate) status: CpmsSiteStatus,
    #[serde(default = "default_cpms_site_version")]
    pub(crate) version: u64,
    #[serde(default)]
    pub(crate) province_code: String,
    pub(crate) admin_province: String,
    #[serde(default)]
    pub(crate) city_name: String,
    #[serde(default)]
    pub(crate) city_code: String,
    #[serde(default)]
    pub(crate) institution_code: String,
    #[serde(default)]
    pub(crate) institution_name: String,
    #[serde(default)]
    pub(crate) qr1_payload: String,
    #[serde(default)]
    pub(crate) cpms_pubkey_hash: Option<String>,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_by: Option<String>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub(crate) struct GenerateCpmsInstallInput {
    pub(crate) province: Option<String>,
    pub(crate) city: String,
    pub(crate) institution: String,
}

/// ARCHIVE 档案码验真输入。
#[derive(Deserialize)]
pub(crate) struct CpmsArchiveVerifyInput {
    pub(crate) qr_payload: String,
}

#[derive(Deserialize)]
pub(crate) struct UpdateCpmsSiteStatusInput {
    pub(crate) reason: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CpmsKeysListOutput {
    pub(crate) total: usize,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) rows: Vec<CpmsSiteKeysListRow>,
}

#[derive(Serialize)]
pub(crate) struct CpmsSiteKeysListRow {
    pub(crate) sfid_number: String,
    pub(crate) install_token_status: InstallTokenStatus,
    pub(crate) status: CpmsSiteStatus,
    pub(crate) version: u64,
    pub(crate) province_code: String,
    pub(crate) admin_province: String,
    pub(crate) city_name: String,
    pub(crate) city_code: String,
    pub(crate) institution_code: String,
    pub(crate) institution_name: String,
    pub(crate) qr1_payload: String,
    pub(crate) cpms_pubkey_bound: bool,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_by: Option<String>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

/// 档案码解析后的档案业务载荷（SFID_CPMS_V1）。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct CpmsArchiveCodePayload {
    #[serde(default)]
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
    #[serde(default = "default_wallet_sig_alg")]
    pub(crate) wallet_sig_alg: String,
    pub(crate) sig: String,
}

fn default_wallet_sig_alg() -> String {
    "sr25519".to_string()
}

/// CPMS 档案二维码中只允许 SFID 解开的归属密文内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CpmsGeoSealClaims {
    /// 中文注释：归属密文只放机构身份ID；省市由 SFID 从 sfid_number 解码。
    pub(crate) sfid_number: String,
}

/// SFID 验证 CPMS 档案二维码后的可信结果。
#[derive(Debug, Clone)]
pub(crate) struct VerifiedCpmsArchive {
    pub(crate) archive_no: String,
    pub(crate) citizen_status: CitizenStatus,
    pub(crate) voting_eligible: bool,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) status_updated_at: i64,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) sfid_number: String,
    pub(crate) wallet_address: String,
    pub(crate) wallet_pubkey: String,
    pub(crate) wallet_sig_alg: String,
}

/// 生成 SFID + QR1 的输出。
#[derive(Serialize)]
pub(crate) struct GenerateCpmsInstallOutput {
    pub(crate) sfid_number: String,
    pub(crate) qr1_payload: String,
}

/// 档案码验真结果。正式公民绑定必须在 citizens 模块完成。
#[derive(Serialize)]
pub(crate) struct CpmsArchiveVerifyOutput {
    pub(crate) archive_no: String,
    pub(crate) citizen_status: crate::citizens::model::CitizenStatus,
    pub(crate) voting_eligible: bool,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) status_updated_at: i64,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) sfid_number: String,
    pub(crate) status: &'static str,
}
