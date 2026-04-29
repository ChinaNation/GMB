# 扫码支付 PaymentIntent 跨端 Golden Vectors

- **日期**:2026-04-20
- **范围**:Rust 运行时 + Dart 客户端两端互锁的 `PaymentIntent` SCALE 字节与 `signing_hash`
  golden 测试(E2E 冒烟第一层 · 协议字节对齐)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2C_I_PAY_PAGE.md`(Dart `NodePaymentIntent`)+ `batch_item.rs`
  (Rust `PaymentIntent`)
- **后续**:E2E 冒烟第二层 · 节点进程端到端 smoke

---

## 1. 目标

Step 2c-i 打通付款端后,**最大静默失败风险** = 跨端 SCALE 字节顺序 / 字节序 /
签名域前缀 / 哈希算法对不齐:提交成功但节点 `sr25519_verify` 失败,整个批次
revert,用户 L3 付款看似成功实则全挂。

本步用**同一组 fixture,两端分别断言同一串 hex**,任一端实现漂移立即两端 CI
同时红,作为跨端契约的硬锁。

---

## 2. Fixtures

3 组 fixture,覆盖同行 / 跨行 / 边界:

| # | 语义 | 关键字段 |
|---|---|---|
| 1 | 简单同行支付 | tx_id=0 / payer=[1;32] / payer_bank=recipient_bank=[2;32] / amount=10000 / fee=5 / nonce=1 / expires_at=100 |
| 2 | 跨行 + 极值 | tx_id=[0xFF;32] / payer_bank=[0xAA;32] / recipient_bank=[0xBB;32] / u128::MAX / u64::MAX / u32::MAX |
| 3 | 零金额 + tx_id 递增 | tx_id=[0x00..0x1F] / payer_bank=recipient_bank=[0x77;32] / amount=fee=nonce=expires_at=0 |

每组 fixture 锁两个 hex 期望:
- `encoded_hex`:SCALE 编码 204 字节(5×32 + 16+16 + 8 + 4)的 hex
- `signing_hash_hex`:`blake2_256(b"GMB_L3_PAY_V1" ++ encoded)` 的 hex(32 字节 → 64 hex 字符)

## 3. 锁定的 signing hashes

| Fixture | signing_hash hex (小写) |
|---|---|
| 1 | `f50eeb66b681e445ee6fcffa318288b915fdea9791eae1d094645d4eb5f7008f` |
| 2 | `d6f381b931ad0f2c7f7fba5d83bdd24892ccbd0e063d831ebc00d2e6d21c9bd8` |
| 3 | `8e99dbc826503544b240ed3e113f999bc3928048aa69989118f517309286a1b2` |

运行时 `PaymentIntent::signing_hash` 或 Dart `NodePaymentIntent.signingHash()`
的实现一旦改动,这 3 条 hex 必须同步重新捕获(Rust 跑一遍 → 读断言失败输出 →
同步到 Dart 测试)。

## 4. 文件清单

### 4.1 Rust 端

`citizenchain/runtime/transaction/offchain-transaction-pos/src/batch_item.rs`
内部 `tests` 模块追加 3 个单测 + 2 个工具(`hex_lower` / `assert_hex_eq`):
- `golden_fixture1_simple_same_bank`
- `golden_fixture2_cross_bank_big_values`
- `golden_fixture3_zero_amount_incrementing_tx`

### 4.2 Dart 端

`wuminapp/test/trade/payment_intent_golden_test.dart` 新建,5 个 test:
- fixture 1 / 2 / 3(与 Rust 同 fixture 同期望 hex)
- `signing domain bytes match Rust L3_PAY_SIGNING_DOMAIN`(锁 13 字节 ASCII)
- `scaleEncode length is always 204 bytes`

---

## 5. 顺带修复

### 5.1 `lib.rs:2406` mock `Test` 测试 runtime 补 `MaxAdminsPerInstitution`

`voting_engine::Config for Test` 缺少新近引入的
`type MaxAdminsPerInstitution`,导致整个 pallet 测试二进制编译失败。补
`ConstU32<32>` 后编译恢复,golden 测试可运行。

### 5.2 发现的 3 个**预存在失败**(非本步引入,已单独登记任务)

mock 修好后暴露出 3 个之前因编译失败而根本无法运行的业务测试:
- `tests::enqueue_offchain_batch_validates_admin_and_signature`(`NoPermission`)
- `tests::rate_update_requires_internal_vote_pass`
- `tests::submit_batch_executes_real_settlement_and_marks_processed`

不属于 Step 2c-ii-a 范围,已通过 spawn_task 登记独立修复。

---

## 6. 验证

```
$ cd citizenchain && cargo test --package offchain-transaction-pos --lib 'batch_item::tests'
test result: ok. 5 passed; 0 failed  # 3 个 golden + 2 个旧 signing_hash 基础测试

$ cd wuminapp && flutter test test/trade/payment_intent_golden_test.dart
All tests passed!  # 5 个

$ flutter analyze
No issues found!
```

---

## 7. 为什么不跑更高层冒烟

E2E 冒烟第二层(启 dev chain + 走真实 bind/deposit/pay)需要:
- `chain_spec.rs` dev preset 预置清算行 + 费率
- SFID 后端 mock 或 skip
- 进程管理 test harness

工作量 1-2 天。本步**先不做**,第一层协议字节锁定已经覆盖 Step 2c-i 引入的
最大风险(字节对齐)。第二层在 Step 2c-iii / Step 3 前视需要再做。

---

## 8. 变更记录

- 2026-04-20:Rust 侧 3 个 golden fixture 断言 + Dart 侧 5 个对应测试落地。
  顺带修复 `lib.rs` 测试 mock `MaxAdminsPerInstitution` 缺失(pallet 测试二进制
  重新可编译),暴露的 3 个预存在业务测试失败已 spawn_task 登记独立修复。
