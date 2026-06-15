use crate::{
    home,
    im::{endpoint::ImNodeEndpoint, rpc},
    shared::{rpc as shared_rpc, security},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs,
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

const COMMUNICATION_NODE_FILE_NAME: &str = "communication-node.json";
const IM_NODE_PAIRING_QR_PROTO: &str = "WUMIN_QR_V1";
const IM_NODE_PAIRING_KIND: &str = "im_node_pairing";
const IM_NODE_PAIRING_BODY_PROTO: &str = "GMB_IM_NODE_PAIRING_V1";
const IM_NODE_PAIRING_TTL_MILLIS: u64 = 10 * 60 * 1000;
const IM_P2P_PORT: u16 = 30333;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunicationNodeState {
    pub enabled: bool,
    pub peer_id: Option<String>,
    pub rpc_url: String,
    pub node_multiaddr: Option<String>,
    pub endpoint_kind: Option<String>,
    pub pairing_payload: Option<String>,
    pub expires_at_millis: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredCommunicationNode {
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommunicationNodeEndpoint {
    rpc_url: String,
    multiaddr: String,
    kind: String,
}

fn communication_node_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join(COMMUNICATION_NODE_FILE_NAME))
}

fn load_enabled(app: &AppHandle) -> Result<bool, String> {
    let path = communication_node_path(app)?;
    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(format!("read communication node setting failed: {err}")),
    };
    let stored: StoredCommunicationNode = serde_json::from_str(&raw)
        .map_err(|err| format!("parse communication node setting failed: {err}"))?;
    Ok(stored.enabled)
}

fn save_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredCommunicationNode { enabled })
        .map_err(|err| format!("encode communication node setting failed: {err}"))?;
    security::write_text_atomic_restricted(&communication_node_path(app)?, &format!("{raw}\n"))
        .map_err(|err| format!("write communication node setting failed: {err}"))
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}

fn lan_ip() -> IpAddr {
    detect_lan_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)))
        .or_else(|| {
            detect_lan_ip(IpAddr::V6(Ipv6Addr::new(
                0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888,
            )))
        })
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

fn detect_lan_ip(remote_ip: IpAddr) -> Option<IpAddr> {
    let bind_addr = match remote_ip {
        IpAddr::V4(_) => SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)),
        IpAddr::V6(_) => SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)),
    };
    let socket = UdpSocket::bind(bind_addr).ok()?;
    socket.connect(SocketAddr::from((remote_ip, 80))).ok()?;
    Some(socket.local_addr().ok()?.ip())
}

fn endpoint_for_peer(
    peer_id: &str,
    ip: IpAddr,
    rpc_port: u16,
) -> Result<CommunicationNodeEndpoint, String> {
    let (kind, ip_text) = match ip {
        IpAddr::V4(value) => ("ip4", value.to_string()),
        IpAddr::V6(value) => ("ip6", value.to_string()),
    };
    let rpc_host = match ip {
        IpAddr::V4(value) => value.to_string(),
        IpAddr::V6(value) => format!("[{value}]"),
    };
    let multiaddr = format!("/{kind}/{ip_text}/tcp/{IM_P2P_PORT}/wss/p2p/{peer_id}");
    let endpoint = ImNodeEndpoint::checked(peer_id, &multiaddr)?;
    Ok(CommunicationNodeEndpoint {
        rpc_url: format!("http://{rpc_host}:{rpc_port}/"),
        multiaddr: endpoint.multiaddr,
        kind: endpoint.kind,
    })
}

fn pairing_nonce() -> String {
    hex::encode(rand::random::<[u8; 16]>())
}

fn build_pairing_payload(
    peer_id: &str,
    endpoint: &CommunicationNodeEndpoint,
    now: u64,
    nonce: &str,
) -> Result<(String, u64), String> {
    let expires_at_millis = now.saturating_add(IM_NODE_PAIRING_TTL_MILLIS);
    let issued_at = now / 1000;
    let expires_at = expires_at_millis / 1000;
    let payload = json!({
        "proto": IM_NODE_PAIRING_QR_PROTO,
        "kind": IM_NODE_PAIRING_KIND,
        "id": format!("im-node-{nonce}"),
        "issued_at": issued_at,
        "expires_at": expires_at,
        "body": {
            "proto": IM_NODE_PAIRING_BODY_PROTO,
            "node_peer_id": peer_id,
            "rpc_url": endpoint.rpc_url,
            "node_multiaddr": endpoint.multiaddr,
            "endpoint_kind": endpoint.kind,
            "pairing_nonce": nonce,
            "created_at_millis": now,
            "expires_at_millis": expires_at_millis,
        }
    });
    let raw = serde_json::to_string(&payload)
        .map_err(|err| format!("encode communication node pairing payload failed: {err}"))?;
    Ok((raw, expires_at_millis))
}

