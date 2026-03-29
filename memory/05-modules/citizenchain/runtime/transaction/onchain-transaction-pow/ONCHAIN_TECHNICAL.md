# ONCHAIN Transaction Pow Technical Notes

## 0. 功能需求
### 0.1 核心职责
`onchain-transaction-pow` 的功能需求是：
- 对链上 PoW 交易按“交易金额 + 最低费”规则收取手续费。
- 支持按制度规则把手续费分配给全节点、国储会与黑洞销毁。
- 支持代付账户与交易金额提取策略由 runtime 注入。

### 0.2 功能边界
- 本模块是运行时手续费适配 crate，不是独立 pallet，不维护自己的 storage。
- 本模块不负责识别“哪些交易属于 PoW 交易”，该判断由 runtime 在 `CallAmount` / `CallFeePayer` 中注入。
- 本模块不负责默认 weight fee、length fee、动态费率调整等 `pallet-transaction-payment` 常规策略，而是覆盖为“按业务金额收费”的制度模型。
- 本模块不负责执行后退款；一旦扣费成功，最终只做分账与销毁，不回退已扣手续费。

### 0.3 计费规则需求
- 当 `CallAmount` 返回 `Amount(v)` 时，基础手续费必须按 `v * ONCHAIN_FEE_RATE` 计算，并按“分”四舍五入。
- 当按比例计算结果低于 `ONCHAIN_MIN_FEE` 时，必须提升到最低费。
- 当 `CallAmount` 返回 `NoAmount` 时，仅允许收取 tip，不收基础手续费。
- 当 `CallAmount` 返回 `Unknown` 时，必须拒绝交易，避免制度内应收费交易被漏收。
- 最终扣费金额必须等于 `base_fee + tip`，且 `can_withdraw_fee` 与 `withdraw_fee` 使用完全一致的计算口径。

### 0.4 分账规则需求
- 已扣手续费必须按常量 `ONCHAIN_FEE_FULLNODE_PERCENT`、`ONCHAIN_FEE_NRC_PERCENT`、`ONCHAIN_FEE_BLACKHOLE_PERCENT` 分配。
- 三项分账比例必须固定总和为 `100`，避免“名为百分比、实为任意权重”的语义漂移。
- 全节点分成必须仅发给“当前区块作者绑定的钱包地址”；若作者不存在或未绑定钱包，该份额必须销毁。
- 国储会分成必须仅发给 `NrcAccountProvider` 提供的账户；若账户缺失或无法入账，该份额必须销毁。
- 黑洞分成必须直接销毁，不得转入任何中间地址或保留地址。
- tip 与基础手续费必须走同一条 Router 分账路径，避免出现两套分账口径。

### 0.5 资金安全需求
- 任何分账失败都不能把手续费错误打给未知账户。
- 作者缺失、奖励钱包未绑定、NRC 账户缺失等异常场景必须安全退化为销毁并留下日志。
- 协议明确不做执行后退款，`correct_and_deposit_fee` 只负责最终分账。

### 0.6 可配置与可扩展需求
- Runtime 必须可以替换交易金额提取逻辑（`CallAmount`）。
- Runtime 必须可以替换代付账户判定逻辑（`CallFeePayer`）。
- Runtime 必须可以替换 NRC 收款账户来源（`NrcAccountProvider`）。
- Runtime 必须可以替换块作者识别逻辑（`FindAuthor`）。

### 0.7 可观测性需求
- 对作者缺失、钱包未绑定、NRC 账户缺失、resolve 失败等异常场景，模块必须输出 `runtime::onchain_transaction_pow` 目标日志。
- 日志必须能够区分“全节点份额销毁”和“NRC 份额销毁”的原因，便于运维排障。

### 0.8 Benchmark 需求
- 本模块不是 FRAME pallet，因此不进入 runtime 的 `define_benchmarks!` 注册表。
- 需要提供专项 benchmark，覆盖：
  - 扣费热路径（计算费用 + 扣费 + 分账）
  - Router 分账热路径

---

