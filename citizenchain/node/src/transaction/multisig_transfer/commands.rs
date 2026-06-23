//! 多签转账 Tauri 命令。

use crate::{governance, home};
use tauri::AppHandle;

/// 构建多签转账提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_multisig_transfer_request(
    app: AppHandle,
    pubkey_hex: String,
    cid_number: String,
    institution_code: String,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
) -> Result<governance::signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    let code = primitives::institution_code::code_bytes(&institution_code);
    tauri::async_runtime::spawn_blocking(move || {
        super::signing::build_propose_transfer_sign_request(
            &pubkey_hex,
            &cid_number,
            code,
            &beneficiary_address,
            amount_yuan,
            &remark,
        )
    })
    .await
    .map_err(|e| format!("build multisig transfer request task failed: {e}"))?
}

/// 验证签名响应并提交多签转账提案。
#[tauri::command]
pub async fn submit_multisig_transfer(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    cid_number: String,
    institution_code: String,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<governance::signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    let code = primitives::institution_code::code_bytes(&institution_code);
    tauri::async_runtime::spawn_blocking(move || {
        let amount_fen = (amount_yuan * 100.0).round() as u128;
        let beneficiary_bytes = governance::signing::decode_ss58_to_pubkey(&beneficiary_address)?;
        let remark_bytes = remark.as_bytes();
        let call_data = super::signing::build_transfer_call_data(
            &cid_number,
            &code,
            &beneficiary_bytes,
            amount_fen,
            remark_bytes,
        )?;

        governance::signing::verify_and_submit(
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
    .map_err(|e| format!("submit multisig transfer task failed: {e}"))?
}

/// 构建安全基金转账提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_multisig_safety_fund_request(
    app: AppHandle,
    pubkey_hex: String,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
) -> Result<governance::signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        super::signing::build_propose_safety_fund_sign_request(
            &pubkey_hex,
            &beneficiary_address,
            amount_yuan,
            &remark,
        )
    })
    .await
    .map_err(|e| format!("build multisig safety fund request task failed: {e}"))?
}

/// 验证签名响应并提交安全基金转账提案。
#[tauri::command]
pub async fn submit_multisig_safety_fund(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<governance::signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = super::signing::build_safety_fund_call_data(
            &beneficiary_address,
            amount_yuan,
            &remark,
        )?;
        governance::signing::verify_and_submit(
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
    .map_err(|e| format!("submit multisig safety fund task failed: {e}"))?
}

/// 构建手续费划转提案签名请求。
#[tauri::command]
pub async fn build_multisig_sweep_request(
    app: AppHandle,
    pubkey_hex: String,
    cid_number: String,
    amount_yuan: f64,
) -> Result<governance::signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        super::signing::build_propose_sweep_sign_request(&pubkey_hex, &cid_number, amount_yuan)
    })
    .await
    .map_err(|e| format!("build multisig sweep failed: {e}"))?
}

/// 验证签名并提交手续费划转提案。
#[tauri::command]
pub async fn submit_multisig_sweep(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    cid_number: String,
    amount_yuan: f64,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<governance::signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = super::signing::build_sweep_call_data(&cid_number, amount_yuan)?;
        governance::signing::verify_and_submit(
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
    .map_err(|e| format!("submit multisig sweep failed: {e}"))?
}
