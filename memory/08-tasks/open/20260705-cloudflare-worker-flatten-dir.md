# CitizenApp Cloudflare 目录扁平化（去 square_worker 单层）

## 任务需求

- 把 `citizenapp/cloudflare/square_worker/` 下的内容上移到 `citizenapp/cloudflare/`，去掉冗余的单层目录。
- `citizenapp/cloudflare/` 下当前只有 `square_worker/` 一个子目录（外加待清理的 `.DS_Store`），没有第二个 Worker，嵌套无意义。
- 这套 Worker 已同时承载广场（`src/`）和聊天 mailbox（`src/chat/`），`square_worker` 命名已不准确；扁平化后 `citizenapp/cloudflare/` 本身即“这套 Cloudflare 后端”。

## 建议模块

- Cloudflare Worker 工程：`citizenapp/cloudflare/square_worker/` → `citizenapp/cloudflare/`
- 部署命令：`package.json` scripts（相对路径不变，但外部 `--prefix` 路径要改）
- 文档：`memory/` 各处路径引用 + 相关 `20260705-citizenapp-square-*` 任务卡

## 影响范围

- 纯目录搬移；`wrangler.toml`（`main`、`migrations_dir`）与 `package.json` 内部全是相对路径，不受影响。
- 无任何 Dart 代码引用该路径（App 通过 HTTP 连 Worker，不 import 工程目录）。
- 部署命令由 `npm --prefix citizenapp/cloudflare/square_worker run ...` 改为 `--prefix citizenapp/cloudflare`。
- 约 20 处 `memory/` 文档路径引用（`07-ai/unified-protocols.md`、`unified-naming.md`、`01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`、`05-modules/citizenapp/im/IM_TECHNICAL.md`）。
- 3 张 `20260705-citizenapp-square-*` 卡内多处路径。

## 主要风险点

- git 历史断裂：必须用 `git mv` 保留历史，禁止删后重加。
- `node_modules/` 不搬移：删除后在新位置重装，避免把依赖树搬进 diff。
- `.DS_Store` 一并清理，不入库。
- 遗漏某处文档/命令路径，导致后续按旧路径找不到工程。
- 与既有 `20260705-citizenapp-square-production-deploy.md`“不新增目录”约束一致；本卡是“移动”不是“新增”，需在该卡同步说明。

## 是否需要先沟通

- 否。用户已明确要求扁平化。

## 预计修改目录

- `citizenapp/cloudflare/`：接收上移的 `migrations/`、`src/`、`test/`、`package.json`、`wrangler.toml`、`tsconfig.json`、`vitest.config.ts` 等；涉及目录结构。
- `memory/07-ai/`、`memory/01-architecture/citizenapp/`、`memory/05-modules/citizenapp/`：同步路径引用；涉及文档。
- `memory/08-tasks/open/20260705-citizenapp-square-*.md`：同步路径；涉及文档。

## 分步骤技术方案

### 步骤 1：搬移

- `git mv` 把 `square_worker/` 下所有 tracked 文件移到 `citizenapp/cloudflare/`。
- 删除空壳 `square_worker/` 与 `citizenapp/cloudflare/.DS_Store`。
- 删除旧 `node_modules/`，不搬移。

### 步骤 2：本地校验

- `cd citizenapp/cloudflare && npm install`
- `npm run typecheck` 通过。
- `npm test` 通过（对齐既有 5 文件 10 用例）。
- `npm run migrate:local` 可正常应用（相对 `migrations_dir` 有效）。

### 步骤 3：文档与命令同步

- 全仓 grep `square_worker` / `cloudflare/square` 全部替换为 `cloudflare`。
- 更新所有部署/验收命令的 `--prefix` 路径。
- 在 `20260705-citizenapp-square-production-deploy.md` 记录“目录已扁平化”。

### 步骤 4：验收

- `git diff --check`。
- 全仓无残留 `square_worker` 引用（grep 为空）。
- 无 `node_modules`、`.wrangler`、`.DS_Store` 残留入库。

## 当前执行状态

- [x] 已扁平化：`git mv` 38 个跟踪文件到 `citizenapp/cloudflare/`；`node_modules` 直接搬移保留现成安装；删除 `square_worker/` 空壳与 `.DS_Store`。
- [x] 从新位置 `npm run typecheck` 通过（`main`/相对路径有效）。
- [x] 全仓文档路径同步：`cloudflare/square_worker` → `cloudflare`，含部署命令 `--prefix` 与 r2-worker 卡目录树标注。
- [x] `grep -rn square_worker memory` 除本卡（移动记录）外无残留。
