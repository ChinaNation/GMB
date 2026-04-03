// 身份管理子模块：节点身份信息、状态查询。

use crate::ui::settings::bootnodes_address;
use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;

use super::process::AppState;
use super::rpc::rpc_post;
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
    pub peer_id: Option<String>,
    pub role: Option<String>,
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
    let managed_running = {
        let app_state = app.state::<AppState>();
        let state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;
        state.node_handle.is_some()
    };
    let managed_pid: Option<u32> = None;
    Ok(NodeStatus {
        running: managed_running,
        state: if managed_running {
            "running"
        } else {
            "stopped"
        }
        .to_string(),
        pid: managed_pid,
    })
}

fn get_node_status_sync(app: AppHandle) -> Result<NodeStatus, String> {
    current_status(&app)
}

fn get_node_identity_sync(app: AppHandle) -> Result<NodeIdentity, String> {
    if !current_status(&app)?.running {
        return Ok(NodeIdentity {
            peer_id: None,
            role: Some("全节点".to_string()),
        });
    }

    let local_peer_id = rpc_post("system_localPeerId", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let role = role_from_peer_id(local_peer_id.as_deref());

    Ok(NodeIdentity {
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
pub async fn get_node_identity(app: AppHandle) -> Result<NodeIdentity, String> {
    super::join_blocking_task(
        "get_node_identity",
        tauri::async_runtime::spawn_blocking(move || get_node_identity_sync(app)),
    )
    .await
}
