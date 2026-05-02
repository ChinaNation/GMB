//! 中文注释:SFID → 链上 push extrinsic 共享 helper(ADR-008 phase7 真实实现)。
//!
//! ## 背景
//!
//! ADR-008 决议:SFID 仅 push 4 个 `Pays::No` extrinsic
//! (add_sheng_admin_backup / remove_sheng_admin_backup /
//! activate_sheng_signing_pubkey / rotate_sheng_signing_pubkey)。
//!
//! 推链三件套(见 `feedback_sfid_pow_chain_recipe.md`):
//! 1. 显式 nonce(SFID 后端生成 32 字节随机数,链端 ValidateUnsigned 防重放)
//! 2. immortal extrinsic(unsigned 自带 immortal,SFID 不绑 mortality)
//! 3. 等 InBestBlock(只确认 block 包含,不等 finalized;PoW 链 finalized 滞后)
//! 4. `Pays::No` 转换为 1010 错误的 fail-fast 友好提示(由 runtime extrinsic 标注)
//!
//! ## phase7 真实实现
//!
//! 本模块通过 subxt `OnlineClient<PolkadotConfig>` 把裸 SCALE 编码的 unsigned
//! extrinsic 提交到链端。提交链路:
//!
//! ```text
//! call_bytes (pallet_idx ++ call_idx ++ args)
//!   → V4 BARE 包装(version_byte=4 + compact_len 前缀)
//!   → SubmittableTransaction::from_bytes(client, raw)
//!   → submit_and_watch()
//!   → 等 TxStatus::InBestBlock → 返回 H256 tx hash
//! ```
//!
//! WebSocket 不可达时按指数退避重试 3 次,失败映射为 [`ChainPushError::Other`]。
//!
//! ## 与 `chain/runtime_align.rs` 的关系
//!
//! `runtime_align.rs` 负责 **凭证签名 / SCALE 编码 / genesis_hash 缓存**(chain pull
//! 凭证,SFID main signer)。本文件负责 **chain push 推 extrinsic**。两者职责不重叠,
//! 但都依赖 subxt OnlineClient — 本文件维护独立 OnceCell client 单例,避开 runtime_align
//! 的内部细节(若未来合并需求清晰再统一)。

#![allow(dead_code)]

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use parity_scale_codec::{Compact, Encode};
use subxt::{tx::SubmittableTransaction, utils::H256, OnlineClient, PolkadotConfig};
use tokio::sync::OnceCell;

/// SFID 推链 pallet 在 citizenchain runtime 中的 pallet_index。
/// 与 `citizenchain/runtime/src/lib.rs::SfidSystem` 的 `#[runtime::pallet_index(10)]` 严格对齐。
pub(crate) const SFID_SYSTEM_PALLET_INDEX: u8 = 10;

/// `add_sheng_admin_backup` 的 call_index(`#[pallet::call_index(2)]`)。
pub(crate) const CALL_INDEX_ADD_SHENG_ADMIN_BACKUP: u8 = 2;
/// `remove_sheng_admin_backup` 的 call_index(`#[pallet::call_index(3)]`)。
pub(crate) const CALL_INDEX_REMOVE_SHENG_ADMIN_BACKUP: u8 = 3;
/// `activate_sheng_signing_pubkey` 的 call_index(`#[pallet::call_index(4)]`)。
pub(crate) const CALL_INDEX_ACTIVATE_SHENG_SIGNING: u8 = 4;
/// `rotate_sheng_signing_pubkey` 的 call_index(`#[pallet::call_index(5)]`)。
pub(crate) const CALL_INDEX_ROTATE_SHENG_SIGNING: u8 = 5;

/// 推链返回的稳定 TxHash 包装类型。
///
/// `hex` 字段保留 0x 小写 hex(64 字符 + 0x 前缀,共 66 字符),与既有 handler 输出格式
/// 兼容(`tx_hash: tx.hex`);phase45 历史占位类型已在 phase7 切真时移除。
#[derive(Debug, Clone)]
pub(crate) struct TxHash {
    pub(crate) hex: String,
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex)
    }
}

impl TxHash {
    /// 从 subxt `H256` 构造 0x 小写 hex(死规则 `feedback_pubkey_format_rule.md`)。
    pub(crate) fn from_h256(hash: H256) -> Self {
        Self {
            hex: format!("0x{}", hex::encode(hash.0)),
        }
    }
}

/// 推链调用统一错误。
#[derive(Debug)]
pub(crate) enum ChainPushError {
    /// 真实推链阶段:1010 InvalidTransaction(签名 / nonce / 验证失败)。
    InvalidTx(String),
    /// 其它链端错误(连接失败、解码失败、超时等)。
    Other(String),
}

