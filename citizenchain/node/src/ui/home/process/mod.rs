//! 节点进程管理子模块：在进程内启动/停止 Substrate 节点。
//!
//! 替代旧的子进程模式——不再启动外部二进制，直接在 Tauri 进程内运行 Substrate 节点。
//!
//! 2026-04-25 起：节点生命周期与 App 进程绑定。
//! - App 启动 → `start_node_blocking`（在 `ui::run_desktop` 的 setup 后台线程中触发）
//! - App 退出 → `cleanup_on_exit`（`RunEvent::Exit` 触发）
//! - 关窗 = 最小化（在 `ui::run_desktop` 的 `WindowEvent::CloseRequested` 拦截）
//! - 不再暴露 `start_node` / `stop_node` Tauri command，前端无启停按钮、无密码框。
//! - `start_node_blocking` 仍被设置页 `set_grandpa_key` / `set_bootnode_key` 复用做"保存即重启"。

use crate::ui::node_runner::{self, NodeHandle};
use crate::ui::{
    settings::grandpa_address,
    shared::{keystore, rpc, security},
};
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
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self { node_handle: None }
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
/// 句柄 drop 内部会发送 shutdown 信号 + join 后台线程，
/// 等待 RocksDB LOCK 真正释放后才返回。
pub(crate) fn cleanup_on_exit(app: &AppHandle) {
    let _lifecycle_guard = lock_node_lifecycle();
    // 先把 handle 取出再 drop，避免在持有 state 锁期间 join 线程。
    let old_handle = match app.state::<AppState>().0.lock() {
        Ok(mut state) => state.node_handle.take(),
        Err(_) => None,
    };
    drop(old_handle);
}

fn start_node_sync(app: AppHandle) -> Result<NodeStatus, String> {
    let _lifecycle_guard = lock_node_lifecycle();
    if let Err(e) = security::append_audit_log(&app, "start_node", "attempt") {
        eprintln!("[审计] start_node attempt 日志写入失败: {e}");
    }
    let result = (|| -> Result<NodeStatus, String> {
        // 停止已有节点（如果在运行）。
        // 取出后立即释放 state 锁再 drop，避免 drop（join 线程）期间阻塞 get_node_status 等查询。
        let old_handle = {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            state.node_handle.take()
        };
        drop(old_handle);

        rpc::clear_genesis_hash_cache();
        thread::sleep(Duration::from_millis(250));

        // 扫码支付 Step 2b-iv-a 清理:原"检测签名管理员 → 加载旧 offchain_ledger →
        // 注入 set_offchain_config"的老省储行清算切入已删除(ADR-006 省储行退出清算)。
        // UI 模式下 `node_runner.rs` 传 `None,None,None` 到 `new_full`,不启动清算行
        // 组件,启动路径仅承担 PoW + GRANDPA 基础职能;生产清算行节点通过 CLI 的
        // `--clearing-bank` 启动。

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
            let bad_handle = match app.state::<AppState>().0.lock() {
                Ok(mut state) => state.node_handle.take(),
                Err(_) => None,
            };
            drop(bad_handle);
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
        // 扫码支付 Step 2b-iv-a 清理:原老省储行清算的 "pending_count > 0 拒绝停止"
        // 保护已删除。UI 模式暂不启动清算行组件(通过 CLI `--clearing-bank` 启动),
        // 该保护对 UI 运行时一直是死代码;CLI 模式下的 graceful shutdown + pending
        // 检查留给 Step 3 独立任务实现(需要把 OffchainComponents 挂到 task_manager
        // 的 spawn_essential_handle 生命周期而非全局 static)。
        let old_handle = {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            state.node_handle.take()
        };
        drop(old_handle);

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

/// 启动节点（同步阻塞调用）。供 `ui::run_desktop` 的 setup 自动启动以及
/// 设置页 `set_grandpa_key` / `set_bootnode_key` 的"保存即重启"复用。
pub(crate) fn start_node_blocking(app: AppHandle) -> Result<NodeStatus, String> {
    start_node_sync(app)
}

/// 停止节点（同步阻塞调用）。供设置页"保存即重启"流程复用。
pub(crate) fn stop_node_blocking(app: AppHandle) -> Result<NodeStatus, String> {
    stop_node_sync(app)
}
