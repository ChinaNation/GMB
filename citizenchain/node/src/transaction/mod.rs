//! 交易模块桌面端聚合目录。
//!
//! 与 runtime/transaction/ 边界对齐，承载链上链下交易、机构多签账户、多签交易相关功能。
//! multisig_transfer：多签转账模块，用于机构/个人的多签转账；
//! offchain_transaction：链下支付模块，与清算行系统对接的个人链下支付方；
//! onchain_transaction：链上支付模块，统一的链上支付；

pub mod multisig_transfer;
pub mod offchain_transaction;
pub mod onchain_transaction;
