//! 清算行扫码支付收单与本地交易账本。
//!
//! 中文注释:
//! - 本目录负责 wuminapp 钱包绑定清算行、扫码支付提交、本地 pending /
//!   confirmed 账本与清算行节点声明。
//! - 真正把批次发送到链上的后台任务位于 `settlement` 目录。

pub mod commands;
pub mod endpoint;
pub mod health;
pub mod ledger;
pub mod rpc;
pub mod signing;
