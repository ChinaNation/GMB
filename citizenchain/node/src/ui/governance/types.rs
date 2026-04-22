// 治理模块数据类型，对应链上 AdminsOriginGov 等 pallet 的存储结构。

use serde::Serialize;

/// 机构类型枚举，数值与链上 `org` 编码一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OrgType {
    /// 国储会 National Reserve Committee
    Nrc = 0,
    /// 省储会 Provincial Reserve Committee
    Prc = 1,
    /// 省储行 Provincial Reserve Bank
    Prb = 2,
}

impl OrgType {
    pub fn label(&self) -> &'static str {
        match self {
            OrgType::Nrc => "国储会",
            OrgType::Prc => "省储会",
            OrgType::Prb => "省储行",
        }
    }
}

/// 机构详情，返回给前端的聚合结果。
/// 管理员信息（公钥 + 链上余额）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminInfo {
    /// 管理员公钥（hex，不含 0x 前缀）。
    pub pubkey_hex: String,
    /// 链上余额（分），节点未运行时为 null。
    pub balance_fen: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionDetail {
    /// 机构名称。
    pub name: String,
    /// 链上身份标识。
    pub shenfen_id: String,
    /// 机构类型：0=NRC, 1=PRC, 2=PRB。
    pub org_type: u8,
    /// 机构类型显示标签。
    pub org_type_label: String,
    /// 主账户地址 hex，由前端再转成 SS58 显示。
    pub main_address: String,
    /// 主账户链上余额（分），节点未运行时为 null。
    pub balance_fen: Option<String>,
    /// 管理员列表（含公钥和链上余额），节点未运行时为空。
    pub admins: Vec<AdminInfo>,
    /// 内部投票通过阈值。
    pub internal_threshold: u32,
    /// 联合投票权重。
    pub joint_vote_weight: u32,
    /// 永久质押账户地址 hex（仅 PRB）。
    pub staking_address: Option<String>,
    /// 永久质押账户余额（分，仅 PRB）。
    pub staking_balance_fen: Option<String>,
    /// 手续费账户地址 hex（仅 PRB）。
    pub fee_address: Option<String>,
    /// 手续费账户余额（分，仅 PRB）。
    pub fee_balance_fen: Option<String>,
    /// 储委会费用账户地址 hex（省储会，仅 PRC）。
    pub cb_fee_address: Option<String>,
    /// 储委会费用账户余额（分，仅 PRC）。
    pub cb_fee_balance_fen: Option<String>,
    /// 国储会费用账户地址 hex（仅 NRC）。
    pub nrc_fee_address: Option<String>,
    /// 国储会手续费账户余额（分，仅 NRC）。
    pub nrc_fee_balance_fen: Option<String>,
    /// 国储会安全基金账户地址 hex（仅 NRC）。
    pub nrc_anquan_address: Option<String>,
    /// 国储会安全基金账户余额（分，仅 NRC）。
    pub nrc_anquan_balance_fen: Option<String>,
    /// 告警信息。
    pub warning: Option<String>,
}

/// 治理详情页余额更新事件，仅覆盖链上金额和告警，不改页面结构。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionBalanceUpdate {
    /// 链上身份标识，用于前端过滤当前页面事件。
    pub shenfen_id: String,
    /// 主账户链上余额（分）。
    pub balance_fen: Option<String>,
    /// 永久质押账户链上余额（分，仅 PRB）。
    pub staking_balance_fen: Option<String>,
    /// 费用账户链上余额（分，仅 PRB）。
    pub fee_balance_fen: Option<String>,
    /// 省储会费用账户链上余额（分，仅 PRC）。
    pub cb_fee_balance_fen: Option<String>,
    /// 国储会费用账户链上余额（分，仅 NRC）。
    pub nrc_fee_balance_fen: Option<String>,
    /// 国储会安全基金账户链上余额（分，仅 NRC）。
    pub nrc_anquan_balance_fen: Option<String>,
    /// 链上查询告警。
    pub warning: Option<String>,
}

/// 机构列表项，用于治理首页机构列表。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionListItem {
    pub name: String,
    pub shenfen_id: String,
    pub org_type: u8,
    pub org_type_label: String,
    /// 主账户地址 hex，由前端转成 SS58 显示。
    pub main_address: String,
}

/// 治理首页聚合数据。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceOverview {
    /// 国储会列表。
    pub national_councils: Vec<InstitutionListItem>,
    /// 省储会列表。
    pub provincial_councils: Vec<InstitutionListItem>,
    /// 省储行列表。
    pub provincial_banks: Vec<InstitutionListItem>,
    pub warning: Option<String>,
}
