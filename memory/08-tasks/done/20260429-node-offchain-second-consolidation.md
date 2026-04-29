# 任务卡：node offchain 二次收口

## 任务需求

继续执行 node 清算行 offchain 功能域收口：

- `offchain_keystore.rs` 不再放在 `node/src` 根目录，迁入 `node/src/offchain/keystore.rs`。
- `service.rs` 中清算行启动逻辑抽入 `node/src/offchain/bootstrap.rs`。
- 前端清算行 API 从全局 `frontend/api.ts` 抽入 `frontend/offchain/api.ts`。
- 前端清算行样式从全局 CSS 抽入 `frontend/offchain/styles.css`。
- 更新文档、完善中文注释、清理残留。

## 建议模块

- `citizenchain/node/src/offchain`
- `citizenchain/node/src/service.rs`
- `citizenchain/node/frontend/offchain`
- `citizenchain/node/frontend/api.ts`
- `citizenchain/node/frontend/assets/styles/global.css`
- `memory/05-modules/citizenchain/node/offchain`
- `memory/08-tasks`

## 影响范围

- 清算行密钥存储与解锁
- 清算行启动 worker
- 清算行 JSON-RPC 注入
- 清算行前端 API 调用
- 清算行前端样式
- node offchain 技术文档

## 技术方案

1. 后端二次收口
   - 移动 `offchain_keystore.rs` 到 `offchain/keystore.rs`。
   - 更新 `settlement/signer.rs`、`settlement/submitter.rs`、`service.rs` 引用。
   - 新增 `offchain/bootstrap.rs`，把清算行 CLI 启动、密钥解锁、worker spawn 统一放入 offchain 功能域。

2. 前端二次收口
   - 新增 `frontend/offchain/api.ts`，只封装清算行 Tauri commands。
   - 更新 `frontend/offchain/*.tsx` 引用 `offchainApi`。
   - 从 `frontend/api.ts` 删除清算行 API。
   - 新增 `frontend/offchain/styles.css`，并由 `section.tsx` 引入。

3. 文档、注释、清理残留
   - 更新 node offchain 文档中的目录真源。
   - 修正 `offchain_keystore` 旧命名注释。
   - 搜索并清理旧路径、旧 API 聚合残留。

## 验收标准

- `node/src/offchain_keystore.rs` 不存在。
- 清算行密钥代码在 `node/src/offchain/keystore.rs`。
- `service.rs` 不再保留大段清算行启动逻辑。
- 清算行前端 API 在 `frontend/offchain/api.ts`。
- 清算行样式在 `frontend/offchain/styles.css`。
- 文档已更新、中文注释已修正、残留已清理。
- 构建检查通过。

## 当前状态

- 状态：已完成
- 创建时间：2026-04-29

## 执行记录

- 已将根级 `node/src/offchain_keystore.rs` 迁入 `node/src/offchain/keystore.rs`。
- 已新增 `node/src/offchain/bootstrap.rs`，`service.rs` 只保留节点通用启动接线。
- 已将清算行前端专属 Tauri API 迁入 `node/frontend/offchain/api.ts`。
- 已将清算行页面样式迁入 `node/frontend/offchain/styles.css`，并由 `section.tsx` 引入。
- 已更新 node offchain、SFID 清算行资格、ADR-007 与相关任务卡文档。
- 已用 `rg` 检查旧代码路径残留；当前代码层无旧 `ui/clearing_bank`、`frontend/clearing-bank`、`crate::offchain_keystore` 引用。

## 验证记录

- 通过：`npm run build`（`citizenchain/node/frontend`）
- 通过：`WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node`
- 已执行：`rustfmt --edition 2021`（本轮触达 Rust 文件）
