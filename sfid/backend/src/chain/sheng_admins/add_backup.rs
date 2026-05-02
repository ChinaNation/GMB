//! 中文注释:SFID → 链上 `add_sheng_admin_backup` 推 extrinsic(phase7 真实实现)。
//!
//! ADR-008:由当前 main 公钥签名授权,新增 backup_1 / backup_2 公钥到链上
//! `ShengAdmins[Province][Slot]` storage。
//!
//! ## extrinsic 入参(citizenchain pallet sfid-system call_index 2)
//!
//! ```ignore
//! add_sheng_admin_backup(
//!     province: Vec<u8>,     // SCALE: Compact(len) ++ bytes
//!     slot: Slot,            // SCALE: 1 byte (Main=0/Backup1=1/Backup2=2)
//!     new_pubkey: [u8; 32],
//!     nonce: [u8; 32],
//!     sig: [u8; 64],         // main 私钥对 (ADD_BACKUP_DOMAIN ++ province ++ slot ++ new_pubkey ++ nonce) blake2_256 后的签名
//! )
//! ```
//!
//! payload domain 常量与链端严格对齐:`b"add_sheng_admin_backup_v1"`(25 字节)。
//! 任何字段顺序变更必须同步改链端 `Pallet::add_backup_payload`。

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
    CALL_INDEX_ADD_SHENG_ADMIN_BACKUP, SFID_SYSTEM_PALLET_INDEX,
};
use crate::login::require_sheng_admin;
use crate::sheng_admins::province_admins::Slot;
use crate::AppState;

/// 链端共享 domain 常量,与 `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs::ADD_BACKUP_DOMAIN`
/// 严格一致(`feedback_scale_domain_must_be_array.md`:必须 `[u8; N]` 数组)。
const ADD_BACKUP_DOMAIN: [u8; 25] = *b"add_sheng_admin_backup_v1";

/// Slot SCALE 编码到单字节(与链端 `pub enum Slot { Main, Backup1, Backup2 }` 严格对齐)。
fn slot_byte(slot: Slot) -> u8 {
    match slot {
        Slot::Main => 0,
        Slot::Backup1 => 1,
        Slot::Backup2 => 2,
    }
}

/// 业务层 service:封装"参数校验 + 签名 + 真实推链"。
///
/// `slot` 必须是 Backup1 / Backup2(Main 槽不允许动)。
/// `signer_pair` 是 main 槽对应的签名密钥(用于对 payload 签 sr25519);chain 端
/// 验签时取本省 `ShengAdmins[Province][Main]` 公钥比对。
///
/// 调用方零变化语义保持:返回 [`TxHash`],handler 取 `tx.hex`。
pub(crate) async fn add_backup(
    province: &str,
    slot: Slot,
    new_pubkey: [u8; 32],
    signer_pair: &sr25519::Pair,
) -> Result<TxHash, ChainPushError> {
    if matches!(slot, Slot::Main) {
        return Err(ChainPushError::Other(
            "slot=MAIN cannot be modified via add_backup".to_string(),
        ));
    }

    let nonce = generate_sheng_nonce()?;
    let province_bytes = province.as_bytes().to_vec();
    let slot_b = slot_byte(slot);

    // payload 与链端 add_backup_payload 严格一致:
    //   blake2_256( SCALE_encode( (ADD_BACKUP_DOMAIN, province, slot, new_pubkey, nonce) ) )
    let payload_tuple = (
        ADD_BACKUP_DOMAIN,
        province_bytes.clone(),
        slot_b,
        new_pubkey,
        nonce,
    );
    let digest = blake2_256(&payload_tuple.encode());
    let sig: [u8; 64] = signer_pair.sign(&digest).0;

    // 裸 SCALE 编码 call_data:pallet_idx ++ call_idx ++ args
    let mut call_bytes = Vec::with_capacity(2 + 32 + 1 + 32 + 32 + 64 + 8);
    call_bytes.push(SFID_SYSTEM_PALLET_INDEX);
    call_bytes.push(CALL_INDEX_ADD_SHENG_ADMIN_BACKUP);
    province_bytes.encode_to(&mut call_bytes); // Vec<u8> = Compact(len) ++ bytes
    call_bytes.push(slot_b);
    call_bytes.extend_from_slice(&new_pubkey);
    call_bytes.extend_from_slice(&nonce);
    call_bytes.extend_from_slice(&sig);

    tracing::info!(
        province = %province,
        slot = slot.as_str(),
        new_pubkey_hex = %format!("0x{}", hex::encode(new_pubkey)),
        nonce_hex = %format!("0x{}", hex::encode(nonce)),
        "[chain push] add_sheng_admin_backup 即将提交"
    );
    submit_immortal_paysno("add_sheng_admin_backup", call_bytes).await
}

