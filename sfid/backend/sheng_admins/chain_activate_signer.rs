//! 中文注释:SFID → 链上 `activate_sheng_signing_pubkey` 推 extrinsic(phase7 真实实现)。
//!
//! ADR-008:每省 3 把独立签名密钥(每个 admin slot 各一把)。某 admin 首登
//! 后,SFID `sheng_admins/bootstrap.rs::ensure_signing_keypair` 生成 seed 并落
//! 盘加密。本模块负责把生成出的签名公钥写入链上 `ShengSigningPubkey` storage。
//!
//! ## extrinsic 入参(citizenchain pallet sfid-system call_index 4)
//!
//! ```ignore
//! activate_sheng_signing_pubkey(
//!     province: Vec<u8>,
//!     admin_pubkey: [u8; 32],   // 调用方所属 slot 的 admin 公钥(花名册中)
//!     signing_pubkey: [u8; 32], // bootstrap 生成的 sr25519 签名公钥
//!     nonce: [u8; 32],
//!     sig: [u8; 64],            // admin 私钥对 (ACTIVATE_DOMAIN ++ province ++ admin_pubkey ++ signing_pubkey ++ nonce) 的签名
//! )
//! ```
//!
//! 链端 verifier:首次激活走 first-come-first-serve(占 Main 槽),后续替换由
//! 已记录的 admin pubkey 签名授权(详 ADR-008 第 4 节)。
//!
//! ## phase7 签名约束(留待 Step 2 联调收口)
//!
//! 链端 `activate_sheng_signing_pubkey` 验证 sig 由 `admin_pubkey` 签发,但 SFID
//! 后端不持有 admin slot 私钥(admin 私钥仅在冷钱包内)。当前实现以
//! `state.sheng_admin_signing_cache` 中的 signing pair 作为 stand-in 签名,链端验签会失败
//! 直至 Step 2 联调阶段引入冷钱包签名通路。卡点已在 phase7 任务卡 progress 章节记录。

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use parity_scale_codec::Encode;
use serde::Serialize;
use sp_core::{blake2_256, sr25519, Pair};

use crate::app_core::chain_client::{
    generate_sheng_nonce, submit_immortal_paysno, ChainPushError, TxHash,
    CALL_INDEX_ACTIVATE_SHENG_SIGNING, SFID_SYSTEM_PALLET_INDEX,
};
use crate::login::require_sheng_admin;
use crate::AppState;

/// 链端共享 domain 常量(32 字节),与 `ACTIVATE_DOMAIN` 严格一致。
const ACTIVATE_DOMAIN: [u8; 32] = *b"activate_sheng_signing_pubkey_v1";

/// 业务层 service:封装"参数校验 + 签名 + 真实推链"。
///
/// 真实推链 payload:
///   `blake2_256(SCALE_encode((ACTIVATE_DOMAIN, province, admin_pubkey, signing_pubkey, nonce)))`
/// 用 `signer_pair` 签 sr25519 → 得 64 字节 sig。
pub(crate) async fn activate(
    province: &str,
    admin_pubkey: &[u8; 32],
    signing_pubkey: &[u8; 32],
    signer_pair: &sr25519::Pair,
) -> Result<TxHash, ChainPushError> {
    let nonce = generate_sheng_nonce()?;
    let province_bytes = province.as_bytes().to_vec();

    let payload_tuple = (
        ACTIVATE_DOMAIN,
        province_bytes.clone(),
        *admin_pubkey,
        *signing_pubkey,
        nonce,
    );
    let digest = blake2_256(&payload_tuple.encode());
    let sig: [u8; 64] = signer_pair.sign(&digest).0;

    let mut call_bytes = Vec::with_capacity(2 + 32 + 32 + 32 + 32 + 64 + 8);
    call_bytes.push(SFID_SYSTEM_PALLET_INDEX);
    call_bytes.push(CALL_INDEX_ACTIVATE_SHENG_SIGNING);
    province_bytes.encode_to(&mut call_bytes);
    call_bytes.extend_from_slice(admin_pubkey);
    call_bytes.extend_from_slice(signing_pubkey);
    call_bytes.extend_from_slice(&nonce);
    call_bytes.extend_from_slice(&sig);

    tracing::info!(
        province = %province,
        admin_pubkey = %format!("0x{}", hex::encode(admin_pubkey)),
        signing_pubkey = %format!("0x{}", hex::encode(signing_pubkey)),
        nonce_hex = %format!("0x{}", hex::encode(nonce)),
        "[chain push] activate_sheng_signing_pubkey 即将提交"
    );
    submit_immortal_paysno("activate_sheng_signing_pubkey", call_bytes).await
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
/// 从 `state.sheng_admin_signing_cache` 取 bootstrap 生成的签名 keypair → 推链。
pub(crate) async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    };
    let Some(admin_pubkey) = crate::login::parse_sr25519_pubkey_bytes(ctx.admin_pubkey.as_str())
    else {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_pubkey must be 0x + 64 hex",
        );
    };
    let Some(pair) = state
        .sheng_admin_signing_cache
        .get(province.as_str(), &admin_pubkey)
    else {
        return crate::api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1503,
            "签名密钥未就绪,请重新登录触发 bootstrap",
        );
    };
    let signing_pubkey: [u8; 32] = sp_core::Pair::public(&pair).0;

    match activate(province.as_str(), &admin_pubkey, &signing_pubkey, &pair).await {
        Ok(tx) => Json(ActivateOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "activate_sheng_signing_pubkey submit failed");
            crate::api_error(StatusCode::SERVICE_UNAVAILABLE, 1502, "chain push failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activate_payload_signs_and_verifies() {
        let pair = sr25519::Pair::from_seed(&[0x88u8; 32]);
        let signer_pubkey: [u8; 32] = pair.public().0;

        let payload_tuple = (
            ACTIVATE_DOMAIN,
            "AH".as_bytes().to_vec(),
            [0x99u8; 32],
            [0xAAu8; 32],
            [0xBBu8; 32],
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
