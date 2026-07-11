use super::{call_data, signing as runtime_signing, types::ProposeUpgradeRequestResult};
use crate::{
    admins::management::activation,
    governance::registry,
    governance::signing::{self, VoteSignRequestResult, VoteSubmitResult},
    home,
};
use tauri::AppHandle;

fn normalize_pubkey_hex(pubkey_hex: &str) -> String {
    pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase()
}

async fn ensure_nrc_activated_admin(app: &AppHandle, pubkey_hex: &str) -> Result<(), String> {
    let nrc_cid_number = registry::governance_overview()
        .national_councils
        .first()
        .map(|item| item.cid_number.clone())
        .ok_or_else(|| "国家储委会机构常量缺失，无法发起开发升级".to_string())?;
    let pubkey_clean = normalize_pubkey_hex(pubkey_hex);
    let admins = activation::get_activated_admins(app.clone(), nrc_cid_number, None, None).await?;
    if admins
        .iter()
        .any(|admin| normalize_pubkey_hex(&admin.pubkey_hex) == pubkey_clean)
    {
        Ok(())
    } else {
        Err("开发升级仅允许已激活国家储委会管理员发起".to_string())
    }
}

/// 构建开发期直接升级签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_developer_upgrade_request(
    app: AppHandle,
    pubkey_hex: String,
    wasm_path: String,
) -> Result<VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    ensure_nrc_activated_admin(&app, &pubkey_hex).await?;
    tauri::async_runtime::spawn_blocking(move || {
        runtime_signing::build_developer_upgrade_sign_request(&pubkey_hex, &wasm_path)
    })
    .await
    .map_err(|e| format!("build developer upgrade request task failed: {e}"))?
}

/// 验证签名响应并提交开发期直接升级。
#[tauri::command]
pub async fn submit_developer_upgrade(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    wasm_path: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交升级".to_string());
    }
    ensure_nrc_activated_admin(&app, &expected_pubkey_hex).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = call_data::developer_direct_upgrade_from_file(&wasm_path)?;
        signing::verify_and_submit(
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
    .map_err(|e| format!("submit developer upgrade task failed: {e}"))?
}

/// 构建运行期协议升级提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_propose_upgrade_request(
    app: AppHandle,
    pubkey_hex: String,
    wasm_path: String,
    reason: String,
) -> Result<ProposeUpgradeRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let sign_result = runtime_signing::build_propose_runtime_upgrade_sign_request(
            &pubkey_hex,
            &wasm_path,
            &reason,
        )?;
        Ok(ProposeUpgradeRequestResult {
            request_json: sign_result.request_json,
            request_id: sign_result.request_id,
            expected_payload_hash: sign_result.expected_payload_hash,
            sign_nonce: sign_result.sign_nonce,
            sign_block_number: sign_result.sign_block_number,
        })
    })
    .await
    .map_err(|e| format!("build propose upgrade request task failed: {e}"))?
}

/// 验证签名响应并提交运行期协议升级提案。
#[tauri::command]
pub async fn submit_propose_upgrade(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    wasm_path: String,
    reason: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = call_data::propose_runtime_upgrade_from_file(&wasm_path, &reason)?;
        signing::verify_and_submit(
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
    .map_err(|e| format!("submit propose upgrade task failed: {e}"))?
}
