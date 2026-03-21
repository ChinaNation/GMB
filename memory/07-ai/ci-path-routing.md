# GMB CI 路径分流规则

## 1. 目标

GMB 的 GitHub Actions 采用“按改动目录精确触发”的策略，避免无关模块互相拖慢。

核心原则：

- 改哪个模块，就优先只跑哪个模块
- 共享依赖变更时，允许多模块联动触发
- 安全门禁与 Claude 审查属于跨模块能力，继续对 PR 全局生效

## 2. citizenchain 当前规则

### 2.1 node

- workflow：`.github/workflows/citizenchain-node.yml`
- 主要命中目录：
  - `citizenchain/node/**`
  - `citizenchain/runtime/primitives/**`
  - `citizenchain/runtime/src/**`
  - `citizenchain/runtime/Cargo.toml`
  - `citizenchain/Cargo.toml`
  - `citizenchain/Cargo.lock`

### 2.2 nodeui

- workflow：`.github/workflows/citizenchain-nodeui.yml`
- 主要命中目录：
  - `citizenchain/nodeui/**`
  - `.github/scripts/prepare-nodeui-sidecar.sh`

### 2.3 runtime/governance

- workflow：`.github/workflows/citizenchain-runtime-governance.yml`
- 主要命中目录：
  - `citizenchain/runtime/governance/**`
  - `citizenchain/runtime/Cargo.toml`
  - `citizenchain/Cargo.toml`
  - `citizenchain/Cargo.lock`

### 2.4 runtime/issuance

- workflow：`.github/workflows/citizenchain-runtime-issuance.yml`
- 主要命中目录：
  - `citizenchain/runtime/issuance/**`
  - `citizenchain/runtime/Cargo.toml`
  - `citizenchain/Cargo.toml`
  - `citizenchain/Cargo.lock`

### 2.5 runtime/otherpallet

- workflow：`.github/workflows/citizenchain-runtime-otherpallet.yml`
- 主要命中目录：
  - `citizenchain/runtime/otherpallet/**`
  - `citizenchain/runtime/Cargo.toml`
  - `citizenchain/Cargo.toml`
  - `citizenchain/Cargo.lock`

### 2.6 runtime/primitives

- workflow：`.github/workflows/citizenchain-runtime-primitives.yml`
- 主要命中目录：
  - `citizenchain/runtime/primitives/**`
  - `citizenchain/runtime/Cargo.toml`
  - `citizenchain/Cargo.toml`
  - `citizenchain/Cargo.lock`

### 2.7 runtime/src

- workflow：`.github/workflows/citizenchain-runtime-src.yml`
- 主要命中目录：
  - `citizenchain/runtime/src/**`
  - `citizenchain/runtime/Cargo.toml`
  - `citizenchain/Cargo.toml`
  - `citizenchain/Cargo.lock`

### 2.8 runtime/transaction

- workflow：`.github/workflows/citizenchain-runtime-transaction.yml`
- 主要命中目录：
  - `citizenchain/runtime/transaction/**`
  - `citizenchain/runtime/Cargo.toml`
  - `citizenchain/Cargo.toml`
  - `citizenchain/Cargo.lock`

## 3. 其他模块的分流方向

当前仓库规则已经明确为：

- `sfid`
  - CI：`.github/workflows/sfid-ci.yml`
  - 部署：`.github/workflows/sfid-deploy.yml`
- `cpms`
  - CI：`.github/workflows/cpms-ci.yml`
- `wuminapp`
  - CI：`.github/workflows/wuminapp-ci.yml`
- `docs`
  - Pages：`.github/workflows/pages.yml`

## 4. 当前结论

路径分流的目的不是减少安全检查，而是减少无关重复构建。

因此：

- 全局门禁继续保留
- Claude 审查继续保留
- 模块级构建和测试按目录精确触发
- 共享 Rust 根目录变更允许触发多个 citizenchain workflow
