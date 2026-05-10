# 2026-05-09 node 端 transaction 目录整合 Step 2：offchain → offchain-transaction（扁平化）

## 任务需求

把 node 端 offchain 模块从顶层下沉到 `transaction/` 目录并改名为 offchain-transaction，**取消所有同名嵌套**：内层 `offchain/offchain_transaction/` 与 `offchain/common/` 子目录被扁平化合并到新 parent，仅保留 `settlement/` 子目录（独立 worker 子语义）。runtime 侧零改动。

- 后端：`node/src/offchain/` → `node/src/transaction/offchain_transaction/`
- 前端：`node/frontend/offchain/` → `node/frontend/transaction/offchain-transaction/`

## 影响范围

- `citizenchain/node/src/offchain/`（搬迁源，全删）
- `citizenchain/node/src/transaction/offchain_transaction/`（新落地）
- `citizenchain/node/src/transaction/mod.rs`
- `citizenchain/node/src/main.rs`
- `citizenchain/node/src/core/service.rs`
- `citizenchain/node/src/core/rpc.rs`
- `citizenchain/node/src/desktop/mod.rs`
- `citizenchain/node/src/mining/network-overview/mod.rs`
- `citizenchain/node/frontend/offchain/`（搬迁源，全删）
- `citizenchain/node/frontend/transaction/offchain-transaction/`（新落地）
- `citizenchain/node/frontend/app/App.tsx`
- `citizenchain/node/frontend/governance/organization-manage/institution-detail.tsx`
- `citizenchain/node/frontend/governance/organization-manage/create-multisig.tsx`

## 风险点

- 9+4=13 个 Tauri invoke 命令名串必须保持不变（commands::* / settlement::commands::*）。
- `PALLET_NAME = b"OffchainTransaction"` 与 runtime 一致，绝不动。
- 内层 `offchain/offchain_transaction/mod.rs` 与 `offchain/common/mod.rs` 是空壳 `pub mod` 声明，扁平合并后整个删除，零信息丢失。
- libp2p clearing-bank gossip / packer / listener / reserve_monitor 启动逻辑零改动。
- runtime 侧零改动。

## 执行状态

- [x] 后端 git mv 文件到 `transaction/offchain_transaction/` 平铺结构（18 文件 + settlement/ 子目录）
- [x] 删除 2 个空壳 mod.rs（`common/mod.rs`、`offchain_transaction/mod.rs`），原 `offchain/` 三层目录全删
- [x] 改写新 `mod.rs` 顶部 `pub mod` 列表（8 个平铺文件 + settlement），`use self::offchain_transaction::*` 简化为 `use self::*`
- [x] `transaction/mod.rs` 增加 `pub mod offchain_transaction;`
- [x] `main.rs` 删除 `mod offchain;`
- [x] `core/service.rs / core/rpc.rs / desktop/mod.rs / mining/network-overview/mod.rs` use 路径改写（共 17 处）
- [x] 5 个搬迁文件内部 `crate::offchain::common::types::*` → `crate::transaction::offchain_transaction::types::*`
- [x] 内部跨子目录引用 7 处改 `super::super::*`（rpc.rs / settlement/{packer,bootstrap,listener,reserve}.rs）
- [x] 前端 git mv 文件到 `transaction/offchain-transaction/` 平铺结构（6 文件 + settlement/ 子目录）
- [x] `section.tsx / api.ts / types.ts` 内部相对路径补一级 `../→../../`，并把 `'./offchain-transaction/node-register'` 改 `'./node-register'`
- [x] `node-register.tsx` `../api` `../types` → `./api` `./types`（提升后同级）
- [x] 外部 3 文件（`App.tsx / institution-detail.tsx / create-multisig.tsx`）import 路径改写
- [x] `cargo check -p node`（WASM_FILE）通过，仅 5 条与本次无关的旧 dead_code 警告
- [x] `npx tsc --noEmit` 通过（exit 0）
- [x] 残留扫描 4 条全零（crate::offchain / mod offchain / ../offchain / ../../offchain）
