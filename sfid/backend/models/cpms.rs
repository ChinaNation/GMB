//! 中文注释:CPMS 站点凭证、安装 token、QR1/QR2/QR3/QR4 载荷以及匿名证书 DTO。

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
    pub(crate) site_sfid: String,
    #[serde(default)]
    pub(crate) install_token: String,
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
    pub(crate) institution_code: String,
    #[serde(default)]
    pub(crate) institution_name: String,
    #[serde(default)]
    pub(crate) qr1_payload: String,
    /// QR3 匿名证书载荷(QR2 注册成功后持久化,吊销/重发时清除)
    #[serde(default)]
    pub(crate) qr3_payload: Option<String>,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_by: Option<String>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct CpmsRegisterScanInput {
    pub(crate) qr_payload: String,
}

#[derive(Deserialize)]
pub(crate) struct GenerateCpmsInstitutionSfidInput {
    pub(crate) province: Option<String>,
    pub(crate) city: String,
    pub(crate) institution: String,
    #[serde(default)]
    pub(crate) institution_name: Option<String>,
}

/// QR2 注册请求输入。
#[derive(Deserialize)]
pub(crate) struct CpmsRegisterInput {
    pub(crate) qr_payload: String,
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
    pub(crate) site_sfid: String,
    pub(crate) install_token_status: InstallTokenStatus,
    pub(crate) status: CpmsSiteStatus,
    pub(crate) version: u64,
    pub(crate) province_code: String,
    pub(crate) admin_province: String,
    pub(crate) city_name: String,
    pub(crate) institution_code: String,
    pub(crate) institution_name: String,
    pub(crate) qr1_payload: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) qr3_payload: Option<String>,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_by: Option<String>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

/// QR2 解析后的注册请求（SFID_CPMS_V1）。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct CpmsRegisterReqPayload {
    #[serde(default)]
    pub(crate) proto: String,
    #[serde(alias = "qr_type")]
    pub(crate) r#type: String,
    pub(crate) sfid: String,
    pub(crate) token: String,
    pub(crate) blind: String,
}

/// QR4 解析后的档案业务载荷（SFID_CPMS_V1）。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct CpmsArchiveQrPayload {
    #[serde(default)]
    pub(crate) proto: String,
    #[serde(alias = "qr_type")]
    pub(crate) r#type: String,
    pub(crate) prov: String,
    pub(crate) ano: String,
    pub(crate) cs: String,
    pub(crate) ve: bool,
    pub(crate) cert: AnonCert,
    pub(crate) sig: String,
}

/// 匿名证书。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AnonCert {
    pub(crate) prov: String,
    pub(crate) pk: String,
    pub(crate) sig: String,
    #[serde(default)]
    pub(crate) mr: Option<String>,
}

/// 生成 SFID + QR1 的输出。
#[derive(Serialize)]
pub(crate) struct GenerateCpmsInstallOutput {
    pub(crate) site_sfid: String,
    pub(crate) qr1_payload: String,
}

/// 处理 QR2 注册请求后返回 QR3。
#[derive(Serialize)]
pub(crate) struct CpmsRegisterOutput {
    pub(crate) qr3_payload: String,
}

/// 档案录入结果。
#[derive(Serialize)]
pub(crate) struct CpmsArchiveImportOutput {
    pub(crate) archive_no: String,
    pub(crate) province_code: String,
    pub(crate) status: &'static str,
}

#[derive(Deserialize)]
pub(crate) struct CpmsStatusScanInput {
    pub(crate) qr_payload: String,
}

#[derive(Serialize)]
pub(crate) struct CpmsStatusScanOutput {
    pub(crate) archive_no: String,
    pub(crate) status: super::citizen::CitizenStatus,
    pub(crate) message: &'static str,
}
