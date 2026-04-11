//! 中文注释：省级签名密钥的进程内内存缓存。
//!
//! 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B。
//!
//! 设计要点：
//! - 启动时从 SFID MAIN seed(signing_seed_hex)构造 in-memory SFID MAIN `sr25519::Pair`
//! - Wrap key 由 HKDF-SHA256(SFID_MAIN_seed, salt, info)派生，用于 AES-256-GCM
//!   加/解密 sheng admin 的签名私钥种子
//! - 省登录管理员登录时载入本省 Pair 到 cache，登出/idle 超时驱逐
//! - SFID MAIN 轮换时通过 `rotate_main_seed` 同步替换内存 Pair 和 wrap key
//!
//! 用户拍板：wrap key 派生自 SFID MAIN seed，轮换时级联重新加密所有 sheng
//! admin 的密文(由调用方 `replace_active_main_seed` 配合完成)。
//!
//! 注意：为了对齐本仓库既有推链流程(见 `sheng-admins/institutions.rs` 中的
//! `partial_tx + sign_with_account_and_signature`)，本 cache 存放的是
//! `sp_core::sr25519::Pair` 明文对，**不**包装为 subxt::tx::PairSigner。调用方
//! 可直接用 `pair.public().0` 拿公钥，用 `pair.sign(&payload).0` 签名。

use std::collections::HashMap;
use std::sync::RwLock;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine as _;
use hkdf::Hkdf;
use sha2::Sha256;
use sp_core::{sr25519, Pair};
use zeroize::Zeroize;

/// 省签名 keypair 类型别名(明文 sr25519 Pair)。
pub(crate) type ProvinceSigner = sr25519::Pair;

const WRAP_SALT: &[u8] = b"sfid-sheng-signer-v1-salt";
const WRAP_INFO: &[u8] = b"sfid-sheng-signer-v1-info";
const NONCE_LEN: usize = 12;

pub(crate) struct ShengSignerCache {
    signers: RwLock<HashMap<String, ProvinceSigner>>,
    sfid_main_signer: RwLock<ProvinceSigner>,
    wrap_key: RwLock<[u8; 32]>,
}

impl ShengSignerCache {
    /// 从 SFID MAIN 种子构造。seed 调用后会被 zeroize(就地清零)。
    pub(crate) fn new_from_seed(sfid_main_seed: &mut [u8; 32]) -> Result<Self, String> {
        let pair = sr25519::Pair::from_seed(sfid_main_seed);
        let wrap_key = derive_wrap_key(sfid_main_seed)?;
        sfid_main_seed.zeroize();
        Ok(Self {
            signers: RwLock::new(HashMap::new()),
            sfid_main_signer: RwLock::new(pair),
            wrap_key: RwLock::new(wrap_key),
        })
    }

    /// 克隆当前 SFID MAIN 的 sr25519 Pair(用于签管理 extrinsic)。
    pub(crate) fn sfid_main_signer(&self) -> ProvinceSigner {
        self.sfid_main_signer
            .read()
            .expect("sheng_signer_cache sfid_main_signer poisoned")
            .clone()
    }

    /// 加密 32 字节种子 → base64 密文。
    pub(crate) fn encrypt_seed(&self, seed: &[u8; 32]) -> Result<String, String> {
        let wrap = self.wrap_key.read().map_err(|_| "cache poisoned")?;
        encrypt_with_wrap(&*wrap, seed)
    }

    /// 解密 base64 密文 → 32 字节种子。
    pub(crate) fn decrypt_seed(&self, encrypted_b64: &str) -> Result<[u8; 32], String> {
        let wrap = self.wrap_key.read().map_err(|_| "cache poisoned")?;
        decrypt_with_wrap(&*wrap, encrypted_b64)
    }

    pub(crate) fn load_province(&self, province: String, signer: ProvinceSigner) {
        if let Ok(mut g) = self.signers.write() {
            g.insert(province, signer);
        }
    }

    pub(crate) fn unload_province(&self, province: &str) {
        if let Ok(mut g) = self.signers.write() {
            g.remove(province);
        }
    }

    pub(crate) fn get(&self, province: &str) -> Option<ProvinceSigner> {
        self.signers.read().ok()?.get(province).cloned()
    }

    #[allow(dead_code)]
    pub(crate) fn active_province_count(&self) -> usize {
        self.signers.read().map(|g| g.len()).unwrap_or(0)
    }

    /// SFID MAIN seed 轮换：替换内存 Pair 和 wrap key。
    ///
    /// **注意**：调用方必须先完成数据库中密文的级联重加密，否则已有 sheng
    /// admin 登录后会解不开旧密文。
    pub(crate) fn rotate_main_seed(&self, new_seed: &mut [u8; 32]) -> Result<(), String> {
        let new_pair = sr25519::Pair::from_seed(new_seed);
        let new_wrap = derive_wrap_key(new_seed)?;
        new_seed.zeroize();

        *self
            .sfid_main_signer
            .write()
            .map_err(|_| "cache poisoned")? = new_pair;
        *self.wrap_key.write().map_err(|_| "cache poisoned")? = new_wrap;
        // 注意：self.signers 不清空。已载入省份的 Pair 是明文 seed 构造的，
        // 与 wrap key 无关，继续有效。但数据库里的密文必须已被调用方更新。
        Ok(())
    }

    /// 获取当前 wrap key 的副本(给 `replace_active_main_seed` 做级联重加密用)。
    #[allow(dead_code)]
    pub(crate) fn current_wrap_key(&self) -> Result<[u8; 32], String> {
        self.wrap_key
            .read()
            .map(|g| *g)
            .map_err(|_| "cache poisoned".to_string())
    }
}

pub(crate) fn derive_wrap_key(seed: &[u8; 32]) -> Result<[u8; 32], String> {
    let mut wrap = [0u8; 32];
    Hkdf::<Sha256>::new(Some(WRAP_SALT), seed.as_slice())
        .expand(WRAP_INFO, &mut wrap)
        .map_err(|_| "hkdf expand failed".to_string())?;
    Ok(wrap)
}

pub(crate) fn encrypt_with_wrap(wrap: &[u8; 32], seed: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(wrap));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| format!("rng: {e}"))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, seed.as_slice())
        .map_err(|e| format!("aes encrypt: {e}"))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(&out))
}

pub(crate) fn decrypt_with_wrap(wrap: &[u8; 32], encrypted_b64: &str) -> Result<[u8; 32], String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(wrap));
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|e| format!("base64: {e}"))?;
    if bytes.len() < NONCE_LEN + 16 {
        return Err("ciphertext too short".into());
    }
    let (nonce_bytes, ct) = bytes.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    let mut plaintext = cipher
        .decrypt(nonce, ct)
        .map_err(|e| format!("aes decrypt: {e}"))?;
    if plaintext.len() != 32 {
        plaintext.zeroize();
        return Err("seed length wrong".into());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&plaintext);
    plaintext.zeroize();
    Ok(out)
}
