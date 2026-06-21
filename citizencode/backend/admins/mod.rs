/// 中文注释:管理员安全动作入口:注册局管理员治理直接 apply,业务写操作签发一次性 grant。
pub(crate) mod actions;
pub(crate) mod catalog;
pub(crate) mod city_registry_admins;
/// 中文注释:管理员登录认证能力,归入 admins 边界。
pub(crate) mod login;
/// 中文注释:联邦注册局管理员/市注册局管理员实体、角色和列表 DTO。
pub(crate) mod model;
/// 中文注释:管理端操作权限分级唯一入口。
pub(crate) mod operation_auth;
/// 中文注释:管理员 Passkey 注册与 WebAuthn 凭据校验工具。
pub(crate) mod passkeys;
/// 中文注释:管理员结构化表读写，唯一持久化入口。
pub(crate) mod repo;
/// 中文注释:管理员 Passkey、公民钱包确认和一次性安全授权模型。
pub(crate) mod security_model;

pub(crate) use catalog::list_federal_registry_admins;
pub(crate) use city_registry_admins::list_city_registry_admins;
