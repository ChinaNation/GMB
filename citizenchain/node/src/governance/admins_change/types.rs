use serde::Serialize;

/// `AdminsChange::Subjects` 的桌面端展示状态。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSubjectState {
    /// 48 字节 SubjectId，hex 不含 0x。
    pub subject_id_hex: String,
    /// 内置机构入口使用的 sfidNumber；账户级主体可为空。
    pub sfid_number: Option<String>,
    /// 链上 org 编码：0=NRC,1=PRC,2=PRB,3=REN(个人多签),4=PUP(公权机构账户),5=OTH(其他机构账户)。
    pub org: u8,
    pub org_label: String,
    /// 链上 AdminSubjectKind 枚举值。
    pub kind: u8,
    pub kind_label: String,
    /// 当前管理员公钥，hex 不含 0x。
    pub admins: Vec<String>,
    pub threshold: u32,
    pub creator_hex: String,
    pub created_at: u32,
    pub updated_at: u32,
    /// 链上 AdminSubjectStatus 枚举值。
    pub status: u8,
    pub status_label: String,
}

/// 解码后的链上管理员主体原始值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminSubjectDecoded {
    pub org: u8,
    pub kind: u8,
    pub admins: Vec<String>,
    pub threshold: u32,
    pub creator_hex: String,
    pub created_at: u32,
    pub updated_at: u32,
    pub status: u8,
}

pub fn org_label(org: u8) -> &'static str {
    match org {
        0 => "国储会",
        1 => "省储会",
        2 => "省储行",
        3 => "个人多签",
        4 => "公权机构账户",
        5 => "其他机构账户",
        _ => "未知组织",
    }
}

pub fn kind_label(kind: u8) -> &'static str {
    match kind {
        0 => "内置治理机构",
        1 => "SFID机构归属",
        2 => "个人多签",
        3 => "机构账户",
        _ => "未知主体",
    }
}

pub fn is_valid_org(org: u8) -> bool {
    matches!(org, 0..=5)
}

pub fn is_governance_org(org: u8) -> bool {
    matches!(org, 0 | 1 | 2)
}

pub fn is_dynamic_admin_org(org: u8) -> bool {
    matches!(org, 3 | 4 | 5)
}

/// 返回必须与 wumin 冷钱包 PayloadDecoder 解码出的 org 字段一致的展示值。
pub fn qr_org_display_value(org: u8) -> String {
    match org {
        0 | 1 | 2 => org_label(org).to_string(),
        3 => "个人多签".to_string(),
        4 => "公权机构账户".to_string(),
        5 => "其他机构账户".to_string(),
        _ => format!("机构{org}"),
    }
}

pub fn status_label(status: u8) -> &'static str {
    match status {
        0 => "待激活",
        1 => "已激活",
        2 => "已关闭",
        _ => "未知状态",
    }
}