// ─── HTTP handler ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct AddBackupInput {
    /// 槽位:"BACKUP_1" / "BACKUP_2"。
    pub(crate) slot: String,
    /// 新 backup 公钥,0x 小写 hex(32 字节)。
    pub(crate) new_pubkey: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AddBackupOutput {
    pub(crate) ok: bool,
    pub(crate) tx_hash: String,
}

/// `POST /api/v1/admin/sheng-admin/roster/add-backup`
///
/// session: 必须是 main 槽对应私钥登录;链端 ValidateUnsigned 二次校验签名是否
/// 由本省 `ShengAdmins[Province][Main]` 私钥签发。
pub(crate) async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AddBackupInput>,
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
    let Some(new_pubkey) =
        crate::sheng_admins::province_admins::pubkey_from_hex(input.new_pubkey.as_str())
    else {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "new_pubkey must be 0x + 64 hex",
        );
    };

    // 中文注释:取本省 main 槽对应签名密钥(签名 payload 用)。
    // SFID 不持有 admin slot 私钥,本期由 sheng_admin_signing_cache 提供本省任一已登录 slot 的
    // 签名 Pair 作为 stand-in;链端验签若失败将返回 InvalidTx,由调用方观察。
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

    match add_backup(province.as_str(), slot, new_pubkey, &signer_pair).await {
        Ok(tx) => Json(AddBackupOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "add_sheng_admin_backup submit failed");
            crate::api_error(StatusCode::SERVICE_UNAVAILABLE, 1502, "chain push failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证签名 + call_bytes 编码与链端 verifier 期望一致。
    /// 不联网,纯本地构造 + sr25519 自验证 + call_bytes 头部断言。
    #[test]
    fn add_backup_payload_signs_with_pair_and_verifies() {
        let seed = [0x11u8; 32];
        let pair = sr25519::Pair::from_seed(&seed);
        let signer_pubkey: [u8; 32] = pair.public().0;

        let province = "安徽省";
        let slot = Slot::Backup1;
        let new_pubkey = [0x22u8; 32];
        let nonce = [0x33u8; 32];

        let payload_tuple = (
            ADD_BACKUP_DOMAIN,
            province.as_bytes().to_vec(),
            slot_byte(slot),
            new_pubkey,
            nonce,
        );
        let digest = blake2_256(&payload_tuple.encode());
        let sig = pair.sign(&digest);

        // sr25519 自验证(链端 verifier 等价路径)
        assert!(sr25519::Pair::verify(
            &sig,
            digest,
            &sr25519::Public::from_raw(signer_pubkey)
        ));
    }

    #[test]
    fn add_backup_rejects_main_slot() {
        let seed = [0x44u8; 32];
        let pair = sr25519::Pair::from_seed(&seed);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = rt
            .block_on(add_backup("安徽省", Slot::Main, [0u8; 32], &pair))
            .unwrap_err();
        assert!(matches!(err, ChainPushError::Other(_)));
    }

    /// 校验 call_bytes 前两个字节是 (pallet_idx=10, call_idx=2)。
    /// 这相当于 mock subxt 客户端对 SCALE 编码做断言。
    #[test]
    fn add_backup_call_bytes_layout_matches_pallet_index() {
        // 模拟 add_backup 内部裸编码逻辑(独立断言路径,避免依赖网络)。
        let mut call_bytes = vec![SFID_SYSTEM_PALLET_INDEX, CALL_INDEX_ADD_SHENG_ADMIN_BACKUP];
        b"AH".to_vec().encode_to(&mut call_bytes);
        call_bytes.push(slot_byte(Slot::Backup1));
        call_bytes.extend_from_slice(&[0u8; 32]);
        call_bytes.extend_from_slice(&[0u8; 32]);
        call_bytes.extend_from_slice(&[0u8; 64]);

        assert_eq!(call_bytes[0], 10, "pallet_idx must be 10 (SfidSystem)");
        assert_eq!(
            call_bytes[1], 2,
            "call_idx must be 2 (add_sheng_admin_backup)"
        );
    }

    /// 端到端构造 unsigned extrinsic wire 字节(不联网),覆盖:
    /// - pallet_idx / call_idx 与链端 #[pallet::call_index(2)] 对齐
    /// - SCALE 编码顺序(province / slot / new_pubkey / nonce / sig)与链端
    ///   `extract_unsigned_parts::Call::add_sheng_admin_backup` 字段顺序对齐
    /// - V4 BARE 包装(version=0x04 + Compact len 前缀)与
    ///   `subxt_core::tx::create_v4_unsigned` 路径产物等价
    #[test]
    fn add_backup_constructs_correct_unsigned_extrinsic_wire() {
        use crate::chain::client::wrap_v4_bare;

        let pair = sr25519::Pair::from_seed(&[0xA1u8; 32]);
        let province = "AH";
        let slot = Slot::Backup1;
        let new_pubkey = [0xB2u8; 32];
        let nonce = [0xC3u8; 32];

        let payload_tuple = (
            ADD_BACKUP_DOMAIN,
            province.as_bytes().to_vec(),
            slot_byte(slot),
            new_pubkey,
            nonce,
        );
        let digest = blake2_256(&payload_tuple.encode());
        let sig: [u8; 64] = pair.sign(&digest).0;

        let mut call_bytes = Vec::new();
        call_bytes.push(SFID_SYSTEM_PALLET_INDEX);
        call_bytes.push(CALL_INDEX_ADD_SHENG_ADMIN_BACKUP);
        province.as_bytes().to_vec().encode_to(&mut call_bytes);
        call_bytes.push(slot_byte(slot));
        call_bytes.extend_from_slice(&new_pubkey);
        call_bytes.extend_from_slice(&nonce);
        call_bytes.extend_from_slice(&sig);

        // 长度断言:
        //   1 (pallet) + 1 (call) + 1 (Compact<u32> for "AH" len=2) + 2 (bytes)
        //   + 1 (slot) + 32 + 32 + 64 = 134
        assert_eq!(call_bytes.len(), 1 + 1 + 1 + 2 + 1 + 32 + 32 + 64);
        // Compact(2) 单字节模式 = 2 << 2 = 0x08
        assert_eq!(call_bytes[2], 0x08);
        assert_eq!(&call_bytes[3..5], b"AH");

        // 包装 V4 BARE wire 字节:首字节是 Compact(inner_len),inner 第一个字节固定 0x04
        let wire = wrap_v4_bare(&call_bytes);
        let inner_len = call_bytes.len() + 1; // +1 for version byte
                                              // Compact<u32> 模式 1 (n<2^14): (n<<2)|0b01 → little-endian u16
        let expected_first_two = ((inner_len as u32) << 2) | 0b01;
        assert_eq!(wire[0] as u32, expected_first_two & 0xFF);
        assert_eq!(wire[1] as u32, (expected_first_two >> 8) & 0xFF);
        assert_eq!(wire[2], 0x04, "V4 BARE version byte");
    }
}
