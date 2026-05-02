//! 中文注释:省管理员 4 个 unsigned extrinsic 的签名 payload。
//!
//! 这些 domain 常量和字段顺序必须与 SFID 后端 `chain/sheng_admins/`
//! 完全一致,否则链端 ValidateUnsigned 会判定 BadProof。

use codec::Encode;
use sp_io::hashing::blake2_256;

use super::types::Slot;

/// 中文注释:ADR-008 Step 2 签名 payload domain 常量。
/// 必须使用 `&[u8; N]` 数组类型(对应 feedback_scale_domain_must_be_array.md),
/// 链上 verifier 验签时与 SFID 后端 sign 端必须严格一致。
pub const ADD_BACKUP_DOMAIN: &[u8; 25] = b"add_sheng_admin_backup_v1";
pub const REMOVE_BACKUP_DOMAIN: &[u8; 28] = b"remove_sheng_admin_backup_v1";
pub const ACTIVATE_DOMAIN: &[u8; 32] = b"activate_sheng_signing_pubkey_v1";
pub const ROTATE_DOMAIN: &[u8; 30] = b"rotate_sheng_signing_pubkey_v1";

pub fn add_backup_payload(
    province: &[u8],
    slot: Slot,
    new_pubkey: &[u8; 32],
    nonce: &[u8; 32],
) -> [u8; 32] {
    let payload = (ADD_BACKUP_DOMAIN, province, slot, new_pubkey, nonce);
    blake2_256(&payload.encode())
}

pub fn remove_backup_payload(province: &[u8], slot: Slot, nonce: &[u8; 32]) -> [u8; 32] {
    let payload = (REMOVE_BACKUP_DOMAIN, province, slot, nonce);
    blake2_256(&payload.encode())
}

pub fn activate_payload(
    province: &[u8],
    admin_pubkey: &[u8; 32],
    signing_pubkey: &[u8; 32],
    nonce: &[u8; 32],
) -> [u8; 32] {
    let payload = (
        ACTIVATE_DOMAIN,
        province,
        admin_pubkey,
        signing_pubkey,
        nonce,
    );
    blake2_256(&payload.encode())
}

pub fn rotate_payload(
    province: &[u8],
    admin_pubkey: &[u8; 32],
    new_signing_pubkey: &[u8; 32],
    nonce: &[u8; 32],
) -> [u8; 32] {
    let payload = (
        ROTATE_DOMAIN,
        province,
        admin_pubkey,
        new_signing_pubkey,
        nonce,
    );
    blake2_256(&payload.encode())
}
