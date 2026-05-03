//! 机构多签业务分区。
//!
//! 机构多签由 SFID 机构主体凭证发起,业务字段绑定 sfid_id + account_name,
//! 链上派生地址 `derive_institution_address(sfid_id, role)`。
//! 本子模块包含机构级类型、登记、创建、关闭、账户表与投票回调分支。

pub mod accounts;
pub mod close;
pub mod create;
pub mod execute;
pub mod register;
pub mod types;
pub mod vote;

pub use types::*;
