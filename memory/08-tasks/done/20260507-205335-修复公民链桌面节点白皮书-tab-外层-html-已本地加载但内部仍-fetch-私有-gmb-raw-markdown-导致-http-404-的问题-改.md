# 任务卡：修复公民链桌面节点白皮书 tab 外层 HTML 已本地加载但内部仍 fetch 私有 GMB raw Markdown 导致 HTTP 404 的问题；改为由节点前端 Vite 在开发与构建时提供本地 memory/00-vision/GMB_WHITEPAPER.md，并同步更新文档、注释和任务记录。

- 任务编号：20260507-205335
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-05-07 20:53:35

## 任务需求

修复公民链桌面节点白皮书 tab 外层 HTML 已本地加载但内部仍 fetch 私有 GMB raw Markdown 导致 HTTP 404 的问题；改为由节点前端 Vite 在开发与构建时提供本地 memory/00-vision/GMB_WHITEPAPER.md，并同步更新文档、注释和任务记录。

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
- `memory/05-modules/citizenchain/node`

### 先沟通条件

- 修改桌面前端与节点后端的交互边界
- 修改桌面打包、sidecar 或安装包行为

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
- 已确认白皮书 tab 的外层 `/GMB_README.html` 已本地加载，失败点是页面内部仍
  `fetch` 私有 GMB 仓库 raw Markdown，返回 `HTTP 404`。
- 已将白皮书 HTML 内部 Markdown 地址改为应用内 `/GMB_WHITEPAPER.md`。
- 已在 Vite 配置中新增本地白皮书插件：开发模式从
  `memory/00-vision/GMB_WHITEPAPER.md` 读取，正式构建时发射到 `frontend/dist/`。
- 已执行 `npm run build`，确认 `dist/GMB_README.html` 请求 `/GMB_WHITEPAPER.md`，
  且 `dist/GMB_WHITEPAPER.md` 已生成。
- 已用 `cmp` 确认构建产物 `dist/GMB_WHITEPAPER.md` 与
  `memory/00-vision/GMB_WHITEPAPER.md` 完全一致。
- 已执行 `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.wasm cargo check -p node`，
  检查通过。
- 已清理本次验证生成的 `citizenchain/node/frontend/dist/` 构建产物。

## 完成信息

- 完成时间：2026-05-07 20:54:56
- 完成摘要：修复白皮书 tab 内层 Markdown 仍请求私有 GMB raw 地址导致 HTTP 404 的问题；改为应用内 /GMB_WHITEPAPER.md，并由 Vite 从 memory/00-vision/GMB_WHITEPAPER.md 真源提供开发与构建产物；前端构建、Markdown 一致性比对和 node cargo check 均通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
