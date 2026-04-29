# 任务卡：治理销毁与运行时升级模块统一新名称

- 任务编号：20260429-082345
- 状态：done
- 所属模块：citizenchain/runtime/governance
- 当前负责人：Codex
- 创建时间：2026-04-29 08:23:45

## 任务需求

治理销毁模块统一使用 `resolution-destro` / `resolution_destro` / `ResolutionDestro`。
运行时升级模块统一使用 `runtime-upgrade` / `runtime_upgrade` / `RuntimeUpgrade`。
两个模块仍保留在 `citizenchain/runtime/governance/` 下，不保留兼容别名，代码、文档、脚本和索引统一使用新模块名称。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/resolution-destro/RESOLUTIONDESTRO_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/runtime-upgrade/RUNTIMEUPGRADE_TECHNICAL.md

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
- 已确认本轮不保留旧模块兼容名，runtime pallet 名、crate 名、目录名、文档和脚本统一切换到新名称。
- 已完成源码目录与文档目录迁移：
  - `citizenchain/runtime/governance/resolution-destro`
  - `citizenchain/runtime/governance/runtime-upgrade`
  - `memory/05-modules/citizenchain/runtime/governance/resolution-destro`
  - `memory/05-modules/citizenchain/runtime/governance/runtime-upgrade`
- 已完成全仓旧模块名残留扫描，旧模块名、旧 crate 标识、旧 runtime pallet 类型名、旧 Dart camelCase 常量名、旧技术文档文件名均为 0 命中。
- 已将 runtime `spec_version` 更新到 6；pallet index、call index、MODULE_TAG 保持不变。
- 已更新 `runtime-upgrade` 与 `resolution-destro` 技术文档、跨模块矩阵、模块标签注册表、上下文加载脚本、冷钱包注册表和 SFID 事件解析口径。
- 验证通过：
  - `cargo check --offline --manifest-path citizenchain/Cargo.toml -p resolution-destro -p runtime-upgrade`
  - `cargo test --offline --manifest-path citizenchain/runtime/governance/resolution-destro/Cargo.toml`
  - `cargo test --offline --manifest-path citizenchain/runtime/governance/runtime-upgrade/Cargo.toml`
  - `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo test --offline --manifest-path citizenchain/Cargo.toml -p citizenchain governance_module_tags_are_globally_unique`
  - `flutter test test/signer/pallet_registry_test.dart`（在 `wumin/` 下）
- 2026-04-29 复查补充：
  - 已修复额外 runtime 单测编译挡板：`duoqian_manage_pow::propose_create` 测试构造补 `account_name`，`verify_institution_registration` 测试调用补 `signing_province = None`。
  - 已补 `citizenchain/scripts/benchmark.sh` 中两个旧 benchmark 映射残留，统一为 `resolution_destro` / `runtime_upgrade`。
  - 主工作树以 `rg --hidden --no-ignore` 复扫旧模块名、旧 crate 标识、旧 runtime pallet 类型名、旧 Dart camelCase 常量名、旧技术文档文件名，均为 0 命中；`.claude/` 为 gitignore 忽略的本地工具目录，未纳入可提交源码/文档改名范围。
- 外部验证受阻（非本次两个模块改名范围）：
  - `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo check --offline --manifest-path citizenchain/Cargo.toml -p citizenchain`
  - 当前被 `runtime/issuance/resolution-issuance` 既有未完成改动挡住：`DispatchError` 导入私有、`Get` trait 未导入、`JointVoteEngine` trait 未导入。

## 完成信息

- 完成时间：2026-04-29 08:33:50
- 完成摘要：完成治理销毁与运行时升级两个 governance 模块的新名称统一：resolution-destro / runtime-upgrade；旧模块名残留扫描为 0；相关 Rust、文档、脚本、SFID、wumin 冷钱包口径已同步；模块单测、runtime 模块标签单测与冷钱包注册表测试通过；整 runtime cargo check 当前被 `resolution-issuance` 既有未完成改动阻塞，非本次两个模块改名引起。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
