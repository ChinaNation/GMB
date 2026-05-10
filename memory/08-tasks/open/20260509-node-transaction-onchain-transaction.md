# 2026-05-09 node 端 transaction 目录整合 Step 3：home/transaction → transaction/onchain-transaction（整体搬）

## 任务需求

把 node 端首页钱包/转账模块从 `home/transaction/` 整体搬到 `transaction/onchain_transaction/`，对齐 [runtime/transaction/onchain-transaction](citizenchain/runtime/transaction/onchain-transaction) 边界。钱包管理与转账捆绑保留（强耦合，不拆分）。runtime 侧零改动。

- 后端：`node/src/home/transaction/` → `node/src/transaction/onchain_transaction/`
- 前端：`node/frontend/home/transaction/` → `node/frontend/transaction/onchain-transaction/`

## 影响范围

- `citizenchain/node/src/home/transaction/`（搬迁源，全删）
- `citizenchain/node/src/transaction/onchain_transaction/`（新落地）
- `citizenchain/node/src/home/mod.rs`
- `citizenchain/node/src/transaction/mod.rs`
- `citizenchain/node/src/desktop/mod.rs`
- `citizenchain/node/frontend/home/transaction/`（搬迁源，全删）
- `citizenchain/node/frontend/transaction/onchain-transaction/`（新落地）
- `citizenchain/node/frontend/app/App.tsx`

## 风险点

- 8 个 Tauri invoke 命令名串必须保持不变（钱包 5 + 转账 3）。
- `Balances::transfer_keep_alive` 调用、QR 签名、fee 计算（`onchain_transaction::calculate_onchain_fee`）、wallet store 持久化全部零业务改动。
- `core/rpc.rs` 对 runtime `OnchainTransaction::FeePaid` 事件订阅零改动（pallet enum 名不变）。
- runtime 侧零改动。
- 前端文件深度不变（home/transaction/ 与 transaction/onchain-transaction/ 都是 2 级），相对路径零改。

## 执行状态

- [x] 后端 git mv `mod.rs / wallet_store.rs` → `transaction/onchain_transaction/`
- [x] `home/mod.rs` 删除 `pub mod transaction;`，并更新顶部注释
- [x] `transaction/mod.rs` 增加 `pub mod onchain_transaction;`
- [x] `desktop/mod.rs` 8 个 `home::transaction::*` → `crate::transaction::onchain_transaction::*`
- [x] 前端 git mv 7 文件 → `transaction/onchain-transaction/`（深度 2→2 不变，内部相对路径零改）
- [x] `app/App.tsx` import 路径改写
- [x] `cargo check -p node`（WASM_FILE）通过，仅 5 条与本次无关的旧 dead_code 警告
- [x] `npx tsc --noEmit` 通过（exit 0）
- [x] 残留扫描全零：`crate::home::transaction` / `home::transaction::` / `'../home/transaction'` / `'../../home/transaction'`
