use tauri::AppHandle;

use crate::{governance::signing::VoteSignRequestResult, home};

use super::{
    signing, storage, subject_id,
    types::{is_dynamic_admin_org, is_governance_org, is_valid_org, AdminSubjectState},
};

fn resolve_subject_state(
    sfid_number: Option<String>,
    subject_id_hex: Option<String>,
    expected_org: Option<u8>,
) -> Result<AdminSubjectState, String> {
    validate_subject_lookup(
        expected_org,
        subject_id_hex.as_deref(),
        sfid_number.as_deref(),
    )?;
    if let Some(subject_id_hex) = subject_id_hex.filter(|item| !item.trim().is_empty()) {
        let subject_id = subject_id::subject_id_from_hex(&subject_id_hex)?;
        let state = storage::fetch_admin_subject(&subject_id, sfid_number)?
            .ok_or_else(|| "链上不存在该管理员主体".to_string())?;
        return ensure_expected_org(state, expected_org);
    }
    if let Some(sfid_number) = sfid_number.filter(|item| !item.trim().is_empty()) {
        let state = storage::fetch_admin_subject_by_sfid_number(&sfid_number)?
            .ok_or_else(|| "链上不存在该管理员主体".to_string())?;
        return ensure_expected_org(state, expected_org);
    }
    Err("必须提供 sfidNumber 或 subjectIdHex".to_string())
}

fn validate_subject_lookup(
    expected_org: Option<u8>,
    subject_id_hex: Option<&str>,
    sfid_number: Option<&str>,
) -> Result<(), String> {
    let has_subject_id = subject_id_hex
        .map(|item| !item.trim().is_empty())
        .unwrap_or(false);
    let has_sfid = sfid_number
        .map(|item| !item.trim().is_empty())
        .unwrap_or(false);
    if let Some(org) = expected_org {
        if !is_valid_org(org) {
            return Err("org 必须在 0..=5 范围内".to_string());
        }
        if is_dynamic_admin_org(org) && !has_subject_id {
            return Err("个人多签或机构账户管理员更换必须提供 subjectIdHex".to_string());
        }
        if is_governance_org(org) && !has_subject_id && !has_sfid {
            return Err("治理机构管理员更换必须提供 sfidNumber 或 subjectIdHex".to_string());
        }
    }
    if !has_subject_id && !has_sfid {
        return Err("必须提供 sfidNumber 或 subjectIdHex".to_string());
    }
    Ok(())
}

fn ensure_expected_org(
    state: AdminSubjectState,
    expected_org: Option<u8>,
) -> Result<AdminSubjectState, String> {
    if let Some(org) = expected_org {
        if state.org != org {
            return Err(format!(
                "管理员主体 org 不匹配：请求 org={}，链上 org={}",
                org, state.org
            ));
        }
    }
    Ok(state)
}

/// 获取管理员主体状态。
#[tauri::command]
pub async fn get_admin_subject_state(
    app: AppHandle,
    sfid_number: Option<String>,
    subject_id_hex: Option<String>,
    expected_org: Option<u8>,
) -> Result<Option<AdminSubjectState>, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询管理员主体".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        validate_subject_lookup(
            expected_org,
            subject_id_hex.as_deref(),
            sfid_number.as_deref(),
        )?;
        if let Some(subject_id_hex) = subject_id_hex.filter(|item| !item.trim().is_empty()) {
            let subject_id = subject_id::subject_id_from_hex(&subject_id_hex)?;
            let state = storage::fetch_admin_subject(&subject_id, sfid_number)?;
            match state {
                Some(state) => ensure_expected_org(state, expected_org).map(Some),
                None => Ok(None),
            }
        } else if let Some(sfid_number) = sfid_number.filter(|item| !item.trim().is_empty()) {
            let state = storage::fetch_admin_subject_by_sfid_number(&sfid_number)?;
            match state {
                Some(state) => ensure_expected_org(state, expected_org).map(Some),
                None => Ok(None),
            }
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
    expected_org: Option<u8>,
    new_admins: Vec<String>,
) -> Result<VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建管理员更换签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let state = resolve_subject_state(sfid_number, subject_id_hex, expected_org)?;
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
    expected_org: Option<u8>,
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
        let state = resolve_subject_state(sfid_number, subject_id_hex, expected_org)?;
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
