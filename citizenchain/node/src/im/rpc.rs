use jsonrpsee::{
    types::{error::ErrorObject, ErrorObjectOwned},
    RpcModule,
};
use std::sync::atomic::{AtomicBool, Ordering};

use super::{
    binding::RegisterImDeviceRequest,
    direct::{ImDirectDeliveryRequest, ImDirectNetworkCapability},
    keypackage::{
        ConsumeImKeyPackageRequest, FetchImKeyPackagesRequest, ImDirectKeyPackageConsumeRequest,
        ImDirectKeyPackageFetchRequest, PublishImKeyPackageRequest,
    },
};

/// IM debug RPC 开关环境变量。
///
/// 这些 RPC 只用于 headless 双节点运行态验收。正式节点默认不注册，避免把
/// mailbox 调试能力暴露成长期外部接口。
pub(crate) const IM_DEBUG_RPC_ENV: &str = "GMB_IM_DEBUG_RPC";
/// IM 本机手机 RPC 开关环境变量。
///
/// 公民手机连接自己的通信节点时使用 `im_*` RPC。该入口必须由用户
/// 明确启用，避免普通节点默认暴露 mailbox 管理面。
pub(crate) const IM_OWNER_RPC_ENV: &str = "GMB_IM_OWNER_RPC";

const IM_DEBUG_RPC_ERROR: i32 = -32_070;
static IM_OWNER_RPC_RUNTIME_ENABLED: AtomicBool = AtomicBool::new(false);

/// 判断当前进程是否允许注册 IM debug RPC。
pub(crate) fn debug_rpc_enabled() -> bool {
    std::env::var(IM_DEBUG_RPC_ENV).is_ok()
}

/// 判断当前进程是否允许注册 IM 本机手机 RPC。
pub(crate) fn owner_rpc_enabled() -> bool {
    true
}

/// 根据桌面设置页通信节点功能开关更新本机手机 RPC 运行态。
pub(crate) fn set_owner_rpc_runtime_enabled(enabled: bool) {
    IM_OWNER_RPC_RUNTIME_ENABLED.store(enabled, Ordering::Relaxed);
}

fn owner_rpc_runtime_enabled() -> bool {
    std::env::var(IM_OWNER_RPC_ENV).is_ok()
        || debug_rpc_enabled()
        || IM_OWNER_RPC_RUNTIME_ENABLED.load(Ordering::Relaxed)
}

fn ensure_owner_rpc_runtime_enabled() -> Result<(), ErrorObjectOwned> {
    if owner_rpc_runtime_enabled() {
        return Ok(());
    }
    Err(rpc_error(
        "通信节点功能未启用，请先在桌面设置页开启通信节点功能".to_string(),
    ))
}

/// 注册 IM 本机手机 RPC。
pub(crate) fn register_owner_rpc(
    module: &mut RpcModule<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    module.register_method("im_getCapability", move |_params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        Ok::<ImDirectNetworkCapability, ErrorObjectOwned>(ImDirectNetworkCapability::current())
    })?;

    module.register_method("im_registerOwnerDevice", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: RegisterImDeviceRequest = params.one()?;
        super::commands::register_im_owner_device(request).map_err(rpc_error)
    })?;

    module.register_method("im_submitEnvelope", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: super::envelope::SubmitImEnvelopeRequest = params.one()?;
        super::commands::submit_im_encrypted_envelope(request).map_err(rpc_error)
    })?;

    module.register_method("im_fetchPending", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let (owner_wallet_account, device_id): (String, String) = params.parse()?;
        super::commands::fetch_im_pending_envelopes(owner_wallet_account, device_id)
            .map_err(rpc_error)
    })?;

    module.register_method("im_ackEnvelope", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let (owner_wallet_account, device_id, envelope_id): (String, String, String) =
            params.parse()?;
        super::commands::ack_im_envelope(owner_wallet_account, device_id, envelope_id)
            .map_err(rpc_error)
    })?;

    module.register_method("im_submitDirectEnvelope", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: ImDirectDeliveryRequest = params.one()?;
        futures::executor::block_on(super::network::send_registered_direct_delivery(request))
            .map_err(rpc_error)
    })?;

    module.register_method("im_publishKeyPackage", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: PublishImKeyPackageRequest = params.one()?;
        super::commands::publish_im_key_package(request).map_err(rpc_error)
    })?;

    module.register_method("im_fetchKeyPackages", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: FetchImKeyPackagesRequest = params.one()?;
        super::commands::fetch_im_key_packages(request).map_err(rpc_error)
    })?;

    module.register_method("im_consumeKeyPackage", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: ConsumeImKeyPackageRequest = params.one()?;
        super::commands::consume_im_key_package(request).map_err(rpc_error)
    })?;

    module.register_method("im_fetchDirectKeyPackages", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: ImDirectKeyPackageFetchRequest = params.one()?;
        futures::executor::block_on(super::network::send_registered_direct_keypackage_fetch(
            request,
        ))
        .map_err(rpc_error)
    })?;

    module.register_method("im_consumeDirectKeyPackage", move |params, _, _| {
        ensure_owner_rpc_runtime_enabled()?;
        let request: ImDirectKeyPackageConsumeRequest = params.one()?;
        futures::executor::block_on(super::network::send_registered_direct_keypackage_consume(
            request,
        ))
        .map_err(rpc_error)
    })?;

    Ok(())
}

