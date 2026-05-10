use serde::Serialize;

/// `AdminsChange::Subjects` 的桌面端展示状态。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSubjectState {
    /// 48 字节 SubjectId，hex 不含 0x。
    pub subject_id_hex: String,
    /// 内置机构入口使用的 sfidNumber；账户级主体可为空。
    pub sfid_number: Option<String>,
    /// 链上 org 编码：0=NRC,1=PRC,2=PRB,3=REN/多钱账户。
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
        3 => "多钱账户",
        _ => "未知组织",
    }
}

pub fn kind_label(kind: u8) -> &'static str {
    match kind {
        0 => "内置治理机构",
        1 => "SFID机构",
        2 => "个人多签",
        3 => "机构账户",
        _ => "未知主体",
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
