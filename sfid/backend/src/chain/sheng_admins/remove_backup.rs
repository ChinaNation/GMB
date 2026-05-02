//! 中文注释:SFID → 链上 `remove_sheng_admin_backup` 推 extrinsic(phase7 真实实现)。
//!
//! ADR-008:由当前 main 公钥签名授权,从链上 `ShengAdmins[Province][Slot]`
//! storage 注销 backup_1 / backup_2 公钥(Main 槽不允许动)。同时链端会级联清掉
//! 该 admin 的 ShengSigningPubkey 行。
//!
//! ## extrinsic 入参(citizenchain pallet sfid-system call_index 3)
//!
//! ```ignore
//! remove_sheng_admin_backup(
//!     province: Vec<u8>,
//!     slot: Slot,
//!     nonce: [u8; 32],
//!     sig: [u8; 64],         // main 私钥对 (REMOVE_BACKUP_DOMAIN ++ province ++ slot ++ nonce) blake2_256 后的签名
//! )
//! ```

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
    CALL_INDEX_REMOVE_SHENG_ADMIN_BACKUP, SFID_SYSTEM_PALLET_INDEX,
};
use crate::login::require_sheng_admin;
use crate::sheng_admins::province_admins::Slot;
use crate::AppState;

/// 链端共享 domain 常量(28 字节),与 `REMOVE_BACKUP_DOMAIN` 严格一致。
const REMOVE_BACKUP_DOMAIN: [u8; 28] = *b"remove_sheng_admin_backup_v1";

fn slot_byte(slot: Slot) -> u8 {
    match slot {
        Slot::Main => 0,
        Slot::Backup1 => 1,
        Slot::Backup2 => 2,
    }
}

/// 业务层 service:封装"参数校验 + 签名 + 真实推链"。
pub(crate) async fn remove_backup(
    province: &str,
    slot: Slot,
    signer_pair: &sr25519::Pair,
) -> Result<TxHash, ChainPushError> {
    if matches!(slot, Slot::Main) {
        return Err(ChainPushError::Other(
            "slot=MAIN cannot be modified via remove_backup".to_string(),
        ));
    }

    let nonce = generate_sheng_nonce()?;
    let province_bytes = province.as_bytes().to_vec();
    let slot_b = slot_byte(slot);

    let payload_tuple = (REMOVE_BACKUP_DOMAIN, province_bytes.clone(), slot_b, nonce);
    let digest = blake2_256(&payload_tuple.encode());
    let sig: [u8; 64] = signer_pair.sign(&digest).0;

    let mut call_bytes = Vec::with_capacity(2 + 32 + 1 + 32 + 64 + 8);
    call_bytes.push(SFID_SYSTEM_PALLET_INDEX);
    call_bytes.push(CALL_INDEX_REMOVE_SHENG_ADMIN_BACKUP);
    province_bytes.encode_to(&mut call_bytes);
    call_bytes.push(slot_b);
    call_bytes.extend_from_slice(&nonce);
    call_bytes.extend_from_slice(&sig);

    tracing::info!(
        province = %province,
        slot = slot.as_str(),
        nonce_hex = %format!("0x{}", hex::encode(nonce)),
        "[chain push] remove_sheng_admin_backup 即将提交"
    );
    submit_immortal_paysno("remove_sheng_admin_backup", call_bytes).await
}

// ─── HTTP handler ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct RemoveBackupInput {
    /// 槽位:"BACKUP_1" / "BACKUP_2"。
    pub(crate) slot: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RemoveBackupOutput {
    pub(crate) ok: bool,
    pub(crate) tx_hash: String,
}

/// `POST /api/v1/admin/sheng-admin/roster/remove-backup`
pub(crate) async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RemoveBackupInput>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    };
    let slot = match input.slot.as_str() {
        "BACKUP_1" => Slot::Backup1,
        "BACKUP_2" => Slot::Backup2,
        _ => {
            return crate::api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "slot must be BACKUP_1 or BACKUP_2",
            );
        }
    };

    let Some(signer_pair) = state
        .sheng_admin_signing_cache
        .any_for_province(province.as_str())
    else {
        return crate::api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1503,
            "本省签名密钥未就绪,请重新登录触发 bootstrap",
        );
    };

    match remove_backup(province.as_str(), slot, &signer_pair).await {
        Ok(tx) => Json(RemoveBackupOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "remove_sheng_admin_backup submit failed");
            crate::api_error(StatusCode::SERVICE_UNAVAILABLE, 1502, "chain push failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_backup_payload_signs_with_pair_and_verifies() {
        let pair = sr25519::Pair::from_seed(&[0x55u8; 32]);
        let signer_pubkey: [u8; 32] = pair.public().0;

        let payload_tuple = (
            REMOVE_BACKUP_DOMAIN,
            "AH".as_bytes().to_vec(),
            slot_byte(Slot::Backup2),
            [0x66u8; 32],
        );
        let digest = blake2_256(&payload_tuple.encode());
        let sig = pair.sign(&digest);
        assert!(sr25519::Pair::verify(
            &sig,
            digest,
            &sr25519::Public::from_raw(signer_pubkey)
        ));
    }

    #[test]
    fn remove_backup_rejects_main_slot() {
        let pair = sr25519::Pair::from_seed(&[0x77u8; 32]);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = rt
            .block_on(remove_backup("安徽省", Slot::Main, &pair))
            .unwrap_err();
        assert!(matches!(err, ChainPushError::Other(_)));
    }
}
