# ONCHAIN Transaction Technical Notes

## 0. 功能需求
### 0.1 核心职责
`onchain-transaction` 的功能需求是：
- 对链上交易按"交易金额 + 最低费"规则收取手续费。
- 支持按制度规则把手续费分配给全节点、国储会手续费账户与安全基金账户。
- 支持代付账户与交易金额提取策略由 runtime 注入。

### 0.2 功能边界
- 本模块是运行时手续费适配 crate，不是独立 pallet，不维护自己的 storage。
- 本模块不负责识别"哪些交易属于链上交易"，该判断由 runtime 在 `CallAmount` / `CallFeePayer` 中注入。
- 本模块不负责默认 weight fee、length fee、动态费率调整等 `pallet-transaction-payment` 常规策略，而是覆盖为"按业务金额收费"的制度模型。
- 本模块不负责执行后退款；一旦扣费成功，最终只做分账，不回退已扣手续费。

### 0.3 计费规则需求
- 当 `CallAmount` 返回 `Amount(v)` 时，基础手续费必须按 `v * ONCHAIN_FEE_RATE` 计算，并按"分"四舍五入。
- 当按比例计算结果低于 `ONCHAIN_MIN_FEE` 时，必须提升到最低费。
- 当 `CallAmount` 返回 `NoAmount` 时，仅允许收取 tip，不收基础手续费。
- 当 `CallAmount` 返回 `Unknown` 时，必须拒绝交易，避免制度内应收费交易被漏收。
- 最终扣费金额必须等于 `base_fee + tip`，且 `can_withdraw_fee` 与 `withdraw_fee` 使用完全一致的计算口径。

### 0.4 分账规则需求
- 已扣手续费必须按常量 `ONCHAIN_FEE_FULLNODE_PERCENT`、`ONCHAIN_FEE_NRC_PERCENT`、`ONCHAIN_FEE_SAFETY_FUND_PERCENT` 分配。
- 三项分账比例必须固定总和为 `100`，避免"名为百分比、实为任意权重"的语义漂移。
- 全节点分成（80%）必须仅发给"当前区块作者绑定的钱包地址"；若作者不存在或未绑定钱包，该份额自动销毁。
- 国储会手续费账户分成（10%）必须仅发给 `NrcAccountProvider` 提供的账户（`NRC_FEIYONG_ADDRESS`）；若账户缺失或无法入账，该份额自动销毁。
- 安全基金分成（10%）必须转入 `SafetyFundAccountProvider` 提供的安全基金账户（当前 runtime 映射到 `NRC_ANQUAN_ADDRESS`）。
- tip 与基础手续费必须走同一条 Router 分账路径，避免出现两套分账口径。

### 0.5 资金安全需求
- 任何分账失败都不能把手续费错误打给未知账户。
- 作者缺失、奖励钱包未绑定、NRC 手续费账户缺失等异常场景必须安全退化为销毁，并同时留下链上事件和日志。
- 协议明确不做执行后退款，`correct_and_deposit_fee` 只负责最终分账。

### 0.6 可配置与可扩展需求
- Runtime 必须可以替换交易金额提取逻辑（`CallAmount`）。
- Runtime 必须可以替换代付账户判定逻辑（`CallFeePayer`）。
- Runtime 必须可以替换 NRC 收款账户来源（`NrcAccountProvider`）。
- Runtime 必须可以替换安全基金账户来源（`SafetyFundAccountProvider`）。
- Runtime 必须可以替换块作者识别逻辑（`FindAuthor`）。

### 0.7 可观测性需求
- 对作者缺失、钱包未绑定、NRC 账户缺失、resolve 失败等异常场景，模块必须输出 `runtime::onchain_transaction` 目标日志。
- 对所有手续费份额销毁路径，模块必须发出 `FeeShareBurnt { reason, amount }` 链上事件。
- 日志必须能够区分"全节点份额销毁"和"NRC 份额销毁"的原因，便于运维排障。
- 链上事件必须能够被区块浏览器、RPC 聚合或运维任务直接统计，不能只依赖节点本地日志。

