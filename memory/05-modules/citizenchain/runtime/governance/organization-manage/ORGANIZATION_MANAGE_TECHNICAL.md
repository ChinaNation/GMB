# DUOQIAN_TECHNICAL

模块：`organization-manage`  
最新更新：2026-05-17，动态阈值由用户输入并交 `internal-vote` 保存；机构账户关闭成功后清空资金并删除账户当前状态索引。

## 1. 当前边界

`organization-manage` 负责链上注册机构和机构账户的创建、激活、关闭提案，以及与内部投票引擎 `ORG_PUP / ORG_OTH` 的对接。

管理员和管理员人数的长期真源是 `admins-change::Subjects`；动态阈值长期真源是 `internal-vote::ActiveDynamicThresholds`。本模块只负责机构归属、机构账户、资金和生命周期。

ADR-015 后，机构管理按账户级治理：

- 机构只是账户归属分组，不是管理员集合真源。
- 一个机构可以下挂多个账户，每个可操作账户独立持有管理员集合。
- 主账户不管理其他账户，每个账户只由自己的管理员管理。
- 当前 `propose_create_institution` 创建的是机构主账户的管理员主体；主体由主账户地址派生为 `InstitutionAccount(0x05)`，不是 `注册机构归属关系(0x02)`。
- `admin_org` 必须是 `ORG_PUP` 或 `ORG_OTH`；`ORG_REN` 只属于个人多签。
- 省储行永久质押账户永远不可操作，不得进入本模块账户治理。
- 注册机构账户管理员数量范围为 `2..=1989`。
- 注册机构账户创建和关闭必须由该账户管理员全员投票通过。
- 动态账户普通业务阈值由用户在注册或管理员变更时输入，投票引擎统一校验 `threshold * 2 > admin_count && threshold <= admin_count` 并保存。

## 2. 目录结构

- `src/address.rs`：`DUOQIAN` 地址角色语义，包含主账户、费用账户、自定义账户的角色定义。
- `src/institution/`：机构多签业务分区。
- `src/institution/types.rs`：机构级 storage/action 类型。
- `src/personal/`：个人多签业务分区。
- `src/lib.rs`：FRAME pallet 宏、storage、extrinsic、投票回调。

由于 FRAME pallet 宏对 storage/call 定义位置有约束，storage/call 定义保留在 `lib.rs`，业务类型按机构多签和个人多签分目录维护。

## 3. 地址规则

机构账户地址继续严格遵守 `DUOQIAN`：

| 账户 | op_tag | preimage |
|---|---:|---|
| 主账户 | `OP_MAIN = 0x00` | `DUOQIAN || OP_MAIN || ss58_le || sfid_number` |
| 费用账户 | `OP_FEE = 0x01` | `DUOQIAN || OP_FEE || ss58_le || sfid_number` |
| 自定义账户 | `OP_INSTITUTION = 0x05` | `DUOQIAN || OP_INSTITUTION || ss58_le || sfid_number || account_name` |
| 个人多签 | `OP_PERSONAL = 0x04` | `DUOQIAN || OP_PERSONAL || ss58_le || creator || account_name` |

`"主账户"` 和 `"费用账户"` 是保留名，只能分别落到 `OP_MAIN` 和 `OP_FEE`；禁止作为自定义账户名进入 `OP_INSTITUTION` 命名空间。

## 4. 新增机构级模型

核心 storage：

- `Institutions<sfid_number, InstitutionInfo>`：机构归属、主账户、费用账户、`admin_org`、机构状态。ADR-015 后不得作为机构级管理员真源；动态阈值真源在 `internal-vote`。
- `InstitutionAccounts<(sfid_number, account_name), InstitutionAccountInfo>`：机构下每个账户名对应的地址、初始余额、状态。
- `PendingInstitutionCreate<proposal_id, CreateInstitutionAction>`：创建提案 pending 期间的 reserve 资金和账户列表。

- `SfidRegisteredAccount` / `AccountRegisteredSfid`：继续作为链上账户索引。
- 个人多签账户不在本模块保存，当前真源为 `personal-manage::PersonalDuoqians`。

管理员主体：

- 机构多签创建提案发起时，主账户地址会转换为 `InstitutionAccount(0x05)` 的 `AccountId`，并按 `admin_org=ORG_PUP/ORG_OTH` 通过 `admins-change::SubjectLifecycle` 写入 `Pending` 主体。
- 个人多签创建提案发起时，个人多签地址会通过 `admins-change::SubjectLifecycle` 写入 `PersonalDuoqian` 类型的 `Pending` 主体。
- 创建投票通过后自动执行激活主体；自动执行暂时失败时提案保持 `STATUS_PASSED` 并进入 votingengine retry state，最终 `EXECUTION_FAILED` 时统一清理主体和 pending 数据；多签关闭成功后删除账户当前状态主体。
- 创建机构多签时，投票提案必须走 `InternalVoteEngine::create_registered_account_create_proposal_with_data`，由投票引擎用显式管理员列表锁定全员创建投票快照，并保存用户填写的动态阈值。
- 关闭多签必须走 `InternalVoteEngine::create_lifecycle_internal_proposal_with_data`，由投票引擎按 Active 管理员快照写全员关闭投票阈值。
- 其他普通业务必须走 `InternalVoteEngine::create_general_internal_proposal_with_data`，只接受 Active 主体和 active 动态阈值。

## 5. 机构创建入口

新增：

```text
propose_create_institution(
  sfid_number,
  sfid_full_name,
  accounts,
  admin_org,
  admin_count,
  duoqian_admins,
  threshold,
  register_nonce,
  signature,
  province,
  signer_admin_pubkey
)
```

核心规则：

