use primitives::cid::code::{is_valid_governance_code, InstitutionCode};
use serde::Serialize;

/// 管理员钱包在机构岗位上的一条有效任职。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionRoleAssignmentInfo {
    pub role_code: String,
    pub role_name: String,
    pub term_required: bool,
    pub term_start: u32,
    pub term_end: u32,
    pub assignment_source: u8,
    pub assignment_source_label: String,
    pub assignment_source_ref: String,
}

/// 一个机构管理员钱包及其在本机构的全部有效岗位任职。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionAdminInfo {
    /// 管理员钱包账户，hex 不含 0x。
    pub account: String,
    pub assignments: Vec<InstitutionRoleAssignmentInfo>,
}

/// 公权或私权机构 `AdminAccounts[cid_number]` 的桌面端联合展示状态。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAccountState {
    /// 机构唯一主键。
    pub cid_number: String,
    /// 链上机构码（CID institution_code，[u8;4]，治理分类唯一真源）。
    pub institution_code: InstitutionCode,
    pub institution_code_label: String,
    /// Node 按实际命中的公权/私权管理员 pallet 派生的类型编码。
    pub kind: u8,
    pub kind_label: String,
    /// 当前管理员钱包及其有效岗位任职；钱包在本集合内唯一。
    pub admins: Vec<InstitutionAdminInfo>,
}

/// 解码后的链上管理员账户原始值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAccountDecoded {
    pub institution_code: InstitutionCode,
    pub admins: Vec<String>,
}

/// 机构码 4 字符展示串（末尾 `\0` 填充字节去掉）。
pub fn institution_code_label(code: &InstitutionCode) -> String {
    let end = code.iter().position(|&b| b == 0).unwrap_or(code.len());
    String::from_utf8_lossy(&code[..end]).into_owned()
}

pub fn kind_label(kind: u8) -> &'static str {
    match kind {
        0 => "公权机构",
        1 => "私权机构",
        _ => "未知账户",
    }
}

pub fn is_valid_institution_code(code: &InstitutionCode) -> bool {
    is_valid_governance_code(code)
}
