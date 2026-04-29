//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use citizenchain::{self as runtime, opaque::Block, AccountId, Balance, Nonce};
use codec::{Decode, Encode};
use jsonrpsee::RpcModule;
use sc_client_api::StorageProvider;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::Core as CoreApi;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::crypto::KeyTypeId;
use sp_keystore::Keystore;
use sp_runtime::{
    generic::Era, traits::IdentifyAccount, MultiSigner, OpaqueExtrinsic, SaturatedConversion,
};
use substrate_frame_rpc_system::AccountNonceApi;

/// PoW 矿工密钥类型（与 service.rs 中 POW_AUTHOR_KEY_TYPE 一致）。
const POW_AUTHOR_KEY_TYPE: KeyTypeId = KeyTypeId(*b"powr");

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Keystore（用于签名奖励钱包绑定交易）。
    pub keystore: sp_keystore::KeystorePtr,
    /// CPU 哈希率查询函数（hashes/sec）。
    pub cpu_hashrate_fn: fn() -> f64,
    /// GPU 哈希率查询函数（仅在 gpu-mining feature 启用且有 GPU 时为 Some）。
    pub gpu_hashrate_fn: Option<fn() -> f64>,
    /// Chain spec（用于 sync_state_genSyncSpec RPC，供轻节点获取 lightSyncState）。
    pub chain_spec: Box<dyn sc_chain_spec::ChainSpec + Send>,
    /// 扫码支付 Step 2b-ii-β-2-b 新增:清算行节点的 RPC 命名空间实现。
    /// None 表示本节点未以清算行角色启动,跳过 `offchain_*` RPC 注入。
    pub offchain_clearing_rpc: Option<Arc<crate::offchain::rpc::OffchainClearingRpcImpl>>,
}

/// 构造并签名一笔交易，提交到交易池。
fn submit_reward_wallet_tx<C, P>(
    client: &Arc<C>,
    pool: &Arc<P>,
    keystore: &sp_keystore::KeystorePtr,
    call: runtime::RuntimeCall,
) -> Result<(), jsonrpsee::types::ErrorObjectOwned>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + 'static,
    C::Api: AccountNonceApi<Block, AccountId, Nonce> + CoreApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
{
    use jsonrpsee::types::error::ErrorObject;

    // 1. 从 keystore 取 powr 公钥
    let keys = keystore.sr25519_public_keys(POW_AUTHOR_KEY_TYPE);
    let public = keys
        .first()
        .copied()
        .ok_or_else(|| ErrorObject::owned(-1, "未找到矿工密钥，请先启动节点", None::<()>))?;

    // 2. 推导 AccountId
    let miner_account: AccountId =
        MultiSigner::from(sp_core::sr25519::Public::from(public)).into_account();

    // 3. 查询链信息
    let info = (*client).info();
    let best_hash = info.best_hash;
    let best_number = info.best_number;
    let genesis_hash = client
        .hash(0)
        .map_err(|e| ErrorObject::owned(-1, format!("查询创世块哈希失败: {e}"), None::<()>))?
        .ok_or_else(|| ErrorObject::owned(-1, "创世块不存在", None::<()>))?;

    // 4. 查询 nonce
    let nonce = client
        .runtime_api()
        .account_nonce(best_hash, miner_account.clone())
        .map_err(|e| ErrorObject::owned(-1, format!("查询账户 nonce 失败: {e}"), None::<()>))?;

    // 4b. 查询链上 WASM 运行时的版本号（不使用 native 编译时常量，
    //     避免 spec_version 升级后 native 与链上 WASM 不一致导致 BadProof）
    let on_chain_version = client
        .runtime_api()
        .version(best_hash)
        .map_err(|e| ErrorObject::owned(-1, format!("查询运行时版本失败: {e}"), None::<()>))?;

    // 5. 构造 TxExtension（与 benchmarking.rs 完全一致）
    let period = runtime::configs::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;
    let tx_ext: runtime::TxExtension = (
        frame_system::AuthorizeCall::<runtime::Runtime>::new(),
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        runtime::CheckNonStakeSender,
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(Era::mortal(
            period,
            best_number.saturated_into(),
        )),
        frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
        frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
        frame_system::WeightReclaim::<runtime::Runtime>::new(),
    );

    // 6. 签名
    let raw_payload = runtime::SignedPayload::from_raw(
        call.clone(),
        tx_ext.clone(),
        (
            (),
            (),
            (),
            on_chain_version.spec_version,
            on_chain_version.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
            None,
            (),
        ),
    );
    let signature = raw_payload
        .using_encoded(|payload| keystore.sr25519_sign(POW_AUTHOR_KEY_TYPE, &public, payload));
    let signature = signature
        .map_err(|e| ErrorObject::owned(-1, format!("keystore 签名失败: {e}"), None::<()>))?
        .ok_or_else(|| ErrorObject::owned(-1, "keystore 未返回签名", None::<()>))?;

    // 7. 构造 UncheckedExtrinsic
    let extrinsic = runtime::UncheckedExtrinsic::new_signed(
        call,
        sp_runtime::MultiAddress::Id(miner_account),
        runtime::Signature::Sr25519(signature),
        tx_ext,
    );

    // 8. 编码并提交到交易池
    let encoded = extrinsic.encode();
    let opaque = OpaqueExtrinsic::try_from_encoded_extrinsic(&encoded)
        .map_err(|_| ErrorObject::owned(-1, "交易编码失败", None::<()>))?;

    // submit_one 是 async，但我们在同步上下文中，使用 futures::executor::block_on
    futures::executor::block_on(pool.submit_one(best_hash, TransactionSource::Local, opaque))
        .map_err(|e| ErrorObject::owned(-1, format!("提交交易到交易池失败: {e}"), None::<()>))?;

    Ok(())
}

