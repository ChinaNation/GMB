# 任务卡：检查 onchain-transaction fee 修正分账与 FeePaid tip 文档假设是否存在 L1-L4 问题

- 任务编号：20260501-091353
- 状态：done
- 所属模块：citizenchain/runtime/transaction/onchain-transaction
- 当前负责人：Codex
- 创建时间：2026-05-01 09:13:53

## 任务需求

检查 onchain-transaction fee 修正分账与 FeePaid tip 文档假设是否存在 L1-L4 问题

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

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
- 2026-05-01 只读核查结论：
  - L1 部分成立：`correct_and_deposit_fee` 确实有意忽略 `_corrected_fee_with_tip`，代码已有中文注释说明“不做执行后退款”，但没有 `debug_assert!` 或醒目的 `PROTOCOL` 标识来约束未来改动。
  - L2 成立：`total_percent != EXPECTED_FEE_PERCENT_TOTAL` 的运行时保底分支直接 `return`，没有 `log::error!`；虽然顶部已有编译期 `assert!`，但保底分支一旦触发不可观测。
  - L3 成立：安全基金账户每次分账时通过 `T::AccountId::decode(&NRC_ANQUAN_ADDRESS[..])` 解码常量地址，位于手续费 Router 热路径。
  - L4 部分成立：`FeePaid.fee` 的确不含 tip，`lib.rs` 顶部也没有醒目 NOTE 链接文档 §11；但当前 `node/src/core/rpc.rs::fee_blockFees` 已同时累加 `FeePaid.fee` 和 `TransactionFeePaid.tip`，因此“RPC 仍假设 FeePaid.fee 等于实扣总额”在当前代码中不成立。
  - 文档差异：`ONCHAIN_TECHNICAL.md` §11 仍写 `fee_blockFees` 只累加 `FeePaid.fee` 并假定它等于总额；`NODE_TECHNICAL.md` 与实际代码已记录/实现 tip 累加，说明 runtime 技术文档 §11 需要同步更新。
  - 本轮仅做只读检查和任务卡记录，未修改业务代码，未执行测试。
