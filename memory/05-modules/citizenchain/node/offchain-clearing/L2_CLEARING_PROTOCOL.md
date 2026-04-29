# L2 链下清算协议

> **Legacy / 已废弃**
>
> 本文描述的是旧"43 个省储行组成 L2 清算网络"方案，仅作为历史留档。
> 当前有效模型为 **注册清算行节点 + 收款方清算行主导 settlement**，
> 入口见 `memory/04-decisions/ADR-007-clearing-bank-three-phase.md` 与
> `memory/05-modules/citizenchain/node/offchain/STEP2B_II_B_2_B_INTEGRATION.md`。
> 新实现不再使用本文中的 `/gmb/offchain-clearing/1`、`offchain_submitSignedTx`
> 或全局 remote_pending 账本。

## 概述

43 个省储行节点组成第 2 层清算网络（L2），通过自定义 P2P 通知协议广播链下交易的待结算状态。每笔链下支付由收款方绑定的省储行确认后，向其他 42 个省储行广播，所有节点维护同一份全局待结算账本，用于防止跨省储行双花。

## 协议标识

- 协议名称：`/gmb/offchain-clearing/1`
- 最大消息大小：1024 字节
- 连接配置：in_peers=43，out_peers=43，NonReservedPeerMode::Accept

## 消息类型

### PendingDebit（待结算通知）

收款方省储行确认交易后广播，通知其他省储行冻结该 payer 的虚拟余额。

| 字段 | 类型 | 说明 |
|------|------|------|
| tx_id | H256 | 交易唯一标识 |
| payer | AccountId32 | 付款方地址 |
| amount_with_fee | u128 | transfer_amount + fee_amount |
| clearing_bank | Vec<u8> | 负责清算的省储行 shenfen_id（UTF-8） |
| timestamp | u64 | 确认时间（Unix 秒） |

### Settled（结算完成通知）

批次上链成功后广播，通知其他省储行释放对应的冻结额度。

| 字段 | 类型 | 说明 |
|------|------|------|
| tx_ids | Vec<H256> | 已结算的交易 ID 列表 |
| clearing_bank | Vec<u8> | 省储行 shenfen_id |

## Gossip 广播流程

1. 省储行节点通过 RPC `offchain_submitSignedTx` 接收用户签名的链下支付
2. 验证通过后调用 `OffchainLedger::confirm_tx` 记入本地账本
3. 通过 gossip channel 向所有已连接 peer 发送 `PendingDebit` 消息
4. 其他省储行收到后调用 `add_remote_pending` 更新远程待结算账本
5. 打包上链成功后发送 `Settled` 消息，其他省储行调用 `remove_remote_settled` 释放额度

## 待结算账本（OffchainLedger）

### 数据结构

- `pending_by_payer`：本地 payer 待结算总额（本省储行确认的交易）
- `pending_txs`：待打包交易列表（按确认时间排序）
- `confirmed_tx_ids`：已确认 tx_id 索引（防重复提交）
- `remote_pending_by_payer`：远程 payer 待结算总额（其他省储行广播）
- `remote_pending_txs`：远程待结算明细

### 虚拟余额计算

```
virtual_balance = onchain_balance - local_pending - remote_pending
```

用于链下支付前的余额校验，防止同一 payer 在多个省储行同时消费超过链上余额。

### 过期清理

远程待结算记录超过 **7200 秒（2 小时）** 未收到结算通知则自动清除。这是打包时间阈值 60 分钟的 2 倍安全边际。

### 持久化

账本通过 BLAKE2-256 XOR 加密持久化到 `{base_path}/offchain/ledger.enc`，节点重启后通过密码解密恢复。

## 批量打包（OffchainPacker）

### 触发阈值

| 条件 | 阈值 | 说明 |
|------|------|------|
| 笔数阈值 | 100,000 笔 | 待打包交易达到此数量立即触发 |
| 时间阈值 | 10 个区块（约 60 分钟） | 距上次打包超过此间隔且有待打包交易 |

### 打包签名

batch 签名消息构造：shenfen_id（补零到 48 字节）+ batch_seq（LE u64）+ 每笔交易的 tx_id/payer/recipient/transfer_amount/fee_amount 拼接后取 BLAKE2-256 哈希，使用省储行管理员 sr25519 密钥签名。与链上 `offchain-transaction` 的验证逻辑保持一致。

### 结算生命周期

```
confirm_tx → PendingDebit 广播 → should_pack 检查阈值
  → pack 取出交易并签名 → 提交 extrinsic 上链
  → 上链成功 → on_settled 清理账本 + Settled 广播
  → 上链失败 → on_pack_failed 放回账本
```

## 相关 RPC

| 方法 | 说明 |
|------|------|
| `offchain_submitSignedTx` | 接收用户签名的链下支付交易 |
| `offchain_queryTxStatus` | 查询交易状态（confirmed/onchain/unknown） |
| `offchain_queryInstitutionRate` | 查询本省储行的链下交易费率 |

## 源码位置

- `citizenchain/node/src/offchain_gossip.rs` - P2P 广播协议
- `citizenchain/node/src/offchain_ledger.rs` - 待结算账本
- `citizenchain/node/src/offchain_packer.rs` - 批量打包器
- `citizenchain/node/src/core/rpc.rs` - 链下 RPC 接口
