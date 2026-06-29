//! 机构/账户两层数据模型
//!
//! 中文注释:链端 `CidRegisteredAccount::<T>(cid_number, name) → account`
//! 是 DoubleMap,一个 cid_number 下可挂多个 name,每个 name 派生独立多签账户。
//! cid 系统这里对应拆两层:
//!
//! - `Institution`:每个 cid_number 唯一,存机构展示信息(cid_full_name 等),
//!   **不**进链。
//! - `InstitutionAccount`:以 `(cid_number, account_name)` 为复合 key,account_name 是
//!   **进链的 name**,一个机构下可挂多个。
//!
//! 详见 `feedback_institutions_two_layer.md`。

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::cid::InstitutionCategory;
use crate::scope::HasProvinceCity;

pub const EDUCATION_TYPE_NATIONAL_CITIZEN_EDU_COMMITTEE: &str = "NATIONAL_CITIZEN_EDU_COMMITTEE";
pub const EDUCATION_TYPE_CITY_CITIZEN_EDU_COMMITTEE: &str = "CITY_CITIZEN_EDU_COMMITTEE";
pub const EDUCATION_TYPE_EARLY_SCHOOL: &str = "EARLY_SCHOOL";
pub const EDUCATION_TYPE_PRIMARY_SCHOOL: &str = "PRIMARY_SCHOOL";
pub const EDUCATION_TYPE_SECONDARY_SCHOOL: &str = "SECONDARY_SCHOOL";
pub const EDUCATION_TYPE_UNIVERSITY: &str = "UNIVERSITY";

// 中文注释:基础教育级别(初学/小学/中学)。大学是独立机构码(GUN/SUN),不属 education_type 级别。
pub const EDUCATION_SCHOOL_TYPES: &[&str] = &[
    EDUCATION_TYPE_EARLY_SCHOOL,
    EDUCATION_TYPE_PRIMARY_SCHOOL,
    EDUCATION_TYPE_SECONDARY_SCHOOL,
];

pub const EDUCATION_COMMITTEE_TYPES: &[&str] = &[
    EDUCATION_TYPE_NATIONAL_CITIZEN_EDU_COMMITTEE,
    EDUCATION_TYPE_CITY_CITIZEN_EDU_COMMITTEE,
];

pub fn is_education_school_type(value: &str) -> bool {
    EDUCATION_SCHOOL_TYPES.contains(&value)
}

fn default_subject_status() -> String {
    "ACTIVE".to_string()
}

// ── 账户链上状态 ───────────────────────────────────────

/// 机构账户链上状态。
///
/// 中文注释:账户是否激活只以链上事实为准。CID 创建账户时只是登记
/// `(cid_number, account_name)`,默认 `NotOnChain`;链上机构注册或新增账户成功后,
/// 由同步接口写成 `ActiveOnChain`;链上注销后写成 `RevokedOnChain`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MultisigChainStatus {
    NotOnChain,
    PendingOnChain,
    ActiveOnChain,
    RevokedOnChain,
}

impl Default for MultisigChainStatus {
    fn default() -> Self {
        Self::NotOnChain
    }
}

/// 机构(每个 cid_number 唯一)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    /// CID 号,参与链上派生。
    pub cid_number: String,
    /// 机构全称。列表可用简称优先展示,详情页同时展示全称和简称。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid_full_name: Option<String>,
    /// 机构简称。确定性公权机构必须写入规范简称,不得重复写全称。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid_short_name: Option<String>,
    /// 主体业务状态。机构列表和详情只展示 ACTIVE / REVOKED,不把链上状态混成业务状态。
    #[serde(default = "default_subject_status")]
    pub status: String,
    /// 机构展示分类(公权机构/私权机构)。法律主体类型由机构码和父级属性单独判定。
    pub category: InstitutionCategory,
    /// 盈利属性("0"/"1")。
    pub p1: String,
    /// 所属省名称(如"安徽省")。
    pub province_name: String,
    /// 所属市名称(如"合肥市")。
    pub city_name: String,
    /// 所属镇名称。非镇目录机构为空。
    #[serde(default)]
    pub town_name: String,
    /// 所属省代码(r5 前 2 字符)。
    pub province_code: String,
    /// 所属市代码(r5 后 3 字符)。自动公权目录的稳定地域键,市名改动时保持不变。
    #[serde(default)]
    pub city_code: String,
    /// 所属镇代码。只有镇目录机构填写。
    #[serde(default)]
    pub town_code: String,
    /// 机构类型代码(ZF/LF/SF/...)。
    pub institution_code: String,
    /// 教育机构业务分类。只用于教育 tab 分类,不参与 CID 号生成。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub education_type: Option<String>,
    /// 私权机构类型。仅私权机构有值,取值见 `private/common::PrivateType`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_type: Option<String>,
    /// 合伙企业形态。仅 private_type=PARTNERSHIP 时有值:GENERAL / LIMITED。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partnership_kind: Option<String>,
    /// 是否具有法人资格。仅私权机构有值;公权机构由主体属性 G 表达法人资格。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_legal_personality: Option<bool>,
    /// 从属关系引用。字段值始终是另一个机构已有的 `cid_number`,不是第二套身份 ID。
    /// - 需要挂靠的非法人组织(UNIN):指向所属法人。
    /// 个体经营(SFGT)和无限合伙(SFGP)是独立非法人,不填写本字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_cid_number: Option<String>,
    /// 法定代表人姓名。初始化目录机构允许为空;机构资料编辑保存时必须补齐。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_rep_name: Option<String>,
    /// 法定代表人身份ID,必须指向正常状态公民的 cid_number。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_rep_cid_number: Option<String>,
    /// 法定代表人证件照服务端存储路径。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_rep_photo_path: Option<String>,
    /// 法定代表人证件照原始文件名。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_rep_photo_name: Option<String>,
    /// 法定代表人证件照 MIME 类型。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_rep_photo_mime: Option<String>,
    /// 法定代表人证件照大小(字节)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_rep_photo_size: Option<u64>,
    /// 创建人 pubkey。
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl HasProvinceCity for Institution {
    fn province(&self) -> &str {
        &self.province_name
    }
    fn city(&self) -> &str {
        &self.city_name
    }
}

