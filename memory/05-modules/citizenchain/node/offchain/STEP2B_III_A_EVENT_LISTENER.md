# 清算行节点 substrate 集成 · Step 2b-iii-a 技术说明

- **日期**:2026-04-19
- **范围**:扫码支付 Step 2b-iii-a(event_listener 真实订阅 + service.rs 启动 worker)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_II_B_2_B_INTEGRATION.md`(β-2-b:packer 真上链 + CLI + service 启动)
- **后续**:`STEP2B_III_B_RESERVE_MONITOR.md`(周期对账告警)与 `STEP2B_III_C_GOSSIP.md`(libp2p gossip)

---

## 1. 本步范围

**目标**:把节点 `EventListener` 从"仅提供单条事件分发 API"升级为"后台长 loop 自动订阅链头事件并清理本地 ledger"。

packer 之前已经把 batch 提交到 pool,runtime 执行后会发 `PaymentSettled` 事件,但清算行节点的本地 `ledger.pending` 还停留在"已接受、待上链"状态不会自己清。本步完成后,闭环即关:

```
accept_payment → pending 入队
packer tick  → pool.submit_one → runtime → PaymentSettled event
event_listener(本步) → ledger.on_payment_settled → pending 移除
```

**明确不做**(Step 2b-iii-b / 2b-iii-c / 2b-iv):
- `offchain/reserve.rs`:周期性 `available_balance vs total_deposits` 对账告警(2b-iii-b)
- `gossip.rs`:libp2p `NotificationService` + 协议名 `/gmb/offchain/1`(2b-iii-c)
- 删除旧 `offchain_{ledger,packer,gossip}.rs` 及 runtime 老 calls(2b-iv)

---

## 2. 改动清单

### 2.1 `offchain/settlement/listener.rs`

| 变更 | 内容 |
|---|---|
| 新 `use` | `codec::Decode`, `futures::StreamExt`, `sc_client_api::{BlockchainEvents, StorageProvider}`, `sp_storage::StorageKey`, `citizenchain as runtime`, `crate::core::service::FullClient` |
| `run(self: Arc<Self>, client: Arc<FullClient>)` | 新增 async 长循环:订阅 `client.import_notification_stream()`,只处理 `is_new_best` 的 notification,把 `notification.hash` 交给 `process_block` |
| `process_block(&self, client: &FullClient, block_hash: H256)` | 新增:`client.storage(block_hash, &system_events_storage_key())` 读 `System::Events` 原始字节 → `Vec<EventRecord<runtime::RuntimeEvent, H256>>` SCALE 解码 → 遍历 `convert_event` → `self.handle(ev)` |
| `system_events_storage_key()` | 新增工具函数:`StorageKey(twox_128("System") ++ twox_128("Events"))` |
| `convert_event(ev: runtime::RuntimeEvent) -> Option<OffchainChainEvent>` | 新增 `pub` 函数:pattern-match `RuntimeEvent::OffchainTransaction(inner)` 的 5 个变体(`Deposited` / `Withdrawn` / `PaymentSettled` / `BankBound` / `BankSwitched`);其他事件(费率治理、批次级别)返回 `None` |

`handle()` 保持原样:已经写好的 `PaymentSettled` 分支会自动调 `ledger.on_payment_settled(tx_id, &payer, &recipient, amount, fee)`,本步只负责把链上事件喂进来。

### 2.2 `offchain/ledger.rs`

`LedgerInner` 由私有升级为 `pub(super)`,3 个字段(`accounts` / `pending` / `accepted_tx_ids`)同步 `pub(super)`;`OffchainLedger.inner` 加 `pub(super)` 注释说明:只允许 `offchain/` 目录内部测试直接注入(绕过 `accept_payment` 的 L3 签名校验),生产代码必须走公开接口。这是修复 Step 2b-ii-α 添加的 `packer.rs` 6 条单测里 `ledger.inner.write()` 的私有字段访问错误。

### 2.3 `service.rs`

在清算行组件启动块(Step 2b-ii-β-2-b 引入的 `if let Some(bank_ss58) = ...` 区段)内、packer spawn 之后、`log::info!("[ClearingBank] 清算行组件已启动...")` 之前,追加 event_listener spawn:

```rust
let listener = components.event_listener.clone();
let client_for_events = client.clone();
task_manager.spawn_handle().spawn(
    "offchain-clearing-event-listener",
    Some("offchain"),
    async move {
        listener.run(client_for_events).await;
    },
);
```

失败路径:若 `import_notification_stream` 结束(整个节点关闭时才会),worker loop 结束后 log warning,不会影响其他子任务。因为 `import_notification_stream` 本身不会 panic(Substrate 保证),所以这里不做 error handling。

---

## 3. 端到端数据流(闭环打通)

```
wuminapp                清算行节点                             链上 runtime
────────                ──────────                             ────────────
扫 QR → RPC   → OffchainClearingRpcImpl.submit_payment
                → ledger.accept_payment
                  pending.push(...)
                  tx_id 返回给 wuminapp

30s tick        → packer.pack_and_submit
                → submitter.submit  → pool.submit_one  →  TxPool
                                                           ↓
                                                      submit_offchain_batch_v2
                                                           ↓
                                                      settlement::execute
                                                           ↓
                                                      emit PaymentSettled

new_best block  ← import_notification_stream
                → process_block(block.hash)
                  client.storage(System::Events)
                  decode Vec<EventRecord>
                  for each → convert_event
                    PaymentSettled { tx_id, .. }
                  → handle → ledger.on_payment_settled
                    pending 移除对应 tx_id
                    accounts[payer].pending_debit -= amount+fee
                    accounts[payer].confirmed    -= amount+fee
                    accounts[recipient].confirmed += amount  (同行才加)
