//! 中文注释：推链签名路由器。
//!
//! - 业务 extrinsic 必须由本省签名密钥签发；省未上线(cache 缺失)→ 503
//! - set_sheng_signing_pubkey / rotate_sfid_keys 等管理 extrinsic 用 SFID MAIN signer
//!
//! 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B。

use axum::http::StatusCode;

use crate::key_admins::sheng_signer_cache::ProvinceSigner;
use crate::login::AdminAuthContext;
use crate::models::AdminRole;
use crate::AppState;

/// 根据管理员上下文路由到本省签名 signer(日常业务推链)。
///
/// 返回 `(Pair, province_name)`。省未上线时返回 503。
#[allow(dead_code)]
pub(crate) fn resolve_business_signer(
    state: &AppState,
    ctx: &AdminAuthContext,
) -> Result<(ProvinceSigner, String), (StatusCode, String)> {
    let province = match ctx.role {
        AdminRole::KeyAdmin => {
            return Err((
                StatusCode::FORBIDDEN,
                "密钥管理员不能直接推送业务交易".to_string(),
            ));
        }
        AdminRole::ShengAdmin | AdminRole::ShiAdmin => ctx.admin_province.as_deref().ok_or_else(
            || (StatusCode::BAD_REQUEST, "管理员缺少省份信息".to_string()),
        )?,
    };
    match state.sheng_signer_cache.get(province) {
        Some(s) => Ok((s, province.to_string())),
        None => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("本省({province})登录管理员未在线，暂无法推链"),
        )),
    }
}

/// 管理 extrinsic 用的 SFID MAIN signer(克隆 Pair)。
#[allow(dead_code)]
pub(crate) fn resolve_sfid_main_signer(state: &AppState) -> ProvinceSigner {
    state.sheng_signer_cache.sfid_main_signer()
}
