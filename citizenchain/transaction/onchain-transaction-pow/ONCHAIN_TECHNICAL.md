# ONCHAIN Transaction Pow Technical Notes

## 1. 模块定位
`onchain-transaction-pow` 是运行时手续费适配模块（crate），不是 FRAME pallet。  
它提供两类核心能力：
- `PowOnchainChargeAdapter`：实现 `OnChargeTransaction`，负责“按金额收费 + 扣费”。
- `PowOnchainFeeRouter`：实现 `OnUnbalanced`，负责“已扣手续费的分账路由”。

代码位置：
- `/Users/rhett/GMB/citizenchain/transaction/onchain-transaction-pow/src/lib.rs`

---

## 2. 编译期常量约束
模块顶部使用 `const` 断言，确保关键常量合法：
- `ONCHAIN_FEE_FULLNODE_PERCENT + ONCHAIN_FEE_NRC_PERCENT + ONCHAIN_FEE_BLACKHOLE_PERCENT > 0`
- `ONCHAIN_MIN_FEE > 0`
- `ONCHAIN_FEE_RATE.deconstruct() > 0`

若约束不满足，编译阶段失败，避免链上运行后才暴露错误。

---

## 3. 费用计算模型
入口函数：`custom_fee_with_tip(...)`

计算规则：
1. 由 `CallAmount` 抽象提取交易金额：
   - `Amount(v)`：按金额收费
   - `NoAmount`：仅收 tip
   - `Unknown`：拒绝交易（`InvalidTransaction::Call`）
2. 按费率计算基础费：`amount * ONCHAIN_FEE_RATE`，使用四舍五入（`mul_perbill_round`）。
3. 应用最低费：`max(by_rate, ONCHAIN_MIN_FEE)`。
4. 最终费用：`base_fee + tip`。

---

## 4. 扣费流程（OnChargeTransaction）
`PowOnchainChargeAdapter` 关键行为：

- `can_withdraw_fee`：预检查付款账户是否可扣费。
- `withdraw_fee`：
  - 先计算 `fee_with_tip`
  - 可选通过 `CallFeePayer` 指定代付账户
  - 余额扣除后把 credit 拆为 `(inclusion_fee, tip_credit)`
- `correct_and_deposit_fee`：
  - 设计上不做执行后退款（`_corrected_fee_with_tip` 被有意忽略）
  - 将 `fee_credit + tip_credit` 一并交给 Router 分配

---

## 5. 分账路由（OnUnbalanced）
`PowOnchainFeeRouter::on_nonzero_unbalanced`：

分账比例（来自 `primitives::core_const`）：
- 全节点分成：`ONCHAIN_FEE_FULLNODE_PERCENT`
- 国储会分成：`ONCHAIN_FEE_NRC_PERCENT`
- 黑洞销毁：`ONCHAIN_FEE_BLACKHOLE_PERCENT`

处理顺序：
1. 先按比例把总手续费拆成：
   - `fullnode_credit`
   - `nrc_credit`
   - `blackhole_credit`
2. 全节点分成：
   - 通过 `FindAuthor` 找当前作者
   - 再查 `fullnode_pow_reward::RewardWalletByMiner`
   - 若可用，`Currency::resolve` 入账；resolve 失败会告警并销毁剩余 credit
   - 作者缺失/未绑定钱包则告警并销毁
3. NRC 分成：
   - 通过 `NrcAccountProvider::nrc_account()` 获取收款账户
   - `resolve` 失败告警并销毁
   - 账户缺失告警并销毁
4. 黑洞分成：直接 `drop(blackhole_credit)`，减少总发行量

---

## 6. 失败与资金安全语义
模块对失败路径采用“可观测 + 安全退化（销毁）”策略：
- `resolve` 失败：记录 `log::warn!`，并销毁未分配 credit
- 作者不存在、钱包未绑定、NRC 账户缺失：记录 `log::warn!` 并销毁对应份额
- 常量异常（理论上被编译期断言阻止）：运行时保底分支会避免错误分配

该设计保证不会把手续费错误分配到未知账户。

---

## 7. 扩展点（Runtime 注入）
模块通过 trait 注入业务差异：
- `CallAmount`：交易金额提取策略
- `CallFeePayer`：可选代付策略
- `NrcAccountProvider`：NRC 收款账户来源
- `FindAuthor`：块作者识别策略

这使模块可在不同 runtime 配置下复用。

---

## 8. 测试覆盖（当前）
当前单测覆盖 14 项，包含：
- 费率四舍五入与最低费
- `Amount/NoAmount/Unknown` 三类金额提取行为
- 默认扣费与自定义代付
- 余额不足失败路径
- Router 正常分配
- 作者未绑定/作者缺失路径
- NRC 账户缺失路径
- `correct_and_deposit_fee` 不退款语义
- tip 与 fee 合并后按同一比例分配

执行命令：
- `cargo test -p onchain-transaction-fee`

---

## 9. 运维排障建议
关注日志目标：`runtime::onchain_transaction_pow`

常见告警含义：
- `burn fullnode fee share: block author not found`
- `burn fullnode fee share: author found but reward wallet not bound`
- `burn fullnode fee share: failed to resolve reward wallet credit`
- `burn nrc fee share: nrc account decode failed`
- `burn nrc fee share: failed to resolve nrc account credit`

出现上述告警时，优先检查：
1. 出块作者识别链路（digest / `FindAuthor`）
2. `RewardWalletByMiner` 映射是否完整
3. NRC 账户配置是否正确
