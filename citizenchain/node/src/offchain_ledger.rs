//! 链下待结算账本模块。
//!
//! 维护已确认但未上链的链下交易，支持：
//! - 内存操作（高性能读写）
//! - 加密持久化（节点重启后恢复）
//! - 虚拟余额计算（防双花）

use codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::AccountId32;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// 中文注释：node 端 blake2b-256 哈希。
fn blake2_256(data: &[u8]) -> [u8; 32] {
    let hash = blake2b_simd::Params::new().hash_length(32).hash(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// 单笔链下交易记录。
#[derive(Clone, Encode, Decode, Debug)]
pub struct OffchainTxItem {
    /// 交易唯一标识（防重放）。
    pub tx_id: H256,
    /// 付款方地址。
    pub payer: AccountId32,
    /// 收款方地址。
    pub recipient: AccountId32,
    /// 转账金额（分）。
    pub transfer_amount: u128,
    /// 手续费金额（分）。
    pub fee_amount: u128,
    /// 确认时间戳（Unix 秒）。
    pub confirmed_at: u64,
}

/// 远程待结算记录（其他省储行广播过来的）。
#[derive(Clone, Encode, Decode, Debug)]
pub struct RemotePendingItem {
    pub payer: AccountId32,
    pub amount_with_fee: u128,
    pub clearing_bank: Vec<u8>,
    pub received_at: u64,
}

/// 远程待结算过期时间：2 小时（打包阈值 60 分钟 × 2 倍安全边际）。
const REMOTE_PENDING_EXPIRE_SECS: u64 = 7200;

/// 链下待结算账本。
#[derive(Default)]
struct LedgerInner {
    /// 中文注释：每个 payer 的本地待结算总额（本省储行确认的交易）。
    pending_by_payer: HashMap<AccountId32, u128>,
    /// 中文注释：待打包交易列表（按确认时间排序）。
    pending_txs: Vec<OffchainTxItem>,
    /// 中文注释：已确认 tx_id 索引（防重复提交）。
    confirmed_tx_ids: HashSet<H256>,
    /// 中文注释：远程待结算——其他省储行广播的待结算记录（防跨省储行双花）。
    remote_pending_by_payer: HashMap<AccountId32, u128>,
    /// 中文注释：远程待结算明细（用于结算后清除和过期清理）。
    remote_pending_txs: HashMap<H256, RemotePendingItem>,
}

/// 线程安全的链下账本。
pub struct OffchainLedger {
    inner: Arc<RwLock<LedgerInner>>,
    /// 加密持久化文件路径。
    file_path: PathBuf,
}

impl OffchainLedger {
    /// 中文注释：创建账本实例。
    pub fn new(base_path: &Path) -> Self {
        let dir = base_path.join("offchain");
        Self {
            inner: Arc::new(RwLock::new(LedgerInner::default())),
            file_path: dir.join("ledger.enc"),
        }
    }

    /// 中文注释：确认一笔链下交易，记入账本。
    pub fn confirm_tx(&self, item: OffchainTxItem) -> Result<(), String> {
        let mut ledger = self.inner.write().map_err(|e| format!("账本锁错误：{e}"))?;

        // 防重复
        if ledger.confirmed_tx_ids.contains(&item.tx_id) {
            return Err("交易已确认，重复提交".to_string());
        }

        // 累加 payer 待结算额
        let total = item.transfer_amount.saturating_add(item.fee_amount);
        *ledger.pending_by_payer.entry(item.payer.clone()).or_insert(0) += total;

        ledger.confirmed_tx_ids.insert(item.tx_id);
        ledger.pending_txs.push(item);

        Ok(())
    }

    /// 中文注释：计算虚拟余额 = 链上余额 - 本地待结算 - 远程待结算。
    pub fn virtual_balance(&self, payer: &AccountId32, onchain_balance: u128) -> u128 {
        let ledger = self.inner.read().unwrap_or_else(|e| e.into_inner());
        let local_pending = ledger.pending_by_payer.get(payer).copied().unwrap_or(0);
        let remote_pending = ledger.remote_pending_by_payer.get(payer).copied().unwrap_or(0);
        onchain_balance
            .saturating_sub(local_pending)
            .saturating_sub(remote_pending)
    }

    /// 中文注释：检查 tx_id 是否已确认。
    pub fn is_duplicate(&self, tx_id: &H256) -> bool {
        let ledger = self.inner.read().unwrap_or_else(|e| e.into_inner());
        ledger.confirmed_tx_ids.contains(tx_id)
    }

    /// 中文注释：获取待打包交易数量。
    pub fn pending_count(&self) -> usize {
        let ledger = self.inner.read().unwrap_or_else(|e| e.into_inner());
        ledger.pending_txs.len()
    }

    /// 中文注释：取出所有待打包交易（打包用）。取出后账本清空。
    pub fn take_all_pending(&self) -> Vec<OffchainTxItem> {
        let mut ledger = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let txs = std::mem::take(&mut ledger.pending_txs);
        ledger.pending_by_payer.clear();
        // confirmed_tx_ids 保留（防止打包期间重复提交）
        txs
    }

    /// 中文注释：上链成功后清除对应记录。
    pub fn remove_settled(&self, tx_ids: &[H256]) {
        let mut ledger = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let id_set: HashSet<_> = tx_ids.iter().collect();

        // 先收集需要扣减的金额
        let mut deductions: HashMap<AccountId32, u128> = HashMap::new();
        for item in ledger.pending_txs.iter() {
            if id_set.contains(&item.tx_id) {
                let total = item.transfer_amount.saturating_add(item.fee_amount);
                *deductions.entry(item.payer.clone()).or_insert(0) += total;
            }
        }

        // 扣减 payer 待结算额
        for (payer, amount) in &deductions {
            if let Some(pending) = ledger.pending_by_payer.get_mut(payer) {
                *pending = pending.saturating_sub(*amount);
                if *pending == 0 {
                    ledger.pending_by_payer.remove(payer);
                }
            }
        }

        // 从 pending_txs 中移除已结算的
        ledger.pending_txs.retain(|item| !id_set.contains(&item.tx_id));

        // confirmed_tx_ids 中也移除（已上链，链上有记录）
        for id in tx_ids {
            ledger.confirmed_tx_ids.remove(id);
        }
    }

    /// 中文注释：添加远程待结算记录（收到其他省储行的广播通知）。
    pub fn add_remote_pending(
        &self,
        tx_id: H256,
        payer: AccountId32,
        amount_with_fee: u128,
        clearing_bank: Vec<u8>,
        timestamp: u64,
    ) {
        let mut ledger = self.inner.write().unwrap_or_else(|e| e.into_inner());
        // 防重复
        if ledger.remote_pending_txs.contains_key(&tx_id) {
            return;
        }
        *ledger.remote_pending_by_payer.entry(payer.clone()).or_insert(0) += amount_with_fee;
        ledger.remote_pending_txs.insert(
            tx_id,
            RemotePendingItem {
                payer,
                amount_with_fee,
                clearing_bank,
                received_at: timestamp,
            },
        );
    }

    /// 中文注释：清除已结算的远程待结算记录（收到结算完成广播）。
    pub fn remove_remote_settled(&self, tx_ids: &[H256]) {
        let mut ledger = self.inner.write().unwrap_or_else(|e| e.into_inner());
        for tx_id in tx_ids {
            if let Some(item) = ledger.remote_pending_txs.remove(tx_id) {
                if let Some(pending) = ledger.remote_pending_by_payer.get_mut(&item.payer) {
                    *pending = pending.saturating_sub(item.amount_with_fee);
                    if *pending == 0 {
                        ledger.remote_pending_by_payer.remove(&item.payer);
                    }
                }
            }
        }
    }

    /// 中文注释：清理过期的远程待结算记录（超过 2 小时未收到结算通知）。
    pub fn cleanup_expired_remote(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut ledger = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let expired: Vec<H256> = ledger
            .remote_pending_txs
            .iter()
            .filter(|(_, item)| now.saturating_sub(item.received_at) > REMOTE_PENDING_EXPIRE_SECS)
            .map(|(tx_id, _)| *tx_id)
            .collect();
        for tx_id in &expired {
            if let Some(item) = ledger.remote_pending_txs.remove(tx_id) {
                if let Some(pending) = ledger.remote_pending_by_payer.get_mut(&item.payer) {
                    *pending = pending.saturating_sub(item.amount_with_fee);
                    if *pending == 0 {
                        ledger.remote_pending_by_payer.remove(&item.payer);
                    }
                }
            }
        }
        if !expired.is_empty() {
            log::debug!(
                "[Offchain] 清理 {} 笔过期远程待结算",
                expired.len()
            );
        }
    }

    /// 中文注释：加密持久化到磁盘。
    pub fn save_to_disk(&self, password: &str) -> Result<(), String> {
        let ledger = self.inner.read().map_err(|e| format!("账本锁错误：{e}"))?;

        // SCALE 编码待打包交易列表
        let encoded = ledger.pending_txs.encode();
        let tx_ids_encoded: Vec<H256> = ledger.confirmed_tx_ids.iter().cloned().collect();
        let ids_encoded = tx_ids_encoded.encode();

        // 组装明文：[txs_len:4][txs_data][ids_len:4][ids_data]
        let txs_len = (encoded.len() as u32).to_le_bytes();
        let ids_len = (ids_encoded.len() as u32).to_le_bytes();
        let mut plaintext = Vec::new();
        plaintext.extend_from_slice(&txs_len);
        plaintext.extend_from_slice(&encoded);
        plaintext.extend_from_slice(&ids_len);
        plaintext.extend_from_slice(&ids_encoded);

        // 简单加密（XOR + HMAC，与 keystore 一致）
        let key = blake2_256(password.as_bytes());
        let encrypted: Vec<u8> = plaintext
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ key[i % 32])
            .collect();
        let tag = blake2_256(&[&key[..], &encrypted].concat());

        let mut data = Vec::new();
        data.extend_from_slice(&encrypted);
        data.extend_from_slice(&tag);

        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败：{e}"))?;
        }
        fs::write(&self.file_path, &data).map_err(|e| format!("写入账本失败：{e}"))?;
        log::debug!("[Offchain] 账本已持久化（{} 笔待上链）", ledger.pending_txs.len());
        Ok(())
    }

    /// 中文注释：从磁盘解密恢复账本。
    pub fn load_from_disk(&self, password: &str) -> Result<usize, String> {
        if !self.file_path.exists() {
            return Ok(0);
        }

        let data = fs::read(&self.file_path).map_err(|e| format!("读取账本失败：{e}"))?;
        if data.len() < 32 {
            return Err("账本文件格式错误".to_string());
        }

        let (encrypted, tag) = data.split_at(data.len() - 32);
        let key = blake2_256(password.as_bytes());
        let expected_tag = blake2_256(&[&key[..], encrypted].concat());
        if tag != expected_tag {
            return Err("密码错误或账本文件损坏".to_string());
        }

        // 解密
        let plaintext: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ key[i % 32])
            .collect();

        // 解析
        if plaintext.len() < 4 {
            return Err("账本数据不完整".to_string());
        }
        let txs_len = u32::from_le_bytes(plaintext[..4].try_into().unwrap()) as usize;
        if plaintext.len() < 4 + txs_len + 4 {
            return Err("账本数据不完整".to_string());
        }
        let txs_data = &plaintext[4..4 + txs_len];
        let ids_len = u32::from_le_bytes(
            plaintext[4 + txs_len..4 + txs_len + 4].try_into().unwrap(),
        ) as usize;
        let ids_data = &plaintext[4 + txs_len + 4..4 + txs_len + 4 + ids_len];

        let pending_txs: Vec<OffchainTxItem> =
            Decode::decode(&mut &txs_data[..]).map_err(|e| format!("解码交易失败：{e}"))?;
        let tx_ids_vec: Vec<H256> =
            Decode::decode(&mut &ids_data[..]).map_err(|e| format!("解码 tx_id 失败：{e}"))?;

        // 恢复到内存
        let mut ledger = self.inner.write().map_err(|e| format!("账本锁错误：{e}"))?;
        let count = pending_txs.len();
        for item in &pending_txs {
            let total = item.transfer_amount.saturating_add(item.fee_amount);
            *ledger.pending_by_payer.entry(item.payer.clone()).or_insert(0) += total;
        }
        ledger.pending_txs = pending_txs;
        ledger.confirmed_tx_ids = tx_ids_vec.into_iter().collect();

        log::info!("[Offchain] 账本已从磁盘恢复（{count} 笔待上链）");
        Ok(count)
    }
}

impl Clone for OffchainLedger {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            file_path: self.file_path.clone(),
        }
    }
}
