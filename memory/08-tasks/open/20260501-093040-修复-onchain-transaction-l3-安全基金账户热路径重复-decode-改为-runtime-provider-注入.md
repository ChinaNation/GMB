# 任务卡：修复 onchain-transaction L3：安全基金账户热路径重复 decode 改为 runtime provider 注入

- 任务编号：20260501-093040
- 状态：done
- 所属模块：citizenchain/runtime/transaction/onchain-transaction
- 当前负责人：Codex
- 创建时间：2026-05-01 09:30:40

## 任务需求

修复 onchain-transaction L3：`OnchainFeeRouter` 在手续费分账热路径中每次 decode `NRC_ANQUAN_ADDRESS`，改为由 runtime 注入安全基金账户 provider，完成后更新文档、完善注释、清理残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/CROSS_MODULE_INTEGRATION.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有手续费分账契约
- 不可改变 80% 全节点 / 10% 国储会 / 10% 安全基金分账比例
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理
- 验证记录

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已新增 `SafetyFundAccountProvider<AccountId>`，安全基金账户改由 runtime 注入。
- 已将 `OnchainFeeRouter` 泛型扩展为 `NrcProvider + SafetyFundProvider`，移除热路径中的 `T::AccountId::decode(&NRC_ANQUAN_ADDRESS[..])`。
- 已在 runtime 接线中新增 `RuntimeSafetyFundAccountProvider`，并同步 `pallet_transaction_payment::Config::OnChargeTransaction` 与 `TransferFeeRouter` 两条入口。
- 已同步测试与 benchmark mock provider。
- 已清理安全基金 decode 失败事件分支及相关文档残留。
- 已更新 `ONCHAIN_TECHNICAL.md` 与 `CROSS_MODULE_INTEGRATION.md`。

## 验证记录

- `rustfmt citizenchain/runtime/transaction/onchain-transaction/src/lib.rs citizenchain/runtime/src/configs/mod.rs citizenchain/runtime/transaction/onchain-transaction/benches/transaction_fee_paths.rs`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml --features runtime-benchmarks`
- `cargo test --manifest-path citizenchain/runtime/transaction/onchain-transaction/Cargo.toml --all-targets`
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml`：被仓库 `runtime/build.rs` 的 `WASM_FILE` 统一 WASM 门禁阻断。
- `WASM_FILE=/private/tmp/dummy_wasm.wasm cargo check --manifest-path citizenchain/runtime/Cargo.toml`
