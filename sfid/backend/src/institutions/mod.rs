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
//! - [`model`]    — 结构体、DTO
//! - [`store`]    — cache entry 读写层
//! - [`service`]  — 业务校验、唯一性、分类
//! - [`chain`]    — 链交互(搬自 sheng-admins/institutions.rs 的 submit 函数)
//! - [`handler`]  — HTTP handler
//!
//! 铁律见 `feedback_institutions_two_layer.md`。

#![allow(unused_imports)]
#![allow(dead_code)]

pub mod chain;
pub mod handler;
pub mod model;
pub mod service;
pub mod store;

pub use model::{
    account_key_from_string, account_key_to_string, AccountKey, CreateAccountInput,
    CreateAccountOutput, CreateInstitutionInput, CreateInstitutionOutput, InstitutionDetailOutput,
    InstitutionListRow, MultisigAccount, MultisigInstitution,
};
