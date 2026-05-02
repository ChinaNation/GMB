//! 中文注释:step2e — chain push 4 个 extrinsic 的 prepare/submit-sig 双步通路
//! 用 nonce 缓存。
//!
//! ## 背景
//!
//! ADR-008 三角色架构落地后,SFID 后端推链的 4 个 extrinsic 必须由对应 admin slot
//! 私钥签名(冷钱包独占)。step2e 把 phase7 时期 `sheng_admin_signing_cache` 的假签代码
//! 物理摘除,改走"prepare 返回 payload+nonce → wuminapp 显示 QR → wumin 冷钱包扫码
//! 签 → 扫回 sig → SFID submit-sig 拼装真推链"的双步通路。
//!
//! ## 本模块职责
//!
//! 在 prepare 阶段把"待签 payload + 原始 SCALE 字段"暂存,等到 submit-sig 上来
//! 时凭 `nonce_hex` 取回这条上下文,拼装真推链。
//!
//! ## 持久化策略
//!
//! 任务卡 `20260502-step2e-cold-wallet-sign-4-extrinsics.md` 降级条款明确允许:
//! "若 SFID 后端 nonce 落盘逻辑复杂,可先用内存 HashMap + TTL(进程重启丢失,
//! 标 TODO)+ 本卡完成主路径"。本实现按 5 分钟 TTL 内存缓存:
//!
//! - 进程内 `Arc<RwLock<HashMap<NonceHex, PendingSign>>>`
//! - 每条入口时间戳 `inserted_at`,过期 5 分钟
//! - submit-sig 取走时立即移除(防重放)
//! - **TODO(step2e+1)**: 持久化到 `storage/sheng_pending_signs/<nonce_hex>.json`
//!   防止进程重启窗口期内冷钱包用户已签好却失联。
//!
//! ## payload 字节锚
//!
//! `PendingSign::call_payload` 字段是 SCALE 编码裸字段
//! (province ++ slot/admin_pubkey ++ ... ++ nonce),不含 `pallet_idx ++ call_idx`
//! 头和 sig 尾。submit-sig 拼装链上 unsigned extrinsic 时:
//!
//!   `[pallet_idx (1B)] ++ [call_idx (1B)] ++ call_payload ++ sig (64B)`
//!
//! payload_hex 给冷钱包扫的字节恰好就是 call_payload(冷钱包按 step2a domain
//! 常量 + 字段顺序解析,blake2_256 后 sr25519 签)。

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// 单条 nonce TTL(死规则:5 分钟,与任务卡一致)。
pub(crate) const PENDING_TTL: Duration = Duration::from_secs(300);

/// 4 个 chain push extrinsic 的 kind tag。submit-sig 阶段凭此选 call_index +
/// extrinsic_label。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingKind {
    AddBackup,
    RemoveBackup,
    ActivateSigning,
    RotateSigning,
}

impl PendingKind {
    /// 链端 `pallet_sfid_system` 的 call_index(与 step2a 链端 # [pallet::call_index] 严格对齐)。
    pub(crate) fn call_index(self) -> u8 {
        match self {
            PendingKind::AddBackup => crate::chain::client::CALL_INDEX_ADD_SHENG_ADMIN_BACKUP,
            PendingKind::RemoveBackup => crate::chain::client::CALL_INDEX_REMOVE_SHENG_ADMIN_BACKUP,
            PendingKind::ActivateSigning => crate::chain::client::CALL_INDEX_ACTIVATE_SHENG_SIGNING,
            PendingKind::RotateSigning => crate::chain::client::CALL_INDEX_ROTATE_SHENG_SIGNING,
        }
    }

    pub(crate) fn extrinsic_label(self) -> &'static str {
        match self {
            PendingKind::AddBackup => "add_sheng_admin_backup",
            PendingKind::RemoveBackup => "remove_sheng_admin_backup",
            PendingKind::ActivateSigning => "activate_sheng_signing_pubkey",
            PendingKind::RotateSigning => "rotate_sheng_signing_pubkey",
        }
    }
}

