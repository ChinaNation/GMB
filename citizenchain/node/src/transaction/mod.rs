//! 交易模块桌面端聚合目录。
//!
//! 与 runtime/transaction/ 边界对齐，承载链上链下交易、机构多签账户、多签交易相关功能。

pub mod multisig_transfer;
pub mod offchain_transaction;
pub mod onchain_transaction;
