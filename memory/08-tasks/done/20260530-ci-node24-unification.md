# 任务卡：全仓库 CI 与前端 Node.js 24 统一

## 任务需求

统一全仓库自有前端项目和 GitHub Actions CI 的 Node.js 版本到 24，修复 GitHub Actions Node.js 20 运行时弃用预警；同步更新文档、完善说明注释并清理旧版本残留。

## 建议模块

- GitHub Actions workflow
- CitizenChain 桌面前端
- CPMS 前端
- CID 前端
- 官网前端
- CitizenChain 构建文档

## 影响范围

- `.github/workflows/`：升级 checkout/setup-node action，并固定 CI Node.js 24。
- `citizenchain/node/frontend/`：声明前端运行基线为 Node.js 24。
- `citizenpassport/frontend/`：声明前端运行基线为 Node.js 24。
- `citizencode/frontend/`：声明前端运行基线为 Node.js 24。
- `website/`：声明前端运行基线为 Node.js 24。
- `memory/05-modules/citizenchain/node/`：同步桌面端跨平台构建文档。

## 主要风险点

- 不能只改 CI，不改项目自身 Node 约束，否则本地与 CI 仍会版本不一致。
- 不修改第三方 vendored 依赖目录，避免把外部包的 Node 约束改成项目策略。
- 不保留 Node.js 20 的旧说明和旧 workflow action 残留。

## 是否需要先沟通

- 否。用户已确认统一改成 Node.js 24。

## 执行清单

- [x] 升级 GitHub Actions 中的 checkout/setup-node。
- [x] 四个自有前端项目声明 Node.js 24 运行基线。
- [x] 更新 lockfile 与构建文档。
- [x] 清理 Node.js 20 残留并运行验证。

## 完成记录

- 2026-05-30：创建任务卡，开始执行。
- 2026-05-30：GitHub Actions 的 `checkout` 升级为 v5，`setup-node` 升级为 v5，CI Node.js 固定为 24；四个自有前端项目补充 `engines.node >=24`。
- 2026-05-30：同步 package-lock，清理 CPMS/CID lockfile 中已不在 package.json 的 `qr-scanner` 残留，更新 CitizenChain 跨平台构建文档。
- 2026-05-30：运行 `npm ci --ignore-scripts`、四个前端 `npm run build`、workflow YAML 解析、`git diff --check` 和 Node.js 20/action v4 残留扫描；CitizenChain 构建顺带刷新本地文档生成物，使其与现有白皮书源文档一致。
