# 省储行第 2 层清算网络技术文档

> **Legacy / 已废弃**
>
> 本文记录的是旧省储行 L2 gossip 清算网络。当前生产方向已切换为
> **注册清算行节点**，由收款方清算行接收 wuminapp 支付意图、生成
> L2 ACK、打包 `submit_offchain_batch_v2` 并由链上 runtime 原子 settlement。
> 当前入口见 `memory/04-decisions/ADR-007-clearing-bank-three-phase.md`。

## 1. 概述

43 个省储行节点组成第 2 层清算网络，通过自定义 P2P 通知协议广播链下交易的待结算状态。每笔链下支付由收款方绑定的省储行确认后，向其他 42 个省储行广播待结算通知。所有省储行维护同一份全局待结算账本，用于防止跨省储行双花。

## 2. 协议

- 协议名：`/gmb/offchain-clearing/1`
- 传输层：复用 Substrate libp2p 的 `NotificationService`
- 消息编码：SCALE
- 最大消息：1KB

## 3. 消息类型

### PendingDebit（待结算通知）
- 触发：省储行确认一笔链下支付后
- 内容：tx_id、payer、amount_with_fee、clearing_bank、timestamp
- 效果：其他省储行记录该 payer 的远程待结算，扣减虚拟余额

### Settled（结算完成通知）
- 触发：省储行打包批次上链成功后
- 内容：tx_ids 列表、clearing_bank
- 效果：其他省储行清除对应的远程待结算记录

## 4. 虚拟余额计算

```
可用余额 = 链上余额 - 本地待结算 - 远程待结算
```

- 本地待结算：本省储行确认但未上链的交易
- 远程待结算：其他省储行广播过来的待结算记录

## 5. 过期清理

远程待结算超过 2 小时（打包阈值 60 分钟 × 2 倍安全边际）未收到 Settled 通知，自动清除。

## 6. 防双花时序

```
T=0.0s  付款方 A → 广东省储行支付 1000 元
T=0.1s  广东省储行确认，广播 PendingDebit
T=0.2s  上海省储行收到广播，A 远程待结算 +1000
T=0.5s  付款方 A → 上海省储行支付 1000 元
T=0.5s  上海省储行查虚拟余额：链上 1500 - 远程 1000 = 500 → 不足，拒绝
```

广播延迟窗口约 100-500ms，在此窗口内的双花仍可能成功，但概率极低。

## 7. 涉及文件

| 文件 | 职责 |
|------|------|
| `node/src/offchain_gossip.rs` | P2P 广播协议定义、消息收发 worker |
| `node/src/offchain_ledger.rs` | 账本扩展（remote_pending_by_payer、remote_pending_txs） |
| `node/src/core/rpc.rs` | 支付确认后发送 PendingDebit 广播 |
| `node/src/offchain_packer.rs` | 结算成功后发送 Settled 广播 |
| `node/src/core/service.rs` | 注册 P2P 协议、启动 gossip worker |

## 8. 节点角色

- **省储行节点**：发送 PendingDebit（确认支付时）+ Settled（上链成功时），接收其他省储行的广播
- **普通全节点**：接收广播但不发送（无链下清算能力），维护远程待结算供未来扩展
