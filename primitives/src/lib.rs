//! 公民币 GMB 全链统一常量模块（primitives/constants）
//! 所有 Pallet、runtime、chain_spec、node 均可安全引用。
//! 不允许依赖 runtime，以避免循环依赖。

pub mod citizen_const;
pub mod core_const;
pub mod count_const;
pub mod pow_const;
pub mod reserve_nodes_const;
pub mod shengbank_nodes_const;
pub mod shengbank_stakes_const;