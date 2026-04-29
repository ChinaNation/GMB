# 任务卡：禁止 node/frontend 根层 Vite 配置生成物产出

- 任务编号：20260429-114553
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-04-29 11:45:53

## 任务需求

调整 node/frontend TypeScript 构建配置，使 npm run build 不再产出 vite.config.js 与 vite.config.d.ts，清理已有生成文件，并验证构建后这些文件不再复生。

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
- 已删除 `frontend/tsconfig.node.json`，由主 `tsconfig.json` 直接纳入 `vite.config.ts` 类型检查。
- 已将 `frontend/package.json` 的 `build` 脚本从 `tsc -b && vite build` 改为 `tsc --noEmit && vite build`，避免 TypeScript build mode 生成 `*.tsbuildinfo`。
- 已删除已有生成物：`vite.config.js`、`vite.config.d.ts`、`tsconfig.tsbuildinfo`、`tsconfig.node.tsbuildinfo`。
- 验证记录：
  - `npm run build`：通过。
  - 构建后 `vite.config.js`、`vite.config.d.ts`、`tsconfig.node.json`、`tsconfig.tsbuildinfo`、`tsconfig.node.tsbuildinfo` 均未复生。
  - `frontend` 根层剩余 7 个必要工程文件：`.nvmrc`、`index.html`、`package.json`、`package-lock.json`、`tsconfig.json`、`vite-env.d.ts`、`vite.config.ts`。

## 完成信息

- 完成时间：2026-04-29 11:48:29
- 完成摘要：完成 node/frontend 根层生成物清理：构建改为 tsc --noEmit，删除 tsconfig.node.json 与 vite.config.js/.d.ts 产物，确认 npm run build 后无生成物复生。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
