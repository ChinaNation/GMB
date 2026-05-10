use serde::Serialize;

/// 构建 propose_runtime_upgrade 签名请求的返回值。
///
/// 当前第 1 步保持 runtime 既有参数兼容,所以仍携带 SFID 人口快照字段;
/// 第 2 步更新 runtime 模块时再统一削减链上 call 参数。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeUpgradeRequestResult {
    pub request_json: String,
    pub request_id: String,
    pub expected_payload_hash: String,
    pub sign_nonce: u32,
    pub sign_block_number: u64,
    pub eligible_total: u64,
    pub snapshot_nonce: String,
    pub snapshot_signature: String,
}
