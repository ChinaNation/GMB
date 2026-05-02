//! 中文注释:SFID → 链上 `rotate_sheng_signing_pubkey` 推 extrinsic(phase45 mock)。
//!
//! ADR-008:某 admin slot 主动轮换签名密钥时调用。
//!
//! ## extrinsic 入参(phase7 切真时使用)
//!
//! ```ignore
//! rotate_sheng_signing_pubkey(
//!     province: ProvinceCode,
//!     admin_pubkey: [u8; 32],       // 调用方所属 slot 的 admin 公钥
//!     new_signing_pubkey: [u8; 32], // 轮换后的新 sr25519 签名公钥
//!     sig: [u8; 64],                // 原 admin 私钥对 (province, new_signing_pubkey, nonce) 的签名
//! )
//! ```
//!
//! ## phase45 行为
//!
//! 与 `activation.rs` 同结构,handler 入参带 `new_signing_pubkey`(0x hex)。

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::chain::client::{submit_immortal_paysno_mock, ChainPushError, MockTxHash};
use crate::login::require_sheng_admin;
use crate::AppState;

pub(crate) async fn rotate(
    province: &str,
    admin_pubkey: &[u8; 32],
    new_signing_pubkey: &[u8; 32],
) -> Result<MockTxHash, ChainPushError> {
    tracing::info!(
        province = %province,
        admin_pubkey = %format!("0x{}", hex::encode(admin_pubkey)),
        new_signing_pubkey = %format!("0x{}", hex::encode(new_signing_pubkey)),
        "[chain push] rotate_sheng_signing_pubkey 即将提交"
    );
    submit_immortal_paysno_mock("rotate_sheng_signing_pubkey").await
}

// ─── HTTP handler ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct RotateInput {
    /// 新的签名公钥,0x 小写 hex(32 字节)。
    pub(crate) new_signing_pubkey: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RotateOutput {
    pub(crate) ok: bool,
    pub(crate) tx_hash: String,
}

/// `POST /api/v1/admin/sheng-signer/rotate`
pub(crate) async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RotateInput>,
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
    let Some(new_signing_pubkey) =
        crate::sfid::province::pubkey_from_hex(input.new_signing_pubkey.as_str())
    else {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "new_signing_pubkey must be 0x + 64 hex",
        );
    };
    match rotate(province.as_str(), &admin_pubkey, &new_signing_pubkey).await {
        Ok(tx) => Json(RotateOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "rotate_sheng_signing_pubkey mock submit failed");
            crate::api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1502,
                "chain push mock failed",
            )
        }
    }
}