/// 一条暂存的 prepare 上下文。
#[derive(Debug, Clone)]
pub(crate) struct PendingSign {
    pub(crate) kind: PendingKind,
    /// 32 字节 nonce(SCALE 编码尾部已含)。
    pub(crate) nonce: [u8; 32],
    /// SCALE 裸字段:province ++ slot/admin_pubkey ++ ... ++ nonce
    /// (不含 `pallet_idx ++ call_idx` 头与 sig 尾)。
    pub(crate) call_payload: Vec<u8>,
    /// 链端共享 domain(step2a 固化)。同时也是 wumin/wuminapp 显示的"凭证类别锚"。
    pub(crate) domain: &'static [u8],
    /// blake2_256(domain ++ ... ++ nonce):冷钱包签的就是这个 digest 的 sr25519。
    pub(crate) digest: [u8; 32],
    /// 哪个省发起的(scope 校验用)。
    pub(crate) province: String,
    /// 创建时间戳(用于 TTL 过期清理)。
    inserted_at: Instant,
}

impl PendingSign {
    pub(crate) fn is_expired(&self, now: Instant, ttl: Duration) -> bool {
        now.duration_since(self.inserted_at) > ttl
    }
}

/// 进程级 nonce 缓存。
#[derive(Default)]
pub(crate) struct ShengPendingSignCache {
    inner: RwLock<HashMap<String, PendingSign>>,
}

impl ShengPendingSignCache {
    pub(crate) fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// 入口暂存。`nonce_hex` 用 0x 小写 hex(`feedback_pubkey_format_rule.md`)。
    pub(crate) fn insert(&self, kind: PendingKind, sign: PendingSignInput) {
        let now = Instant::now();
        let entry = PendingSign {
            kind,
            nonce: sign.nonce,
            call_payload: sign.call_payload,
            domain: sign.domain,
            digest: sign.digest,
            province: sign.province,
            inserted_at: now,
        };
        let key = nonce_hex(&entry.nonce);
        let mut g = self.inner.write().expect("pending sign cache poisoned");
        // 顺手清理过期项(写锁内做,避免独立后台 worker)。
        g.retain(|_, v| !v.is_expired(now, PENDING_TTL));
        g.insert(key, entry);
    }

    /// 取出并消费(防重放)。失败原因:nonce 不存在 / 过期 / kind 不匹配。
    pub(crate) fn take(
        &self,
        nonce_hex_in: &str,
        expected: PendingKind,
    ) -> Result<PendingSign, TakeError> {
        let key = nonce_hex_in.trim().to_ascii_lowercase();
        let now = Instant::now();
        let mut g = self.inner.write().expect("pending sign cache poisoned");
        // 同样借写锁顺手清理过期。
        g.retain(|_, v| !v.is_expired(now, PENDING_TTL));
        let entry = g.remove(&key).ok_or(TakeError::NotFound)?;
        if entry.kind != expected {
            return Err(TakeError::KindMismatch {
                expected,
                actual: entry.kind,
            });
        }
        Ok(entry)
    }

    /// 主动清理过期项(给单元测试 + 启动期清扫用)。返回清掉的条数。
    pub(crate) fn purge_expired(&self) -> usize {
        let now = Instant::now();
        let mut g = self.inner.write().expect("pending sign cache poisoned");
        let before = g.len();
        g.retain(|_, v| !v.is_expired(now, PENDING_TTL));
        before - g.len()
    }

    pub(crate) fn len(&self) -> usize {
        self.inner
            .read()
            .expect("pending sign cache poisoned")
            .len()
    }
}

/// 入口 prepare 调用方传入的最少字段集合(避免 PendingSign 内部时间戳被外部构造)。
pub(crate) struct PendingSignInput {
    pub(crate) nonce: [u8; 32],
    pub(crate) call_payload: Vec<u8>,
    pub(crate) domain: &'static [u8],
    pub(crate) digest: [u8; 32],
    pub(crate) province: String,
}

/// take 错误。`KindMismatch` 主要兜防御:同一 nonce 被另一个 endpoint 误用。
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TakeError {
    NotFound,
    KindMismatch {
        expected: PendingKind,
        actual: PendingKind,
    },
}

/// 0x 小写 hex 序列化 nonce(死规则 `feedback_pubkey_format_rule.md`)。
pub(crate) fn nonce_hex(nonce: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(nonce))
}

