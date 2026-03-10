use crate::{home::home_node, rpc, settings::bootnodes_address, settings::security};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::Mutex,
    time::Duration,
};
use tauri::AppHandle;

const EXPECTED_SS58_PREFIX: u64 = 2027;
const KNOWN_PEERS_MAX: usize = 5000;
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_RPC_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

static KNOWN_PEERS_IO_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkOverview {
    pub total_nodes: u64,
    pub online_nodes: u64,
    pub guochuhui_nodes: u64,
    pub shengchuhui_nodes: u64,
    pub shengchuhang_nodes: u64,
    pub full_nodes: u64,
    pub light_nodes: u64,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct StoredKnownPeers {
    #[serde(default)]
    peer_ids: Vec<String>,
}

fn known_peers_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("known-peers.json"))
}

fn normalize_peer_id(input: &str) -> Option<String> {
    let v = input.trim();
    if v.is_empty() || v.len() > 128 {
        return None;
    }
    if !v.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(v.to_string())
}

fn load_known_peers(app: &AppHandle) -> Result<Vec<String>, String> {
    let path = known_peers_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read known peers failed: {e}"))?;
    let record: StoredKnownPeers =
        serde_json::from_str(&raw).map_err(|e| format!("parse known peers failed: {e}"))?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut peer_ids: Vec<String> = Vec::new();
    for id in record.peer_ids {
        if let Some(pid) = normalize_peer_id(&id) {
            if seen.insert(pid.clone()) {
                peer_ids.push(pid);
            }
        }
    }
    Ok(peer_ids)
}

fn save_known_peers(app: &AppHandle, peer_ids: &[String]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredKnownPeers {
        peer_ids: peer_ids.to_vec(),
    })
    .map_err(|e| format!("encode known peers failed: {e}"))?;
    let path = known_peers_path(app)?;
    security::write_text_atomic(&path, &format!("{raw}\n"))
        .map_err(|e| format!("write known peers failed ({}): {e}", path.display()))
}

fn trim_known_peers_fifo(peer_ids: &mut Vec<String>) -> bool {
    if peer_ids.len() <= KNOWN_PEERS_MAX {
        return false;
    }
    let excess = peer_ids.len().saturating_sub(KNOWN_PEERS_MAX);
    peer_ids.drain(0..excess);
    true
}

fn push_unique_peer(peer_ids: &mut Vec<String>, present: &mut HashSet<String>, pid: String) {
    if present.insert(pid.clone()) {
        peer_ids.push(pid);
    }
}

struct KnownPeersMergeResult {
    peer_ids: Vec<String>,
    warnings: Vec<String>,
}

fn merge_known_peers(app: &AppHandle, observed_peer_ids: &[String]) -> KnownPeersMergeResult {
    let _guard = KNOWN_PEERS_IO_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut warnings: Vec<String> = Vec::new();

    let mut merged_peer_ids = match load_known_peers(app) {
        Ok(v) => v,
        Err(err) => {
            warnings.push(format!("读取 known-peers 失败，使用空集合: {err}"));
            Vec::new()
        }
    };
    let mut changed = false;
    let mut known_set: HashSet<String> = merged_peer_ids.iter().cloned().collect();
    for pid in observed_peer_ids {
        if known_set.insert(pid.clone()) {
            merged_peer_ids.push(pid.clone());
            changed = true;
        }
    }

    let trimmed = trim_known_peers_fifo(&mut merged_peer_ids);
    if changed || trimmed {
        if let Err(err) = save_known_peers(app, &merged_peer_ids) {
            warnings.push(format!("保存 known-peers 失败: {err}"));
        }
    }
    if trimmed {
        warnings.push(format!(
            "known-peers 超出上限，已按 FIFO 截断到 {} 条",
            KNOWN_PEERS_MAX
        ));
    }

    KnownPeersMergeResult {
        peer_ids: merged_peer_ids,
        warnings,
    }
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(method, params, RPC_REQUEST_TIMEOUT, MAX_RPC_RESPONSE_BYTES)
}

