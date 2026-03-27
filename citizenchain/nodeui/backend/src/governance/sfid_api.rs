// SFID 人口快照 API 客户端：获取公民投票 eligible_total / snapshot_nonce / signature。

use serde::Deserialize;
use std::time::Duration;

/// SFID 服务地址（与 wuminapp 生产环境一致）。
const SFID_BASE_URL: &str = "http://147.224.14.117:8899";
const SFID_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// 人口快照数据，用于联合投票提案创建时附带人口证明。
#[derive(Debug, Clone)]
pub struct PopulationSnapshot {
    pub eligible_total: u64,
    /// 快照 nonce（UTF-8 字符串，直接作为 BoundedVec<u8> 提交）。
    pub snapshot_nonce: String,
    /// sr25519 签名（hex 编码，提交时需解码为原始字节）。
    pub signature: String,
}

#[derive(Deserialize)]
struct SfidResponse {
    code: Option<i32>,
    data: Option<SfidSnapshotData>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct SfidSnapshotData {
    eligible_total: Option<u64>,
    snapshot_nonce: Option<String>,
    signature: Option<String>,
}

/// 从 SFID 获取公民人口快照（eligible_total + nonce + signature）。
pub fn fetch_population_snapshot(pubkey_hex: &str) -> Result<PopulationSnapshot, String> {
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();

    let url = format!(
        "{}/api/v1/app/voters/count?account_pubkey={}",
        SFID_BASE_URL, pubkey_clean
    );

    let client = reqwest::blocking::Client::builder()
        .connect_timeout(SFID_REQUEST_TIMEOUT)
        .timeout(SFID_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("创建 SFID HTTP 客户端失败: {e}"))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("SFID 人口快照请求失败: {e}"))?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("SFID 返回 HTTP {}", response.status()));
    }

    let body: SfidResponse = response
        .json()
        .map_err(|e| format!("SFID 响应解析失败: {e}"))?;

    if body.code != Some(0) {
        let msg = body.message.unwrap_or_default();
        return Err(format!("SFID 返回错误: code={:?}, message={msg}", body.code));
    }

    let data = body.data.ok_or("SFID 响应缺少 data 字段")?;

    let eligible_total = data.eligible_total.unwrap_or(0);
    let snapshot_nonce = data
        .snapshot_nonce
        .filter(|s| !s.is_empty())
        .ok_or("SFID 响应缺少 snapshot_nonce")?;
    let signature = data
        .signature
        .filter(|s| !s.is_empty())
        .ok_or("SFID 响应缺少 signature")?;

    Ok(PopulationSnapshot {
        eligible_total,
        snapshot_nonce,
        signature,
    })
}
