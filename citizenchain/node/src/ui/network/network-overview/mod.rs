use crate::ui::{
    home,
    settings::bootnodes_address,
    shared::{constants, rpc, security},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::AppHandle;

const KNOWN_PEERS_MAX: usize = 5000;
// 网络统计需要查询较多 peer 信息，给予额外 1 秒余量。
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(4);
use crate::ui::shared::constants::RPC_RESPONSE_LIMIT_LARGE;

struct CachedKnownPeers {
    peers: Vec<String>,
    peer_set: HashSet<String>,
    dirty: bool,
    loaded: bool,
}

impl CachedKnownPeers {
    fn new() -> Self {
        Self {
            peers: Vec::new(),
            peer_set: HashSet::new(),
            dirty: false,
            loaded: false,
        }
    }
}

static KNOWN_PEERS_CACHE: OnceLock<Mutex<CachedKnownPeers>> = OnceLock::new();

fn known_peers_cache() -> &'static Mutex<CachedKnownPeers> {
    KNOWN_PEERS_CACHE.get_or_init(|| Mutex::new(CachedKnownPeers::new()))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 网络总览面板对前端返回的聚合统计结果。
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

struct LoadKnownPeersResult {
    peer_ids: Vec<String>,
    normalized_changed: bool,
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
    // libp2p Ed25519 PeerId 以 "12D3KooW" 开头，长度通常 >= 46 字符。
    if !v.starts_with("12D3KooW") || v.len() < 46 {
        return None;
    }
    Some(v.to_string())
}

fn load_known_peers(app: &AppHandle) -> Result<LoadKnownPeersResult, String> {
    let path = known_peers_path(app)?;
    if !path.exists() {
        return Ok(LoadKnownPeersResult {
            peer_ids: Vec::new(),
            normalized_changed: false,
        });
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read known peers failed: {e}"))?;
    let record: StoredKnownPeers =
        serde_json::from_str(&raw).map_err(|e| format!("parse known peers failed: {e}"))?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut peer_ids: Vec<String> = Vec::new();
    let mut normalized_changed = false;
    for id in record.peer_ids {
        if let Some(pid) = normalize_peer_id(&id) {
            if seen.insert(pid.clone()) {
                if pid != id {
                    normalized_changed = true;
                }
                peer_ids.push(pid);
            } else {
                normalized_changed = true;
            }
        } else {
            normalized_changed = true;
        }
    }
    Ok(LoadKnownPeersResult {
        peer_ids,
        normalized_changed,
    })
}

fn save_known_peers(app: &AppHandle, peer_ids: &[String]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredKnownPeers {
        peer_ids: peer_ids.to_vec(),
    })
    .map_err(|e| format!("encode known peers failed: {e}"))?;
    let path = known_peers_path(app)?;
    security::write_text_atomic_restricted(&path, &format!("{raw}\n")).map_err(|e| {
        format!(
            "write known peers failed ({}): {e}",
            security::sanitize_path(&path)
        )
    })
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

// known-peers 内存缓存：首次访问时从文件加载，后续在内存操作，
// 仅当有新 peer 加入（dirty=true）时才回写磁盘。
fn merge_known_peers(app: &AppHandle, observed_peer_ids: &[String]) -> KnownPeersMergeResult {
    let mut cache = known_peers_cache()
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    let mut warnings: Vec<String> = Vec::new();

    // 首次加载：从文件读取到内存
    if !cache.loaded {
        match load_known_peers(app) {
            Ok(result) => {
                cache.peers = result.peer_ids;
                cache.peer_set = cache.peers.iter().cloned().collect();
                if result.normalized_changed {
                    cache.dirty = true;
                }
            }
            Err(err) => {
                warnings.push(format!("读取 known-peers 失败，使用空集合: {err}"));
            }
        }
        cache.loaded = true;
    }

    // LRU 合并：已知且在线的 peer 移到队尾，全新 peer 追加到队尾。
    // 截断时队首的不活跃旧节点优先被淘汰，活跃节点得以保留。
    for pid in observed_peer_ids {
        if cache.peer_set.contains(pid) {
            if let Some(pos) = cache.peers.iter().position(|p| p == pid) {
                cache.peers.remove(pos);
                cache.peers.push(pid.clone());
                cache.dirty = true;
            }
        } else {
            cache.peer_set.insert(pid.clone());
            cache.peers.push(pid.clone());
            cache.dirty = true;
        }
    }

    let trimmed = trim_known_peers_fifo(&mut cache.peers);
    if trimmed {
        cache.peer_set = cache.peers.iter().cloned().collect();
        cache.dirty = true;
        warnings.push(format!(
            "known-peers 超出上限，已按 LRU 截断到 {} 条",
            KNOWN_PEERS_MAX
        ));
    }

    // 仅 dirty 时写入文件
    if cache.dirty {
        if let Err(err) = save_known_peers(app, &cache.peers) {
            warnings.push(format!("保存 known-peers 失败: {err}"));
        } else {
            cache.dirty = false;
        }
    }

    KnownPeersMergeResult {
        peer_ids: cache.peers.clone(),
        warnings,
    }
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_LARGE,
    )
}

// 网络统计必须建立在"当前 RPC 确认属于目标链"的前提上，
// 否则宁可降级返回告警，也不要继续产出可能误导的网络数据。
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
    if ss58 != constants::EXPECTED_SS58_PREFIX {
        return Err(format!("RPC 链前缀不匹配：expected=2027, got={ss58}"));
    }

    let name = rpc_post("system_name", Value::Array(vec![]))
        .map_err(|err| format!("读取 system_name 失败: {err}"))?
        .as_str()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Err("RPC 节点名称为空".to_string());
    }

    rpc::verify_genesis_hash().map_err(|e| format!("genesis hash 校验失败: {e}"))?;

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

