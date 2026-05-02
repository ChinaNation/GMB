//! 中文注释:省管理员 3-tier 签名 seed 加密持久化。
//!
//! ADR-008 决议(2026-05-01):每个 admin slot 独立签名密钥。
//! 加密 seed 落盘:`storage/sheng_signer/{province}_{pubkey_hex}.enc`
//! 加密算法:AES-256-GCM
//! Wrap key:`HKDF(SFID_MASTER_KEK, salt = admin_pubkey)`
//!
//! `SFID_MASTER_KEK`:取自环境变量 `SFID_MASTER_KEK_HEX`(64 hex chars,32 byte)。
//! 缺失时启动期会 fallback 到 `SFID_SIGNING_SEED_HEX`(SFID main seed)兼容
//! 现有 dev 环境;production 必须显式设置 `SFID_MASTER_KEK_HEX`。

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

const NONCE_LEN: usize = 12;
const STORAGE_DIR_ENV: &str = "SFID_SHENG_SIGNER_DIR";
const STORAGE_DIR_DEFAULT: &str = "storage/sheng_signer";

/// 取 SFID_MASTER_KEK 32 字节。
fn master_kek() -> Result<[u8; 32], String> {
    let raw = std::env::var("SFID_MASTER_KEK_HEX")
        .or_else(|_| std::env::var("SFID_SIGNING_SEED_HEX"))
        .map_err(|_| "SFID_MASTER_KEK_HEX (or SFID_SIGNING_SEED_HEX fallback) not set".to_string())?;
    let trimmed = raw
        .trim()
        .strip_prefix("0x")
        .or_else(|| raw.trim().strip_prefix("0X"))
        .unwrap_or_else(|| raw.trim());
    let bytes = hex::decode(trimmed).map_err(|e| format!("hex decode master kek: {e}"))?;
    if bytes.len() != 32 {
        return Err("master kek must decode to 32 bytes".to_string());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// HKDF 派生 wrap key:salt = admin_pubkey。
fn derive_wrap_key(master: &[u8; 32], admin_pubkey: &[u8; 32]) -> Result<[u8; 32], String> {
    let mut wrap = [0u8; 32];
    Hkdf::<Sha256>::new(Some(admin_pubkey.as_slice()), master.as_slice())
        .expand(b"sfid-sheng-signer-3tier-v1", &mut wrap)
        .map_err(|_| "hkdf expand failed".to_string())?;
    Ok(wrap)
}

fn storage_dir() -> PathBuf {
    std::env::var(STORAGE_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(STORAGE_DIR_DEFAULT))
}

fn seed_path(province: &str, admin_pubkey: &[u8; 32]) -> PathBuf {
    let mut p = storage_dir();
    let safe_province = province.replace(['/', '\\', '\0'], "_");
    p.push(format!("{safe_province}_{}.enc", hex::encode(admin_pubkey)));
    p
}

fn ensure_dir_exists(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir storage dir: {e}"))?;
    }
    Ok(())
}

fn encrypt(wrap: &[u8; 32], plaintext: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(wrap));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| format!("rng: {e}"))?;
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_slice())
        .map_err(|e| format!("aes-gcm encrypt: {e}"))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

fn decrypt(wrap: &[u8; 32], blob: &[u8]) -> Result<[u8; 32], String> {
    if blob.len() < NONCE_LEN + 16 {
        return Err("ciphertext too short".to_string());
    }
    let (nonce_bytes, ct) = blob.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(wrap));
    let mut pt = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ct)
        .map_err(|e| format!("aes-gcm decrypt: {e}"))?;
    if pt.len() != 32 {
        pt.zeroize();
        return Err("decrypted seed must be 32 bytes".to_string());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&pt);
    pt.zeroize();
    Ok(out)
}

/// 把 (province, admin_pubkey) 的 32 字节 seed 加密落盘。
pub(crate) fn save_seed(
    province: &str,
    admin_pubkey: &[u8; 32],
    seed: &[u8; 32],
) -> Result<(), String> {
    let dir = storage_dir();
    ensure_dir_exists(&dir)?;
    let kek = master_kek()?;
    let wrap = derive_wrap_key(&kek, admin_pubkey)?;
    let blob = encrypt(&wrap, seed)?;
    let p = seed_path(province, admin_pubkey);
    std::fs::write(&p, blob).map_err(|e| format!("write seed file {}: {e}", p.display()))?;
    Ok(())
}

/// 读取并解密 (province, admin_pubkey) 的 seed。文件不存在返回 Ok(None)。
pub(crate) fn load_seed(
    province: &str,
    admin_pubkey: &[u8; 32],
) -> Result<Option<[u8; 32]>, String> {
    let p = seed_path(province, admin_pubkey);
    if !p.exists() {
        return Ok(None);
    }
    let blob = std::fs::read(&p).map_err(|e| format!("read seed file {}: {e}", p.display()))?;
    let kek = master_kek()?;
    let wrap = derive_wrap_key(&kek, admin_pubkey)?;
    let seed = decrypt(&wrap, &blob)?;
    Ok(Some(seed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_tmp_dir(label: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("sfid_sheng_signer_test_{label}_{pid}_{nanos}"));
        p
    }

    #[test]
    fn encrypt_decrypt_roundtrip_in_memory() {
        let kek = [0x42u8; 32];
        let pk = [0xAAu8; 32];
        let seed = [0xBBu8; 32];
        let wrap = derive_wrap_key(&kek, &pk).expect("wrap");
        let blob = encrypt(&wrap, &seed).expect("enc");
        let got = decrypt(&wrap, &blob).expect("dec");
        assert_eq!(got, seed);
    }

    #[test]
    fn roundtrip_seed_persistence() {
        let dir = unique_tmp_dir("roundtrip");
        std::env::set_var(STORAGE_DIR_ENV, &dir);
        std::env::set_var(
            "SFID_MASTER_KEK_HEX",
            "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        );
        let pk = [0xAAu8; 32];
        let seed = [0xBBu8; 32];
        save_seed("测试省", &pk, &seed).expect("save");
        let got = load_seed("测试省", &pk).expect("load").expect("some");
        assert_eq!(got, seed);
        // 清理
        let _ = std::fs::remove_dir_all(&dir);
    }
}
