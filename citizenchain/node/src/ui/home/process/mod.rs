//! 节点进程管理子模块：在进程内启动/停止 Substrate 节点。
//!
//! 替代旧的子进程模式——不再启动外部二进制，直接在 Tauri 进程内运行 Substrate 节点。

use crate::ui::{
    settings::{device_password, grandpa_address},
    shared::{keystore, rpc, security},
};
use crate::ui::node_runner::{self, NodeHandle};
use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Manager};

use super::identity::{current_status, NodeStatus};

// 串行化节点启停，避免并发命令冲突。
static NODE_LIFECYCLE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// 进程托管状态。
pub struct RuntimeState {
    /// 进程内运行的 Substrate 节点句柄。
    pub node_handle: Option<NodeHandle>,
    /// 链下清算账本（启动时由 service 注入，用于停止前检查待上链交易）。
    pub offchain_ledger: Option<crate::offchain_ledger::OffchainLedger>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            node_handle: None,
            offchain_ledger: None,
        }
    }
}

/// Tauri 全局状态，供节点相关命令共享。
pub struct AppState(pub Mutex<RuntimeState>);

pub(super) fn lock_node_lifecycle() -> std::sync::MutexGuard<'static, ()> {
    NODE_LIFECYCLE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

pub(super) fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    keystore::node_data_dir(app)
}

/// 启动时清理（进程内模式下只需清理 RPC 缓存）。
pub(crate) fn cleanup_on_startup(app: &AppHandle) {
    let _ = app; // 不再需要杀孤儿进程或清理临时文件
    rpc::clear_genesis_hash_cache();
}

/// 退出时清理：停止进程内节点。
pub(crate) fn cleanup_on_exit(app: &AppHandle) {
    let _lifecycle_guard = lock_node_lifecycle();
    if let Ok(mut state) = app.state::<AppState>().0.lock() {
        // drop NodeHandle 会终止节点线程。
        state.node_handle.take();
    }
}

fn verify_start_unlock_password(app: &AppHandle, unlock_password: &str) -> Result<(), String> {
    let unlock = security::ensure_unlock_password(unlock_password)?;
    device_password::verify_device_login_password(app, unlock)?;
    Ok(())
}

fn start_node_sync(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    let _lifecycle_guard = lock_node_lifecycle();
    if let Err(e) = security::append_audit_log(&app, "start_node", "attempt") {
        eprintln!("[审计] start_node attempt 日志写入失败: {e}");
    }
    let result = (|| -> Result<NodeStatus, String> {
        let unlock_password = security::ensure_unlock_password(&unlock_password)?.to_string();
        verify_start_unlock_password(&app, &unlock_password)?;

        // 停止已有节点（如果在运行）。
        {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            state.node_handle.take(); // drop 停止旧节点
        }

        rpc::clear_genesis_hash_cache();
        thread::sleep(Duration::from_millis(250));

        // 中文注释：检测签名管理员，有则解密并创建链下清算配置。
        {
            let app_data = security::app_data_dir(&app)?;
            let ks = crate::offchain_keystore::OffchainKeystore::new(&app_data);
            if ks.has_signing_key() {
                match ks.load_signing_key(&unlock_password) {
                    Ok(signing_key) => {
                        let ledger = crate::offchain_ledger::OffchainLedger::new(&app_data);
                        match ledger.load_from_disk(&unlock_password) {
                            Ok(count) => {
                                log::info!("[Offchain] 恢复 {count} 笔待上链交易");
                            }
                            Err(e) => {
                                log::warn!("[Offchain] 账本恢复失败（首次启动或密码不匹配）：{e}");
                            }
                        }
                        // 存入全局配置供 service 读取
                        crate::service::set_offchain_config(Some(
                            crate::service::OffchainConfig {
                                ledger: ledger.clone(),
                                shenfen_id: signing_key.shenfen_id.clone(),
                            },
                        ));
                        // 存入 AppState 供 stop_node 检查
                        if let Ok(mut state) = app.state::<AppState>().0.lock() {
                            state.offchain_ledger = Some(ledger);
                        }
                        log::info!(
                            "[Offchain] 签名管理员已加载（{}），链下清算功能已启用",
                            signing_key.shenfen_id
                        );
                    }
                    Err(e) => {
                        log::warn!("[Offchain] 签名管理员解密失败：{e}");
                        crate::service::set_offchain_config(None);
                    }
                }
            } else {
                crate::service::set_offchain_config(None);
                log::info!("[Offchain] 未检测到签名管理员，链下清算未启用");
            }
        }

        // 准备启动参数。
        let base_path = node_data_dir(&app)?;
        let rpc_port = rpc::current_rpc_port();
        let enable_grandpa_validator = grandpa_address::prepare_grandpa_for_start(&app)?;
        let mining_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // 在进程内启动 Substrate 节点。
        let handle = node_runner::start_node_in_process(
            base_path,
            rpc_port,
            None, // 节点名称已移除，不再使用
            enable_grandpa_validator,
            mining_threads,
            None, // gpu_device
        )?;

        // 存储句柄。
        {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            state.node_handle = Some(handle);
        }

        // 等待 RPC 就绪。
        thread::sleep(Duration::from_millis(2000));

        // 验证 GRANDPA 配置。
        if let Err(err) = grandpa_address::verify_grandpa_after_start(&app) {
            // 回滚：停止节点。
            if let Ok(mut state) = app.state::<AppState>().0.lock() {
                state.node_handle.take();
            }
            return Err(format!("verify grandpa after start failed: {err}"));
        }

        current_status(&app)
    })();
    if let Err(e) = security::append_audit_log(
        &app,
        "start_node",
        if result.is_ok() { "success" } else { "failed" },
    ) {
        eprintln!("[审计] start_node 结果日志写入失败: {e}");
    }
    result
}

