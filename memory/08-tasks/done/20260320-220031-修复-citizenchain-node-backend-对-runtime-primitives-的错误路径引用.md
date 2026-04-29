# 任务卡：修复 citizenchain/node backend 对 runtime primitives 的错误路径引用，恢复 clean-dev.sh 的本地启动能力

- 任务编号：20260320-220031
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-03-20 22:00:31

## 任务需求

修复 citizenchain/node backend 对 runtime primitives 的错误路径引用，恢复 clean-dev.sh 的本地启动能力

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-node.md

### 默认改动范围

- `citizenchain/node`
- `memory/05-modules/citizenchain/node`

### 先沟通条件

- 修改节点 UI 与 node 的交互边界
- 修改桌面打包、sidecar 或安装包行为

## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`、`node` 或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 文档与残留必须一起收口

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenchain.md

# CitizenChain 完成标准

- 改动范围和所属模块清晰
- 关键逻辑已补中文注释
- 文档已同步更新
- 影响链规则、存储或发布行为的点都已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已定位启动失败根因：`citizenchain/node/backend/Cargo.toml` 仍引用历史顶层 `primitives/` 路径，导致 `cargo metadata` 在 `cargo tauri dev` 前置阶段失败
- 已将 `primitives` 本地依赖修正为 `citizenchain/runtime/primitives`
- 已补充 node 模块文档，记录当前仓库结构下的正确依赖路径约束
- 已定位第二层构建阻塞：`backend/build.rs` 复制 sidecar 前未创建 `backend/binaries/` 目录，首次本地构建会直接失败
- 已修复 `backend/build.rs`：复制前先创建 sidecar 目录，并按当前 `TARGET` 生成带架构的 sidecar 文件名
- 已定位第三层构建阻塞：Tauri 在编译期要求 `frontend/dist` 存在，当前仓库未预置该目录
- 已修复 `backend/build.rs`：在执行 `tauri_build::build()` 前自动创建 `frontend/dist/`
- 已验证 `cargo metadata --manifest-path citizenchain/node/Cargo.toml` 通过
- 已验证 `cargo check --manifest-path citizenchain/node/backend/Cargo.toml -q` 通过
- 已补装 `citizenchain/node/frontend` 本地依赖并验证 `npm run build` 通过
- 已实跑 `./citizenchain/scripts/clean-dev.sh`，日志到达 `Running /Users/rhett/GMB/citizenchain/node/target/debug/node`，随后由人工 `Ctrl+C` 正常结束

## 完成信息

- 完成时间：2026-03-20 22:08:28
- 完成摘要：已修复 node backend 的 primitives 路径、sidecar 目录与 frontendDist 前置目录问题，并完成 cargo、npm 与 clean-dev.sh 启动验证。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
