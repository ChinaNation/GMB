# 统一 Flutter 版本与单账户链上转账入口

## 任务需求

1. 将公民 App 和公民钱包 CI / 本机开发 Flutter 版本统一为 `3.44.4`。
2. 所有外部单账户链上转账统一走 `OnchainTransaction::transfer_with_remark`。
3. `Balances` 只保留为底层余额账本和内部 `Currency` 能力,禁止外部直接调用 `Balances` 普通转账入口。

## 影响范围

- `.fvm/fvm_config.json`
- `.gitignore`
- `.github/workflows/citizenapp-ci.yml`
- `.github/workflows/citizenwallet-ci.yml`
- `citizenchain/runtime/src/configs/mod.rs`
- `citizenchain/runtime/src/tests/cases.rs`
- `memory/05-modules/citizenapp/rpc/RPC_TECHNICAL.md`
- `memory/05-modules/citizenwallet/CITIZENWALLET_PQC_TECHNICAL.md`
- 本任务卡

## 实施步骤

- [x] 创建 Flutter 版本锁文件。
- [x] 固定公民 App / 公民钱包 CI Flutter 版本。
- [x] 禁用外部 `Balances` 普通转账类 extrinsic。
- [x] 更新 runtime 测试,证明 `OnchainTransaction` 是外部单账户转账唯一入口。
- [x] 更新模块文档和任务记录。
- [x] 运行本地验证并清理残留。

## 验收标准

- CI workflow 不再使用浮动 `channel: stable`。
- 仓库有可 Git 跟踪的 Flutter `3.44.4` 版本真源。
- 外部直调 `Balances.transfer_allow_death` / `transfer_keep_alive` / `transfer_all` / `burn` 被 `RuntimeCallFilter` 拒绝。
- `OnchainTransaction::transfer_with_remark` 仍允许并继续内部使用 `Balances` 完成余额转移。
- 公民钱包 signer 测试和 runtime 相关测试通过。

## 验证记录

- `cargo fmt --manifest-path citizenchain/runtime/Cargo.toml --check`：通过。
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml`：通过。
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml --no-default-features`：通过。
- `cargo test --manifest-path citizenchain/runtime/Cargo.toml runtime_fee_kind_classifier_covers_free_onchain_vote_and_unknown_paths`：通过。
- `cargo test --manifest-path citizenchain/runtime/Cargo.toml runtime_call_filter_blocks_external_balances_calls`：通过。
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml`：通过。
- `flutter test test/signer`（公民钱包 signer 测试）：通过。
- `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/citizenapp-ci.yml"); YAML.load_file(".github/workflows/citizenwallet-ci.yml")'`：通过。
- `git diff --check`：通过。
- 本机全局 Flutter 仍是 `3.41.0`，且当前机器未安装 `fvm` 命令；仓库版本真源已锁为 `.fvm/fvm_config.json` 的 `3.44.4`，CI 已改为读取该版本真源。
