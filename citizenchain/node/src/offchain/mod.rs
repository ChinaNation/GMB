//! 扫码支付清算体系节点层(ADR-006)。
//!
//! 中文注释:
//! - 本目录统一承载 node 层清算行功能,包括清算行管理命令、本地账本、
//!   对 wuminapp 的 RPC、批次打包器、链上事件监听同步、主账对账。
//! - `commands`:Tauri 前端清算行页面的 SFID 查询、链上查询、扫码签名、解密入口。
//! - `service::new_full` 检测到 `--clearing-bank` CLI flag 时,调
//!   `start_clearing_bank_components` 启动本目录下的组件,并 spawn:
//!     - `offchain-clearing-packer`(30 秒 tick)
//!     - `offchain-clearing-event-listener`(订阅 import_notification_stream)
//!     - `offchain-clearing-reserve-monitor`(主账对账)
//!   不加 `--clearing-bank` 的节点仅跑 PoW + GRANDPA,跳过本目录所有启动。
//!
//! 模块边界(对照上层 ADR-006/ADR-007):
//! - `sfid` / `chain` / `health` / `signing` / `decrypt`:清算行管理流程。
//! - `ledger`:清算行本地 L3 存款缓存(权威账本在链上 `DepositBalance`)
//! - `rpc`:对 wuminapp 的查询 / 扫码支付提交
//! - `settlement`:批次聚合、清算行多签、上链提交、链上事件监听。
//! - `reserve`:本地 `Σ confirmed` 与链上 `BankTotalDeposits` 周期对账

pub(crate) mod bootstrap;
pub mod chain;
pub(crate) mod commands;
pub mod decrypt;
pub mod health;
pub mod keystore;
pub mod ledger;
pub mod reserve;
pub mod rpc;
pub mod settlement;
pub mod sfid;
pub mod signing;
pub mod types;

use codec::{Decode, Encode};
use sc_client_api::StorageProvider;
use sp_blockchain::HeaderBackend;
use sp_runtime::AccountId32;
use sp_storage::StorageKey;
use std::path::Path;
use std::sync::Arc;

use self::ledger::OffchainLedger;
use self::reserve::ReserveMonitor;
use self::rpc::OffchainClearingRpcImpl;
use self::settlement::listener::EventListener;
use self::settlement::packer::{
    BatchSigner, BatchSubmitter, NoopBatchSigner, NoopBatchSubmitter, OffchainPacker,
};
pub use self::settlement::signer::KeystoreBatchSigner;

/// Step 2b 新增:清算行节点一次性启动时组装的组件集合。
///
/// 中文注释:
/// - `service.rs` 在检测到节点角色是"清算行"时调 `start_clearing_bank_components`
///   拿回本结构,并把 `rpc_impl` 注册到 JSON-RPC 命名空间,`packer` 交给后台
///   worker,`event_listener` 订阅链上事件。
/// - Step 2b-i 本步仅完成 **组装 + 持久化恢复**;真正的 extrinsic 提交 / libp2p
///   gossip / 链上事件订阅由 Step 2b-ii / 2b-iii 接入。
pub struct OffchainComponents {
    /// 本地 L3 账本句柄。当前 worker 通过 `packer` / `event_listener` / `rpc_impl`
    /// 间接持有它,字段保留给后续清算行运维状态查询使用。
    #[allow(dead_code)]
    pub ledger: Arc<OffchainLedger>,
    pub packer: Arc<OffchainPacker>,
    pub event_listener: Arc<EventListener>,
    /// Step 2b-iii-b 新增:本地 `Σ confirmed` 与链上 `BankTotalDeposits`
    /// 主账对账 worker(`run` 需要调用方 spawn 到 `task_manager`)。
    pub reserve_monitor: Arc<ReserveMonitor>,
    pub rpc_impl: Arc<OffchainClearingRpcImpl>,
}

