//! SFID 管理员登录认证模块。
//!
//! 对外通过 `admins::login::...` 暴露;内部按模型、普通登录、扫码登录、鉴权守卫、
//! 签名工具拆分,避免认证链路继续集中在单个超大文件。

mod guards;
mod handler;
mod model;
mod qr_login;
mod signature;

const LOGIN_CHALLENGE_TTL_SECONDS: i64 = 90;

pub(crate) use guards::{require_admin_any, require_federal_registry};
pub(crate) use handler::{
    admin_auth_challenge, admin_auth_check, admin_auth_identify, admin_auth_verify, admin_logout,
    require_admin_session_middleware,
};
pub(crate) use model::{AdminAuthContext, AdminSession, LoginChallenge, QrLoginResultRecord};
pub(crate) use qr_login::{admin_auth_qr_challenge, admin_auth_qr_complete, admin_auth_qr_result};
pub(crate) use signature::verify_admin_signature;
pub(crate) use signature::{
    build_admin_display_name, parse_sr25519_pubkey, parse_sr25519_pubkey_bytes,
};
