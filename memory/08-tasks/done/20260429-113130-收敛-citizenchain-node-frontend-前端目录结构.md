# 任务卡：收敛 citizenchain/node/frontend 前端目录结构

- 任务编号：20260429-113130
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-04-29 11:31:30

## 任务需求

参考 node/src 的 core、desktop/app、shared、home、mining、governance、offchain、settings、other 收敛逻辑，重组 node/frontend：根层只保留构建配置，App/main 进入 app，Tauri invoke 进入 core，跨功能格式化/SS58/QR 扫码进入 shared，各功能 API 与 types 回收到对应功能目录，并同步修正引用、文档和残留路径。

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
- 已建立 `frontend/app/`、`frontend/core/`、`frontend/shared/` 基础层：
  - `app/` 承载 `App.tsx`、`main.tsx` 与全局样式。
  - `core/tauri.ts` 统一封装 Tauri `invoke` 与错误消息清理。
  - `shared/format.ts`、`shared/ss58.ts`、`shared/qr/` 承载跨功能复用能力。
- 已按功能拆分 API 与类型：
  - `home/home-node/api.ts|types.ts`
  - `home/transaction/api.ts|types.ts`
  - `mining/dashboard/api.ts|types.ts`
  - `governance/api.ts|types.ts`
  - `settings/api.ts|types.ts`
  - `other/other-tabs/api.ts|types.ts`
  - `offchain/api.ts|types.ts` 保持在清算行模块内
- 已删除前端根层 `api.ts`、`types.ts`、`format.ts`，并清理旧 `assets/`、`qr/`、`utils/` 空目录。
- 已同步模块文档，明确前端目录与后端 `src` 功能目录的对应关系。
- 验证记录：
  - `npm run build`：通过。
  - 根层残留断言：`api.ts`、`types.ts`、`format.ts`、`App.tsx`、`main.tsx` 均已不存在。
  - 旧目录残留断言：`assets/`、`qr/`、`utils/` 均已不存在。
  - 旧导入残留扫描：`governance-types`、`transaction-types`、旧 QR 路径、旧 `mining/mining-dashboard` 与旧 `frontend/transaction` 路径均无命中。

## 完成信息

- 完成时间：2026-04-29 11:40:55
- 完成摘要：完成 citizenchain/node/frontend 前端目录收敛：app/core/shared 基础层已建立，各功能 API 与 types 已回收到对应目录，旧根层 api/types/format 与旧 QR/工具目录已清理，npm run build 通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
