// 首页模块入口，按职责拆分为进程管理、RPC、身份管理三个子模块。

mod identity;
mod process;
mod rpc;

// 公共类型与 Tauri 命令，供 main.rs 直接使用。
pub use identity::{get_node_identity, get_node_status, set_node_name};
pub(crate) use process::{cleanup_on_exit, cleanup_on_startup};
pub use process::{start_node, stop_node, AppState, RuntimeState};
pub use rpc::get_chain_status;

// crate 内部使用的阻塞版本函数。
pub(crate) use identity::{current_status, get_node_identity_blocking};
pub(crate) use process::{start_node_blocking, stop_node_blocking};

async fn join_blocking_task<T>(
    task: &'static str,
    result: tauri::async_runtime::JoinHandle<Result<T, String>>,
) -> Result<T, String> {
    result
        .await
        .map_err(|e| format!("{task} join failed: {e}"))?
}
