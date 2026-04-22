//! # 节点服务层 (service)
//!
//! 实现 CitizenChain 全节点的双共识架构：
//! - **PoW 共识**：SimplePow 算法，blake2_256(pre_hash ++ nonce)，难度从链上 Runtime API 读取。
//! - **GRANDPA 最终性**：权威节点运行 voter，普通节点运行 observer。
//!
//! 挖矿特性：
//! - CPU 多线程挖矿，各线程 nonce 不重叠（stride = 线程数）。
//! - GPU 挖矿（可选 `gpu-mining` feature），使用 nonce 高半区（bit63=1）。
//! - 空交易池时不挖矿（避免空块），离线或 major sync 时禁止出块（防分叉）。
//! - 出块目标时间从 genesis-pallet 链上存储读取，启动时获取一次。

use citizenchain::{self, apis::RuntimeApi, opaque::Block};
use codec::{Decode, Encode};
use futures::FutureExt;
use genesis_pallet::GenesisPalletApi;
use pow_difficulty_module::PowDifficultyApi;
use sc_client_api::{Backend, BlockBackend};
use sc_consensus_pow::{MiningHandle, PowAlgorithm, PowBlockImport};
use sc_network::NetworkBackend as _;
use sc_service::WarpSyncConfig;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ProvideRuntimeApi;
use sp_consensus::{NoNetwork, SyncOracle};
use sp_core::{crypto::KeyTypeId, hashing::blake2_256, sr25519, Pair as _, U256};
use sp_keystore::Keystore;
use sp_runtime::traits::Block as BlockT;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

/// CPU 全线程合计哈希率（hashes/sec），以 f64 bits 存入 AtomicU64。
static CPU_HASHRATE: AtomicU64 = AtomicU64::new(0);

// 空块 propose 防护在 start_mining_worker_no_empty 中实现。

/// 上次成功提交区块的时刻（自 epoch 起的纳秒数）。
/// CPU 和 GPU 矿工共享此门控，防止出块频率超过 MILLISECS_PER_BLOCK。
/// 初始值 u64::MAX 表示"从未提交过"，首次提交直接放行。
pub static LAST_SUBMIT_NS: AtomicU64 = AtomicU64::new(u64::MAX);

/// 获取当前 CPU 哈希率（hashes/sec）。
pub(crate) fn cpu_hashrate() -> f64 {
    f64::from_bits(CPU_HASHRATE.load(Ordering::Relaxed))
}

pub(crate) type FullClient = sc_service::TFullClient<
    Block,
    RuntimeApi,
    sc_executor::WasmExecutor<sp_io::SubstrateHostFunctions>,
>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub type Service = sc_service::PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
    (
        sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
        sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
        Option<Telemetry>,
    ),
>;

// PoW 作者密钥类型：纯 PoW 链使用独立 key type，避免与 Aura 语义混用。
const POW_AUTHOR_KEY_TYPE: KeyTypeId = KeyTypeId(*b"powr");
const POW_MINING_TIMEOUT_SECS: u64 = 2;
const POW_PROPOSAL_BUILD_SECS: u64 = 2;
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 64;

#[derive(Clone)]
pub(crate) struct SimplePow {
    /// 持有 client 引用，用于通过 Runtime API 读取链上最新难度值。
    client: Arc<FullClient>,
}

impl SimplePow {
    fn new(client: Arc<FullClient>) -> Self {
        Self { client }
    }
}

impl PowAlgorithm<Block> for SimplePow {
    type Difficulty = U256;

    /// 从链上读取当前 PoW 难度。
    /// 若 Runtime API 调用失败（如节点启动初期），回退到 POW_INITIAL_DIFFICULTY 初始值。
    fn difficulty(
        &self,
        parent: <Block as BlockT>::Hash,
    ) -> Result<Self::Difficulty, sc_consensus_pow::Error<Block>> {
        let difficulty = self
            .client
            .runtime_api()
            .current_pow_difficulty(parent)
            .unwrap_or(primitives::pow_const::POW_INITIAL_DIFFICULTY);
        Ok(U256::from(difficulty))
    }

