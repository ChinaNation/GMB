// 清算行 tab Tauri command 用 DTO 集合,与前端 offchain/types.ts 对齐。
//
// 类型设计与对应的链上 storage / SFID 接口对齐:
// - `EligibleClearingBankCandidate` — SFID `/clearing-banks/eligible-search` 响应
// - `ClearingBankNodeOnChainInfo`     — 链上 `ClearingBankNodes[sfid_id]` 反序列化
// - `ConnectivityTestReport`          — node Tauri 4 重连通性自测结果
// - `DecryptedAdminInfo`              — 已解密私钥的清算行管理员条目(内存内)

use serde::{Deserialize, Serialize};

/// 节点桌面"添加清算行"页用的候选机构记录(序列化给 Tauri 前端)。
///
/// 反序列化 SFID 响应的 DTO 在 [`super::sfid::SfidEligibleRow`](snake_case),
/// 本结构只做 Serialize → TS 端 camelCase。两段 DTO 解耦,避免之前
/// "误用同一 struct 同时跨 SFID 入口/Tauri 出口"导致的契约 mismatch P0。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EligibleClearingBankCandidate {
    pub sfid_id: String,
    /// 机构中文名;两步式未命名时为空串。
    pub institution_name: String,
    pub a3: String,
    pub sub_type: Option<String>,
    pub parent_sfid_id: Option<String>,
    pub parent_institution_name: Option<String>,
    pub parent_a3: Option<String>,
    pub province: String,
    pub city: String,
    /// 主账户当前链上状态:`Inactive` / `Pending` / `Registered` / `Failed`,
    /// 由 `super::sfid::map_chain_status` 从 SFID 端 SCREAMING_SNAKE_CASE 枚举映射。
    pub main_chain_status: String,
    pub main_account: Option<String>,
    pub fee_account: Option<String>,
}

/// 链上 `ClearingBankNodes[sfid_id]` 解码后的对前端形态。
///
/// 字段为字符串/u32 友好类型,前端无需做 Bytes/SS58 自行处理。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearingBankNodeOnChainInfo {
    pub sfid_id: String,
    /// libp2p PeerId 字符串("12D3KooW..." 形式)。
    pub peer_id: String,
    pub rpc_domain: String,
    pub rpc_port: u16,
    /// 链上注册区块高度。
    pub registered_at: u64,
    /// 注册管理员公钥(0x 前缀 hex,小写)。
    pub registered_by_pubkey_hex: String,
    /// 注册管理员 SS58(便于前端展示)。
    pub registered_by_ss58: String,
}

/// 连通性自测的逐项结果。每项要么 ok=true,要么带 detail 解释失败原因。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectivityCheck {
    pub label: &'static str,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// 4 重自测汇总报告。`all_ok` 任一项失败即 false,前端据此置灰提交按钮。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectivityTestReport {
    pub all_ok: bool,
    pub checks: Vec<ConnectivityCheck>,
}

/// 当前内存中已解密(可用于自动签 batch)的清算行管理员条目。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedAdminInfo {
    /// 管理员公钥(0x 前缀 hex,小写)。
    pub pubkey_hex: String,
    pub sfid_id: String,
    /// 解密时间(毫秒时间戳)。
    pub decrypted_at_ms: u64,
}

/// 解密管理员密钥的请求构建结果(WUMIN_QR_V1 challenge envelope)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptAdminRequestResult {
    pub request_json: String,
    pub request_id: String,
    pub expected_payload_hash: String,
    /// challenge payload hex(用于本地验证 sr25519 签名)。
    pub payload_hex: String,
}

// ─── 清算行机构详情(链上 Institutions[sfid_id] 的对前端形态) ────────

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

/// 机构详情 = `duoqian-manage::Institutions[sfid_id]` + 各账户余额 + 友好标签。
///
/// 当链上不存在该 sfid_id 的机构时,Tauri 命令返回 `Option::None`,
/// 节点桌面据此进入"创建多签机构"流程。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionDetail {
    pub sfid_id: String,
    pub institution_name: String,
    /// 机构类型对前端友好标签(由 a3 + sub_type 推):
    /// 私法人多签 / 私非法人多签 / 公权多签 / 公安局多签 等。
    pub institution_type_label: String,
    pub a3: String,
    pub sub_type: Option<String>,
    pub parent_sfid_id: Option<String>,

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
///
/// 当前阶段仅返回空列表占位(全量扫描 voting-engine `Proposals` 并按机构过滤
/// 留 follow-up 任务卡)。前端 UI 依然会显示"暂无提案"行,未来填充时无需改 UI。
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

// ─── SFID `app_get_institution` 响应(含 chain pull 凭证) ──────────

/// `chain/institution_info::app_get_institution` 的反序列化形态。
///
/// SFID 端响应是 `MultisigInstitution` 全部字段平铺 + 末尾 2 个签名字段。
/// 节点桌面发起 `propose_create_institution` extrinsic 时,
/// 直接把本结构里的 register_nonce / signature / province (做 signing_province)/
/// institution_name / a3 / sub_type / parent_sfid_id 透传给 extrinsic。
///
/// 同时实现 Serialize 以便 Tauri 命令把本结构透传给前端,前端再回传到
/// `build_propose_create_institution_request` Tauri 命令。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InstitutionCredentialResp {
    pub sfid_id: String,
    #[serde(default)]
    pub institution_name: Option<String>,
    pub a3: String,
    #[serde(default)]
    pub sub_type: Option<String>,
    #[serde(default)]
    pub parent_sfid_id: Option<String>,
    pub province: String,
    pub city: String,
    /// 防重放 nonce(本次响应生成的随机 hex)。
    pub register_nonce: String,
    /// 省级签名密钥对凭证 payload 的 sr25519 签名(64 字节 hex)。
    pub signature: String,
}
