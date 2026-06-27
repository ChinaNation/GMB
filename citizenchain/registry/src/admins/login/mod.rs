//! CID 管理员登录认证模块。
//!
//! 对外通过 `admins::login::...` 暴露;内部按模型、普通登录、扫码登录、鉴权守卫、
//! 签名工具拆分,避免认证链路继续集中在单个超大文件。

mod guards;
mod handler;
mod model;
mod onchain_gate;
mod qr_login;
mod signature;

const LOGIN_SIGN_REQUEST_TTL_SECONDS: i64 = 90;

pub(crate) use guards::require_admin_any;
pub(crate) use handler::{
    admin_auth_challenge, admin_auth_check, admin_auth_identify, admin_auth_verify, admin_logout,
    require_admin_session_middleware,
};
pub(crate) use model::{AdminAuthContext, AdminSession, LoginSignRequest, QrLoginResultRecord};
pub(crate) use onchain_gate::revoke_stale_admin_sessions_loop;
pub(crate) use qr_login::{
    admin_auth_qr_complete, admin_auth_qr_result, admin_auth_qr_sign_request,
};
pub(crate) use signature::verify_admin_signature;
pub(crate) use signature::{build_admin_name, parse_sr25519_pubkey, parse_sr25519_pubkey_bytes};
