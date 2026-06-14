use std::sync::{Mutex, OnceLock};

use super::{
    binding::{ImDeviceBinding, RegisterImDeviceRequest},
    endpoint::ImNodeEndpoint,
    envelope::{ImEnvelope, ImEnvelopeAck, SubmitImEnvelopeRequest},
    mailbox::ImMailbox,
    policy::ImPrivateNodePolicy,
};

static IM_MAILBOX: OnceLock<Mutex<ImMailbox>> = OnceLock::new();

pub(crate) fn mailbox() -> &'static Mutex<ImMailbox> {
    IM_MAILBOX.get_or_init(|| Mutex::new(ImMailbox::default()))
}

/// 查询 IM 私人通信全节点边界。
///
/// 该命令用于桌面设置页和后续调试页展示当前 IM 模式的硬约束。真实 mailbox、
/// KeyPackage 池和 sc-network 协议接入会在网络 Spike 后继续落地。
#[tauri::command]
pub(crate) fn get_im_private_node_policy() -> ImPrivateNodePolicy {
    ImPrivateNodePolicy::current()
}

/// 查询 IM 直连网络 Spike 能力。
#[tauri::command]
pub(crate) fn get_im_direct_network_capability() -> super::direct::ImDirectNetworkCapability {
    super::direct::ImDirectNetworkCapability::current()
}

/// 校验 IM 直连投递请求边界。
#[tauri::command]
pub(crate) fn validate_im_direct_delivery_request(
    request: super::direct::ImDirectDeliveryRequest,
) -> Result<super::direct::ImDirectDeliveryRequest, String> {
    request.validate()?;
    Ok(request)
}

/// 通过已启动的 sc-network 向对方私人通信全节点直连投递密文信封。
#[tauri::command]
pub(crate) async fn submit_im_direct_encrypted_envelope(
    request: super::direct::ImDirectDeliveryRequest,
) -> Result<super::network::ImNetworkResponse, String> {
    super::network::send_registered_direct_delivery(request).await
}

/// 校验 IM 私人节点端点。
#[tauri::command]
pub(crate) fn validate_im_node_endpoint(
    endpoint: ImNodeEndpoint,
) -> Result<ImNodeEndpoint, String> {
    ImNodeEndpoint::checked(endpoint.peer_id, endpoint.multiaddr)
}

/// 登记 owner 手机设备到本机私人通信全节点。
#[tauri::command]
pub(crate) fn register_im_owner_device(
    request: RegisterImDeviceRequest,
) -> Result<ImDeviceBinding, String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .register_owner_device(request)
}

/// 提交密文信封到本机 owner mailbox。
#[tauri::command]
pub(crate) fn submit_im_encrypted_envelope(
    request: SubmitImEnvelopeRequest,
) -> Result<ImEnvelopeAck, String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .submit_envelope(request)
}

/// owner 手机拉取本机 mailbox 中待收密文。
#[tauri::command]
pub(crate) fn fetch_im_pending_envelopes(
    owner_wallet_account: String,
    device_id: String,
) -> Result<Vec<ImEnvelope>, String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .fetch_pending(&owner_wallet_account, &device_id)
}

/// owner 手机确认已处理密文信封。
#[tauri::command]
pub(crate) fn ack_im_envelope(
    owner_wallet_account: String,
    device_id: String,
    envelope_id: String,
) -> Result<ImEnvelopeAck, String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .ack_envelope(&owner_wallet_account, &device_id, &envelope_id)
}
