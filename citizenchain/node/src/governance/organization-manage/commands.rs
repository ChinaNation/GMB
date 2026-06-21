//! 清算行注册机构多签管理 Tauri 命令。
//!
//! 中文注释:
//! - 本文件只面向"清算行注册机构"的多签创建和机构详情查询。
//! - 普通注册机构多签、个人多签仍由 citizenapp 操作,不进入节点软件目录。

use tauri::AppHandle;

use super::types::{
    EligibleClearingBankCandidate, InstitutionDetail, InstitutionProposalPage,
    InstitutionRegistrationInfoResp,
};
use crate::governance::signing as gov_signing;
use crate::home;

use super::signing::InitialAccountInput;

/// 搜索资格白名单内的清算行候选机构(包含未激活,供"添加清算行"页选择)。
#[tauri::command]
pub async fn search_eligible_clearing_banks(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<EligibleClearingBankCandidate>, String> {
    let limit = limit.unwrap_or(20);
    tauri::async_runtime::spawn_blocking(move || {
        super::cid::search_eligible_clearing_banks(&query, limit)
    })
    .await
    .map_err(|e| format!("search_eligible_clearing_banks task failed:{e}"))?
}

/// 链上查询某机构的多签信息。返回 `None` = 该 cid_number 链上尚未创建机构。
#[tauri::command]
pub async fn fetch_clearing_bank_institution_detail(
    app: AppHandle,
    cid_number: String,
) -> Result<Option<InstitutionDetail>, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法查询链上数据".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        super::chain::fetch_institution_detail(&cid_number)
    })
    .await
    .map_err(|e| format!("fetch_clearing_bank_institution_detail task failed:{e}"))?
}

/// 机构提案分页查询。本阶段返回空列表占位。
#[tauri::command]
pub async fn fetch_clearing_bank_institution_proposals(
    app: AppHandle,
    cid_number: String,
    start_id: u64,
    page_size: u32,
) -> Result<InstitutionProposalPage, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法查询链上数据".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        super::chain::fetch_institution_proposals(&cid_number, start_id, page_size)
    })
    .await
    .map_err(|e| format!("fetch_clearing_bank_institution_proposals task failed:{e}"))?
}

/// 调 CID 拉链上注册专用机构信息 + 签发凭证。
#[tauri::command]
pub async fn fetch_clearing_bank_institution_registration_info(
    cid_number: String,
) -> Result<InstitutionRegistrationInfoResp, String> {
    tauri::async_runtime::spawn_blocking(move || {
        super::cid::fetch_institution_registration_info(&cid_number)
    })
    .await
    .map_err(|e| format!("fetch_clearing_bank_institution_registration_info task failed:{e}"))?
}

/// 中文注释:从 TS 端传入的账户初始资金条目。
/// 单位"分"用字符串透传,避免 JS 数字精度溢出。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialAccountInputDto {
    pub account_name: String,
    pub amount_fen: String,
}

fn parse_initial_accounts(
    raw: &[InitialAccountInputDto],
) -> Result<Vec<InitialAccountInput>, String> {
    raw.iter()
        .map(|a| {
            let amount_fen = a
                .amount_fen
                .parse::<u128>()
                .map_err(|e| format!("amount_fen 解析失败({}):{e}", a.amount_fen))?;
            Ok(InitialAccountInput {
                account_name: a.account_name.clone(),
                amount_fen,
            })
        })
        .collect()
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn build_propose_create_institution_request(
    app: AppHandle,
    pubkey_hex: String,
    cid_number: String,
    cid_full_name: String,
    accounts: Vec<InitialAccountInputDto>,
    org: u8,
    admins: Vec<String>,
    threshold: u32,
    register_nonce: String,
    signature_hex: String,
    issuer_cid_number: String,
    issuer_main_account: String,
    signer_pubkey: String,
    scope_province_name: String,
    scope_city_name: String,
) -> Result<gov_signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let parsed_accounts = parse_initial_accounts(&accounts)?;
        let admins_len = admins.len() as u32;
        super::signing::build_propose_create_institution_sign_request(
            &pubkey_hex,
            &cid_number,
            &cid_full_name,
            &parsed_accounts,
            org,
            admins_len,
            &admins,
            threshold,
            &register_nonce,
            &signature_hex,
            &issuer_cid_number,
            &issuer_main_account,
            &signer_pubkey,
            &scope_province_name,
            &scope_city_name,
        )
    })
    .await
    .map_err(|e| format!("build_propose_create_institution_request task failed:{e}"))?
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn submit_propose_create_institution(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    cid_number: String,
    cid_full_name: String,
    accounts: Vec<InitialAccountInputDto>,
    org: u8,
    admins: Vec<String>,
    threshold: u32,
    register_nonce: String,
    signature_hex: String,
    issuer_cid_number: String,
    issuer_main_account: String,
    signer_pubkey: String,
    scope_province_name: String,
    scope_city_name: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<gov_signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法提交交易".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let parsed_accounts = parse_initial_accounts(&accounts)?;
        let admins_len = admins.len() as u32;
        let call_data = super::signing::build_propose_create_institution_call_data(
            &cid_number,
            &cid_full_name,
            &parsed_accounts,
            org,
            admins_len,
            &admins,
            threshold,
            &register_nonce,
            &signature_hex,
            &issuer_cid_number,
            &issuer_main_account,
            &signer_pubkey,
            &scope_province_name,
            &scope_city_name,
        )?;
        gov_signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit_propose_create_institution task failed:{e}"))?
}