fn ensure_expected_rpc_node() -> Result<Option<String>, String> {
    let properties = rpc_post("system_properties", Value::Array(vec![]))?;
    let ss58 = properties
        .get("ss58Format")
        .and_then(|v| {
            if let Some(raw) = v.as_u64() {
                Some(raw)
            } else {
                v.as_str().and_then(|s| s.parse::<u64>().ok())
            }
        })
        .ok_or_else(|| "RPC 节点缺少 ss58Format".to_string())?;
    if ss58 != EXPECTED_SS58_PREFIX {
        return Err(format!("RPC 链前缀不匹配：expected=2027, got={ss58}"));
    }

    let name = match rpc_post("system_name", Value::Array(vec![])) {
        Ok(v) => v.as_str().map(str::trim).unwrap_or("").to_string(),
        Err(err) => {
            return Ok(Some(format!(
                "读取 system_name 失败，继续使用链前缀校验结果: {err}"
            )));
        }
    };
    if name.is_empty() {
        return Ok(Some("RPC 节点名称为空，继续使用链前缀校验结果".to_string()));
    }

    Ok(None)
}

fn extract_light_role(roles_value: &Value) -> bool {
    if let Some(s) = roles_value.as_str() {
        return s.to_ascii_lowercase().contains("light");
    }
    if let Some(arr) = roles_value.as_array() {
        return arr.iter().any(|v| {
            v.as_str()
                .map(|s| s.to_ascii_lowercase().contains("light"))
                .unwrap_or(false)
        });
    }
    false
}

#[tauri::command]
pub async fn get_network_overview(app: AppHandle) -> Result<NetworkOverview, String> {
    tauri::async_runtime::spawn_blocking(move || get_network_overview_blocking(app))
        .await
        .map_err(|e| format!("network overview task failed: {e}"))?
}

