use serde::{Deserialize, Serialize};

/// IM 私人通信全节点对外公布的可达端点。
///
/// 该结构只描述用户自己通信节点的入口，不表示公共 relay、公共 DHT 或公共
/// rendezvous。端点可以由用户手工配置，也可以由后续 sc-network Spike 从本机
/// 网络服务读取。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImNodeEndpoint {
    /// 节点 PeerId，必须和 multiaddr 末尾 `/p2p/<peer_id>` 一致。
    pub(crate) peer_id: String,
    /// libp2p multiaddr，支持 ip4、ip6、dns4、dnsaddr 四类入口。
    pub(crate) multiaddr: String,
    /// 端点类型：ip4 / ip6 / dns4 / dnsaddr。
    pub(crate) kind: String,
}

impl ImNodeEndpoint {
    /// 构造并校验 IM 节点端点。
    pub(crate) fn checked(
        peer_id: impl Into<String>,
        multiaddr: impl Into<String>,
    ) -> Result<Self, String> {
        let peer_id = peer_id.into();
        let multiaddr = multiaddr.into();
        let kind = endpoint_kind(&multiaddr)?;
        let endpoint = Self {
            peer_id,
            multiaddr,
            kind: kind.to_string(),
        };
        endpoint.validate()?;
        Ok(endpoint)
    }

    /// 校验端点只属于允许的私人节点入口类型。
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.peer_id.trim().is_empty() {
            return Err("IM 节点 PeerId 不能为空".to_string());
        }
        if self.multiaddr.trim().is_empty() {
            return Err("IM 节点 multiaddr 不能为空".to_string());
        }
        let kind = endpoint_kind(&self.multiaddr)?;
        if self.kind != kind {
            return Err(format!("IM 节点端点类型应为 {kind}"));
        }
        let expected_peer_suffix = format!("/p2p/{}", self.peer_id);
        if !self.multiaddr.ends_with(&expected_peer_suffix) {
            return Err("IM 节点 multiaddr 必须以 /p2p/<peer_id> 结束".to_string());
        }
        Ok(())
    }
}

fn endpoint_kind(multiaddr: &str) -> Result<&'static str, String> {
    if multiaddr.starts_with("/ip4/") {
        Ok("ip4")
    } else if multiaddr.starts_with("/ip6/") {
        Ok("ip6")
    } else if multiaddr.starts_with("/dns4/") {
        Ok("dns4")
    } else if multiaddr.starts_with("/dnsaddr/") {
        Ok("dnsaddr")
    } else {
        Err("IM 节点端点只允许 ip4、ip6、dns4 或 dnsaddr".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ImNodeEndpoint;

    #[test]
    fn accepts_ipv6_endpoint() {
        let endpoint = ImNodeEndpoint::checked(
            "12D3KooWTest",
            "/ip6/2001:db8::1/tcp/443/wss/p2p/12D3KooWTest",
        )
        .expect("IPv6 endpoint should be accepted");

        assert_eq!(endpoint.kind, "ip6");
    }

    #[test]
    fn accepts_user_owned_dnsaddr_endpoint() {
        let endpoint =
            ImNodeEndpoint::checked("12D3KooWTest", "/dnsaddr/im.example.org/p2p/12D3KooWTest")
                .expect("dnsaddr endpoint should be accepted");

        assert_eq!(endpoint.kind, "dnsaddr");
    }

    #[test]
    fn rejects_endpoint_without_peer_suffix() {
        let err = ImNodeEndpoint::checked("12D3KooWTest", "/ip4/127.0.0.1/tcp/30333")
            .expect_err("missing peer suffix must be rejected");

        assert!(err.contains("/p2p/<peer_id>"));
    }
}
