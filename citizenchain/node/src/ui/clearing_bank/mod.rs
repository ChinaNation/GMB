// 清算行(Clearing Bank)tab 的 Tauri 命令入口。ADR-007 Step 2 阶段 B 实现。
//
// 模块构成:
// - sfid_proxy   : 转发 SFID `/api/v1/app/clearing-banks/eligible-search`
// - chain        : 链上 ClearingBankNodes / NodePeerToInstitution storage 查询 + 计数
// - connectivity : DNS/wss/链 ID/PeerId 4 重连通性自测
// - signing      : register/update/unregister extrinsic 的 QR 签名请求构造
// - admin_decrypt: 管理员密钥"解密"(wumin sign challenge → 内存标记) 流程
//
// **不**在本模块改造的:
// - submit 通用路径走 `governance::signing::verify_and_submit` 复用
// - 其他业务/治理交易仍由 governance/transaction 模块负责

pub mod admin_decrypt;
pub mod chain;
pub mod connectivity;
pub mod signing;
pub mod sfid_proxy;
pub mod types;

use serde_json::Value;
use std::time::Duration;
use tauri::AppHandle;

use crate::ui::governance::signing as gov_signing;
use crate::ui::home;
use crate::ui::shared::{constants::RPC_RESPONSE_LIMIT_SMALL, rpc};

use self::admin_decrypt::VerifyDecryptAdminInput;
use self::types::{
    ClearingBankNodeOnChainInfo, ConnectivityTestReport, DecryptAdminRequestResult,
    DecryptedAdminInfo, EligibleClearingBankCandidate,
};

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    )
}

// ══════════════════ B1. SFID 候选搜索 ══════════════════

/// 搜索资格白名单内的清算行候选机构(包含未激活,供"添加清算行"页选择)。
#[tauri::command]
pub async fn search_eligible_clearing_banks(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<EligibleClearingBankCandidate>, String> {
    let limit = limit.unwrap_or(20);
    tauri::async_runtime::spawn_blocking(move || {
        sfid_proxy::search_eligible_clearing_banks(&query, limit)
    })
    .await
    .map_err(|e| format!("search_eligible_clearing_banks task failed:{e}"))?
}

// ══════════════════ B2. 链上节点信息查询 ══════════════════

/// 链上查询某机构的清算行节点声明信息。返回 None = 该机构未声明节点。
#[tauri::command]
pub async fn query_clearing_bank_node_info(
    app: AppHandle,
    sfid_id: String,
) -> Result<Option<ClearingBankNodeOnChainInfo>, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法查询链上数据".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || chain::fetch_clearing_bank_node(&sfid_id))
        .await
        .map_err(|e| format!("query_clearing_bank_node_info task failed:{e}"))?
}

// ══════════════════ B3. 本机 PeerId ══════════════════

/// 通过 RPC `system_localPeerId` 拿本机 libp2p PeerId。NodeUI 注册清算行时,
/// 自动把本字段填到"节点 PeerId"输入框,避免人工输入错误。
#[tauri::command]
pub async fn query_local_peer_id(app: AppHandle) -> Result<String, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法查询 PeerId".to_string());
    }
    tauri::async_runtime::spawn_blocking(|| {
        let v = rpc_post("system_localPeerId", Value::Array(vec![]))?;
        v.as_str()
            .map(str::to_string)
            .ok_or_else(|| "system_localPeerId 返回格式无效".to_string())
    })
    .await
    .map_err(|e| format!("query_local_peer_id task failed:{e}"))?
}

// ══════════════════ B4. 连通性自测 ══════════════════

/// 用户填的对外 RPC 域名+端口连通性自测,提交注册前强制 all_ok 才允许签名。
#[tauri::command]
pub async fn test_clearing_bank_endpoint_connectivity(
    domain: String,
    port: u16,
    expected_peer_id: String,
) -> Result<ConnectivityTestReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok::<ConnectivityTestReport, String>(connectivity::run_endpoint_connectivity_test(
            &domain,
            port,
            &expected_peer_id,
        ))
    })
    .await
    .map_err(|e| format!("connectivity test task failed:{e}"))?
}

// ══════════════════ B5. register_clearing_bank ══════════════════

#[tauri::command]
pub async fn build_register_clearing_bank_request(
    app: AppHandle,
    pubkey_hex: String,
    sfid_id: String,
    peer_id: String,
    rpc_domain: String,
    rpc_port: u16,
) -> Result<gov_signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_register_sign_request(&pubkey_hex, &sfid_id, &peer_id, &rpc_domain, rpc_port)
    })
    .await
    .map_err(|e| format!("build_register_clearing_bank task failed:{e}"))?
}

#[tauri::command]
pub async fn submit_register_clearing_bank(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    sfid_id: String,
    peer_id: String,
    rpc_domain: String,
    rpc_port: u16,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<gov_signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行,无法提交交易".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data =
            signing::build_register_call_data(&sfid_id, &peer_id, &rpc_domain, rpc_port)?;
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
    .map_err(|e| format!("submit_register_clearing_bank task failed:{e}"))?
}

// ══════════════════ B6. update_clearing_bank_endpoint ══════════════════

#[tauri::command]
pub async fn build_update_clearing_bank_endpoint_request(
    app: AppHandle,
    pubkey_hex: String,
    sfid_id: String,
    new_domain: String,
    new_port: u16,
) -> Result<gov_signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_update_endpoint_sign_request(&pubkey_hex, &sfid_id, &new_domain, new_port)
    })
    .await
    .map_err(|e| format!("build_update_clearing_bank_endpoint task failed:{e}"))?
}

#[tauri::command]
pub async fn submit_update_clearing_bank_endpoint(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    sfid_id: String,
    new_domain: String,
    new_port: u16,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<gov_signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data =
            signing::build_update_endpoint_call_data(&sfid_id, &new_domain, new_port)?;
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
    .map_err(|e| format!("submit_update_clearing_bank_endpoint task failed:{e}"))?
}

// ══════════════════ B7. unregister_clearing_bank ══════════════════

#[tauri::command]
pub async fn build_unregister_clearing_bank_request(
    app: AppHandle,
    pubkey_hex: String,
    sfid_id: String,
) -> Result<gov_signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_unregister_sign_request(&pubkey_hex, &sfid_id)
    })
    .await
    .map_err(|e| format!("build_unregister_clearing_bank task failed:{e}"))?
}

#[tauri::command]
pub async fn submit_unregister_clearing_bank(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    sfid_id: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<gov_signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = signing::build_unregister_call_data(&sfid_id)?;
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
    .map_err(|e| format!("submit_unregister_clearing_bank task failed:{e}"))?
}

// ══════════════════ B8. 管理员"解密"流程 ══════════════════

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
        admin_decrypt::build_decrypt_admin_request(&pubkey_hex, &sfid_id)
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
        admin_decrypt::verify_and_decrypt_admin(VerifyDecryptAdminInput {
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
    Ok(admin_decrypt::list_decrypted_admins(&sfid_id))
}

#[tauri::command]
pub fn lock_decrypted_admin(pubkey_hex: String) -> Result<(), String> {
    admin_decrypt::lock_decrypted_admin(&pubkey_hex)
}
