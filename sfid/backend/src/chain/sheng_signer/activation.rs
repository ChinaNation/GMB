//! 中文注释:SFID → 链上 `activate_sheng_signing_pubkey` 推 extrinsic(phase45 mock)。
//!
//! ADR-008:每省 3 把独立签名密钥(每个 admin slot 各一把)。某 admin 首登
//! 后,SFID `sheng_admins/bootstrap.rs::ensure_signing_keypair` 生成 seed 并落
//! 盘加密。本模块负责把生成出的签名公钥写入链上 `ShengSigningPubkey` storage。
//!
//! ## extrinsic 入参(phase7 切真时使用)
//!
//! ```ignore
//! activate_sheng_signing_pubkey(
//!     province: ProvinceCode,
//!     admin_pubkey: [u8; 32],   // 调用方所属 slot 的 admin 公钥
//!     signing_pubkey: [u8; 32], // bootstrap 生成的 sr25519 签名公钥
//!     sig: [u8; 64],            // admin 私钥对 (province, signing_pubkey, nonce) 的签名
//! )
//! ```
//!
//! 链端 verifier:首次激活走 first-come-first-serve,后续替换由记录在案的
//! admin pubkey 签名授权(详 ADR-008 第 4 节)。
//!
//! ## phase45 行为
//!
//! `activate` service 接收已构造好的(`admin_pubkey`, `signing_pubkey`),
//! 返回 [`MockTxHash`]。

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::chain::client::{submit_immortal_paysno_mock, ChainPushError, MockTxHash};
use crate::login::require_sheng_admin;
use crate::AppState;

/// 业务层 service:封装"参数校验 + mock 推链"。
///
/// 真实推链(phase7)签名 payload:`(province, signing_pubkey, nonce)`,
/// 用 admin slot 自己的私钥签。phase45 mock 不计算签名,只 emit 日志。
pub(crate) async fn activate(
    province: &str,
    admin_pubkey: &[u8; 32],
    signing_pubkey: &[u8; 32],
) -> Result<MockTxHash, ChainPushError> {
    tracing::info!(
        province = %province,
        admin_pubkey = %format!("0x{}", hex::encode(admin_pubkey)),
        signing_pubkey = %format!("0x{}", hex::encode(signing_pubkey)),
        "[chain push] activate_sheng_signing_pubkey 即将提交"
    );
    submit_immortal_paysno_mock("activate_sheng_signing_pubkey").await
}

// ─── HTTP handler ───────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct ActivateOutput {
    pub(crate) ok: bool,
    pub(crate) tx_hash: String,
}

/// `POST /api/v1/admin/sheng-signer/activate`
///
/// 当前登录的 admin slot 触发,无请求体:从 session 取 (province, admin_pubkey),
/// 从 `state.sheng_signer_cache` 取 bootstrap 生成的签名 keypair → 推链。
pub(crate) async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        );
    };
    let Some(admin_pubkey) =
        crate::login::parse_sr25519_pubkey_bytes(ctx.admin_pubkey.as_str())
    else {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_pubkey must be 0x + 64 hex",
        );
    };
    let Some(pair) = state.sheng_signer_cache.get(province.as_str(), &admin_pubkey) else {
        return crate::api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1503,
            "签名密钥未就绪,请重新登录触发 bootstrap",
        );
    };
    let signing_pubkey: [u8; 32] = sp_core::Pair::public(&pair).0;
    match activate(province.as_str(), &admin_pubkey, &signing_pubkey).await {
        Ok(tx) => Json(ActivateOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "activate_sheng_signing_pubkey mock submit failed");
            crate::api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1502,
                "chain push mock failed",
            )
        }
    }
}
