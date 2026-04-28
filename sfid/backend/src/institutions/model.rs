//! 机构/账户两层数据模型
//!
//! 中文注释:链端 `SfidRegisteredAddress::<T>(sfid_id, name) → duoqian_address`
//! 是 DoubleMap,一个 sfid_id 下可挂多个 name,每个 name 派生独立多签地址。
//! sfid 系统这里对应拆两层:
//!
//! - `MultisigInstitution`:每个 sfid_id 唯一,存机构展示信息(institution_name 等),
//!   **不**进链。
//! - `MultisigAccount`:以 `(sfid_id, account_name)` 为复合 key,account_name 是
//!   **进链的 name**,一个机构下可挂多个。
//!
//! 详见 `feedback_institutions_two_layer.md`。

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::MultisigChainStatus;
use crate::scope::HasProvinceCity;
use crate::sfid::InstitutionCategory;

/// 机构(每个 sfid_id 唯一)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigInstitution {
    /// SFID 号,参与链上派生。
    pub sfid_id: String,
    /// 机构展示名称(如"广州市公安局"),**不进链**,只在 sfid 系统内部显示。
    ///
    /// 两步式创建(2026-04-19):
    ///   - 私权机构(SFR/FFR)第一步创建时为 `None`,由详情页 `update_institution` 补填
    ///   - 公权机构(GFR)/公安局创建时必传,不会为 `None`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub institution_name: Option<String>,
    /// 机构分类(公安局/公权机构/私权机构)。
    pub category: InstitutionCategory,
    /// 主体属性(GFR/SFR/FFR)。
    pub a3: String,
    /// 盈利属性("0"/"1")。
    pub p1: String,
    /// 所属省(名称,如"安徽省")。
    pub province: String,
    /// 所属市(名称,如"合肥市")。
    pub city: String,
    /// 所属省代码(r5 前 2 字符)。
    pub province_code: String,
    /// 所属市代码(r5 后 3 字符)。任务卡 6 新增:
    /// 作为公安局对账的稳定主键,市名改动时保持不变。
    /// 老记录在后端启动时由 `backfill_and_reconcile_public_security` 补齐。
    #[serde(default)]
    pub city_code: String,
    /// 机构类型代码(ZF/LF/SF/...)。
    pub institution_code: String,
    /// 私法人子类型(仅 A3=SFR 时有值)。
    /// 取值:SOLE_PROPRIETORSHIP / PARTNERSHIP / LIMITED_LIABILITY / JOINT_STOCK / NON_PROFIT
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<String>,
    /// 所属法人机构 SFID(**仅 A3=FFR 非法人必填**)。
    /// 指向一个私法人(SFR)或公法人(GFR)机构的 sfid_id。
    /// 非法人机构必须挂在某个法人机构下,全国范围可选。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_sfid_id: Option<String>,
    /// sfid_id 是否已通过首次 QR1 生成固化。
    /// reconcile 批量创建时为 false,首次生成 QR1 时设为 true 并替换 sfid_id,
    /// 此后永久不变。
    #[serde(default)]
    pub sfid_finalized: bool,
    /// 创建人 pubkey。
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl HasProvinceCity for MultisigInstitution {
    fn province(&self) -> &str {
        &self.province
    }
    fn city(&self) -> &str {
        &self.city
    }
}

/// 机构下的多签账户(复合 key = (sfid_id, account_name))。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigAccount {
    /// 所属机构的 sfid_id。
    pub sfid_id: String,
    /// 账户名称,**进链的 name 字段**。同 sfid_id 下必须唯一。
    pub account_name: String,
    /// 链上派生的多签地址(hex, 不含 0x 前缀)。上链成功后填入。
    pub duoqian_address: Option<String>,
    /// 链上状态。
    #[serde(default)]
    pub chain_status: MultisigChainStatus,
    pub chain_tx_hash: Option<String>,
    pub chain_block_number: Option<u64>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

/// 复合 key:`(sfid_id, account_name)`。
pub type AccountKey = (String, String);

