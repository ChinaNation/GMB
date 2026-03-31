# 第2步：node 节点软件改造 — 省储行链下交易接收与批量打包

## 状态：open

## 前置依赖

- 第1步 wuminapp 改造完成（可先用 mock RPC 并行开发）

## 任务目标

省储行节点（运行区块链节点软件的云服务器）新增链下交易接收能力：
- 接收顾客签名的支付交易
- 维护链下待结算账本（虚拟余额扣减，防双花）
- 累积交易达到阈值后批量打包上链

## 改动范围

### 1. 新增自定义 RPC 方法（citizenchain/node/src/rpc.rs）

参考现有 `reward_bindWallet`、`fee_blockFees` 等自定义 RPC 模式。

#### offchain_submitSignedTx

- 入参：签名交易数据（payer、recipient、amount、fee、签名、商户绑定的省储行）
- 流程：
  1. 验证签名有效性（sr25519）
  2. 查链上余额
  3. 扣减链下待结算账本中该 payer 的虚拟余额
  4. 虚拟余额 = 链上余额 - 待结算总额
  5. 虚拟余额 ≥ amount + fee → 确认，记入链下账本，返回确认回执
  6. 虚拟余额不足 → 拒绝，返回余额不足错误

#### offchain_queryTxStatus

- 入参：tx_id
- 返回：交易状态（已确认待上链 / 已上链 / 未知）

### 2. 新增链下待结算账本（citizenchain/node/src/offchain_ledger.rs）

内存数据结构，支持并发读写：

```rust
// 每个 payer 的待结算累计金额
HashMap<AccountId, Balance>

// 待结算交易明细列表
Vec<OffchainBatchItem>

// 已确认交易索引（防重复提交）
HashSet<Hash>  // tx_id 集合
```

- 批量上链成功后清除对应记录
- 节点重启时从链上状态恢复（或清空，因为未上链的交易丢失由省储行承担）

### 3. 新增批量打包触发器（citizenchain/node/src/offchain_packer.rs）

后台任务（tokio spawn）：
- 监控待结算交易数量和时间
- 达到笔数阈值（PACK_TX_THRESHOLD = 100,000）或时间阈值（PACK_BLOCK_THRESHOLD = 60 分钟）
- 构造 OffchainBatch → 调用 enqueue_offchain_batch extrinsic 提交上链
- 上链成功 → 清除链下账本对应记录

### 4. RPC 依赖注入（citizenchain/node/src/service.rs）

将 offchain_ledger 和 offchain_packer 实例注入 FullDeps，供 RPC 方法使用。

## 涉及文件

- `citizenchain/node/src/rpc.rs` — 新增 2 个 RPC 方法
- `citizenchain/node/src/offchain_ledger.rs` — 新建
- `citizenchain/node/src/offchain_packer.rs` — 新建
- `citizenchain/node/src/service.rs` — 注入依赖
- `citizenchain/node/Cargo.toml` — 可能需要新增依赖

## 关键设计决策

- 链下账本使用内存结构，不持久化（省储行承担未上链交易的担保责任）
- 防双花靠链下账本虚拟余额扣减，不靠链上冻结
- 省储行节点监控交易池可发现链上普通转账冲突，但不强制阻止

## 验收标准

- [ ] wuminapp 可通过 WSS 连接省储行节点调用 offchain_submitSignedTx
- [ ] 签名验证、余额校验、虚拟余额扣减正常工作
- [ ] 重复提交同一 tx_id 被拒绝
- [ ] 达到阈值后自动批量打包上链
- [ ] 上链成功后链下账本正确清理
