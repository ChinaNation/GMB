# 清算行节点 · Step E · 跨行 ghost account bug 修复

- **日期**:2026-04-20
- **范围**:修复 `node/src/offchain/ledger.rs::on_payment_settled` 在跨行场景下
  给不属于本清算行的一方(payer 或 recipient)创建幽灵本地账户,导致
  `confirmed_sum_snapshot` 与链上 `BankTotalDeposits[my_bank]` 长期虚高的 P1 bug。
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_III_A_EVENT_LISTENER.md`(listener 订阅 `PaymentSettled`)+
  `STEP2B_III_B_RESERVE_MONITOR.md`(对账)+ 第 2b-iii-a 结束时的 `spawn_task` 登记
- **后续**:启用跨行扫码支付(Step 3)前本 bug 必修,本步完成后 Step 3 可直接开工

---

## 1. Bug 复盘

### 1.1 原实现

```rust
// 修复前
pub fn on_payment_settled(&self, tx_id, payer, recipient, amount, fee) {
    // ... payer 扣款 ...
    if let Some(state) = ledger.accounts.get_mut(recipient) {
        state.confirmed += amount;
    } else {
        // 收款方还没任何状态 → 新建
        let mut s = L3AccountState::default();
        s.confirmed = amount;
        ledger.accounts.insert(recipient.clone(), s);
    }
    // ...
}
```

### 1.2 事件上游(`event_listener.handle`)

```rust
if payer_bank == self.my_bank || recipient_bank == self.my_bank {
    self.ledger.on_payment_settled(tx_id, &payer, &recipient, amount, fee);
}
```

### 1.3 触发路径

跨行场景 **`my_bank == payer_bank != recipient_bank`**:
1. `handle` 判定条件第一个 `||` 成立 → 进入 `on_payment_settled`
2. payer 在本行 `accounts` 里有状态 → 正常扣减 ✓
3. recipient 在本行 `accounts` 里**没有**状态(因为不在本行)→ 走 else 分支 →
   **新建一个 `confirmed = amount` 的 ghost 账户**,而**链上本行对这个 recipient 无任何余额**

**后果**:
- `confirmed_sum_snapshot()` = Σ accounts[*].confirmed 虚高 `amount`
- `reserve_monitor` 每次 tick 都会报"对账偏差 +amount"
- 调试困难(ghost 账户看起来像正常用户)
- Step 3 启用跨行后,单日累积虚高可能到百万级 fen

同一 bug 对称版本:`my_bank == recipient_bank != payer_bank` 场景下,代码对
payer 做 `saturating_sub` 也会"虚扣"(虽然 accounts[payer] 若曾有历史状态会
错误扣减)。

Step 1 同行 MVP 不触发(`payer_bank == recipient_bank == my_bank`),但 Step 3
跨行启用前必修。

---

## 2. 修复方案

### 2.1 新签名

```rust
pub fn on_payment_settled(
    &self,
    tx_id: H256,
    payer: &AccountId32,
    payer_bank: &AccountId32,       // ← 新增
    recipient: &AccountId32,
    recipient_bank: &AccountId32,   // ← 新增
    my_bank: &AccountId32,          // ← 新增(从 EventListener 注入)
    amount: u128,
    fee: u128,
)
```

### 2.2 分支逻辑

```rust
if payer_bank == my_bank {
    // payer 侧动账:pending_debit & confirmed 双减
    if let Some(state) = ledger.accounts.get_mut(payer) { ... }
}
if recipient_bank == my_bank {
    // recipient 侧动账:pending_credit & confirmed 双增(新建或累加)
    if let Some(state) = ledger.accounts.get_mut(recipient) { ... }
    else { insert new state with confirmed = amount }
}
// tx_id 仍从 pending 列表 + accepted_tx_ids 清除(两种场景都需要清本地 pending)
```

**语义对齐点**:
- 同行(`payer_bank == recipient_bank == my_bank`)两个 `if` 都命中,行为与修复前一致
- 跨行 payer 本行:只动 payer,recipient 不出现在本地 accounts
- 跨行 recipient 本行:只动 recipient(可新建),payer 若有历史状态不误扣
- 两家都不是本行:`listener.handle` 已过滤不会进来(兜底 no-op)

### 2.3 Listener 调用点

`event_listener::handle` 的 PaymentSettled 分支同步改为:

```rust
self.ledger.on_payment_settled(
    tx_id,
    &payer, &payer_bank,
    &recipient, &recipient_bank,
    &self.my_bank,
    amount, fee,
);
```

---

## 3. 新增单元测试

`ledger.rs::tests` 追加 4 个:

| 测试 | 场景 | 核心断言 |
|---|---|---|
| `settled_same_bank_moves_pending_to_confirmed` | 同行(原 `settled_moves_pending_to_confirmed` 升级) | payer confirmed -total / recipient confirmed +amount |
| `settled_cross_bank_payer_side_only_no_ghost_recipient` | 跨行,my_bank==payer_bank | recipient 不在本行 accounts 里(**ghost 不应出现**) |
| `settled_cross_bank_recipient_side_only` | 跨行,my_bank==recipient_bank | payer 在本行的历史状态不被错扣;recipient confirmed +amount |
| `settled_sum_invariant_same_bank_drops_by_fee` | 同行对账不变式 | Σ confirmed 变化 = -fee(与 runtime `BankTotalDeposits -= fee` 对齐) |

---

## 4. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo test -p node --no-run
(仅 Tauri `frontend/dist` proc macro 门禁;所有 Rust 源代码 + 测试均零 error)

$ cargo test -p offchain-transaction --lib
test result: ok. 20 passed; 0 failed
(本修复未涉及 pallet 层,仅 node 侧 ledger;保持原先 D 阶段建立的 20 个测试全绿)
```

