use crate::{home::home_node, settings::bootnodes_address, settings::security};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    time::Duration,
};
use tauri::AppHandle;

const RPC_ADDR: &str = "127.0.0.1:9944";
const EXPECTED_SS58_PREFIX: u64 = 2027;
const MAX_RPC_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const KNOWN_PEERS_MAX: usize = 5000;

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

fn load_known_peers(app: &AppHandle) -> Result<HashSet<String>, String> {
    let path = known_peers_path(app)?;
    if !path.exists() {
        return Ok(HashSet::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read known peers failed: {e}"))?;
    let record: StoredKnownPeers =
        serde_json::from_str(&raw).map_err(|e| format!("parse known peers failed: {e}"))?;
    Ok(record
        .peer_ids
        .into_iter()
        .filter_map(|id| normalize_peer_id(&id))
        .collect())
}

fn save_known_peers(app: &AppHandle, peers: &HashSet<String>) -> Result<(), String> {
    let mut peer_ids: Vec<String> = peers.iter().cloned().collect();
    peer_ids.sort();
    let raw = serde_json::to_string_pretty(&StoredKnownPeers { peer_ids })
        .map_err(|e| format!("encode known peers failed: {e}"))?;
    fs::write(known_peers_path(app)?, format!("{raw}\n"))
        .map_err(|e| format!("write known peers failed: {e}"))
}

fn trim_known_peers(peers: &mut HashSet<String>) -> bool {
    if peers.len() <= KNOWN_PEERS_MAX {
        return false;
    }
    let mut ids: Vec<String> = peers.iter().cloned().collect();
    ids.sort();
    ids.truncate(KNOWN_PEERS_MAX);
    peers.clear();
    peers.extend(ids);
    true
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    })
    .to_string();

    let req = format!(
        "POST / HTTP/1.1\r\nHost: {RPC_ADDR}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let addr = RPC_ADDR
        .parse()
        .map_err(|e| format!("parse RPC socket address failed: {e}"))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(600))
        .map_err(|e| format!("RPC 连接失败: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set RPC read timeout failed: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set RPC write timeout failed: {e}"))?;

    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("RPC 写入失败: {e}"))?;

    let mut response = String::new();
    stream
        .take(MAX_RPC_RESPONSE_BYTES)
        .read_to_string(&mut response)
        .map_err(|e| format!("RPC 读取失败: {e}"))?;

    let Some((header, body)) = response.split_once("\r\n\r\n") else {
        return Err("RPC 响应格式错误：缺少 header/body 分隔符".to_string());
    };
    let status_line = header
        .lines()
        .next()
        .ok_or_else(|| "RPC 响应格式错误：缺少状态行".to_string())?;
    if !status_line.contains(" 200 ") {
        return Err(format!("RPC HTTP 状态异常: {status_line}"));
    }

    let json: Value = serde_json::from_str(body).map_err(|e| format!("RPC JSON 解析失败: {e}"))?;
    if let Some(err) = json.get("error") {
        return Err(format!("RPC 返回错误: {err}"));
    }

    Ok(json.get("result").cloned().unwrap_or(Value::Null))
}

fn ensure_expected_rpc_node() -> Result<(), String> {
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

    let name = rpc_post("system_name", Value::Array(vec![]))?
        .as_str()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Err("RPC 节点名称为空".to_string());
    }

    Ok(())
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
pub fn get_network_overview(app: AppHandle) -> Result<NetworkOverview, String> {
    let bootnodes = bootnodes_address::genesis_bootnode_options()?;
    let bootnode_map: HashMap<String, String> = bootnodes
        .iter()
        .map(|n| (n.peer_id.clone(), n.name.clone()))
        .collect();
    let genesis_peer_ids: HashSet<String> = bootnode_map.keys().cloned().collect();

    let mut warnings: Vec<String> = Vec::new();

    let mut known_peer_ids = match load_known_peers(&app) {
        Ok(v) => v,
        Err(err) => {
            warnings.push(format!("读取 known-peers 失败，使用空集合: {err}"));
            HashSet::new()
        }
    };
    let known_before = known_peer_ids.clone();

    let rpc_ready = match ensure_expected_rpc_node() {
        Ok(()) => true,
        Err(err) => {
            warnings.push(format!("网络 RPC 校验失败: {err}"));
            false
        }
    };

    let mut online_peer_ids: HashSet<String> = HashSet::new();
    let mut light_nodes: u64 = 0;
    let mut invalid_peer_count: u64 = 0;
    let mut local_online_extra: u64 = 0;

    if rpc_ready {
        match rpc_post("system_peers", Value::Array(vec![])) {
            Ok(peers) => {
                if let Some(arr) = peers.as_array() {
                    for p in arr {
                        if let Some(pid_raw) = p.get("peerId").and_then(Value::as_str) {
                            if let Some(pid) = normalize_peer_id(pid_raw) {
                                online_peer_ids.insert(pid.clone());
                                known_peer_ids.insert(pid);
                            } else {
                                invalid_peer_count = invalid_peer_count.saturating_add(1);
                            }
                        } else {
                            invalid_peer_count = invalid_peer_count.saturating_add(1);
                        }

                        let is_light = p.get("roles").map(extract_light_role).unwrap_or(false);
                        if is_light {
                            light_nodes = light_nodes.saturating_add(1);
                        }
                    }
                } else {
                    warnings.push("system_peers 返回格式无效".to_string());
                }
            }
            Err(err) => warnings.push(format!("读取 system_peers 失败: {err}")),
        }
    }

    let status = home_node::current_status(&app)?;
    if status.running {
        if rpc_ready {
            match rpc_post("system_localPeerId", Value::Array(vec![])) {
                Ok(v) => {
                    if let Some(pid_raw) = v.as_str() {
                        if let Some(pid) = normalize_peer_id(pid_raw) {
                            online_peer_ids.insert(pid.clone());
                            known_peer_ids.insert(pid);
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
        } else {
            local_online_extra = 1;
        }
    }

    let trimmed = trim_known_peers(&mut known_peer_ids);
    let known_changed = known_peer_ids != known_before;
    if known_changed {
        if let Err(err) = save_known_peers(&app, &known_peer_ids) {
            warnings.push(format!("保存 known-peers 失败: {err}"));
        }
    }

    if invalid_peer_count > 0 {
        warnings.push(format!("忽略了 {} 条无效 peerId 记录", invalid_peer_count));
    }
    if trimmed {
        warnings.push(format!(
            "known-peers 超出上限，已截断到 {} 条",
            KNOWN_PEERS_MAX
        ));
    }

    let online_nodes = (online_peer_ids.len() as u64).saturating_add(local_online_extra);

    let mut guochuhui_nodes = 0u64;
    let mut shengchuhui_nodes = 0u64;
    let mut shengchuhang_nodes = 0u64;
    for pid in &online_peer_ids {
        if let Some(name) = bootnode_map.get(pid) {
            if name.contains("国储会") {
                guochuhui_nodes = guochuhui_nodes.saturating_add(1);
            } else if name.contains("储行") {
                shengchuhang_nodes = shengchuhang_nodes.saturating_add(1);
            } else {
                shengchuhui_nodes = shengchuhui_nodes.saturating_add(1);
            }
        }
    }

    let full_nodes = online_nodes.saturating_sub(light_nodes);
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
