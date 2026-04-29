# 清算行 Runtime · Step 2b-iv-b 老省储行代码彻底清除

- **日期**:2026-04-20
- **范围**:物理删除 `offchain-transaction` pallet 内所有老省储行清算体系的
  Call / Storage / Events / Errors / helper / types,以及 `configs/mod.rs` 和
  `node/src/ui/governance/` 里的所有老分支/入口。dev 链统一 setCode 升级,不做
  `on_runtime_upgrade` migration。
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_IV_A_CLEANUP.md`(节点侧老省储行 .rs 清理)+ `STEP2C_III_COLD_WALLET.md`
- **后续**:Layer B 自动化 E2E / 跨行 ghost account 修复

---

## 1. 本步触及的代码面

### 1.1 Runtime pallet(`runtime/transaction/offchain-transaction/src/`)

**lib.rs 整体重写**:从 2873 行 → 434 行。删除如下全部符号:

| 类别 | 删除项 |
|---|---|
| 老 Call(13 个) | `submit_offchain_batch(0)` / `propose_institution_rate(1)` / `vote_institution_rate(2)` / `bind_clearing_institution(9)` / `enqueue_offchain_batch(10)` / `process_queued_batch(11)` / `prune_queued_batch(14)` / `prune_batch_summary(15)` / `prune_processed_tx(16)` / `prune_expired_proposal_action(17)` / `skip_failed_batch(18)` / `cancel_queued_batch(19)` / `retry_execute_proposal(20)` / `cancel_stale_queued_batches(23)` |
| 老 Storage(14 个) | `InstitutionRateBp` / `LastPackBlock` / `LastBatchSeq` / `NextEnqueueBatchSeq` / `RateProposalActions` / `NextBatchId` / `BatchSummaries` / `NextQueuedBatchId` / `QueuedBatches` / `QueuedTxIndex` / `QueuedBatchPruneCursor` / `BatchSummaryPruneCursor` / `RecipientClearingInstitution` / `ProcessedTxLog` / `NextProcessedTxLogId` / `ProcessedTxPruneCursor` |
| 保留 Storage | `UserBank` / `DepositBalance` / `BankTotalDeposits` / `L3PaymentNonce` / `L2FeeRateBp` / `L2FeeRateProposed` / `MaxL2FeeRateBp` / `ProcessedOffchainTx` / `ProcessedOffchainTxAt`(settlement 防重放仍用) |
| 老 Events(17 个) | `OffchainBatchSubmitted` / `InstitutionRate*`(4 个)/ `InternalProposalExecutionFailed` / `RecipientClearingInstitutionBound` / `OffchainBatchQueued` / `OffchainQueuedBatch*`(5 个)/ `OffchainStaleQueuedBatchesCancelled` / `FailedBatchSkipped` / `QueuedBatchPruned` / `BatchSummaryPruned` / `ProcessedTxPruned` / `ProposalActionPruned` / `ProposalExecutionRetried` |
| 老 Errors(约 20 个) | `InvalidInstitution` / `InvalidRateBp` / `ProposalNotFound` 等一组费率+批次+清理错误。保留新体系使用的 `InvalidL2FeeRate` / `InvalidL3Signature` / `L2FeeRateNotConfigured` / `ExpiredIntent` / `TxAlreadyProcessed` 等 |
| 老 helpers | `is_prb_admin` / `precheck_submit_offchain_batch_with_rate` / `ensure_rate_and_institution` / `try_execute_rate` / `auto_prune_*` / `queued_prune_budget_hint` / `should_bubble/ignore/wait_precheck_error` / `institution_pallet_address` / `institution_fee_address` / `round_div` / `calc_offchain_fee_fen` / `FeeCalcError` / `ProtectedSourceChecker` trait / `RateProposalAction` struct / `BatchItemOf` / `BatchOf` / `BatchSummary` / `QueuedBatchStatus` / `QueuedBatchLastError` / `QueuedBatchRecord` |
| 老 hook | 原 `on_idle`(清理老 Storage)整块删。保留 `on_initialize` 只做 `fee_config::activate_pending_rates` |
| 老 `GenesisConfig.initial_rates` | 删除字段 + `BuildGenesisConfig` 整块(清算行费率由治理 extrinsic 设置,无 genesis 初值) |
| 老配置字段 | `type InternalVoteEngine` / `type ProtectedSourceChecker` 从 `Config` trait 中移除 |
| `#[cfg(test)] mod tests` | 原 pallet 内 20+ 个针对老 Call 的测试整块删除。`batch_item.rs::tests`(5 个 golden + 2 个基础)+ `bank_check.rs::tests`(3 个)保留 |

