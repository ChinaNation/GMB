# 清算行节点 substrate 集成 · Step 2b-iii-b 技术说明

- **日期**:2026-04-19
- **范围**:扫码支付 Step 2b-iii-b(`reserve_monitor.rs` 主账对账 + CLI flag + service.rs 启动)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_III_A_EVENT_LISTENER.md`(listener 订阅 runtime 事件同步 ledger)
- **后续**:`STEP2B_III_C_GOSSIP.md`(跨行意图推送,Step 3 范围)与 `STEP2B_IV_CLEANUP.md`(删除旧 offchain_*.rs)

---

## 1. 为什么要做这一步

Step 2b-iii-a 完成后,扫码支付进入"wuminapp RPC → ledger → packer → pool → runtime → listener → ledger 清 pending"的完整闭环。但这个闭环里 listener 是**唯一**的本地账面同步路径:

- listener 进程崩溃 / tokio task panic
- `import_notification_stream` 短暂断流
- runtime 发了 `Deposited` / `Withdrawn` / `PaymentSettled` 事件但节点没看到
- ledger 持久化文件被破坏后加载了半截状态

以上任何一种,都会让 **本地 `Σ accounts[*].confirmed` 偏离链上 `BankTotalDeposits[my_bank]`**。偏差一旦出现,没有自动监测就没人发现,扫码支付会在余额虚增 / 虚减状态下继续运行,引发严重资金风险。

本步的 `reserve_monitor` 就是这个保底:**周期性读链上单点值,对比本地 snapshot,偏差 log::error!**。

---

## 2. 不变式分析(对账公式的正确性)

### 2.1 runtime 侧

`deposit.rs` / `settlement.rs` 逐 mutate 路径都同步更新 `DepositBalance` 和 `BankTotalDeposits`,由此 runtime 保证:

```text
BankTotalDeposits[my_bank] == Σ DepositBalance[my_bank][*]
```

### 2.2 node listener 侧

`ledger.rs` 的三个事件 handler:

- `on_deposited(user, amount)`:`accounts[user].confirmed += amount`
- `on_withdrawn(user, amount)`:`accounts[user].confirmed -= amount`
- `on_payment_settled(...)`:payer `confirmed -= (amount+fee)`,同行 recipient `confirmed += amount`

净效应(同行):`Σ confirmed` 变化 `= -(amount+fee) + amount = -fee`。

runtime 同行 `BankTotalDeposits -= fee`,正好对齐。

### 2.3 pending 期间

扫码支付 `accept_payment` 只改本地 `pending_debit` / `pending[]`,**不动** `confirmed`,同时链上 `DepositBalance` 也**没变**(settlement 还没上链)。所以 pending 期间两边仍然相等。

### 2.4 结论

```text
Σ ledger.accounts[*].confirmed == BankTotalDeposits[my_bank]   (listener 正常时恒成立)
```

主账对账足够发现 listener 层任何漂移。逐户 diff(`DepositBalance::iter_key_prefix(my_bank)` vs `accounts[user].confirmed`)是"定位"工具,不是"发现"工具,放 Step 3 再做。

---

## 3. 改动清单

### 3.1 新增 `offchain/reserve_monitor.rs`(~190 行)

| 组件 | 内容 |
|---|---|
| `ReserveMonitor { ledger, my_bank }` | 持账本 + 本行主账户 |
| `run(self: Arc<Self>, client, interval)` | 长循环;**首 tick 跳过**(等待 listener 追 chain 头稳定),之后按 interval 触发 |
| `check_once(&self, client)` | 读 `client.info().best_hash` → 读链上 `BankTotalDeposits[my_bank]` → 与 `ledger.confirmed_sum_snapshot()` 比较;相等 `debug!`,不等 `error!` |
| `bank_total_deposits_key(bank)` | 构造 `twox_128("OffchainTransactionPos") ++ twox_128("BankTotalDeposits") ++ blake2_128(encoded) ++ encoded` |
| `read_bank_total_deposits(client, block, bank)` | 读 storage → SCALE decode u128;`None → 0`(`ValueQuery` 语义) |
| 单测 | 5 个(`confirmed_sum_empty` / `confirmed_sum_adds` / `confirmed_sum_after_withdraw` / `storage_key_layout_stable` / `storage_key_differs_per_bank`) |

### 3.2 `offchain/ledger.rs`

新增 `pub fn confirmed_sum_snapshot(&self) -> u128`:`Σ accounts[*].confirmed`。

注释明确:`pending_debit` / `pending_credit` 不计入 —— pending 期间链上和本地都"未扣",两边保持相等。

### 3.3 `offchain/mod.rs`

- `pub mod reserve_monitor;`
- `OffchainComponents` 追加 `pub reserve_monitor: Arc<ReserveMonitor>`
- `start_clearing_bank_components` 构造 `Arc::new(ReserveMonitor::new(ledger.clone(), bank_main))`
- `event_listener::new(...)` 因此要 `bank_main.clone()`(给 reserve_monitor 留 ownership)

### 3.4 `cli.rs`

```rust
#[arg(long, value_name = "SECS")]
pub clearing_reserve_monitor_interval_secs: Option<u64>,
```

缺省 `None`(实际生效 300 秒);`Some(0)` 关闭对账。

### 3.5 `service.rs`

`new_full` 签名再加 1 个参数:

```rust
pub fn new_full(
    mut config: Configuration,
    mining_threads: usize,
    gpu_device: Option<usize>,
    clearing_bank: Option<String>,
    clearing_bank_password: Option<String>,
    clearing_reserve_monitor_interval_secs: Option<u64>,   // ← 新增
) -> Result<TaskManager, ServiceError>
```

清算行启动块内、event_listener spawn 之后追加:

```rust
let monitor_interval_secs = clearing_reserve_monitor_interval_secs.unwrap_or(300);
if monitor_interval_secs > 0 {
    task_manager.spawn_handle().spawn(
        "offchain-clearing-reserve-monitor",
        Some("offchain"),
        async move {
            monitor.run(client_for_monitor, Duration::from_secs(monitor_interval_secs)).await;
        },
    );
} else {
    log::warn!("[ClearingBank] reserve_monitor 已关闭(interval=0),仅用于排障...");
}
```

### 3.6 `command.rs`

`runner.run_node_until_exit` 闭包加 `clearing_reserve_monitor_interval_secs` 透传。

### 3.7 `ui/node_runner.rs`

Tauri UI 入口 `new_full(..., None, None, None)`:UI 暂不支持清算行角色,对账参数也透传 `None`。

---

## 4. 启动示例

```bash
# 默认对账周期(5 分钟)
./target/release/citizenchain \
  --chain local \
  --clearing-bank 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --clearing-bank-password "my-strong-password"

