# GMB CI 路径分流规则

## 1. 目标

GMB 的 GitHub Actions 采用“按改动目录精确触发”的策略，避免无关模块互相拖慢。

核心原则：

- 改哪个模块，就优先只跑哪个模块
- 共享依赖变更时，允许多模块联动触发
- 安全门禁与 Claude 审查属于跨模块能力，继续对 PR 全局生效

## 2. citizenchain 当前规则

### 2.1 runtime WASM

- workflow：`.github/workflows/citizenchain-wasm.yml`
- 主要命中目录：
  - `citizenchain/runtime/**`
  - `.github/workflows/citizenchain-wasm.yml`

### 2.2 node 桌面安装包

- workflow：
  - `.github/workflows/citizenchain-linux.yml`
  - `.github/workflows/citizenchain-macos.yml`
  - `.github/workflows/citizenchain-windows.yml`
- 主要命中目录：
  - 当前三端打包 workflow 由 `CitizenChain WASM` 成功完成后触发
  - 手动触发时同样从最新 `citizenchain-wasm` artifact 读取统一 WASM
- 代码目录：
  - `citizenchain/node/**`
  - `citizenchain/node/frontend/**`
  - `citizenchain/node/src/<功能名>/**`

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