/// 启动清算行节点所需的 offchain 组件套件。
///
/// [`base_path`]  节点数据根目录(下挂 `offchain_step1/ledger.enc`)。
/// [`bank_main`]  本清算行**主账户地址**,用于 `EventListener` 过滤与本行相关
///                的链上事件,以及 packer 批次 signing message 拼接。
/// [`password`]   节点启动时用于 AES-256-GCM 风格加密 ledger 的对称密钥字符串
///                (目前实现为 blake2b_256(password) XOR 流 + HMAC,见 `ledger.rs`)。
/// [`signer`]     批次签名器。Step 2b-ii-α 传 `NoopBatchSigner`;Step 2b-ii-β
///                传真实 `KeystoreBatchSigner`(从 `offchain::keystore` 派生)。
/// [`submitter`]  extrinsic 提交器。Step 2b-ii-α 传 `NoopBatchSubmitter`;Step
///                2b-ii-β 传真实 `PoolBatchSubmitter`(拼 RuntimeCall + 调
///                `TransactionPool`)。
///
/// 返回值:`Arc` 包裹的组件集合,调用方持有 ownership 决定生命周期。
pub fn start_clearing_bank_components(
    base_path: &Path,
    bank_main: AccountId32,
    password: &str,
    signer: Arc<dyn BatchSigner>,
    submitter: Arc<dyn BatchSubmitter>,
    client: Arc<crate::core::service::FullClient>,
) -> Result<OffchainComponents, String> {
    let ledger = Arc::new(OffchainLedger::new(base_path));
    // 若磁盘有上次加密持久化的 ledger,尝试恢复;首次启动(文件不存在)返回 Ok(0)。
    ledger.load_from_disk(password)?;
    let initial_batch_seq = read_last_clearing_batch_seq(client.as_ref(), &bank_main)
        .map_err(|e| format!("读取 LastClearingBatchSeq 失败:{e}"))?;

    let packer = Arc::new(OffchainPacker::new_with_initial_seq(
        ledger.clone(),
        bank_main.clone(),
        signer.clone(),
        submitter,
        initial_batch_seq,
    ));
    let event_listener = Arc::new(EventListener::new(ledger.clone(), bank_main.clone()));
    let reserve_monitor = Arc::new(ReserveMonitor::new(ledger.clone(), bank_main.clone()));
    // RPC 层需要 client 读取 UserBank / L2FeeRateBp,也复用 signer 生成真实 L2 ACK。
    let rpc_impl = Arc::new(OffchainClearingRpcImpl::new(
        ledger.clone(),
        client,
        bank_main,
        signer,
    ));

    Ok(OffchainComponents {
        ledger,
        packer,
        event_listener,
        reserve_monitor,
        rpc_impl,
    })
}

const PALLET_NAME: &[u8] = b"OffchainTransaction";

/// 构造 `LastClearingBatchSeq[bank]` 的 storage key。
fn last_clearing_batch_seq_key(bank: &AccountId32) -> StorageKey {
    let encoded = bank.encode();
    let mut k = Vec::with_capacity(16 + 16 + 16 + encoded.len());
    k.extend_from_slice(&sp_io::hashing::twox_128(PALLET_NAME));
    k.extend_from_slice(&sp_io::hashing::twox_128(b"LastClearingBatchSeq"));
    k.extend_from_slice(&sp_io::hashing::blake2_128(&encoded));
    k.extend_from_slice(&encoded);
    StorageKey(k)
}

/// 读取链上已成功落账的最新 batch_seq。storage 不存在时按 `ValueQuery` 映射为 0。
fn read_last_clearing_batch_seq(
    client: &crate::core::service::FullClient,
    bank: &AccountId32,
) -> Result<u64, String> {
    let best = client.info().best_hash;
    let raw = client
        .storage(best, &last_clearing_batch_seq_key(bank))
        .map_err(|e| format!("storage 读取失败:{e}"))?;
    match raw {
        Some(data) => u64::decode(&mut &data.0[..]).map_err(|e| format!("u64 解码失败:{e}")),
        None => Ok(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acc(b: u8) -> AccountId32 {
        AccountId32::new([b; 32])
    }

    #[test]
    fn last_clearing_batch_seq_key_layout_stable() {
        let bank = acc(0xAA);
        let encoded = bank.encode();
        let key = last_clearing_batch_seq_key(&bank);
        assert_eq!(key.0.len(), 16 + 16 + 16 + encoded.len());
        assert_eq!(&key.0[..16], &sp_io::hashing::twox_128(PALLET_NAME));
        assert_eq!(
            &key.0[16..32],
            &sp_io::hashing::twox_128(b"LastClearingBatchSeq")
        );
        assert_eq!(&key.0[32..48], &sp_io::hashing::blake2_128(&encoded));
        assert_eq!(&key.0[48..], &encoded[..]);
    }
}

/// Step 2b-ii-α 默认启动器:signer / submitter 走 Noop 占位。
///
/// 测试或降级启动时调用此便捷版;上链提交与 `submit_payment` 的 L2 ACK 签名
/// 都会 fail-fast,只读查询路径仍可用。
#[allow(dead_code)]
pub fn start_clearing_bank_components_with_noop(
    base_path: &Path,
    bank_main: AccountId32,
    password: &str,
    client: Arc<crate::core::service::FullClient>,
) -> Result<OffchainComponents, String> {
    start_clearing_bank_components(
        base_path,
        bank_main,
        password,
        Arc::new(NoopBatchSigner),
        Arc::new(NoopBatchSubmitter),
        client,
    )
}
