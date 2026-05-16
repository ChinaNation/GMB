//! 中文注释:CPMS 安装授权、INSTALL 安装码、ARCHIVE 档案二维码和省市归属 DTO。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// QR4 档案录入输入。
#[derive(Deserialize)]
pub(crate) struct CpmsArchiveImportInput {
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

/// QR4 解析后的档案业务载荷（SFID_CPMS_V1）。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct CpmsArchiveQrPayload {
    #[serde(default)]
    pub(crate) proto: String,
    pub(crate) r#type: String,
    pub(crate) ano: String,
    pub(crate) cs: String,
    pub(crate) ve: bool,
    pub(crate) cpms_pubkey: String,
    pub(crate) geo_seal: String,
    pub(crate) sig: String,
}

/// CPMS 档案二维码中只允许 SFID 解开的归属密文内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CpmsGeoSealClaims {
    /// 中文注释：归属密文只放机构 SFID 号；省市由 SFID 从 sfid_number 解码。
    pub(crate) sfid_number: String,
}

/// SFID 验证 CPMS 档案二维码后的可信结果。
#[derive(Debug, Clone)]
pub(crate) struct VerifiedCpmsArchive {
    pub(crate) archive_no: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) sfid_number: String,
    pub(crate) cpms_pubkey_hash: String,
    pub(crate) geo_seal_hash: String,
}

/// 生成 SFID + QR1 的输出。
#[derive(Serialize)]
pub(crate) struct GenerateCpmsInstallOutput {
    pub(crate) sfid_number: String,
    pub(crate) qr1_payload: String,
}

/// 档案录入结果。
#[derive(Serialize)]
pub(crate) struct CpmsArchiveImportOutput {
    pub(crate) archive_no: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) sfid_number: String,
    pub(crate) status: &'static str,
}

#[derive(Deserialize)]
pub(crate) struct CpmsStatusScanInput {
    pub(crate) qr_payload: String,
}

#[derive(Serialize)]
pub(crate) struct CpmsStatusScanOutput {
    pub(crate) archive_no: String,
    pub(crate) status: crate::citizens::model::CitizenStatus,
    pub(crate) message: &'static str,
}
