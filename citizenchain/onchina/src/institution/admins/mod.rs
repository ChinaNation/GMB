//! 机构管理员链下私密资料子模块。
//!
//! 管理员姓名/职务/任期/cid/来源属链上 AdminProfile(ADR-030/A2),不落本库;
//! 本子模块只承接链下私密档案(部门/岗位/联系方式/证件照/passkey 绑定)与链投影,
//! 落库到 `institution_admins` 省级分区表。控制台登录元数据走独立的 `admins` 表,与此无关。

pub(crate) mod model;
pub(crate) mod repo;

#[allow(unused_imports)]
pub(crate) use model::InstitutionAdmin;