/// 机构下的多签账户(复合 key = (cid_number, account_name))。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionAccount {
    /// 所属机构的 cid_number。
    pub cid_number: String,
    /// 账户名称,**进链的 name 字段**。同 cid_number 下必须唯一。
    pub account_name: String,
    /// 链上派生的多签账户(hex, 不含 0x 前缀)。上链成功后填入。
    pub account: Option<String>,
    /// 链上状态。
    #[serde(default)]
    pub chain_status: MultisigChainStatus,
    /// 最近一次链上状态同步时间。CID 后台不直接激活账户,只记录同步事实。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_synced_at: Option<DateTime<Utc>>,
    pub chain_tx_hash: Option<String>,
    pub chain_block_number: Option<u64>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

/// 复合 key:`(cid_number, account_name)`。
pub type AccountKey = (String, String);

/// 把复合 key 序列化为 "cid_number|account_name" 字符串(用作 HashMap 的 String key)。
pub fn account_key_to_string(cid_number: &str, account_name: &str) -> String {
    format!("{cid_number}|{account_name}")
}

/// 从 "cid_number|account_name" 字符串解析回元组。
pub fn account_key_from_string(s: &str) -> Option<AccountKey> {
    let mut parts = s.splitn(2, '|');
    let cid_number = parts.next()?.to_string();
    let account_name = parts.next()?.to_string();
    Some((cid_number, account_name))
}

/// 机构资料库文档(注册文件/许可证/章程等)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionDocument {
    /// 自增文档 ID。
    pub id: u64,
    /// 所属机构 cid_number。
    pub cid_number: String,
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
    pub p1: Option<String>,
    pub province_name: Option<String>,
    pub city_name: String,
    pub institution: String,
    /// 教育机构业务分类。仅 `institution=JY` 的教育入口使用,不参与 CID 号生成。
    #[serde(default)]
    pub education_type: Option<String>,
    /// 机构全称。私权、公权和教育新增都应在创建阶段写入 cid_full_name。
    pub cid_full_name: Option<String>,
    /// 所属法人身份ID。仅需要挂靠的非法人(F)使用;个体经营和无限合伙是独立非法人,
    /// 不接受所属法人。
    #[serde(default)]
    pub parent_cid_number: Option<String>,
    pub cid_short_name: Option<String>,
    /// 私权机构类型。私权入口创建时必传,由后端锁定主体属性和机构码。
    #[serde(default)]
    pub private_type: Option<String>,
    /// 合伙类型。private_type=PARTNERSHIP 时必传,其它类型不接收。
    #[serde(default)]
    pub partnership_kind: Option<String>,
    /// 法定代表人姓名,新增机构必填。
    #[serde(default)]
    pub legal_rep_name: Option<String>,
    /// 法定代表人身份ID,新增机构必填,且必须选择正常状态公民。
    #[serde(default)]
    pub legal_rep_cid_number: Option<String>,
    /// 证件照上传接口返回的服务端路径,新增机构必填。
    #[serde(default)]
    pub legal_rep_photo_path: Option<String>,
    #[serde(default)]
    pub legal_rep_photo_name: Option<String>,
    #[serde(default)]
    pub legal_rep_photo_mime: Option<String>,
    #[serde(default)]
    pub legal_rep_photo_size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateInstitutionOutput {
    pub cid_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid_full_name: Option<String>,
    pub category: InstitutionCategory,
}

