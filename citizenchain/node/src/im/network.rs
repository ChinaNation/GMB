use std::{
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use sc_network::{
    config::{IncomingRequest, OutgoingResponse},
    request_responses::IfDisconnected,
    service::traits::{NetworkBackend, NetworkRequest, NetworkService},
    ProtocolName, ReputationChange,
};
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Block as BlockT;

use super::{
    direct::ImDirectDeliveryRequest,
    endpoint::ImNodeEndpoint,
    envelope::{ImEnvelopeAck, SubmitImEnvelopeRequest},
    keypackage::{
        ConsumeImKeyPackageRequest, FetchImKeyPackagesRequest, ImDirectKeyPackageConsumeRequest,
        ImDirectKeyPackageFetchRequest, ImKeyPackage, ImKeyPackagePool,
    },
    mailbox::ImMailbox,
};

/// IM request-response 协议名。
pub(crate) const IM_REQUEST_RESPONSE_PROTOCOL: &str = "/gmb/im/1";

const INBOUND_CHANNEL_SIZE: usize = 128;
const MAX_REQUEST_SIZE: u64 = 1024 * 1024;
const MAX_RESPONSE_SIZE: u64 = 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

static IM_NETWORK: OnceLock<Mutex<Option<Arc<dyn NetworkService>>>> = OnceLock::new();

fn registered_network() -> &'static Mutex<Option<Arc<dyn NetworkService>>> {
    IM_NETWORK.get_or_init(|| Mutex::new(None))
}

/// `/gmb/im/1` 请求。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "body")]
pub(crate) enum ImNetworkRequest {
    /// 向对方钱包账号 mailbox 投递密文信封。
    SubmitEnvelope(SubmitImEnvelopeRequest),
    /// 从对方私人节点按钱包地址拉取 OpenMLS KeyPackage。
    FetchKeyPackages(FetchImKeyPackagesRequest),
    /// 声明已消费对方钱包账号的一次性 KeyPackage。
    ConsumeKeyPackage(ConsumeImKeyPackageRequest),
}

/// `/gmb/im/1` 响应。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "body")]
pub(crate) enum ImNetworkResponse {
    /// 密文信封 ack。
    EnvelopeAck(ImEnvelopeAck),
    /// 可用 OpenMLS KeyPackage。
    KeyPackages(Vec<ImKeyPackage>),
    /// 已消费的 OpenMLS KeyPackage。
    KeyPackageConsumed(ImKeyPackage),
    /// 业务错误。
    Error(String),
}

impl ImNetworkRequest {
    /// 编码为 Spike 阶段 wire bytes。
    pub(crate) fn encode_wire(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("IM 网络请求编码失败: {e}"))
    }

    /// 从 Spike 阶段 wire bytes 解码。
    pub(crate) fn decode_wire(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("IM 网络请求解码失败: {e}"))
    }
}

impl ImNetworkResponse {
    /// 编码为 Spike 阶段 wire bytes。
    pub(crate) fn encode_wire(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("IM 网络响应编码失败: {e}"))
    }

    /// 从 Spike 阶段 wire bytes 解码。
    pub(crate) fn decode_wire(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("IM 网络响应解码失败: {e}"))
    }
}

/// 构造 IM request-response 协议配置。
pub(crate) fn request_response_config<B, N>() -> (
    N::RequestResponseProtocolConfig,
    async_channel::Receiver<IncomingRequest>,
)
where
    B: BlockT,
    N: NetworkBackend<B, <B as BlockT>::Hash>,
{
    let (inbound_tx, inbound_rx) = async_channel::bounded(INBOUND_CHANNEL_SIZE);
    let config = N::request_response_config(
        ProtocolName::from(IM_REQUEST_RESPONSE_PROTOCOL),
        Vec::new(),
        MAX_REQUEST_SIZE,
        MAX_RESPONSE_SIZE,
        REQUEST_TIMEOUT,
        Some(inbound_tx),
    );
    (config, inbound_rx)
}

/// 启动 IM incoming request handler。
pub(crate) fn spawn_incoming_handler(
    task_manager: &mut sc_service::TaskManager,
    inbound_rx: async_channel::Receiver<IncomingRequest>,
) {
    task_manager
        .spawn_handle()
        .spawn("im-request-response-handler", Some("im"), async move {
            while let Ok(request) = inbound_rx.recv().await {
                handle_incoming_request(request);
            }
            log::warn!("[IM] /gmb/im/1 incoming request channel 已关闭");
        });
}

/// 注册当前节点的网络服务句柄，供 Tauri 调试命令触发直连投递。
pub(crate) fn register_network_service(network: Arc<dyn NetworkService>) -> Result<(), String> {
    let mut slot = registered_network()
        .lock()
        .map_err(|_| "IM 网络句柄锁已损坏".to_string())?;
    *slot = Some(network);
    Ok(())
}

