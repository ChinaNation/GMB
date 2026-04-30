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
  website/
```

## 2. 目录职责

- `.github/`：GitHub Actions、PR 自动化、构建发布流程
- `memory/`：AI 编程系统、项目长期记忆、产品文档与模块文档真源
- `citizenchain/`：区块链 runtime、节点程序、节点桌面 UI、打包发布
- `sfid/`：在线身份系统
- `cpms/`：离线实名系统
- `wuminapp/`：手机 App
- `website/`：GMB 官网前端工程，当前使用 React + TypeScript + Vite 构建静态站点

## 3. 文档集中管理规则

正式文档统一收口到 `memory/`：

- `memory/00-vision/`：白皮书、愿景、总目标
- `memory/01-architecture/`：仓库级与产品级架构文档
- `memory/05-modules/`：模块级技术文档
- `memory/07-ai/`：AI 编程系统规则
- `memory/08-tasks/`：任务卡、执行记录与归档

产品目录默认只保留：

- 源代码
- 配置
- 测试
- 构建与部署脚本
- 数据库迁移
- 运行所需资源文件

## 4. citizenchain 目标结构

`citizenchain` 作为一个完整区块链桌面产品进行管理，目标结构如下：

```text
citizenchain/
  node/
  runtime/
    governance/
    issuance/
    otherpallet/
    transaction/
    primitives/
  packaging/
  docs/
```

## 5. citizenchain 当前结构

当前仓库已经按目标结构落地：

- `citizenchain/runtime/governance`
- `citizenchain/runtime/issuance`
- `citizenchain/runtime/otherpallet`
- `citizenchain/runtime/transaction`
- `citizenchain/runtime/primitives`
- `citizenchain/node`

其中：

- 四类 runtime 业务目录已经统一收敛到 `citizenchain/runtime/`
- 原仓库根目录 `primitives/` 已迁入 `citizenchain/runtime/primitives`
- 原生节点、桌面节点 UI、Tauri 壳与打包入口统一收口到 `citizenchain/node`
- 桌面端 Rust 后端模块已经扁平化到 `citizenchain/node/src/<功能名>`，不再保留 `src/ui` 目录层
- 历史旧目录 `citizenchain/node` 与独立 `citizenchain/node` 均不再作为当前实现

## 6. 当前落地策略

当前结构已经完成物理整合，后续新增 桌面节点 Rust 后端功能直接放在 `citizenchain/node/src/<功能名>`，前端功能放在 `citizenchain/node/frontend/<功能名>`；新增 runtime 相关 crate 与文档均直接放在 `citizenchain/runtime/` 下，不再回到旧顶层目录。

## 7. GitHub Actions 路径分流原则

GMB 的自动化已经改为“每个系统 / 模块一个 workflow”：

- `citizenchain/node`
  - `.github/workflows/citizenchain-linux.yml`
  - `.github/workflows/citizenchain-macos.yml`
  - `.github/workflows/citizenchain-windows.yml`
- `citizenchain/runtime/governance`
  - `.github/workflows/citizenchain-runtime-governance.yml`
- `citizenchain/runtime/issuance`
  - `.github/workflows/citizenchain-runtime-issuance.yml`
- `citizenchain/runtime/otherpallet`
  - `.github/workflows/citizenchain-runtime-otherpallet.yml`
- `citizenchain/runtime/primitives`
  - `.github/workflows/citizenchain-runtime-primitives.yml`
- `citizenchain/runtime/src`
  - `.github/workflows/citizenchain-runtime-src.yml`
- `citizenchain/runtime/transaction`
  - `.github/workflows/citizenchain-runtime-transaction.yml`
- `sfid`
  - `.github/workflows/sfid-ci.yml`
- `cpms`
  - `.github/workflows/cpms-ci.yml`
- `wuminapp`
  - `.github/workflows/wuminapp-ci.yml`
- `website`
  - 当前暂无专用 GitHub Actions，官网发布前需在本地执行 `npm run build` 并部署 `website/dist/`

补充说明：

- `sfid` 部署仍由 `.github/workflows/sfid-deploy.yml` 单独负责
- Pages 只在 `docs/**` 或自身 workflow 变更时触发
- 共享 Rust 根目录变更允许触发多个 citizenchain workflow，这是保留的安全边界
