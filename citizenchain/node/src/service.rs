//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use codec::{Decode, Encode};
use futures::FutureExt;
use citizenchain::{self, apis::RuntimeApi, opaque::Block};
use pow_difficulty_module::PowDifficultyApi;
use sc_client_api::{Backend, BlockBackend};
use sc_consensus_pow::{MiningHandle, PowAlgorithm, PowBlockImport};
use sc_service::WarpSyncConfig;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ProvideRuntimeApi;
use sp_consensus::NoNetwork;
use sp_core::{crypto::KeyTypeId, hashing::blake2_256, U256};
use sp_keystore::Keystore;
use sp_runtime::traits::{Block as BlockT, IdentifyAccount};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

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
const POW_MINING_TIMEOUT_SECS: u64 = 10;
const POW_PROPOSAL_BUILD_SECS: u64 = 2;
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 64;

#[derive(Clone)]
struct SimplePow {
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
        // 中文注释：协议层仅要求 pre_digest 可解码为矿工账户；是否绑定钱包只影响奖励/手续费分配，不影响出块有效性。
        let Some(pre_digest) = pre_digest else {
            return Ok(false);
        };
        match citizenchain::AccountId::decode(&mut &pre_digest[..]) {
            Ok(_) => (),
            Err(_) => return Ok(false),
        };

        let nonce = u64::decode(&mut &seal[..]).map_err(sc_consensus_pow::Error::<Block>::Codec)?;
        let hash = pow_hash(pre_hash.as_ref(), nonce);
        Ok(hash_meets_difficulty(&hash, difficulty))
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

fn author_pre_digest(keystore: &sp_keystore::KeystorePtr) -> Option<Vec<u8>> {
    // 中文注释：直接从 keystore 获取 powr 密钥，不再读取环境变量，避免明文密钥泄露。
    let keys = keystore.sr25519_public_keys(POW_AUTHOR_KEY_TYPE);
    let author_public = keys.into_iter().next()?;
    let account: citizenchain::AccountId =
        sp_runtime::MultiSigner::from(author_public).into_account();
    Some(account.encode())
}

fn ensure_powr_key(keystore: &sp_keystore::KeystorePtr) -> Result<(), ServiceError> {
    // 中文注释：密钥仅通过 keystore 管理，不再从环境变量读取，避免明文泄露于 /proc/PID/environ。
    let keys = keystore.sr25519_public_keys(POW_AUTHOR_KEY_TYPE);
    if !keys.is_empty() {
        return Ok(());
    }
    // 中文注释：节点首启自动生成唯一 powr 密钥，避免"无 key 仅告警继续跑"。
    keystore
        .sr25519_generate_new(POW_AUTHOR_KEY_TYPE, None)
        .map_err(|e| ServiceError::Other(format!("failed to generate powr key: {e}")))?;
    Ok(())
}

fn start_cpu_miner<Proof: Send + 'static>(
    worker: MiningHandle<Block, SimplePow, (), Proof>,
    num_threads: usize,
) {
    // 中文注释：提交门控，防止"早产块"触发 timestamp inherent 的 future 校验失败。
    // 所有矿工线程共享同一个门控。
    let min_submit_interval = Duration::from_millis(primitives::pow_const::MILLISECS_PER_BLOCK);
    let stride = (num_threads as u64).max(1);

    for thread_id in 0..num_threads {
        let worker = worker.clone();
        thread::spawn(move || loop {
            let Some(metadata) = worker.metadata() else {
                thread::sleep(Duration::from_millis(200));
                continue;
            };

            let build_version = worker.version();

            // 中文注释：共同随机基址（来自 pre_hash 前 8 字节）+ 线程号错位 + stride = 线程数。
            // 每轮 metadata 变化时基址自动更换；同一轮内各线程搜索的 nonce 集合完全不重叠。
            let random_base = {
                let seed_bytes = metadata.pre_hash.as_ref();
                u64::from_le_bytes(seed_bytes[..8].try_into().unwrap_or([0u8; 8]))
            };
            let mut nonce = random_base.wrapping_add(thread_id as u64);

            loop {
                if worker.version() != build_version {
                    break;
                }

                let hash = pow_hash(metadata.pre_hash.as_ref(), nonce);
                if hash_meets_difficulty(&hash, metadata.difficulty) {
                    // 中文注释：仅限制"提交频率"，挖矿过程仍持续进行。
                    // 这样可以保证区块时间戳不会持续跑在本地时间之前导致 future 错误。
                    static MINER_GATE: std::sync::OnceLock<std::sync::Mutex<Instant>> =
                        std::sync::OnceLock::new();
                    let gate = MINER_GATE
                        .get_or_init(|| Mutex::new(Instant::now() - min_submit_interval));
                    // 中文注释：使用 unwrap_or_else 恢复 poison，避免某线程 panic 后打死其他线程。
                    let mut last_submit =
                        gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    let elapsed = last_submit.elapsed();
                    if elapsed < min_submit_interval {
                        thread::sleep(min_submit_interval - elapsed);
                    }
                    let _ = futures::executor::block_on(worker.submit(nonce.encode()));
                    *last_submit = Instant::now();
                    break;
                }

                nonce = nonce.wrapping_add(stride);
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

/// Builds a new service for a full client.
pub fn new_full<
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
>(
    config: Configuration,
    mining_threads: usize,
) -> Result<TaskManager, ServiceError> {
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

    let mut net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        N,
    >::new(&config.network, config.prometheus_registry().cloned());
    let metrics = N::register_notification_metrics(config.prometheus_registry());
    let peer_store_handle = net_config.peer_store_handle();
    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    let (grandpa_protocol_config, grandpa_notification_service) =
        sc_consensus_grandpa::grandpa_peers_set_config::<_, N>(
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

    let role = config.role;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |_| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let keystore = keystore_container.keystore();
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
    let pre_runtime = author_pre_digest(&keystore)
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
    );

    task_manager.spawn_essential_handle().spawn(
        "pow-worker",
        Some("block-authoring"),
        worker_task.boxed(),
    );

    start_cpu_miner(worker, mining_threads);

    if enable_grandpa {
        let local_grandpa_keys = keystore.ed25519_public_keys(sp_consensus_grandpa::KEY_TYPE);
        let current_authorities = grandpa_link.shared_authority_set().current_authorities();
        let has_local_grandpa_authority = current_authorities.iter().any(|(id, _)| {
            local_grandpa_keys
                .iter()
                .any(|local| id.encode() == local.encode())
        });
        let grandpa_keystore = if role.is_authority() && has_local_grandpa_authority {
            Some(keystore.clone())
        } else {
            None
        };
        if role.is_authority() && grandpa_keystore.is_none() {
            eprintln!(
                "WARNING: authority role enabled but no matching local GRANDPA key for current authority set; this node will not cast finality votes."
            );
        }
        let grandpa_config = sc_consensus_grandpa::Config {
            gossip_duration: Duration::from_millis(333),
            justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
            name: Some(name),
            observer_enabled: !role.is_authority() || grandpa_keystore.is_none(),
            keystore: grandpa_keystore,
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
    }

    Ok(task_manager)
}
