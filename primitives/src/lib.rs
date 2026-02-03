//! 公民币全链统一常量模块（primitives/constants）
//! 所有 Pallet、runtime、chain_spec、node 均可安全引用。
//! 不允许依赖 runtime，以避免循环依赖。

pub mod citizen_const;              // 公民轻节点发行常量
pub mod constants;                  // 创世宣言
pub mod core_const;                 // 核心常量
pub mod count_const;                // 投票治理常量
pub mod pow_const;                  // 全节点铸块与发行常量
pub mod reserve_nodes_const;        // 国储会 + 43 个省储会节点的常量
pub mod shengbank_nodes_const;      // 省储行节点的常量
pub mod shengbank_stakes_const;     // 省储行质押的常量