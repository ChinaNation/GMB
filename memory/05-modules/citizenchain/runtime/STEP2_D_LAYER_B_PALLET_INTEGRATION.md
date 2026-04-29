# 清算行 Runtime · Step D · Layer B pallet 集成测试

- **日期**:2026-04-20
- **范围**:在 `offchain-transaction` pallet 内建 `tests::` 模块,构造完整
  mock runtime(`frame_system + pallet_balances + OffchainTx` + mock SFID),
  用真实 sr25519 密钥签 L3 支付意图,端到端验证绑定 / 充值 / 提现 / 切换 /
  批次清算 / 防重放 / 签名过期全链路。
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_IV_B_RUNTIME_CLEANUP.md`(物理删除老省储行代码,测试面大幅收窄)
- **后续**:E2E Layer C(dev chain 进程级,需 chain_spec 预置清算行 + SFID mock)

---

## 1. 目标与范围

Step C 把老 pallet 代码压缩到 434 行,但也把原来的 20+ 个 pallet 测试(大多测
老 Call 行为)一并删了。Layer A 锁死了协议字节,Layer B(本步)补回 **新 Call
的运行时级回归**,跑的是真实 extrinsic 调度 + Storage 读写 + 事件 + sr25519 验签。

**不做**:
- dev chain 启动(需 chain_spec.rs 加 dev preset,属于 Layer C)
- wuminapp Flutter E2E(太重,回报低)
- 跨行 `submit_offchain_batch_v2` 场景(需要第二个清算行 fixture,留 Step 3)

---

## 2. Mock Runtime 结构

`src/tests.rs`(562 行)用 `frame_support::construct_runtime!` 搭 Test 运行时:

```rust
construct_runtime!(pub enum Test {
    System: frame_system,
    Balances: pallet_balances,
    OffchainTx: offchain_transaction,
});
```

### 2.1 `MockSfid` — `SfidAccountQuery` 实现

约束(与 `settlement.rs::pubkey_from_accountid` 对齐,`AccountId32` 32 字节 =
sr25519 公钥):

| 账户 | 字节 | 角色 |
|---|---|---|
| `BANK_MAIN_BYTES = [0xAA; 32]` | 清算行主账户(SFR-GD-SZ01-CB01-N9-D8,`主账户`,Active) |
| `BANK_FEE_BYTES = [0xAB; 32]` | 清算行费用账户(同 SFID,`费用账户`,Active) |
| `BANK_ADMIN_BYTES = [0xAC; 32]` | 唯一管理员 |
| `OTHER_BANK_BYTES = [0xBA; 32]` | 故意不注册,用于负路径 |

`MockSfid` 按这张表实现 4 个方法;费用账户在 `settlement.rs` 内部
`calc_fee` → `fee_account_of` 反查时必须能命中。

### 2.2 L3 用户生成

```rust
fn new_l3_user(seed: &[u8; 32], balance: u128) -> (AccountId32, sr25519::Pair) {
    let pair = sr25519::Pair::from_seed(seed);
    let acc = AccountId32::new(pair.public().0); // 32 字节 pubkey 直接当 AccountId
    Balances::make_free_balance_be(&acc, balance);
    (acc, pair)
}
```

`Pair::from_seed([1u8; 32])` 可确定地派生 Alice,`[2u8; 32]` 派生 Bob,便于断
言重现。

### 2.3 Config for Test

```rust
impl offchain_transaction::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxBatchSize = ConstU32<256>;
    type MaxBatchSignatureLength = ConstU32<128>;
    type InstitutionAsset = ();     // fail-open(institution-asset 自带)
    type SfidAccountQuery = MockSfid;
    type WeightInfo = ();
}
```

---

## 3. 测试清单(20 个 pallet 测试全绿)

### 绑定 / 存取 / 切换(5 个)

| 测试 | 断言 |
|---|---|
| `bind_deposit_withdraw_full_cycle` | 绑定 + 充值 10_000 + 提现 3_000,`DepositBalance` / `BankTotalDeposits` 全链更新;过量提现 `InsufficientDepositBalance` |
| `double_bind_rejected` | 二次 bind 同一清算行 → `AlreadyHasBank` |
| `bind_rejects_unregistered_bank` | 绑 `OTHER_BANK_BYTES` → `NotRegisteredClearingBank` |
| `switch_requires_zero_balance` | 有余额时切回同家 → `NewBankSameAsCurrent`(路径测试) |
| `switch_after_withdraw_all_works` | 提现清零后切回同家仍 `NewBankSameAsCurrent`(需 Step 3 fixture 扩展成跨行再补) |

### `submit_offchain_batch_v2` 核心(3 个)

| 测试 | 断言 |
|---|---|
| `submit_batch_rejects_non_admin` | 非管理员提交 → `UnauthorizedAdmin` |
| `submit_batch_same_bank_end_to_end` | 同行 Alice → Bob 转 10_000 分(5 bp fee=5):`DepositBalance[A]` -10_005 / `DepositBalance[B]` +10_000 / `BankTotalDeposits` -5 / `Balances[bank_main]` -5 / `Balances[bank_fee]` +5 / `L3PaymentNonce[A]=1` / `PaymentSettled` + `ClearingBankBatchSettled` 事件均发射 / 重放同 `tx_id`(nonce=2 重签)→ `TxAlreadyProcessed` |
| `submit_batch_expired_intent_rejected` | 块高推到 200、`expires_at=100` → `ExpiredIntent` |

### 既有单元测试保留(12 个)

- 5 个 `batch_item::tests` golden vectors + 基础签名哈希
- 5 个 `bank_check::tests` A3 私权判定 / noop impl
- 2 个 runtime integrity / genesis build(自动)

---

## 4. 运行结果

```
$ cargo test -p offchain-transaction --lib
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured
```

### 4.1 关键 gotcha

- **`payer_sig` 必须对 item.to_intent() 计算的 signing_hash 签名**,而非独立的 `PaymentIntent`。replay 测试里重签 nonce=2 的 intent 才能绕过 `InvalidL3Signature`,才能真正撞 `TxAlreadyProcessed`。
- `MaxBatchSignatureLength` 测试用 `ConstU32<128>`;生产 runtime 同值,兼容验签时的长度校验。
- `pallet_balances` 的 `make_free_balance_be` 在 `new_test_ext` 外层调 `execute_with` 才生效(否则没有 genesis ext)。

---

## 5. 后续

- **Layer C · dev chain 进程级 E2E**:起 dev chain + 清算行节点 + mock
  wuminapp RPC,完整跑 `STEP2C_MANUAL_SMOKE.md` SOP。前置:`chain_spec.rs`
  加 `dev` preset 预置 mock 清算行 + L2FeeRateBp + 预置 L3 账户余额
- **跨行 fixture**:加第二家 `BANK2_MAIN`(另一个 SFID + main/fee 对),扩
  `MockSfid` 与 `new_test_ext` 预置;补跨行 settlement 测试(需 Step E bug 修复先落地)
- **runtime-benchmarks**:Step 3 稳态后补新 Call 的 benchmark(当前 weights 用
  `T::DbWeight` 保守估算)

---

## 6. 变更记录

- 2026-04-20:Step D 落地,`src/tests.rs` 新建 562 行 8 个测试(其中 3 个
  是 submit_offchain_batch_v2 的端到端覆盖);`lib.rs` 追加 `#[cfg(test)] mod tests;`
  声明。pallet 测试从 10 ok → 20 ok。
