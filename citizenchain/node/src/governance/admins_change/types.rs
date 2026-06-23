use primitives::institution_code::{
    is_fixed_governance_code, is_registered_multisig_code, is_valid_governance_code,
    InstitutionCode,
};
use serde::Serialize;

/// `AdminsChange::AdminAccounts` 的桌面端展示状态。
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
    /// 当前管理员公钥，hex 不含 0x。
    pub admins: Vec<String>,
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
    pub admins: Vec<String>,
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
        0 => "内置治理机构",
        1 => "个人多签",
        2 => "机构账户",
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
