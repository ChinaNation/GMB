# 合并 onchain-transaction benches 目录

## 任务需求

检查 `citizenchain/runtime/transaction/onchain-transaction/benches` 的用途,并把其中有效逻辑合并到 `src` 目录下,删除单独的 `benches` 目录残留。

## 影响范围

- `citizenchain/runtime/transaction/onchain-transaction/benches/`
- `citizenchain/runtime/transaction/onchain-transaction/src/`
- `citizenchain/runtime/transaction/onchain-transaction/Cargo.toml`
- `citizenchain/Cargo.lock`
- 相关模块文档
- 本任务卡

## 实施步骤

- [x] 检查 `benches` 文件内容和 Cargo 配置。
- [x] 判断 benchmark 是否仍需要保留为 Cargo bench。
- [x] 将有效断言迁移到 `src` 下已有测试模块。
- [x] 删除 `benches` 目录残留。
- [x] 更新文档和验证记录。

## 验收标准

- `onchain-transaction/benches` 不再作为独立目录存在于 Git 跟踪文件中。
- 被迁移的测试/断言仍能验证手续费路径。
- `onchain-transaction` 测试通过。
- runtime 相关检查通过。

## 验收记录

- `cargo fmt --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml --check`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml`
- `git diff --check`
- `test ! -d citizenchain/runtime/transaction/onchain-transaction/benches`
- `! rg -n 'criterion|transaction_fee_paths|\[\[bench\]\]|benches/transaction_fee_paths' citizenchain/runtime/transaction/onchain-transaction`

以上命令均已通过；`rg` 无匹配表示源码与 Cargo 配置中已无 benchmark/criterion 残留。
