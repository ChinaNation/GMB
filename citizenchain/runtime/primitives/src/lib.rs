//! 全链统一常量模块=primitives
//! 所有 Pallet、runtime、chain_spec、node、citizenapp等均可安全引用。

#![cfg_attr(not(feature = "std"), no_std)]

#[path = "../china/mod.rs"]
pub mod china; // 机构常量
pub mod citizen_const; // 公民发行常量
pub mod core_const; // 核心常量
pub mod count_const; // 投票治理常量
pub mod fee_policy; // 费率规则常量(链上/链下/投票/分账)
pub mod genesis; // 创世常量
pub mod multisig; // 多签治理跨 pallet 共用 trait + 类型
pub mod pow_const; // 全节点铸块与发行常量
