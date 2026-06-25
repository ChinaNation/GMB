//! 全链统一常量模块=primitives
//! 所有 Pallet、runtime、chain_spec、node、citizenapp等均可安全引用。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod account_derive; // 账户地址派生唯一真源(op_tag/保留名/路由/payload)
#[path = "../china/mod.rs"]
pub mod china; // 机构常量
pub mod citizen_const; // 公民发行常量
pub mod code; // 机构码链上表示 + 治理分类(全链机构分类唯一真源)
pub mod core_const; // 核心常量
pub mod count_const; // 投票治理常量
pub mod fee_policy; // 费率规则常量(链上/链下/投票/分账)
pub mod genesis; // 创世常量
pub mod multisig; // 多签治理跨 pallet 共用 trait + 类型
pub mod pow_const; // 全节点铸块与发行常量
pub mod sign; // 全仓签名消息唯一原语(signing_message + op_tag 注册表,ADR-026)
