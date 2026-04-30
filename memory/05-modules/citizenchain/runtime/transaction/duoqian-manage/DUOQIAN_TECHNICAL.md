# DUOQIAN_TECHNICAL

模块：`duoqian-manage`  
最新更新：2026-04-30

## 1. 当前边界

`duoqian-manage` 负责链上多签机构和个人多签的创建、激活、关闭提案，以及与内部投票引擎 `ORG_DUOQIAN` 的对接。

管理员、阈值和管理员人数的长期真源是 `admins-change::Institutions`；本模块只负责多签账户、机构账户、资金和生命周期。

## 2. 目录结构

- `src/address.rs`：`DUOQIAN_V1` 地址角色语义，包含主账户、费用账户、自定义账户的角色定义。
- `src/institution/`：机构多签业务分区。
- `src/institution/types.rs`：机构级 storage/action 类型。
- `src/personal/`：个人多签业务分区。
- `src/lib.rs`：FRAME pallet 宏、storage、extrinsic、投票回调。

由于 FRAME pallet 宏对 storage/call 定义位置有约束，storage/call 定义保留在 `lib.rs`，业务类型按机构多签和个人多签分目录维护。

## 3. 地址规则

机构账户地址继续严格遵守 `DUOQIAN_V1`：

| 账户 | op_tag | preimage |
|---|---:|---|
| 主账户 | `OP_MAIN = 0x00` | `DUOQIAN_DOMAIN || OP_MAIN || ss58_prefix_le || sfid_id` |
| 费用账户 | `OP_FEE = 0x01` | `DUOQIAN_DOMAIN || OP_FEE || ss58_prefix_le || sfid_id` |
| 自定义账户 | `OP_INSTITUTION = 0x05` | `DUOQIAN_DOMAIN || OP_INSTITUTION || ss58_prefix_le || sfid_id || account_name` |
| 个人多签 | `OP_PERSONAL = 0x04` | `DUOQIAN_DOMAIN || OP_PERSONAL || ss58_prefix_le || creator || account_name` |

`"主账户"` 和 `"费用账户"` 是保留名，只能分别落到 `OP_MAIN` 和 `OP_FEE`；禁止作为自定义账户名进入 `OP_INSTITUTION` 命名空间。

## 4. 新增机构级模型

核心 storage：

- `Institutions<sfid_id, InstitutionInfo>`：机构级管理员、阈值、主账户、费用账户、机构状态。
- `InstitutionAccounts<(sfid_id, account_name), InstitutionAccountInfo>`：机构下每个账户名对应的地址、初始余额、状态。
- `PendingInstitutionCreate<proposal_id, CreateInstitutionAction>`：创建提案 pending 期间的 reserve 资金和账户列表。

- `DuoqianAccounts<main_address, DuoqianAccount>`：只保存多签账户生命周期和创建时阈值快照，不作为管理员长期真源。
- `SfidRegisteredAddress` / `AddressRegisteredSfid`：继续作为链上账户索引。
- `InstitutionMetadata`：继续保存 a3、sub_type、parent_sfid_id。
- `PersonalDuoqianInfo`：个人多签索引。

管理员主体：

- 机构多签创建提案发起时，主账户地址会转换为 `InstitutionPalletId`，写入 `admins-change::Institutions` 的 `Pending` 主体。
- 个人多签创建提案发起时，个人多签地址会写入 `PersonalDuoqian` 类型的 `Pending` 主体。
- 创建投票通过后激活主体；创建拒绝或执行失败后清理主体；多签关闭后关闭主体。
- 创建机构多签/个人多签时，投票提案必须走 `VotingEngine::create_pending_subject_internal_proposal`，由 Pending 快照 API 锁定管理员和阈值。
- 关闭多签和其他普通业务必须走 `VotingEngine::create_internal_proposal`，只接受 Active 主体。

## 5. 机构创建入口

新增：

```text
propose_create_institution(
  sfid_id,
  institution_name,
  accounts,
  admin_count,
  duoqian_admins,
  threshold,
  register_nonce,
  signature,
  signing_province,
  a3,
  sub_type,
  parent_sfid_id
)
```

核心规则：

- 创建的是机构，不是单个账户。
- `accounts` 必须包含 `"主账户"` 和 `"费用账户"`。
- 每个账户初始余额都必须 `>= MinCreateAmount`，当前配置语义为最低 1.11 元。
- 账户名不得重复。
- 管理员数量必须 `>= 2`，阈值必须满足 `ceil(admin_count / 2) <= threshold <= admin_count` 且最小为 2。
- 创建者必须在管理员列表中。
- SFID 登记 nonce 必须未使用，签名必须通过 `SfidInstitutionVerifier`。
- a3 / sub_type / parent_sfid_id 必须满足机构元数据形态规则。

资金规则：

- 发起提案时计算 `initial_total = sum(accounts.amount)`。
- 手续费按 `onchain-transaction::calculate_onchain_fee(initial_total)` 计算。
- 发起提案时从创建者账户 reserve `initial_total + fee`。
- 投票通过执行时，先 unreserve，再扣手续费，再把各账户初始余额划入对应机构账户。
- 投票拒绝、超时清理或执行失败时，释放 reserve 并清理 pending 索引。

## 6. 投票回调

新增 proposal action：

- `ACTION_CREATE_INSTITUTION = 3`

投票引擎终态回调规则：

- `approved = true`：调用 `execute_create_institution`，激活 `Institutions`、`InstitutionAccounts`、主账户生命周期记录和管理员主体。
- `approved = false`：调用 `cleanup_pending_institution_create`，释放创建者 reserve，删除机构 pending storage、SFID 地址索引和管理员主体。

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

- `RuntimeInternalAdminProvider / RuntimeInternalThresholdProvider / RuntimeInternalAdminCountProvider` 统一读取 `admins-change`。
- 普通业务路径读取 `admins-change` 的 Active-only API。
- 创建多签主体路径读取 `admins-change` 的 Pending 快照 API。
- `DuoqianSfidAccountQuery::is_admin_of` 通过 `resolve_admin_subject_for_account` 映射到账户所属管理员主体。
- `DuoqianSfidAccountQuery::is_active` 对 SFID 机构账户读取 `InstitutionAccounts` 的激活状态。

## 8. 测试覆盖

`cargo test -p duoqian-manage --lib` 已覆盖：

- 机构级创建通过后激活所有账户，并把 reserve 资金划入对应账户。
- 机构级创建被拒绝后释放 reserve 并清理索引。
- 缺少主账户时拒绝。
- 账户初始余额低于最低金额时拒绝。
- 个人多签路径可创建和激活。
- 关闭、重复管理员、重放投票等回归路径通过。

关联验证：

- `cargo test -p admins-change --lib`
- `cargo test -p duoqian-transfer --lib`
- `cargo test -p offchain-transaction --lib`