fn stop_node_sync(app: AppHandle) -> Result<NodeStatus, String> {
    let _lifecycle_guard = lock_node_lifecycle();
    if let Err(e) = security::append_audit_log(&app, "stop_node", "attempt") {
        eprintln!("[审计] stop_node attempt 日志写入失败: {e}");
    }
    let result = (|| -> Result<NodeStatus, String> {
        // 检查链下账本是否有待上链交易，如有则拒绝停止
        {
            let app_state = app.state::<AppState>();
            let state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            if let Some(ref ledger) = state.offchain_ledger {
                let pending = ledger.pending_count();
                if pending > 0 {
                    return Err(format!(
                        "有 {} 笔交易待上链，不允许停止节点",
                        pending
                    ));
                }
            }
        }

        {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            state.node_handle.take(); // drop 停止节点
        }

        rpc::clear_genesis_hash_cache();
        thread::sleep(Duration::from_millis(500));
        current_status(&app)
    })();
    if let Err(e) = security::append_audit_log(
        &app,
        "stop_node",
        if result.is_ok() { "success" } else { "failed" },
    ) {
        eprintln!("[审计] stop_node 结果日志写入失败: {e}");
    }
    result
}

#[tauri::command]
pub async fn start_node(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    let app2 = app.clone();
    let status = super::join_blocking_task(
        "start_node",
        tauri::async_runtime::spawn_blocking(move || start_node_sync(app, unlock_password)),
    )
    .await?;

    // 节点启动成功后，异步同步本地已保存的奖励钱包绑定到链上。
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if let Err(err) =
            crate::ui::settings::fee_address::sync_saved_reward_wallet_inner(&app2).await
        {
            eprintln!("[reward-wallet] 启动后自动同步链上绑定失败: {err}");
        }
    });

    Ok(status)
}

#[tauri::command]
pub async fn stop_node(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    super::join_blocking_task(
        "stop_node",
        tauri::async_runtime::spawn_blocking(move || {
            let unlock = security::ensure_unlock_password(&unlock_password)?;
            device_password::verify_device_login_password(&app, unlock)?;
            stop_node_sync(app)
        }),
    )
    .await
}

pub(crate) fn start_node_blocking(
    app: AppHandle,
    unlock_password: String,
) -> Result<NodeStatus, String> {
    start_node_sync(app, unlock_password)
}

pub(crate) fn stop_node_blocking(app: AppHandle) -> Result<NodeStatus, String> {
    stop_node_sync(app)
}
