//! 进程内 Substrate 节点管理。
//!
//! 替代旧的子进程模式（spawn_node）。在 Tauri 进程内直接启动 Substrate 节点服务，
//! 不再需要外部二进制、SHA256 校验、子进程管理。

use std::path::PathBuf;
use std::thread::JoinHandle;

/// 节点运行句柄。节点在后台线程中运行，drop 时通过信号停止。
pub struct NodeHandle {
    _thread: JoinHandle<()>,
}

/// 在进程内启动 Substrate 节点。
///
/// 节点在单独的线程中运行（因为 `run_node_until_exit` 会阻塞）。
/// 返回 NodeHandle 用于生命周期管理。
pub fn start_node_in_process(
    base_path: PathBuf,
    rpc_port: u16,
    node_name: Option<String>,
    validator: bool,
    mining_threads: usize,
    gpu_device: Option<usize>,
) -> Result<NodeHandle, String> {
    // 构造 CLI 参数（和之前 spawn_node 传的命令行参数一致）。
    let mut args: Vec<String> = vec![
        "node".into(),
        "--base-path".into(),
        base_path.display().to_string(),
        "--chain".into(),
        "citizenchain".into(),
        "--listen-addr".into(),
        "/ip4/0.0.0.0/tcp/30333/wss".into(),
        "--listen-addr".into(),
        "/ip6/::/tcp/30333/wss".into(),
        "--rpc-port".into(),
        rpc_port.to_string(),
        "--rpc-methods".into(),
        "Unsafe".into(),
        "--rpc-cors".into(),
        "all".into(),
        "--no-prometheus".into(),
    ];

    // 节点身份密钥文件（如果存在）。
    let node_key_file = base_path.join("node-key").join("secret_ed25519");
    if node_key_file.is_file() {
        args.push("--node-key-file".into());
        args.push(node_key_file.display().to_string());
    }

    if let Some(name) = node_name {
        args.push("--name".into());
        args.push(name);
    }

    if validator {
        args.push("--validator".into());
    }

    // 用于检测启动是否成功。
    let (error_tx, error_rx) = std::sync::mpsc::channel::<String>();

    let thread = std::thread::Builder::new()
        .name("substrate-node".into())
        .spawn(move || {
            // 设置 SS58 地址前缀。
            let _ = sp_core::crypto::set_default_ss58_version(
                sp_core::crypto::Ss58AddressFormat::custom(primitives::core_const::SS58_FORMAT),
            );

            // 解析 CLI 参数。
            use clap::Parser;
            use sc_cli::SubstrateCli;
            let cli = match crate::cli::Cli::try_parse_from(&args) {
                Ok(c) => c,
                Err(e) => {
                    let _ = error_tx.send(format!("解析节点参数失败: {e}"));
                    return;
                }
            };

            let runner = match cli.create_runner(&cli.run) {
                Ok(r) => r,
                Err(e) => {
                    let _ = error_tx.send(format!("创建 runner 失败: {e}"));
                    return;
                }
            };

            // run_node_until_exit 阻塞直到节点退出。
            // 它内部创建 tokio runtime 并管理 TaskManager。
            if let Err(e) = runner.run_node_until_exit(|config| async move {
                crate::service::new_full(config, mining_threads, gpu_device)
                    .map_err(sc_cli::Error::Service)
            }) {
                let _ = error_tx.send(format!("节点运行失败: {e}"));
            }
        })
        .map_err(|e| format!("启动节点线程失败: {e}"))?;

    // 等待一段时间检查是否有启动错误。
    std::thread::sleep(std::time::Duration::from_secs(5));

    match error_rx.try_recv() {
        Ok(err) => Err(err),
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            // 没有错误，节点正在运行。
            Ok(NodeHandle { _thread: thread })
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            // 发送端已 drop，说明线程正常运行中（没有发送错误就 drop 了 tx）。
            Ok(NodeHandle { _thread: thread })
        }
    }
}
