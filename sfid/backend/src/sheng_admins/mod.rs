/// 中文注释:省管理员首登 bootstrap(seed 落盘 + cache 载入,推链留 Phase 4)。
pub(crate) mod bootstrap;
pub(crate) mod catalog;
pub(crate) mod institutions;
pub(crate) mod operators;
/// 中文注释:省管理员 main/backup 槽位、公钥与省份归属清单。
pub(crate) mod province_admins;
/// 中文注释:3-tier 名册 service(add/remove backup,推链全部 mock)。
pub(crate) mod roster;
/// 中文注释:省管理员 3-tier 签名 keypair 进程内缓存(ADR-008)。
pub(crate) mod signing_cache;
/// 中文注释:省管理员签名 seed 加密持久化,由 bootstrap 读写。
pub(crate) mod signing_seed_store;

pub(crate) use catalog::{list_sheng_admins, replace_sheng_admin};
pub(crate) use institutions::{
    archive_import, delete_cpms_keys, disable_cpms_keys, enable_cpms_keys,
    generate_cpms_institution_sfid_qr, get_cpms_site_by_institution, list_cpms_keys, register_cpms,
    reissue_install_token, revoke_cpms_keys, revoke_install_token,
};
pub(crate) use operators::{
    create_operator, delete_operator, list_operators, update_operator, update_operator_status,
};
