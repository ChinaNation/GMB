//! 省储行链下清算 P2P 广播协议。
//! 43 个省储行节点组成第 2 层清算网络，通过自定义通知协议广播链下交易的待结算状态。
//! 每笔链下支付由收款方绑定的省储行确认后，向其他 42 个省储行广播待结算通知。
//! 所有省储行维护同一份全局待结算账本，用于防止跨省储行双花。

use codec::{Decode, Encode};
use sc_network::{
    config::{NonDefaultSetConfig, NonReservedPeerMode, SetConfig},
    service::traits::NotificationEvent,
    NotificationService, PeerId, ProtocolName,
};
use sp_core::H256;
use sp_runtime::AccountId32;
use std::collections::HashSet;

use crate::offchain_ledger::OffchainLedger;

/// 协议名称。
const PROTOCOL_PREFIX: &str = "/gmb/offchain-clearing/1";

/// 最大消息大小（1KB）。
const MAX_MESSAGE_SIZE: u64 = 1024;

/// 广播消息类型。
#[derive(Clone, Debug, Encode, Decode)]
pub enum OffchainGossipMessage {
    /// 新增待结算通知：某用户在某省储行有一笔待结算交易。
    PendingDebit {
        tx_id: H256,
        payer: AccountId32,
        /// transfer_amount + fee_amount
        amount_with_fee: u128,
        /// 负责清算的省储行 shenfen_id（UTF-8 字节）
        clearing_bank: Vec<u8>,
        /// 确认时间（Unix 秒）
        timestamp: u64,
    },
    /// 结算完成通知：批次已上链，清除这些待结算。
    Settled {
        tx_ids: Vec<H256>,
        clearing_bank: Vec<u8>,
    },
}

/// 创建链下清算通知协议配置。
pub fn offchain_clearing_protocol_config() -> (NonDefaultSetConfig, Box<dyn NotificationService>) {
    let protocol_name = ProtocolName::from(PROTOCOL_PREFIX);
    NonDefaultSetConfig::new(
        protocol_name,
        Vec::new(),
        MAX_MESSAGE_SIZE,
        None,
        SetConfig {
            in_peers: 43,
            out_peers: 43,
            reserved_nodes: vec![],
            non_reserved_mode: NonReservedPeerMode::Accept,
        },
    )
}

/// 运行链下清算广播 worker（收发合一）。
///
/// - 接收：监听其他省储行的广播消息，更新远程待结算账本。
/// - 发送：从 channel 接收广播请求，通过 P2P 通知发送给已连接的 peer。
pub async fn run_offchain_gossip_worker(
    mut notification_service: Box<dyn NotificationService>,
    ledger: OffchainLedger,
    mut send_rx: Option<tokio::sync::mpsc::UnboundedReceiver<OffchainGossipMessage>>,
) {
    log::info!("[OffchainGossip] 链下清算广播 worker 已启动");

    // 记录已连接的 peer（通过 stream open/close 事件维护）
    let mut connected_peers: HashSet<PeerId> = HashSet::new();

    loop {
        tokio::select! {
            event = notification_service.next_event() => {
                match event {
                    Some(NotificationEvent::NotificationReceived { peer, notification }) => {
                        match OffchainGossipMessage::decode(&mut &notification[..]) {
                            Ok(msg) => handle_gossip_message(&ledger, msg),
                            Err(e) => {
                                log::warn!(
                                    "[OffchainGossip] 解码消息失败 (peer={peer:?}): {e}"
                                );
                            }
                        }
                    }
                    Some(NotificationEvent::ValidateInboundSubstream { result_tx, .. }) => {
                        let _ = result_tx.send(sc_network::service::traits::ValidationResult::Accept);
                    }
                    Some(NotificationEvent::NotificationStreamOpened { peer, .. }) => {
                        log::debug!("[OffchainGossip] 连接已建立: {peer:?}");
                        connected_peers.insert(peer);
                    }
                    Some(NotificationEvent::NotificationStreamClosed { peer }) => {
                        log::debug!("[OffchainGossip] 连接已关闭: {peer:?}");
                        connected_peers.remove(&peer);
                    }
                    None => {
                        log::warn!("[OffchainGossip] 通知流结束");
                        break;
                    }
                }
            }
            msg = async {
                if let Some(ref mut rx) = send_rx {
                    rx.recv().await
                } else {
                    std::future::pending().await
                }
            } => {
                if let Some(msg) = msg {
                    let encoded = msg.encode();
                    for peer in &connected_peers {
                        notification_service.send_sync_notification(peer, encoded.clone());
                    }
                }
            }
        }

        // 定期清理过期远程待结算
        ledger.cleanup_expired_remote();
    }
}

/// 处理收到的广播消息。
fn handle_gossip_message(ledger: &OffchainLedger, msg: OffchainGossipMessage) {
    match msg {
        OffchainGossipMessage::PendingDebit {
            tx_id,
            payer,
            amount_with_fee,
            clearing_bank,
            timestamp,
        } => {
            let bank_name = String::from_utf8_lossy(&clearing_bank);
            log::debug!(
                "[OffchainGossip] 收到待结算通知: payer={}, amount={}, bank={}, tx={}",
                payer, amount_with_fee, bank_name, tx_id
            );
            ledger.add_remote_pending(tx_id, payer, amount_with_fee, clearing_bank, timestamp);
        }
        OffchainGossipMessage::Settled {
            tx_ids,
            clearing_bank,
        } => {
            let bank_name = String::from_utf8_lossy(&clearing_bank);
            log::debug!(
                "[OffchainGossip] 收到结算通知: {} 笔, bank={}",
                tx_ids.len(), bank_name
            );
            ledger.remove_remote_settled(&tx_ids);
        }
    }
}