- 创建的是机构，不是单个账户。
- `accounts` 必须包含 `"主账户"` 和 `"费用账户"`。
- `admin_org` 只能是 `ORG_PUP(4)` 或 `ORG_OTH(5)`。
- 每个账户初始余额都必须 `>= MinCreateAmount`，当前配置语义为最低 1.11 元。
- 账户名不得重复。
- 管理员数量必须 `>= 2`。ADR-015 后注册机构账户管理员数量必须 `<= 1989`；动态阈值由用户输入，必须严格过半且不得超过管理员数量。
- 创建者必须在管理员列表中。
- SFID 登记 nonce 必须未使用，签名必须通过 `SfidInstitutionVerifier`。
- `SfidInstitutionVerifier` 的注册业务字段只覆盖 `sfid_number / sfid_full_name / account_names[]`。
- `province_name + signer_admin_pubkey` 只用于在 `sfid-system::ShengSigningPubkey` 中定位联邦管理员派生签名公钥。
- `subject_property / sub_type / parent_sfid_number` 只属于 SFID 系统候选资格判断,不进入链上注册 storage、action 或 call payload。

资金规则：

- 发起提案时计算 `initial_total = sum(accounts.amount)`。
- 手续费按 `onchain-transaction::calculate_onchain_fee(initial_total)` 计算。
- 发起提案时从创建者账户 reserve `initial_total + fee`。
- 投票通过执行时，先 unreserve，再扣手续费，再把各账户初始余额划入对应机构账户。
- 投票拒绝时释放 reserve 并清理 pending 索引；自动执行暂时失败时保留 pending 数据供重试；进入 `STATUS_EXECUTION_FAILED` 终态时由 votingengine 的终态回调释放 reserve 并清理 pending 索引。
- 机构账户关闭执行时，先扣链上手续费，再把 `free_balance - fee` 转入用户提供的收款地址；执行阶段再次拒绝 reserved 余额，保证账户能被清空。
- 机构账户关闭成功后删除 `InstitutionAccounts[(sfid, account_name)]`、`SfidRegisteredAccount[(sfid, account_name)]`、`AccountRegisteredSfid[address]` 和 `admins-change::Subjects[subject]` 当前状态。历史事件和历史提案不删除。

## 6. 投票回调

新增 proposal action：

- `ACTION_CREATE_INSTITUTION = 3`

投票引擎终态回调规则：

- `approved = true`：调用 `execute_create_institution`，激活 `Institutions`、`InstitutionAccounts`、主账户生命周期记录和管理员主体。
- `approved = false`：调用 `cleanup_pending_institution_create`，释放创建者 reserve，删除机构 pending storage、SFID 地址索引和管理员主体。
- 管理员主体的激活、拒绝清理、执行失败终态清理和关闭都必须带 `proposal_id` 调用 `admins-change::SubjectLifecycle`，由 admins-change 校验提案 owner、状态和 callback 作用域。

执行成功事件：

- `InstitutionCreateProposed`
- `InstitutionCreated`
- `InstitutionCreateRejected`
- `InstitutionCreateExecutionFailed`

## 7. 对外入口

- `register_sfid_institution`
- `propose_create`
- `propose_create_personal`
- `propose_close`
- `propose_create_institution`
- `cleanup_rejected_proposal`

runtime 适配：

- `RuntimeInternalAdminProvider / RuntimeInternalAdminCountProvider` 统一读取 `admins-change`。
- 普通业务路径读取 `admins-change` 的 Active-only 管理员 API，并从 `internal-vote` 读取动态阈值。
- 创建多签主体路径把初始管理员列表和动态阈值直接交给 `internal-vote`。
- `DuoqianSfidAccountQuery::is_admin_of` 通过 `resolve_admin_account_for_account` 映射到账户级管理员主体，并通过 `resolve_admin_org_for_account` 读取 `ORG_PUP / ORG_OTH`。
- `DuoqianSfidAccountQuery::is_active` 对 SFID 机构账户读取 `InstitutionAccounts` 的激活状态。
- `DuoqianSfidAccountQuery::is_clearing_bank_eligible` 不再读取机构类型元数据;SFID 负责 `eligible-search` 候选筛选,链上只确认地址属于已注册且 Active 的 SFID 机构账户。

## 8. 测试覆盖

`cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib` 已覆盖：

- 机构级创建通过后激活所有账户，并把 reserve 资金划入对应账户。
- 机构级创建被拒绝后释放 reserve 并清理索引。
- 机构级创建提案在提案、Pending 主体、reserve、地址索引任一步失败时整体回滚。
- 缺少主账户时拒绝。
- 账户初始余额低于最低金额时拒绝。
- 批量 SFID 机构注册按 `sfid_full_name + account_names[]` 验签并写入地址索引。
- 个人多签路径可创建和激活。
- 关闭、重复管理员、重放投票等回归路径通过；关闭用例覆盖余额转出、pending 清理、账户索引清理、管理员主体清理和动态阈值清理。

关联验证：

- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：44 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote --lib`：86 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib`：24 passed。

## 9. 变更记录

- 2026-05-02:机构注册协议对齐 SFID `registration-info`。删除链上 `InstitutionMetadata` 与注册参数中的 `subject_property/sub_type/parent_sfid_number`,签名业务字段收口为 `sfid_number / sfid_full_name / account_names[]`。
- 2026-05-02:创建 Pending 多签主体改为 votingengine 显式快照提案 + admins-change `SubjectLifecycle`，生命周期写状态不再依赖裸公共 mutator。
- 2026-05-17:机构账户关闭成功后删除账户正向/反向索引和管理员主体当前状态；已转出的余额不继承到重新注册的新当前状态。
