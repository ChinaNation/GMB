//! 清算行 offchain 组件启动接线。
//!
//! 中文注释:
//! - `service.rs` 负责节点通用启动,本文件负责清算行专属启动。
//! - 这里统一处理 CLI 参数、密钥解锁、packer/listener/reserve worker spawn。

use sc_service::TaskManager;
use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;
use std::{
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};

use super::keystore::{OffchainKeystore, SigningKey};
use super::rpc::OffchainClearingRpcImpl;
use super::settlement::packer::{BatchSigner, BatchSubmitter};
use super::settlement::submitter::{PoolBatchSubmitter, TxPool};
use super::{start_clearing_bank_components, KeystoreBatchSigner};

/// 根据 CLI 参数启动清算行 offchain 运行组件。
///
/// 返回值为可注入 JSON-RPC 的 `OffchainClearingRpcImpl`。如果未指定清算行、
/// 地址无效或组件启动失败,返回 `None`,普通 PoW + GRANDPA 节点继续运行。
pub(crate) fn start_from_cli(
    clearing_bank: Option<&str>,
    clearing_bank_password: Option<&str>,
    reserve_monitor_interval_secs: Option<u64>,
    base_path: &Path,
    client: Arc<crate::core::service::FullClient>,
    transaction_pool: Arc<TxPool>,
    task_manager: &TaskManager,
) -> Option<Arc<OffchainClearingRpcImpl>> {
    let Some(bank_ss58) = clearing_bank else {
        return None;
    };

    let bank_main = match AccountId32::from_ss58check(bank_ss58) {
        Ok(account) => account,
        Err(e) => {
            log::warn!("[ClearingBank] --clearing-bank SS58 解析失败:{e:?},清算行组件不启动");
            return None;
        }
    };

    let password = clearing_bank_password.unwrap_or("");
    let signing_key_slot: Arc<RwLock<Option<SigningKey>>> = Arc::new(RwLock::new(None));
    let keystore = OffchainKeystore::new(base_path);

    if keystore.has_signing_key() && !password.is_empty() {
        match keystore.load_signing_key(password) {
            Ok(key) => {
                *signing_key_slot.write().expect("lock") = Some(key);
                log::info!("[ClearingBank] 签名密钥已解锁");
            }
            Err(e) => {
                log::warn!("[ClearingBank] 签名密钥解锁失败:{e},packer 将拒绝提交 extrinsic");
            }
        }
    } else {
        log::warn!(
            "[ClearingBank] 签名密钥未加载(密码或密钥文件缺失),packer 会在有 pending 时 rollback"
        );
    }

    let signer: Arc<dyn BatchSigner> = Arc::new(KeystoreBatchSigner::new(signing_key_slot.clone()));
    let submitter: Arc<dyn BatchSubmitter> = Arc::new(PoolBatchSubmitter::new(
        client.clone(),
        transaction_pool,
        signing_key_slot,
    ));

    let components = match start_clearing_bank_components(
        base_path,
        bank_main.clone(),
        password,
        signer,
        submitter,
        client.clone(),
    ) {
        Ok(components) => components,
        Err(e) => {
            log::warn!("[ClearingBank] 组件启动失败:{e}");
            return None;
        }
    };

    spawn_packer_worker(task_manager, client.clone(), components.packer.clone());
    spawn_listener_worker(
        task_manager,
        client.clone(),
        components.event_listener.clone(),
    );
    spawn_reserve_worker(
        task_manager,
        client,
        components.reserve_monitor.clone(),
        reserve_monitor_interval_secs,
    );

    log::info!(
        "[ClearingBank] 清算行组件已启动,bank_main={}",
        bank_main.to_ss58check()
    );
    Some(components.rpc_impl.clone())
}

fn spawn_packer_worker(
    task_manager: &TaskManager,
    client: Arc<crate::core::service::FullClient>,
    packer: Arc<super::settlement::packer::OffchainPacker>,
) {
    task_manager
        .spawn_handle()
        .spawn("offchain-clearing-packer", Some("offchain"), async move {
            // HeaderBackend/SaturatedConversion 只在 worker 内使用,放在这里避免污染 service.rs。
            use sp_blockchain::HeaderBackend as _;
            use sp_runtime::traits::SaturatedConversion as _;

            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let info = client.info();
                let current_block: u64 = info.best_number.saturated_into();
                if packer.should_pack(current_block).await {
                    match packer.pack_and_submit(current_block).await {
                        Ok(Some(hash)) => log::info!("[ClearingPacker] batch ok tx=0x{:x}", hash),
                        Ok(None) => {}
                        Err(e) => log::warn!("[ClearingPacker] {e}"),
                    }
                }
            }
        });
}

fn spawn_listener_worker(
    task_manager: &TaskManager,
    client: Arc<crate::core::service::FullClient>,
    listener: Arc<super::settlement::listener::EventListener>,
) {
    task_manager.spawn_handle().spawn(
        "offchain-clearing-event-listener",
        Some("offchain"),
        async move {
            listener.run(client).await;
        },
    );
}

fn spawn_reserve_worker(
    task_manager: &TaskManager,
    client: Arc<crate::core::service::FullClient>,
    monitor: Arc<super::reserve::ReserveMonitor>,
    reserve_monitor_interval_secs: Option<u64>,
) {
    let monitor_interval_secs = reserve_monitor_interval_secs.unwrap_or(300);
    if monitor_interval_secs == 0 {
        log::warn!(
            "[ClearingBank] reserve monitor 已关闭(interval=0),仅用于排障,生产环境请保留默认 300 秒"
        );
        return;
    }

    task_manager.spawn_handle().spawn(
        "offchain-clearing-reserve-monitor",
        Some("offchain"),
        async move {
            monitor
                .run(client, Duration::from_secs(monitor_interval_secs))
                .await;
        },
    );
}
