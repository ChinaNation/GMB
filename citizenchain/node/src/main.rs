//! 公民链节点 — 区块链节点 + 桌面界面合一。
//!
//! 无参数启动：打开桌面窗口 + 区块链节点（用户使用）。
//! 有子命令启动：运行工具命令后退出（CI/开发者使用，如 build-spec）。

#![warn(missing_docs)]

mod benchmarking;
mod chain_spec;
mod cli;
mod command;
#[cfg(feature = "gpu-mining")]
mod gpu_miner;
mod rpc;
mod service;
mod tls_cert;
mod ui;

fn main() {
    // 检测命令行参数：有子命令走 CLI 工具路径，无子命令走桌面 GUI。
    let args: Vec<String> = std::env::args().collect();
    let has_subcommand = args.len() > 1 && !args[1].starts_with('-');

    if has_subcommand {
        // CLI 工具命令（build-spec、purge-chain、export-blocks 等）。
        if let Err(e) = command::run() {
            eprintln!("{e}");
            std::process::exit(1);
        }
    } else {
        // 桌面应用：Tauri 窗口 + 内嵌区块链节点。
        ui::run_desktop();
    }
}
