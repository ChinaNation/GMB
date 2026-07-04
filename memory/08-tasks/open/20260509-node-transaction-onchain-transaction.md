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
- 当时普通转账调用、QR 签名、fee 计算（`onchain_transaction::calculate_onchain_fee`）、wallet store 持久化全部零业务改动；2026-07-04 后续变更已把普通转账入口切到带备注的新链上调用。
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

## 2026-07-04 后续变更：普通转账支持链上备注

- 普通链上转账入口改为 `OnchainTransaction::transfer_with_remark`，call data 为 `[pallet=4][call=0][beneficiary:AccountId32][amount:u128_le][remark:BoundedVec<u8>]`。
- 转账备注按 UTF-8 字节计数，链上、node、CitizenApp、CitizenWallet、桌面前端统一限制为 99 字节。
- node 冷钱包二维码签名请求、矿工热钱包 `transaction_submitMinerTransfer`、CitizenApp 热钱包/冷钱包签名、CitizenWallet payload 解码全部同步新动作码 `0x0400`。
- CitizenApp 本地流水新增 `remark` 字段；本机 pending 与区块 `TransferWithRemark` 事件合并时保留非空备注，交易详情页展示并支持复制。
- 旧 Balances 直转调用不再作为官方普通转账路径，文档和扫码动作码已同步清理。

验证记录：

- `cargo test -p onchain-transaction transfer_with_remark --features std`：2 项通过。
- `cargo test -p citizenchain runtime_fee_kind_classifier --features std`：2 项通过。
- `cargo test -p node transfer --features std`：5 项通过。
- `flutter test test/transaction/local_tx_store_status_test.dart test/qr/qr_router_test.dart test/qr/qr_sign_session_test.dart test/signer/qr_signer_test.dart`（CitizenApp）：26 项通过。
- `flutter test test/signer/payload_decoder_test.dart test/signer/offline_sign_service_test.dart test/signer/qr_signer_test.dart`（CitizenWallet）：92 项通过。
- `npm run build`（citizenchain node frontend）：通过，构建同步生成 `generated/local-docs.generated.ts`。
- 旧入口残留扫描：目标源码/文档范围内旧直转调用名、旧 Dart action 名和旧动作码均无命中。
