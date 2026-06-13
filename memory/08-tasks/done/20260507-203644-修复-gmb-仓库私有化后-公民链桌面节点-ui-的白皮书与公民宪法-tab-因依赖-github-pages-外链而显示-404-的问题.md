# 任务卡：修复 GMB 仓库私有化后，公民链桌面节点 UI 的白皮书与公民宪法 tab 因依赖 GitHub Pages 外链而显示 404 的问题，改为加载随桌面前端打包的本地静态 HTML，并同步更新模块文档、注释与残留说明。

- 任务编号：20260507-203644
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-05-07 20:36:44

## 任务需求

修复 GMB 仓库私有化后，公民链桌面节点 UI 的白皮书与公民宪法 tab 因依赖 GitHub Pages 外链而显示 404 的问题，改为加载随桌面前端打包的本地静态 HTML，并同步更新模块文档、注释与残留说明。

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
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 公民钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 公民钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
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
- 已复现 `https://chinanation.github.io/GMB/GMB_README.html` 与
  `https://chinanation.github.io/GMB/FRC_README.html?v=20260310-1` 返回 404。
- 已确认本地 `docs/GMB_README.html` 与 `docs/FRC_README.html` 存在。
- 修复方案：节点后端 tab 配置改为返回应用内静态路径；Vite 构建将仓库根 `docs/`
  复制进桌面前端产物；Tauri CSP 移除 GitHub Pages iframe 白名单。
- 已执行 `npm run build`，确认 Vite 构建通过，且 `dist/GMB_README.html` 与
  `dist/FRC_README.html` 已生成。
- 已执行 `cargo check -p node`，首次检查被既有 runtime `build.rs` 的 `WASM_FILE`
  门禁拦截；随后使用本地已有 `target/ci-wasm/citizenchain.wasm` 显式设置
  `WASM_FILE` 复查通过。
- 已确认 Tauri `generate_context!` 需要 `frontend/dist` 存在；复查前先执行前端构建，
  再执行带 `WASM_FILE` 的 `cargo check -p node`，最终通过。
- 已清理本次验证生成的 `citizenchain/node/frontend/dist/` 构建产物。

## 完成信息

- 完成时间：2026-05-07 20:38:52
- 完成摘要：修复公民链桌面节点白皮书与公民宪法 tab 依赖 GitHub Pages 外链导致私有仓库后 404 的问题；改为通过 Vite 将仓库根 docs 静态 HTML 打入前端产物并使用应用内 iframe 路径；同步更新模块文档并清理本地构建产物。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
