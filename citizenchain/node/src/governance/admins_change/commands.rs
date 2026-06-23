use primitives::code::{code_bytes, InstitutionCode};
use tauri::AppHandle;

use crate::{governance::signing::VoteSignRequestResult, home};

use super::{
    account_id, signing, storage,
    types::{
        institution_code_label, is_dynamic_code, is_governance_code, is_valid_institution_code,
        AdminAccountState,
    },
};

/// 把前端传入的机构码字符串(如 "NRC"/"CGOV")转成链上 [u8;4]。空串/缺省 → None。
fn parse_expected_code(expected: Option<&str>) -> Option<InstitutionCode> {
    expected
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(code_bytes)
}

fn resolve_account_state(
    cid_number: Option<String>,
    account_hex: Option<String>,
    expected_institution_code: Option<String>,
) -> Result<AdminAccountState, String> {
    let expected_code = parse_expected_code(expected_institution_code.as_deref());
    validate_account_lookup(expected_code, account_hex.as_deref(), cid_number.as_deref())?;
    if let Some(account_hex) = account_hex.filter(|item| !item.trim().is_empty()) {
        let account_id = account_id::account_id_from_hex(&account_hex)?;
        let state = storage::fetch_admin_account(&account_id, cid_number)?
            .ok_or_else(|| "链上不存在该管理员账户".to_string())?;
        return ensure_expected_code(state, expected_code);
    }
    if let Some(cid_number) = cid_number.filter(|item| !item.trim().is_empty()) {
        let state = storage::fetch_admin_account_by_cid_number(&cid_number)?
            .ok_or_else(|| "链上不存在该管理员账户".to_string())?;
        return ensure_expected_code(state, expected_code);
    }
    Err("必须提供 cidNumber 或 accountHex".to_string())
}

fn validate_account_lookup(
    expected_code: Option<InstitutionCode>,
    account_hex: Option<&str>,
    cid_number: Option<&str>,
) -> Result<(), String> {
    let has_account_id = account_hex
        .map(|item| !item.trim().is_empty())
        .unwrap_or(false);
    let has_cid = cid_number
        .map(|item| !item.trim().is_empty())
        .unwrap_or(false);
    if let Some(code) = expected_code {
        if !is_valid_institution_code(&code) {
            return Err("机构码非法".to_string());
        }
        if is_dynamic_code(&code) && !has_account_id {
            return Err("个人多签或机构账户管理员更换必须提供 accountHex".to_string());
        }
        if is_governance_code(&code) && !has_account_id && !has_cid {
            return Err("治理机构管理员更换必须提供 cidNumber 或 accountHex".to_string());
        }
    }
    if !has_account_id && !has_cid {
        return Err("必须提供 cidNumber 或 accountHex".to_string());
    }
    Ok(())
}

fn ensure_expected_code(
    state: AdminAccountState,
    expected_code: Option<InstitutionCode>,
) -> Result<AdminAccountState, String> {
    if let Some(code) = expected_code {
        if state.institution_code != code {
            return Err(format!(
                "管理员账户机构码不匹配：请求 {}，链上 {}",
                institution_code_label(&code),
                institution_code_label(&state.institution_code)
            ));
        }
    }
    Ok(state)
}

/// 获取管理员账户状态。
#[tauri::command]
pub async fn get_admin_account_state(
    app: AppHandle,
    cid_number: Option<String>,
    account_hex: Option<String>,
    expected_institution_code: Option<String>,
) -> Result<Option<AdminAccountState>, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询管理员账户".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let expected_code = parse_expected_code(expected_institution_code.as_deref());
        validate_account_lookup(expected_code, account_hex.as_deref(), cid_number.as_deref())?;
        if let Some(account_hex) = account_hex.filter(|item| !item.trim().is_empty()) {
            let account_id = account_id::account_id_from_hex(&account_hex)?;
            let state = storage::fetch_admin_account(&account_id, cid_number)?;
            match state {
                Some(state) => ensure_expected_code(state, expected_code).map(Some),
                None => Ok(None),
            }
        } else if let Some(cid_number) = cid_number.filter(|item| !item.trim().is_empty()) {
            let state = storage::fetch_admin_account_by_cid_number(&cid_number)?;
            match state {
                Some(state) => ensure_expected_code(state, expected_code).map(Some),
                None => Ok(None),
            }
        } else {
            Err("必须提供 cidNumber 或 accountHex".to_string())
        }
    })
    .await
    .map_err(|e| format!("admin account task failed: {e}"))?
}

/// 构建管理员更换提案签名请求。
#[tauri::command]
pub async fn build_admin_set_change_request(
    app: AppHandle,
    pubkey_hex: String,
    cid_number: Option<String>,
    account_hex: Option<String>,
    expected_institution_code: Option<String>,
    admins: Vec<String>,
) -> Result<VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建管理员更换签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let state = resolve_account_state(cid_number, account_hex, expected_institution_code)?;
        signing::build_admin_set_change_sign_request(&state, &pubkey_hex, &admins)
    })
    .await
    .map_err(|e| format!("build admin set change request task failed: {e}"))?
}

/// 验证签名响应并提交管理员更换提案。
#[tauri::command]
pub async fn submit_admin_set_change(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    cid_number: Option<String>,
    account_hex: Option<String>,
    expected_institution_code: Option<String>,
    admins: Vec<String>,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<crate::governance::signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交管理员更换提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let state = resolve_account_state(cid_number, account_hex, expected_institution_code)?;
        signing::submit_admin_set_change(
            &state,
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &admins,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit admin set change task failed: {e}"))?
}
