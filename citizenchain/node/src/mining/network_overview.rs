use crate::{
    home,
    settings::bootnodes_address,
    shared::{constants, rpc},
};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tauri::AppHandle;

// 网络统计需要查询较多 peer 信息，给予额外 1 秒余量。
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(4);
use crate::shared::constants::RPC_RESPONSE_LIMIT_LARGE;

/// 网络总览面板对前端返回的聚合统计结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkOverview {
    pub online_nodes: u64,
    pub nrc_nodes: u64,
    pub prc_nodes: u64,
    pub prb_nodes: u64,
    pub full_nodes: u64,
    pub light_nodes: u64,
    pub warning: Option<String>,
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

/// 获取网络总览数据（在线节点、治理节点状态、全节点与轻节点统计）。
/// 前端定期轮询此命令；内部通过 spawn_blocking 避免阻塞 Tauri 主线程。
#[tauri::command]
pub async fn get_network_overview(app: AppHandle) -> Result<NetworkOverview, String> {
    tauri::async_runtime::spawn_blocking(move || get_network_overview_blocking(app))
        .await
        .map_err(|e| format!("network overview task failed: {e}"))?
}

fn get_network_overview_blocking(app: AppHandle) -> Result<NetworkOverview, String> {
    // 网络总览是一个"尽量返回"的聚合接口：
    // 只要能确认当前 RPC 属于目标链，就尽量返回当前可观测在线节点和本机角色状态。
    let bootnodes = bootnodes_address::genesis_bootnode_options()?;
    let bootnode_role_map: HashMap<String, String> = bootnodes
        .iter()
        .map(|n| (n.peer_id.clone(), n.role.clone()))
        .collect();

    let status = home::current_status(&app)?;
    if !status.running {
        return Ok(NetworkOverview {
            online_nodes: 0,
            nrc_nodes: 0,
            prc_nodes: 0,
            prb_nodes: 0,
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

    // 全网节点按角色分成两个互斥集合，都以 peerId 去重。
    //
    // 三个对外数字全部由这两个集合直接得出：full = full 集合大小、light = light 集合大小、
    // online = 两者之和。曾经的写法是 light 汇总全网、online 只取本机直连，再用
    // `online - light` 反推 full —— 减数与被减数根本不是同一个集合，本机没直连到的轻节点
    // 每多一个就少算一个全节点，减到负数还被 saturating_sub 压成 0。同源之后守恒由集合本身
    // 保证，不再需要减法，也不再需要靠猜测本机角色来凑等式。
    let mut network_full_ids: HashSet<String> = HashSet::new();
    let mut network_light_ids: HashSet<String> = HashSet::new();
    // 治理节点计数只认引导节点，按本机与全网汇总到的全部 peerId 匹配。
    let mut local_role_known = false;
    let mut local_is_light = false;
    let mut invalid_peer_count: u64 = 0;

    if rpc_ready {
        match rpc_post("system_peers", Value::Array(vec![])) {
            Ok(peers) => {
                if let Some(arr) = peers.as_array() {
                    for p in arr {
                        if let Some(pid_raw) = p.get("peerId").and_then(Value::as_str) {
                            if let Some(pid) = normalize_peer_id(pid_raw) {
                                // 本机直连到的 peer 同样是全网的一部分，与远程汇总合并去重。
                                let is_light =
                                    p.get("roles").map(extract_light_role).unwrap_or(false);
                                if is_light {
                                    let _ = network_light_ids.insert(pid);
                                } else {
                                    let _ = network_full_ids.insert(pid);
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

    // 遍历所有引导节点远程查询 system_peers，按 peerId 去重汇总全网 full/light 节点。
    // 使用多线程并行查询，总超时 5 秒，避免阻塞 UI；块的返回值是未响应的引导节点数。
    let unreachable_bootnodes: u64 = {
        const REMOTE_RPC_TIMEOUT: Duration = Duration::from_secs(3);
        const REMOTE_RPC_PORT: u16 = 9944;
        let bootnode_domains: Vec<String> = bootnodes
            .iter()
            .filter(|n| !n.domain.is_empty())
            .map(|n| n.domain.clone())
            .collect();

        // 一次遍历同时收 full 与 light：响应里本就带 roles，不必为 full 再发一轮 RPC。
        // 探测失败必须回传，否则该引导节点视野内的节点静默消失、数字偏低且无从察觉。
        struct BootnodeProbe {
            full: HashSet<String>,
            light: HashSet<String>,
            reachable: bool,
        }

        let probed_bootnodes = bootnode_domains.len() as u64;
        let (tx, rx) = std::sync::mpsc::channel::<BootnodeProbe>();
        let active_threads = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for domain in bootnode_domains {
            let tx = tx.clone();
            let active = active_threads.clone();
            active.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            std::thread::spawn(move || {
                let url = format!("http://{}:{}/", domain, REMOTE_RPC_PORT);
                let mut probe = BootnodeProbe {
                    full: HashSet::new(),
                    light: HashSet::new(),
                    reachable: false,
                };
                if let Ok(peers) = rpc::rpc_post_url(
                    &url,
                    "system_peers",
                    Value::Array(vec![]),
                    REMOTE_RPC_TIMEOUT,
                    RPC_RESPONSE_LIMIT_LARGE,
                ) {
                    probe.reachable = true;
                    if let Some(arr) = peers.as_array() {
                        for p in arr {
                            let Some(pid) = p
                                .get("peerId")
                                .and_then(Value::as_str)
                                .and_then(normalize_peer_id)
                            else {
                                continue;
                            };
                            if p.get("roles").map(extract_light_role).unwrap_or(false) {
                                probe.light.insert(pid);
                            } else {
                                probe.full.insert(pid);
                            }
                        }
                    }
                }
                let _ = tx.send(probe);
                active.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            });
        }
        drop(tx); // 关闭发送端，让 rx 在所有线程结束后自然终止

        // 最多等 5 秒收集所有线程的结果
        let mut answered_bootnodes: u64 = 0;
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while let Ok(probe) =
            rx.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
        {
            if probe.reachable {
                answered_bootnodes = answered_bootnodes.saturating_add(1);
            }
            for pid in probe.full {
                let _ = network_full_ids.insert(pid);
            }
            for pid in probe.light {
                let _ = network_light_ids.insert(pid);
            }
        }
        // 超时被丢弃的线程同样计为不可达：它们的节点视野没有进入汇总。
        probed_bootnodes.saturating_sub(answered_bootnodes)
    };

    // 本机自身：拿到 PeerId 与角色后并入对应集合。本机可能已被某个引导节点看到并
    // 汇总进来，HashSet 去重保证不会重复计数。
    let mut local_peer_id: Option<String> = None;
    if rpc_ready {
        match rpc_post("system_localPeerId", Value::Array(vec![])) {
            Ok(v) => match v.as_str().and_then(normalize_peer_id) {
                Some(pid) => local_peer_id = Some(pid),
                None => warnings
                    .push("system_localPeerId 返回为空或格式无效，本机未计入节点数".to_string()),
            },
            Err(err) => warnings.push(format!(
                "读取 system_localPeerId 失败，本机未计入节点数: {err}"
            )),
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
    }

    match (&local_peer_id, local_role_known) {
        (Some(pid), true) => {
            if local_is_light {
                let _ = network_light_ids.insert(pid.clone());
            } else {
                let _ = network_full_ids.insert(pid.clone());
            }
        }
        (Some(_), false) => {
            // 角色未知时**不猜**。旧实现在这里默认按全节点计入，是为了让
            // `full + light == online` 这个等式成立而做的猜测；同源之后守恒由集合保证，
            // 宁可少计本机一台，也不把它归错类。
            warnings.push("未能判定本机轻/全节点，本机未计入节点数".to_string());
        }
        (None, _) => {}
    }

    if invalid_peer_count > 0 {
        warnings.push(format!("忽略了 {} 条无效 peerId 记录", invalid_peer_count));
    }
    if unreachable_bootnodes > 0 {
        // 不可达的引导节点，其视野内的节点不会进入汇总，三个数都会偏低。
        // 不说出来，用户无从判断数字为何变小。
        warnings.push(format!(
            "{} 个引导节点未响应，节点数可能偏低",
            unreachable_bootnodes
        ));
    }

    // 三个对外数字同源于上面两个互斥集合，天然守恒，无减法、无兜底猜测。
    let full_nodes = network_full_ids.len() as u64;
    let light_nodes = network_light_ids.len() as u64;
    let online_nodes = full_nodes.saturating_add(light_nodes);

    let mut nrc_nodes = 0u64;
    let mut prc_nodes = 0u64;
    let mut prb_nodes = 0u64;
    let mut uncategorized_bootnodes = 0u64;
    // 治理节点是引导节点里已在线的那部分；遍历两个集合的并集，与三个数字同源。
    for pid in network_full_ids.iter().chain(network_light_ids.iter()) {
        if let Some(role) = bootnode_role_map.get(pid) {
            match role.as_str() {
                "nrc" => nrc_nodes = nrc_nodes.saturating_add(1),
                "prc" => prc_nodes = prc_nodes.saturating_add(1),
                "prb" => prb_nodes = prb_nodes.saturating_add(1),
                _ => uncategorized_bootnodes = uncategorized_bootnodes.saturating_add(1),
            }
        }
    }
    if uncategorized_bootnodes > 0 {
        warnings.push(format!(
            "{} 个引导节点角色未命中\u{201c}国家储委会/省储委会/省储行\u{201d}，按全节点口径处理",
            uncategorized_bootnodes
        ));
    }

    Ok(NetworkOverview {
        online_nodes,
        nrc_nodes,
        prc_nodes,
        prb_nodes,
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
    fn network_overview_json_excludes_removed_cards() {
        let overview = NetworkOverview {
            online_nodes: 2,
            nrc_nodes: 1,
            prc_nodes: 0,
            prb_nodes: 0,
            full_nodes: 1,
            light_nodes: 1,
            warning: None,
        };
        let json = serde_json::to_value(&overview).expect("NetworkOverview should serialize");
        let object = json
            .as_object()
            .expect("NetworkOverview should be an object");

        assert!(!object.contains_key("totalNodes"));
        assert!(!object.contains_key("clearingNodes"));
        assert!(object.contains_key("onlineNodes"));
        assert!(object.contains_key("fullNodes"));
        assert!(object.contains_key("lightNodes"));
    }
}
