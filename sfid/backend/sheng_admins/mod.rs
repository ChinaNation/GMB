pub(crate) mod catalog;
pub(crate) mod operators;
/// 中文注释:省管理员 main/backup 槽位、公钥与省份归属清单。
pub(crate) mod province_admins;
/// 中文注释:注册局-省级管理员页面的一主两备名册展示。
pub(crate) mod roster;
/// 中文注释:省管理员 3-tier 签名 keypair 进程内缓存(ADR-008)。
pub(crate) mod signing_cache;
/// 中文注释:省管理员本人签名密钥自动加载与手动生成/更换接口。
pub(crate) mod signing_keys;
/// 中文注释:省管理员签名 seed 加密持久化,由 signing_keys 读写。
pub(crate) mod signing_seed_store;

pub(crate) use catalog::list_sheng_admins;
pub(crate) use operators::{
    create_operator, delete_operator, list_operators, update_operator, update_operator_status,
};