/// 机构详情页提交的可编辑字段。私权类型由身份 ID 机构码决定,创建后不允许改。
#[derive(Debug, Deserialize)]
pub struct UpdateInstitutionInput {
    #[serde(default)]
    pub cid_full_name: Option<String>,
    #[serde(default)]
    pub cid_short_name: Option<String>,
    /// 所属法人 cid_number(仅 F 可设置;S/G 传值会被拒)
    #[serde(default)]
    pub parent_cid_number: Option<String>,
    /// 法定代表人三项资料在机构编辑保存时必填。
    #[serde(default)]
    pub legal_rep_name: Option<String>,
    #[serde(default)]
    pub legal_rep_cid_number: Option<String>,
    #[serde(default)]
    pub legal_rep_photo_path: Option<String>,
    #[serde(default)]
    pub legal_rep_photo_name: Option<String>,
    #[serde(default)]
    pub legal_rep_photo_mime: Option<String>,
    #[serde(default)]
    pub legal_rep_photo_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalRepresentativePhoto {
    pub file_path: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub account_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountOutput {
    pub cid_number: String,
    pub account_name: String,
    pub chain_status: MultisigChainStatus,
    pub chain_synced_at: Option<DateTime<Utc>>,
    pub chain_tx_hash: Option<String>,
    pub chain_block_number: Option<u64>,
    pub account: Option<String>,
}

/// /api/v1/institution/list 的列表过滤维度(查询参数,不是存储 category)。
///
/// 中文注释:JY 教育机构统一归教育 tab,私权目标类型归 private tab,公权目录仍承接公权本体
/// 和公权下属非法人:
/// - `Private`:私权 tab = 目标私权类型,可用 private_type 继续过滤;
/// - `Gov`:公权 tab = 非 JY 公权机构 + 父级为公法人的非 JY 非法人;
/// - `Education`:教育 tab = 确定性国家/市公民教育委员会 + 法人学校 + F+JY 分支机构。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstitutionListFilter {
    Private,
    Gov,
    Education,
}

impl InstitutionListFilter {
    /// 拼进列表 SQL 的静态过滤子句(三分支均为静态字面量,无注入面)。
    pub fn sql_clause(&self) -> &'static str {
        match self {
            Self::Private => {
                "AND s.category = 'PRIVATE_INSTITUTION' AND s.private_type IS NOT NULL"
            }
            Self::Gov => {
                "AND ((s.category = 'GOV_INSTITUTION'
                       AND s.institution_code NOT IN ('NED', 'CEDU', 'GUN', 'SUN', 'GSCH', 'SFSC'))
                      OR (s.institution_code IN ('SFGT', 'SFGP', 'UNIN')
                          AND s.institution_code NOT IN ('NED', 'CEDU', 'GUN', 'SUN', 'GSCH', 'SFSC')
                          AND par.category = 'GOV_INSTITUTION')))"
            }
            Self::Education => {
                "AND s.institution_code IN ('GUN', 'SUN', 'GSCH', 'SFSC')"
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InstitutionListRow {
    pub cid_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid_full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid_short_name: Option<String>,
    pub status: String,
    pub category: InstitutionCategory,
    pub p1: String,
    pub province_name: String,
    pub city_name: String,
    #[serde(default)]
    pub town_name: String,
    pub institution_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partnership_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_legal_personality: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_cid_number: Option<String>,
    pub account_count: usize,
    pub created_at: DateTime<Utc>,
    /// 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users)
    /// 命中:admin_name;未命中:None(前端显示为"未知")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_name: Option<String>,
    /// 创建者角色:"FEDERAL_REGISTRY" / "CITY_REGISTRY" / None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_role: Option<String>,
}

/// 法人机构搜索结果项(用于 F 详情页"所属法人"选择器)
#[derive(Debug, Serialize)]
pub struct ParentInstitutionRow {
    pub cid_number: String,
    pub cid_full_name: String,
    /// 私权机构类型。前端只用于展示父级机构事实,不派生链上业务角色。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partnership_kind: Option<String>,
    pub category: InstitutionCategory,
    /// 盈利属性。非法人创建时前端按"盈利属性附属于所属法人"用它推导 F 的 p1
    /// (公法人父级恒 0;私法人父级继承该值),后端 `unincorporated_org::inherited_p1` 复核。
    pub p1: String,
    pub province_name: String,
    pub city_name: String,
    #[serde(default)]
    pub town_name: String,
}

#[derive(Debug, Serialize)]
pub struct InstitutionDetailOutput {
    pub institution: Institution,
    pub accounts: Vec<InstitutionAccount>,
    /// 创建该机构的登录管理员姓名(按 created_by pubkey 反查 admin_users)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_name: Option<String>,
    /// 创建者角色:"FEDERAL_REGISTRY" / "CITY_REGISTRY"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChainSyncAccountInput {
    pub account_name: String,
    pub chain_status: MultisigChainStatus,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub chain_tx_hash: Option<String>,
    #[serde(default)]
    pub chain_block_number: Option<u64>,
}
