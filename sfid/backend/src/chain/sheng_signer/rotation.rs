//! 中文注释:SFID → 链上 `rotate_sheng_signing_pubkey` 推 extrinsic(phase7 真实实现)。
//!
//! ADR-008:某 admin slot 主动轮换签名密钥时调用。链端要求 admin_pubkey ∈
//! ShengAdmins[province][\*] 且原 signing pubkey 已记录。
//!
//! ## extrinsic 入参(citizenchain pallet sfid-system call_index 5)
//!
//! ```ignore
//! rotate_sheng_signing_pubkey(
//!     province: Vec<u8>,
//!     admin_pubkey: [u8; 32],
//!     new_signing_pubkey: [u8; 32],
//!     nonce: [u8; 32],
//!     sig: [u8; 64],   // admin 私钥对 (ROTATE_DOMAIN ++ province ++ admin_pubkey ++ new_signing_pubkey ++ nonce) 的签名
//! )
//! ```
//!
//! 与 `activation.rs` 共享同一签名密钥来源约束(详见该模块文档头注释)。

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use sp_core::{blake2_256, sr25519, Pair};

use crate::chain::client::{
    generate_sheng_nonce, submit_immortal_paysno, ChainPushError, TxHash,
    CALL_INDEX_ROTATE_SHENG_SIGNING, SFID_SYSTEM_PALLET_INDEX,
};
use crate::login::require_sheng_admin;
use crate::AppState;

/// 链端共享 domain 常量(30 字节),与 `ROTATE_DOMAIN` 严格一致。
const ROTATE_DOMAIN: [u8; 30] = *b"rotate_sheng_signing_pubkey_v1";

pub(crate) async fn rotate(
    province: &str,
    admin_pubkey: &[u8; 32],
    new_signing_pubkey: &[u8; 32],
    signer_pair: &sr25519::Pair,
) -> Result<TxHash, ChainPushError> {
    let nonce = generate_sheng_nonce()?;
    let province_bytes = province.as_bytes().to_vec();

    let payload_tuple = (
        ROTATE_DOMAIN,
        province_bytes.clone(),
        *admin_pubkey,
        *new_signing_pubkey,
        nonce,
    );
    let digest = blake2_256(&payload_tuple.encode());
    let sig: [u8; 64] = signer_pair.sign(&digest).0;

    let mut call_bytes = Vec::with_capacity(2 + 32 + 32 + 32 + 32 + 64 + 8);
    call_bytes.push(SFID_SYSTEM_PALLET_INDEX);
    call_bytes.push(CALL_INDEX_ROTATE_SHENG_SIGNING);
    province_bytes.encode_to(&mut call_bytes);
    call_bytes.extend_from_slice(admin_pubkey);
    call_bytes.extend_from_slice(new_signing_pubkey);
    call_bytes.extend_from_slice(&nonce);
    call_bytes.extend_from_slice(&sig);

    tracing::info!(
        province = %province,
        admin_pubkey = %format!("0x{}", hex::encode(admin_pubkey)),
        new_signing_pubkey = %format!("0x{}", hex::encode(new_signing_pubkey)),
        nonce_hex = %format!("0x{}", hex::encode(nonce)),
        "[chain push] rotate_sheng_signing_pubkey 即将提交"
    );
    submit_immortal_paysno("rotate_sheng_signing_pubkey", call_bytes).await
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
    let Some(pair) = state.sheng_signer_cache.get(province.as_str(), &admin_pubkey) else {
        return crate::api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1503,
            "签名密钥未就绪,请重新登录触发 bootstrap",
        );
    };

    match rotate(province.as_str(), &admin_pubkey, &new_signing_pubkey, &pair).await {
        Ok(tx) => Json(RotateOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "rotate_sheng_signing_pubkey submit failed");
            crate::api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1502,
                "chain push failed",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_payload_signs_and_verifies() {
        let pair = sr25519::Pair::from_seed(&[0xCCu8; 32]);
        let signer_pubkey: [u8; 32] = pair.public().0;

        let payload_tuple = (
            ROTATE_DOMAIN,
            "AH".as_bytes().to_vec(),
            [0xDDu8; 32],
            [0xEEu8; 32],
            [0xFFu8; 32],
        );
        let digest = blake2_256(&payload_tuple.encode());
        let sig = pair.sign(&digest);
        assert!(sr25519::Pair::verify(
            &sig,
            digest,
            &sr25519::Public::from_raw(signer_pubkey)
        ));
    }
}
