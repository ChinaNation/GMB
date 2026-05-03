//! 个人多签业务分区。
//!
//! 个人多签由用户创建,无 SFID 归属,以 `creator + account_name` 派生地址。
//! 本子模块包含个人多签的类型定义、创建/关闭业务实现、投票回调分支。

pub mod close;
pub mod create;
pub mod execute;
pub mod types;

pub use types::{
    CloseDuoqianAction, CreateDuoqianAction, DuoqianAccount, DuoqianStatus, PersonalDuoqianMeta,
};
