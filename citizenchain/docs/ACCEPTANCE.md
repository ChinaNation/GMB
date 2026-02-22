# 自动化验收说明

## 1. 脚本位置

`/Users/rhett/GMB/citizenchain/scripts/acceptance.sh`

## 2. 使用方式

在 `/Users/rhett/GMB/citizenchain` 目录执行：

```bash
./scripts/acceptance.sh quick
./scripts/acceptance.sh full
./scripts/acceptance.sh coverage
./scripts/acceptance.sh coverage --html --lcov
```

## 3. 三种模式说明

- `quick`
  - `cargo check --workspace`
  - `cargo test --workspace -q`
- `full`
  - 包含 `quick` 全部内容
  - 增加关键模块回归测试（runtime、voting、sfid、链上/链下手续费）
- `coverage`
  - 运行 `cargo llvm-cov --workspace --summary-only`
  - 可选 `--html` 输出 HTML 报告
  - 可选 `--lcov` 输出 LCOV 报告

## 4. 关于 llvm-cov 的 WASM 问题

你当前环境在覆盖率场景下会触发：`can't find crate for profiler_builtins`。
脚本在 coverage 模式已固定使用：

```bash
SKIP_WASM_BUILD=1
```

用于跳过 wasm 覆盖率构建，保证覆盖率流程可运行。

## 5. 报告位置

- 验收日志：`/Users/rhett/GMB/citizenchain/target/acceptance/report-*.log`
- LCOV：`/Users/rhett/GMB/citizenchain/coverage.lcov`
- HTML：`/Users/rhett/GMB/citizenchain/target/llvm-cov/html`
