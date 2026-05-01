# 任务卡：修复 issuance runtime-benchmarks feature 下游传播缺口

- 任务编号：20260501-094932
- 状态：done
- 所属模块：citizenchain/runtime/issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 09:49:32

## 任务需求

按检查结论修复四个 issuance 模块的 `runtime-benchmarks` feature 传播缺口，完成后更新文档、完善注释、清理残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/08-tasks/open/20260501-094334-检查-issuance-runtime-benchmarks-feature-传播缺口.md
- memory/05-modules/citizenchain/runtime/issuance/citizen-issuance/CITIZENISS_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/issuance/fullnode-issuance/FULLNODE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/issuance/resolution-issuance/RESOLUTIONISSUANCE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/issuance/shengbank-interest/SHENGBANK_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不修改业务逻辑、权重或 benchmark 实现
- 不新增 `primitives/runtime-benchmarks`，因为 `primitives` 当前未暴露该 feature
- 改代码后必须更新文档和清理残留

## 输出物

- Cargo feature 修复
- 中文注释
- 文档更新
- 残留清理
- 验证记录

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已为 `citizen-issuance` 的 `runtime-benchmarks` 补充 `pallet-balances/runtime-benchmarks`。
- 已为 `fullnode-issuance` 的 `runtime-benchmarks` 补充 `pallet-balances/runtime-benchmarks`。
- 已为 `resolution-issuance` 的 `runtime-benchmarks` 补充 `pallet-balances/runtime-benchmarks` 与 `voting-engine/runtime-benchmarks`。
- 已为 `shengbank-interest` 的 `runtime-benchmarks` 补充 `pallet-balances/runtime-benchmarks`。
- 已在四个 Cargo feature 列表中加入中文注释，说明 benchmark feature 传播原因。
- 已更新四份 issuance 模块技术文档与 issuance README，明确 `primitives` 当前不暴露 benchmark feature，不应传播。

## 验证记录

- `cargo check --manifest-path citizenchain/runtime/issuance/citizen-issuance/Cargo.toml --features runtime-benchmarks`
- `cargo check --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml --features runtime-benchmarks`
- `cargo check --manifest-path citizenchain/runtime/issuance/resolution-issuance/Cargo.toml --features runtime-benchmarks`
- `cargo check --manifest-path citizenchain/runtime/issuance/shengbank-interest/Cargo.toml --features runtime-benchmarks`
- `cargo test --manifest-path citizenchain/runtime/issuance/citizen-issuance/Cargo.toml --features runtime-benchmarks`
- `cargo test --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml --features runtime-benchmarks`
- `cargo test --manifest-path citizenchain/runtime/issuance/resolution-issuance/Cargo.toml --features runtime-benchmarks`
- `cargo test --manifest-path citizenchain/runtime/issuance/shengbank-interest/Cargo.toml --features runtime-benchmarks`
