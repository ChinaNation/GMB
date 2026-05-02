//! 转发 SFID `/api/v1/app/clearing-banks/eligible-search`,把"资格白名单内但可能未激活"
//! 候选列表给前端"添加清算行"页用。
//!
//! 与 `governance/sfid_api.rs` 共用 crate 根层 `sfid_config` 配置。
//! 本地 debug 默认查 127.0.0.1,正式 release 默认查 147 服务器。
//!
//! ## 反序列化契约(2026-05-01 修复 P0)
//!
//! SFID 端响应形态(参见 sfid/backend/src/chain/institution_info/dto.rs):
//! ```json
//! {
//!   "code": 0,
//!   "message": "ok",
//!   "data": [
//!     { "sfid_id": "...", "a3": "...", "main_chain_status": "NOT_ON_CHAIN", ... }
//!   ]
//! }
//! ```
//!
//! 关键点(历史踩坑):
//! - 顶层 `data` 是数组,**不是** `{ "items": [...] }` 信封
//! - 字段是 snake_case,**不是** camelCase(SFID 后端不挂 `rename_all`)
//! - `institution_name` 在两步式未命名时可能整个字段缺失(SFID 端 `skip_serializing_if = is_none`)
//! - `main_chain_status` 是 SCREAMING_SNAKE_CASE 枚举(`NOT_ON_CHAIN` / `PENDING_ON_CHAIN` /
//!   `ACTIVE_ON_CHAIN` / `REVOKED_ON_CHAIN`),不是友好字符串
//!
//! 因此本文件采用"双 DTO"模式:
//! - `SfidEligibleRow`:用于反序列化 SFID 响应(snake_case + Option),内部用
//! - [`crate::offchain::common::types::EligibleClearingBankCandidate`]:用于序列化给 Tauri 前端
//!   (camelCase + 友好状态字符串),公开导出
//!
//! 任何字段或顺序调整都必须两端同步,否则节点桌面"添加清算行"会报
//! "SFID 响应解析失败:error decoding response body"。

use serde::Deserialize;
use std::time::Duration;

use crate::shared::sfid_config;

use crate::offchain::common::types::{EligibleClearingBankCandidate, InstitutionCredentialResp};

const SFID_REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

/// SFID `ApiResponse<Vec<EligibleClearingBankRow>>` 的反序列化形态。
///
/// SFID 端用统一 `ApiResponse { code, message, data }`,`data` 直接是 Vec(无 items 信封)。
#[derive(Deserialize)]
struct EligibleSearchEnvelope {
    code: Option<u32>,
    #[serde(default)]
    data: Vec<SfidEligibleRow>,
    #[serde(default)]
    message: Option<String>,
}

/// SFID 端原始字段(snake_case)。仅本文件内部用,不对外暴露。
#[derive(Deserialize)]
struct SfidEligibleRow {
    sfid_id: String,
    #[serde(default)]
    institution_name: Option<String>,
    a3: String,
    #[serde(default)]
    sub_type: Option<String>,
    #[serde(default)]
    parent_sfid_id: Option<String>,
    #[serde(default)]
    parent_institution_name: Option<String>,
    #[serde(default)]
    parent_a3: Option<String>,
    province: String,
    city: String,
    #[serde(default)]
    main_account: Option<String>,
    #[serde(default)]
    fee_account: Option<String>,
    main_chain_status: SfidMultisigChainStatus,
}

/// 与 SFID 端 `MultisigChainStatus` 一一对应。
///
/// SFID 端 enum 用 `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`,
/// 因此线上字符串是 `"NOT_ON_CHAIN"` / `"PENDING_ON_CHAIN"` /
/// `"ACTIVE_ON_CHAIN"` / `"REVOKED_ON_CHAIN"`。
#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SfidMultisigChainStatus {
    NotOnChain,
    PendingOnChain,
    ActiveOnChain,
    RevokedOnChain,
}

/// 把 SFID 端枚举映射成节点 UI 友好的字符串。
///
/// 节点桌面 TS 端([citizenchain/node/frontend/offchain/types.ts])期望:
/// `'Inactive' | 'Pending' | 'Registered' | 'Failed'`。
fn map_chain_status(status: SfidMultisigChainStatus) -> &'static str {
    match status {
        SfidMultisigChainStatus::NotOnChain => "Inactive",
        SfidMultisigChainStatus::PendingOnChain => "Pending",
        SfidMultisigChainStatus::ActiveOnChain => "Registered",
        SfidMultisigChainStatus::RevokedOnChain => "Failed",
    }
}

fn into_candidate(row: SfidEligibleRow) -> EligibleClearingBankCandidate {
    EligibleClearingBankCandidate {
        sfid_id: row.sfid_id,
        institution_name: row.institution_name.unwrap_or_default(),
        a3: row.a3,
        sub_type: row.sub_type,
        parent_sfid_id: row.parent_sfid_id,
        parent_institution_name: row.parent_institution_name,
        parent_a3: row.parent_a3,
        province: row.province,
        city: row.city,
        main_chain_status: map_chain_status(row.main_chain_status).to_string(),
        main_account: row.main_account,
        fee_account: row.fee_account,
    }
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

    Ok(body.data.into_iter().map(into_candidate).collect())
}

// ─── 拉机构注册凭证(链上 propose_create_institution 必备入参) ───────

/// `chain/institution_info::app_get_institution` 响应反序列化封装。
#[derive(Deserialize)]
struct InstitutionCredentialEnvelope {
    code: Option<u32>,
    #[serde(default)]
    data: Option<InstitutionCredentialResp>,
    #[serde(default)]
    message: Option<String>,
}

/// 调 SFID `GET /api/v1/app/institutions/:sfid_id` 拉机构详情 + chain pull 凭证。
///
/// 响应携带 `register_nonce + signature`(由本机构所属省的省级签名密钥签发),
/// 节点桌面发起 `propose_create_institution` extrinsic 时直接透传。
///
/// 反序列化契约(snake_case)在 [`crate::offchain::common::types::InstitutionCredentialResp`] 锁定。
pub fn fetch_institution_credential(sfid_id: &str) -> Result<InstitutionCredentialResp, String> {
    // sfid_id 字符集仅 ASCII 字母 + 数字 + `-`(SFID 生成器锁定),无需 URL 编码。
    let url = format!(
        "{}/api/v1/app/institutions/{}",
        sfid_config::sfid_base_url(),
        sfid_id
    );
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(SFID_REQUEST_TIMEOUT)
        .timeout(SFID_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("创建 SFID HTTP 客户端失败:{e}"))?;
    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("SFID 机构详情请求失败:{e}"))?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("SFID 返回 HTTP {}", response.status()));
    }
    let body: InstitutionCredentialEnvelope = response
        .json()
        .map_err(|e| format!("SFID 响应解析失败:{e}"))?;
    if body.code != Some(0) {
        let msg = body.message.unwrap_or_default();
        return Err(format!("SFID 返回错误:code={:?}, message={msg}", body.code));
    }
    let data = body
        .data
        .ok_or_else(|| "SFID 响应缺少 data 字段".to_string())?;
    if data.register_nonce.is_empty() || data.signature.is_empty() {
        return Err(
            "SFID 未返回机构注册凭证(机构名为空 / 省级签名密钥未就绪),请联系运维".to_string(),
        );
    }
    Ok(data)
}
