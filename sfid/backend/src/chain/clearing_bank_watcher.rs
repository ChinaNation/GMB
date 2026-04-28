//! 清算行节点声明缓存(ADR-007 Step 2 阶段 D 新增)。
//!
//! 链上权威 storage 在 `OffchainTransactionPos::ClearingBankNodes`,key = sfid_id 字节串,
//! value = ClearingBankNodeInfo。SFID 后端 `app_search_clearing_banks` 的"第 2 轮过滤"
//! 需要 `AND sfid_id ∈ ClearingBankNodes` 才能保证只返回**已加入清算网络**的候选。
//!
//! 实现策略(2026-04-27 阶段 D):
//! - 启动:全量 scan 一次,把所有当前注册的 sfid_id 灌入内存 HashSet
//! - 增量:tokio task 每 30 秒重新跑全量 scan,差异更新 HashSet
//! - 容错:RPC 失败时保留上次状态,只追加 warn 日志,不清空缓存(避免误过滤)
//!
//! 为什么不订阅 finalized blocks 事件:
//! - PoW 链 finality 显著落后 best block,事件订阅延迟相对差
//! - ClearingBankNodes 写入频率极低(机构注册/注销),30 秒延迟可接受
//! - 全量 scan 实现简单,且自带"对账"语义(不依赖事件流不丢)
//! Step 3 扫码支付完整联调时,如有更高时延要求再切换为事件订阅模型。

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde_json::{json, Value};

const POLL_INTERVAL: Duration = Duration::from_secs(30);
const RPC_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_KEYS_PER_PAGE: u32 = 1000;
/// twox_128("OffchainTransactionPos") + twox_128("ClearingBankNodes") 的 hex 前缀。
/// 启动时初始化(避免反复哈希),后续读 storage 复用。
const STORAGE_PREFIX_INIT: () = ();

/// 内存中已声明清算行节点的 sfid_id 集合。
#[derive(Default)]
pub struct ClearingBankNodeCache {
    inner: RwLock<HashSet<String>>,
    /// 最近一次 scan 是否成功(供调试 / 健康检查)。
    last_scan_ok: RwLock<bool>,
}

impl ClearingBankNodeCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// 是否包含某 sfid_id。读锁极轻,handler 路径直接调。
    pub fn contains(&self, sfid_id: &str) -> bool {
        match self.inner.read() {
            Ok(g) => g.contains(sfid_id),
            Err(e) => e.into_inner().contains(sfid_id),
        }
    }

    /// 当前缓存大小(测试 / 监控用)。
    pub fn len(&self) -> usize {
        match self.inner.read() {
            Ok(g) => g.len(),
            Err(e) => e.into_inner().len(),
        }
    }

    /// 上次 scan 是否成功。健康路径:启动后 30 秒内置 false,首次 scan 完成后置 true。
    pub fn last_scan_ok(&self) -> bool {
        match self.last_scan_ok.read() {
            Ok(g) => *g,
            Err(e) => *e.into_inner(),
        }
    }

    fn replace_set(&self, new_set: HashSet<String>) {
        if let Ok(mut g) = self.inner.write() {
            *g = new_set;
        }
    }

    fn mark_scan(&self, ok: bool) {
        if let Ok(mut g) = self.last_scan_ok.write() {
            *g = ok;
        }
    }
}

/// 启动 watcher tokio task。返回 Arc 句柄,handler 端读 cache。
///
/// `chain_http_url` 由调用方传入(避免本模块再依赖 chain::url 模块)。
pub fn spawn_watcher(chain_http_url: String) -> Arc<ClearingBankNodeCache> {
    let cache = Arc::new(ClearingBankNodeCache::new());
    let cache_clone = Arc::clone(&cache);
    tokio::spawn(async move {
        run_watcher_loop(chain_http_url, cache_clone).await;
    });
    cache
}

