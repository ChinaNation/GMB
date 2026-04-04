//! 链下交易批量打包器。
//!
//! 后台任务，监控链下账本中的待结算交易，
//! 达到笔数阈值或时间阈值时自动打包并提交上链。

use crate::offchain_keystore::SigningKey;
use crate::offchain_ledger::{OffchainLedger, OffchainTxItem};
use sp_core::{sr25519, Pair};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 中文注释：node 端 blake2b-256 哈希。
fn blake2_256(data: &[u8]) -> [u8; 32] {
    let hash = blake2b_simd::Params::new().hash_length(32).hash(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// 中文注释：打包触发笔数阈值（10 万笔）。
const PACK_TX_THRESHOLD: usize = 100_000;

/// 中文注释：打包触发区块间隔阈值（10 个区块 ≈ 60 分钟）。
const PACK_BLOCK_THRESHOLD: u64 = 10;

/// 批量打包器状态。
pub struct OffchainPacker {
    /// 链下账本。
    ledger: OffchainLedger,
    /// 签名管理员密钥（内存中）。
    signing_key: Arc<RwLock<Option<SigningKey>>>,
    /// 省储行 shenfen_id。
    shenfen_id: String,
    /// 上次打包的区块号。
    last_pack_block: Arc<RwLock<u64>>,
    /// 节点启动密码（用于账本持久化）。
    password: String,
    /// 链下清算广播发送端（用于结算后通知其他省储行）。
    gossip_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::offchain_gossip::OffchainGossipMessage>>,
}

impl OffchainPacker {
    /// 中文注释：创建打包器。
    pub fn new(
        ledger: OffchainLedger,
        signing_key: SigningKey,
        password: String,
        gossip_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::offchain_gossip::OffchainGossipMessage>>,
    ) -> Self {
        let shenfen_id = signing_key.shenfen_id.clone();
        Self {
            ledger,
            signing_key: Arc::new(RwLock::new(Some(signing_key))),
            shenfen_id,
            last_pack_block: Arc::new(RwLock::new(0)),
            password,
            gossip_tx,
        }
    }

    /// 中文注释：检查是否应该触发打包。
    pub async fn should_pack(&self, current_block: u64) -> bool {
        let count = self.ledger.pending_count();
        if count == 0 {
            return false;
        }

        // 笔数阈值
        if count >= PACK_TX_THRESHOLD {
            log::info!(
                "[Offchain] 触发打包（笔数阈值）：{count} 笔 >= {PACK_TX_THRESHOLD}"
            );
            return true;
        }

        // 时间阈值
        let last = *self.last_pack_block.read().await;
        if last == 0 {
            // 首次打包，等待时间阈值
            if current_block >= PACK_BLOCK_THRESHOLD {
                log::info!(
                    "[Offchain] 触发打包（时间阈值，首次）：区块 {current_block}"
                );
                return true;
            }
        } else if current_block.saturating_sub(last) >= PACK_BLOCK_THRESHOLD {
            log::info!(
                "[Offchain] 触发打包（时间阈值）：距上次打包 {} 个区块",
                current_block - last
            );
            return true;
        }

        false
    }

    /// 中文注释：执行打包。取出所有待上链交易，签署 batch，返回打包数据。
    ///
    /// 返回 (batch_items, batch_signature, shenfen_id, batch_seq)
    /// 调用方负责构造 extrinsic 并提交到交易池。
    pub async fn pack(
        &self,
        batch_seq: u64,
        current_block: u64,
    ) -> Result<PackedBatch, String> {
        let items = self.ledger.take_all_pending();
        if items.is_empty() {
            return Err("无待打包交易".to_string());
        }

        let key_guard = self.signing_key.read().await;
        let signing_key = key_guard
            .as_ref()
            .ok_or("签名管理员私钥未加载".to_string())?;

        // 中文注释：构造 batch 签名消息（与链上 batch_signing_message 一致）。
        let message = Self::batch_signing_message(&self.shenfen_id, batch_seq, &items);
        let signature = <sr25519::Pair as Pair>::sign(&signing_key.pair, &message);

        // 持久化账本（打包后账本已清空，但 confirmed_tx_ids 还在）
        if let Err(e) = self.ledger.save_to_disk(&self.password) {
            log::warn!("[Offchain] 打包后持久化账本失败：{e}");
        }

        // 更新上次打包区块
        *self.last_pack_block.write().await = current_block;

        let tx_ids: Vec<_> = items.iter().map(|item| item.tx_id).collect();

        log::info!(
            "[Offchain] 打包完成：{} 笔交易，batch_seq={batch_seq}",
            items.len()
        );

        Ok(PackedBatch {
            items,
            signature: signature.0.to_vec(),
            shenfen_id: self.shenfen_id.clone(),
            batch_seq,
            tx_ids,
        })
    }

    /// 中文注释：上链成功后清理已结算交易，并向其他省储行广播结算通知。
    pub fn on_settled(&self, tx_ids: &[sp_core::H256]) {
        self.ledger.remove_settled(tx_ids);
        if let Err(e) = self.ledger.save_to_disk(&self.password) {
            log::warn!("[Offchain] 结算后持久化账本失败：{e}");
        }
        // 向其他省储行广播结算完成通知
        if let Some(ref tx) = self.gossip_tx {
            let _ = tx.send(
                crate::offchain_gossip::OffchainGossipMessage::Settled {
                    tx_ids: tx_ids.to_vec(),
                    clearing_bank: self.shenfen_id.as_bytes().to_vec(),
                },
            );
        }
    }

    /// 中文注释：上链失败后将交易放回账本。
    pub fn on_pack_failed(&self, items: Vec<OffchainTxItem>) {
        for item in items {
            if let Err(e) = self.ledger.confirm_tx(item) {
                log::warn!("[Offchain] 放回交易失败：{e}");
            }
        }
        if let Err(e) = self.ledger.save_to_disk(&self.password) {
            log::warn!("[Offchain] 失败恢复后持久化账本失败：{e}");
        }
    }

    /// 中文注释：获取待上链交易数量。
    pub fn pending_count(&self) -> usize {
        self.ledger.pending_count()
    }

    /// 中文注释：构造 batch 签名消息（blake2_256 哈希）。
    /// 与链上 offchain-transaction-pos 的 batch_signing_message 保持一致。
    fn batch_signing_message(
        shenfen_id: &str,
        batch_seq: u64,
        items: &[OffchainTxItem],
    ) -> [u8; 32] {
        let mut data = Vec::new();
        // shenfen_id 补零到 48 字节
        let id_bytes = shenfen_id.as_bytes();
        let mut id_fixed = [0u8; 48];
        let copy_len = id_bytes.len().min(48);
        id_fixed[..copy_len].copy_from_slice(&id_bytes[..copy_len]);
        data.extend_from_slice(&id_fixed);
        // batch_seq
        data.extend_from_slice(&batch_seq.to_le_bytes());
        // 每笔交易的 SCALE 编码
        for item in items {
            data.extend_from_slice(item.tx_id.as_bytes());
            data.extend_from_slice(item.payer.as_ref());
            data.extend_from_slice(item.recipient.as_ref());
            data.extend_from_slice(&item.transfer_amount.to_le_bytes());
            data.extend_from_slice(&item.fee_amount.to_le_bytes());
        }
        blake2_256(&data)
    }
}

/// 打包结果。
pub struct PackedBatch {
    /// 批次中的交易列表。
    pub items: Vec<OffchainTxItem>,
    /// 管理员签名（sr25519，64 字节）。
    pub signature: Vec<u8>,
    /// 省储行 shenfen_id。
    pub shenfen_id: String,
    /// 批次序号。
    pub batch_seq: u64,
    /// 批次中所有 tx_id（用于结算后清理）。
    pub tx_ids: Vec<sp_core::H256>,
}
