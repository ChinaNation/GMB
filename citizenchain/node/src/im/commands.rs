use std::{
    path::Path,
    sync::{Mutex, OnceLock},
};

use super::{
    binding::{ImDeviceBinding, RegisterImDeviceRequest},
    endpoint::ImNodeEndpoint,
    envelope::{ImEnvelope, ImEnvelopeAck, SubmitImEnvelopeRequest},
    keypackage::{
        ConsumeImKeyPackageRequest, FetchImKeyPackagesRequest, ImDirectKeyPackageConsumeRequest,
        ImDirectKeyPackageFetchRequest, ImKeyPackage, ImKeyPackagePool, PublishImKeyPackageRequest,
    },
    mailbox::ImMailbox,
    policy::ImPrivateNodePolicy,
};

static IM_MAILBOX: OnceLock<Mutex<ImMailbox>> = OnceLock::new();
static IM_KEYPACKAGE_POOL: OnceLock<Mutex<ImKeyPackagePool>> = OnceLock::new();

pub(crate) fn mailbox() -> &'static Mutex<ImMailbox> {
    IM_MAILBOX.get_or_init(|| Mutex::new(ImMailbox::default()))
}

pub(crate) fn keypackage_pool() -> &'static Mutex<ImKeyPackagePool> {
    IM_KEYPACKAGE_POOL.get_or_init(|| Mutex::new(ImKeyPackagePool::default()))
}

/// 初始化 IM 持久化路径。
pub(crate) fn init_mailbox_storage(base_path: &Path) -> Result<(), String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .attach_storage(base_path.join("im").join("mailbox.json"))?;
    keypackage_pool()
        .lock()
        .map_err(|_| "IM KeyPackage 池锁已损坏".to_string())?
        .attach_storage(base_path.join("im").join("keypackages.json"))
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

/// 通过已启动的 sc-network 向对方私人通信全节点直连拉取 KeyPackage。
#[tauri::command]
pub(crate) async fn fetch_im_direct_key_packages(
    request: ImDirectKeyPackageFetchRequest,
) -> Result<super::network::ImNetworkResponse, String> {
    super::network::send_registered_direct_keypackage_fetch(request).await
}

/// 通过已启动的 sc-network 向对方私人通信全节点声明消费 KeyPackage。
#[tauri::command]
pub(crate) async fn consume_im_direct_key_package(
    request: ImDirectKeyPackageConsumeRequest,
) -> Result<super::network::ImNetworkResponse, String> {
    super::network::send_registered_direct_keypackage_consume(request).await
}

/// 校验 IM 私人节点端点。
#[tauri::command]
pub(crate) fn validate_im_node_endpoint(
    endpoint: ImNodeEndpoint,
) -> Result<ImNodeEndpoint, String> {
    ImNodeEndpoint::checked(endpoint.peer_id, endpoint.multiaddr)
}

/// 登记已授权手机设备到本机私人通信全节点。
#[tauri::command]
pub(crate) fn register_im_owner_device(
    request: RegisterImDeviceRequest,
) -> Result<ImDeviceBinding, String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .register_owner_device(request)
}

/// 提交密文信封到本机钱包账号 mailbox。
#[tauri::command]
pub(crate) fn submit_im_encrypted_envelope(
    request: SubmitImEnvelopeRequest,
) -> Result<ImEnvelopeAck, String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .submit_envelope(request)
}

/// 已授权手机拉取本机 mailbox 中待收密文。
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

/// 已授权手机确认已处理密文信封。
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

/// 已授权手机向自己的私人通信全节点发布 OpenMLS KeyPackage。
#[tauri::command]
pub(crate) fn publish_im_key_package(
    request: PublishImKeyPackageRequest,
) -> Result<ImKeyPackage, String> {
    mailbox()
        .lock()
        .map_err(|_| "IM mailbox 锁已损坏".to_string())?
        .ensure_owner_account(&request.owner_wallet_account)?;
    keypackage_pool()
        .lock()
        .map_err(|_| "IM KeyPackage 池锁已损坏".to_string())?
        .publish(request)
}

/// 查询本机 KeyPackage 池，供本机调试和验收使用。
#[tauri::command]
pub(crate) fn fetch_im_key_packages(
    request: FetchImKeyPackagesRequest,
) -> Result<Vec<ImKeyPackage>, String> {
    keypackage_pool()
        .lock()
        .map_err(|_| "IM KeyPackage 池锁已损坏".to_string())?
        .fetch_available(request)
}

/// 消费本机 KeyPackage，供本机调试和验收使用。
#[tauri::command]
pub(crate) fn consume_im_key_package(
    request: ConsumeImKeyPackageRequest,
) -> Result<ImKeyPackage, String> {
    keypackage_pool()
        .lock()
        .map_err(|_| "IM KeyPackage 池锁已损坏".to_string())?
        .consume(request)
}
