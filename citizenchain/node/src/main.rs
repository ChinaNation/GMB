//! 公民链节点 — 区块链节点 + 桌面界面合一。
//!
//! 启动后同时运行 Tauri 桌面窗口和 Substrate 区块链节点。
//! 用户点击图标即可进入区块链。

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
    // 启动桌面应用（内嵌 Substrate 区块链节点）。
    ui::run_desktop();
}