async fn run_watcher_loop(http_url: String, cache: Arc<ClearingBankNodeCache>) {
    let _ = STORAGE_PREFIX_INIT;
    let prefix_hex = clearing_bank_nodes_prefix_hex();
    tracing::info!(
        prefix = %prefix_hex,
        url = %http_url,
        "ClearingBankWatcher 启动,storage prefix 已计算"
    );

    let mut backoff = Duration::from_secs(1);
    loop {
        match scan_once(&http_url, &prefix_hex).await {
            Ok(set) => {
                let n = set.len();
                cache.replace_set(set);
                cache.mark_scan(true);
                tracing::debug!(n, "ClearingBankWatcher 完成 scan");
                backoff = Duration::from_secs(1);
                tokio::time::sleep(POLL_INTERVAL).await;
            }
            Err(e) => {
                cache.mark_scan(false);
                tracing::warn!(error = %e, ?backoff, "ClearingBankWatcher scan 失败,稍后重试");
                tokio::time::sleep(backoff).await;
                // 指数退避,上限 60s
                backoff = (backoff * 2).min(Duration::from_secs(60));
            }
        }
    }
}

/// 计算 `twox_128(b"OffchainTransactionPos")||twox_128(b"ClearingBankNodes")` 的 hex 前缀(0x...)。
fn clearing_bank_nodes_prefix_hex() -> String {
    let pallet = twox_128(b"OffchainTransactionPos");
    let storage = twox_128(b"ClearingBankNodes");
    let mut combined = Vec::with_capacity(32);
    combined.extend_from_slice(&pallet);
    combined.extend_from_slice(&storage);
    format!("0x{}", hex::encode(combined))
}

/// 通过 `state_getKeysPaged` 拉所有 ClearingBankNodes 的 storage key,
/// 解出 key 末尾的 sfid_id 字段并返回 set。
///
/// key 编码:`pallet_prefix(16) || storage_prefix(16) || blake2_128(sfid_id)(16) || compact_u32_len(sfid_id) || sfid_id_bytes`
async fn scan_once(http_url: &str, prefix_hex: &str) -> Result<HashSet<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(RPC_TIMEOUT)
        .build()
        .map_err(|e| format!("create http client failed: {e}"))?;

    let mut total: HashSet<String> = HashSet::new();
    let mut start_key: Option<String> = None;
    loop {
        let mut params = vec![
            Value::String(prefix_hex.to_string()),
            Value::Number(serde_json::Number::from(MAX_KEYS_PER_PAGE)),
        ];
        if let Some(s) = start_key.as_ref() {
            params.push(Value::String(s.clone()));
        }
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "state_getKeysPaged",
            "params": params,
        });
        let resp = client
            .post(http_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("rpc post failed:{e}"))?;
        if !resp.status().is_success() {
            return Err(format!("rpc http {}", resp.status()));
        }
        let v: Value = resp.json().await.map_err(|e| format!("rpc json:{e}"))?;
        if let Some(err) = v.get("error") {
            return Err(format!("rpc error: {err}"));
        }
        let keys = v
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        let n = keys.len();
        for k in &keys {
            if let Some(s) = k.as_str() {
                if let Some(sfid) = decode_sfid_from_storage_key(s, prefix_hex) {
                    total.insert(sfid);
                }
            }
        }
        if n < MAX_KEYS_PER_PAGE as usize {
            break;
        }
        start_key = keys.last().and_then(|v| v.as_str().map(|s| s.to_string()));
        if start_key.is_none() {
            break;
        }
    }
    Ok(total)
}

/// 从完整 storage key 反向取出 sfid_id 字符串。
///
/// key = prefix(32B = 64 hex 字符) + blake2_128(16B) + Compact<u32>(len) + sfid_id_bytes
/// prefix_hex 含 0x,长度 2 + 64 = 66。
fn decode_sfid_from_storage_key(key_hex: &str, prefix_hex: &str) -> Option<String> {
    let key = key_hex.strip_prefix("0x").unwrap_or(key_hex);
    let prefix = prefix_hex.strip_prefix("0x").unwrap_or(prefix_hex);
    if !key.starts_with(prefix) {
        return None;
    }
    let after_prefix = &key[prefix.len()..];
    // blake2_128 = 32 hex 字符
    if after_prefix.len() < 32 {
        return None;
    }
    let after_hash = &after_prefix[32..];
    let bytes = hex::decode(after_hash).ok()?;
    // SCALE Compact<u32> 解码
    if bytes.is_empty() {
        return None;
    }
    let (len, consumed) = decode_compact_u32(&bytes)?;
    if bytes.len() < consumed + len as usize {
        return None;
    }
    let sfid = std::str::from_utf8(&bytes[consumed..consumed + len as usize]).ok()?;
    Some(sfid.to_string())
}

