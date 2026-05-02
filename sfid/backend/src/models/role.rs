//! 中文注释:管理员角色 / 状态 / 实体 + Operator 列表与维护接口 DTO。
//! AdminRole::KeyAdmin 在 ADR-008 决议下进入 Step 1 phase23e 子卡删除,本文件保留
//! 是为了 phase23a/b/c/d 期间所有 caller 仍可正常 build。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 中文注释:三种管理员角色(原命名铁律,见 feedback_sfid_three_roles_naming.md)
//   - KeyAdmin  → 密钥管理员(全国 3 人)      目录 key-admins/  ★ ADR-008 决议:Step 1 全删
//   - ShengAdmin → 省级管理员(每省 3 人 main/backup_1/backup_2,自治) 目录 sheng_admins/
//   - ShiAdmin   → 市级管理员(每市 N 人)      目录 shi_admins/
// 序列化为 KEY_ADMIN / SHENG_ADMIN / SHI_ADMIN,数据库字段值同。
// KeyAdmin 仅在 Step 1 过渡期保留,Phase 2 删除。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminRole {
    KeyAdmin,
    ShengAdmin,
    ShiAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminUser {
    pub(crate) id: u64,
    pub(crate) admin_pubkey: String,
    #[serde(default)]
    pub(crate) admin_name: String,
    pub(crate) role: AdminRole,
    pub(crate) status: AdminStatus,
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
    /// ShiAdmin 所属的市名称（仅 ShiAdmin 必填，其他角色为空字符串）
    #[serde(default)]
    pub(crate) city: String,
    /// 中文注释:仅 ShengAdmin 使用。AES-256-GCM 加密的省签名私钥种子(32 字节明文)。
    /// 格式:base64(nonce_12B || ciphertext || tag_16B)
    /// Wrap key:HKDF-SHA256(SfidMainSeed, salt, info),见 key_admins::sheng_signer_cache。
    /// 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 引入。
    #[serde(default)]
    pub(crate) encrypted_signing_privkey: Option<String>,
    /// 中文注释:仅 ShengAdmin 使用。对应签名公钥 hex(便于对账/UI 显示)。
    #[serde(default)]
    pub(crate) signing_pubkey: Option<String>,
    /// 签名密钥生成时间(仅 ShengAdmin,bootstrap 时写入)。
    #[serde(default)]
    pub(crate) signing_created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub(crate) struct OperatorRow {
    pub(crate) id: u64,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) role: AdminRole,
    pub(crate) status: AdminStatus,
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) city: String,
}

#[derive(Serialize)]
pub(crate) struct OperatorListOutput {
    pub(crate) total: usize,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) rows: Vec<OperatorRow>,
}

// 机构管理员对外行（API 序列化）。
//
// SFID 业务语义：机构是永久存在的（43 个省份固定），机构管理员只是当前
// 替机构发声的人；不存在"停用"的机构管理员（被替换即彻底失效）。
// 因此对外暴露的行**不带 status 字段**。
#[derive(Serialize)]
pub(crate) struct ShengAdminRow {
    pub(crate) id: u64,
    pub(crate) province: String,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) built_in: bool,
    pub(crate) created_at: DateTime<Utc>,
    /// 最近一次更新时间（含签名密钥 bootstrap），None 表示从未更新
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
    // 链上签名 pubkey：None 表示该省登录管理员尚未首次 bootstrap
    #[serde(default)]
    pub(crate) signing_pubkey: Option<String>,
    /// 签名密钥生成时间
    #[serde(default)]
    pub(crate) signing_created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub(crate) struct CreateOperatorInput {
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    /// ShiAdmin 所属的市，必填，且必须属于 created_by 对应机构管理员的省份（不可为省辖市）
    pub(crate) city: String,
    /// 可选：指定该 operator 归属的机构管理员 pubkey。
    /// 仅 KeyAdmin 可指定，且必须是已存在的 ShengAdmin。
    /// ShengAdmin 调用时若指定则必须等于自己 pubkey，否则 403。
    /// 不指定则默认为调用者自身。
    #[serde(default)]
    pub(crate) created_by: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ReplaceShengAdminInput {
    pub(crate) admin_pubkey: String,
    /// 新省级管理员姓名，可选；未提供时保留原值
    #[serde(default)]
    pub(crate) admin_name: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateOperatorInput {
    pub(crate) admin_pubkey: Option<String>,
    pub(crate) admin_name: Option<String>,
    /// 可选：修改 ShiAdmin 所属的市，必须属于该 operator 所属机构的省份（不可为省辖市）
    #[serde(default)]
    pub(crate) city: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateOperatorStatusInput {
    pub(crate) status: AdminStatus,
}
