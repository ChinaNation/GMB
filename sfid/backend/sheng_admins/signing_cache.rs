//! 中文注释:省管理员 3-tier 签名密钥进程内缓存。
//!
//! ADR-008 决议(2026-05-01):每省 3 把独立签名密钥(每个 admin slot 各一把,
//! 互不共享)。登录时根据 (province, admin_pubkey) 载入 Pair,登出/idle 驱逐。
//!
//! 缓存粒度:每省 3 把(main / backup_1 / backup_2),key 是 (province, admin_pubkey)
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

    /// 中文注释:本省任意已登录 admin slot 的签名 Pair(chain pull 凭证签发用)。
    ///
    /// ADR-008:每省 3 把签名密钥(main / backup_1 / backup_2)。链端
    /// `propose_create_institution` 验签时,只要凭证签名能对上链上记录的本省任一
    /// 当前 ShengSigningPubkey,就放行。本函数返回 cache 中第一把命中本省的 Pair,
    /// 调用方不应假定具体是哪一把。
    pub(crate) fn any_for_province(&self, province: &str) -> Option<Sr25519Pair> {
        self.any_for_province_with_admin(province)
            .map(|(_, pair)| pair)
    }

    /// 中文注释:本省任意已登录 admin slot 的 `(admin_pubkey, Pair)`。
    ///
    /// 机构注册信息凭证需要把 `signer_admin_pubkey` 一并返回给链端:
    /// 链端先用 `(province, signer_admin_pubkey)` 查本省签名公钥,再验签。
    /// 因此这里不能只返回 Pair,否则链端无法定位是哪一个省级 admin slot 签发。
    pub(crate) fn any_for_province_with_admin(
        &self,
        province: &str,
    ) -> Option<([u8; 32], Sr25519Pair)> {
        let g = self.inner.lock().ok()?;
        g.iter()
            .find(|((p, _), _)| p == province)
            .map(|((_, admin_pubkey), pair)| (*admin_pubkey, pair.clone()))
    }

    /// 中文注释:驱逐本省所有 slot 的 cache(登出 / session 过期时使用)。
    pub(crate) fn unload_province(&self, province: &str) {
        if let Ok(mut g) = self.inner.lock() {
            g.retain(|(p, _), _| p != province);
        }
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

/// 中文注释:业务推链 signer 路由(ADR-008 Phase 23e)。
///
/// 当前业务推链规则:
/// - ShengAdmin / ShiAdmin:取登录态对应省的 cache(ShengAdmin 用自己 pubkey,
///   ShiAdmin 用其上级 sheng admin pubkey;后者实现走 `any_for_province` 退化)
/// - 任何角色 cache 缺失:返回 503,提示让本省 admin 先登录
pub(crate) fn resolve_business_signer(
    state: &crate::AppState,
    ctx: &crate::login::AdminAuthContext,
) -> Result<(Sr25519Pair, String), (axum::http::StatusCode, String)> {
    let province = ctx.admin_province.as_deref().ok_or_else(|| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            "管理员缺少省份信息".to_string(),
        )
    })?;
    // 优先按 (province, ctx.admin_pubkey) 精确取(ShengAdmin 自己登录场景)
    let admin_bytes = crate::login::parse_sr25519_pubkey_bytes(ctx.admin_pubkey.as_str());
    if let Some(bytes) = admin_bytes {
        if let Some(pair) = state.sheng_admin_signing_cache.get(province, &bytes) {
            return Ok((pair, province.to_string()));
        }
    }
    // 退化:任意已上线 slot(ShiAdmin 推链场景,凭其上级 sheng 已登录)
    state
        .sheng_admin_signing_cache
        .any_for_province(province)
        .map(|p| (p, province.to_string()))
        .ok_or_else(|| {
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                format!("本省({province})登录管理员未在线，暂无法推链"),
            )
        })
}
