//! 公民币全链统一常量模块=primitives/src
//! 所有 Pallet、runtime、chain_spec、node、fcrcnode、fullnode、wuminapp等均可安全引用。

#![cfg_attr(not(feature = "std"), no_std)]

#[path = "../china/mod.rs"]
pub mod china; // 中国制度常量目录（primitives/china）
pub mod citizen_const; // 公民轻节点发行常量
pub mod core_const; // 核心常量
pub mod count_const; // 投票治理常量
pub mod genesis; // 创世宣言与创世发行常量
pub mod pow_const; // 全节点铸块与发行常量
pub mod reserve_nodes_const; // 国储会 + 43个省储会节点常量
pub mod sheng_code;
pub mod shengbank_nodes_const; // 43个省储行节点常量 // 省份代码常量
