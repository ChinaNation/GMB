# 任务卡：修复 voting-engine internal_vote provider fallback threshold provider 默认实现 active proposal 上限硬编码与主体合法性语义问题

- 任务编号：20260430-215740
- 状态：done
- 所属模块：citizenchain-runtime-governance
- 当前负责人：Codex
- 创建时间：2026-04-30 21:57:40

## 任务需求

修复 voting-engine internal_vote provider fallback threshold provider 默认实现 active proposal 上限硬编码与主体合法性语义问题

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
- 已删除 `internal_vote::is_internal_admin` 在 `#[cfg(test)]` 下回退常量管理员的分支，测试侧改由 mock provider 显式注入管理员来源。
- 已为 `InternalThresholdProvider` 增加 `is_known_subject` / `is_known_pending_subject` 显式主体存在性 API，`ORG_DUOQIAN` 机构合法性不再通过 `pass_threshold(...).is_some()` 间接判断。
- 已将 `InternalThresholdProvider for ()` 改为不提供默认阈值，runtime 与 mock runtime 均显式注入阈值 provider。
- 已将 active proposal 每机构上限改为 `Config::MaxActiveProposals`，生产 runtime 当前配置为 `ConstU32<10>`。
- 已在 `admins-change` 暴露 `active_subject_exists` / `pending_subject_exists_for_snapshot`，供 runtime provider 区分 Active 与 Pending 主体合法性。
- 已同步更新 voting-engine、admins-change 与 wuminapp governance 技术文档。
- 验证通过：`cargo fmt`。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/governance/voting-engine`（73 passed）。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/governance/admins-change`（26 passed）。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/transaction/duoqian-manage`（21 passed）。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/transaction/duoqian-transfer`（20 passed）。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/governance/runtime-upgrade`（16 passed）。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/governance/resolution-destro`（14 passed）。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/governance/grandpakey-change`（15 passed）。
- 验证通过：`cargo test --lib` in `citizenchain/runtime/issuance/resolution-issuance`（12 passed）。
- 源码检索确认：不存在旧 `MAX_ACTIVE_PROPOSALS` 常量引用，不存在测试侧 `InternalThresholdProvider = ()` 配置，不存在旧 `pass_threshold(...).is_some()` 合法性判断残留。
- `cargo check --lib` in `citizenchain/runtime` 被 `runtime/build.rs` 按仓库策略阻断：未设置 `WASM_FILE`，未进入代码编译错误阶段。

## 完成信息

- 完成时间：2026-04-30 22:17:36
- 完成摘要：修复 internal_vote 测试回退、主体合法性显式 API、阈值 provider 默认实现和 active proposal 上限硬编码，并同步测试与文档。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
