# 任务卡：修复 onchain-transaction L1 L2 L4：明确 no refund 协议、fee percent 保底日志、FeePaid tip 源码说明

- 任务编号：20260501-091854
- 状态：done
- 所属模块：citizenchain/runtime/transaction/onchain-transaction
- 当前负责人：Codex
- 创建时间：2026-05-01 09:18:54

## 任务需求

修复 onchain-transaction L1 L2 L4：明确 no refund 协议、fee percent 保底日志、FeePaid tip 源码说明

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md

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
- 已在 `FeePaid` 事件旁补充 NOTE，明确 `fee` 不含 tip，并链回技术文档第 11 节及下游 RPC / dashboard 统计口径。
- 已在 `correct_and_deposit_fee` 增加 `PROTOCOL: no post-dispatch refund` 注释块，固定协议上不做执行后退款的语义。
- 已在手续费比例运行时保底分支增加 `runtime::onchain_transaction` 目标 `log::error!`，异常时可观测。
- 已更新 `ONCHAIN_TECHNICAL.md`：补充 no-refund 源码约束、比例异常日志语义，并修正 `fee_blockFees` 当前按 `FeePaid.fee + TransactionFeePaid.tip` 聚合的说明。
- L3 安全基金账户重复 decode 暂未处理，按用户要求留到下一步。

## 验证记录

- `rustfmt citizenchain/runtime/transaction/onchain-transaction/src/lib.rs`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml --features runtime-benchmarks`
