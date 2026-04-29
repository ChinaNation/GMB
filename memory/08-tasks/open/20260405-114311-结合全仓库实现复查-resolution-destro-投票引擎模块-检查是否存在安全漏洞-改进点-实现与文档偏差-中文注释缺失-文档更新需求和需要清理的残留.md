# 任务卡：结合全仓库实现复查 resolution-destro 投票引擎模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

- 任务编号：20260405-114311
- 状态：open
- 所属模块：citizenchain/runtime/governance/resolution-destro
- 当前负责人：Codex
- 创建时间：2026-04-05 11:43:11

## 任务需求

结合全仓库实现复查 resolution-destro 投票引擎模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

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
- citizenchain/runtime/governance/resolution-destro/src/lib.rs
- citizenchain/runtime/governance/voting-engine/src/lib.rs
- citizenchain/runtime/src/configs/mod.rs
- memory/05-modules/citizenchain/runtime/governance/resolution-destro/RESOLUTIONDESTRO_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/MODULE_TAG_REGISTRY.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/transaction`
- 必要时联动 `primitives`
- 必要时联动 `citizenchain/node` 与 `memory/05-modules/wuminapp`

### 先沟通条件

- 修改 runtime 存储结构
- 修改销毁治理核心规则
- 修改管理员资格或投票资格模型

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
- 已复查 `resolution-destro`、`voting-engine`、runtime 接线、`memory` 文档与 `wuminapp` 治理文档口径。
- 旧任务里最关键的“跨模块 proposal_data 误解码后误执行销毁”高危问题已由 `MODULE_TAG = b\"res-dst\"` + 前缀校验修复，本轮未发现新的可利用漏洞。
- 修复模块自带 mock 测试残留：补齐 `voting_engine::Config::MaxAdminsPerInstitution`，并为 `TestInternalAdminProvider` 增加 `get_admin_list`，使管理员快照与当前投票引擎契约一致。
- 清理 benchmark 残留：`resolution-destro` benchmark 改为读取投票引擎真实分配的 `proposal_id`，不再把 `proposal_id` 写死成 `0`。
- 已更新模块技术文档，修正测试口径并补充 `weights.rs` 重生成的真实阻塞说明。

## 当前结论

- 未发现新的链上可利用漏洞；当前主要残留是 `weights.rs` 仍为旧代码生成产物，权重值保守但 proof 注释已过期。
- 标准 CI WASM 不带 benchmarking runtime api，直接用它构建的本地节点无法重跑 pallet benchmark；如需更新正式 `weights.rs`，需要单独使用带 benchmark api 的 runtime blob。

## 已执行验证

- `cargo test --offline --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/resolution-destro/Cargo.toml -- --nocapture`
- `cargo check --offline --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p resolution-destro --features runtime-benchmarks`
- `cargo check --offline --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p node`
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo build --offline --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p node --features runtime-benchmarks`
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm /Users/rhett/GMB/citizenchain/target/debug/citizenchain benchmark pallet --chain citizenchain --pallet resolution_destro --extrinsic '*' --steps 2 --repeat 1 --output /tmp/resolution_destro_weights.rs`

## 验证阻塞

- 标准 `cargo check --offline --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p node` 会被 runtime `build.rs` 的 `WASM_FILE` 约束拦下；若要本地编 node，需要显式提供 CI WASM 路径。
- 标准 CI WASM 构建出的本地 benchmark 节点在执行 `benchmark pallet` 时返回 `Did not find the benchmarking runtime api`，说明当前本地可用 runtime blob 不带 benchmark api，无法直接重生成正式 `weights.rs`。