/// 从 SS58 地址解析 AccountId。
fn parse_ss58_account(address: &str) -> Result<AccountId, jsonrpsee::types::ErrorObjectOwned> {
    use sp_core::crypto::Ss58Codec;
    sp_runtime::AccountId32::from_ss58check(address).map_err(|e| {
        jsonrpsee::types::error::ErrorObject::owned(
            -1,
            format!("SS58 地址解析失败: {e:?}"),
            None::<()>,
        )
    })
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: StorageProvider<Block, sc_service::TFullBackend<Block>> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    C::Api: CoreApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps {
        client,
        pool,
        keystore,
        cpu_hashrate_fn,
        gpu_hashrate_fn,
        chain_spec,
        offchain_clearing_rpc,
    } = deps;

    // 扫码支付 Step 2b-ii-β-2-b:若清算行组件已启动,合并 offchain_* RPC 命名空间。
    if let Some(impl_) = offchain_clearing_rpc {
        use crate::offchain::rpc::OffchainClearingRpcServer;
        module
            .merge(OffchainClearingRpcServer::into_rpc((*impl_).clone()))
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::other(format!(
                    "合并 offchain 清算 RPC 失败:{e:?}"
                )))
            })?;
    }

    module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    // sync_state_genSyncSpec: 返回包含 lightSyncState 的 chainspec 快照，
    // 供 smoldot 轻节点跳过历史区块头验证。
    // 标准 sc-sync-state-rpc 依赖 BABE，citizenchain 用 PoW 没有 BABE，
    // 因此用自定义实现：从 chain_spec + 当前 finalized header + GRANDPA authority set 构造。
    {
        let client = client.clone();
        let chain_spec_for_rpc = chain_spec;
        module.register_method("sync_state_genSyncSpec", move |_params, _, _| {
            use jsonrpsee::types::error::ErrorObject;

            // 1. 解析原始 chain_spec JSON
            let spec_json_str = chain_spec_for_rpc.as_json(true).map_err(|e| {
                ErrorObject::owned(-1, format!("chain_spec 序列化失败: {e}"), None::<()>)
            })?;
            let mut spec: serde_json::Value =
                serde_json::from_str(&spec_json_str).map_err(|e| {
                    ErrorObject::owned(-1, format!("chain_spec JSON 解析失败: {e}"), None::<()>)
                })?;

            // 2. 获取 finalized block header
            let finalized_hash = client.info().finalized_hash;
            let finalized_header = client
                .header(finalized_hash)
                .map_err(|e| {
                    ErrorObject::owned(-1, format!("获取 finalized header 失败: {e}"), None::<()>)
                })?
                .ok_or_else(|| ErrorObject::owned(-1, "finalized header 不存在", None::<()>))?;
            let finalized_header_hex = format!("0x{}", hex::encode(finalized_header.encode()));

            // 3. 读取 GRANDPA authority set（从 storage 中读取 Grandpa::CurrentSetId 和 Grandpa::Authorities）
            let grandpa_set_id_key = {
                let mut k = Vec::new();
                k.extend_from_slice(&sp_io::hashing::twox_128(b"Grandpa"));
                k.extend_from_slice(&sp_io::hashing::twox_128(b"CurrentSetId"));
                k
            };
            let set_id_bytes = client
                .storage(finalized_hash, &sp_storage::StorageKey(grandpa_set_id_key))
                .map_err(|e| {
                    ErrorObject::owned(-1, format!("读取 GRANDPA set_id 失败: {e}"), None::<()>)
                })?;
            let set_id: u64 = set_id_bytes
                .map(|d| u64::decode(&mut &d.0[..]).unwrap_or(0))
                .unwrap_or(0);

            let grandpa_authorities_key = {
                let mut k = Vec::new();
                k.extend_from_slice(&sp_io::hashing::twox_128(b"Grandpa"));
                k.extend_from_slice(&sp_io::hashing::twox_128(b"Authorities"));
                k
            };
            let auth_bytes = client
                .storage(
                    finalized_hash,
                    &sp_storage::StorageKey(grandpa_authorities_key),
                )
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!("读取 GRANDPA authorities 失败: {e}"),
                        None::<()>,
                    )
                })?;

            // 中文注释：将 GRANDPA AuthoritySet 编码为 smoldot 要求的完整格式。
            // smoldot authority_set 解析器期望：
            //   Vec<(AuthorityId, u64)>    ← authorities（从 Grandpa::Authorities 存储读取）
            //   u64                        ← set_id
            //   ForkTree<PendingChange>    ← pending_standard_changes（空 = 0x00 0x00）
            //   Vec<PendingChange>         ← pending_forced_changes（空 = 0x00）
            //   Vec<(u64, u32)>            ← authority_set_changes（空 = 0x00）
            let authority_set_hex = {
                let auth_raw = auth_bytes.map(|d| d.0).unwrap_or_default();
                let set_id_encoded = set_id.encode();
                let mut combined = Vec::with_capacity(auth_raw.len() + set_id_encoded.len() + 4);
                combined.extend_from_slice(&auth_raw); // Vec<(AuthorityId, u64)>
                combined.extend_from_slice(&set_id_encoded); // u64 set_id
                combined.push(0x00u8); // ForkTree roots: Compact<0>
                combined.push(0x00u8); // ForkTree best_finalized_number: Option::None
                combined.push(0x00u8); // Vec<PendingChange>: Compact<0>
                combined.push(0x00u8); // Vec<(u64, u32)>: Compact<0>
                format!("0x{}", hex::encode(&combined))
            };

            // 4. 构造 lightSyncState
            let light_sync_state = serde_json::json!({
                "finalizedBlockHeader": finalized_header_hex,
                "grandpaAuthoritySet": authority_set_hex,
            });
            spec["lightSyncState"] = light_sync_state;

            Ok::<serde_json::Value, jsonrpsee::types::ErrorObjectOwned>(spec)
        })?;
    }

    // CPU 哈希率 RPC：mining_cpuHashrate
    // 返回值：当前 CPU 全线程合计哈希率（hashes/sec），u64 整数。
    module.register_method("mining_cpuHashrate", move |_, _, _| {
        cpu_hashrate_fn() as u64
    })?;

    // GPU 哈希率 RPC：mining_gpuHashrate
    // 返回值：当前 GPU 哈希率（hashes/sec），u64 整数。
    if let Some(get_hashrate) = gpu_hashrate_fn {
        module.register_method("mining_gpuHashrate", move |_, _, _| get_hashrate() as u64)?;
    }

    // reward_bindWallet(wallet_ss58: String)
    // 由 node 端签名并提交 bind_reward_wallet 交易。
    {
        let client = client.clone();
        let pool = pool.clone();
        let keystore = keystore.clone();
        module.register_method("reward_bindWallet", move |params, _, _| {
            let wallet_ss58: String = params.one()?;
            let wallet = parse_ss58_account(&wallet_ss58)?;
            let call = runtime::RuntimeCall::FullnodeIssuance(
                fullnode_issuance::pallet::Call::bind_reward_wallet { wallet },
            );
            submit_reward_wallet_tx(&client, &pool, &keystore, call)?;
            Ok::<&str, jsonrpsee::types::ErrorObjectOwned>("ok")
        })?;
    }

    // reward_rebindWallet(new_wallet_ss58: String)
    // 由 node 端签名并提交 rebind_reward_wallet 交易。
    {
        let client = client.clone();
        let pool = pool.clone();
        let keystore = keystore.clone();
        module.register_method("reward_rebindWallet", move |params, _, _| {
            let wallet_ss58: String = params.one()?;
            let new_wallet = parse_ss58_account(&wallet_ss58)?;
            let call = runtime::RuntimeCall::FullnodeIssuance(
                fullnode_issuance::pallet::Call::rebind_reward_wallet { new_wallet },
            );
            submit_reward_wallet_tx(&client, &pool, &keystore, call)?;
            Ok::<&str, jsonrpsee::types::ErrorObjectOwned>("ok")
        })?;
    }

    // fee_blockFees(block_hash_hex: String) -> u128
    // 读取指定区块的 System::Events，累加所有 FeePaid.fee（base_fee）
    // 和 TransactionFeePaid.tip，返回真实总手续费。
    {
        let client = client.clone();
        module.register_method("fee_blockFees", move |params, _, _| {
            use jsonrpsee::types::error::ErrorObject;

            let hash_hex: String = params.one()?;
            let hash_bytes = hex::decode(hash_hex.trim_start_matches("0x"))
                .map_err(|e| ErrorObject::owned(-1, format!("无效区块哈希: {e}"), None::<()>))?;
            if hash_bytes.len() != 32 {
                return Err(ErrorObject::owned(-1, "区块哈希长度错误", None::<()>));
            }
            let block_hash = sp_core::H256::from_slice(&hash_bytes);

            // System::Events 的 storage key = twox_128("System") ++ twox_128("Events")
            let key = {
                let mut k = Vec::with_capacity(32);
                k.extend_from_slice(&sp_io::hashing::twox_128(b"System"));
                k.extend_from_slice(&sp_io::hashing::twox_128(b"Events"));
                k
            };
            let storage = client
                .storage(block_hash, &sp_storage::StorageKey(key))
                .map_err(|e| ErrorObject::owned(-1, format!("读取存储失败: {e}"), None::<()>))?;

            let Some(data) = storage else {
                return Ok(0u128);
            };

            type EventRecord = frame_system::EventRecord<runtime::RuntimeEvent, sp_core::H256>;
            let events: Vec<EventRecord> = Decode::decode(&mut &data.0[..])
                .map_err(|e| ErrorObject::owned(-1, format!("解码事件失败: {e}"), None::<()>))?;

            let mut total_fee: u128 = 0;
            for record in &events {
                match &record.event {
                    // base_fee（不含 tip）
                    runtime::RuntimeEvent::OnchainTransaction(
                        onchain_transaction::pallet::Event::FeePaid { fee, .. },
                    ) => {
                        total_fee = total_fee.saturating_add(*fee);
                    }
                    // tip 部分（由 pallet-transaction-payment 事件记录）
                    runtime::RuntimeEvent::TransactionPayment(
                        pallet_transaction_payment::Event::TransactionFeePaid { tip, .. },
                    ) => {
                        total_fee = total_fee.saturating_add(*tip);
                    }
                    _ => {}
                }
            }

            Ok(total_fee)
        })?;
    }

    Ok(module)
}
