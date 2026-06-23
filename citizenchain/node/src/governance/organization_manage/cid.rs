//! 转发 CID `/api/v1/app/clearing-banks/eligible-search`,把"资格白名单内但可能未激活"
//! 候选列表给前端"添加清算行"页用。
//!
//! 机构注册凭证客户端，只服务 organization-manage 业务边界。
//! 本地 debug 默认查 127.0.0.1,正式 release 默认查 147 服务器。
//!
//! ## 反序列化契约
//!
//! CID 端响应形态(参见 citizencode/backend/institutions/chain_duoqian_info.rs):
//! ```json
//! {
//!   "code": 0,
//!   "message": "ok",
//!   "data": [
//!     { "cid_number": "...", "ref_property": "...", "main_chain_status": "NOT_ON_CHAIN", ... }
//!   ]
//! }
//! ```
//!
//! 关键点:
//! - 顶层 `data` 是数组,**不是** `{ "items": [...] }` 信封
//! - 字段是 snake_case,**不是** camelCase(CID 后端不挂 `rename_all`)
//! - `cid_full_name` 在两步式未命名时可能整个字段缺失(CID 端 `skip_serializing_if = is_none`)
//! - `main_chain_status` 是 SCREAMING_SNAKE_CASE 枚举(`NOT_ON_CHAIN` / `PENDING_ON_CHAIN` /
//!   `ACTIVE_ON_CHAIN` / `REVOKED_ON_CHAIN`),不是友好字符串
//!
//! 因此本文件采用"双 DTO"模式:
//! - `CidEligibleRow`:用于反序列化 CID 响应(snake_case + Option),内部用
//! - [`super::types::EligibleClearingBankCandidate`]:用于序列化给 Tauri 前端
//!   (camelCase + 友好状态字符串),公开导出
//!
//! 任何字段或顺序调整都必须两端同步,否则节点桌面"添加清算行"会报
//! "CID 响应解析失败:error decoding response body"。

use serde::Deserialize;
use std::time::Duration;

use crate::shared::cid_config;

use super::types::{EligibleClearingBankCandidate, InstitutionRegistrationInfoResp};

const CID_REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

/// CID `ApiResponse<Vec<EligibleClearingBankRow>>` 的反序列化形态。
///
/// CID 端用统一 `ApiResponse { code, message, data }`,`data` 直接是 Vec(无 items 信封)。
#[derive(Deserialize)]
struct EligibleSearchEnvelope {
    code: Option<u32>,
    #[serde(default)]
    data: Vec<CidEligibleRow>,
    #[serde(default)]
    message: Option<String>,
}

/// CID 端原始字段(snake_case)。仅本文件内部用,不对外暴露。
#[derive(Deserialize)]
struct CidEligibleRow {
    cid_number: String,
    #[serde(default)]
    cid_full_name: Option<String>,
    ref_property: String,
    #[serde(default)]
    sub_type: Option<String>,
    #[serde(default)]
    parent_cid_number: Option<String>,
    #[serde(default)]
    parent_cid_full_name: Option<String>,
    #[serde(default)]
    parent_ref_property: Option<String>,
    province_name: String,
    city_name: String,
    #[serde(default)]
    main_account: Option<String>,
    #[serde(default)]
    fee_account: Option<String>,
    main_chain_status: CidMultisigChainStatus,
}

/// 与 CID 端 `MultisigChainStatus` 一一对应。
///
/// CID 端 enum 用 `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`,
/// 因此线上字符串是 `"NOT_ON_CHAIN"` / `"PENDING_ON_CHAIN"` /
/// `"ACTIVE_ON_CHAIN"` / `"REVOKED_ON_CHAIN"`。
#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum CidMultisigChainStatus {
    NotOnChain,
    PendingOnChain,
    ActiveOnChain,
    RevokedOnChain,
}

/// 把 CID 端枚举映射成节点 UI 友好的字符串。
///
/// 节点桌面 TS 端([citizenchain/node/frontend/offchain/types.ts])期望:
/// `'Pending' | 'Active' | 'Closed' | 'Failed'`。
fn map_chain_status(status: CidMultisigChainStatus) -> &'static str {
    match status {
        CidMultisigChainStatus::NotOnChain => "Pending",
        CidMultisigChainStatus::PendingOnChain => "Pending",
        CidMultisigChainStatus::ActiveOnChain => "Active",
        CidMultisigChainStatus::RevokedOnChain => "Closed",
    }
}