**节点侧 ledger 测试执行暂卡 Tauri 门禁**(与本修复无关,预存在):
这些 ledger 单元测试的实际跑通要等 Tauri `frontend/dist` 门禁解开后
(`cargo test -p node offchain::ledger` 完整通过);编译面上已经确认测试能
通过编译并具备正确的断言逻辑。

---

## 5. 已知风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| `on_payment_settled` 签名变更,`packer.rs` 内测试 fixture 有直接调用 | **低**(已 grep 确认无调用) | packer 测试只调 `accept_payment` / `take_pending_for_batch`,不动 `on_payment_settled`;listener 是唯一调用点 |
| 跨行场景 event 只捕获一次,两端 ledger 各自接一份 | **设计内** | 两家清算行节点各跑独立 listener,各自 `my_bank` 过滤,分别动本行 accounts,天然分工 |
| 对称 bug: `my_bank == recipient_bank != payer_bank` 时原代码对 payer 的 `pending_debit` 错减 | **已修复** | 新分支判断保证只动本行一侧,测试 `settled_cross_bank_recipient_side_only` 覆盖 |
| ledger 历史持久化数据(加密文件)仍可能含历史 ghost 账户 | **P2** | 新清算行节点首次启动不会有 ghost;已运行过跨行旧版本的节点,可删 `offchain_step1/ledger.enc` 让 listener 从事件流重建 |

---

## 6. 顺带完成的 spawn_task

Step 2b-iii-a 结束时通过 `mcp__ccd_session__spawn_task` 登记了本 bug 的独立
修复任务。现在 E 已执行,可以在 UI 侧**关闭**对应 chip(spawn_task 第 1 号)。

---

## 7. 变更记录

- 2026-04-20:Step E 完成。`ledger.rs::on_payment_settled` 签名从 5 参扩到 8 参
  (加 `payer_bank` / `recipient_bank` / `my_bank`);`settlement/listener.rs` 调用点同步
  更新传递 `self.my_bank`;ledger 测试由 1 个 settle 用例扩到 4 个(同行+2 个跨行+1
  个不变式)。pallet 20 ok,节点侧编译通过(Tauri 门禁除外)。