/// 获取网络总览数据（连接节点数、引导节点状态、已知 peer 列表）。
/// 前端定期轮询此命令；内部通过 spawn_blocking 避免阻塞 Tauri 主线程。
#[tauri::command]
pub async fn get_network_overview(app: AppHandle) -> Result<NetworkOverview, String> {
    tauri::async_runtime::spawn_blocking(move || get_network_overview_blocking(app))
        .await
        .map_err(|e| format!("network overview task failed: {e}"))?
}

fn get_network_overview_blocking(app: AppHandle) -> Result<NetworkOverview, String> {
    // 网络总览是一个"尽量返回"的聚合接口：
    // 只要能确认当前 RPC 属于目标链，就尽量返回已知在线节点、历史节点和本机状态。
    let bootnodes = bootnodes_address::genesis_bootnode_options()?;
    let bootnode_name_map: HashMap<String, String> = bootnodes
        .iter()
        .map(|n| (n.peer_id.clone(), n.name.clone()))
        .collect();
    let bootnode_role_map: HashMap<String, String> = bootnodes
        .iter()
        .map(|n| (n.peer_id.clone(), n.role.clone()))
        .collect();
    let genesis_peer_ids: HashSet<String> = bootnode_name_map.keys().cloned().collect();

    let status = home::current_status(&app)?;
    if !status.running {
        return Ok(NetworkOverview {
            total_nodes: 0,
            online_nodes: 0,
            guochuhui_nodes: 0,
            shengchuhui_nodes: 0,
            shengchuhang_nodes: 0,
            full_nodes: 0,
            light_nodes: 0,
            warning: None,
        });
    }

    let mut warnings: Vec<String> = Vec::new();
    let rpc_ready = match ensure_expected_rpc_node() {
        Ok(()) => true,
        Err(err) => {
            warnings.push(format!("网络 RPC 校验失败: {err}"));
            false
        }
    };

    let mut online_peer_ids: HashSet<String> = HashSet::new();
    let mut known_peer_observed: Vec<String> = Vec::new();
    let mut known_peer_observed_set: HashSet<String> = HashSet::new();
    let mut remote_light_peer_ids: HashSet<String> = HashSet::new();
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
                        if let Some(pid_raw) = p.get("peerId").and_then(Value::as_str) {
                            if let Some(pid) = normalize_peer_id(pid_raw) {
                                online_peer_ids.insert(pid.clone());
                                push_unique_peer(
                                    &mut known_peer_observed,
                                    &mut known_peer_observed_set,
                                    pid.clone(),
                                );
                                let is_light =
                                    p.get("roles").map(extract_light_role).unwrap_or(false);
                                if is_light {
                                    let _ = remote_light_peer_ids.insert(pid);
                                }
                            } else {
                                invalid_peer_count = invalid_peer_count.saturating_add(1);
                            }
                        } else {
                            invalid_peer_count = invalid_peer_count.saturating_add(1);
                        }
                    }
                } else {
                    warnings.push("system_peers 返回格式无效".to_string());
                }
            }
            Err(err) => warnings.push(format!("读取 system_peers 失败: {err}")),
        }
    }

    // 遍历所有引导节点远程查询 system_peers，汇总全网轻节点（按 peerId 去重）。
    // 使用多线程并行查询，总超时 5 秒，避免阻塞 UI。
    {
        const REMOTE_RPC_TIMEOUT: Duration = Duration::from_secs(3);
        const REMOTE_RPC_PORT: u16 = 9944;
        let bootnode_domains: Vec<String> = bootnodes
            .iter()
            .filter(|n| !n.domain.is_empty())
            .map(|n| n.domain.clone())
            .collect();

        let (tx, rx) = std::sync::mpsc::channel::<HashSet<String>>();
        let active_threads = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for domain in bootnode_domains {
            let tx = tx.clone();
            let active = active_threads.clone();
            active.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            std::thread::spawn(move || {
                let url = format!("http://{}:{}/", domain, REMOTE_RPC_PORT);
                let mut light_pids = HashSet::new();
                if let Ok(peers) = rpc::rpc_post_url(
                    &url,
                    "system_peers",
                    Value::Array(vec![]),
                    REMOTE_RPC_TIMEOUT,
                    RPC_RESPONSE_LIMIT_LARGE,
                ) {
                    if let Some(arr) = peers.as_array() {
                        for p in arr {
                            let is_light = p.get("roles").map(extract_light_role).unwrap_or(false);
                            if is_light {
                                if let Some(pid_raw) = p.get("peerId").and_then(Value::as_str) {
                                    if let Some(pid) = normalize_peer_id(pid_raw) {
                                        light_pids.insert(pid);
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = tx.send(light_pids);
                active.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            });
        }
        drop(tx); // 关闭发送端，让 rx 在所有线程结束后自然终止

        // 最多等 5 秒收集所有线程的结果
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while let Ok(light_pids) =
            rx.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
        {
            for pid in light_pids {
                let _ = remote_light_peer_ids.insert(pid);
            }
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

    // full/light 统计需要与 onlineNodes 口径一致：
    // 远端按唯一 peerId 去重，本机角色未知时按 full 兜底，避免 total 不守恒。
    let online_nodes = (online_peer_ids.len() as u64).saturating_add(local_online_extra);
    let remote_light_nodes = remote_light_peer_ids.len() as u64;
    let local_light_nodes = if status.running && local_role_known && local_is_light {
        1
    } else {
        0
    };
    let light_nodes = remote_light_nodes.saturating_add(local_light_nodes);
    let remote_online_nodes =
        (online_peer_ids.len() as u64).saturating_sub(if local_in_online_set { 1 } else { 0 });
    let remote_full_nodes = remote_online_nodes.saturating_sub(remote_light_nodes);
    let local_full_nodes = if status.running && (!local_role_known || !local_is_light) {
        1
    } else {
        0
    };
    if status.running && !local_role_known {
        warnings.push("未能判定本机轻/全节点，本机按全节点口径计入 fullNodes".to_string());
    }

    let mut guochuhui_nodes = 0u64;
    let mut shengchuhui_nodes = 0u64;
    let mut shengchuhang_nodes = 0u64;
    let mut uncategorized_bootnodes = 0u64;
    for pid in &online_peer_ids {
        if let Some(role) = bootnode_role_map.get(pid) {
            match role.as_str() {
                "guochuhui" => guochuhui_nodes = guochuhui_nodes.saturating_add(1),
                "shengchuhui" => shengchuhui_nodes = shengchuhui_nodes.saturating_add(1),
                "shengchuhang" => shengchuhang_nodes = shengchuhang_nodes.saturating_add(1),
                _ => uncategorized_bootnodes = uncategorized_bootnodes.saturating_add(1),
            }
        }
    }
    if uncategorized_bootnodes > 0 {
        warnings.push(format!(
            "{} 个引导节点名称未命中\u{201c}国储会/省储会/省储行\u{201d}，按全节点口径处理",
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

#[cfg(test)]
mod tests {
    use super::*;

    // 合法 PeerId 示例：以 "12D3KooW" 开头 + 至少 46 字符 + 纯 ASCII 字母数字
    const VALID_PEER_ID: &str = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";

    #[test]
    fn normalize_peer_id_valid() {
        assert_eq!(
            normalize_peer_id(VALID_PEER_ID),
            Some(VALID_PEER_ID.to_string())
        );
    }

    #[test]
    fn normalize_peer_id_trims_whitespace() {
        let padded = format!("  {VALID_PEER_ID}  ");
        assert_eq!(normalize_peer_id(&padded), Some(VALID_PEER_ID.to_string()));
    }

    #[test]
    fn normalize_peer_id_empty_rejected() {
        assert_eq!(normalize_peer_id(""), None);
        assert_eq!(normalize_peer_id("   "), None);
    }

    #[test]
    fn normalize_peer_id_too_long_rejected() {
        let long = format!("12D3KooW{}", "a".repeat(121)); // 8 + 121 = 129 > 128
        assert_eq!(normalize_peer_id(&long), None);
    }

    #[test]
    fn normalize_peer_id_max_length_ok() {
        let max = format!("12D3KooW{}", "a".repeat(120)); // 8 + 120 = 128
        assert!(normalize_peer_id(&max).is_some());
    }

    #[test]
    fn normalize_peer_id_too_short_rejected() {
        // 以 12D3KooW 开头但不足 46 字符
        assert_eq!(normalize_peer_id("12D3KooWShort"), None);
    }

    #[test]
    fn normalize_peer_id_non_alphanumeric_rejected() {
        assert_eq!(normalize_peer_id("12D3/KooW"), None);
        assert_eq!(normalize_peer_id("peer-id"), None);
    }

    #[test]
    fn trim_known_peers_fifo_under_limit() {
        let mut peers: Vec<String> = (0..10).map(|i| format!("peer{i}")).collect();
        assert!(!trim_known_peers_fifo(&mut peers));
        assert_eq!(peers.len(), 10);
    }

    #[test]
    fn trim_known_peers_fifo_over_limit() {
        let mut peers: Vec<String> = (0..KNOWN_PEERS_MAX + 5)
            .map(|i| format!("peer{i}"))
            .collect();
        assert!(trim_known_peers_fifo(&mut peers));
        assert_eq!(peers.len(), KNOWN_PEERS_MAX);
        // FIFO: earliest entries removed
        assert_eq!(peers[0], "peer5");
    }
}
