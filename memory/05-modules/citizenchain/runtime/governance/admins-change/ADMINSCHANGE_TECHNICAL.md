# ADMINS_ORIGIN_GOV Technical Notes

最新更新：2026-04-30，新增管理员更换互斥、执行失败终态、创建事务回滚与回调最终事件收口。

## 1. 模块定位

`admins-change` 是链上管理员主体的统一真源，同时负责管理员等长替换治理。

统一纳入的主体：

- 国储会、各省储会、各省储行等创世内置机构。
- SFID 机构在链上注册后的机构多签主体。
- 用户创建的个人多签主体。

代码位置：

- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-change/src/lib.rs`

## 2. 存储模型

`STORAGE_VERSION = 2`。

核心存储：

```text
Institutions<InstitutionPalletId, AdminInstitution>
```

`AdminInstitution` 字段：

- `org`：内部投票组织类型，含 `ORG_NRC / ORG_PRC / ORG_PRB / ORG_DUOQIAN`。
- `kind`：`BuiltinInstitution / SfidInstitution / PersonalDuoqian`。
- `admins`：当前管理员列表。
- `threshold`：内部投票通过阈值。
- `creator`：主体创建者。
- `created_at / updated_at`：生命周期时间。
- `status`：`Pending / Active / Closed`。

创世构建：

- 从 `CHINA_CB` 写入国储会、省储会主体。
- 从 `CHINA_CH` 写入省储行主体。
- 创世主体均为 `BuiltinInstitution + Active`。

## 3. 生命周期

- `Pending`：多签创建提案已发起，投票引擎可以锁定管理员快照。
- `Active`：创建提案通过并执行成功，主体可继续发起转账、清算、管理员替换等内部投票。
- `Closed`：主体已关闭，管理员不再有效。

`duoqian-manage` 在创建机构多签或个人多签时调用：

- `create_pending_subject`
- `activate_subject`
- `remove_pending_subject`
- `close_subject`

## 4. 管理员读取 API

管理员读取 API 按主体状态拆成两组，避免普通业务把 `Pending` 主体误当成可执行主体。

Active-only 公共业务 API：

- `is_active_subject_admin(org, institution, who)`
- `active_subject_admins(org, institution)`
- `active_subject_threshold(org, institution)`
- `active_subject_admin_count(org, institution)`

Pending 快照专用 API：

- `is_pending_subject_admin_for_snapshot(org, institution, who)`
- `pending_subject_admins_for_snapshot(org, institution)`
- `pending_subject_threshold_for_snapshot(org, institution)`
- `pending_subject_admin_count_for_snapshot(org, institution)`

规则：

- 普通业务授权、普通内部提案创建和长期管理员真源读取只能使用 Active-only API。
- Pending 快照 API 仅供投票引擎的 Pending 主体创建提案入口使用，用于创建/激活该主体时锁定管理员和阈值快照。
- `Closed` 主体不返回管理员、阈值或人数。
- `BuiltinInstitution` 只允许 `ORG_NRC / ORG_PRC / ORG_PRB`。
- `SfidInstitution / PersonalDuoqian` 只允许 `ORG_DUOQIAN`。

## 5. 管理员替换流程

`propose_admin_replacement(org, institution, old_admin, new_admin)`：

1. 读取 `Institutions[institution]`。
2. 校验主体为 `Active` 且 `subject.org == org`。
3. 校验发起人是当前管理员。
4. 校验 `old_admin` 存在、`new_admin` 不存在。
5. 在同一个 `with_transaction` 中调 `voting-engine` 的管理员集合变更内部提案入口创建投票（只接受 Active 主体，并申请同主体独占锁）。
6. 在同一事务中将 `AdminReplacementAction` 写入投票引擎 `ProposalData`，写入 `ProposalMeta`，并发出 `AdminReplacementProposed`。

创建事务语义：

- 投票引擎提案、互斥锁、管理员快照、阈值快照、业务数据、业务元数据和业务事件必须全部成功才提交。
- 任一步失败都会整体回滚，避免留下无 `ProposalData` 的管理员更换提案或独占锁。

投票通过后由 `InternalVoteExecutor` 自动执行；回调成功返回 `ProposalExecutionOutcome::Executed`，失败返回 `ProposalExecutionOutcome::FatalFailed`。最终 `ProposalFinalized` 事件、清理登记和互斥锁释放统一由投票引擎外层发出/处理一次。

执行规则：

- 执行前必须校验投票引擎提案元数据与业务动作一致：
  - `proposal.kind == PROPOSAL_KIND_INTERNAL`
  - `proposal.stage == STAGE_INTERNAL`
  - `proposal.internal_institution == Some(action.institution)`
  - `proposal.internal_org == Some(subject.org)`
- 执行前必须校验当前 `proposal_id` 仍然持有该 `(org, institution)` 的管理员集合变更独占锁。
- 只有 `proposal.status == STATUS_PASSED` 的一致提案可以执行。
- 只允许等长替换，不增删管理员人数。
- 内置机构仍校验固定人数：国储会 19，省储会 9，省储行 9。
- 动态多签主体校验人数在 `2..=MaxAdminsPerInstitution`。
- 执行成功后写回 `Institutions[institution].admins` 并更新 `updated_at`。

互斥规则：

- 同一治理主体下，有管理员更换提案处于活跃/已通过但未终态时，不能创建普通内部提案。
- 同一治理主体下，有普通内部提案活跃时，不能创建管理员更换提案。
- 普通内部提案之间默认不互斥。
- 不同治理主体互不影响。
- 自动执行失败映射为 `FatalFailed`，进入 `STATUS_EXECUTION_FAILED` 后释放独占锁，且该提案不能再手动执行。

## 6. 运行时接线

`/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs` 中：

- `RuntimeInternalAdminProvider` 普通路径从 `is_active_subject_admin / active_subject_admins` 读取。
- `RuntimeInternalAdminProvider` Pending 快照路径从 `is_pending_subject_admin_for_snapshot / pending_subject_admins_for_snapshot` 读取。
- `RuntimeInternalThresholdProvider` 普通路径从 `active_subject_threshold` 读取。
- `RuntimeInternalThresholdProvider` Pending 快照路径从 `pending_subject_threshold_for_snapshot` 读取。
- `RuntimeInternalAdminCountProvider` 从 `active_subject_admin_count` 读取。
- `EnsureNrcAdmin` 与联合治理发起人校验也从统一主体表读取。

`duoqian-manage` 不再作为管理员长期真源；它只保留账户、资金和生命周期 storage。

## 7. 事件

- `AdminReplacementProposed`
- `AdminReplacementExecutionFailed`
- `AdminReplaced`
- `AdminSubjectPendingCreated`
- `AdminSubjectActivated`
- `AdminSubjectPendingRemoved`
- `AdminSubjectClosed`

## 8. 测试

运行命令：

```bash
cargo test -p admins-change --lib
```

当前结果：

- 24 passed。

覆盖重点：

- NRC/PRC/PRB 管理员替换。
- org 与 institution 不匹配拒绝。
- 自动执行失败后进入 `STATUS_EXECUTION_FAILED`，释放独占锁，且不能手动重试。
- 替换后新管理员可继续发起提案。
- 无效机构、旧管理员缺失、新管理员已存在等错误路径。
- Pending 主体不会暴露给 Active 业务 API，但可通过 Pending 快照 API 读取。
- 执行路径拒绝 kind / stage / org / institution 与 `AdminReplacementAction` 不一致的提案。
- 执行路径要求提案仍是管理员集合变更独占锁 owner。
- 同主体普通内部提案活跃时，管理员更换业务入口会被投票引擎互斥规则拒绝。
- 成功自动执行和失败自动执行都只产生一次投票引擎 `ProposalFinalized` 最终事件。
