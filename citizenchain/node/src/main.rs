//! 公民链节点 — 区块链节点 + 桌面界面合一。
//!
//! 自动适应环境：
//! - 有显示器：打开桌面窗口 + 区块链节点
//! - 无显示器（服务器）：直接运行区块链节点
//! - 有子命令（build-spec 等）：运行工具命令后退出

#![warn(missing_docs)]

mod benchmarking;
mod chain_spec;
mod cli;
mod command;
#[cfg(feature = "gpu-mining")]
mod gpu_miner;
mod offchain_gossip;
mod offchain_keystore;
mod offchain_ledger;
mod offchain_packer;
mod rpc;
mod service;
mod tls_cert;
mod ui;

fn main() {
    // 有子命令（build-spec、purge-chain 等）→ CLI 工具模式
    let args: Vec<String> = std::env::args().collect();
    let has_subcommand = args.len() > 1 && !args[1].starts_with('-');
    if has_subcommand {
        if let Err(e) = command::run() {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

    // 调试用逃生口：CITIZENCHAIN_HEADLESS=1 强制无头模式（绕过 GUI），
    // 用来在另一个端口/数据目录跑诊断节点，不影响桌面 GUI 实例。
    if std::env::var("CITIZENCHAIN_HEADLESS").is_ok() {
        eprintln!("CITIZENCHAIN_HEADLESS 已设置，以无头模式运行节点...");
        if let Err(e) = command::run() {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

    // 检测是否有显示环境（Linux 看 DISPLAY/WAYLAND_DISPLAY，macOS/Windows 始终有）
    if has_display() {
        // 有显示器 → 桌面窗口 + 内嵌节点
        ui::run_desktop();
    } else {
        // 无显示器（服务器）→ 直接运行节点
        eprintln!("未检测到显示环境，以无窗口模式运行节点...");
        if let Err(e) = command::run() {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

/// 检测当前环境是否有显示器。
fn has_display() -> bool {
    if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        return true;
    }
    // Linux：检查 DISPLAY 或 WAYLAND_DISPLAY
    std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
}
