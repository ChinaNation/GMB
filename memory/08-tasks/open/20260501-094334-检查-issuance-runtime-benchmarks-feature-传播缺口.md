# 任务卡：检查 issuance runtime-benchmarks feature 传播缺口

- 任务编号：20260501-094334
- 状态：done
- 所属模块：citizenchain/runtime/issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 09:43:34

## 任务需求

检查 `citizen-issuance`、`fullnode-issuance`、`resolution-issuance`、`shengbank-interest` 的 `runtime-benchmarks` feature 是否遗漏向下游依赖传播，并评估影响与推荐修复方案。

## 检查结论

- `primitives` 当前不暴露 `runtime-benchmarks` feature，不应在上游 feature 中引用 `primitives/runtime-benchmarks`。
- `voting-engine` 当前暴露 `runtime-benchmarks` feature。
- `citizen-issuance` 当前未传播 `pallet-balances/runtime-benchmarks`。
- `fullnode-issuance` 当前未传播 `pallet-balances/runtime-benchmarks`，该依赖位于 dev-dependencies，但 benchmark/test feature 链路仍建议显式保持一致。
- `resolution-issuance` 当前未传播 `voting-engine/runtime-benchmarks` 与 `pallet-balances/runtime-benchmarks`。
- `shengbank-interest` 当前未传播 `pallet-balances/runtime-benchmarks`，该依赖位于 dev-dependencies，但 benchmark/test feature 链路仍建议显式保持一致。

## 影响评估

- 当前单独执行各 crate 的 `cargo test --features runtime-benchmarks` 不一定失败，因为 FRAME benchmark feature 不强制所有下游依赖同步开启。
- 在 workspace 聚合、benchmark 构建或未来下游依赖新增 benchmark 条件代码时，feature 链路可能出现不一致，导致编译行为与预期 benchmark runtime 不完全一致。
- 风险等级低，属于 Cargo feature hygiene 与后续 benchmark 可维护性问题。

## 推荐方案

- 四个 crate 只补已存在的下游 feature，不新增 `primitives/runtime-benchmarks`。
- 补充后运行各 crate `cargo test --features runtime-benchmarks` 或至少 `cargo check --features runtime-benchmarks` 验证。

## 验证记录

- `cargo check --manifest-path citizenchain/runtime/issuance/citizen-issuance/Cargo.toml --features runtime-benchmarks`：通过。
- `cargo check --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml --features runtime-benchmarks`：通过。
- `cargo check --manifest-path citizenchain/runtime/issuance/resolution-issuance/Cargo.toml --features runtime-benchmarks`：通过。
- `cargo check --manifest-path citizenchain/runtime/issuance/shengbank-interest/Cargo.toml --features runtime-benchmarks`：通过。
