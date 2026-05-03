//! 机构/账户两层数据模型 + CRUD + 链交互
//!
//! 中文注释:任务卡 2 的核心模块。链端 `SfidRegisteredAddress::<T>(sfid_id, name)`
//! 是 DoubleMap,sfid 系统这里拆两层对齐:
//!
//! - `MultisigInstitution`:每个 sfid_id 唯一,存机构展示信息,不进链
//! - `MultisigAccount`:(sfid_id, account_name) 复合 key,account_name 就是链上 name,进链
//!
//! ## 子模块
//!
//! - [`model`]              — 机构、账户、资料库结构体与 DTO
//! - [`store`]              — cache entry 读写层
//! - [`service`]            — 机构本地业务校验、唯一性、分类
//! - [`handler`]            — 机构本地 HTTP handler
//! - [`chain_duoqian_info`] — 机构信息提供给区块链/钱包查询的入口
//!
//! 铁律见 `feedback_institutions_two_layer.md`。

#![allow(unused_imports)]
#![allow(dead_code)]

/// 中文注释:机构模块与区块链/钱包交互的唯一入口,文件名按 `chain_` 规则固定。
pub mod chain_duoqian_info;
pub mod derive;
pub mod handler;
pub mod model;
pub mod service;
pub mod store;

pub use model::{
    account_key_from_string, account_key_to_string, AccountKey, ChainSyncAccountInput,
    ChainSyncInput, ChainSyncOutput, CreateAccountInput, CreateAccountOutput,
    CreateInstitutionInput, CreateInstitutionOutput, InstitutionChainStatus,
    InstitutionDetailOutput, InstitutionDocument, InstitutionListRow, MultisigAccount,
    MultisigChainStatus, MultisigInstitution, ParentInstitutionRow, UpdateInstitutionInput,
    VALID_DOC_TYPES,
};
