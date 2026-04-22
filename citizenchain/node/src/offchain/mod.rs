//! 扫码支付清算体系节点层(ADR-006)。
//!
//! 中文注释:
//! - 本目录承载"清算行(L2)"全节点运行时所需的链下组件,包括本地账本、
//!   对 wuminapp 的 RPC、批次打包器、链上事件监听同步、主账对账。
//! - `service::new_full` 检测到 `--clearing-bank` CLI flag 时,调
//!   `start_clearing_bank_components` 启动本目录下的组件,并 spawn:
//!     - `offchain-clearing-packer`(30 秒 tick)
//!     - `offchain-clearing-event-listener`(订阅 import_notification_stream)
//!     - `offchain-clearing-reserve-monitor`(主账对账)
//!   不加 `--clearing-bank` 的节点仅跑 PoW + GRANDPA,跳过本目录所有启动。
//!
//! 模块边界(对照上层 ADR-006):
//! - `ledger`:清算行本地 L3 存款缓存(权威账本在链上 `DepositBalance`)
//! - `rpc`:对 wuminapp 的查询 / 扫码支付提交
//! - `packer`:批次聚合 + 清算行多签 + 上链
//! - `keystore_signer`:`BatchSigner` 接口 + `offchain_keystore::SigningKey` 实现
//! - `pool_submitter`:`BatchSubmitter` 接口 + 签名 extrinsic + `pool.submit_one`
//! - `event_listener`:监听链上 `Deposited` / `Withdrawn` / `PaymentSettled`
//!   事件,同步本地 ledger 缓存
//! - `reserve_monitor`:本地 `Σ confirmed` 与链上 `BankTotalDeposits` 周期对账

pub mod event_listener;
pub mod keystore_signer;
pub mod ledger;
pub mod packer;
pub mod pool_submitter;
pub mod reserve_monitor;
pub mod rpc;

use sp_runtime::AccountId32;
use std::path::Path;
use std::sync::Arc;

use self::event_listener::EventListener;
// Step 2b-ii-β-1 新增导出:供 β-2 的 service.rs 启动清算行 worker 时注入真实
// 签名器。本步尚未被 service.rs 使用,故 `#[allow(unused_imports)]` 抑制
// 编译器对 re-export 的"未使用"提示;β-2 接入后可去掉。
#[allow(unused_imports)]
pub use self::keystore_signer::KeystoreBatchSigner;
use self::ledger::OffchainLedger;
use self::packer::{
    BatchSigner, BatchSubmitter, NoopBatchSigner, NoopBatchSubmitter, OffchainPacker,
};
use self::reserve_monitor::ReserveMonitor;
use self::rpc::OffchainClearingRpcImpl;

/// Step 2b 新增:清算行节点一次性启动时组装的组件集合。
///
/// 中文注释:
/// - `service.rs` 在检测到节点角色是"清算行"时调 `start_clearing_bank_components`
///   拿回本结构,并把 `rpc_impl` 注册到 JSON-RPC 命名空间,`packer` 交给后台
///   worker,`event_listener` 订阅链上事件。
/// - Step 2b-i 本步仅完成 **组装 + 持久化恢复**;真正的 extrinsic 提交 / libp2p
///   gossip / 链上事件订阅由 Step 2b-ii / 2b-iii 接入。
pub struct OffchainComponents {
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
///                传真实 `KeystoreBatchSigner`(从 offchain_keystore 派生)。
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
    client: Arc<crate::service::FullClient>,
) -> Result<OffchainComponents, String> {
    let ledger = Arc::new(OffchainLedger::new(base_path));
    // 若磁盘有上次加密持久化的 ledger,尝试恢复;首次启动(文件不存在)返回 Ok(0)。
    ledger.load_from_disk(password)?;

    let packer = Arc::new(OffchainPacker::new(
        ledger.clone(),
        bank_main.clone(),
        signer,
        submitter,
    ));
    let event_listener = Arc::new(EventListener::new(ledger.clone(), bank_main.clone()));
    let reserve_monitor = Arc::new(ReserveMonitor::new(ledger.clone(), bank_main));
    // Step 2c-i 新增:RPC 层需要 client 以读 `UserBank` / `L2FeeRateBp` storage。
    let rpc_impl = Arc::new(OffchainClearingRpcImpl::new(ledger.clone(), client));

    Ok(OffchainComponents {
        ledger,
        packer,
        event_listener,
        reserve_monitor,
        rpc_impl,
    })
}

/// Step 2b-ii-α 默认启动器:signer / submitter 走 Noop 占位。
///
/// Step 2b-ii-β 之前,service.rs 启动清算行节点时调用此便捷版;上链/签名
/// 会 fail-fast 但其他读路径(query_balance / submit_payment 的 accept_payment)
/// 完全可用。
pub fn start_clearing_bank_components_with_noop(
    base_path: &Path,
    bank_main: AccountId32,
    password: &str,
    client: Arc<crate::service::FullClient>,
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
