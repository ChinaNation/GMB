# SHENGBANK Stake Interest Technical Notes

## 0. 功能需求
`shengbank-stake-interest` 的功能需求是：按年度向 `CHINA_CH` 中固定 43 家省储行的多签账户发放质押利息，并把结算结果、失败原因和补救动作全部体现在链上。

模块必须满足以下要求：
- 利息接收账户只能是 `CHINA_CH` 中硬编码的省储行多签地址，不能由外部调用临时改写。
- 结算按照年度执行，基于 runtime 注入的 `BlocksPerYear` 在年度边界自动触发。
- 每年只允许顺序结算，不能跳过前一年直接结算后一年。
- 利率按制度常量执行：支持首年固定利率和逐年递减，到制度年限后自动归零。
- 只有当 43 家省储行在某一年度全部成功处理后，该年度才算真正 settled。
- 自动结算失败时必须保留在当前年度，等待 Root 手动补结算或强制推进，不允许静默跳过。
- Root 可以手动补结算若干已到期年度，也可以在故障无法修复时强制推进到某个已到期年度。
- 所有异常情况都要链上可审计，包括地址解码失败、身份编码失败、金额转换溢出和年度结算失败。

## 1. 模块定位
`shengbank-stake-interest` 是一个 FRAME pallet，用于按年度向 `CHINA_CH` 省储行账户发放质押利息。

核心目标：
- 年度自动结算，在年度边界区块触发。
- 失败链上可审计，不依赖节点本地日志。
- 运行期可恢复，支持 Root 补结算和故障年度跳过。
- 收款地址固定，不依赖运行期治理修改。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/issuance/shengbank-stake-interest/src/lib.rs`

## 2. 关键常量与配置
模块内常量：
- `AUTO_BACKFILL_MAX_YEARS_PER_BLOCK = 8`
- `MAX_FORCE_SETTLE_YEARS = SHENGBANK_INTEREST_DURATION_YEARS`
- `SETTLEMENT_CPU_OP_WEIGHT = 50_000`

制度常量：
- `ENABLE_SHENGBANK_INTEREST_DECAY`
- `SHENGBANK_INITIAL_INTEREST_BP`
- `SHENGBANK_INTEREST_DECREASE_BP`
- `SHENGBANK_INTEREST_DURATION_YEARS`

Runtime 注入：
- `Config::Currency = Balances`
- `Config::BlocksPerYear = ConstU64<{ primitives::pow_const::BLOCKS_PER_YEAR }>` — 白皮书定义 87,600 块/年，与出块时间无关（空块不允许上链，区块高度仅在有交易时推进）
- `Config::WeightInfo = shengbank_stake_interest::weights::SubstrateWeight<Runtime>`

Runtime 接线：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs:404`

## 3. 存储结构
- `LastSettledYear: u32`
  说明：记录已经完整结算成功的最后年度，`0` 表示尚未结算任何一年。

当前实现没有“账户覆盖表”或其他可变地址存储，收款地址完全来自 `CHINA_CH` 常量。

## 4. 事件与错误
主要事件：
- `ShengBankInterestMinted { year, pallet_id, account, amount }`
- `ShengBankDecodeFailed { year, pallet_id }`
- `ShengBankIdEncodeFailed { year, index }`
- `ShengBankPrincipalOverflow { year, pallet_id }`
- `ShengBankYearSettled { year }`
- `ShengBankYearSettlementFailed { year, success_count, total_count }`
- `ShengBankYearForceAdvanced { year }`
- `ShengBankInterestBelowED { year, pallet_id, amount }` — 利息低于 Existential Deposit，跳过发币（链上可审计）

主要错误：
- `InvalidOperationCount`
- `InvalidYear`

## 5. 自动结算流程
入口：`Hooks::on_initialize`

触发条件：
- `BlocksPerYear != 0`
- 当前区块非 0
- `block % BlocksPerYear == 0`

流程：
1. 计算 `current_year` 和 `last_settled_year`。
2. 若当前已经进入更高年度，且尚未达到制度年限，则调用 `settle_next_years(...)`。
3. 自动补结算单个区块最多推进 8 年，防止一次区块做过多历史回填。
4. 若在年度边界块但当前没有待结算年度，则只计一次 `LastSettledYear` 读取。

## 6. 年度结算逻辑
入口：`settle_next_years(current_year, max_years, block)`

