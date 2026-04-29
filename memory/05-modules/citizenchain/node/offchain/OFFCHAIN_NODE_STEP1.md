# citizenchain/node/src/offchain/ · Step 1 技术说明

- **日期**:2026-04-19
- **范围**:扫码支付清算体系 Step 1 在 **Node 层**的已落地代码
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **总任务卡**:`memory/08-tasks/open/20260419-扫码支付-step1-同行MVP.md`
- **Runtime 对应文档**:`memory/05-modules/citizenchain/runtime/transaction/offchain-transaction/STEP1_TECHNICAL.md`

---

## 1. 范围

Step 1 节点层交付**骨架 + 数据结构 + 单测**,**不接入 `service.rs` 启动器**。旧 `offchain_ledger.rs` / `offchain_packer.rs` / `offchain_gossip.rs` 保留以支撑现有省储行清算路径,仅在文件头部注释加弃用说明。Step 2 起 `service.rs` 按节点角色切换到新 `offchain/` 组件,然后删除旧 3 文件。

## 2. 新目录结构

```
citizenchain/node/src/offchain/
├── mod.rs              # 模块聚合 + Step 1/2 范围说明
├── ledger.rs           # 清算行本地 L3 存款缓存账本
├── rpc.rs              # 对 wuminapp 的 JSON-RPC(Step 1 只读查询)
├── reserve.rs          # 主账对账
└── settlement/
    ├── packer.rs       # 批次打包器骨架(Step 2 启用 submit)
    └── listener.rs     # 链上事件 → 本地 ledger 同步器
```

`citizenchain/node/src/main.rs` 的 `mod` 声明已加入 `mod offchain;`,与旧 3 mod 并存。

## 3. 各文件职责

### 3.1 `offchain/ledger.rs`

本地账本缓存(权威账本在链上 `DepositBalance`)。数据结构:

```rust
pub struct L3AccountState {
    pub confirmed: u128,
    pub pending_debit: u128,
    pub pending_credit: u128,
    pub cached_nonce: u64,
}

pub struct PendingPayment {
    pub tx_id: H256, payer, payer_bank, recipient, recipient_bank,
    pub amount: u128, pub fee: u128, pub nonce: u64, pub expires_at: u32,
    pub payer_sig: [u8; 64], pub accepted_at: u64,
}

pub struct OffchainLedger { /* 内部 Arc<RwLock<LedgerInner>> + 加密持久化文件路径 */ }
```

对外接口:
- `get_state(user)` / `available_balance(user)` / `next_nonce(user)` / `pending_count()`
- `on_deposited(user, amount)` / `on_withdrawn(user, amount)` / `on_payment_settled(tx_id, payer, recipient, amount, fee)`
- `save_to_disk(password)` / `load_from_disk(password)`

**加密持久化**沿用原 `offchain_ledger.rs` 的 blake2_256 XOR + HMAC tag 方案,Step 2 再考虑升级到 AES-256-GCM。

### 3.2 `offchain/rpc.rs`

JSON-RPC namespace `offchain`,Step 1 暴露 3 个只读方法:
- `offchain_queryBalance(user) -> u128`
- `offchain_queryNextNonce(user) -> u64`
- `offchain_queryPendingCount() -> u64`

Step 2 起增加:
- `offchain_submitPayment(intent_hex, payer_sig_hex) -> {tx_id, l2_ack_sig}`
- `offchain_subscribeNotifications(user)` WebSocket 推送

### 3.3 `offchain/settlement/packer.rs`

骨架实现:
- `PACK_TX_THRESHOLD = 100_000`,`PACK_BLOCK_THRESHOLD = 10 块`
- `should_pack(current_block)`:检查触发条件(已实现)
- `pack_and_submit(current_block)`:返回 `Err("Step 1 not yet implemented, pending Step 2")`

Step 2 补齐:
- 从 `ledger.pending` 取 batch
- 清算行多签签批次
- 构造 `offchain_transaction::Call::submit_offchain_batch` extrinsic
- 通过 `TransactionPool` 提交

### 3.4 `offchain/settlement/listener.rs`

定义抽象 `OffchainChainEvent` 枚举(Step 2 由 `sc-client-api` 事件订阅解码填充):
- `Deposited { user, bank, amount }`
- `Withdrawn { user, bank, amount }`
- `PaymentSettled { tx_id, payer, payer_bank, recipient, recipient_bank, amount, fee }`
- `BankBound / BankSwitched`(仅日志)

`EventListener::handle(ev)` 按 `my_bank` 过滤,只处理与本清算行相关的事件。

## 4. 历史文件清理

| 文件 | 当前结论 |
|---|---|
| `node/src/offchain_ledger.rs` | 已删除,统一使用 `offchain/ledger.rs` |
| `node/src/offchain_packer.rs` | 已删除,统一使用 `offchain/settlement/packer.rs` |
| `node/src/offchain_gossip.rs` | 已删除,省储行间 gossip 路线不再作为 node 目录入口 |

清算行业务目录统一收口到 `citizenchain/node/src/offchain`。

## 5. 与 service.rs / rpc.rs 的接入(Step 2)

**本步不改 `service.rs` 和 `rpc.rs`**,保持现有节点运行逻辑不变。Step 2 新增:
- `service.rs`:按 `clearing_bank` 节点角色,启动 `OffchainLedger` + `Packer` + `EventListener`
- `rpc.rs`:注册 `offchain::rpc::OffchainClearingRpcImpl` 到 JSON-RPC module

## 6. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node
# `offchain/` 下 4 个文件编译通过(grep offchain 0 错误)
# node 最终链接阶段在 tauri proc macro 报错(frontend/dist 缺失,项目固有),与本步无关
# 生产 build 路径走 citizenchain/scripts/run.sh(先 build 前端 + CI WASM)
```

## 7. 单元测试覆盖

| 文件 | 测试 |
|---|---|
| `ledger.rs` | `deposited_then_withdrawn_roundtrip` / `save_load_roundtrip` / `wrong_password_rejected` / `settled_moves_pending_to_confirmed` |
| `rpc.rs` | `query_balance_returns_zero_for_unknown_user` / `query_balance_reflects_deposited` / `query_next_nonce_starts_at_one` |
| `settlement/packer.rs` | `should_pack_is_false_when_empty` |
| `settlement/listener.rs` | `deposited_event_updates_own_bank_ledger` / `deposited_event_ignored_for_other_bank` / `withdrawn_decreases_confirmed` |

Step 2 接入后补齐:`accept_payment` + `submit batch` + WS 推送的端到端测试。

## 8. 变更记录

- 2026-04-19:Step 1 节点层骨架落地,5 新文件 + main.rs 挂载 + 旧 3 文件标 deprecated。
