# 任务卡：修复 onchain-transaction N1/N2：补充 FeeShareBurnt 测试断言与 fee 乘法防御注释

- 任务编号：20260501-100019
- 状态：done
- 所属模块：citizenchain/runtime/transaction/onchain-transaction
- 当前负责人：Codex
- 创建时间：2026-05-01 10:00:19

## 任务需求

修复 N1/N2：为关键手续费销毁路径补充 `FeeShareBurnt` 事件断言，并提升 `mul_perbill_round` 中 `whole * parts` 的可读性与防御性。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/08-tasks/open/20260501-095643-检查-onchain-transaction-n1-n2-n3-事件断言-乘法可读性与-router-bound-收紧.md
- memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md

## 必须遵守

- 不改变手续费计算制度语义
- 不改变 80% 全节点 / 10% NRC / 10% 安全基金分账比例
- 不改 N3 的 `T: pallet::Config` bound
- 改代码后必须更新文档和清理残留

## 输出物

- 测试断言
- 中文注释
- 文档更新
- 残留清理
- 验证记录

## 实施记录

- 任务卡已创建
- 已新增 `fee_share_burn_event_count` 与 `fee_share_burn_event_total` 测试 helper。
- 已在 `correct_and_deposit_does_not_refund_overpayment` 中断言 `AuthorMissing` 与 `NrcMissing` 两类 `FeeShareBurnt` 各出现 1 次。
- 已在 `tip_is_routed_with_fee_using_same_distribution` 中断言成功分账路径不产生 `FeeShareBurnt`。
- 已将 `mul_perbill_round` 的整分量乘法改为 `saturating_mul`，并补充中文注释说明 `Perbill` 边界与防御性目的。
- 已更新 `ONCHAIN_TECHNICAL.md`，记录 fee 计算防御与新增测试不变量。

## 验证记录

- `rustfmt citizenchain/runtime/transaction/onchain-transaction/src/lib.rs`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml --features runtime-benchmarks`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml --all-targets`
