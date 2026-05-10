# 2026-05-09 node 端 transaction 目录整合 Step 1：duoqian_transfer

## 任务需求

把 node 端 duoqian_transfer 模块从顶层下沉到 `transaction/` 目录，前后端同步对齐 runtime/transaction/ 边界。本次只动 duoqian_transfer，offchain 与 onchain 留待后续 Step 2/3。

- 后端：`node/src/duoqian_transfer/` → `node/src/transaction/duoqian_transfer/`
- 前端：`node/frontend/duoqian-transfer/` → `node/frontend/transaction/duoqian-transfer/`
- `node/src/transaction/mod.rs` 仅做 `pub mod duoqian_transfer;` 壳子，不预先声明 offchain / onchain 占位（避免半完成实现）

## 影响范围

- `citizenchain/node/src/duoqian_transfer/`（搬迁源）
- `citizenchain/node/src/transaction/`（新建壳目录 + 落地）
- `citizenchain/node/src/main.rs`
- `citizenchain/node/src/desktop/mod.rs`
- `citizenchain/node/src/shared/proposal_business.rs`
- `citizenchain/node/frontend/duoqian-transfer/`（搬迁源）
- `citizenchain/node/frontend/transaction/duoqian-transfer/`（落地）
- `citizenchain/node/frontend/governance/ProposalDetailPage.tsx`
- `citizenchain/node/frontend/governance/NrcSection.tsx`
- `citizenchain/node/frontend/governance/PrcSection.tsx`
- `citizenchain/node/frontend/governance/PrbSection.tsx`
- `citizenchain/node/frontend/governance/types.ts`

## 风险点

- 6 个 Tauri invoke 命令名串必须保持不变，只动目录路径不动 ABI。
- 文件内部代码、签名逻辑、提案解码逻辑零改动，纯路径重构。
- runtime 侧零改动，MODULE_TAG b"duoqian-tr" 等常量零改动。
- `cargo check` 与前端 `tsc --noEmit` 均需通过。

## 执行状态

- [x] 后端文件 git mv 到 `node/src/transaction/duoqian_transfer/`
- [x] 创建 `node/src/transaction/mod.rs`（壳子，仅 `pub mod duoqian_transfer;`）
- [x] 更新 `main.rs / desktop/mod.rs / shared/proposal_business.rs` 三处 use 路径
- [x] 前端文件 git mv 到 `node/frontend/transaction/duoqian-transfer/`
- [x] 更新 governance 5 个 .tsx/.ts 文件 import 路径
- [x] 修复 6 个搬迁文件的 `../core/` `../shared/` 上溯路径 → `../../core/` `../../shared/`
- [x] `cargo check -p node`（WASM_FILE 走 target/wasm 既有产物）通过，仅 5 条与本次无关的 dead_code 警告
- [x] 前端 `npx tsc --noEmit` 通过
- [x] 残留扫描：`grep -rn "crate::duoqian_transfer\|from '../duoqian-transfer" citizenchain/node/` 零命中
