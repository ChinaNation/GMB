//! 身份主体公共边界。
//!
//! 中文注释:`subjects` 只承接所有 SFID 身份共有的索引、详情入口与统一查询边界。
//! 公权机构业务放 `gov`,私权机构业务放 `private`,公民仍放 `citizens`。

pub(crate) mod admin;
pub(crate) mod chain_duoqian_info;
pub(crate) mod http;
pub(crate) mod model;
pub(crate) mod schema;
pub(crate) mod service;
pub(crate) mod uninorg;

#[allow(unused_imports)]
pub use model::{
    account_key_from_string, account_key_to_string, AccountKey, ChainSyncAccountInput,
    CreateAccountInput, CreateAccountOutput, CreateInstitutionInput, CreateInstitutionOutput,
    InstitutionDetailOutput, InstitutionDocument, InstitutionListRow, LegalRepresentativePhoto,
    Institution, InstitutionAccount, MultisigChainStatus, ParentInstitutionRow,
    UpdateInstitutionInput, VALID_DOC_TYPES,
};
