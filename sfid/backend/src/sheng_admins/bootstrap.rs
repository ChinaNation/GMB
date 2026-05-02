//! 中文注释:省管理员首次登录时的签名 keypair bootstrap。
//!
//! ADR-008 决议(2026-05-01):每个 admin slot 独立签名密钥。
//!
//! 行为:
//! 1. 若 store_shards/sheng_signer 已存在加密 seed → 解密 → 构造 Pair → 载入 cache
//! 2. 否则 → 随机生成 32 字节 seed → 加密落盘 → 构造 Pair → 载入 cache
//!
//! 注意:本期**不**推链(`activate_sheng_signing_pubkey` 留 Phase 4 子卡)。
//! Phase 4 接入后,首次 bootstrap 还需要推链记录签名公钥到 ShengAdmins 旁边的
//! ShengSigningPubkey storage。

#![allow(dead_code)]

use sp_core::{sr25519, Pair};
use zeroize::Zeroizing;

use crate::sheng_admins::signing_cache::{ShengSigningCache, Sr25519Pair};
use crate::store_shards::sheng_signer::{load_seed, save_seed};

/// bootstrap 失败原因。
#[derive(Debug)]
pub(crate) enum BootstrapError {
    Rng(String),
    Persist(String),
    Load(String),
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootstrapError::Rng(s) => write!(f, "rng error: {s}"),
            BootstrapError::Persist(s) => write!(f, "persist error: {s}"),
            BootstrapError::Load(s) => write!(f, "load error: {s}"),
        }
    }
}

impl std::error::Error for BootstrapError {}

/// 确保 (province, admin_pubkey) 的签名 Pair 已就绪并载入 cache。
///
/// 返回签名公钥(33 byte sr25519,但实际取 .public().0 即 32 byte)。
pub(crate) fn ensure_signing_keypair(
    cache: &ShengSigningCache,
    province: &str,
    admin_pubkey: &[u8; 32],
) -> Result<Sr25519Pair, BootstrapError> {
    if let Some(existing) = cache.get(province, admin_pubkey) {
        return Ok(existing);
    }

    let seed = match load_seed(province, admin_pubkey).map_err(BootstrapError::Load)? {
        Some(s) => s,
        None => {
            // 首次登录:随机生成 seed → 加密持久化。
            let mut seed_arr: Zeroizing<[u8; 32]> = Zeroizing::new([0u8; 32]);
            getrandom::getrandom(seed_arr.as_mut_slice())
                .map_err(|e| BootstrapError::Rng(e.to_string()))?;
            save_seed(province, admin_pubkey, &seed_arr)
                .map_err(BootstrapError::Persist)?;
            *seed_arr
        }
    };

    let pair = sr25519::Pair::from_seed(&seed);
    cache.load(province.to_string(), *admin_pubkey, pair.clone());
    Ok(pair)
}
