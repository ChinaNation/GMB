# 任务卡：结合全仓库实现复查 shengbank-stake-interest 模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留；本轮只负责检查

- 任务编号：20260405-122920
- 状态：open
- 所属模块：citizenchain/issuance
- 当前负责人：Codex
- 创建时间：2026-04-05 12:29:20

## 任务需求

结合全仓库实现复查 shengbank-stake-interest 模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留；本轮只负责检查

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

- 开工前先确认任务属于 `runtime`、`node`、`nodeui` 或 `primitives`
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
- 已读取 `shengbank-stake-interest` 源码、`weights.rs`、`benchmarks.rs`、runtime 接线、制度常量、模块技术文档和上次审查任务卡
- 已执行：
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/issuance/shengbank-stake-interest/Cargo.toml`
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/issuance/shengbank-stake-interest/Cargo.toml --features runtime-benchmarks`
- 已确认上次审查中的两项旧问题已修复：
  - `runtime-benchmarks` 变体当前可以通过，`impl_benchmark_test_suite!` 不再被私有 `new_test_ext()` 卡住
  - 年度周期当前明确绑定 `pow_const::BLOCKS_PER_YEAR = 87_600`，不再是旧结论中的“沿用 30 秒创世占位值”

## 审查结论

- 未发现新的直接权限绕过、任意账户收款、越权补结算或资金记账不一致漏洞
- 主路径实现成立：
  - 收款地址固定来自 `CHINA_CH`
  - 自动结算按年度边界顺序推进
  - 任一年失败即停止后续年度
  - Root 补结算与强制推进边界明确
- 仍有 1 个工程稳定性问题和 2 个文档/流程残留需要后续处理

## 主要发现

1. `on_initialize` 结算路径的 weight 仍依赖手工估算，而不是由 benchmark 产物直接驱动。
   - 代码使用 `reads_writes + SETTLEMENT_CPU_OP_WEIGHT * ops` 手工返回 hook weight。
   - `benchmarks.rs` 虽然已经补了 `on_initialize_settlement` / `on_initialize_noop`，但 `weights.rs` 仍只生成 `force_settle_years` / `force_advance_year` 两个函数，hook benchmark 结果没有真正接入运行时权重口径。
   - 风险不是资金错误，而是后续实现变更后，年度结算 hook 的重量可能继续漂移却无人发现。

2. 失败分支测试覆盖仍然偏薄。
   - 现有测试已覆盖正常年度推进、Root 权限、自动补结算上限和年限边界。
   - 但没有看到对 `ShengBankDecodeFailed`、`ShengBankIdEncodeFailed`、`ShengBankPrincipalOverflow`、`ShengBankInterestBelowED`、`ShengBankYearSettlementFailed` 这些失败/审计事件分支的显式测试。
   - 当前这些分支主要靠代码阅读确认，不是由回归测试固定。

3. 自动上下文装载链路仍未登记该模块。
   - `memory/scripts/load-context.sh` 仍无法识别 `citizenchain/runtime/issuance/shengbank-stake-interest`，实际调用会落到“未识别模块”分支。
   - 这会导致后续同类任务无法自动装载 `SHENGBANK_TECHNICAL.md`。

4. 模块技术文档存在轻微漂移。
   - 文档中的 runtime 接线行号仍写成 `configs/mod.rs:404`，而当前实现已在更靠后位置。
   - 文档里“当前覆盖（16 个业务测试）”也已落后于当前测试数量（普通测试 18 个，`runtime-benchmarks` 变体 22 个）。
