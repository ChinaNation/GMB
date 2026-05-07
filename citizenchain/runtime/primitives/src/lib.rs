//! 公民币全链统一常量模块=primitives/src
//! 所有 Pallet、runtime、chain_spec、node、fullnode、wuminapp等均可安全引用。

#![cfg_attr(not(feature = "std"), no_std)]

#[path = "../china/mod.rs"]
pub mod china; // 机构常量
pub mod citizen_const; // 公民发行常量
pub mod core_const; // 核心常量
pub mod count_const; // 投票治理常量
pub mod derive; // 治理主体 ID 派生函数(subject_id_from_account / subject_id_from_sfid_id)
pub mod fee_policy; // 费率规则常量(链上/链下/投票/分账)
pub mod genesis; // 创世常量
pub mod pow_const; // 全节点铸块与发行常量
pub mod traits; // 跨 pallet 共用的地址校验 / 资金保护抽象
pub mod types; // 跨 pallet 共用的轻量数据类型(MultisigConfigSnapshot 等)
