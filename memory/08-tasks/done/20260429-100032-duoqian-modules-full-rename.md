# 任务卡：duoqian 模块全仓库新命名收敛

- 任务编号：20260429-100032
- 状态：done
- 所属模块：citizenchain/runtime/transaction
- 当前负责人：Codex
- 创建时间：2026-04-29 10:00:32

## 任务需求

将 duoqian 管理与转账两个 runtime 模块全仓库收敛到新命名，覆盖代码目录、Cargo 包名、Rust crate 标识、runtime pallet 名、外部端 storage/event 字符串、CI 脚本、AI 上下文脚本、技术文档与任务索引残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/transaction/duoqian-manage/DUOQIAN_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/CROSS_MODULE_INTEGRATION.md

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
- 已完成 runtime 代码目录与 `memory/05-modules` 技术文档目录的新命名。
- 已完成 Cargo workspace、runtime dependency、crate 标识、runtime pallet 名、外部端 storage/event 字符串、CI 脚本与文档引用的新命名。
- 已将 runtime `spec_version` 推进到 10，并同步 wumin 冷钱包支持版本。
- 已清理普通源码与文档中的旧命名残留；生成目录与 `.git` 不纳入源码残留口径。
- 已执行格式化：`cargo fmt --package duoqian-manage --package duoqian-transfer`、`rustfmt sfid/backend/src/indexer/event_parser.rs`、`dart format` 相关 wumin / wuminapp 文件。
- 已验证：`cargo check -p duoqian-manage -p duoqian-transfer --offline` 通过。
- 已验证：`cargo test -p duoqian-manage --lib --offline` 通过，21 个测试全绿。
- 已验证：`cargo test -p duoqian-transfer --lib --offline` 通过，20 个测试全绿。
- 已验证：`cargo check --manifest-path sfid/backend/Cargo.toml --offline` 通过，仅保留既有 unused warnings。
- 已验证：`flutter test test/signer/pallet_registry_test.dart` 通过，8 个测试全绿。
- 补充说明：`SKIP_WASM_BUILD=1 cargo check -p citizenchain --offline` 被 runtime `build.rs` 的统一 WASM 策略阻塞，报错为 `WASM_FILE 环境变量未设置`；该限制来自仓库构建策略，不是本次命名解析错误。
- 复查补充：再次全仓库扫描发现 `citizenchain/scripts/benchmark.sh` 仍有旧 benchmark pallet 标识和旧 weights 路径，已改为 `duoqian_manage` / `duoqian_transfer` 与新路径。
- 复查补充：再次全仓库扫描发现 `citizenchain/target` 与 `sfid/backend/target/doc` 中有旧生成缓存，已通过 `cargo clean -p citizenchain` 与 `cargo clean --manifest-path sfid/backend/Cargo.toml` 清理。
- 复查补充：清理后重新执行旧命名文本扫描和旧命名文件名扫描，普通源码、文档、脚本与生成缓存均未再命中旧名。

## 完成信息

- 完成时间：2026-04-29 10:11:47
- 完成摘要：完成 duoqian 管理与转账模块全仓库新命名收敛，覆盖代码目录、Cargo、runtime pallet 名、外部端、CI、文档、任务索引与残留检查。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