## 1. 模块定位
`onchain-transaction-pow` 是运行时手续费适配模块（crate），不是 FRAME pallet。  
它提供两类核心能力：
- `PowOnchainChargeAdapter`：实现 `OnChargeTransaction`，负责“按金额收费 + 扣费”。
- `PowOnchainFeeRouter`：实现 `OnUnbalanced`，负责“已扣手续费的分账路由”。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/transaction/onchain-transaction-pow/src/lib.rs`

---

## 2. 编译期常量约束
模块顶部使用 `const` 断言，确保关键常量合法：
- `ONCHAIN_FEE_FULLNODE_PERCENT + ONCHAIN_FEE_NRC_PERCENT + ONCHAIN_FEE_BLACKHOLE_PERCENT == 100`
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

`transfer_all` 特殊说明：
`PowTxAmountExtractor` 对 `transfer_all` 按扣费前的 `reducible_balance` 提取金额。这是有意设计——按用户"转出全部"的意图金额收费，实际转出额 = 可用余额 - 手续费。如果改为按扣费后金额收费会产生循环依赖（手续费取决于转出额，转出额取决于手续费）。

---

## 4. 扣费流程（OnChargeTransaction）
`PowOnchainChargeAdapter` 关键行为：

- `can_withdraw_fee`：预检查付款账户是否可扣费。
- `withdraw_fee`：
  - 先计算 `fee_with_tip`
  - 可选通过 `CallFeePayer` 指定代付账户
  - 余额扣除后把 credit 拆为 `(inclusion_fee, tip_credit)`
  - 发出 `FeePaid { who, fee }` 事件，其中 `fee` 只包含基础手续费（不含 tip）
- `correct_and_deposit_fee`：
  - 设计上不做执行后退款（`_corrected_fee_with_tip` 被有意忽略）
  - 将 `fee_credit` 与 `tip_credit` 一并交给 Router，由 `on_unbalanceds` 合并后按同一口径统一分账

---

## 5. 分账路由（OnUnbalanced）
`PowOnchainFeeRouter::on_nonzero_unbalanced`：

分账比例（来自 `primitives::core_const`）：
- 全节点分成：`ONCHAIN_FEE_FULLNODE_PERCENT`
- 国储会分成：`ONCHAIN_FEE_NRC_PERCENT`
- 黑洞销毁：`ONCHAIN_FEE_BLACKHOLE_PERCENT`

处理顺序：
1. 先按 `fullnode : (nrc + blackhole)` 把总手续费拆成两部分：
   - `fullnode_credit`
   - `remainder`
2. 再把 `remainder` 按 `nrc : blackhole` 二次拆分成：
   - `nrc_credit`
   - `blackhole_credit`
3. 全节点分成：
   - 通过 `FindAuthor` 找当前作者
   - 再查 `fullnode_pow_reward::RewardWalletByMiner`
   - 若可用，`Currency::resolve` 入账；resolve 失败会告警并销毁剩余 credit
   - 作者缺失/未绑定钱包则告警并销毁
4. NRC 分成：
   - 通过 `NrcAccountProvider::nrc_account()` 获取收款账户
   - `resolve` 失败告警并销毁
   - 账户缺失告警并销毁
5. 黑洞分成：直接 `drop(blackhole_credit)`，减少总发行量

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

## 8. 专项 Benchmark
- 本模块新增独立 benchmark harness：`benches/transaction_fee_paths.rs`
- 基准用例：
  - `onchain_fee_charge_transaction_amount_path`
  - `onchain_fee_router_distribution_success`
- 执行命令：
  - `cargo bench -p onchain-transaction-pow --bench transaction_fee_paths`
- 说明：
  - 这里不是标准 pallet benchmark，而是针对交易扣费与分账热路径的专项性能验证。

---

## 9. 测试覆盖（当前）
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
- `cargo test -p onchain-transaction-pow`

---

## 10. 运维排障建议
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

---

## 11. FeePaid 事件与外部依赖

### 事件语义
`FeePaid { who, fee }` 中的 `fee` 只记录基础手续费（`base_fee = fee_with_tip - tip`），不包含 tip。当前 PoW 链无 tip UI 入口，tip 实际值始终为 0，因此 `fee` 等同于真实手续费。若未来引入 tip 机制，需同步更新下游消费方。

### 外部依赖方
- **node RPC `fee_blockFees`**（`node/src/rpc.rs`）：累加指定区块内所有 `FeePaid.fee`，返回该区块的手续费总额。
- **nodeui mining-dashboard**（`nodeui/backend/src/mining/mining-dashboard/mod.rs`）：读取 `fee_blockFees` 统计矿工收益。
- **runtime 注册**（`runtime/src/lib.rs` pallet_index 4）：`OnchainTransactionPow` 发出事件。

### 注意
上述依赖方均假定 `FeePaid.fee` 等于实际扣费总额。当 tip > 0 时该假设不成立，统计会少算 tip 部分。
3. NRC 账户配置是否正确
