//! 机构生命周期类型统一出口。
//!
//! 定义已上提 `entity-primitives` 单源(公权/私权 pallet 逐字段一致),本模块仅 re-export,
//! 保持 `crate::institution::types::*` 与对外 `private_manage::{...}` API 不变。

pub use entity_primitives::{
    CloseInstitutionAction, CreateInstitutionAccount, InstitutionAccountInfo, InstitutionInfo,
    InstitutionInitialAccount, InstitutionLifecycleStatus, RegisteredInstitution,
};
