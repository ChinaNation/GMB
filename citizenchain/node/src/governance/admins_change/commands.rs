use tauri::AppHandle;

use crate::{governance::signing::VoteSignRequestResult, home};

use super::{signing, storage, subject_id, types::AdminSubjectState};

fn resolve_subject_state(
    sfid_number: Option<String>,
    subject_id_hex: Option<String>,
) -> Result<AdminSubjectState, String> {
    if let Some(subject_id_hex) = subject_id_hex.filter(|item| !item.trim().is_empty()) {
        let subject_id = subject_id::subject_id_from_hex(&subject_id_hex)?;
        return storage::fetch_admin_subject(&subject_id, sfid_number)?
            .ok_or_else(|| "链上不存在该管理员主体".to_string());
    }
    if let Some(sfid_number) = sfid_number.filter(|item| !item.trim().is_empty()) {
        return storage::fetch_admin_subject_by_sfid_number(&sfid_number)?
            .ok_or_else(|| "链上不存在该管理员主体".to_string());
    }
    Err("必须提供 sfidNumber 或 subjectIdHex".to_string())
}

/// 获取管理员主体状态。
#[tauri::command]
pub async fn get_admin_subject_state(
    app: AppHandle,
    sfid_number: Option<String>,
    subject_id_hex: Option<String>,
) -> Result<Option<AdminSubjectState>, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询管理员主体".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(subject_id_hex) = subject_id_hex.filter(|item| !item.trim().is_empty()) {
            let subject_id = subject_id::subject_id_from_hex(&subject_id_hex)?;
            storage::fetch_admin_subject(&subject_id, sfid_number)
        } else if let Some(sfid_number) = sfid_number.filter(|item| !item.trim().is_empty()) {
            storage::fetch_admin_subject_by_sfid_number(&sfid_number)
        } else {
            Err("必须提供 sfidNumber 或 subjectIdHex".to_string())
        }
    })
    .await
    .map_err(|e| format!("admin subject task failed: {e}"))?
}

/// 构建管理员更换提案签名请求。
#[tauri::command]
pub async fn build_admin_set_change_request(
    app: AppHandle,
    pubkey_hex: String,
    sfid_number: Option<String>,
    subject_id_hex: Option<String>,
    new_admins: Vec<String>,
) -> Result<VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建管理员更换签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let state = resolve_subject_state(sfid_number, subject_id_hex)?;
        signing::build_admin_set_change_sign_request(&state, &pubkey_hex, &new_admins)
    })
    .await
    .map_err(|e| format!("build admin set change request task failed: {e}"))?
}

/// 验证签名回执并提交管理员更换提案。
#[tauri::command]
pub async fn submit_admin_set_change(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    sfid_number: Option<String>,
    subject_id_hex: Option<String>,
    new_admins: Vec<String>,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<crate::governance::signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交管理员更换提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let state = resolve_subject_state(sfid_number, subject_id_hex)?;
        signing::submit_admin_set_change(
            &state,
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &new_admins,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit admin set change task failed: {e}"))?
}
