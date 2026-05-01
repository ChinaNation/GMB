# Issuance 目录说明

本目录用于承载 CitizenChain runtime 下的发行相关 pallet 与文档。
当前发行相关 crate 已统一放在本目录下，后续新增发行 pallet 也必须直接落在这里。

## Benchmark feature 传播规则

- 发行模块启用 `runtime-benchmarks` 时，必须把已使用且已暴露同名 feature 的下游 runtime 依赖一并传播。
- 当前需要显式传播的下游包括 `pallet-balances`、`voting-engine` 和 `sfid-system`。
- `primitives` 当前不暴露 `runtime-benchmarks` feature，发行模块不得引用 `primitives/runtime-benchmarks`。
