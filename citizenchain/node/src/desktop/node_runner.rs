//! 进程内 Substrate 节点管理。
//!
//! 替代旧的子进程模式。在 Tauri 进程内直接启动 Substrate 节点服务。
//!
//! 关键约束：drop `NodeHandle` 必须真正终结后台 substrate 线程并释放
//! Backend（含 RocksDB LOCK）。否则同进程内"停 → 启"会撞
//! `lock hold by current process` 错误。
//! 实现办法：握有 shutdown oneshot + JoinHandle，Drop 时先发信号再 join。

use std::path::PathBuf;
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::sync::oneshot;

/// 节点运行句柄。drop 时会通知后台线程退出并 join。
pub struct NodeHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for NodeHandle {
    fn drop(&mut self) {
        // 发送停机信号；接收端可能已被 task_manager 退出路径丢弃，忽略错误。
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // join 线程，等待 task_manager + tokio runtime 完整 drop，
        // 确保 Substrate Backend 释放 RocksDB LOCK 后再返回。
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
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
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let thread = std::thread::Builder::new()
        .name("substrate-node".into())
        .spawn(move || {
            use clap::Parser;
            use sc_cli::CliConfiguration;

            // 设置 SS58 地址前缀。
            let _ = sp_core::crypto::set_default_ss58_version(
                sp_core::crypto::Ss58AddressFormat::custom(primitives::core_const::SS58_FORMAT),
            );

            let cli = match crate::core::cli::Cli::try_parse_from(&args) {
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
            // 扫码支付 Step 2b-ii-β-2-b / 2b-iii-b:UI 启动路径暂不支持清算行角色,
            // 全部透传 None(bank / password / reserve_monitor_interval);生产用户
            // 通过 CLI 的 `--clearing-bank` 进入无 UI 模式启动清算行节点。
            tokio_runtime.block_on(async {
                match crate::core::service::new_full(
                    config,
                    mining_threads,
                    gpu_device,
                    None,
                    None,
                    None,
                ) {
                    Ok(mut task_manager) => {
                        // drop error_tx 表示启动成功。
                        drop(error_tx);
                        // 退出条件二选一：essential task 失败 / 收到外部 shutdown 信号。
                        tokio::select! {
                            _ = task_manager.future() => {},
                            _ = shutdown_rx => {},
                        }
                        // 显式 drop task_manager 触发 Backend 释放（含 RocksDB LOCK）。
                        // 必须发生在 tokio_runtime 仍存活时，否则 drop 内的异步 cleanup 无法执行。
                        drop(task_manager);
                    }
                    Err(e) => {
                        let _ = error_tx.send(format!("节点启动失败: {e}"));
                    }
                }
            });
            // 等待 tokio runtime 内残余任务退出（包括 Backend flush）。
            tokio_runtime.shutdown_timeout(Duration::from_secs(10));
        })
        .map_err(|e| format!("启动节点线程失败: {e}"))?;

    // 等待一段时间检查是否有启动错误。
    std::thread::sleep(Duration::from_secs(5));

    match error_rx.try_recv() {
        Ok(err) => Err(err),
        Err(std::sync::mpsc::TryRecvError::Empty)
        | Err(std::sync::mpsc::TryRecvError::Disconnected) => Ok(NodeHandle {
            shutdown_tx: Some(shutdown_tx),
            thread: Some(thread),
        }),
    }
}
