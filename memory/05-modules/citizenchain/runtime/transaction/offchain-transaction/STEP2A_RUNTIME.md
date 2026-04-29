# offchain-transaction · 扫码支付 Step 2a 技术说明(Runtime 重写层)

- **日期**:2026-04-19
- **范围**:扫码支付清算体系 Step 2 在 **Runtime 层**的落地代码(本体)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`(其第 5 节 "Step 2 留档"即本步)
- **总任务卡**:`memory/08-tasks/open/20260419-扫码支付-step1-同行MVP.md`
- **前置文档**:`STEP1_TECHNICAL.md`(Step 1 Runtime)
- **后续文档**:`STEP2B_NODE.md`(节点接入)、`STEP2C_WUMINAPP.md`(前端)、`STEP2D_CLEANUP.md`(删除旧代码)

---

## 1. 本步范围

Step 2a 只做 **Runtime 新增**,与旧"省储行清算"代码路径**共存**:

- 新增 3 个子模块:`fee_config.rs` / `solvency.rs` / `settlement.rs`
- 新增 1 个结构:`batch_item::OffchainBatchItemV2`
- 新增 4 个 Storage:`L2FeeRateBp` / `L2FeeRateProposed` / `MaxL2FeeRateBp` / `LastClearingBatchSeq`
- 新增 5 个 Event:`L2FeeRateProposed` / `L2FeeRateActivated` / `MaxL2FeeRateUpdated` / `PaymentSettled` / `ClearingBankBatchSettled`
- 新增 Error 覆盖清算行结算、费率、偿付、batch 签名与用户绑定一致性:
  `InstitutionMismatch` / `ExpiredIntent` / `L2FeeRateNotConfigured` / `InvalidL3Signature` /
  `InvalidL2FeeRate` / `SolvencyProtected` / `InvalidBatchSignature` / `InvalidBatchSeq` /
  `UserBankMismatch`
- 新增 3 个 Call:`submit_offchain_batch_v2`(34)/ `propose_l2_fee_rate`(40)/ `set_max_l2_fee_rate`(41)
- 新增 trait 方法:`SfidAccountQuery::is_admin_of`
- 扩展 `on_initialize`:激活到期费率提案
- runtime 层 `DuoqianSfidAccountQuery` 实现 `is_admin_of`
- runtime 层 `OnchainTxAmountExtractor` 分类 3 个新 call

**明确不做**(留 Step 2b/2c/2d):
- Node 层接入(Step 2b)
- wuminapp 改造(Step 2c)
- 删除旧 `submit_offchain_batch` / `bind_clearing_institution` / `RecipientClearingInstitution` / `InstitutionRateBp`(Step 2d)
- 联合投票回调 `set_max_l2_fee_rate`(Step 2b 接入;当前为 Root Origin)
- 争议仲裁 / 保证金(Step 3)

## 2. 新增文件

### 2.1 `fee_config.rs`

清算行费率自治:
- `L2_FEE_RATE_BP_MIN = 1` / `L2_FEE_RATE_BP_MAX = 10`(bp)
- `RATE_CHANGE_DELAY_BLOCKS = 20_160`(约 7 天,30 秒/块)
- `do_propose_l2_fee_rate(who, bank, new_rate)`:管理员提案,写 `L2FeeRateProposed`
- `do_set_max_l2_fee_rate(new_max)`:设全局上限(Step 2b 改为联合投票回调)
- `activate_pending_rates(now)`:`on_initialize` 调用,搬到期提案到 `L2FeeRateBp`
- `current_rate_bp(bank)`:查当前生效费率(供 settlement 查收款方费率用)

### 2.2 `solvency.rs`

偿付能力自动保护:
- `ensure_can_debit(bank_main, debit_fen)`:校验扣款后主账户余额仍 ≥ `BankTotalDeposits`
- `solvency_ratio_bp(bank_main)`:返回偿付率(万分之)供监控用
- Step 3 追加 `emit_warning_if_low` 告警事件 + 自动冻结

### 2.3 `settlement.rs`

清算行批次的新 execute 路径:
- `execute_clearing_bank_batch(submitter, institution_main, batch)`:批次级执行入口
  - 批次级预检:submitter 管理员身份 / batch_signature / batch_seq / UserBank 绑定一致性 / 费率正确性 / 偿付充足
  - 逐笔 `execute_single_item`:L3 签名验证 / nonce / 分账(同行 vs 跨行)/ 防重放
- 费率按 **收款方清算行** `L2FeeRateBp[recipient_bank]` 计算
- 手续费**全部归收款方清算行的费用账户**,无省储行分成

### 2.4 `batch_item::OffchainBatchItemV2`

与现有 `OffchainBatchItem` 并存的新批次项:
```rust
pub struct OffchainBatchItemV2<AccountId, BlockNumber> {
    pub tx_id: H256,
    pub payer: AccountId,
    pub payer_bank: AccountId,
    pub recipient: AccountId,
    pub recipient_bank: AccountId,
    pub transfer_amount: u128,
    pub fee_amount: u128,
    pub payer_sig: [u8; 64],   // L3 sr25519 签名
    pub payer_nonce: u64,
    pub expires_at: BlockNumber,
}
```

`to_intent()` 反向构造 `PaymentIntent` 用于重算签名哈希验签。

### 2.5 `bank_check::SfidAccountQuery::is_admin_of`

trait 新增方法,`()` 默认返回 false。runtime 侧 `DuoqianSfidAccountQuery` 先通过 `duoqian_manage::Pallet::resolve_admin_subject_for_account` 找到机构或个人多签对应的管理员主体，再委托 `admins_change::Pallet::is_subject_admin` 校验。

2026-04-29 补齐：清算行管理员真源不再是 `DuoqianAccounts.duoqian_admins`，所有内部投票和清算权限统一读取 `admins-change::Institutions`。

## 3. lib.rs 扩展

### 3.1 Storage

```rust
L2FeeRateBp<Bank, u32>                           // 当前生效费率
L2FeeRateProposed<Bank, (u32, BlockNumber)>      // 待生效提案
MaxL2FeeRateBp: StorageValue<u32>                // 全局上限
LastClearingBatchSeq<Bank, u64>                  // 已成功落账的最新批次序号
```

2026-04-28 补齐:`LastClearingBatchSeq` 与 batch 级签名一起启用。`submit_offchain_batch_v2`
要求 `batch_seq == LastClearingBatchSeq[bank] + 1`,并只在 settlement 成功后推进序号。

### 3.2 Call(3 个新)

| call_index | 方法 | 费用 | 归类 |
|---|---|---|---|
| 34 | `submit_offchain_batch_v2` | sum(fee) × 0.1% 最低 0.1 元 | 链下资金交易 |
| 40 | `propose_l2_fee_rate(bank, new_rate)` | 1 元/次 | 付费调用 |
| 41 | `set_max_l2_fee_rate(new_max)`(Root) | 免费 | 治理执行 |

### 3.3 on_initialize

每块调用 `fee_config::activate_pending_rates(now)`,把到期费率搬到生效位置,发 `L2FeeRateActivated` 事件。

### 3.4 权重入口

2026-04-29 补齐:清算行 pallet 已从裸 `T::DbWeight` 和空 `WeightInfo`
迁移为统一权重入口:

- `runtime/src/configs/mod.rs`:生产 runtime 使用
  `offchain_transaction::weights::SubstrateWeight<Runtime>`
- `weights.rs`:为 `bind_clearing_bank` / `deposit` / `withdraw` /
  `switch_bank` / `submit_offchain_batch_v2(items)` / 费率治理 /
  清算行节点声明三类 Call 提供非零保守权重
- `benchmarks.rs`:保留正式 benchmark 入口。由于该 pallet 通过
  `SfidAccountQuery` 解耦 `duoqian-manage`,正式自动生成权重需要在完整
  runtime + benchmarking runtime api 的 WASM 下构造机构、管理员和清算行节点
  fixture 后执行

当前权重不是自动 benchmark 产物,但已经替换掉空权重占位,并覆盖
`submit_offchain_batch_v2` 的按 item 线性增长。

## 4. 与 Step 1 的兼容关系

| Step 1 结构 | Step 2a 状态 |
|---|---|
| `OffchainBatchItem`(旧) | 保留,`submit_offchain_batch`(call_index 0)继续走旧 `execute_batch` |
| `RecipientClearingInstitution`(旧绑省储行) | 保留,Step 2d 删 |
| `InstitutionRateBp`(旧省储行费率) | 保留,Step 2d 删 |
| `bind_clearing_institution`(call_index 9) | 保留,Step 2d 删 |
| `bind_clearing_bank` / `deposit` / `withdraw` / `switch_bank`(30~33) | 保留,依然工作 |
| `UserBank` / `DepositBalance` / `BankTotalDeposits` / `L3PaymentNonce` | 保留,被 V2 清算路径正式使用 |

2026-04-28 补齐:V2 settlement 现在显式要求 `UserBank[payer] == item.payer_bank`
且 `UserBank[recipient] == item.recipient_bank`,防止移动端或节点绕过 UI 构造出绑定漂移的批次。

## 5. 编译验证

```
$ cargo check -p offchain-transaction
   Checking offchain-transaction v1.0.0
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.69s
```

零 warning 零 error。

## 6. 后续 Step 2b / 2c / 2d 清单

**Step 2b · Node**:
- `offchain/settlement/packer.rs::pack_and_submit` 补实现:取 ledger pending → 组 `OffchainBatchItemV2` → 多签 → 调 `submit_offchain_batch_v2`
- `offchain/ledger.rs.accept_payment` 完整实现(签名验证 + 本地扣款 + 加入 pending)
- `offchain/gossip.rs` 新建:清算行间 libp2p 协议推送 `{intent, a_sig, sender_ack}`
- `offchain/rpc.rs` 增补 `offchain_submitPayment` + WS 订阅
- `service.rs` / `rpc.rs` 按节点角色启动 + 注册 RPC namespace
- 删除旧 `offchain_ledger.rs` / `offchain_packer.rs` / `offchain_gossip.rs`(Step 2b 完成时)

**Step 2c · wuminapp**:
- 重写 `trade/offchain/offchain_pay_page.dart`:走清算行节点 `offchain_submitPayment`,每笔 L3 签名
- 改造 `wallet/ui/receive_qr_page.dart`:`body.bank` 填清算行主账户 SS58
- 冷钱包 QR 签名接入绑定/充值/提现(沿用旧 `bind_clearing_page.dart` 模式)
- 删除 `wallet/ui/bind_clearing_page.dart` + `trade/offchain/clearing_banks.dart` + `rpc/offchain.dart` + `rpc/onchain.dart.bindClearingInstitution`

**Step 2d · 清理**:
- 删除 pallet 内旧 `submit_offchain_batch` / `enqueue_offchain_batch` / `process_queued_batch`(走省储行分账的老路径)
- 删除 `RecipientClearingInstitution` / `InstitutionRateBp` / `bind_clearing_institution` / `propose_institution_rate` / `vote_institution_rate`
- 清理 Event / Error 中的旧变体
- 删除现有 `pallet::execute_batch` 与 `validate_batch_items` 等辅助函数
- 运行完整单元/集成测试

**Step 2b `set_max_l2_fee_rate` 联合投票**:
把本步的 `ensure_root(origin)` 入口改为接 `voting-engine` 联合投票 pallet 的 `JointVoteEngine::execute_if_passed`,让提案通过后再由投票引擎回调本 pallet 的内部执行函数。

## 7. 风险与验证后续

- **BlockLength 5 MB → 16 MB 升级**:Step 2b 做 runtime setCode 前必须升
- **批量 sr25519 验签**:当前 settlement 逐笔 `sr25519_verify`,10 万笔需要 ~5s,Step 2b 前切到 `sp_io::crypto::sr25519_batch_verify`
- **解码 H256 → T::Hash**:本步用 `T::Hash::decode` 跨类型兼容,依赖 runtime 中 `T::Hash == H256`(frame_system 默认)。若将来改 hasher 要同步改

## 8. 变更记录

- 2026-04-19:Step 2a 落地,Runtime 新增 3 子模块 + V2 结构 + 3 Storage + 3 Call + hook 扩展,零编译错误。
- 2026-04-28:批次级安全补齐:新增 `LastClearingBatchSeq`,严格校验
  `batch_signature` / `batch_seq`,settlement 增加 `UserBank` 绑定一致性校验;
  runtime `spec_version` 3 → 4,`transaction_version` 保持 2;单测增至 23 个并通过。
- 2026-04-29:权重收口:新增 `SubstrateWeight<Runtime>` 生产配置,所有清算行
  Call 改走 `T::WeightInfo`,不再使用空 `WeightInfo` 占位;`cargo test -p
  offchain-transaction --lib` 23 个测试通过。
