//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use codec::{Decode, Encode};
use futures::FutureExt;
use sc_client_api::Backend;
use sc_consensus_pow::{MiningHandle, PowAlgorithm, PowBlockImport};
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use gmb_runtime::{self, apis::RuntimeApi, opaque::Block};
use sp_consensus::NoNetwork;
use sp_core::{crypto::KeyTypeId, hashing::blake2_256, U256};
use sp_runtime::traits::{Block as BlockT, IdentifyAccount};
use std::{sync::Arc, thread, time::Duration};

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
	Option<Telemetry>,
>;

// PoW 作者密钥类型：纯 PoW 链使用独立 key type，避免与 Aura 语义混用。
const POW_AUTHOR_KEY_TYPE: KeyTypeId = KeyTypeId(*b"powr");
const POW_DIFFICULTY: u64 = 1_000_000;
const POW_MINING_TIMEOUT_SECS: u64 = 10;
const POW_PROPOSAL_BUILD_SECS: u64 = 2;

#[derive(Clone)]
struct SimplePow {
	difficulty: U256,
}

impl Default for SimplePow {
	fn default() -> Self {
		Self { difficulty: U256::from(POW_DIFFICULTY) }
	}
}

impl PowAlgorithm<Block> for SimplePow {
	type Difficulty = U256;

	fn difficulty(&self, _parent: <Block as BlockT>::Hash) -> Result<Self::Difficulty, sc_consensus_pow::Error<Block>> {
		Ok(self.difficulty)
	}

	fn verify(
		&self,
		_parent: &sp_runtime::generic::BlockId<Block>,
		pre_hash: &<Block as BlockT>::Hash,
		_pre_digest: Option<&[u8]>,
		seal: &sp_consensus_pow::Seal,
		difficulty: Self::Difficulty,
	) -> Result<bool, sc_consensus_pow::Error<Block>> {
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
	let author_public = keystore.sr25519_public_keys(POW_AUTHOR_KEY_TYPE).into_iter().next()?;
	let account: gmb_runtime::AccountId = sp_runtime::MultiSigner::from(author_public).into_account();
	Some(account.encode())
}

fn start_cpu_miner<Proof: Send + 'static>(worker: MiningHandle<Block, SimplePow, (), Proof>) {
	thread::spawn(move || loop {
		let Some(metadata) = worker.metadata() else {
			thread::sleep(Duration::from_millis(200));
			continue;
		};

		let build_version = worker.version();
		let mut nonce = 0u64;

		loop {
			if worker.version() != build_version {
				break;
			}

			let hash = pow_hash(metadata.pre_hash.as_ref(), nonce);
			if hash_meets_difficulty(&hash, metadata.difficulty) {
				let _ = futures::executor::block_on(worker.submit(nonce.encode()));
				break;
			}

			nonce = nonce.wrapping_add(1);
		}
	});
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
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
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

	let algorithm = SimplePow::default();
	let pow_block_import = PowBlockImport::new(
		client.clone(),
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
		None,
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
		other: telemetry,
	})
}

/// Builds a new service for a full client.
pub fn new_full<
	N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
>(
	config: Configuration,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: mut telemetry,
	} = new_partial(&config)?;

	let net_config = sc_network::config::FullNetworkConfiguration::<
		Block,
		<Block as sp_runtime::traits::Block>::Hash,
		N,
	>::new(&config.network, config.prometheus_registry().cloned());
	let metrics = N::register_notification_metrics(config.prometheus_registry());

	let (network, system_rpc_tx, tx_handler_controller, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_config: None,
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
			offchain_workers.run(client.clone(), task_manager.spawn_handle()).boxed(),
		);
	}

	let role = config.role;
	let prometheus_registry = config.prometheus_registry().cloned();

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |_| {
			let deps = crate::rpc::FullDeps { client: client.clone(), pool: pool.clone() };
			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: Arc::new(network.clone()),
		client: client.clone(),
		keystore: keystore_container.keystore(),
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

	if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let algorithm = SimplePow::default();
		let pre_runtime = author_pre_digest(&keystore_container.keystore());
		if pre_runtime.is_none() {
			eprintln!("WARN [pow] No sr25519 key with key type 'powr' found; PoW rewards may not be issued.");
		}

		let pow_block_import = PowBlockImport::new(
			client.clone(),
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
			pre_runtime,
			|_, ()| async {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
				Ok((timestamp,))
			},
			Duration::from_secs(POW_MINING_TIMEOUT_SECS),
			Duration::from_secs(POW_PROPOSAL_BUILD_SECS),
		);

		task_manager
			.spawn_essential_handle()
			.spawn("pow-worker", Some("block-authoring"), worker_task.boxed());

		start_cpu_miner(worker);
	}

	Ok(task_manager)
}