# 自定义对账周期(1 分钟,压测时用)
./target/release/citizenchain \
  --chain local \
  --clearing-bank 5GrwvaEF5...  \
  --clearing-bank-password "..." \
  --clearing-reserve-monitor-interval-secs 60

# 关闭对账(仅用于排障,生产务必保留默认)
./target/release/citizenchain \
  --chain local \
  --clearing-bank 5GrwvaEF5... \
  --clearing-bank-password "..." \
  --clearing-reserve-monitor-interval-secs 0
```

---

## 5. 运行时日志期望

正常:
```
INFO  [ClearingBank] 清算行组件已启动,bank_main=5Grwv...
INFO  [ReserveMonitor] 启动主账对账 interval=300s bank=...
DEBUG [ReserveMonitor] ok local=chain=123456 block=0xabc...
DEBUG [ReserveMonitor] ok local=chain=123500 block=0xbcd...
```

偏差:
```
ERROR [ReserveMonitor] 对账偏差! local=123456 chain=123500 diff=-44 block=0xabc...
```

读取异常:
```
WARN  [ReserveMonitor] 对账失败:storage 读取失败:...
```

---

## 6. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node --tests
(offchain / service / rpc / cli / command / node_runner 全部零 error;
 链接阶段仅剩 ui/mod.rs:91 tauri proc macro `frontend/dist` 门禁,与本步无关)
```

---

## 7. 已知风险与缓解

| 风险 | 等级 | 现状 |
|---|---|---|
| storage key 哈希算法与 runtime 不一致 → 永远读到 0 | **P0** | 单测 `bank_total_deposits_key_layout_stable` 锁死布局;若 runtime 改 `Blake2_128Concat` 为其他 hasher,本测试会立刻失败 |
| 节点启动瞬间 listener 尚未追完 → 误报偏差 | **P1** | `run()` 的 `ticker.tick().await` 首次立即返回被跳过,等一个 interval 再对账;虽然不完美,但 300 秒足够一般场景追上 |
| 跨行 ghost account 已知 bug 会干扰对账 | **P2** | `ledger.on_payment_settled` 在"跨行 + my_bank=payer_bank"时会给 recipient 新建一个 confirmed=amount 的幽灵账户,导致本地 sum 虚高。Step 1 同行 MVP 不触发;Step 3 跨行前必须先修(新开任务) |
| 对账窗口内反复读同一块 hash | **P3** | 每 tick 拿 `client.info().best_hash`,若链不出块 diff 会在"上一个确认块"读取,正确性不受影响 |
| 偏差后无自动停扫码支付 | **P2** | 本步仅 log::error!;Step 3 引入"偏差 > 阈值 → 停新 accept_payment"的保护 |

---

## 8. 不做(留后续)

**Step 2b-iii-c**:
- `offchain/gossip.rs`:libp2p `NotificationService` + 协议名 `/gmb/offchain/1`(跨行扫码支付预检推送)

**Step 2b-iv**(清理):
- 删除 `node/src/offchain_{ledger,packer,gossip}.rs`
- runtime `offchain-transaction-pos` 删 call_index 0/1/2/9 老 calls

**Step 3**:
- 逐户 diff(`DepositBalance::iter_key_prefix`)
- Prometheus metric `clearing_bank_reserve_diff`
- 偏差超阈值自动停 accept_payment
- 修复跨行 ghost account bug

---

## 9. 变更记录

- 2026-04-19:Step 2b-iii-b 完整落地。reserve_monitor.rs 新建;ledger.rs 加 `confirmed_sum_snapshot`;mod.rs / cli.rs / service.rs / command.rs / ui/node_runner.rs 全链路透传 interval 参数;5 个单测(snapshot 3 + storage key 2)。零编译错误。
