//! 机构管理员链下私密资料子模块。
//!
//! 管理员链上身份只有钱包账户；岗位、任期和来源由 entity 模块的任职关系表达。
//! 本子模块只承接链下私密档案(部门/联系方式/证件照/passkey 绑定)与链投影,
//! 落库到 `institution_admins` 省级分区表。控制台登录元数据走独立的 `admins` 表,与此无关。

pub(crate) mod chain_roles;
pub(crate) mod model;
pub(crate) mod repo;

#[allow(unused_imports)]
pub(crate) use model::InstitutionAdmin;
