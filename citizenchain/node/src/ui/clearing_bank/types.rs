// 清算行 tab Tauri command 用 DTO 集合。
//
// 类型设计与对应的链上 storage / SFID 接口对齐:
// - `EligibleClearingBankCandidate` — SFID `/clearing-banks/eligible-search` 响应
// - `ClearingBankNodeOnChainInfo`     — 链上 `ClearingBankNodes[sfid_id]` 反序列化
// - `ConnectivityTestReport`          — node Tauri 4 重连通性自测结果
// - `DecryptedAdminInfo`              — 已解密私钥的清算行管理员条目(内存内)

use serde::{Deserialize, Serialize};

/// SFID `/clearing-banks/eligible-search` 单条候选(资格白名单内,可能未激活)。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EligibleClearingBankCandidate {
    pub sfid_id: String,
    pub institution_name: String,
    pub a3: String,
    #[serde(default)]
    pub sub_type: Option<String>,
    #[serde(default)]
    pub parent_sfid_id: Option<String>,
    #[serde(default)]
    pub parent_institution_name: Option<String>,
    #[serde(default)]
    pub parent_a3: Option<String>,
    pub province: String,
    pub city: String,
    /// 主账户当前链上状态:Inactive / Pending / Registered / Failed。
    pub main_chain_status: String,
    #[serde(default)]
    pub main_account: Option<String>,
    #[serde(default)]
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
