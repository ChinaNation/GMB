// 转发 SFID `/api/v1/app/clearing-banks/eligible-search`,把"资格白名单内但可能未激活"
// 候选列表给前端"添加清算行"页用。
//
// 与 governance/sfid_api.rs 共用 crate 根层 `sfid_config` 配置。
// 本地 debug 默认查 127.0.0.1，正式 release 默认查 147 服务器。

use serde::Deserialize;
use std::time::Duration;

use crate::shared::sfid_config;

use super::types::EligibleClearingBankCandidate;

const SFID_REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Deserialize)]
struct EligibleSearchEnvelope {
    code: Option<i32>,
    #[serde(default)]
    data: Option<EligibleSearchData>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct EligibleSearchData {
    #[serde(default)]
    items: Vec<EligibleClearingBankCandidate>,
}

/// `q` 关键字模糊匹配 sfid_id 或机构名。`limit` 上限 50,默认 20。
pub fn search_eligible_clearing_banks(
    q: &str,
    limit: u32,
) -> Result<Vec<EligibleClearingBankCandidate>, String> {
    let limit = limit.clamp(1, 50);
    let q_trim = q.trim();
    let url = format!(
        "{}/api/v1/app/clearing-banks/eligible-search",
        sfid_config::sfid_base_url()
    );

    let client = reqwest::blocking::Client::builder()
        .connect_timeout(SFID_REQUEST_TIMEOUT)
        .timeout(SFID_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("创建 SFID HTTP 客户端失败:{e}"))?;

    let response = client
        .get(&url)
        // reqwest::query 自动按 application/x-www-form-urlencoded 转义 q 中的特殊字符,
        // 避免手动拼接时遇到中文/% 等导致 SFID 端解析失败。
        .query(&[("q", q_trim), ("limit", &limit.to_string())])
        .send()
        .map_err(|e| format!("SFID eligible-search 请求失败:{e}"))?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("SFID 返回 HTTP {}", response.status()));
    }

    let body: EligibleSearchEnvelope = response
        .json()
        .map_err(|e| format!("SFID 响应解析失败:{e}"))?;

    if body.code != Some(0) {
        let msg = body.message.unwrap_or_default();
        return Err(format!("SFID 返回错误:code={:?}, message={msg}", body.code));
    }

    Ok(body.data.map(|d| d.items).unwrap_or_default())
}
