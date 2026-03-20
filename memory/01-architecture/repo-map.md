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
