# 任务卡：将 resolution-issuance-gov 与 resolution-issuance-iss 彻底合并为 resolution-issuance 模块，保留 runtime pallet index 8，并完成代码、文档、联动端与残留清理

- 任务编号：20260429-084207
- 状态：done
- 所属模块：citizenchain/runtime/issuance/resolution-issuance
- 当前负责人：Codex
- 创建时间：2026-04-29 08:42:07

## 任务需求

将 resolution-issuance-gov 与 resolution-issuance-iss 彻底合并为 resolution-issuance 模块，保留 runtime pallet index 8，并完成代码、文档、联动端与残留清理

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已将 `resolution-issuance-gov` 与 `resolution-issuance-iss` 合并为 `citizenchain/runtime/issuance/resolution-issuance`
- 已保留 runtime pallet index 8，并删除原 index 7 的执行模块注册
- 已将提案、联合投票回调、发行执行、暂停维护、幂等清理拆分到 `proposal.rs`、`execution.rs`、`validation.rs`、`migration.rs`、`weights.rs`、`benchmarks.rs`
- 已更新 runtime 配置、创世 JSON key、benchmark 脚本、SFID 事件解析、冷钱包 pallet registry、网站模块说明与技术文档
- 已删除旧代码目录和旧模块技术文档目录，并完成旧模块名残留搜索
- 已验证 `cargo test -p resolution-issuance`、`cargo check -p resolution-issuance --features runtime-benchmarks`、`CARGO_TARGET_DIR=/private/tmp/gmb-citizenchain-target WASM_BUILD_WORKSPACE_HINT=/Users/rhett/GMB/citizenchain WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain`、`flutter test test/signer/pallet_registry_test.dart`

## 验收备注

- `WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain --features runtime-benchmarks` 当前在 wasm 子构建中触发上游 `byte-slice-cast`/`parity-scale-codec` 的 `std` 特性问题；改用 `SKIP_WASM_BUILD=1 WASM_BUILD_FROM_SOURCE=1` 后，继续卡在既有 `offchain-transaction-pos` 与 `duoqian-manage-pow` benchmark 签名问题。新模块自身的 benchmark 默认特性编译已通过。
