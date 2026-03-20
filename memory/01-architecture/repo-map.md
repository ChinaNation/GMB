# GMB 仓库映射

## 1. 仓库原则

GMB 使用唯一仓库模式，所有核心系统、文档、自动化流程和 AI 永久记忆统一放在同一个仓库中管理。

仓库根目录固定如下：

```text
GMB/
  .github/
  memory/
  citizenchain/
  sfid/
  cpms/
  wuminapp/
  primitives/
  scripts/
```

## 2. 目录职责

- `.github/`：GitHub Actions、PR 自动化、构建发布流程
- `memory/`：AI 永久记忆中心
- `citizenchain/`：区块链 runtime、节点程序、节点桌面 UI、打包发布
- `sfid/`：在线身份系统
- `cpms/`：离线实名系统
- `wuminapp/`：手机 App
- `primitives/`：当前仓库级共享基础常量与基础类型 crate
- `scripts/`：统一脚本

## 3. citizenchain 目标结构

`citizenchain` 作为一个完整区块链桌面产品进行管理，目标结构如下：

```text
citizenchain/
  node/
  nodeuitauri/
  nodeui/
  runtime/
    governance/
    issuance/
    otherpallet/
    transaction/
    primitives/
  packaging/
  docs/
```

## 4. citizenchain 当前现状与目标关系

当前仓库处于迁移期：

- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- `citizenchain/nodeuitauri`

目标布局为：

- 上述四类 runtime 业务目录统一归入 `citizenchain/runtime/`
- 现有旧版 Tauri 节点 UI 已迁移为 `citizenchain/nodeuitauri`
- 新版 Flutter Desktop 节点 UI 使用 `citizenchain/nodeui`

## 5. 本阶段落地策略

本阶段先落文档和目标目录基线，不直接进行大规模物理迁移，避免破坏现有代码与构建流程。

## 6. GitHub Actions 路径分流原则

GMB 的自动化不再采用“改了 `citizenchain/**` 就把整条区块链流水线全部跑一遍”的模式，而是逐步改为按二级目录分流。

当前已经落地的规则：

- 改 `citizenchain/runtime`、`citizenchain/governance`、`citizenchain/issuance`、`citizenchain/otherpallet`、`citizenchain/transaction`
  - 只触发 `runtime` 相关 CI
- 改 `citizenchain/node`
  - 只触发 `node` 相关 CI
- 改 `primitives`、`citizenchain/Cargo.toml`、`citizenchain/Cargo.lock`
  - 触发 `runtime` 与 `node` 两侧 CI
- benchmark 自动化只对 `runtime` 相关目录和共享 Rust 目录触发
- `sfid` 部署流程按 `backend / frontend / deploy` 二级目录触发
- Pages 只在 `docs/` 或自身 workflow 配置变更时触发

跨模块共享目录仍然会触发多个模块，这是刻意保留的安全边界，而不是路径分流失效。