/// 注册 IM debug RPC。
pub(crate) fn register_debug_rpc(
    module: &mut RpcModule<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    module.register_method("im_debugGetCapability", move |_params, _, _| {
        Ok::<ImDirectNetworkCapability, ErrorObjectOwned>(ImDirectNetworkCapability::current())
    })?;

    module.register_method("im_debugRegisterOwnerDevice", move |params, _, _| {
        let request: RegisterImDeviceRequest = params.one()?;
        super::commands::register_im_owner_device(request).map_err(rpc_error)
    })?;

    module.register_method("im_debugFetchPending", move |params, _, _| {
        let (owner_wallet_account, device_id): (String, String) = params.parse()?;
        super::commands::fetch_im_pending_envelopes(owner_wallet_account, device_id)
            .map_err(rpc_error)
    })?;

    module.register_method("im_debugAckEnvelope", move |params, _, _| {
        let (owner_wallet_account, device_id, envelope_id): (String, String, String) =
            params.parse()?;
        super::commands::ack_im_envelope(owner_wallet_account, device_id, envelope_id)
            .map_err(rpc_error)
    })?;

    module.register_method("im_debugSubmitDirectEnvelope", move |params, _, _| {
        let request: ImDirectDeliveryRequest = params.one()?;
        futures::executor::block_on(super::network::send_registered_direct_delivery(request))
            .map_err(rpc_error)
    })?;

    module.register_method("im_debugPublishKeyPackage", move |params, _, _| {
        let request: PublishImKeyPackageRequest = params.one()?;
        super::commands::publish_im_key_package(request).map_err(rpc_error)
    })?;

    module.register_method("im_debugFetchKeyPackages", move |params, _, _| {
        let request: FetchImKeyPackagesRequest = params.one()?;
        super::commands::fetch_im_key_packages(request).map_err(rpc_error)
    })?;

    module.register_method("im_debugConsumeKeyPackage", move |params, _, _| {
        let request: ConsumeImKeyPackageRequest = params.one()?;
        super::commands::consume_im_key_package(request).map_err(rpc_error)
    })?;

    module.register_method("im_debugFetchDirectKeyPackages", move |params, _, _| {
        let request: ImDirectKeyPackageFetchRequest = params.one()?;
        futures::executor::block_on(super::network::send_registered_direct_keypackage_fetch(
            request,
        ))
        .map_err(rpc_error)
    })?;

    module.register_method("im_debugConsumeDirectKeyPackage", move |params, _, _| {
        let request: ImDirectKeyPackageConsumeRequest = params.one()?;
        futures::executor::block_on(super::network::send_registered_direct_keypackage_consume(
            request,
        ))
        .map_err(rpc_error)
    })?;

    Ok(())
}

fn rpc_error(message: String) -> ErrorObjectOwned {
    ErrorObject::owned(IM_DEBUG_RPC_ERROR, message, None::<()>)
}

#[cfg(test)]
mod tests {
    use super::{register_debug_rpc, register_owner_rpc};
    use jsonrpsee::RpcModule;

    #[test]
    fn registers_im_debug_rpc_methods() {
        let mut module = RpcModule::new(());
        register_debug_rpc(&mut module).expect("debug rpc should register");

        let methods = module.method_names().collect::<Vec<_>>();
        assert!(methods.contains(&"im_debugGetCapability"));
        assert!(methods.contains(&"im_debugRegisterOwnerDevice"));
        assert!(methods.contains(&"im_debugSubmitDirectEnvelope"));
        assert!(methods.contains(&"im_debugFetchPending"));
        assert!(methods.contains(&"im_debugAckEnvelope"));
        assert!(methods.contains(&"im_debugPublishKeyPackage"));
        assert!(methods.contains(&"im_debugFetchKeyPackages"));
        assert!(methods.contains(&"im_debugConsumeKeyPackage"));
        assert!(methods.contains(&"im_debugFetchDirectKeyPackages"));
        assert!(methods.contains(&"im_debugConsumeDirectKeyPackage"));
    }

    #[test]
    fn registers_im_owner_rpc_methods() {
        let mut module = RpcModule::new(());
        register_owner_rpc(&mut module).expect("local phone rpc should register");

        let methods = module.method_names().collect::<Vec<_>>();
        assert!(methods.contains(&"im_getCapability"));
        assert!(methods.contains(&"im_registerOwnerDevice"));
        assert!(methods.contains(&"im_submitEnvelope"));
        assert!(methods.contains(&"im_submitDirectEnvelope"));
        assert!(methods.contains(&"im_fetchPending"));
        assert!(methods.contains(&"im_ackEnvelope"));
        assert!(methods.contains(&"im_publishKeyPackage"));
        assert!(methods.contains(&"im_fetchKeyPackages"));
        assert!(methods.contains(&"im_consumeKeyPackage"));
        assert!(methods.contains(&"im_fetchDirectKeyPackages"));
        assert!(methods.contains(&"im_consumeDirectKeyPackage"));
    }
}