fn build_state(app: AppHandle, enabled: bool) -> Result<CommunicationNodeState, String> {
    rpc::set_owner_rpc_runtime_enabled(enabled);
    let identity = home::get_node_identity_blocking(app)?;
    let rpc_url = shared_rpc::local_rpc_http_url();
    let Some(peer_id) = identity.peer_id else {
        return Ok(CommunicationNodeState {
            enabled,
            peer_id: None,
            rpc_url,
            node_multiaddr: None,
            endpoint_kind: None,
            pairing_payload: None,
            expires_at_millis: None,
        });
    };
    if !enabled {
        return Ok(CommunicationNodeState {
            enabled,
            peer_id: Some(peer_id),
            rpc_url,
            node_multiaddr: None,
            endpoint_kind: None,
            pairing_payload: None,
            expires_at_millis: None,
        });
    }

    let endpoint = endpoint_for_peer(&peer_id, lan_ip(), shared_rpc::current_rpc_port())?;
    let (pairing_payload, expires_at_millis) =
        build_pairing_payload(&peer_id, &endpoint, now_millis(), &pairing_nonce())?;
    Ok(CommunicationNodeState {
        enabled,
        peer_id: Some(peer_id),
        rpc_url: endpoint.rpc_url,
        node_multiaddr: Some(endpoint.multiaddr),
        endpoint_kind: Some(endpoint.kind),
        pairing_payload: Some(pairing_payload),
        expires_at_millis: Some(expires_at_millis),
    })
}

#[tauri::command]
pub fn get_communication_node(app: AppHandle) -> Result<CommunicationNodeState, String> {
    let enabled = load_enabled(&app)?;
    build_state(app, enabled)
}

#[tauri::command]
pub fn set_communication_node_enabled(
    app: AppHandle,
    enabled: bool,
) -> Result<CommunicationNodeState, String> {
    if let Err(err) = security::append_audit_log(&app, "set_communication_node_enabled", "attempt")
    {
        eprintln!("[审计] set_communication_node_enabled attempt 日志写入失败: {err}");
    }
    save_enabled(&app, enabled)?;
    let state = build_state(app.clone(), enabled)?;
    if let Err(err) = security::append_audit_log(&app, "set_communication_node_enabled", "success")
    {
        eprintln!("[审计] set_communication_node_enabled success 日志写入失败: {err}");
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::{build_pairing_payload, endpoint_for_peer, CommunicationNodeEndpoint};
    use serde_json::Value;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn endpoint_supports_ipv4_and_ipv6() {
        let ipv4 = endpoint_for_peer(
            "12D3KooWTest",
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 8)),
            9944,
        )
        .expect("IPv4 endpoint should build");
        let ipv6 = endpoint_for_peer("12D3KooWTest", IpAddr::V6(Ipv6Addr::LOCALHOST), 9944)
            .expect("IPv6 endpoint should build");

        assert_eq!(ipv4.kind, "ip4");
        assert!(ipv4.multiaddr.starts_with("/ip4/192.168.1.8/"));
        assert_eq!(ipv6.kind, "ip6");
        assert!(ipv6.multiaddr.starts_with("/ip6/::1/"));
    }

    #[test]
    fn pairing_payload_matches_wumin_qr_envelope() {
        let endpoint = CommunicationNodeEndpoint {
            rpc_url: "http://192.168.1.8:9944/".to_string(),
            multiaddr: "/ip4/192.168.1.8/tcp/30333/wss/p2p/12D3KooWTest".to_string(),
            kind: "ip4".to_string(),
        };
        let (raw, expires) = build_pairing_payload("12D3KooWTest", &endpoint, 1_800_000, "nonce-1")
            .expect("payload should encode");
        let value: Value = serde_json::from_str(&raw).expect("payload is JSON");

        assert_eq!(expires, 2_400_000);
        assert_eq!(value["proto"], "WUMIN_QR_V1");
        assert_eq!(value["kind"], "im_node_pairing");
        assert_eq!(value["id"], "im-node-nonce-1");
        assert_eq!(value["issued_at"], 1800);
        assert_eq!(value["expires_at"], 2400);
        assert_eq!(value["body"]["proto"], "GMB_IM_NODE_PAIRING_V1");
        assert_eq!(value["body"]["node_peer_id"], "12D3KooWTest");
        assert_eq!(value["body"]["endpoint_kind"], "ip4");
    }
}
