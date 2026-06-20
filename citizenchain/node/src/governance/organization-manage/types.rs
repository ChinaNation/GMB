// 机构多签管理 Tauri DTO,与前端 governance/organization-manage/types.ts 对齐。
//
// 中文注释:
// - 本文件只承载 OrganizationManage 机构多签管理相关的输入输出类型。
// - 清算行节点声明、连通性检测、管理员解锁等 offchain 网络能力继续留在
//   `offchain/common/types.rs`,避免机构多签边界再次散落。

use serde::{Deserialize, Serialize};

/// 节点桌面"添加清算行"页用的候选机构记录(序列化给 Tauri 前端)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EligibleClearingBankCandidate {
    pub sfid_number: String,
    /// 机构中文名;两步式未命名时为空串。
    pub sfid_full_name: String,
    pub ref_property: String,
    pub sub_type: Option<String>,
    pub parent_sfid_number: Option<String>,
    pub parent_sfid_full_name: Option<String>,
    pub parent_ref_property: Option<String>,
    pub province_name: String,
    pub city_name: String,
    /// 主账户当前链上状态:`Pending` / `Active` / `Closed` / `Failed`。
    pub main_chain_status: String,
    pub main_account: Option<String>,
    pub fee_account: Option<String>,
}

/// 单账户的链上展示形态(地址 SS58 + 余额"分"+ is_default 标识)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWithBalance {
    pub account_name: String,
    /// 32 字节链上地址的 SS58 形式(GMB prefix=2027)。
    pub address_ss58: String,
    /// `frame_system::Account[address].data.free`,最小单位"分"。
    pub balance_min_units: String,
    /// 友好元字符串 `xxx.xx`。
    pub balance_text: String,
    pub is_default: bool,
}

/// 机构详情 = `organization-manage::Institutions[sfid_number]` + 各账户余额 + 友好标签。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionDetail {
    pub sfid_number: String,
    pub sfid_full_name: String,
    /// 管理员更换使用的机构多签 AccountId。当前清算行以主账户作为机构管理员账户。
    pub admin_account_hex: String,
    /// 管理员更换使用的 org。清算行属于 ORG_OTH 机构账户。
    pub admin_org: u8,
    pub main_account: AccountWithBalance,
    pub fee_account: AccountWithBalance,
    /// 主账户/费用账户之外的全部账户(自定义初始账户)。
    pub other_accounts: Vec<AccountWithBalance>,
    pub admin_count: u32,
    pub threshold: u32,
    /// 管理员公钥 32B 的 SS58 列表。
    pub duoqian_admins_ss58: Vec<String>,
    /// 机构生命周期:Pending(投票中)/ Active(已生效)/ Closed(已注销)。
    pub status: String,
    pub creator_ss58: String,
    pub created_at: u64,
    pub account_count: u32,
}

/// 机构提案列表分页结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionProposalPage {
    pub items: Vec<InstitutionProposalItem>,
    pub has_more: bool,
}

/// 提案列表条目。提案完整字段由 governance 模块掌握,这里只透传必需展示项。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionProposalItem {
    pub proposal_id: u64,
    pub kind_label: String,
    pub status_label: String,
    pub summary: String,
}

/// SFID `/api/v1/app/institutions/:sfid_number/registration-info` 的响应形态。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InstitutionRegistrationInfoResp {
    pub sfid_number: String,
    pub sfid_full_name: String,
    pub account_names: Vec<String>,
    pub credential: InstitutionRegistrationCredentialResp,
}

/// SFID 对机构注册 payload 签发的凭证。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InstitutionRegistrationCredentialResp {
    /// 链 genesis hash,节点验签时对应 runtime 的 block_hash(0)。
    pub genesis_hash: String,
    /// 防重放 nonce(本次响应生成的随机字符串)。
    pub register_nonce: String,
    pub province_name: String,
    /// 本次签名所用省管理员公钥(32 字节 hex),链上按 (province_name, signer_admin_pubkey) 查派生签名公钥。
    pub signer_admin_pubkey: String,
    /// 省级签名密钥对凭证 payload 的 sr25519 签名(64 字节 hex)。
    pub signature: String,
    /// SFID 端附带的审计元信息,节点只透传展示/排查,不参与链上注册编码。
    pub meta: serde_json::Value,
}
