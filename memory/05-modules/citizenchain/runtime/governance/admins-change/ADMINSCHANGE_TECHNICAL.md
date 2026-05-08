# ADMINS_CHANGE Technical Notes

最新更新：2026-05-08，生命周期清理和事件协议已收口：Pending 主体不存在时不再静默成功，生命周期事件统一携带 `org`。

## 1. 模块定位

`admins-change` 是链上内部投票管理员主体的统一真源。

当前职责：

- 维护 `Subjects<SubjectId, AdminSubject>`。
- 为治理机构写入创世固定管理员集合。
- 为个人账户、机构账户提供 Pending / Active / Closed 生命周期写入口。
- 提供统一动态阈值工具。
- 创建并执行管理员集合变更提案。

账户级口径来自 `memory/04-decisions/ADR-015-account-admin-internal-vote.md`：

- 省储行质押账户永远不可操作，不进入内部投票主体。
- 治理机构全部可操作账户共享固定管理员集合和固定阈值，只允许等长更换。
- 注册个人账户独立持有管理员集合，管理员数量范围为 `2..=64`。
- 注册机构账户独立持有管理员集合，管理员数量范围为 `2..=1989`。
- 动态账户阈值按管理员数量派生：`2 -> 2`，`>=3 -> ceil(admin_count / 2)`。
- 创建和注销提案使用全员阈值，由投票引擎快照保存，不改变 `Subjects.threshold` 的动态规则。

代码位置：

- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-change/src/lib.rs`
- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-change/src/weights.rs`
- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-change/src/benchmarks.rs`

## 2. SubjectKind

`AdminSubjectKind` 当前值：

- `BuiltinInstitution`：NRC / PRC / PRB 等内置治理主体。
- `SfidInstitution`：保留给过渡期机构级主体和 SFID 归属索引语义；后续机构账户级改造完成后，不应再作为新增账户管理员主体。
- `PersonalDuoqian`：注册个人账户主体。
- `InstitutionAccount`：注册机构某个具体账户的账户级主体。

对应底层 `SubjectId` 协议见 ADR-010：

- `0x01 Builtin`
- `0x02 SfidInstitution`，仅表示同一 SFID 机构归属/检索。
- `0x03 PersonalDuoqian`
- `0x04 OnchainAsset`
- `0x05 InstitutionAccount`，payload 为账户 `AccountId` 前 32 字节并右填零。

## 3. 存储模型

`STORAGE_VERSION = 2`(链未启动+即将重新创世,旧 v1→v2 `Institutions`→`Subjects`
move_prefix migration 已于 2026-05-08 删除,fresh genesis 直接写 `Subjects`)。

核心存储：

```text
Subjects<SubjectId, AdminSubject>
```

`AdminSubject` 字段：

- `org`：内部投票组织类型，含 `ORG_NRC / ORG_PRC / ORG_PRB / ORG_REN`。
- `kind`：管理员主体类型。
- `admins`：当前管理员完整列表。
- `threshold`：当前普通业务阈值。动态账户由 `derived_threshold` 计算后写入，不能由用户自由指定。
- `creator`：主体创建者。
- `created_at / updated_at`：生命周期时间。
- `status`：`Pending / Active / Closed`。

创世构建：

- 从 `CHINA_CB` 写入国储会、省储会主体。
- 从 `CHINA_CH` 写入省储行主体。
- 创世主体均为 `BuiltinInstitution + Active`。

## 4. 阈值工具

统一工具函数：

- `dynamic_threshold(admin_count)`
- `derived_threshold(kind, org, admin_count)`

规则：

- `admin_count < 2` 返回 `None`。
- `admin_count == 2` 返回 `Some(2)`。
- `admin_count >= 3` 返回 `Some(ceil(admin_count / 2))`。
- `BuiltinInstitution` 只接受固定管理员数量，并返回固定制度阈值。

写入规则：

- `do_create_pending_subject` 不接收外部阈值，链上写入时统一调用 `derived_threshold`。
- `SubjectLifecycle::create_pending_subject_for_proposal` 不接收外部阈值，调用方只能传入管理员集合。
- 管理员集合变更执行成功后，按新管理员数量重新推导并写回 `threshold`。
- 创建/注销的全员阈值是投票提案快照语义，由业务模块调用投票引擎时显式传入，不写成长期普通阈值。
- 第 2 步后，投票引擎会拒绝与管理员快照人数不匹配的全员生命周期阈值，避免业务模块误传普通动态阈值完成注册创建或注销关闭。

## 5. 管理员集合校验

统一校验入口：

```text
validate_admin_set_for_subject(kind, org, admins)
```

校验内容：

- 主体类型与 `org` 匹配。
- 管理员列表不能重复。
- `BuiltinInstitution` 必须等于固定人数：NRC 19、PRC 9、PRB 9。
- `PersonalDuoqian` 管理员数量必须在 `2..=MaxPersonalAccountAdmins`。
- `InstitutionAccount` 管理员数量必须在 `2..=MaxAdminsPerInstitution`。
- 当前 runtime 配置：`MaxPersonalAccountAdmins = 64`，`MaxAdminsPerInstitution = 1989`。

## 6. 生命周期

跨 pallet 生命周期写入口统一走 `SubjectLifecycle` trait：

- `create_pending_subject_for_proposal`
- `activate_subject_for_proposal`
- `remove_pending_subject_for_proposal`
- `close_subject_for_proposal`

约束：

- 所有调用必须校验 `proposal_id + module_tag + subject` 的 votingengine owner 与提案上下文。
- `create_pending_subject_for_proposal` 要求提案为 `PROPOSAL_KIND_INTERNAL / STAGE_INTERNAL / STATUS_VOTING`。
- `activate_subject_for_proposal` 与 `close_subject_for_proposal` 要求提案为 `STATUS_PASSED`，且处于 votingengine callback 执行作用域。
- `remove_pending_subject_for_proposal` 仅接受 `STATUS_REJECTED / STATUS_EXECUTION_FAILED`。
- `do_remove_pending_subject` 要求主体必须存在且处于 `Pending`；不存在返回 `InvalidInstitution`，非 Pending 返回 `SubjectNotPending`。
- `BuiltinInstitution` 永远不能关闭。

## 7. 读取 API

Active-only 公共业务 API：

- `is_active_subject_admin(org, subject, who)`
- `active_subject_exists(org, subject)`
- `active_subject_admins(org, subject)`
- `active_subject_threshold(org, subject)`
- `active_subject_admin_count(org, subject)`

Pending 快照专用 API：

- `is_pending_subject_admin_for_snapshot(org, subject, who)`
- `pending_subject_exists_for_snapshot(org, subject)`
- `pending_subject_admins_for_snapshot(org, subject)`
- `pending_subject_threshold_for_snapshot(org, subject)`
- `pending_subject_admin_count_for_snapshot(org, subject)`

规则：

- 普通业务授权、普通内部提案创建和长期管理员真源读取只能使用 Active-only API。
- Pending 快照 API 只供创建/激活该主体时锁定管理员和阈值快照。
- `Closed` 主体不返回管理员、阈值或人数。

## 8. 管理员集合变更流程

入口：

```text
propose_admin_set_change(org, subject, new_admins)
```

语义：

- 输入完整目标管理员集合。
- 链端对比当前集合与目标集合，不再拆分增加、删除、更换、改阈值四类提案。
- 发起人必须是当前 Active 主体管理员。
- 目标集合必须通过统一管理员集合校验。
- 目标集合不能与当前集合完全一致。
- 在同一事务内调用 `create_admin_set_mutation_internal_proposal_with_data` 创建提案、写入 owner/data，并登记同主体管理员集合变更独占锁。

提案数据：

- `ProposalOwner = b"adm-set-v1"`。
- `ProposalData = AdminSetChangeAction<AdminsOf<T>>(SCALE)`。
- `AdminSetChangeAction.subject` 是目标主体。
- `AdminSetChangeAction.new_admins` 是完整目标管理员集合。

执行：

- `InternalVoteExecutor` 通过 `ProposalOwner` 认领 `adm-set-v1`。
- 提案通过后解码 `AdminSetChangeAction`。
- 执行前校验提案 kind/stage/status、`internal_institution`、`internal_org`、独占锁 owner。
- 再次校验目标管理员集合。
- 写回 `Subjects[subject].admins`。
- 按新管理员数量推导并写回 `Subjects[subject].threshold`。
- 更新 `updated_at`。

失败：

- 执行失败发出 `AdminSetChangeExecutionFailed`。
- callback 返回 `ProposalExecutionOutcome::FatalFailed`，投票引擎进入 `STATUS_EXECUTION_FAILED` 并释放互斥锁。

## 9. Runtime 接线

`/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`：

- `admins_change::Config::MaxAdminsPerInstitution = 1989`。
- `admins_change::Config::MaxPersonalAccountAdmins = 64`。
- `RuntimeInternalAdminProvider` 统一读取 `admins-change` Active / Pending API。
- `RuntimeInternalThresholdProvider` 对治理机构返回固定制度阈值，对 `ORG_REN` 返回 `admins-change` 中的 Active / Pending 阈值。
- `RuntimeInternalAdminCountProvider` 从 `active_subject_admin_count` 读取。

## 10. 事件

- `AdminSetChangeProposed`
- `AdminSetChangeExecutionFailed`
- `AdminSetChanged`
- `AdminSubjectPendingCreated`
- `AdminSubjectActivated { subject, org }`
- `AdminSubjectPendingRemoved { subject, org }`
- `AdminSubjectClosed { subject, org }`

生命周期事件必须携带 `org`，用于客户端和索引器按组织分桶，避免只拿 `subject` 后再反查 storage。

## 11. 测试

运行命令：

```bash
cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib
cargo test --manifest-path citizenchain/Cargo.toml -p primitives --lib
```

当前结果：

- `admins-change`：41 passed(新增 L-1/L-2 生命周期清理与事件 org 字段测试,2026-05-08)。
- `primitives`：24 passed。

覆盖重点：

- 动态阈值工具。
- `0x05 InstitutionAccount` 派生与解析。
- NRC / PRC / PRB 固定人数、固定阈值、等长管理员更换。
- 动态主体通过统一管理员集合变更提案增加、删除、更换管理员，并自动重算阈值。
- 管理员集合未变化、重复管理员、无效主体等错误路径。
- Pending 主体不会暴露给 Active 业务 API，但可通过 Pending 快照 API 读取。
- Pending 主体清理不存在时返回 `InvalidInstitution`，非 Pending 状态返回 `SubjectNotPending`。
- 激活、移除 Pending、关闭主体 3 类生命周期事件都包含 `org`。
- 生命周期 trait 拒绝脱离 votingengine 提案上下文的激活/关闭调用。
- 管理员集合变更提案与普通内部提案互斥。
- 自动执行成功/失败都由投票引擎统一推进终态并释放互斥锁。
- `InstitutionAccount` kind 5 条独立单测:最小 2 人、ceil(n/2) 阈值阶梯、< 2 拒绝、
  非 ORG_REN 拒绝、`MaxAdminsPerInstitution` 上界。