    fn verify(
        &self,
        _parent: &sp_runtime::generic::BlockId<Block>,
        pre_hash: &<Block as BlockT>::Hash,
        pre_digest: Option<&[u8]>,
        seal: &sp_consensus_pow::Seal,
        difficulty: Self::Difficulty,
    ) -> Result<bool, sc_consensus_pow::Error<Block>> {
        // 中文注释：pre_digest 包含矿工 sr25519 公钥，seal 包含 (nonce, 签名)。
        // 验证：1) PoW 难度满足  2) 签名证明矿工确实拥有该公钥的私钥。
        let Some(pre_digest) = pre_digest else {
            return Ok(false);
        };
        let public = match sr25519::Public::decode(&mut &pre_digest[..]) {
            Ok(p) => p,
            Err(_) => return Ok(false),
        };

        let (nonce, signature): (u64, sr25519::Signature) =
            Decode::decode(&mut &seal[..]).map_err(sc_consensus_pow::Error::<Block>::Codec)?;

        let hash = pow_hash(pre_hash.as_ref(), nonce);
        if !hash_meets_difficulty(&hash, difficulty) {
            return Ok(false);
        }

        // 中文注释：验证矿工对 pre_hash 的 sr25519 签名，防止冒充他人公钥。
        Ok(sr25519::Pair::verify(
            &signature,
            pre_hash.as_ref(),
            &public,
        ))
    }
}

fn pow_hash(pre_hash: &[u8], nonce: u64) -> [u8; 32] {
    let mut payload = Vec::with_capacity(pre_hash.len() + core::mem::size_of::<u64>());
    payload.extend_from_slice(pre_hash);
    payload.extend_from_slice(&nonce.to_le_bytes());
    blake2_256(&payload)
}

fn hash_meets_difficulty(hash: &[u8; 32], difficulty: U256) -> bool {
    if difficulty.is_zero() {
        return false;
    }
    let target = U256::MAX / difficulty;
    U256::from_big_endian(hash) <= target
}

/// 中文注释：返回 (pre_digest 编码字节, 矿工公钥)。
/// pre_digest 中存储的是 sr25519 公钥而非 AccountId，配合 seal 中的签名实现密码学绑定。
fn author_pre_digest(keystore: &sp_keystore::KeystorePtr) -> Option<(Vec<u8>, sr25519::Public)> {
    let keys = keystore.sr25519_public_keys(POW_AUTHOR_KEY_TYPE);
    let author_public = keys.into_iter().next()?;
    Some((author_public.encode(), author_public))
}

fn ensure_powr_key(keystore: &sp_keystore::KeystorePtr) -> Result<(), ServiceError> {
    let keys = keystore.sr25519_public_keys(POW_AUTHOR_KEY_TYPE);
    if !keys.is_empty() {
        return Ok(());
    }
    // 传 None 让 Substrate 生成 BIP39 助记词并写入 keystore 磁盘文件，
    // nodeui 后续能读取同一把密钥来签名绑定交易。
    // 注意：传 Some(suri) 只存内存不写磁盘，重启后丢失。
    keystore
        .sr25519_generate_new(POW_AUTHOR_KEY_TYPE, None)
        .map_err(|e| ServiceError::Other(format!("failed to generate powr key: {e}")))?;
    Ok(())
}

