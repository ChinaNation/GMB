# 任务卡：node 清算行 offchain 前后端目录对齐

## 任务需求

按已确认规则改造 node 清算行目录结构：

- 清算行在 node 层统一归入 `offchain` 功能域。
- 后端目录使用 `citizenchain/node/src/offchain`。
- 前端目录使用 `citizenchain/node/frontend/offchain`。
- 前后端按同功能同文件夹、同文件名或同名子文件夹对齐。
- 完成结构迁移后，再输出清算行技术实现方案。

## 建议模块

- `citizenchain/node/src/offchain`
- `citizenchain/node/src/ui`
- `citizenchain/node/frontend/offchain`
- `citizenchain/node/frontend/api.ts`
- `memory/05-modules/citizenchain/node`
- `memory/08-tasks`

## 影响范围

- 清算行 Tauri command 注册入口
- 清算行 SFID 查询、链上查询、扫码签名、管理员解密
- 清算行 offchain 运行组件
- 清算行前端页面与类型引用
- node offchain 技术文档

## 技术方案

1. 后端目录迁移
   - 将 `node/src/ui/clearing_bank/*` 合并到 `node/src/offchain/*`。
   - 将原 offchain 运行组件按结算职责下钻到 `node/src/offchain/settlement/*`。
   - 更新 `node/src/offchain/mod.rs` 导出和 Tauri command 使用路径。

2. 前端目录迁移
   - 将 `node/frontend/clearing-bank/*` 迁移到 `node/frontend/offchain/*`。
   - 按对应功能改成 `register.tsx`、`admin.tsx`、`node.tsx`、`types.ts` 等命名。
   - 更新 `App.tsx`、`api.ts` 和页面内部 import。

3. 文档、注释、清理残留
   - 更新 node offchain 文档，说明前后端目录对齐规则。
   - 修正迁移后的中文注释路径说明。
   - 清理旧 `clearing_bank` / `clearing-bank` 业务目录残留。

## 验收标准

- `citizenchain/node/src/ui/clearing_bank` 不再作为清算行业务目录存在。
- `citizenchain/node/frontend/clearing-bank` 不再作为清算行业务目录存在。
- 后端清算行代码统一在 `citizenchain/node/src/offchain`。
- 前端清算行代码统一在 `citizenchain/node/frontend/offchain`。
- 前后端文件或子目录按功能对应。
- 文档已更新、中文注释已修正、残留已清理。

## 当前状态

- 状态：已完成
- 创建时间：2026-04-29

## 执行结果

- 已将 node 后端清算行管理模块从 `ui/clearing_bank` 收口到 `src/offchain`。
- 已将原 offchain 结算运行组件下钻到 `src/offchain/settlement`。
- 已将前端清算行页面从 `frontend/clearing-bank` 迁移到 `frontend/offchain`。
- 已更新 Tauri command 注册路径、前端 import、network overview 链上清算节点统计路径。
- 已删除旧的清算行业务目录。
- 已更新 ADR 与 node offchain 技术文档。
- 已修正旧路径和旧文件名相关中文注释。
- 已清理 `clearing_bank` / `clearing-bank` 目录残留,保留的 `/clearing-banks/...` 仅为 SFID API 路径。

## 验证结果

- `npm run build`：通过。
- `WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node`：通过；存在既有 runtime/node warning。
- `git diff --check`：通过。
