# 任务卡：彻底扁平化 citizenchain/node/src/ui 目录：删除 ui 这一层路径，把桌面 UI Rust 后端模块统一迁移为 citizenchain/node/src/<功能名>，同步修正引用、文档、注释和残留路径。

- 任务编号：20260429-104514
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-04-29 10:45:14

## 任务需求

彻底扁平化 citizenchain/node/src/ui 目录：删除 ui 这一层路径，把桌面 UI Rust 后端模块统一迁移为 citizenchain/node/src/<功能名>，同步修正引用、文档、注释和残留路径。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-node.md

### 默认改动范围

- `citizenchain/node`
- 必要时联动 `citizenchain/runtime`

### 先沟通条件

- 修改节点启动方式
- 修改节点数据库或同步行为
- 修改安装包或发布形态


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`（含桌面端）或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 冷钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 冷钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
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
- 已将 `citizenchain/node/src/ui/mod.rs` 迁移为 `citizenchain/node/src/desktop.rs`。
- 已将 `governance`、`home`、`mining`、`network`、`node_runner.rs`、`other`、`settings`、`sfid_config.rs`、`shared`、`transaction` 从 `citizenchain/node/src/ui/` 上移到 `citizenchain/node/src/`。
- 已删除空的 `citizenchain/node/src/ui` 目录。
- 已将 Rust 引用从 `crate::ui::...` 改为 crate 根层模块引用，并将桌面启动入口改为 `desktop::run_desktop()`。
- 已将模块文档从 `memory/05-modules/citizenchain/node/` 并入 `memory/05-modules/citizenchain/node/`。
- 已同步更新架构文档、AI 路由文档、模块文档和开放任务中的当前路径口径。
- 已执行 `cargo fmt -p node`。
- 已执行 `cargo metadata --no-deps --format-version 1`，workspace/manifest 解析通过。
- 已执行 `npm run build`（`citizenchain/node/frontend`），TypeScript + Vite 构建通过。
- 验证记录：`cargo check -p node` 已执行，但被 `runtime/build.rs` 的 `WASM_FILE` 强制门禁拦截；该门禁要求使用 CI WASM，非本次迁移引入的编译错误。

## 完成信息

- 完成时间：2026-04-29 10:56:39
- 完成摘要：已彻底删除 citizenchain/node/src/ui 目录层，将桌面端 Rust 后端模块迁移到 citizenchain/node/src/<功能名>；同步修正 Rust 引用、桌面入口、模块文档、AI 路由和架构文档；已通过 cargo fmt、cargo metadata 与前端 npm build，cargo check 受 WASM_FILE CI WASM 门禁拦截。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