### 0.8 Benchmark 需求
- 本模块不是 FRAME pallet，因此不进入 runtime 的 `define_benchmarks!` 注册表。
- 需要提供专项 benchmark，覆盖：
  - 扣费热路径（计算费用 + 扣费 + 分账）
  - Router 分账热路径

---

## 1. 模块定位
`onchain-transaction` 是运行时手续费适配模块（crate），不是 FRAME pallet。
它提供两类核心能力：
- `OnchainChargeAdapter`：实现 `OnChargeTransaction`，负责"按金额收费 + 扣费"。
- `OnchainFeeRouter`：实现 `OnUnbalanced`，负责"已扣手续费的分账路由"。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/transaction/onchain-transaction/src/lib.rs`

---

## 2. 编译期常量约束
模块顶部使用 `const` 断言，确保关键常量合法：
- `ONCHAIN_FEE_FULLNODE_PERCENT + ONCHAIN_FEE_NRC_PERCENT + ONCHAIN_FEE_SAFETY_FUND_PERCENT == 100`
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
2. 将金额转为 `u128` 后调用统一公开函数 `calculate_onchain_fee(amount)`。
3. `calculate_onchain_fee` 内部按费率计算基础费：`amount * ONCHAIN_FEE_RATE`，使用四舍五入（`mul_perbill_round`）。
4. 应用最低费：`max(by_rate, ONCHAIN_MIN_FEE)`。
5. 最终费用：`base_fee + tip`。

公开复用入口：
- `calculate_onchain_fee(amount: u128) -> u128`
- 公式：`max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)`
- 返回值单位为"分"

说明：`custom_fee_with_tip` 与 `duoqian-*` 预扣逻辑共用 `calculate_onchain_fee`，避免 transaction-payment 实扣规则与业务 pallet 预扣规则漂移。`mul_perbill_round` 会先拆分整分量和尾量，避免直接执行 `amount * parts`；整分量乘法使用 `saturating_mul` 作为防御性保护，实际按 `Perbill` 约束不会改变结果。

`transfer_all` 特殊说明：
`OnchainTxAmountExtractor` 对 `transfer_all` 按扣费前的 `reducible_balance` 提取金额。这是有意设计——按用户"转出全部"的意图金额收费，实际转出额 = 可用余额 - 手续费。如果改为按扣费后金额收费会产生循环依赖（手续费取决于转出额，转出额取决于手续费）。

---

## 4. 扣费流程（OnChargeTransaction）
`OnchainChargeAdapter` 关键行为：

- `can_withdraw_fee`：预检查付款账户是否可扣费。
- `withdraw_fee`：
  - 先计算 `fee_with_tip`
  - 若 `fee_with_tip == 0`，直接返回 `Ok(None)`，不扣费也不发 `FeePaid`
  - 可选通过 `CallFeePayer` 指定代付账户
  - 余额扣除后把 credit 拆为 `(inclusion_fee, tip_credit)`
  - 发出 `FeePaid { who, fee }` 事件，其中 `fee` 只包含基础手续费（不含 tip）
- `correct_and_deposit_fee`：
  - 设计上不做执行后退款（`_corrected_fee_with_tip` 被有意忽略）
  - 源码使用 `PROTOCOL: no post-dispatch refund` 注释块固定该协议语义，避免未来把 corrected fee 误当作退款入口
  - 将 `fee_credit` 与 `tip_credit` 一并交给 Router，由 `on_unbalanceds` 合并后按同一口径统一分账
  - 当 `liquidity_info == None` 时是 no-op，直接返回 `Ok(())`

---

## 5. 分账路由（OnUnbalanced）
`OnchainFeeRouter::on_nonzero_unbalanced`：

分账比例（来自 `primitives::fee_policy`，2026-05-03 起单一权威源）：
- 全节点分成：`ONCHAIN_FEE_FULLNODE_PERCENT`（80%）
- 国储会手续费账户分成：`ONCHAIN_FEE_NRC_PERCENT`（10%）
- 安全基金账户分成：`ONCHAIN_FEE_SAFETY_FUND_PERCENT`（10%）

处理顺序：
1. 先按 `fullnode : (nrc + safety_fund)` 把总手续费拆成两部分：
   - `fullnode_credit`
   - `remainder`
2. 再把 `remainder` 按 `nrc : safety_fund` 二次拆分成：
   - `nrc_credit`
   - `safety_fund_credit`
3. 全节点分成：
   - 通过 `FindAuthor` 找当前作者
   - 再查 `fullnode_issuance::RewardWalletByMiner`
   - 若可用，`Currency::resolve` 入账；resolve 失败会告警、发 `FeeShareBurnt(FullnodeResolveFailed)` 并销毁剩余 credit
   - 作者缺失/未绑定钱包则告警、发 `FeeShareBurnt(AuthorMissing|WalletUnbound)` 并销毁
4. NRC 手续费账户分成：
   - 通过 `NrcAccountProvider::nrc_account()` 获取收款账户（`NRC_FEIYONG_ADDRESS`）
   - `resolve` 失败告警、发 `FeeShareBurnt(NrcResolveFailed)` 并销毁
   - 账户缺失告警、发 `FeeShareBurnt(NrcMissing)` 并销毁
5. 安全基金分成：
   - 通过 `SafetyFundAccountProvider::safety_fund_account()` 获取收款账户
   - 当前 runtime provider 返回 `NRC_ANQUAN_ADDRESS` 对应账户，避免 Router 在每笔分账热路径重复 decode 32 字节常量
   - `resolve` 失败告警、发 `FeeShareBurnt(SafetyFundResolveFailed)` 并销毁

---

## 6. 失败与资金安全语义
模块对失败路径采用"可观测 + 安全退化（销毁）"策略：
- `resolve` 失败：记录 `log::warn!`，发 `FeeShareBurnt`，并销毁未分配 credit
- 作者不存在、钱包未绑定、NRC 账户缺失：记录 `log::warn!`，发 `FeeShareBurnt`，并销毁对应份额
- 常量异常（理论上被编译期断言阻止）：运行时保底分支会记录 `log::error!`，并避免错误分配

该设计保证不会把手续费错误分配到未知账户。

`BurnReason` 当前取值：
- `AuthorMissing`
- `WalletUnbound`
- `FullnodeResolveFailed`
- `NrcMissing`
- `NrcResolveFailed`
- `SafetyFundResolveFailed`

---

## 7. 扩展点（Runtime 注入）
模块通过 trait 注入业务差异：
- `CallAmount`：交易金额提取策略
- `CallFeePayer`：可选代付策略
- `NrcAccountProvider`：NRC 手续费账户来源
- `SafetyFundAccountProvider`：安全基金账户来源
- `FindAuthor`：块作者识别策略

这使模块可在不同 runtime 配置下复用。

当前 CitizenChain runtime 的治理类金额提取规则中，`VotingEngine::internal_vote(proposal_id, approve)` 返回 `Amount(100_000)`，按固定 1 元/次作为管理员主动投票操作计费。若该票触发内部治理提案达阈值，后续 executor 自动回调仍属于同一笔 extrinsic 的执行路径，不再单独产生第二笔 `CallAmount` 提取。

---

## 8. 专项 Benchmark
- 本模块新增独立 benchmark harness：`benches/transaction_fee_paths.rs`
- 基准用例：
  - `onchain_fee_charge_transaction_amount_path`
  - `onchain_fee_router_distribution_success`
- 执行命令：
  - `cargo bench -p onchain-transaction --bench transaction_fee_paths`
- 说明：
  - 这里不是标准 pallet benchmark，而是针对交易扣费与分账热路径的专项性能验证。

---

## 9. 测试覆盖（当前）
当前单测覆盖 20 项，包含：
- 费率四舍五入与最低费
- `Amount/NoAmount/Unknown` 三类金额提取行为
- 默认扣费与自定义代付
- `NoAmount && tip == 0` 零费用短路，不扣费且不发 `FeePaid`
- 余额不足失败路径
- Router 正常分配（全节点 + NRC 手续费账户 + 安全基金）
- 作者未绑定/作者缺失路径及对应 `FeeShareBurnt` 事件
- 全节点奖励钱包 resolve 失败路径及对应 `FeeShareBurnt` 事件
- NRC 账户缺失、NRC resolve 失败路径及对应 `FeeShareBurnt` 事件
- 安全基金 resolve 失败路径及对应 `FeeShareBurnt` 事件
- `correct_and_deposit_fee` 不退款语义和 `liquidity_info=None` no-op 语义；不退款测试同时断言 `AuthorMissing` / `NrcMissing` 两类销毁事件
- tip 与 fee 合并后按同一比例分配；成功分账路径断言不产生 `FeeShareBurnt`

说明：安全基金账户现在由 `SafetyFundAccountProvider` 注入，Router 热路径不再执行 `NRC_ANQUAN_ADDRESS` decode，也不再保留 decode 失败事件分支。

执行命令：
- `cargo test -p onchain-transaction`

---

## 10. 运维排障建议
关注日志目标：`runtime::onchain_transaction`

常见告警含义：
常见日志与 `FeeShareBurnt.reason` 含义：
- `AuthorMissing`：区块作者未找到，全节点份额销毁
- `WalletUnbound`：作者已找到但未绑定奖励钱包，全节点份额销毁
- `FullnodeResolveFailed`：奖励钱包入账失败，全节点份额销毁
- `NrcMissing`：国储会手续费账户未配置，NRC 份额销毁
- `NrcResolveFailed`：国储会手续费账户入账失败，NRC 份额销毁
- `SafetyFundResolveFailed`：安全基金账户入账失败，安全基金份额销毁

出现上述告警时，优先检查：
1. 出块作者识别链路（digest / `FindAuthor`）
2. `RewardWalletByMiner` 映射是否完整
3. NRC 手续费账户配置（`NrcAccountProvider`）是否正确
4. 安全基金账户配置（`SafetyFundAccountProvider`）是否正确

---

## 11. FeePaid 事件与外部依赖

### 事件语义
`FeePaid { who, fee }` 中的 `fee` 只记录基础手续费（`base_fee = fee_with_tip - tip`），不包含 tip。当前 PoW 链无 tip UI 入口，tip 实际值始终为 0，因此 `fee` 等同于真实手续费。若未来引入 tip 机制，需同步更新下游消费方。

`FeeShareBurnt { reason, amount }` 记录分账份额无法安全入账后的实际销毁金额。该事件不改变 `FeePaid` 的统计语义：`FeePaid` 表示已扣基础手续费，`FeeShareBurnt` 表示其中某个制度份额最终没有成功入账。

### 外部依赖方
- **node RPC `fee_blockFees`**（`node/src/core/rpc.rs`）：累加指定区块内所有 `FeePaid.fee` 与 `TransactionFeePaid.tip`，返回该区块的实际手续费总额。
- **节点桌面端 mining-dashboard**（`node/src/mining/mining-dashboard/mod.rs`）：读取 `fee_blockFees` 结果作为矿工手续费收益统计口径。
- **runtime 注册**（`runtime/src/lib.rs` pallet_index 4）：`OnchainTransaction` 发出事件。

### 注意
`FeePaid.fee` 不含 tip；源码事件旁已有 NOTE 链回本节。当前 RPC 已显式补加 `TransactionFeePaid.tip`，因此当未来引入 tip 入口时，必须同步检查 runtime 事件、RPC 聚合、mining-dashboard 展示与本文档，避免不同入口对"手续费总额"采用不同口径。