/// 使用已注册网络服务执行直连投递。
pub(crate) async fn send_registered_direct_delivery(
    delivery: ImDirectDeliveryRequest,
) -> Result<ImNetworkResponse, String> {
    let network = registered_network()
        .lock()
        .map_err(|_| "IM 网络句柄锁已损坏".to_string())?
        .clone()
        .ok_or_else(|| "IM 网络服务尚未启动".to_string())?;

    send_direct_delivery(network, delivery).await
}

/// 处理单个 incoming request。
pub(crate) fn handle_incoming_request(request: IncomingRequest) {
    let response = handle_payload(
        &request.payload,
        super::commands::mailbox(),
        super::commands::keypackage_pool(),
    );
    let response_bytes = response.encode_wire().map_err(|_| ());
    let _ = request.pending_response.send(OutgoingResponse {
        result: response_bytes,
        reputation_changes: Vec::<ReputationChange>::new(),
        sent_feedback: None,
    });
}

fn handle_payload(
    payload: &[u8],
    mailbox: &Mutex<ImMailbox>,
    keypackages: &Mutex<ImKeyPackagePool>,
) -> ImNetworkResponse {
    let request = match ImNetworkRequest::decode_wire(payload) {
        Ok(request) => request,
        Err(err) => return ImNetworkResponse::Error(err),
    };

    match request {
        ImNetworkRequest::SubmitEnvelope(submit) => {
            let mut mailbox = match mailbox.lock() {
                Ok(mailbox) => mailbox,
                Err(_) => return ImNetworkResponse::Error("IM mailbox 锁已损坏".to_string()),
            };
            match mailbox.submit_envelope(submit) {
                Ok(ack) => ImNetworkResponse::EnvelopeAck(ack),
                Err(err) => ImNetworkResponse::Error(err),
            }
        }
        ImNetworkRequest::FetchKeyPackages(fetch) => {
            let mut keypackages = match keypackages.lock() {
                Ok(keypackages) => keypackages,
                Err(_) => {
                    return ImNetworkResponse::Error("IM KeyPackage 池锁已损坏".to_string());
                }
            };
            match keypackages.fetch_available(fetch) {
                Ok(packages) => ImNetworkResponse::KeyPackages(packages),
                Err(err) => ImNetworkResponse::Error(err),
            }
        }
        ImNetworkRequest::ConsumeKeyPackage(consume) => {
            let mut keypackages = match keypackages.lock() {
                Ok(keypackages) => keypackages,
                Err(_) => {
                    return ImNetworkResponse::Error("IM KeyPackage 池锁已损坏".to_string());
                }
            };
            match keypackages.consume(consume) {
                Ok(package) => ImNetworkResponse::KeyPackageConsumed(package),
                Err(err) => ImNetworkResponse::Error(err),
            }
        }
    }
}

/// 使用 sc-network 向显式 PeerId + multiaddr 发送 IM 请求。
///
/// 本函数不做 DHT、rendezvous 或 relay；它只把 IM 路由记录里的显式 multiaddr 写入
/// sc-network 地址簿，然后向该 PeerId 发起 `/gmb/im/1` request。
pub(crate) async fn send_direct_delivery(
    network: Arc<dyn NetworkService>,
    delivery: ImDirectDeliveryRequest,
) -> Result<ImNetworkResponse, String> {
    delivery.validate()?;
    send_direct_request(
        network,
        &delivery.remote_endpoint,
        ImNetworkRequest::SubmitEnvelope(delivery.submit),
    )
    .await
}

/// 使用已注册网络服务直连拉取 KeyPackage。
pub(crate) async fn send_registered_direct_keypackage_fetch(
    request: ImDirectKeyPackageFetchRequest,
) -> Result<ImNetworkResponse, String> {
    let network = registered_network()
        .lock()
        .map_err(|_| "IM 网络句柄锁已损坏".to_string())?
        .clone()
        .ok_or_else(|| "IM 网络服务尚未启动".to_string())?;

    send_direct_keypackage_fetch(network, request).await
}

/// 使用已注册网络服务直连消费 KeyPackage。
pub(crate) async fn send_registered_direct_keypackage_consume(
    request: ImDirectKeyPackageConsumeRequest,
) -> Result<ImNetworkResponse, String> {
    let network = registered_network()
        .lock()
        .map_err(|_| "IM 网络句柄锁已损坏".to_string())?
        .clone()
        .ok_or_else(|| "IM 网络服务尚未启动".to_string())?;

    send_direct_keypackage_consume(network, request).await
}

