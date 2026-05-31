/// 中文注释:管理员安全动作入口:省级治理直接 apply,业务写操作签发一次性 grant。
pub(crate) mod actions;
pub(crate) mod catalog;
/// 中文注释:管理端操作权限分级唯一入口。
pub(crate) mod operation_auth;
pub(crate) mod operators;
/// 中文注释:管理员 Passkey 注册与 WebAuthn 凭据校验工具。
pub(crate) mod passkeys;
/// 中文注释:内置初始省管理员公钥与省份归属清单。
pub(crate) mod province_admins;

pub(crate) use catalog::list_province_admins;
pub(crate) use operators::list_operators;