fn get_network_overview_blocking(app: AppHandle) -> Result<NetworkOverview, String> {
    let bootnodes = bootnodes_address::genesis_bootnode_options()?;
    let bootnode_map: HashMap<String, String> = bootnodes
        .iter()
        .map(|n| (n.peer_id.clone(), n.name.clone()))
        .collect();
    let genesis_peer_ids: HashSet<String> = bootnode_map.keys().cloned().collect();

    let mut warnings: Vec<String> = Vec::new();
    let rpc_ready = match ensure_expected_rpc_node() {
        Ok(name_warning) => {
            if let Some(msg) = name_warning {
                warnings.push(msg);
            }
            true
        }
        Err(err) => {
            warnings.push(format!("网络 RPC 校验失败: {err}"));
            false
        }
    };

    let status = home_node::current_status(&app)?;

    let mut online_peer_ids: HashSet<String> = HashSet::new();
    let mut known_peer_observed: Vec<String> = Vec::new();
    let mut known_peer_observed_set: HashSet<String> = HashSet::new();
    let mut remote_light_nodes: u64 = 0;
    let mut local_role_known = false;
    let mut local_is_light = false;
    let mut invalid_peer_count: u64 = 0;
    let mut local_online_extra: u64 = 0;
    let mut local_in_online_set = false;

    if rpc_ready {
        match rpc_post("system_peers", Value::Array(vec![])) {
            Ok(peers) => {
                if let Some(arr) = peers.as_array() {
                    for p in arr {
                        let mut valid_peer = false;
                        if let Some(pid_raw) = p.get("peerId").and_then(Value::as_str) {
                            if let Some(pid) = normalize_peer_id(pid_raw) {
                                online_peer_ids.insert(pid.clone());
                                push_unique_peer(
                                    &mut known_peer_observed,
                                    &mut known_peer_observed_set,
                                    pid,
                                );
                                valid_peer = true;
                            } else {
                                invalid_peer_count = invalid_peer_count.saturating_add(1);
                            }
                        } else {
                            invalid_peer_count = invalid_peer_count.saturating_add(1);
                        }

                        let is_light =
                            valid_peer && p.get("roles").map(extract_light_role).unwrap_or(false);
                        if is_light {
                            remote_light_nodes = remote_light_nodes.saturating_add(1);
                        }
                    }
                } else {
                    warnings.push("system_peers 返回格式无效".to_string());
                }
            }
            Err(err) => warnings.push(format!("读取 system_peers 失败: {err}")),
        }
    }

    if status.running {
        if rpc_ready {
            match rpc_post("system_localPeerId", Value::Array(vec![])) {
                Ok(v) => {
                    if let Some(pid_raw) = v.as_str() {
                        if let Some(pid) = normalize_peer_id(pid_raw) {
                            let _ = online_peer_ids.insert(pid.clone());
                            local_in_online_set = true;
                            push_unique_peer(
                                &mut known_peer_observed,
                                &mut known_peer_observed_set,
                                pid,
                            );
                        } else {
                            local_online_extra = 1;
                            warnings
                                .push("system_localPeerId 格式无效，按本机在线+1 估算".to_string());
                        }
                    } else {
                        local_online_extra = 1;
                        warnings.push("system_localPeerId 返回为空，按本机在线+1 估算".to_string());
                    }
                }
                Err(err) => {
                    local_online_extra = 1;
                    warnings.push(format!(
                        "读取 system_localPeerId 失败，按本机在线+1 估算: {err}"
                    ));
                }
            }

            match rpc_post("system_nodeRoles", Value::Array(vec![])) {
                Ok(roles) => {
                    local_is_light = extract_light_role(&roles);
                    local_role_known = true;
                }
                Err(err) => warnings.push(format!(
                    "读取 system_nodeRoles 失败，无法判定本机轻/全节点: {err}"
                )),
            }
        } else {
            local_online_extra = 1;
        }
    }

    let merge_result = merge_known_peers(&app, &known_peer_observed);
    let known_peer_ids = merge_result.peer_ids;
    warnings.extend(merge_result.warnings);

    if invalid_peer_count > 0 {
        warnings.push(format!("忽略了 {} 条无效 peerId 记录", invalid_peer_count));
    }

    let online_nodes = (online_peer_ids.len() as u64).saturating_add(local_online_extra);
    let local_light_nodes = if status.running && local_role_known && local_is_light {
        1
    } else {
        0
    };
    let light_nodes = remote_light_nodes.saturating_add(local_light_nodes);
    let remote_online_nodes =
        (online_peer_ids.len() as u64).saturating_sub(if local_in_online_set { 1 } else { 0 });
    let remote_full_nodes = remote_online_nodes.saturating_sub(remote_light_nodes);
    let local_full_nodes = if status.running && local_role_known && !local_is_light {
        1
    } else {
        0
    };
    if status.running && !local_role_known {
        warnings.push("未能判定本机轻/全节点，本机未计入 fullNodes".to_string());
    }

    let mut guochuhui_nodes = 0u64;
    let mut shengchuhui_nodes = 0u64;
    let mut shengchuhang_nodes = 0u64;
    let mut uncategorized_bootnodes = 0u64;
    for pid in &online_peer_ids {
        if let Some(name) = bootnode_map.get(pid) {
            if name.contains("国储会") {
                guochuhui_nodes = guochuhui_nodes.saturating_add(1);
            } else if name.contains("省储会") {
                shengchuhui_nodes = shengchuhui_nodes.saturating_add(1);
            } else if name.contains("省储行") || name.contains("储行") {
                shengchuhang_nodes = shengchuhang_nodes.saturating_add(1);
            } else {
                uncategorized_bootnodes = uncategorized_bootnodes.saturating_add(1);
            }
        }
    }
    if uncategorized_bootnodes > 0 {
        warnings.push(format!(
            "{} 个引导节点名称未命中“国储会/省储会/省储行”，按全节点口径处理",
            uncategorized_bootnodes
        ));
    }

    let full_nodes = remote_full_nodes.saturating_add(local_full_nodes);
    let known_non_genesis = known_peer_ids
        .iter()
        .filter(|pid| !genesis_peer_ids.contains(*pid))
        .count() as u64;
    let total_nodes = (bootnodes.len() as u64).saturating_add(known_non_genesis);

    Ok(NetworkOverview {
        total_nodes,
        online_nodes,
        guochuhui_nodes,
        shengchuhui_nodes,
        shengchuhang_nodes,
        full_nodes,
        light_nodes,
        warning: if warnings.is_empty() {
            None
        } else {
            Some(warnings.join("；"))
        },
    })
}