/// 极简的 SCALE Compact<u32> 解码,只支持 single-byte / two-byte / four-byte 模式
/// (sfid_id 上限 64 字节远未达 big-integer 模式)。返回 (value, consumed_bytes)。
fn decode_compact_u32(bytes: &[u8]) -> Option<(u32, usize)> {
    if bytes.is_empty() {
        return None;
    }
    let first = bytes[0];
    match first & 0x03 {
        0 => Some(((first >> 2) as u32, 1)),
        1 => {
            if bytes.len() < 2 {
                return None;
            }
            let v = ((first as u16) | ((bytes[1] as u16) << 8)) >> 2;
            Some((v as u32, 2))
        }
        2 => {
            if bytes.len() < 4 {
                return None;
            }
            let v = (first as u32)
                | ((bytes[1] as u32) << 8)
                | ((bytes[2] as u32) << 16)
                | ((bytes[3] as u32) << 24);
            Some((v >> 2, 4))
        }
        _ => None,
    }
}

/// twox_128 哈希(纯函数,与 substrate `sp_core::hashing::twox_128` 等价)。
fn twox_128(data: &[u8]) -> [u8; 16] {
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&twox_64(data, 0).to_le_bytes());
    out[8..].copy_from_slice(&twox_64(data, 1).to_le_bytes());
    out
}

fn twox_64(data: &[u8], seed: u64) -> u64 {
    use twox_hash::XxHash64;
    use std::hash::Hasher;
    let mut hasher = XxHash64::with_seed(seed);
    hasher.write(data);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_hex_starts_with_0x_and_correct_length() {
        let p = clearing_bank_nodes_prefix_hex();
        assert!(p.starts_with("0x"));
        // 32 字节 → 64 hex 字符 + 2 prefix
        assert_eq!(p.len(), 2 + 64);
    }

    #[test]
    fn decode_compact_u32_single_byte() {
        // 13 << 2 = 52 (mode 0)
        assert_eq!(decode_compact_u32(&[52u8, 0xAA, 0xBB]), Some((13u32, 1)));
    }

    #[test]
    fn decode_compact_u32_two_bytes() {
        // 64 → mode 1: ((64 << 2) | 0x01).to_le = 0x01 0x01
        assert_eq!(decode_compact_u32(&[0x01, 0x01, 0xAA]), Some((64u32, 2)));
    }

    #[test]
    fn decode_sfid_round_trip_single_byte_compact() {
        // 构造一个完整的 storage key:prefix(32B) + blake2_128(16B,任意) + compact(len) + sfid_bytes
        let prefix = clearing_bank_nodes_prefix_hex();
        let sfid = b"SFR-12345-AAAA-678901234-20260101";
        let blake = [0xAAu8; 16];
        let mut key_bytes = Vec::new();
        let p_bytes = hex::decode(prefix.strip_prefix("0x").unwrap()).unwrap();
        key_bytes.extend_from_slice(&p_bytes);
        key_bytes.extend_from_slice(&blake);
        key_bytes.push((sfid.len() as u8) << 2); // compact single byte
        key_bytes.extend_from_slice(sfid);
        let key_hex = format!("0x{}", hex::encode(&key_bytes));
        let decoded = decode_sfid_from_storage_key(&key_hex, &prefix).unwrap();
        assert_eq!(decoded.as_bytes(), sfid);
    }

    #[test]
    fn cache_contains_after_replace() {
        let c = ClearingBankNodeCache::new();
        let mut s = HashSet::new();
        s.insert("SFR-X".to_string());
        c.replace_set(s);
        assert!(c.contains("SFR-X"));
        assert!(!c.contains("SFR-Y"));
        assert_eq!(c.len(), 1);
    }
}