约束：
- 从 `last_settled_year + 1` 开始顺序推进。
- 任一年失败后立刻停止后续年度，避免出现跨年错位。
- 只有 `success_count == total_count` 才会把该年写入 `LastSettledYear`。

单年发放：`mint_interest_for_year(year)`

对每家省储行执行：
1. 将 `shenfen_id` 编码成固定 48 字节 `pallet_id`。
2. 由 `duoqian_address` 解码出固定收款账户。
3. 将 `stake_amount` 转成运行时 `Balance`，并做回写校验防止饱和截断。
4. 计算 `interest = principal * rate_bp / 10_000`。
5. `interest == 0` 时视为成功但不发币。
6. `interest < minimum_balance` 时视为成功但跳过发币，作为 dust 防御兜底。
7. 其他情况用 `deposit_creating` 直接增发到省储行多签账户，并记录 `ShengBankInterestMinted`。

## 7. Root 补救接口
### 7.1 `force_settle_years(max_years)`（call index = 0）
- 作用：手动补结算若干已到期年度。
- 约束：`0 < max_years <= MAX_FORCE_SETTLE_YEARS`
- 返回：按实际读写和 CPU 操作量回填 `actual_weight`。

### 7.2 `force_advance_year(year)`（call index = 1）
- 作用：跳过已经到期但无法修复的故障年度。
- 约束：
  - `year > last_settled_year`
  - `year <= current_year`
  - `year <= SHENGBANK_INTEREST_DURATION_YEARS`

说明：
- 该接口不能提前跳过未来尚未到期的年度，避免误操作直接抹掉未来若干年的利息发放。

## 8. 权重策略
当前实现采用“读写估算 + CPU 兜底”的组合：
- 存储权重：`T::DbWeight::reads_writes(reads, writes)`
- 计算权重：`Weight::from_parts(ops * SETTLEMENT_CPU_OP_WEIGHT, 0)`

当前 `WeightInfo` 提供：
- `force_settle_years(max_years)`
- `force_advance_year()`

补充说明：
- `weights.rs` 当前为 `frame-benchmarking` 生成产物。
- `benchmarks.rs` 覆盖两个 Root 调用和两个 `on_initialize` 路径（结算路径 + 空操作路径），`force_settle_years` 的组件范围应与 `MAX_FORCE_SETTLE_YEARS` 保持一致。
- 若 benchmark 组件范围、执行路径或常量边界发生变化，需要重新生成 `weights.rs`，不是历史上跑过一次就可以永久沿用。

## 9. try-runtime 支持
Cargo.toml 已启用 `try-runtime` feature，依赖链：
- `frame-support/try-runtime`
- `frame-system/try-runtime`
- `sp-runtime/try-runtime`

Hooks 中实现了 `try_state` 钩子，校验：
- `LastSettledYear <= SHENGBANK_INTEREST_DURATION_YEARS`

## 10. 测试覆盖
执行命令：
- `cargo test -p shengbank-stake-interest`

当前覆盖（16 个业务测试）：
- 第 1 / 2 年正常发放与利率递减。
- 晚到边界时自动补结算。
- 年限达到上限后停止继续发放。
- Root 手动补结算。
- Root 强制推进恢复。
- 非 Root 调用拒绝。
- `force_settle_years` 参数校验。
- 自动补结算上限为 8 年。
- `BlocksPerYear == 0` 时禁用自动结算。
- 未来年度不能被 `force_advance_year` 提前跳过。
- 故障恢复后自动结算恢复（`force_advance_then_settle_resumes`）。
- `force_settle_years` 不超过当前年度（`force_settle_years_caps_at_current_year`）。
- 第 100 年边界利率 1 BP 正确发放，第 101 年不再发放（`year_100_boundary_settles_with_minimum_rate`）。

## 11. 审查结论与建议
本轮没有发现新的高风险权限绕过或资金记账一致性漏洞。

建议：
1. 若调整 benchmark 组件范围、年度上限或结算路径，应同步重新跑 benchmark 并更新 `weights.rs`。
2. `force_advance_year` 属于恢复型 Root 接口，运维上应优先修复故障再跳过年度，只把它当成最后手段。
3. 若未来 `CHINA_CH` 的 `stake_amount` 或运行时 `Balance` 精度发生变化，建议补一条”利息乘法溢出”显式检测事件，避免继续依赖饱和算术。
