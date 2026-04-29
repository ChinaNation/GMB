# 任务卡：修复 wasm CI checkout 失败并将 runtime wasm 版本归零

- 任务编号：20260429-wasm-ci-checkout-and-version-zero
- 状态：done
- 所属模块：citizenchain/runtime、wumin、memory
- 当前负责人：Codex
- 创建时间：2026-04-29

## 任务需求

修复 wasm CI 失败点，检查其他潜在 CI 失败点，并把 runtime wasm 版本号整体归零，支持重新创世。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/requirement-analysis-template.md
- memory/07-ai/thread-model.md
- memory/05-modules/citizenchain/runtime/STEP2B_IV_B_RUNTIME_CLEANUP.md

## 必须遵守

- 系统在开发期，一切按彻底改造进行设计。
- 每次出技术方案都要包含更新文档、完善注释和清理残留。
- 每次执行技术方案后都要更新文档、完善注释、清理残留。

## 实施记录

- 已确认 wasm CI 最新失败点发生在 `actions/checkout@v4`，不是 Rust/WASM 编译阶段。
- 已将超过 Linux 单文件名字节限制的任务卡文件名改为短文件名：
  `20260429-111541-node-src-and-frontend-layout-cleanup.md`。
- 已将 `citizenchain/runtime/src/lib.rs` 中 runtime wasm 版本整体归零：
  `authoring_version`、`spec_version`、`impl_version`、`transaction_version`、`system_version` 均为 `0`。
- 已同步 wumin 冷钱包 `PalletRegistry.supportedSpecVersions = {0}`，避免旧 spec 离线签名请求继续被接受。
- 已更新 runtime 技术文档中的开发期升级策略，改为重新创世策略。
- 已清理版本归零后不准确的 runtime 注释。

## 验证记录

- `git diff --check`：通过。
- 实际存在文件扫描：未发现单文件名超过 240 字节的潜在 Linux checkout 风险。
- 提交后路径扫描：未发现单文件名超过 240 字节或路径超过 3500 字节的潜在 checkout 风险。
- 版本残留扫描：未发现当前代码/当前文档继续写死归零前的旧 runtime 版本号组合。
- `cargo fmt -p citizenchain`：通过。
- `dart format lib/signer/pallet_registry.dart test/signer/pallet_registry_test.dart`：通过，无格式变更。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test -p citizenchain runtime_version_and_block_types_are_sane`：通过。
- `flutter test test/signer/pallet_registry_test.dart test/signer/payload_decoder_test.dart test/signer/offline_sign_service_test.dart`：通过。
- `flutter analyze`：通过。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --release -p citizenchain`：通过，已生成 CI 上传路径下的三份 WASM 文件。

## 完成信息

- 完成时间：2026-04-29
- 完成摘要：修复 wasm CI checkout 阶段的超长文件名失败点，完成 runtime wasm 版本归零与 wumin 冷钱包 spec 同步。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
