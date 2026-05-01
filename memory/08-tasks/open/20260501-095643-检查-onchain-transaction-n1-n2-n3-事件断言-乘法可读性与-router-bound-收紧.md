# 任务卡：检查 onchain-transaction N1/N2/N3：事件断言、乘法可读性与 router bound 收紧

- 任务编号：20260501-095643
- 状态：done
- 所属模块：citizenchain/runtime/transaction/onchain-transaction
- 当前负责人：Codex
- 创建时间：2026-05-01 09:56:43

## 任务需求

检查 `onchain-transaction` 是否存在 N1/N2/N3 三项问题，评估是否值得修复，并给出推荐修复方案。

## 检查项

- N1：`correct_and_deposit_does_not_refund_overpayment` 与 `tip_is_routed_with_fee_using_same_distribution` 是否缺少 `FeeShareBurnt` 事件断言。
- N2：`mul_perbill_round` 中 `whole * parts` 是否存在溢出风险或可读性问题。
- N3：`OnchainFeeRouter` where 子句新增 `T: pallet::Config` 是否合理。

## 输出物

- 检查结论
- 影响评估
- 推荐修复方案

## 实施记录

- 任务卡已创建
- N1 存在：`correct_and_deposit_does_not_refund_overpayment` 已验证销毁金额，但未显式断言 `AuthorMissing` 与 `NrcMissing` 两类 `FeeShareBurnt` 事件；`tip_is_routed_with_fee_using_same_distribution` 是成功分账路径，未断言无销毁事件。
- N2 存在：`mul_perbill_round` 中 `whole * parts` 按 `Perbill` 数学约束不会溢出，但源码可读性不足。
- N3 存在但合理：`OnchainFeeRouter` 需要 `pallet::Config` 才能在 `emit_fee_share_burn` 中 `deposit_event`，bound 收紧符合当前实现。
- 推荐修复：补 N1 测试断言；N2 改为 `saturating_mul` 或补充边界注释；N3 不需要修复。
