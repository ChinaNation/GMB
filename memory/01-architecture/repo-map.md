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
  scripts/
```

## 2. 目录职责

- `.github/`：GitHub Actions、PR 自动化、构建发布流程
- `memory/`：AI 编程系统、项目长期记忆、产品文档与模块文档真源
- `citizenchain/`：区块链 runtime、节点程序、节点桌面 UI、打包发布
- `sfid/`：在线身份系统
- `cpms/`：离线实名系统
- `wuminapp/`：手机 App
- `scripts/`：统一脚本

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

## 5. citizenchain 当前结构

当前仓库已经按目标结构落地：

- `citizenchain/runtime/governance`
- `citizenchain/runtime/issuance`
- `citizenchain/runtime/otherpallet`
- `citizenchain/runtime/transaction`
- `citizenchain/runtime/primitives`
- `citizenchain/nodeuitauri`
- `citizenchain/nodeui`

其中：

- 四类 runtime 业务目录已经统一收敛到 `citizenchain/runtime/`
- 原仓库根目录 `primitives/` 已迁入 `citizenchain/runtime/primitives`
- 现有旧版 Tauri 节点 UI 使用 `citizenchain/nodeuitauri`
- 新版 Flutter Desktop 节点 UI 使用 `citizenchain/nodeui`

## 6. 当前落地策略

当前结构已经完成物理整合，后续新增 runtime 相关 crate 与文档均直接放在 `citizenchain/runtime/` 下，不再回到旧顶层目录。

## 7. GitHub Actions 路径分流原则

GMB 的自动化不再采用“改了 `citizenchain/**` 就把整条区块链流水线全部跑一遍”的模式，而是逐步改为按二级目录分流。

当前已经落地的规则：

- 改 `citizenchain/runtime/**`
  - 触发 `runtime` 相关 CI
- 改 `citizenchain/node`
  - 只触发 `node` 相关 CI
- 改 `citizenchain/runtime/primitives/**`、`citizenchain/Cargo.toml`、`citizenchain/Cargo.lock`
  - 触发 `runtime` 与 `node` 两侧 CI
- benchmark 自动化只对 `runtime` 相关目录和共享 Rust 目录触发
- `sfid` 部署流程按 `backend / frontend / deploy` 二级目录触发
- Pages 只在 `docs/` 或自身 workflow 配置变更时触发

跨模块共享目录仍然会触发多个模块，这是刻意保留的安全边界，而不是路径分流失效。
