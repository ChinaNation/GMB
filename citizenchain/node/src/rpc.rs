//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use citizenchain::{opaque::Block, AccountId, Balance, Nonce};
use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// CPU 哈希率查询函数（hashes/sec）。
    pub cpu_hashrate_fn: fn() -> f64,
    /// GPU 哈希率查询函数（仅在 gpu-mining feature 启用且有 GPU 时为 Some）。
    pub gpu_hashrate_fn: Option<fn() -> f64>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps {
        client,
        pool,
        cpu_hashrate_fn,
        gpu_hashrate_fn,
    } = deps;

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client).into_rpc())?;

    // CPU 哈希率 RPC：mining_cpuHashrate
    // 返回值：当前 CPU 全线程合计哈希率（hashes/sec），u64 整数。
    module.register_method("mining_cpuHashrate", move |_, _, _| {
        cpu_hashrate_fn() as u64
    })?;

    // GPU 哈希率 RPC：mining_gpuHashrate
    // 返回值：当前 GPU 哈希率（hashes/sec），u64 整数。
    if let Some(get_hashrate) = gpu_hashrate_fn {
        module.register_method("mining_gpuHashrate", move |_, _, _| {
            get_hashrate() as u64
        })?;
    }

    Ok(module)
}
