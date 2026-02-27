# SHENGBANK Stake Interest Technical Notes

## 1. 模块定位
`shengbank-stake-interest` 是一个 FRAME pallet，用于按年度向 `CHINA_CH` 省储行账户发放质押利息。

核心目标：
- 年度自动结算（在年度边界块触发）。
- 失败可审计（链上事件可见）。
- 运行期可恢复（Root 手动补结算/强制推进）。
- 地址可治理覆盖（无需 runtime 升级即可改收款账户）。

代码位置：
- `/Users/rhett/GMB/citizenchain/issuance/shengbank-stake-interest/src/lib.rs`

---

## 2. 关键常量与配置
来自 pallet 内部：
- `AUTO_BACKFILL_MAX_YEARS_PER_BLOCK = 8`
- `MAX_FORCE_SETTLE_YEARS = SHENGBANK_INTEREST_DURATION_YEARS`
- `SETTLEMENT_CPU_OP_WEIGHT = 50_000`

来自 `primitives::core_const`：
- `ENABLE_SHENGBANK_INTEREST_DECAY`
- `SHENGBANK_INITIAL_INTEREST_BP`
- `SHENGBANK_INTEREST_DECREASE_BP`
- `SHENGBANK_INTEREST_DURATION_YEARS`

Runtime 注入配置：
- `Config::Currency`
- `Config::BlocksPerYear`

---

## 3. 存储结构
- `LastSettledYear: u32`
  - 已成功结算的最后年度（`0` 表示未结算）。
- `ShengBankAccountOverrides: Map<[u8;48] -> AccountId>`
  - 省储行收款账户覆盖表，key 为 `shenfen_id_to_fixed48`。

---

## 4. 事件与错误
主要事件：
- `ShengBankInterestMinted { year, pallet_id, account, amount }`
- `ShengBankYearSettled { year }`
- `ShengBankYearSettlementFailed { year, success_count, total_count }`
- `ShengBankDecodeFailed { year, pallet_id }`
- `ShengBankIdEncodeFailed { year, index }`
- `ShengBankPrincipalOverflow { year, pallet_id }`
- `ShengBankYearForceAdvanced { year }`
- `ShengBankAccountOverrideSet { pallet_id }`
- `ShengBankAccountOverrideCleared { pallet_id }`

主要错误：
- `InvalidOperationCount`
- `InvalidYear`
- `UnknownShengBankId`

---

## 5. 自动结算流程（on_initialize）
触发条件：
- `per_year != 0`
- `block != 0`
- `block % per_year == 0`（年度边界）

快速路径：
- 非边界块直接 `Weight::zero()` 返回，不读存储。

边界块流程：
1. 读取 `current_year` 与 `last_settled_year`。
2. 若 `current_year > last_year` 且未超过制度年限，调用 `settle_next_years(...)`。
3. 按 `(reads, writes)` + `ops * SETTLEMENT_CPU_OP_WEIGHT` 计重返回。
4. 若边界块但无需结算，只计 `reads(1)`。

---

## 6. 年度结算核心逻辑
入口：`settle_next_years(current_year, max_years, block_opt) -> (reads, writes, ops)`

行为：
- 从 `last_year + 1` 开始顺序结算。
- 单次最多推进 `max_years` 年。
- 任一年失败会停止后续年度推进。

单年发放：`mint_interest_for_year(year) -> (reads, writes, success_count, total_count)`
- `total_count = CHINA_CH.len()`
- `rate_bp == 0` 时直接返回成功（无链上发放）
- 对每家银行：
  1. `shenfen_id_to_fixed48` 失败：发 `ShengBankIdEncodeFailed` 并跳过。
  2. 解析收款账户：
     - 优先读 `ShengBankAccountOverrides`
     - 否则 decode `duoqian_address`
     - decode 失败：发 `ShengBankDecodeFailed` 并跳过
  3. `stake_amount -> BalanceOf<T>` 做饱和回写校验：
     - 若截断：发 `ShengBankPrincipalOverflow` 并跳过
  4. 计算利息：
     - `interest = principal * rate_bp / 10_000`
     - `interest == 0`：计为成功并跳过
     - `interest < minimum_balance`：warn + 计为成功并跳过
  5. `deposit_creating` 发放 + `ShengBankInterestMinted`

---

## 7. 治理接口（Root）
### 7.1 `force_settle_years(max_years)` call index = 0
- 作用：手动补结算若干年。
- 约束：`0 < max_years <= MAX_FORCE_SETTLE_YEARS`
- 返回：`DispatchResultWithPostInfo`，按实际执行量回填 `actual_weight`。

### 7.2 `force_advance_year(year)` call index = 1
- 作用：跳过故障年度，强制推进结算进度。
- 约束：`year > current` 且 `year <= SHENGBANK_INTEREST_DURATION_YEARS`

### 7.3 `set_shengbank_account_override(pallet_id, account)` call index = 2
- 作用：设置省储行收款账户覆盖。
- 约束：`pallet_id` 必须属于已知省储行。

### 7.4 `clear_shengbank_account_override(pallet_id)` call index = 3
- 作用：清除省储行账户覆盖。
- 约束：同样要求 `pallet_id` 合法。

---

## 8. Weight 策略
当前实现为“存储读写估算 + CPU 兜底”：
- 存储部分：`T::DbWeight::reads_writes(reads, writes)`
- 计算部分：`Weight::from_parts(ops * SETTLEMENT_CPU_OP_WEIGHT, 0)`

说明：
- 已尽量保守估算 `mint_interest_for_year` 的读取写入。
- 尚未接入 benchmark 自动生成权重；后续可引入 `frame_benchmarking` 做精确化。

---

## 9. 测试覆盖（当前）
`cargo test -p shengbank-stake-interest` 当前覆盖：
- 年度 1/2 正常发放
- 边界块自动补结算
- 年限上限停止
- Root 强制推进
- Root 手动补结算
- 地址覆盖设置/清理
- 非 Root 调用拒绝
- 参数非法拒绝
- 自动补结算上限（8 年）
- `BlocksPerYear == 0` 禁用自动结算

---

## 10. 运维建议
- 结算失败先看链上事件：
  - `ShengBankIdEncodeFailed`
  - `ShengBankDecodeFailed`
  - `ShengBankPrincipalOverflow`
  - `ShengBankYearSettlementFailed`
- 若为账户问题，优先用 `set_shengbank_account_override` 修复。
- 若某年度必须跳过，使用 `force_advance_year`。
- 若系统落后多个年度，使用 `force_settle_years` 快速追平。