fn start_cpu_miner<Proof: Send + 'static>(
    worker: MiningHandle<Block, SimplePow, (), Proof>,
    num_threads: usize,
    epoch: Instant,
    pool_ready: Arc<dyn Fn() -> usize + Send + Sync>,
    target_block_time_ms: u64,
    keystore: sp_keystore::KeystorePtr,
    author_public: sr25519::Public,
) {
    // 提交门控，防止"早产块"触发 timestamp inherent 的 future 校验失败。
    // 使用全局 AtomicU64 (LAST_SUBMIT_NS) 存储上次成功提交的时刻（自 epoch 的纳秒数），
    // 避免 Mutex 在 sleep 期间持锁阻塞其他线程。CPU 和 GPU 矿工共享此门控。
    // 中文注释：出块目标时间从 genesis-pallet Runtime API 读取，替代编译期常量。
    let min_submit_interval = Duration::from_millis(target_block_time_ms);
    let stride = (num_threads as u64).max(1);

    for thread_id in 0..num_threads {
        let worker = worker.clone();
        let epoch = epoch;
        let pool_ready = pool_ready.clone();
        let keystore = keystore.clone();
        let author_public = author_public;
        thread::spawn(move || {
            // 哈希率采样：仅 thread 0 每 SAMPLE_INTERVAL 次哈希统计一次，乘以线程数得到总哈希率。
            const SAMPLE_INTERVAL: u64 = 100_000;
            let mut sample_count: u64 = 0;
            let mut sample_start = Instant::now();

            loop {
                let Some(metadata) = worker.metadata() else {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                };

                // 空块不提交：交易池无待打包交易时不挖矿，避免产生空块。
                if pool_ready() == 0 {
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }

                let build_version = worker.version();

                // 共同随机基址（来自 pre_hash 前 8 字节）+ 线程号错位 + stride = 线程数。
                // 每轮 metadata 变化时基址自动更换；同一轮内各线程搜索的 nonce 集合完全不重叠。
                let random_base = {
                    let seed_bytes = metadata.pre_hash.as_ref();
                    let seed = u64::from_le_bytes(seed_bytes[..8].try_into().unwrap_or([0u8; 8]));
                    // CPU 使用低半区 nonce（bit 63 = 0），高半区留给 GPU。
                    seed & 0x7FFFFFFFFFFFFFFF
                };
                let mut nonce = random_base.wrapping_add(thread_id as u64);

                loop {
                    if worker.version() != build_version {
                        break;
                    }

                    // thread 0 负责采样更新全局哈希率。
                    if thread_id == 0 {
                        sample_count += 1;
                        if sample_count >= SAMPLE_INTERVAL {
                            let elapsed = sample_start.elapsed();
                            if elapsed.as_nanos() > 0 {
                                let per_thread = sample_count as f64 / elapsed.as_secs_f64();
                                let total = per_thread * stride as f64;
                                CPU_HASHRATE.store(total.to_bits(), Ordering::Relaxed);
                            }
                            sample_count = 0;
                            sample_start = Instant::now();
                        }
                    }

                    let hash = pow_hash(metadata.pre_hash.as_ref(), nonce);
                    if hash_meets_difficulty(&hash, metadata.difficulty) {
                        // ── 提交门控（无锁版）──────────────────────────────
                        // 读取上次成功提交的时刻，不足间隔则 sleep 补齐（不持锁）。
                        // u64::MAX 表示"从未提交过"，首次直接放行。
                        let last_ns = LAST_SUBMIT_NS.load(Ordering::Acquire);
                        if last_ns != u64::MAX {
                            let now_ns = epoch.elapsed().as_nanos() as u64;
                            let interval_ns = min_submit_interval.as_nanos() as u64;
                            let deadline_ns = last_ns.saturating_add(interval_ns);
                            if now_ns < deadline_ns {
                                let wait = Duration::from_nanos(deadline_ns - now_ns);
                                thread::sleep(wait);
                            }
                        }

                        // sleep 后 build 可能已更新，先检查版本是否仍匹配。
                        if worker.version() != build_version {
                            break; // nonce 已过期，回外层重新获取 metadata
                        }

                        // 中文注释：签名 pre_hash 证明矿工身份，签名失败则丢弃该 nonce。
                        let signature = match keystore.sr25519_sign(
                            POW_AUTHOR_KEY_TYPE,
                            &author_public,
                            metadata.pre_hash.as_ref(),
                        ) {
                            Ok(Some(sig)) => sig,
                            _ => {
                                log::warn!("PoW: keystore 签名失败，丢弃 nonce");
                                break;
                            }
                        };
                        let seal = (nonce, sr25519::Signature::from(signature)).encode();
                        let submitted = futures::executor::block_on(worker.submit(seal));

                        if submitted {
                            let submit_ns = epoch.elapsed().as_nanos() as u64;
                            if pool_ready() > 0 {
                                // 提交后交易池仍有待处理交易 → 当前块是旧 Proposal（不含新交易）。
                                // 将门控起点前移半个周期，使下一个块只需等 MinPeriod
                                // （target_block_time / 2）即可提交，而非完整出块间隔。
                                // 这既保证 timestamp inherent 校验通过（MinPeriod ≤ MAX_DRIFT + elapsed），
                                // 又让包含交易的真实块能尽快上链。
                                let half_ns = min_submit_interval.as_nanos() as u64 / 2;
                                LAST_SUBMIT_NS
                                    .store(submit_ns.saturating_sub(half_ns), Ordering::Release);
                            } else {
                                LAST_SUBMIT_NS.store(submit_ns, Ordering::Release);
                            }
                        }
                        break;
                    }

                    nonce = nonce.wrapping_add(stride);
                }
            }
        });
    }
}

