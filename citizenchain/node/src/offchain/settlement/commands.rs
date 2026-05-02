//! 清算行批次上链前的管理员解锁 Tauri 命令。
//!
//! 中文注释:
//! - `settlement` 目录负责本清算行交易的打包、签名、提交上链。
//! - 管理员解锁只服务清算行批次签名,不等同普通多签账户激活。

use tauri::AppHandle;

use crate::home;
use crate::offchain::common::types::{DecryptAdminRequestResult, DecryptedAdminInfo};

use super::admin_unlock::VerifyDecryptAdminInput;

#[tauri::command]
pub async fn build_decrypt_admin_request(
    app: AppHandle,
    pubkey_hex: String,
    sfid_id: String,
) -> Result<DecryptAdminRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        super::admin_unlock::build_decrypt_admin_request(&pubkey_hex, &sfid_id)
    })
    .await
    .map_err(|e| format!("build_decrypt_admin_request task failed:{e}"))?
}

#[tauri::command]
pub async fn verify_and_decrypt_admin(
    request_id: String,
    pubkey_hex: String,
    expected_payload_hash: String,
    response_json: String,
) -> Result<DecryptedAdminInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        super::admin_unlock::verify_and_decrypt_admin(VerifyDecryptAdminInput {
            request_id,
            pubkey_hex,
            expected_payload_hash,
            response_json,
        })
    })
    .await
    .map_err(|e| format!("verify_and_decrypt_admin task failed:{e}"))?
}

#[tauri::command]
pub async fn list_decrypted_admins(sfid_id: String) -> Result<Vec<DecryptedAdminInfo>, String> {
    Ok(super::admin_unlock::list_decrypted_admins(&sfid_id))
}

#[tauri::command]
pub fn lock_decrypted_admin(pubkey_hex: String) -> Result<(), String> {
    super::admin_unlock::lock_decrypted_admin(&pubkey_hex)
}
