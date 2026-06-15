use serde::{Deserialize, Serialize};

use super::{endpoint::ImNodeEndpoint, envelope::SubmitImEnvelopeRequest};

/// IM 直连投递请求。
///
/// 只允许使用 IM 路由记录或本机显式配置中的 PeerId + multiaddr，不走公共 DHT、
/// 公共 rendezvous 或第三方 relay。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImDirectDeliveryRequest {
    /// 对方私人通信全节点端点。
    pub(crate) remote_endpoint: ImNodeEndpoint,
    /// 投递到对方钱包账号 mailbox 的密文信封。
    pub(crate) submit: SubmitImEnvelopeRequest,
}

impl ImDirectDeliveryRequest {
    /// 校验直连请求不包含非目标钱包 mailbox 或非显式端点。
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.remote_endpoint.validate()?;
        self.submit.validate()?;
        Ok(())
    }
}

/// IM 直连投递能力状态。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImDirectNetworkCapability {
    /// 是否已在 sc-network 注册 `/gmb/im/1` request-response。
    pub(crate) request_response_registered: bool,
    /// 是否存在 incoming handler。
    pub(crate) incoming_handler_registered: bool,
    /// outbound 是否使用显式 PeerId。
    pub(crate) outbound_uses_explicit_peer_id: bool,
    /// outbound 是否把显式 multiaddr 写入 sc-network 地址簿。
    pub(crate) outbound_adds_explicit_multiaddr: bool,
    /// outbound 是否禁止公共发现。
    pub(crate) forbids_public_discovery: bool,
    /// outbound 是否禁止 relay。
    pub(crate) forbids_relay: bool,
    /// sc-network outbound request API 是否可用。
    pub(crate) outbound_request_api_available: bool,
    /// 当前 Spike 结论。
    pub(crate) conclusion: String,
}

impl ImDirectNetworkCapability {
    /// 当前实现能力说明。
    pub(crate) fn current() -> Self {
        Self {
            request_response_registered: true,
            incoming_handler_registered: true,
            outbound_uses_explicit_peer_id: true,
            outbound_adds_explicit_multiaddr: true,
            forbids_public_discovery: true,
            forbids_relay: true,
            outbound_request_api_available: super::network::outbound_request_api_available(),
            conclusion: "已接入 sc-network request-response 注册、incoming handler 和显式 multiaddr 入地址簿的 outbound helper；真实两节点直连需要下一步用两个 base-path 节点做运行态联调".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ImDirectDeliveryRequest;
    use crate::im::{
        endpoint::ImNodeEndpoint,
        envelope::{ImEnvelope, SubmitImEnvelopeRequest, GMB_IM_PROTOCOL_VERSION},
    };

    #[test]
    fn direct_request_rejects_third_party_mailbox() {
        let request = ImDirectDeliveryRequest {
            remote_endpoint: ImNodeEndpoint::checked(
                "12D3KooWRemote",
                "/ip4/127.0.0.1/tcp/30334/wss/p2p/12D3KooWRemote",
            )
            .expect("test endpoint should be valid"),
            submit: SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: "bob".to_string(),
                envelope: ImEnvelope {
                    protocol_version: GMB_IM_PROTOCOL_VERSION,
                    envelope_id: "env-1".to_string(),
                    conversation_id: "conv-1".to_string(),
                    sender_chat_account: "alice".to_string(),
                    recipient_chat_account: "carol".to_string(),
                    sender_device_id: "alice-phone".to_string(),
                    encrypted_payload_hex: "aabbcc".to_string(),
                    created_at_millis: 1,
                    ttl_millis: 60_000,
                },
            },
        };

        let err = request
            .validate()
            .expect_err("non-recipient mailbox must be rejected");
        assert!(err.contains("目标钱包账户"));
    }
}