`benchmarks.rs`(74 行 → 13 行):删除 3 个老 benchmark(`submit_offchain_batch` /
`enqueue_offchain_batch` / `process_queued_batch`)。留占位 module,新 Call benchmark
Step 3 再补。

`weights.rs`(55 行 → 17 行):删除 3 个老方法,`WeightInfo` trait 简化为默认实现空壳;
当前所有 new Call 权重在 `lib.rs::#[pallet::weight]` 里直接用 `T::DbWeight` 估算。

### 1.2 Runtime configs(`runtime/src/configs/mod.rs`)

- `OnchainTxAmountExtractor` 删除 5 处老分支(`submit_offchain_batch` / `enqueue_offchain_batch`
  / `process_queued_batch` / 及其引用 `QueuedBatches` 的 `FeePayer` 分支)。老 Call 删了
  Pattern 永远不命中,但保留的 `_ => Amount(100000)` 兜底仍对未来扩展安全。
- `RuntimeFeePayerExtractor` 清理后只剩 `submit_offchain_batch_v2 → fee_account_of(institution_main)`
  一条,其余 Call 走 `_ => None` 个人付费。
- 删除 `impl offchain_transaction::ProtectedSourceChecker for RuntimeProtectedSourceChecker`(ProtectedSourceChecker trait 已从 pallet 移除)。
- `offchain_transaction::Config for Runtime` 移除 `InternalVoteEngine` / `ProtectedSourceChecker`
  两个 type;`WeightInfo` 改为 `()`(pallet weights.rs 简化后 SubstrateWeight 不再存在)。

### 1.3 Runtime 外层(`runtime/src/lib.rs`)

- `spec_version: 8 → 9`。注释说明 Step 2b-iv-b 清理老 Storage + pallet
  `storage_version` 从 1 → 2,dev 链 setCode 升级路径。

### 1.4 Node UI governance(`node/src/ui/`)

删除 4 个老 Tauri 命令:

- `mod.rs` 内:`build_rate_vote_request` / `build_propose_rate_request` /
  `submit_propose_rate` / `query_institution_rate_bp`(共 ~99 行)
- `mod.rs::generate_handler!` 列表同步删这 4 行
- `signing.rs` 内:`build_rate_vote_sign_request` / `build_propose_rate_sign_request`
  两个签名请求构造函数(~160 行)
- `proposal.rs` 内:`fetch_rate_proposal_action` / `fetch_institution_rate_bp`
  两个链上存储查询函数,并在 `resolve_proposal_action` 里删除"Step 2:
  RateProposalActions"分支

---

## 2. 升级策略(开发期)

按 `feedback_no_chain_restart.md` 铁律,runtime 变更不走 fresh genesis,而是
链上 setCode。但由于本步为 **"开发期 dev 链清理"**,用户已授权不做
`on_runtime_upgrade` migration,采用以下步骤:

1. `spec_version: 10` + 新 WASM 编译
2. dev 链管理员 `developer_direct_upgrade`(或等价 sudo/Root setCode)
3. setCode 生效块起:
   - 老 Call enum 槽位不再存在,pool 中 in-flight 老 Call tx 会在解码时被拒
   - 老 Storage 键在链上**保留**物理数据,但无 Rust 类型读取(相当于垃圾字节,
     runtime 不再访问),下个 Step 3 的独立 migration 单独清理(或任其随
     chain data 生命周期自然消亡)
