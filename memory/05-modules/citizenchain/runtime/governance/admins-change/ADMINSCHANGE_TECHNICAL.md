# ADMINS_CHANGE Technical Notes

最新更新：2026-05-17，投票阈值职责已从 `admins-change` 移出；本模块只维护管理员集合和生命周期，动态阈值由 `votingengine/internal-vote` 校验、保存和更新。动态多签主体关闭成功后删除当前状态记录，不保留 Closed 墓碑。

## 1. 模块定位

`admins-change` 是链上内部投票管理员主体的统一真源。

当前职责：

- 维护 `Subjects<AccountId, AdminAccount>`。
- 为治理机构写入创世固定管理员集合。
- 为个人账户、机构账户提供 Pending / Active 生命周期写入口；关闭成功后删除当前状态记录。
- 创建并执行管理员集合变更提案。

账户级口径来自 `memory/04-decisions/ADR-015-account-admin-internal-vote.md`：

- 省储行永久质押账户永远不可操作，不进入内部投票主体。
- 治理机构全部可操作账户共享固定管理员集合和固定阈值，只允许等长更换；固定阈值由投票引擎读取制度常量。
- 注册个人账户独立持有管理员集合，管理员数量范围为 `2..=64`。
- 注册机构账户独立持有管理员集合，管理员数量范围为 `2..=1989`。
- 动态账户普通业务阈值由注册或管理员变更时的用户输入决定，投票引擎按 `threshold * 2 > admins_len && threshold <= admins_len` 校验后保存。
- 创建和注销提案使用全员阈值，由投票引擎按管理员快照生成。

代码位置：

- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-change/src/lib.rs`
- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-change/src/weights.rs`
- `/Users/rhett/GMB/citizenchain/runtime/governance/admins-change/src/benchmarks.rs`

## 2. AdminAccountKind

`AdminAdminAccountKind` 当前值：

- `BuiltinInstitution`：NRC / PRC / PRB 等内置治理主体。
- `注册机构归属关系`：历史枚举值，新增、变更和生命周期写入路径一律拒绝作为管理员主体。
- `PersonalDuoqian`：注册个人账户主体。
- `InstitutionAccount`：注册机构某个具体账户的账户级主体。

对应底层 `AccountId` 协议见 ADR-010：

- `0x01 Builtin`
- `0x02 注册机构归属关系`，仅表示同一 CID 机构归属/检索。
- `PersonalDuoqian AccountId`
- `0x04 asset_id 资产编号`
- `InstitutionAccount AccountId`，payload 为账户 `AccountId` 前 32 字节并右填零。

## 3. 存储模型

`STORAGE_VERSION = 4`。管理员主体只保存管理员集合和生命周期；阈值存储已移交 `internal-vote`。

版本语义：

- v3：动态主体关闭后曾保留 `Closed` 当前状态记录。
- v4：动态主体关闭后直接删除 `Subjects[subject]` 当前状态；runtime upgrade 会清理旧链上遗留的 Closed 动态主体。历史区块、事件和投票提案仍保留在链历史中。

核心存储：

```text
Subjects<AccountId, AdminAccount>
```

`AdminAccount` 字段：

- `org`：内部投票组织类型，含 `ORG_NRC / ORG_PRC / ORG_PRB / ORG_REN / ORG_PUP / ORG_OTH`。
- `kind`：管理员主体类型。
- `admins`：当前管理员完整列表。
- `creator`：主体创建者。
- `created_at / updated_at`：生命周期时间。
- `status`：`Pending / Active / Closed`。新逻辑下 `Closed` 只代表历史兼容枚举值；动态多签关闭完成后不会再作为当前状态留存。

创世构建：

- 从 `CHINA_CB` 写入国储会、省储会主体。
- 从 `CHINA_CH` 写入省储行主体。
- 创世主体均为 `BuiltinInstitution + Active`。

## 4. 阈值职责边界

本模块不再提供、派生、保存或更新投票阈值。

规则：

- 创建个人多签和机构多签时，业务模块把用户填写的动态阈值提交给 `internal-vote`。
- 管理员集合变更时，本模块只把完整目标管理员集合、新管理员数量和新动态阈值提交给 `internal-vote`。
- `internal-vote` 负责校验 `threshold * 2 > admins_len && threshold <= admins_len`。
- 注册通过后，`internal-vote` 把 pending 动态阈值激活为 active 动态阈值。
- 注销执行成功后，`internal-vote` 删除 active 动态阈值。
- 管理员变更执行成功后，`internal-vote` 用提案里暂存的新阈值更新 active 动态阈值。

## 5. 管理员集合校验

统一校验入口：

```text
validate_admin_set_for_subject(kind, org, admins)
```

校验内容：

- 主体类型与 `org` 匹配。
- 管理员列表不能重复。
- `BuiltinInstitution` 必须等于固定人数：NRC 19、PRC 9、PRB 9。
- `PersonalDuoqian` 必须使用 `ORG_REN`，管理员数量必须在 `2..=MaxPersonalAccountAdmins`。
- `InstitutionAccount` 必须使用 `ORG_PUP / ORG_OTH`，管理员数量必须在 `2..=MaxAdminsPerInstitution`。
- `注册机构归属关系` 不能作为管理员主体，写入和变更路径返回 `InvalidAdminAccountKind`。
- 当前 runtime 配置：`MaxPersonalAccountAdmins = 64`，`MaxAdminsPerInstitution = 1989`。

读侧防线：

