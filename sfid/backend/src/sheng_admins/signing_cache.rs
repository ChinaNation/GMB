//! 中文注释:省管理员 3-tier 签名密钥进程内缓存。
//!
//! ADR-008 决议(2026-05-01):每省 3 把独立签名密钥(每个 admin slot 各一把,
//! 互不共享)。登录时根据 (province, admin_pubkey) 载入 Pair,登出/idle 驱逐。
//!
//! 与旧 `key_admins::sheng_signer_cache` 区别:
//! - 旧:每省 1 把签名密钥,key 是 province
//! - 新:每省 3 把(main / backup_1 / backup_2),key 是 (province, admin_pubkey)
//!
//! 业务凭证签发(institutions / citizens / shi_admins)统一从本 cache 取 Pair。

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use sp_core::{sr25519, Pair};

/// 缓存键:(省名, admin 公钥 32 字节)。
pub(crate) type CacheKey = (String, [u8; 32]);

/// 省级签名 keypair 类型别名。
pub(crate) type Sr25519Pair = sr25519::Pair;

pub(crate) struct ShengSigningCache {
    inner: Mutex<HashMap<CacheKey, Sr25519Pair>>,
}

impl ShengSigningCache {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// 登录时把 (province, admin_pubkey) → Pair 载入 cache。
    pub(crate) fn load(&self, province: String, admin_pubkey: [u8; 32], pair: Sr25519Pair) {
        if let Ok(mut g) = self.inner.lock() {
            g.insert((province, admin_pubkey), pair);
        }
    }

    /// 登出时驱逐。
    pub(crate) fn evict(&self, province: &str, admin_pubkey: &[u8; 32]) {
        if let Ok(mut g) = self.inner.lock() {
            g.remove(&(province.to_string(), *admin_pubkey));
        }
    }

    /// 取签名 Pair(凭证签发用)。
    pub(crate) fn get(&self, province: &str, admin_pubkey: &[u8; 32]) -> Option<Sr25519Pair> {
        self.inner
            .lock()
            .ok()?
            .get(&(province.to_string(), *admin_pubkey))
            .cloned()
    }

    /// 当前 cache 内活跃 (province, admin) 对总数。
    pub(crate) fn active_count(&self) -> usize {
        self.inner.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// 由调用方测试用:从 32 字节 seed 构造 Pair。
    pub(crate) fn pair_from_seed(seed: &[u8; 32]) -> Sr25519Pair {
        sr25519::Pair::from_seed(seed)
    }
}

impl Default for ShengSigningCache {
    fn default() -> Self {
        Self::new()
    }
}
