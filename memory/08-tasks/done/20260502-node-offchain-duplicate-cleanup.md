# node offchain 重复文件清理

- 日期:2026-05-02
- 状态:done
- 完成日期:2026-05-02
- 归属:Blockchain Agent

## 目标

清理 `citizenchain/node` 中 offchain 目录拆分后的重复文件。

## 口径

- 保留新拆分目录:
  - `citizenchain/node/src/offchain/common/`
  - `citizenchain/node/src/offchain/duoqian_manage/`
  - `citizenchain/node/src/offchain/duoqian_transfer/`
  - `citizenchain/node/src/offchain/offchain_transaction/`
  - `citizenchain/node/src/offchain/settlement/`
  - `citizenchain/node/frontend/offchain/duoqian-manage/`
  - `citizenchain/node/frontend/offchain/duoqian-transfer/`
  - `citizenchain/node/frontend/offchain/offchain-transaction/`
  - `citizenchain/node/frontend/offchain/settlement/`
- 删除旧根层重复文件。
- 不删除仍作为共享入口的 `api.ts`、`types.ts`、`styles.css`、`section.tsx` 等文件。

## 验收

- 旧根层重复 Rust 文件删除。
- 旧根层重复 TSX 文件删除。
- `rg` 不再引用已删除文件名作为模块路径。
- 保留的新目录文件仍存在。

## 完成记录

- 已删除 `citizenchain/node/src/offchain` 根层旧重复 Rust 文件:
  `bootstrap.rs`、`chain.rs`、`commands.rs`、`decrypt.rs`、`health.rs`、`keystore.rs`、
  `ledger.rs`、`reserve.rs`、`rpc.rs`、`sfid.rs`、`signing.rs`、`types.rs`。
- 已删除 `citizenchain/node/frontend/offchain` 根层旧重复 TSX 文件:
  `admin_list.tsx`、`create_multisig.tsx`、`institution_detail.tsx`、`other_accounts.tsx`、
  `register.tsx`、`sfid.tsx`。
- 已把 Rust 入口重接到 `duoqian_manage`、`offchain_transaction`、`settlement` 新目录。
- 已把前端 `section.tsx` 重接到 `duoqian-manage`、`offchain-transaction`、`settlement` 新目录。
- 已更新 `NODE_CLEARING_BANK_TECHNICAL.md` 与 `ADR-007` 当前目录说明。

## 验证

- `npm run build` (`citizenchain/node/frontend`) 通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check -p node --bin citizenchain` 通过。
- 构建产生的 `frontend/dist` 残留已清理。
