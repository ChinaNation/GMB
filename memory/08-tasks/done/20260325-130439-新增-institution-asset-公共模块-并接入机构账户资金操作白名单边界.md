# 任务卡：新增 institution-asset 公共模块，并接入机构账户资金操作白名单边界

- 任务编号：20260325-130439
- 状态：open
- 所属模块：citizenchain-runtime-transaction
- 当前负责人：Codex
- 创建时间：2026-03-25 13:04:39

## 任务需求

新增 institution-asset 公共模块，并接入机构账户资金操作白名单边界

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/transaction/README.md
- memory/05-modules/citizenchain/runtime/transaction/duoqian-manage/DUOQIAN_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/transaction/offchain-transaction/STEP2A_RUNTIME.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已新增 `citizenchain/runtime/transaction/institution-asset` 公共 crate
- 已将 `duoqian-manage` 接入关闭流程资金白名单检查
- 已将 `duoqian-transfer` 接入提案预检与执行阶段资金白名单检查
- 已将 `offchain-transaction` 接入批次 payer 与 fee sweep 的资金白名单检查
- 已在 `citizenchain/runtime/src/configs/mod.rs` 落地 runtime 规则：
  - `stake_address` 一律拒绝
  - 制度保留 `main_address` 仅允许 `DuoqianTransferExecute` / `DuoqianCloseExecute`
  - 制度 `fee_account` 仅允许 `OffchainFeeSweepExecute`
- 已新增交易模块技术文档：
  - `memory/05-modules/citizenchain/runtime/transaction/institution-asset/INSTITUTION_ASSET_TECHNICAL.md`
- 已更新现有交易模块文档与目录说明
- 验证通过：
  - `cargo test -p institution-asset`
  - `cargo test -p duoqian-manage`
  - `cargo test -p duoqian-transfer`
  - `cargo test -p offchain-transaction`
  - `cargo check -p citizenchain`
  - `git diff --check -- citizenchain memory/05-modules/citizenchain/runtime/transaction`