pub fn new_partial(config: &Configuration) -> Result<Service, ServiceError> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);
    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let algorithm = SimplePow::new(client.clone());
    let pow_block_import = PowBlockImport::new(
        grandpa_block_import.clone(),
        client.clone(),
        algorithm.clone(),
        0,
        select_chain.clone(),
        |_, ()| async {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            Ok((timestamp,))
        },
    );

    let import_queue = sc_consensus_pow::import_queue(
        Box::new(pow_block_import),
        Some(Box::new(grandpa_block_import.clone())),
        algorithm,
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    )?;

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (grandpa_block_import, grandpa_link, telemetry),
    })
}

/// 网络后端类型：固定使用 libp2p（支持 WSS + DCUtR/Relay/AutoNAT）。
type NetworkBackend = sc_network::NetworkWorker<Block, <Block as sp_runtime::traits::Block>::Hash>;

/// Builds a new service for a full client.
pub fn new_full(
    mut config: Configuration,
    mining_threads: usize,
    gpu_device: Option<usize>,
    // 扫码支付 Step 2b-ii-β-2-b 新增:清算行主账户 SS58(None=本节点不做清算行角色)
    clearing_bank: Option<String>,
    // 扫码支付 Step 2b-ii-β-2-b 新增:解锁 offchain_keystore 的密码
    clearing_bank_password: Option<String>,
    // 扫码支付 Step 2b-iii-b 新增:reserve_monitor 对账周期(秒),None=默认 300,Some(0)=关闭
    clearing_reserve_monitor_interval_secs: Option<u64>,
) -> Result<TaskManager, ServiceError> {
    // 生成或加载 TLS 自签证书，注入到网络配置中。
    let tls_cert = crate::tls_cert::load_or_generate_tls_cert(config.base_path.path())
        .map_err(|e| ServiceError::Other(e.into()))?;
    config.network.tls_private_key_der = Some(tls_cert.private_key_der);
    config.network.tls_certificate_chain_der = Some(tls_cert.certificate_chain_der);

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, mut telemetry),
    } = new_partial(&config)?;

    let keystore = keystore_container.keystore();
    let role = config.role;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let local_grandpa_keys = keystore.ed25519_public_keys(sp_consensus_grandpa::KEY_TYPE);
    let current_authorities = grandpa_link.shared_authority_set().current_authorities();
    let has_local_grandpa_authority = enable_grandpa
        && current_authorities.iter().any(|(id, _)| {
            local_grandpa_keys
                .iter()
                .any(|local| id.encode() == local.encode())
        });

    let mut net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        NetworkBackend,
    >::new(&config.network, config.prometheus_registry().cloned());
    let metrics = NetworkBackend::register_notification_metrics(config.prometheus_registry());
    let peer_store_handle = net_config.peer_store_handle();
    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    // 中文注释：所有节点统一注册 GRANDPA 网络协议，保证协议栈一致，避免协议协商不对称导致连接断开。
    // 权威节点启动 grandpa-voter 消费 notification_service；普通节点启动 grandpa-observer 消费。
    let (grandpa_protocol_config, grandpa_notification_service) =
        sc_consensus_grandpa::grandpa_peers_set_config::<_, NetworkBackend>(
            grandpa_protocol_name.clone(),
            metrics.clone(),
            peer_store_handle,
        );
    net_config.add_notification_protocol(grandpa_protocol_config);

    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::new(),
    ));

    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_config: Some(WarpSyncConfig::WithProvider(warp_sync)),
            block_relay: None,
            metrics,
        })?;

    if config.offchain_worker.enabled {
        let offchain_workers =
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                is_validator: config.role.is_authority(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                enable_http_requests: true,
                custom_extensions: |_| vec![],
            })?;
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-worker",
            offchain_workers
                .run(client.clone(), task_manager.spawn_handle())
                .boxed(),
        );
    }

    let prometheus_registry = config.prometheus_registry().cloned();

    // GPU 哈希率函数指针：仅在 gpu-mining feature 且用户启用 GPU 时传入。
    let gpu_hashrate_fn: Option<fn() -> f64> = {
        #[cfg(feature = "gpu-mining")]
        {
            if gpu_device.is_some() {
                Some(crate::gpu_miner::gpu_hashrate as fn() -> f64)
            } else {
                None
            }
        }
        #[cfg(not(feature = "gpu-mining"))]
        {
            None
        }
    };

    // ─── 扫码支付 Step 2b-ii-β-2-b:清算行组件启动 ───────────────────────
    // 若 CLI 设了 --clearing-bank 且 SS58 合法,启动清算行 offchain 组件
    // (ledger + packer + rpc_impl + event_listener),并挂一个 30 秒 tick 的
    // packer worker 到 `task_manager`。启动失败/地址非法时 log warning 但
    // 不中断 PoW + GRANDPA 启动(基础节点职能优先)。
    let clearing_rpc_impl: Option<Arc<crate::offchain::rpc::OffchainClearingRpcImpl>> =
        if let Some(bank_ss58) = clearing_bank.as_deref() {
            use sp_core::crypto::Ss58Codec;
            match sp_runtime::AccountId32::from_ss58check(bank_ss58) {
                Ok(bank_main) => {
                    let password = clearing_bank_password.as_deref().unwrap_or("");
                    let keystore_path = config.base_path.path();
                    let offchain_keystore =
                        crate::offchain_keystore::OffchainKeystore::new(keystore_path);
                    let signing_key_slot: Arc<
                        std::sync::RwLock<Option<crate::offchain_keystore::SigningKey>>,
                    > = Arc::new(std::sync::RwLock::new(None));
                    if offchain_keystore.has_signing_key() && !password.is_empty() {
                        match offchain_keystore.load_signing_key(password) {
                            Ok(key) => {
                                *signing_key_slot.write().expect("lock") = Some(key);
                                log::info!("[ClearingBank] 签名密钥已解锁");
                            }
                            Err(e) => log::warn!(
                                "[ClearingBank] 签名密钥解锁失败:{e},packer 将拒绝提交 extrinsic"
                            ),
                        }
                    } else {
                        log::warn!(
                            "[ClearingBank] 签名密钥未加载(密码或密钥文件缺失),\
                             packer 会在有 pending 时 rollback"
                        );
                    }

                    let signer: Arc<dyn crate::offchain::packer::BatchSigner> = Arc::new(
                        crate::offchain::KeystoreBatchSigner::new(signing_key_slot.clone()),
                    );
                    let submitter: Arc<dyn crate::offchain::packer::BatchSubmitter> =
                        Arc::new(crate::offchain::pool_submitter::PoolBatchSubmitter::new(
                            client.clone(),
                            transaction_pool.clone(),
                            signing_key_slot,
                        ));

                    match crate::offchain::start_clearing_bank_components(
                        keystore_path,
                        bank_main.clone(),
                        password,
                        signer,
                        submitter,
                        client.clone(),
                    ) {
                        Ok(components) => {
                            let packer = components.packer.clone();
                            let client_for_loop = client.clone();
                            task_manager.spawn_handle().spawn(
                                "offchain-clearing-packer",
                                Some("offchain"),
                                async move {
                                    // 闭包内需要显式引入 HeaderBackend trait 才能调 `info()`。
                                    use sp_blockchain::HeaderBackend as _;
                                    use sp_runtime::traits::SaturatedConversion as _;
                                    let mut interval =
                                        tokio::time::interval(Duration::from_secs(30));
                                    loop {
                                        interval.tick().await;
                                        let info = client_for_loop.info();
                                        let current_block: u64 = info.best_number.saturated_into();
                                        if packer.should_pack(current_block).await {
                                            match packer.pack_and_submit(current_block).await {
                                                Ok(Some(hash)) => log::info!(
                                                    "[ClearingPacker] batch ok tx=0x{:x}",
                                                    hash
                                                ),
                                                Ok(None) => {}
                                                Err(e) => {
                                                    log::warn!("[ClearingPacker] {e}")
                                                }
                                            }
                                        }
                                    }
                                },
                            );

                            // 扫码支付 Step 2b-iii-a:启动链上事件监听 worker。
                            // 订阅 import_notification_stream,每个新块读 `System::Events`
                            // → 解码 → 过滤本 pallet 事件 → 分发到 ledger。
                            // packer 提交 extrinsic 后,runtime 发 `PaymentSettled` 事件,
                            // 本 worker 捕获后调 `ledger.on_payment_settled` 清理 pending,
                            // 从而形成 wuminapp RPC → ledger → pool → runtime → 事件回写
                            // 的完整闭环。
                            let listener = components.event_listener.clone();
                            let client_for_events = client.clone();
                            task_manager.spawn_handle().spawn(
                                "offchain-clearing-event-listener",
                                Some("offchain"),
                                async move {
                                    listener.run(client_for_events).await;
                                },
                            );

                            // 扫码支付 Step 2b-iii-b:启动主账对账 worker(可选)。
                            // 周期对比本地 `Σ confirmed` 与链上 `BankTotalDeposits`,
                            // 偏差触发 log::error!。interval=0 时跳过 spawn(关闭对账,仅
                            // 排障用;生产部署必须保留)。缺省 300 秒。
                            let monitor_interval_secs =
                                clearing_reserve_monitor_interval_secs.unwrap_or(300);
                            if monitor_interval_secs > 0 {
                                let monitor = components.reserve_monitor.clone();
                                let client_for_monitor = client.clone();
                                task_manager.spawn_handle().spawn(
                                    "offchain-clearing-reserve-monitor",
                                    Some("offchain"),
                                    async move {
                                        monitor
                                            .run(
                                                client_for_monitor,
                                                Duration::from_secs(monitor_interval_secs),
                                            )
                                            .await;
                                    },
                                );
                            } else {
                                log::warn!(
                                    "[ClearingBank] reserve_monitor 已关闭(interval=0),\
                                     仅用于排障,生产环境请保留默认 300 秒"
                                );
                            }

                            log::info!(
                                "[ClearingBank] 清算行组件已启动,bank_main={}",
                                bank_main.to_ss58check()
                            );
                            Some(components.rpc_impl.clone())
                        }
                        Err(e) => {
                            log::warn!("[ClearingBank] 组件启动失败:{e}");
                            None
                        }
                    }
                }
                Err(e) => {
                    log::warn!(
                        "[ClearingBank] --clearing-bank SS58 解析失败:{e:?},清算行组件不启动"
                    );
                    None
                }
            }
        } else {
            None
        };

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let keystore = keystore_container.keystore();
        let chain_spec = config.chain_spec.cloned_box();
        let clearing_rpc_impl = clearing_rpc_impl.clone();

        Box::new(move |_| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                keystore: keystore.clone(),
                cpu_hashrate_fn: cpu_hashrate as fn() -> f64,
                gpu_hashrate_fn,
                chain_spec: chain_spec.cloned_box(),
                // 扫码支付 Step 2b-ii-β-2-b:清算行 RPC 命名空间(None 时跳过注入)
                offchain_clearing_rpc: clearing_rpc_impl.clone(),
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::new(network.clone()),
        client: client.clone(),
        keystore: keystore.clone(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        config,
        telemetry: telemetry.as_mut(),
        tracing_execute_block: None,
    })?;

    // 中文注释：本链制度要求"安装全节点软件即可参与挖矿"，不再依赖 authority 角色开关。
    ensure_powr_key(&keystore)?;

    let proposer_factory = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        client.clone(),
        transaction_pool.clone(),
        prometheus_registry.as_ref(),
        telemetry.as_ref().map(|x| x.handle()),
    );

    let algorithm = SimplePow::new(client.clone());
    let (pre_runtime, author_public) = author_pre_digest(&keystore)
        .ok_or_else(|| ServiceError::Other("powr key missing after generation attempt".into()))?;

    let pow_block_import = PowBlockImport::new(
        block_import,
        client.clone(),
        algorithm.clone(),
        0,
        select_chain.clone(),
        |_, ()| async {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            Ok((timestamp,))
        },
    );

    // 空块不提交：构造一个闭包，返回交易池中待打包的交易数。
    // CPU 和 GPU 矿工在交易池为空时跳过挖矿，避免产生空块。
    // 额外门控：节点必须先接入网络并完成主要同步，才允许恢复正常出块，
    // 避免清库后的普通节点在未连上现网前先本地起出一条分叉链。
    let pool_ready: Arc<dyn Fn() -> usize + Send + Sync> = {
        use sc_transaction_pool_api::TransactionPool;
        let pool = transaction_pool.clone();
        let sync_service_for_pool = sync_service.clone();
        Arc::new(move || {
            // 没有同步 peer 或仍在 major sync 时，禁止本地挖矿，
            // 防止离线状态继续出块并与现网分叉。
            if sync_service_for_pool.is_offline() || sync_service_for_pool.is_major_syncing() {
                return 0;
            }

            pool.status().ready
        })
    };

    // PoW mining worker：在 propose 前检查 pool_ready，交易池为空时跳过 propose，
    // 避免触发 runtime 的空块 assert panic。
    let should_propose = {
        let pr = pool_ready.clone();
        move || pr() > 0
    };
    let (worker, worker_task) = sc_consensus_pow::start_mining_worker(
        Box::new(pow_block_import),
        client.clone(),
        select_chain,
        algorithm,
        proposer_factory,
        NoNetwork,
        (),
        Some(pre_runtime),
        |_, ()| async {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            Ok((timestamp,))
        },
        Duration::from_secs(POW_MINING_TIMEOUT_SECS),
        Duration::from_secs(POW_PROPOSAL_BUILD_SECS),
        should_propose,
    );

    task_manager.spawn_essential_handle().spawn(
        "pow-worker",
        Some("block-authoring"),
        worker_task.boxed(),
    );

    // 所有矿工线程共享的时间基准，用于无锁提交门控。
    let miner_epoch = Instant::now();

    // 中文注释：从 genesis-pallet 链上存储读取动态出块目标时间，
    // 替代编译期常量 MILLISECS_PER_BLOCK。若 API 调用失败，回退到常量默认值。
    let target_block_time_ms = {
        use sp_blockchain::HeaderBackend;
        let best = client.info().best_hash;
        client
            .runtime_api()
            .target_block_time_ms(best)
            .unwrap_or(primitives::pow_const::MILLISECS_PER_BLOCK)
    };

    if mining_threads > 0 {
        start_cpu_miner(
            worker.clone(),
            mining_threads,
            miner_epoch,
            pool_ready.clone(),
            target_block_time_ms,
            keystore.clone(),
            author_public,
        );
    }

    // GPU 矿工（仅在 gpu-mining feature 编译时可用）。
    #[cfg(feature = "gpu-mining")]
    if let Some(device) = gpu_device {
        match crate::gpu_miner::try_start(
            worker.clone(),
            device,
            miner_epoch,
            pool_ready.clone(),
            target_block_time_ms,
            keystore.clone(),
            author_public,
        ) {
            Ok(()) => log::info!("GPU miner started on device {}", device),
            Err(e) => log::warn!("GPU not available, CPU only: {}", e),
        }
    }

    // 避免 unused 警告（无 gpu-mining feature 时 gpu_device 未使用）。
    #[cfg(not(feature = "gpu-mining"))]
    let _ = gpu_device;

    drop(worker);

    if enable_grandpa {
        if has_local_grandpa_authority {
            // 中文注释：权威节点启动 grandpa-voter，参与最终性投票。
            let grandpa_config = sc_consensus_grandpa::Config {
                gossip_duration: Duration::from_millis(333),
                justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
                name: Some(name),
                observer_enabled: false,
                keystore: Some(keystore.clone()),
                local_role: role,
                telemetry: telemetry.as_ref().map(|x| x.handle()),
                protocol_name: grandpa_protocol_name,
            };

            let grandpa_params = sc_consensus_grandpa::GrandpaParams {
                config: grandpa_config,
                link: grandpa_link,
                network: network.clone(),
                sync: Arc::new(sync_service),
                notification_service: grandpa_notification_service,
                voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
                prometheus_registry,
                shared_voter_state: sc_consensus_grandpa::SharedVoterState::empty(),
                telemetry: telemetry.as_ref().map(|x| x.handle()),
                offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool),
            };

            task_manager.spawn_essential_handle().spawn_blocking(
                "grandpa-voter",
                None,
                sc_consensus_grandpa::run_grandpa_voter(grandpa_params)?,
            );
        } else {
            // 中文注释：普通节点启动 grandpa-observer，只接收最终性结果不投票，
            // 同时消费 notification_service 避免空接收端导致 EssentialTaskClosed。
            let grandpa_config = sc_consensus_grandpa::Config {
                gossip_duration: Duration::from_millis(333),
                justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
                name: Some(name),
                observer_enabled: false,
                keystore: None,
                local_role: role,
                telemetry: telemetry.as_ref().map(|x| x.handle()),
                protocol_name: grandpa_protocol_name,
            };

            task_manager.spawn_handle().spawn_blocking(
                "grandpa-observer",
                None,
                sc_consensus_grandpa::run_grandpa_observer(
                    grandpa_config,
                    grandpa_link,
                    network.clone(),
                    Arc::new(sync_service),
                    grandpa_notification_service,
                )?,
            );
        }
    }

    Ok(task_manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pow_hash_deterministic() {
        let pre_hash = [0u8; 32];
        let h1 = pow_hash(&pre_hash, 42);
        let h2 = pow_hash(&pre_hash, 42);
        assert_eq!(h1, h2);
    }

    #[test]
    fn pow_hash_differs_with_different_nonce() {
        let pre_hash = [1u8; 32];
        assert_ne!(pow_hash(&pre_hash, 0), pow_hash(&pre_hash, 1));
    }

    #[test]
    fn pow_hash_differs_with_different_pre_hash() {
        assert_ne!(pow_hash(&[0u8; 32], 0), pow_hash(&[1u8; 32], 0));
    }

    #[test]
    fn pow_hash_matches_manual_blake2() {
        let pre_hash = [7u8; 32];
        let nonce = 123u64;
        let mut payload = Vec::new();
        payload.extend_from_slice(&pre_hash);
        payload.extend_from_slice(&nonce.to_le_bytes());
        assert_eq!(pow_hash(&pre_hash, nonce), blake2_256(&payload));
    }

    #[test]
    fn hash_meets_difficulty_zero_always_false() {
        assert!(!hash_meets_difficulty(&[0u8; 32], U256::zero()));
    }

    #[test]
    fn hash_meets_difficulty_one_always_true() {
        // difficulty=1 → target=U256::MAX, any hash passes
        assert!(hash_meets_difficulty(&[0xFF; 32], U256::one()));
    }

    #[test]
    fn hash_meets_difficulty_max_only_zero_hash() {
        // difficulty=U256::MAX → target=1, only hash ≤ 1 passes
        assert!(hash_meets_difficulty(&[0u8; 32], U256::MAX));
        let mut h = [0u8; 32];
        h[31] = 2;
        assert!(!hash_meets_difficulty(&h, U256::MAX));
    }

    #[test]
    fn hash_meets_difficulty_boundary() {
        let difficulty = U256::from(2);
        let target = U256::MAX / difficulty;
        // At target: pass
        let at_target: [u8; 32] = target.to_big_endian();
        assert!(hash_meets_difficulty(&at_target, difficulty));
        // Above target: fail
        let above = target + U256::one();
        let above_bytes: [u8; 32] = above.to_big_endian();
        assert!(!hash_meets_difficulty(&above_bytes, difficulty));
    }
}
