//! 中文注释:SFID 侧 CPMS 系统管理模块。
//!
//! 本模块承接 CPMS 安装二维码、档案码验真、
//! 授权状态治理等能力。它服务于公安局机构详情页,但业务归属是 CPMS 系统
//! 管理,不再混放在 `sheng_admins` 目录。

pub(crate) mod handler;
pub(crate) mod model;
pub(crate) mod scope;

#[allow(unused_imports)]
pub(crate) use model::*;

pub(crate) use handler::{
    archive_verify, delete_cpms_keys, disable_cpms_keys, enable_cpms_keys,
    generate_cpms_install_qr, get_cpms_site_by_institution, list_cpms_keys, reissue_install_token,
    revoke_cpms_keys, revoke_install_token, verify_cpms_archive_qr,
};
