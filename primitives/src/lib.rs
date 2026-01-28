//! 公民币 GMB 全链统一常量模块（primitives/constants）
//! 所有 Pallet、runtime、chain_spec、node 均可安全引用。
//! 不允许依赖 runtime，以避免循环依赖。

pub mod governance;
pub mod agencies;
pub mod province;
pub mod pallet_ids;
pub mod pow;
pub mod balances;