impl fmt::Display for ChainPushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainPushError::InvalidTx(s) => write!(f, "invalid tx: {s}"),
            ChainPushError::Other(s) => write!(f, "chain push error: {s}"),
        }
    }
}

impl std::error::Error for ChainPushError {}

// ─── subxt OnlineClient 单例 ────────────────────────────────────────────

/// 进程级 subxt 客户端(懒初始化,失败可重试)。
static SUBXT_CLIENT: OnceCell<Arc<OnlineClient<PolkadotConfig>>> = OnceCell::const_new();

/// 取共享 subxt OnlineClient。失败时按指数退避重试 3 次。
async fn online_client() -> Result<Arc<OnlineClient<PolkadotConfig>>, ChainPushError> {
    if let Some(c) = SUBXT_CLIENT.get() {
        return Ok(c.clone());
    }
    let client = SUBXT_CLIENT
        .get_or_try_init(|| async {
            let ws_url = super::url::chain_ws_url().map_err(ChainPushError::Other)?;
            connect_with_retries(ws_url.as_str()).await
        })
        .await?;
    Ok(client.clone())
}

async fn connect_with_retries(
    ws_url: &str,
) -> Result<Arc<OnlineClient<PolkadotConfig>>, ChainPushError> {
    let mut last_err = String::new();
    for attempt in 1..=3 {
        match OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url).await {
            Ok(c) => {
                tracing::info!(ws_url = %ws_url, attempt, "subxt OnlineClient connected");
                return Ok(Arc::new(c));
            }
            Err(e) => {
                last_err = e.to_string();
                let backoff_ms = 200u64 * (1 << (attempt - 1)); // 200ms, 400ms, 800ms
                tracing::warn!(
                    attempt,
                    backoff_ms,
                    error = %last_err,
                    "subxt connect failed, will retry"
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }
    }
    Err(ChainPushError::Other(format!(
        "chain unreachable after 3 retries: {last_err}"
    )))
}

// ─── extrinsic 包装 ────────────────────────────────────────────────────

/// 把裸 call bytes(`pallet_idx ++ call_idx ++ args`)包装成 V4 BARE unsigned
/// extrinsic 的完整 wire 格式:`Compact(len) ++ 0x04 ++ call_bytes`。
///
/// 与 substrate `generic::UncheckedExtrinsic` 的 `Preamble::Bare(4)` 编码完全一致
/// (见 `polkadot-sdk/.../generic/unchecked_extrinsic.rs`):
///   `tx_version | BARE_EXTRINSIC = 4 | 0 = 4`,然后整体走 `Compact(N)` 长度前缀。
///
/// `subxt::tx::SubmittableTransaction::from_bytes` 期望的就是这串 wire 字节。
pub(crate) fn wrap_v4_bare(call_bytes: &[u8]) -> Vec<u8> {
    let mut inner = Vec::with_capacity(1 + call_bytes.len());
    // V4 BARE: version_byte = LEGACY_EXTRINSIC_FORMAT_VERSION (4) | BARE_EXTRINSIC (0) = 4
    inner.push(4u8);
    inner.extend_from_slice(call_bytes);

    let len = Compact(inner.len() as u32);
    let mut wire = Vec::with_capacity(len.size_hint() + inner.len());
    len.encode_to(&mut wire);
    wire.extend(inner);
    wire
}

/// SFID 推链三件套封装(phase7 真实实现)。
///
/// 入参 `call_bytes` 由调用方按裸 SCALE 编码:
///   `pallet_idx (1B) ++ call_idx (1B) ++ args (SCALE)`。
///
/// 行为:
/// 1. 取 subxt OnlineClient(失败按指数退避重试 3 次)
/// 2. V4 BARE 包装:`Compact(len) ++ 0x04 ++ call_bytes`
/// 3. `SubmittableTransaction::from_bytes(client, raw)` → `submit_and_watch()`
/// 4. 阻塞等到 `TxStatus::InBestBlock`,返回 H256 tx hash
/// 5. `TxStatus::Invalid` → [`ChainPushError::InvalidTx`](含链端错误 message)
/// 6. `TxStatus::Error` / `Dropped` / 流提前结束 → [`ChainPushError::Other`]
///
/// `extrinsic_label` 仅用于 tracing 日志(grep 定位用)。
pub(crate) async fn submit_immortal_paysno(
    extrinsic_label: &'static str,
    call_bytes: Vec<u8>,
) -> Result<TxHash, ChainPushError> {
    let client = online_client().await?;
    let raw = wrap_v4_bare(&call_bytes);

    tracing::info!(
        extrinsic = extrinsic_label,
        wire_len = raw.len(),
        "[chain push] submitting immortal Pays::No extrinsic"
    );

    let tx: SubmittableTransaction<PolkadotConfig, OnlineClient<PolkadotConfig>> =
        SubmittableTransaction::from_bytes((*client).clone(), raw);

    let mut progress = tx.submit_and_watch().await.map_err(|e| {
        let msg = e.to_string();
        // 1010 InvalidTransaction 通常以 "Invalid Transaction" / "1010" 字样从底层冒上来。
        if msg.contains("1010") || msg.to_ascii_lowercase().contains("invalid") {
            ChainPushError::InvalidTx(msg)
        } else {
            ChainPushError::Other(msg)
        }
    })?;

    use subxt::tx::TxStatus;
    while let Some(status) = progress.next().await {
        match status {
            Ok(TxStatus::InBestBlock(in_block)) => {
                let hash = in_block.extrinsic_hash();
                tracing::info!(
                    extrinsic = extrinsic_label,
                    tx_hash = %format!("0x{}", hex::encode(hash.0)),
                    "[chain push] InBestBlock"
                );
                return Ok(TxHash::from_h256(hash));
            }
            Ok(TxStatus::InFinalizedBlock(in_block)) => {
                // 防御:虽然推链铁律是只等 InBestBlock,但若链端先返回 finalized 也兼容收尾。
                let hash = in_block.extrinsic_hash();
                return Ok(TxHash::from_h256(hash));
            }
            Ok(TxStatus::Invalid { message }) => {
                return Err(ChainPushError::InvalidTx(message));
            }
            Ok(TxStatus::Error { message }) => {
                return Err(ChainPushError::Other(format!("tx error: {message}")));
            }
            Ok(TxStatus::Dropped { message }) => {
                return Err(ChainPushError::Other(format!("tx dropped: {message}")));
            }
            Ok(_) => continue, // Validated / Broadcasted / NoLongerInBestBlock: 等下一个状态
            Err(e) => return Err(ChainPushError::Other(e.to_string())),
        }
    }
    Err(ChainPushError::Other(
        "tx progress stream terminated without InBestBlock".to_string(),
    ))
}

/// 生成 32 字节随机 nonce(链端 ValidateUnsigned 防重放用)。
pub(crate) fn generate_sheng_nonce() -> Result<[u8; 32], ChainPushError> {
    let mut nonce = [0u8; 32];
    getrandom::getrandom(&mut nonce).map_err(|e| ChainPushError::Other(e.to_string()))?;
    Ok(nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_v4_bare_prepends_compact_len_and_version_byte() {
        // 构造一个 3 字节假 call:[pallet_idx=10, call_idx=2, dummy_arg=0xAB]
        let call = vec![10u8, 2, 0xAB];
        let wire = wrap_v4_bare(&call);

        // inner = [0x04, 10, 2, 0xAB] 共 4 字节;Compact<u32>(4) = 0x10(单字节,< 64 走 single-mode)
        assert_eq!(wire[0], 0x10, "compact length prefix for 4 bytes = 0x10");
        assert_eq!(wire[1], 0x04, "v4 bare version byte");
        assert_eq!(wire[2], 10, "pallet idx");
        assert_eq!(wire[3], 2, "call idx");
        assert_eq!(wire[4], 0xAB, "arg byte");
        assert_eq!(wire.len(), 5);
    }

    #[test]
    fn wrap_v4_bare_handles_longer_payloads() {
        // 100 字节 call → inner 101 字节 → Compact(101) = 2 字节(模式 1)
        let call = vec![0u8; 100];
        let wire = wrap_v4_bare(&call);
        // Compact<u32> 模式 1: (n << 2) | 0b01 当 n<2^14。 101*4+1 = 405 = 0x195 → little-endian [0x95, 0x01]
        assert_eq!(wire[0], 0x95);
        assert_eq!(wire[1], 0x01);
        assert_eq!(wire[2], 0x04);
        assert_eq!(wire.len(), 2 + 101);
    }

    #[test]
    fn generate_sheng_nonce_returns_random_32_bytes() {
        let a = generate_sheng_nonce().unwrap();
        let b = generate_sheng_nonce().unwrap();
        assert_eq!(a.len(), 32);
        assert_ne!(a, b, "two random nonces should differ");
    }

    #[test]
    fn tx_hash_from_h256_uses_lowercase_0x_hex() {
        let h = H256::from([0xAB; 32]);
        let tx = TxHash::from_h256(h);
        assert_eq!(tx.hex.len(), 66);
        assert!(tx.hex.starts_with("0x"));
        assert!(tx.hex.chars().skip(2).all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn chain_push_error_display_carries_payload() {
        let err = ChainPushError::InvalidTx("nonce stale".to_string());
        assert!(err.to_string().contains("nonce stale"));
        let err = ChainPushError::Other("ws closed".to_string());
        assert!(err.to_string().contains("ws closed"));
    }
}
