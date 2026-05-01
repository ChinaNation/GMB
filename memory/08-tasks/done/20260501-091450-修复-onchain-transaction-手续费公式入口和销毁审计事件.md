# 任务卡：修复 onchain-transaction 手续费公式入口和销毁审计事件

- 任务编号：20260501-091450
- 状态：done
- 所属模块：citizenchain/transaction
- 当前负责人：Codex
- 创建时间：2026-05-01 09:14:50

## 任务需求

清理 calculate_onchain_fee 旧注释，统一 custom_fee_with_tip 与 calculate_onchain_fee 的手续费公式入口，新增手续费份额销毁链上事件，补充缺口测试，更新文档并清理残留。

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
- 已清理 `calculate_onchain_fee` 的旧扣费/分账文档注释，仅保留公式、单位和复用语义。
- 已将 `custom_fee_with_tip` 的基础费计算改为调用 `calculate_onchain_fee(amount_u128)`，统一 transaction-payment 实扣与 `duoqian-*` 预扣口径。
- 已新增 `BurnReason` 与 `FeeShareBurnt { reason, amount }` 事件，覆盖作者缺失、钱包未绑定、fullnode resolve 失败、NRC 缺失、NRC resolve 失败、安全基金 resolve 失败、安全基金地址解码失败等销毁原因。
- 已补充零费用短路、`correct_and_deposit_fee(None)`、fullnode resolve 失败、安全基金 resolve 失败及销毁事件断言等测试。
- 已更新 `ONCHAIN_TECHNICAL.md`，同步手续费公式入口、销毁事件、测试覆盖和运维排障说明。

## 验证记录

- `cargo fmt`
- `cargo test -p onchain-transaction`
- `WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain`
- `cargo check -p onchain-transaction --features runtime-benchmarks`
- `rg -n '使用旧版 \`Currency\` trait|miner_wallet|\`nrc_account\`：|by_rate\\.max\\(min_fee\\)' citizenchain/runtime/transaction/onchain-transaction/src/lib.rs memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`

## 完成信息

- 完成时间：2026-05-01 09:19:37
- 完成摘要：完成 onchain-transaction 手续费公式单一入口、销毁链上事件、缺口测试与文档更新，并完成验证。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