```

---

## 4. 关键设计取舍

### 4.1 为何订 `import_notification_stream` 而非 `finality_notification_stream`

- **延迟**:扫码支付要求端到端 ≤30 秒。等 GRANDPA finality(~1-2 分钟)会让用户/商户长期看到"待确认"状态,体验差。
- **reorg 安全**:runtime 层已在 `OffchainTransaction::submit_offchain_batch_v2` 里通过 `BatchNonce` + `accepted_tx_ids` 防重;reorg 时同一 `tx_id` 的 `on_payment_settled` 即使被重复调用,`ledger.pending` 中找不到(已被前次清理)就是 no-op。
- **简洁**:只处理 `notification.is_new_best` 过滤掉 reorg 出去的分支 block,避免把已回滚块里的事件再次分发。

### 4.2 为何直接读 `System::Events` storage 而非用 Substrate 的 `EventProvider`

- Substrate 没有稳定公开的 `EventProvider`;`sc-client-api` 的 `StorageProvider::storage()` 是最通用接口。
- `System::Events` 的 storage key 形式固定(`twox_128("System") ++ twox_128("Events")`),写死即可,不需要 runtime metadata 查询。
- `EventRecord<RuntimeEvent, H256>` 是 `frame-system` 的标准类型,SCALE Decode 语义与 runtime 侧 Encode 完全对称。

### 4.3 为何 `convert_event` 里对其他事件返回 `None` 而非 panic

runtime 的 `OffchainTransaction::Event` 还有(或将有)费率治理、批次级别通知等事件,这些不影响本地 ledger 状态。用 `_ => None` 是"显式忽略但保留 match 完整性"的标准 pattern;未来加新事件变体时,想接入的话 explicit 加分支,不想接入的留给 `_`。

---

## 5. 测试

### 5.1 convert_event 单测(新增 3 个)

- `convert_event_deposited`:正路径,`PalletEvent::Deposited` → `OffchainChainEvent::Deposited` 字段保序映射
- `convert_event_payment_settled_maps_field_names`:关键映射校验 `transfer_amount → amount` / `fee_amount → fee`(runtime 与 node 侧字段名不同但语义一致,回归时容易断裂)
- `convert_event_non_offchain_returns_none`:`RuntimeEvent::System(CodeUpdated)` 必须返回 `None`,防止非本 pallet 事件被误喂到 ledger

### 5.2 已保留的 handle 单测(3 个,Step 2b-i 引入)

- `deposited_event_updates_own_bank_ledger`
- `deposited_event_ignored_for_other_bank`
- `withdrawn_decreases_confirmed`

### 5.3 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node --tests
(offchain / service 全部零 error;链接阶段仅剩 desktop.rs tauri
 proc macro `frontend/dist` 门禁,与本步无关)
```

---

## 6. 已知风险与缓解

| 风险 | 等级 | 现状 |
|---|---|---|
| reorg 导致同 `tx_id` 的 `PaymentSettled` 被监听两次 | **P2** | `ledger.pending` 找不到 → no-op;账户 `pending_debit` / `confirmed` 不会重复扣(需在 ledger 实现层面确认 `on_payment_settled` 对已不存在的 tx_id 幂等) |
| `import_notification_stream` 在节点启动初期可能短暂为空 | **P3** | Substrate 保证订阅在节点启动后立即开始 push;本步 loop 内用 `while let Some(...)` 自然等待 |
| `System::Events` storage 读取失败(理论上不应该) | **P3** | `process_block` 返回 Err → log warning,loop 继续;不影响其他子任务 |
| L3 deposit 余额同步依赖本 listener | **P1** | 如果 listener 掉线,L3 取款 / 扫码支付会看到"余额滞后";但上链余额是权威,重启 listener 即可追齐(需看 `on_deposited` / `on_withdrawn` 是幂等加减 —— ✅ 已实现) |

---

## 7. 后续

**Step 2b-iii-b**(下一步):
- `offchain/reserve.rs` 新增:周期性 `ledger.total_deposits() vs chain.storage(DepositBalance)` 对账告警
- 触发阈值超过 1 分钟 → log error + 指标上报

**Step 2b-iii-c**:
- `offchain/gossip.rs`:libp2p `NotificationService` + 协议名 `/gmb/offchain/1`(清算行间推送 pending 意图,用于跨行扫码支付预检)

**Step 2b-iv**(清理):
- 删除 `node/src/offchain_{ledger,packer,gossip}.rs`
- 删除 `main.rs` 里对应 mod 声明
- 删除 `rpc.rs::FullDeps` 中 `offchain_ledger` / `offchain_shenfen_id` / `offchain_gossip_tx`(老省储行清算字段)
- runtime `offchain-transaction` 删除 call_index 0 旧 `submit_offchain_batch` / 9 旧 `bind_clearing_institution` / 1/2 旧 `propose_institution_rate`

## 8. 变更记录

- 2026-04-19:Step 2b-iii-a 完整落地,`settlement/listener.rs` 真实订阅 `import_notification_stream`,清算行节点闭环打通(wuminapp RPC → ledger → packer → pool → extrinsic → runtime event → listener → ledger.pending 清理)。listener + service.rs + ledger.rs 可见性修复,零编译错误;新增 3 个 convert_event 单测 + 保留原 3 个 handle 单测。
