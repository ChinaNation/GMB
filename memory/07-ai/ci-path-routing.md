# GMB CI 路径分流规则

## 1. 目标

GMB 的 GitHub Actions 采用“按改动目录精确触发”的策略，避免无关模块互相拖慢。

核心原则：

- 改哪个模块，就优先只跑哪个模块
- 共享依赖变更时，允许多模块联动触发
- 安全门禁与 Claude 审查属于跨模块能力，继续对 PR 全局生效

## 2. citizenchain 当前规则

### 2.1 runtime 侧

以下目录命中后，触发 `runtime` 相关流水线：

- `citizenchain/runtime/**`
- `citizenchain/governance/**`
- `citizenchain/issuance/**`
- `citizenchain/otherpallet/**`
- `citizenchain/transaction/**`

触发的检查包括：

- `runtime 编译检查`
- `runtime 单元测试`
- `runtime WASM 安全检查`

### 2.2 node 侧

以下目录命中后，触发 `node` 相关流水线：

- `citizenchain/node/**`

触发的检查包括：

- `node 编译检查`
- `node 单元测试`

### 2.3 共享 Rust 目录

以下目录命中后，同时触发 `runtime` 与 `node` 两侧：

- `primitives/**`
- `citizenchain/Cargo.toml`
- `citizenchain/Cargo.lock`

原因：

- 这些目录属于 `citizenchain` 的共享依赖面
- 如果只跑单边，很容易漏掉交叉编译错误

## 3. benchmark 规则

`Benchmark Weights` 只对以下路径触发：

- `citizenchain/runtime/**`
- `citizenchain/governance/**`
- `citizenchain/issuance/**`
- `citizenchain/otherpallet/**`
- `citizenchain/transaction/**`
- `primitives/**`
- `citizenchain/Cargo.toml`
- `citizenchain/Cargo.lock`
- benchmark 脚本与模板文件

明确不再因为单纯的 `citizenchain/node/**` 改动触发 benchmark。

## 4. 其他模块的分流方向

当前仓库规则已经明确为：

- `sfid`：按 `backend / frontend / deploy` 二级目录触发
- `cpms`：后续按 `backend / frontend / deploy` 二级目录补齐
- `wuminapp`：后续按 `lib / android / ios / test` 等二级目录补齐
- `docs`：只触发 Pages 相关流程

## 5. 当前结论

路径分流的目的不是减少安全检查，而是减少无关重复构建。

因此：

- 全局门禁继续保留
- Claude 审查继续保留
- 模块级构建和测试按目录精确触发