/// 把复合 key 序列化为 "sfid_id|account_name" 字符串(用作 HashMap 的 String key)。
pub fn account_key_to_string(sfid_id: &str, account_name: &str) -> String {
    format!("{sfid_id}|{account_name}")
}

/// 从 "sfid_id|account_name" 字符串解析回元组。
pub fn account_key_from_string(s: &str) -> Option<AccountKey> {
    let mut parts = s.splitn(2, '|');
    let sfid_id = parts.next()?.to_string();
    let account_name = parts.next()?.to_string();
    Some((sfid_id, account_name))
}

/// 机构资料库文档(注册文件/许可证/章程等)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionDocument {
    /// 自增文档 ID。
    pub id: u64,
    /// 所属机构 sfid_id。
    pub sfid_id: String,
    /// 原始文件名。
    pub file_name: String,
    /// 文档类型(公司章程/营业许可证/股东会决议/法人授权书/其他)。
    pub doc_type: String,
    /// 文件大小(字节)。
    pub file_size: u64,
    /// 服务端存储路径(相对于 data/documents/)。
    pub file_path: String,
    /// 上传人 pubkey。
    pub uploaded_by: String,
    pub uploaded_at: DateTime<Utc>,
}

/// 文档类型枚举值。
pub const VALID_DOC_TYPES: &[&str] =
    &["公司章程", "营业许可证", "股东会决议", "法人授权书", "其他"];

// ─── 请求/响应 DTO ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateInstitutionInput {
    pub a3: String,
    pub p1: Option<String>,
    pub province: Option<String>,
    pub city: String,
    pub institution: String,
    /// 两步式:私权(SFR/FFR)不传,由详情页 `update_institution` 补填;
    /// 公权(GFR)/公安局必传
    pub institution_name: Option<String>,
    /// 私法人子类型。两步式改造后:**创建阶段不再接受** sub_type,
    /// 统一由 `update_institution` 在详情页设置。保留字段仅为向后兼容旧请求(忽略)。
    #[serde(default)]
    pub sub_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateInstitutionOutput {
    pub sfid_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution_name: Option<String>,
    pub category: InstitutionCategory,
}

/// 两步式第二步:机构详情页提交的可编辑字段。
#[derive(Debug, Deserialize)]
pub struct UpdateInstitutionInput {
    pub institution_name: Option<String>,
    pub sub_type: Option<String>,
    /// 所属法人 sfid_id(仅 FFR 可设置;SFR/GFR 传值会被拒)
    #[serde(default)]
    pub parent_sfid_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub account_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountOutput {
    pub sfid_id: String,
    pub account_name: String,
    pub chain_status: MultisigChainStatus,
    pub chain_tx_hash: Option<String>,
    pub chain_block_number: Option<u64>,
    pub duoqian_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InstitutionListRow {
    pub sfid_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution_name: Option<String>,
    pub category: InstitutionCategory,
    pub a3: String,
    pub p1: String,
    pub province: String,
    pub city: String,
    pub institution_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_sfid_id: Option<String>,
    pub account_count: usize,
    pub created_at: DateTime<Utc>,
    /// 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users)
    /// 命中:admin_name;未命中:None(前端显示为"未知")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_name: Option<String>,
    /// 创建者角色:"KEY_ADMIN" / "SHENG_ADMIN" / "SHI_ADMIN" / None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_role: Option<String>,
}

/// 法人机构搜索结果项(用于 FFR 详情页"所属法人"选择器)
#[derive(Debug, Serialize)]
pub struct ParentInstitutionRow {
    pub sfid_id: String,
    pub institution_name: String,
    pub a3: String,
    /// 私法人子类型(仅 a3=SFR 有值);FFR 前端用此判断父 SFR 是否 JOINT_STOCK 以开放清算行设置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<String>,
    pub category: InstitutionCategory,
    pub province: String,
    pub city: String,
}

#[derive(Debug, Serialize)]
pub struct InstitutionDetailOutput {
    pub institution: MultisigInstitution,
    pub accounts: Vec<MultisigAccount>,
    /// 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_name: Option<String>,
    /// 创建者角色:"KEY_ADMIN" / "SHENG_ADMIN" / "SHI_ADMIN"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_role: Option<String>,
}
