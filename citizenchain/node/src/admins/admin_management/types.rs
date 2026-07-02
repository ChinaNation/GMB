use primitives::cid::code::{
    is_fixed_governance_code, is_registered_multisig_code, is_valid_governance_code,
    InstitutionCode,
};
use serde::Serialize;

/// 管理员资料来源。判别值与链端 `admin-primitives::AdminSource` 对齐。
pub fn source_label(source: u8) -> &'static str {
    match source {
        0 => "创世",
        1 => "注册局",
        2 => "内部投票",
        3 => "互选",
        4 => "普选",
        _ => "",
    }
}

/// 链上机构管理员公开资料，逐字段镜像 `admin-primitives::AdminProfile`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminProfileInfo {
    /// 管理员密码学账户，hex 不含 0x。
    pub account: String,
    /// 管理员实名锚:注册局签发的 CID 号。
    pub admin_cid_number: String,
    /// 姓名快照。
    pub name: String,
    /// 对外法定职务。
    pub admin_role: String,
    /// 任期开始(天数自纪元;无任期为 0)。
    pub term_start: u32,
    /// 任期结束(天数自纪元;无任期为 0)。
    pub term_end: u32,
    /// 职务/任期来源判别值。
    pub source: u8,
    /// 来源中文标签；未知来源留空，前端固定显示字段标签。
    pub source_label: String,
}

impl AdminProfileInfo {
    pub fn account_only(account: String) -> Self {
        Self {
            account,
            admin_cid_number: String::new(),
            name: String::new(),
            admin_role: String::new(),
            term_start: 0,
            term_end: 0,
            source: u8::MAX,
            source_label: String::new(),
        }
    }
}

/// 新 runtime 四类管理员 pallet 的 `AdminAccounts` 桌面端展示状态。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAccountState {
    /// 32 字节机构多签 AccountId，hex 不含 0x。
    pub account_hex: String,
    /// 内置机构入口使用的 cidNumber；动态账户可为空。
    pub cid_number: Option<String>,
    /// 链上机构码（CID institution_code，[u8;4]，治理分类唯一真源）。
    pub institution_code: InstitutionCode,
    pub institution_code_label: String,
    /// 链上 AdminAccountKind 枚举值。
    pub kind: u8,
    pub kind_label: String,
    /// 当前管理员资料。机构管理员为链上 AdminProfile；个人多签仅填 account。
    pub admins: Vec<AdminProfileInfo>,
    pub creator_hex: String,
    pub created_at: u32,
    pub updated_at: u32,
    /// 链上 AdminAccountStatus 枚举值。
    pub status: u8,
    pub status_label: String,
}

/// 解码后的链上管理员账户原始值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAccountDecoded {
    pub institution_code: InstitutionCode,
    pub kind: u8,
    pub admins: Vec<AdminProfileInfo>,
    pub creator_hex: String,
    pub created_at: u32,
    pub updated_at: u32,
    pub status: u8,
}

/// 机构码 4 字符展示串（末尾 `\0` 填充字节去掉）。
pub fn institution_code_label(code: &InstitutionCode) -> String {
    let end = code.iter().position(|&b| b == 0).unwrap_or(code.len());
    String::from_utf8_lossy(&code[..end]).into_owned()
}

pub fn kind_label(kind: u8) -> &'static str {
    match kind {
        0 => "创世管理员",
        1 => "公权机构",
        2 => "私权机构",
        3 => "个人多签",
        _ => "未知账户",
    }
}

pub fn is_valid_institution_code(code: &InstitutionCode) -> bool {
    is_valid_governance_code(code)
}

pub fn is_governance_code(code: &InstitutionCode) -> bool {
    is_fixed_governance_code(code)
}

pub fn is_dynamic_code(code: &InstitutionCode) -> bool {
    is_registered_multisig_code(code)
}

pub fn status_label(status: u8) -> &'static str {
    match status {
        0 => "待激活",
        1 => "已激活",
        2 => "已关闭",
        _ => "未知状态",
    }
}