/// 使用 sc-network 向显式 PeerId + multiaddr 拉取 KeyPackage。
pub(crate) async fn send_direct_keypackage_fetch(
    network: Arc<dyn NetworkService>,
    request: ImDirectKeyPackageFetchRequest,
) -> Result<ImNetworkResponse, String> {
    request.validate()?;
    send_direct_request(
        network,
        &request.remote_endpoint,
        ImNetworkRequest::FetchKeyPackages(request.fetch),
    )
    .await
}

/// 使用 sc-network 向显式 PeerId + multiaddr 消费 KeyPackage。
pub(crate) async fn send_direct_keypackage_consume(
    network: Arc<dyn NetworkService>,
    request: ImDirectKeyPackageConsumeRequest,
) -> Result<ImNetworkResponse, String> {
    request.validate()?;
    send_direct_request(
        network,
        &request.remote_endpoint,
        ImNetworkRequest::ConsumeKeyPackage(request.consume),
    )
    .await
}

async fn send_direct_request(
    network: Arc<dyn NetworkService>,
    remote_endpoint: &ImNodeEndpoint,
    request: ImNetworkRequest,
) -> Result<ImNetworkResponse, String> {
    remote_endpoint.validate()?;
    let (peer_id, multiaddr) = sc_network::config::parse_str_addr(&remote_endpoint.multiaddr)
        .map_err(|e| format!("IM 直连端点解析失败: {e}"))?;
    if peer_id.to_string() != remote_endpoint.peer_id {
        return Err("IM 直连端点 PeerId 与 multiaddr 不一致".to_string());
    }

    network.add_known_address(peer_id, multiaddr);

    let request = request.encode_wire()?;
    let (response, protocol) = network
        .request(
            peer_id,
            ProtocolName::from(IM_REQUEST_RESPONSE_PROTOCOL),
            request,
            None,
            IfDisconnected::TryConnect,
        )
        .await
        .map_err(|e| format!("IM 直连请求失败: {e:?}"))?;

    if protocol != ProtocolName::from(IM_REQUEST_RESPONSE_PROTOCOL) {
        return Err("IM 直连响应协议名不匹配".to_string());
    }
    ImNetworkResponse::decode_wire(&response)
}

/// 返回 outbound request API 是否已在当前编译目标中可调用。
pub(crate) fn outbound_request_api_available() -> bool {
    let _ = send_registered_direct_delivery;
    let _ = send_registered_direct_keypackage_fetch;
    let _ = send_registered_direct_keypackage_consume;
    true
}

#[cfg(test)]
mod tests {
    use super::{ImNetworkRequest, ImNetworkResponse};
    use crate::im::envelope::{ImEnvelope, SubmitImEnvelopeRequest, GMB_IM_PROTOCOL_VERSION};
    use crate::im::keypackage::FetchImKeyPackagesRequest;

    fn sample_request() -> ImNetworkRequest {
        ImNetworkRequest::SubmitEnvelope(SubmitImEnvelopeRequest {
            mailbox_owner_chat_account: "bob".to_string(),
            envelope: ImEnvelope {
                protocol_version: GMB_IM_PROTOCOL_VERSION,
                envelope_id: "env-1".to_string(),
                conversation_id: "conv-1".to_string(),
                sender_chat_account: "alice".to_string(),
                recipient_chat_account: "bob".to_string(),
                sender_device_id: "alice-phone".to_string(),
                encrypted_payload_hex: "aabbcc".to_string(),
                created_at_millis: 1,
                ttl_millis: 60_000,
            },
        })
    }

    fn sample_keypackage_request() -> ImNetworkRequest {
        ImNetworkRequest::FetchKeyPackages(FetchImKeyPackagesRequest {
            owner_wallet_account: "bob".to_string(),
            requester_chat_account: "alice".to_string(),
            limit: 1,
        })
    }

    #[test]
    fn request_wire_round_trip() {
        let request = sample_request();
        let encoded = request.encode_wire().expect("request should encode");
        let decoded = ImNetworkRequest::decode_wire(&encoded).expect("request should decode");
        assert_eq!(decoded, request);
    }

    #[test]
    fn keypackage_request_wire_round_trip() {
        let request = sample_keypackage_request();
        let encoded = request.encode_wire().expect("request should encode");
        let decoded = ImNetworkRequest::decode_wire(&encoded).expect("request should decode");
        assert_eq!(decoded, request);
    }

    #[test]
    fn response_wire_round_trip() {
        let response = ImNetworkResponse::Error("拒绝非目标钱包 mailbox".to_string());
        let encoded = response.encode_wire().expect("response should encode");
        let decoded = ImNetworkResponse::decode_wire(&encoded).expect("response should decode");
        assert_eq!(decoded, response);
    }
}