- `active_subject_*` 与 `pending_subject_*_for_snapshot` 查询在返回前同样校验主体类型与 `org` 是否匹配。
- 升级前误写入的 `注册机构归属关系` 管理员主体，或 `InstitutionAccount + ORG_REN` 等旧脏数据，不再通过读 API 暴露给投票引擎和业务模块。

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
- `close_subject_for_proposal` 关闭动态主体时删除 `Subjects[subject]` 当前状态；同一确定性地址在资金清空后可以重新注册为全新主体。
- `BuiltinInstitution` 永远不能关闭或删除。

## 7. 读取 API

Active-only 公共业务 API：

- `is_active_subject_admin(org, subject, who)`
- `active_subject_exists(org, subject)`
- `active_subject_admins(org, subject)`
- `active_subject_admins_len(org, subject)`

Pending 快照专用 API：

- `is_pending_subject_admin_for_snapshot(org, subject, who)`
- `pending_subject_exists_for_snapshot(org, subject)`
- `pending_subject_admins_for_snapshot(org, subject)`
- `pending_subject_admins_len_for_snapshot(org, subject)`

规则：

- 普通业务授权、普通内部提案创建和长期管理员真源读取只能使用 Active-only API。
- Pending 快照 API 只供创建/激活该主体时锁定管理员快照。
- 关闭完成的动态主体不保留当前状态；升级前遗留的 `Closed` 动态主体会被 v4 迁移删除。

## 8. 管理员集合变更流程

入口：

```text
propose_admin_set_change(org, subject, admins, new_threshold)
```

语义：

- 输入完整目标管理员集合。
- 链端对比当前集合与目标集合，不再拆分增加、删除、更换四类提案。
- 发起人必须是当前 Active 主体管理员。
- 目标集合必须通过统一管理员集合校验。
- 目标集合不能与当前集合完全一致。
- 在同一事务内调用 `create_admin_change_internal_proposal_with_data` 创建提案、写入 owner/data，并登记同主体管理员集合变更独占锁。

提案数据：

- `ProposalOwner = b"adm-set-v1"`。
- `ProposalData = AdminSetChangeAction<AdminsOf<T>>(SCALE)`。
- `AdminSetChangeAction.subject` 是目标主体。
- `AdminSetChangeAction.admins` 是完整目标管理员集合。
- `AdminSetChangeAction.new_threshold` 是变更执行成功后由投票引擎写入的动态阈值；固定治理机构必须等于制度固定阈值。

执行：

- `InternalVoteExecutor` 通过 `ProposalOwner` 认领 `adm-set-v1`。
- 提案通过后解码 `AdminSetChangeAction`。
- 执行前校验提案 kind/stage/status、`internal_institution`、`internal_org`、独占锁 owner。
- 再次校验目标管理员集合。
- 写回 `Subjects[subject].admins`。
- 更新 `updated_at`。

失败：

- 执行失败发出 `AdminSetChangeExecutionFailed`。
- callback 返回 `ProposalExecutionOutcome::FatalFailed`，投票引擎进入 `STATUS_EXECUTION_FAILED` 并释放互斥锁。

## 9. Runtime 接线

`/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`：

- `admins_change::Config::MaxAdminsPerInstitution = 1989`。
- `admins_change::Config::MaxPersonalAccountAdmins = 64`。
- `RuntimeInternalAdminProvider` 统一读取 `admins-change` Active / Pending API。
- `RuntimeInternalAdminsLenProvider` 从 `active_subject_admins_len` 读取。

## 10. 事件

- `AdminSetChangeProposed`
- `AdminSetChangeExecutionFailed`
- `AdminSetChanged`
- `AdminAccountPendingCreated`
- `AdminAccountActivated { subject, org }`
- `AdminAccountPendingRemoved { subject, org }`
- `AdminAccountClosed { subject, org }`

生命周期事件必须携带 `org`，用于客户端和索引器按组织分桶，避免只拿 `subject` 后再反查 storage。

## 11. 测试

运行命令：

```bash
cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib
cargo test --manifest-path citizenchain/Cargo.toml -p primitives --lib
```

当前结果：

- `admins-change`：44 passed(新增动态主体关闭删除当前状态和 v4 迁移清理 Closed 动态主体测试,2026-05-17)。
- `primitives`：24 passed。

覆盖重点：

- `InstitutionAccount AccountId` 派生与解析。
- NRC / PRC / PRB 固定人数、固定阈值、等长管理员更换。
- 动态主体通过统一管理员集合变更提案增加、删除、更换管理员，并把新动态阈值交给投票引擎更新。
- 管理员集合未变化、重复管理员、无效主体等错误路径。
- Pending 主体不会暴露给 Active 业务 API，但可通过 Pending 快照 API 读取。
- Pending 主体清理不存在时返回 `InvalidInstitution`，非 Pending 状态返回 `SubjectNotPending`。
- 激活、移除 Pending、关闭主体 3 类生命周期事件都包含 `org`。
- 生命周期 trait 拒绝脱离 votingengine 提案上下文的激活/关闭调用。
- 管理员集合变更提案与普通内部提案互斥。
- 自动执行成功/失败都由投票引擎统一推进终态并释放互斥锁。
- `InstitutionAccount` kind 独立单测覆盖最小 2 人、`ORG_PUP / ORG_OTH` 成功、`ORG_REN` 拒绝、`MaxAdminsPerInstitution` 上界。
- `注册机构归属关系` 新写入路径拒绝。
- 历史脏数据读侧拦截：`InstitutionAccount + ORG_REN`、`注册机构归属关系 + ORG_PUP` 不再通过 Active/Pending 业务 API 返回。
