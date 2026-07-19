//! 机构管理员「链下私密资料」数据模型。
//!
//! 管理员链上人员记录包含账户、姓、名；授权只使用账户，岗位、任期和来源归 entity。
//! 本模型只承接链下私密档案(部门/联系方式/证件照/passkey 绑定等)
//! 落库到 `institution_admins` 省级分区表。管理员资格只从链上 CID-key `admins` 读取。
//!
//! 复合 key = (province_code, cid_number, admin_account):
//! - cid_number:管理员所属机构身份 ID;
//! - admin_account:进链的管理员钱包账户。

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::scope::HasProvinceCity;

/// 机构管理员链下私密资料(单条 = 一个机构下的一个管理员账户)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionAdmin {
    /// 所属机构身份 ID。
    pub cid_number: String,
    /// 省代码(分区键)。
    pub province_code: String,
    /// 市代码。市级及以下机构填写;省/国家级机构可空。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city_code: Option<String>,
    /// 进链的管理员钱包账户。
    pub admin_account: String,
    /// 链上管理员姓。
    pub family_name: String,
    /// 链上管理员名。
    pub given_name: String,
    /// 部门。链下私密档案。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_department: Option<String>,
    /// 岗位。链下私密档案。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_job: Option<String>,
    /// 联系电话。链下私密档案。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_contact_phone: Option<String>,
    /// 联系邮箱。链下私密档案。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_contact_email: Option<String>,
    /// 证件照服务端存储路径。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_photo_path: Option<String>,
    /// 证件照原始文件名。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_photo_name: Option<String>,
    /// 证件照 MIME 类型。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_photo_mime: Option<String>,
    /// 证件照大小(字节)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_photo_size: Option<u64>,
    /// 绑定的 WebAuthn passkey 凭证 ID。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_passkey_credential_id: Option<String>,
    /// 来源单据 ID(链下资料来源追溯)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_source_id: Option<String>,
    /// 链下资料状态。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_status: Option<String>,
    /// 链下资料最近更新时间。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_updated_at: Option<DateTime<Utc>>,
    /// 创建人 pubkey。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// 操作日志 ID(链下操作审计追溯)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_log_id: Option<String>,
    pub created_at: DateTime<Utc>,
    /// 派生:所属省名称(由 province_code 经 china.sqlite 反查;库里不存名字)。
    #[serde(default)]
    pub province_name: String,
    /// 派生:所属市名称(由 city_code 经 china.sqlite 反查;库里不存名字)。
    #[serde(default)]
    pub city_name: String,
}

impl HasProvinceCity for InstitutionAdmin {
    fn province(&self) -> &str {
        &self.province_name
    }
    fn city(&self) -> &str {
        &self.city_name
    }
}
