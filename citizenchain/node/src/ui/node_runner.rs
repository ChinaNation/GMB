//! 进程内 Substrate 节点管理。
//!
//! 替代旧的子进程模式。在 Tauri 进程内直接启动 Substrate 节点服务。

use std::path::PathBuf;
use std::thread::JoinHandle;

/// 节点运行句柄。节点在后台线程中运行。
pub struct NodeHandle {
    _thread: JoinHandle<()>,
}

/// 在进程内启动 Substrate 节点。
pub fn start_node_in_process(
    base_path: PathBuf,
    rpc_port: u16,
    node_name: Option<String>,
    validator: bool,
    mining_threads: usize,
    gpu_device: Option<usize>,
) -> Result<NodeHandle, String> {
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

    let (error_tx, error_rx) = std::sync::mpsc::channel::<String>();

    let thread = std::thread::Builder::new()
        .name("substrate-node".into())
        .spawn(move || {
            use clap::Parser;
            use sc_cli::{CliConfiguration, SubstrateCli};

            // 设置 SS58 地址前缀。
            let _ = sp_core::crypto::set_default_ss58_version(
                sp_core::crypto::Ss58AddressFormat::custom(primitives::core_const::SS58_FORMAT),
            );

            let cli = match crate::cli::Cli::try_parse_from(&args) {
                Ok(c) => c,
                Err(e) => {
                    let _ = error_tx.send(format!("解析节点参数失败: {e}"));
                    return;
                }
            };

            // 构建 tokio runtime（不用 create_runner，因为它会初始化日志导致冲突）。
            let tokio_runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = error_tx.send(format!("创建 tokio runtime 失败: {e}"));
                    return;
                }
            };

            // 直接构造 Configuration，跳过日志初始化。
            let config = match cli
                .run
                .create_configuration(&cli, tokio_runtime.handle().clone())
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = error_tx.send(format!("创建配置失败: {e}"));
                    return;
                }
            };

            // 在 tokio runtime 中启动节点服务。
            tokio_runtime.block_on(async {
                match crate::service::new_full(config, mining_threads, gpu_device) {
                    Ok(mut task_manager) => {
                        // drop error_tx 表示启动成功。
                        drop(error_tx);
                        // 等待节点退出。
                        task_manager.future().await.ok();
                    }
                    Err(e) => {
                        let _ = error_tx.send(format!("节点启动失败: {e}"));
                    }
                }
            });
        })
        .map_err(|e| format!("启动节点线程失败: {e}"))?;

    // 等待一段时间检查是否有启动错误。
    std::thread::sleep(std::time::Duration::from_secs(5));

    match error_rx.try_recv() {
        Ok(err) => Err(err),
        Err(std::sync::mpsc::TryRecvError::Empty) => Ok(NodeHandle { _thread: thread }),
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Ok(NodeHandle { _thread: thread }),
    }
}