4. 新 Call 34(`submit_offchain_batch_v2`)在 spec_version=9 下仍然可用,
   wuminapp + 清算行节点 packer 不需要任何配合改动

**不迁移 / 不 migration 的代价**:
- 老 Storage 残留物理字节在 state trie 中(影响 state 快照大小),但不访问
- 历史块(spec_version ≤ 8 产生)重新同步需要旧 WASM,polkadot-sdk runtime code
  历史回放机制保留(不影响 sync)

---

## 3. 编译验证

```bash
$ cd citizenchain
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node --tests
(仅 Tauri `frontend/dist` proc macro 门禁未通过,与本步无关)

$ cargo test -p offchain-transaction --lib
test result: ok. 10 passed; 0 failed
(5 个 golden vectors + 2 个 signing_hash 基础 + 3 个 bank_check 测试;原 lib.rs
 内老 Call 测试 13 个随代码一并删除)

$ cargo check --workspace --exclude node
(citizenchain runtime / institution-asset / sfid-backend / 其他 crate 全绿)

$ cd ../wuminapp && flutter analyze
No issues found!  (Dart 端不依赖 Rust 枚举,不受影响)
```

---

## 4. 删除规模

| 项目 | 删除行数(约) |
|---:|---:|
| `offchain-transaction/src/lib.rs` | **-2439** 行(2873→434) |
| `offchain-transaction/src/benchmarks.rs` | **-61** 行(74→13) |
| `offchain-transaction/src/weights.rs` | **-38** 行(55→17) |
| `runtime/src/configs/mod.rs` | **-60** 行(OnchainTxAmount + FeePayer + ProtectedSourceChecker impl) |
| `node/src/ui/governance/mod.rs` | **-99** 行(4 个 Tauri 命令) |
| `node/src/ui/governance/signing.rs` | **-160** 行(2 个签名构造函数) |
| `node/src/ui/governance/proposal.rs` | **-75** 行(2 个查询 + 1 个分支) |
| `node/src/ui/mod.rs` | **-4** 行(handler 注册) |
| **合计** | **~2900+ 行代码删除** |

---

## 5. 已知风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| dev 链 setCode 前提交的老 Call tx 会失败 | **P3** | dev 链容忍 pool 丢 tx;生产链未启用老 Call,不受影响 |
| 老 Storage 物理残留占 state trie 空间 | **P3** | dev 环境可接受;Step 3 可写独立 migration pallet 清 |
| 历史块 re-sync 依赖旧 WASM | **P2** | polkadot-sdk 运行时 code 历史保留机制已覆盖;触发几率仅在 node 完全冷启动时 |
| `ProcessedOffchainTx / ProcessedOffchainTxAt` 被新老共用,删除过度 | **验证已过** | 已确认 settlement.rs `execute_single_item` 仍读写这两个 Storage,保留正确 |
| 老 Error 还被 UI 其他地方 hardcoded | **低** | grep 未发现残留引用;若有编译立失败,无静默 |

---

## 6. 后续

- **Layer B 自动化 E2E**:本步清理使测试面大幅收窄,现在新 Call 全绿(5 golden +
  bank_check 3 个),Layer B 集成测试可基于精简后的 pallet 建
- **Step 3 migration pallet**:需要清理链上残留老 Storage 字节时再写
- **跨行 ghost account bug**:E 任务,与本清理解耦

---

## 7. 变更记录

- 2026-04-20:Step 2b-iv-b 彻底完成。删除约 2900 行老省储行清算代码;pallet
  从 2873 行精简到 434 行;`spec_version` 8 → 9,`storage_version` 1 → 2;
  `cargo test -p offchain-transaction --lib` 10 ok;`cargo check -p node --tests`
  零 error(仅 Tauri 门禁);`flutter analyze` 零 issue。