fn into_candidate(row: CidEligibleRow) -> EligibleClearingBankCandidate {
    EligibleClearingBankCandidate {
        cid_number: row.cid_number,
        cid_full_name: row.cid_full_name.unwrap_or_default(),
        ref_property: row.ref_property,
        sub_type: row.sub_type,
        parent_cid_number: row.parent_cid_number,
        parent_cid_full_name: row.parent_cid_full_name,
        parent_ref_property: row.parent_ref_property,
        province_name: row.province_name,
        city_name: row.city_name,
        main_chain_status: map_chain_status(row.main_chain_status).to_string(),
        main_account: row.main_account,
        fee_account: row.fee_account,
    }
}

/// `q` 关键字模糊匹配 cid_number 或机构名。`limit` 上限 50,默认 20。
pub fn search_eligible_clearing_banks(
    q: &str,
    limit: u32,
) -> Result<Vec<EligibleClearingBankCandidate>, String> {
    let limit = limit.clamp(1, 50);
    let q_trim = q.trim();
    let url = format!(
        "{}/api/v1/app/clearing-banks/eligible-search",
        cid_config::cid_base_url()
    );

    let client = reqwest::blocking::Client::builder()
        .connect_timeout(CID_REQUEST_TIMEOUT)
        .timeout(CID_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("创建 CID HTTP 客户端失败:{e}"))?;

    let response = client
        .get(&url)
        // reqwest::query 自动按 application/x-www-form-urlencoded 转义 q 中的特殊字符,
        // 避免手动拼接时遇到中文/% 等导致 CID 端解析失败。
        .query(&[("q", q_trim), ("limit", &limit.to_string())])
        .send()
        .map_err(|e| format!("CID eligible-search 请求失败:{e}"))?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("CID 返回 HTTP {}", response.status()));
    }

    let body: EligibleSearchEnvelope = response
        .json()
        .map_err(|e| format!("CID 响应解析失败:{e}"))?;

    if body.code != Some(0) {
        let msg = body.message.unwrap_or_default();
        return Err(format!("CID 返回错误:code={:?}, message={msg}", body.code));
    }

    Ok(body.data.into_iter().map(into_candidate).collect())
}

// ─── 拉机构注册信息(链上 propose_create_institution 必备入参) ───────

/// CID `registration-info` 响应反序列化封装。
#[derive(Deserialize)]
struct InstitutionRegistrationInfoEnvelope {
    code: Option<u32>,
    #[serde(default)]
    data: Option<InstitutionRegistrationInfoResp>,
    #[serde(default)]
    message: Option<String>,
}

/// 调 CID `GET /api/v1/app/institutions/:cid_number/registration-info` 拉链上注册专用信息。
///
/// 中文注释:这里刻意不调用普通机构详情接口。普通详情可用于展示,但不能证明
/// "机构名称 + 账户名称列表"确实由 CID 系统签发给链上注册流程。
pub fn fetch_institution_registration_info(
    cid_number: &str,
) -> Result<InstitutionRegistrationInfoResp, String> {
    // cid_number 字符集仅 ASCII 字母 + 数字 + `-`(CID 生成器锁定),无需 URL 编码。
    let url = format!(
        "{}/api/v1/app/institutions/{}/registration-info",
        cid_config::cid_base_url(),
        cid_number
    );
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(CID_REQUEST_TIMEOUT)
        .timeout(CID_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("创建 CID HTTP 客户端失败:{e}"))?;
    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("CID 机构注册信息请求失败:{e}"))?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("CID 返回 HTTP {}", response.status()));
    }
    let body: InstitutionRegistrationInfoEnvelope = response
        .json()
        .map_err(|e| format!("CID 响应解析失败:{e}"))?;
    if body.code != Some(0) {
        let msg = body.message.unwrap_or_default();
        return Err(format!("CID 返回错误:code={:?}, message={msg}", body.code));
    }
    let data = body
        .data
        .ok_or_else(|| "CID 响应缺少 data 字段".to_string())?;
    if data.cid_full_name.trim().is_empty() {
        return Err("CID 未返回机构名称,请先在 CID 系统完善机构信息".to_string());
    }
    if data.account_names.is_empty() || data.account_names.iter().any(|name| name.trim().is_empty())
    {
        return Err("CID 未返回有效账户名称列表,请先在 CID 系统完善机构账户信息".to_string());
    }
    if data.credential.register_nonce.is_empty()
        || data.credential.signature.is_empty()
        || data.credential.issuer_cid_number.is_empty()
        || data.credential.issuer_main_account.is_empty()
        || data.credential.signer_pubkey.is_empty()
        || data.credential.scope_province_name.is_empty()
    {
        return Err("CID 未返回完整机构注册凭证,请确认签发机构管理员已激活".to_string());
    }
    Ok(data)
}
