use jsonrpsee::{
    types::{error::ErrorObject, ErrorObjectOwned},
    RpcModule,
};

use super::{
    binding::RegisterImDeviceRequest,
    direct::{ImDirectDeliveryRequest, ImDirectNetworkCapability},
};

/// IM debug RPC 开关环境变量。
///
/// 这些 RPC 只用于 headless 双节点运行态验收。正式节点默认不注册，避免把 owner
/// mailbox 调试能力暴露成长期外部接口。
pub(crate) const IM_DEBUG_RPC_ENV: &str = "GMB_IM_DEBUG_RPC";

const IM_DEBUG_RPC_ERROR: i32 = -32_070;

/// 判断当前进程是否允许注册 IM debug RPC。
pub(crate) fn debug_rpc_enabled() -> bool {
    std::env::var(IM_DEBUG_RPC_ENV).is_ok()
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

    Ok(())
}

fn rpc_error(message: String) -> ErrorObjectOwned {
    ErrorObject::owned(IM_DEBUG_RPC_ERROR, message, None::<()>)
}

#[cfg(test)]
mod tests {
    use super::register_debug_rpc;
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
    }
}