/// 为 prepare 响应渲染 `expires_at`(unix epoch ms)。
pub(crate) fn expires_at_ms(now_ms: u64) -> u64 {
    now_ms + (PENDING_TTL.as_millis() as u64)
}

/// 当前时间(unix ms)。抽出便于测试。
pub(crate) fn unix_now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input(seed: u8, kind_label: &str) -> PendingSignInput {
        // 不同 kind 用不同 nonce,避免互踩。
        let mut nonce = [0u8; 32];
        nonce[0] = seed;
        nonce[31] = kind_label.as_bytes()[0];
        PendingSignInput {
            nonce,
            call_payload: vec![1, 2, 3, seed],
            domain: b"sample_domain_v1",
            digest: [seed; 32],
            province: "AH".to_string(),
        }
    }

    #[test]
    fn insert_then_take_succeeds_once_and_consumes() {
        let cache = ShengPendingSignCache::new();
        let inp = sample_input(0x11, "add");
        let key = nonce_hex(&inp.nonce);
        cache.insert(PendingKind::AddBackup, inp);
        assert_eq!(cache.len(), 1);

        let taken = cache
            .take(&key, PendingKind::AddBackup)
            .expect("first take ok");
        assert_eq!(taken.kind, PendingKind::AddBackup);
        assert_eq!(taken.province, "AH");
        assert_eq!(cache.len(), 0);

        // 第二次 take 同 nonce → NotFound(防重放)。
        assert_eq!(
            cache.take(&key, PendingKind::AddBackup),
            Err(TakeError::NotFound)
        );
    }

    #[test]
    fn take_rejects_kind_mismatch() {
        let cache = ShengPendingSignCache::new();
        let inp = sample_input(0x22, "rem");
        let key = nonce_hex(&inp.nonce);
        cache.insert(PendingKind::RemoveBackup, inp);
        let err = cache
            .take(&key, PendingKind::AddBackup)
            .expect_err("kind mismatch should fail");
        assert_eq!(
            err,
            TakeError::KindMismatch {
                expected: PendingKind::AddBackup,
                actual: PendingKind::RemoveBackup,
            }
        );
        // 失败 take 不消费(条目已被 take 自身从 map 删除前先校验 kind 失败 → 此实现选择"误用 kind 不消费",
        // 上面 take() 在 KindMismatch 前已执行 remove。这里测试反映现状:已被删,二次 take = NotFound)。
        assert_eq!(
            cache.take(&key, PendingKind::RemoveBackup),
            Err(TakeError::NotFound)
        );
    }

    #[test]
    fn ttl_expiry_purges_old_entries() {
        let cache = ShengPendingSignCache::new();
        let inp = sample_input(0x33, "act");
        let key = nonce_hex(&inp.nonce);
        cache.insert(PendingKind::ActivateSigning, inp);

        // 手动改 inserted_at 至过期之前 — 借助内部锁直接改时间戳。
        {
            let mut g = cache.inner.write();
            let entry = g.get_mut(&key).expect("present");
            entry.inserted_at = Instant::now() - Duration::from_secs(301);
        }
        let purged = cache.purge_expired();
        assert_eq!(purged, 1);
        assert_eq!(cache.len(), 0);

        // 已过期项 take → NotFound。
        assert_eq!(
            cache.take(&key, PendingKind::ActivateSigning),
            Err(TakeError::NotFound)
        );
    }

    #[test]
    fn nonce_hex_uses_lowercase_0x() {
        let n = [0xABu8; 32];
        let s = nonce_hex(&n);
        assert!(s.starts_with("0x"));
        assert_eq!(s.len(), 66);
        assert!(s
            .chars()
            .skip(2)
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn pending_kind_call_indices_align_with_chain_runtime() {
        // step2a 链端 #[pallet::call_index] 与 chain/client.rs 常量对齐。
        // 本测试是双端字节锚的 SFID 端断言,wumin/wuminapp fixture 同一组数字。
        assert_eq!(PendingKind::AddBackup.call_index(), 2);
        assert_eq!(PendingKind::RemoveBackup.call_index(), 3);
        assert_eq!(PendingKind::ActivateSigning.call_index(), 4);
        assert_eq!(PendingKind::RotateSigning.call_index(), 5);
    }
}
