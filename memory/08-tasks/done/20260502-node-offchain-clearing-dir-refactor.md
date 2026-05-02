# 任务卡：重构 node 清算行 offchain 目录为清算行专属能力分层

## 任务需求

按清算行产品边界重构 `citizenchain/node` 的 offchain 目录：

- 在 node 清算行域下拆分 `duoqian_manage`、`duoqian_transfer`、`offchain_transaction`、`settlement`
- `duoqian_manage` 仅承载清算行机构多签注册/创建/状态查询
- `duoqian_transfer` 仅承载清算行机构自己的多签转账能力，不包含国储会安全基金
- `offchain_transaction` 承载清算行节点声明、扫码支付收单、WuMinApp 支付 RPC、本地 L3 账本和 pending 交易
- `settlement` 承载 pending 交易打包、管理员签 batch、提交 Runtime 批量清算、链上事件监听和主账对账
- 完成后更新文档、完善注释、清理残留

## 所属模块

- `citizenchain/node/src/offchain`
- `citizenchain/node/frontend/offchain`
- `memory/05-modules/citizenchain/node/offchain`

## 边界约束

- 本次只做清算行产品域目录重构，不把普通机构、个人多签、国储会安全基金迁入该域
- Runtime pallet 目录不改
- Tauri 命令名称尽量保持兼容，避免前端调用面大改
- 迁移后必须清理旧平铺文件残留

## 验收标准

- 后端清算行目录按目标能力分层
- 前端清算行目录按目标能力分层
- Rust / TypeScript 引用修正完成
- 相关技术文档同步更新
- 执行可行的编译或静态检查，并记录结果

## 执行记录

- 已拆分 `citizenchain/node/src/offchain`：
  - `common`
  - `duoqian_manage`
  - `duoqian_transfer`
  - `offchain_transaction`
  - `settlement`
- 已拆分 `citizenchain/node/frontend/offchain`：
  - `duoqian-manage`
  - `duoqian-transfer`
  - `offchain-transaction`
  - `settlement`
- 已更新 Tauri 注册入口、节点 RPC 接线、清算行启动接线、网络概览统计引用。
- 已更新 `memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md` 与仓库映射文档。
- 已清理 `npm run build` 生成的 `frontend/dist` 构建残留。

## 检查结果

- `npm run build`：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check -p node --bin citizenchain`：通过。
- 说明：直接运行不带 `WASM_FILE` 的 `cargo check -p node --bin citizenchain` 会被 `runtime/build.rs` 拒绝，这是仓库现有统一 WASM 约束。
