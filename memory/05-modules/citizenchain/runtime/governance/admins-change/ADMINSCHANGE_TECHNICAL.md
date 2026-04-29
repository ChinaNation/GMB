# ADMINS_ORIGIN_GOV Technical Notes

最新更新：2026-04-29，第2步统一管理员真源改造。

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

`duoqian-manage-pow` 在创建机构多签或个人多签时调用：

- `create_pending_subject`
- `activate_subject`
- `remove_pending_subject`
- `close_subject`

## 4. 管理员读取 API

生产 runtime 的内部投票 provider 统一调用：

- `is_subject_admin(org, institution, who)`
- `subject_admins(org, institution)`
- `subject_threshold(org, institution)`
- `subject_admin_count(org, institution)`

规则：

- `Closed` 主体不返回管理员、阈值或人数。
- `Pending` 主体可被投票引擎读取，用于创建提案后的管理员快照。
- `BuiltinInstitution` 只允许 `ORG_NRC / ORG_PRC / ORG_PRB`。
- `SfidInstitution / PersonalDuoqian` 只允许 `ORG_DUOQIAN`。

## 5. 管理员替换流程

`propose_admin_replacement(org, institution, old_admin, new_admin)`：

1. 读取 `Institutions[institution]`。
2. 校验主体为 `Active` 且 `subject.org == org`。
3. 校验发起人是当前管理员。
4. 校验 `old_admin` 存在、`new_admin` 不存在。
5. 调 `voting-engine` 创建内部投票。
6. 将 `AdminReplacementAction` 写入投票引擎 `ProposalData`。

投票通过后由 `InternalVoteExecutor` 自动执行；自动执行失败时可由任意签名账户调用 `execute_admin_replacement(proposal_id)` 重试。

执行规则：

- 只允许等长替换，不增删管理员人数。
- 内置机构仍校验固定人数：国储会 19，省储会 9，省储行 9。
- 动态多签主体校验人数在 `2..=MaxAdminsPerInstitution`。
- 执行成功后写回 `Institutions[institution].admins` 并更新 `updated_at`。

## 6. 运行时接线

`/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs` 中：

- `RuntimeInternalAdminProvider` 从 `admins-change::Pallet::is_subject_admin` 读取。
- `RuntimeInternalThresholdProvider` 从 `subject_threshold` 读取。
- `RuntimeInternalAdminCountProvider` 从 `subject_admin_count` 读取。
- `EnsureNrcAdmin` 与联合治理发起人校验也从统一主体表读取。

`duoqian-manage-pow` 不再作为管理员长期真源；它只保留账户、资金和生命周期 storage。

## 7. 事件

- `AdminReplacementProposed`
- `AdminReplacementVoteSubmitted`
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

- 20 passed。

覆盖重点：

- NRC/PRC/PRB 管理员替换。
- org 与 institution 不匹配拒绝。
- 自动执行失败后手动恢复。
- 替换后新管理员可继续发起提案。
- 无效机构、旧管理员缺失、新管理员已存在等错误路径。
