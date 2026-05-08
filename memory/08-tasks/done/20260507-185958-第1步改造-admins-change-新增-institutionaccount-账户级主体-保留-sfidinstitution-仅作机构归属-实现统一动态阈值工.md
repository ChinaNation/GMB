# 任务卡：第1步改造 admins-change：新增 InstitutionAccount 账户级主体；保留 SfidInstitution 仅作机构归属；实现统一动态阈值工具；注册个人账户上限64，注册机构账户上限1989；管理员集合变更提案替代旧等长替换；治理机构仍固定人数和固定阈值。

- 任务编号：20260507-185958
- 状态：done
- 所属模块：citizenchain-runtime
- 当前负责人：Codex
- 创建时间：2026-05-07 18:59:58

## 任务需求

第1步改造 admins-change：新增 InstitutionAccount 账户级主体；保留 SfidInstitution 仅作机构归属；实现统一动态阈值工具；注册个人账户上限64，注册机构账户上限1989；管理员集合变更提案替代旧等长替换；治理机构仍固定人数和固定阈值。

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
- 2026-05-08：已完成 `primitives::derive::SubjectKind::InstitutionAccount = 0x05`、`subject_id_from_institution_account`、`admins-change::dynamic_threshold / derived_threshold`、`propose_admin_set_change`、`AdminSetChangeAction<AdminsOf<T>>`、`MODULE_TAG = b"adm-set-v1"`。
- 2026-05-08：已在 runtime 配置 `MaxPersonalAccountAdmins = 64`、`MaxAdminsPerInstitution = 1989`。
- 2026-05-08：已更新 ADR-010、ADR-015、admins-change 技术文档、MODULE_TAG 注册表、统一协议、统一命名和相关治理/转账文档。
- 2026-05-08：验证通过：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p primitives --lib`：24 passed。
  - `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：34 passed。
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib --no-run`：通过。
  - `cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib --no-run`：通过。
  - `cargo test --manifest-path citizenchain/Cargo.toml -p duoqian-transfer --lib --no-run`：通过。
- 2026-05-08：`cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain` 被 runtime `build.rs` 的 `WASM_FILE` 环境变量强制守门拦截，未进入 runtime 编译；该守门是仓库既有安全规则。

## 完成信息

- 完成时间：2026-05-07 19:12:37
- 完成摘要：完成 admins-change 第1步：0x05 InstitutionAccount、统一动态阈值、管理员集合变更提案、runtime 上限配置、定向测试与文档更新。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
