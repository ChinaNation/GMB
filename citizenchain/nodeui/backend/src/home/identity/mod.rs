// 身份管理子模块：节点身份信息、名称管理、状态查询。

use crate::{
    settings::{bootnodes_address, device_password},
    shared::{security, validation::normalize_node_name},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tauri::AppHandle;

use super::process::{
    refresh_managed_process, trusted_node_process_pids_on_rpc_port, AppState,
};
use super::rpc::{is_expected_rpc_node, rpc_post};
use tauri::Manager;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 首页展示的节点运行状态。
pub struct NodeStatus {
    pub running: bool,
    pub state: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 首页展示的节点身份信息。
pub struct NodeIdentity {
    pub node_name: Option<String>,
    pub peer_id: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredNodeName {
    node_name: String,
}

fn node_name_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("node-name.json"))
}

pub(super) fn load_node_name(app: &AppHandle) -> Result<Option<String>, String> {
    let path = node_name_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read node-name failed: {e}"))?;
    let record: StoredNodeName =
        serde_json::from_str(&raw).map_err(|e| format!("parse node-name failed: {e}"))?;
    Ok(Some(record.node_name))
}

// 角色由 bootnode 名称映射得出，未命中时统一按"全节点"展示。
fn role_from_peer_id(peer_id: Option<&str>) -> String {
    if let Some(pid) = peer_id {
        if let Ok(Some(name)) = bootnodes_address::find_genesis_bootnode_name_by_peer_id(pid) {
            return name;
        }
    }
    "全节点".to_string()
}

pub(crate) fn current_status(app: &AppHandle) -> Result<NodeStatus, String> {
    // 先看本会话托管进程，再看可信监听进程，最后才退化到 RPC 指纹探测，
    // 减少仅凭 9944 端口连通就误判为"节点在运行"的概率。
    let (managed_running, managed_pid) = {
        let app_state = app.state::<AppState>();
        let mut state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;
        refresh_managed_process(&mut state)
    };
    if managed_running {
        return Ok(NodeStatus {
            running: true,
            state: "running".to_string(),
            pid: managed_pid,
        });
    }

    let listener_pids = trusted_node_process_pids_on_rpc_port(app)?;
    if let Some(pid) = listener_pids.into_iter().next() {
        return Ok(NodeStatus {
            running: true,
            state: "running".to_string(),
            pid: Some(pid),
        });
    }

    let fallback_running = is_expected_rpc_node();
    Ok(NodeStatus {
        running: fallback_running,
        state: if fallback_running {
            "running"
        } else {
            "stopped"
        }
        .to_string(),
        pid: None,
    })
}

fn get_node_status_sync(app: AppHandle) -> Result<NodeStatus, String> {
    current_status(&app)
}

fn get_node_identity_sync(app: AppHandle) -> Result<NodeIdentity, String> {
    let configured_node_name = load_node_name(&app)?;
    if !current_status(&app)?.running {
        return Ok(NodeIdentity {
            node_name: configured_node_name,
            peer_id: None,
            role: Some("全节点".to_string()),
        });
    }

    let rpc_node_name = rpc_post("system_name", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let node_name = configured_node_name.or(rpc_node_name);

    let local_peer_id = rpc_post("system_localPeerId", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let role = role_from_peer_id(local_peer_id.as_deref());

    Ok(NodeIdentity {
        node_name,
        peer_id: local_peer_id,
        role: Some(role),
    })
}

pub(crate) fn get_node_identity_blocking(app: AppHandle) -> Result<NodeIdentity, String> {
    get_node_identity_sync(app)
}

#[tauri::command]
pub async fn get_node_status(app: AppHandle) -> Result<NodeStatus, String> {
    super::join_blocking_task(
        "get_node_status",
        tauri::async_runtime::spawn_blocking(move || get_node_status_sync(app)),
    )
    .await
}

#[tauri::command]
pub fn set_node_name(
    app: AppHandle,
    node_name: String,
    unlock_password: String,
) -> Result<NodeIdentity, String> {
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;
    let normalized = normalize_node_name(&node_name)?;
    let raw = serde_json::to_string_pretty(&StoredNodeName {
        node_name: normalized.clone(),
    })
    .map_err(|e| format!("encode node-name failed: {e}"))?;

    security::write_text_atomic(&node_name_path(&app)?, &format!("{raw}\n"))
        .map_err(|e| format!("write node-name failed: {e}"))?;

    let peer_id = rpc_post("system_localPeerId", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let role = role_from_peer_id(peer_id.as_deref());

    Ok(NodeIdentity {
        node_name: Some(normalized),
        peer_id,
        role: Some(role),
    })
}

#[tauri::command]
pub async fn get_node_identity(app: AppHandle) -> Result<NodeIdentity, String> {
    super::join_blocking_task(
        "get_node_identity",
        tauri::async_runtime::spawn_blocking(move || get_node_identity_sync(app)),
    )
    .await
}
