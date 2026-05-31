/// 中文注释:管理员安全动作入口:省级治理直接 apply,业务写操作签发一次性 grant。
pub(crate) mod actions;
pub(crate) mod catalog;
pub(crate) mod operators;
/// 中文注释:省管理员 main/backup 槽位、公钥与省份归属清单。
pub(crate) mod province_admins;

pub(crate) use catalog::list_province_admins;
pub(crate) use operators::list_operators;
