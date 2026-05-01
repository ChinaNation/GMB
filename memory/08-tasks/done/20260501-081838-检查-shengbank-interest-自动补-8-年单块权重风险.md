# 任务卡：检查 shengbank-interest 自动补 8 年单块权重风险

- 任务编号：20260501-081838
- 状态：done
- 所属模块：citizenchain/runtime
- 当前负责人：Codex
- 创建时间：2026-05-01 08:18:38

## 任务需求

检查 shengbank-interest 自动补 8 年单块权重风险

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
- citizenchain/runtime/README.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- 必要时联动 `primitives`

### 先沟通条件

- 修改 runtime 存储结构
- 修改资格模型
- 修改提案、投票、发行核心规则


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`（含桌面端）或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 冷钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 冷钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
- 文档与残留必须一起收口

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenchain.md

# CitizenChain 完成标准

- 改动范围和所属模块清晰
- 关键逻辑已补中文注释
- 文档已同步更新
- 影响链规则、存储或发布行为的点都已先沟通
- 残留已清理


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
- 已装载 `citizenchain/runtime` 上下文。
- 已检查 `citizenchain/runtime/issuance/shengbank-interest/src/lib.rs`、`weights.rs`、`benchmarks.rs`、runtime 配置和创世配置。
- 核查结论：问题“部分存在，且需要修复/重新建模权重”。代码确实允许 `AUTO_BACKFILL_MAX_YEARS_PER_BLOCK = 8` 在单个年度边界块内最多补 8 年；每年遍历固定 43 家省储行，正常路径最多触发 `8 * 43 = 344` 次 `T::Currency::deposit_creating` 和 `344` 个 `ShengBankInterestMinted` 事件。
- 进一步确认：创世余额预置到省储行 `stake_address`，年度利息发给 `main_address`，因此首次年度结算时 43 个 `main_address` 可能走账户创建路径。
- 进一步确认：`deposit_creating` 自身会产生 `Balances::Deposit`，返回的 `PositiveImbalance` drop 时会更新 `TotalIssuance` 并产生 `Balances::Issued`；新账户还可能产生 `Balances::Endowed`。因此事件与账户创建成本不只限于 `ShengBankInterestMinted`。
- 权重问题：`on_initialize` 未使用 benchmark 生成的 `WeightInfo`，而是在 hook 内手写返回 `T::DbWeight::reads_writes(reads, writes) + ops * 50_000`。`benchmarks.rs` 虽定义了 `on_initialize_settlement` / `on_initialize_noop`，但 `weights.rs` 的 `WeightInfo` 只暴露 `force_settle_years` 和 `force_advance_year`。
- 当前 8 年正常路径手写计数约为 `reads = 1033`、`writes = 1048`、`ops = 344`；按 runtime `RocksDbWeight` 粗算返回约 `130,642,200,000` ref_time，低于当前 60 秒区块上限。但这不是 benchmark 证明的上界，且 `50_000` 是 ref_time 不是 50,000ns，CPU/事件序列化/账户创建路径没有独立 benchmark 保证。
- 风险判断：存在单块集中补算和 hook 权重手写估算风险；“一定打爆当前 60 秒区块上限”暂不能仅凭静态代码证明，但当前实现缺少可信 benchmark 上界，属于 runtime 发行逻辑的高优先级修复项。
- 已执行修复：
  - `AUTO_BACKFILL_MAX_YEARS_PER_BLOCK` 从 8 调整为 1，自动年度边界块只结算下一个未结算年度。
  - `MAX_FORCE_SETTLE_YEARS` 从制度年限调整为 8，Root 补结算必须分批执行。
  - `on_initialize` 改为返回 `WeightInfo::on_initialize_settlement()` / `WeightInfo::on_initialize_boundary_noop()`，不再返回手写 `DbWeight + ops`。
  - `force_settle_years` 不再用手写读写计数回填 `actual_weight`，实际扣重保持使用声明的 benchmark 权重。
  - `benchmarks.rs` 将 `force_settle_years` 组件范围收敛为 `1..=8`，并把 hook 空路径 benchmark 调整为年度边界无待结算场景。
  - `weights.rs` 增加 hook 权重接口，结算路径复用单年 `force_settle_years(1)` benchmark 上界并额外加一次年度状态读取。
  - 已更新 `memory/05-modules/citizenchain/runtime/issuance/shengbank-interest/SHENGBANK_TECHNICAL.md`。
- 验证结果：
  - `cargo test -p shengbank-interest`：19 passed。
  - `cargo test -p shengbank-interest --features runtime-benchmarks`：23 passed。

## 完成信息

- 完成时间：2026-05-01 08:55:00
- 完成摘要：修复 shengbank-interest 自动补算单块权重风险，自动路径收敛为 1 年/块，Root 补结算上限 8 年，并接入 hook WeightInfo 权重与测试文档。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
