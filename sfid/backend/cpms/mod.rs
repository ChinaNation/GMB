//! 中文注释:SFID 侧 CPMS 系统管理模块。
//!
//! 本模块承接 CPMS 站点安装二维码、QR2 注册、QR3 匿名证书、档案导入、
//! 站点状态治理等能力。它服务于公安局机构详情页,但业务归属是 CPMS 系统
//! 管理,不再混放在 `sheng_admins` 目录。

pub(crate) mod handler;
pub(crate) mod model;
/// 中文注释:CPMS 匿名证书 RSA 盲签名能力直接放在 CPMS 根目录,不再挂靠 institutions。
pub(crate) mod rsa_blind;
pub(crate) mod scope;

#[allow(unused_imports)]
pub(crate) use model::*;

pub(crate) use handler::{
    archive_import, delete_cpms_keys, disable_cpms_keys, enable_cpms_keys,
    generate_cpms_institution_sfid_qr, get_cpms_site_by_institution, list_cpms_keys, register_cpms,
    reissue_install_token, resolve_site_province_via_shard, revoke_cpms_keys, revoke_install_token,
    verify_sr25519_signature,
};
