# personal-manage 模块技术文档

- pallet 名:`PersonalManage`
- pallet_index:**7**
- crate 路径:`citizenchain/runtime/governance/personal-manage/`
- MODULE_TAG:`b"per-mgmt"`(8 字节)
- 创建日期:2026-05-06(任务卡 B 拆分)
- 最新更新:2026-05-17(注销成功后清空资金并删除当前状态，可用相同 creator + account_name 重新注册)
- 关联 ADR:ADR-009(personal-manage 拆分)、ADR-010(AccountId 协议)、ADR-015(账户级内部投票管理员模型)

## 模块定位

**个人多签账户的注册/创建/关闭生命周期入口**。用户自定义多签账户,无 CID 归属,由 `creator + account_name` 派生地址。

与 `organization-manage`(机构多签)完全独立 — storage / event / error / extrinsic 命名空间完全隔离。

ADR-015 后，个人多签按“注册个人账户”治理：

- 个人多签只有一个账户，该账户独立持有管理员集合。
- 管理员数量范围为 `2..=64`。
- 创建和关闭必须全员投票通过。
- 普通业务提案按动态阈值通过。
- 管理员集合变更使用统一管理员集合变更提案，不拆分增加/删除/更换。
- 普通业务动态阈值由用户在注册或管理员变更时输入，投票引擎统一校验 `threshold * 2 > admins_len && threshold <= admins_len` 并保存。

## 协议参数

| 项 | 值 |
|---|---|
| pallet_index | 7 |
| MODULE_TAG | `b"per-mgmt"`(8 字节,与 `b"org-mgmt"` 长度对仗) |
| ACTION_CREATE | 0(独立命名空间,从 0 起) |
| ACTION_CLOSE | 1 |
| AdminAccountKind | `PersonalAccount AccountId`(D 阶段 ADR-010) |

## storage

| 名 | key/value | 用途 |
|---|---|---|
| `PersonalAccounts` | `StorageMap<account, PersonalAccount>` | 个人多签账户生命周期状态,保存 `creator / account_name / created_at / status` |
| `PendingPersonalCreate` | `StorageMap<proposal_id, PersonalCreateAction>` | 创建提案投票期 reserve 资金与 fee 快照 |
| `PendingCloseProposal` | `StorageMap<account, proposal_id>` | 防并发关闭提案 |

管理员和管理员数量不再存储或镜像在 `PersonalAccounts`。
管理员唯一真源为 `admins-change::Subjects[account_id_from_account(personal_account)]`。
普通动态阈值唯一真源为 `internal-vote::ActiveDynamicThresholds[(个人多签码 PMUL, subject)]`。
旧反向索引表已删除,反查 `creator + account_name` 直接读 `PersonalAccounts`。

## extrinsic

| call_index | 名 | 入参 | 业务 |
|---|---|---|---|
| 0 | `propose_create` | `account_name, admins, regular_threshold, amount` | 发起创建提案；普通动态阈值由用户输入并交投票引擎保存，创建投票阈值为全员 |
| 1 | `propose_close` | `personal_account, beneficiary` | 发起关闭提案(仅个人地址) |
| 2 | `cleanup_rejected_proposal` | `proposal_id` | 清理被否决/超时的 Pending 残留 |

## Event

| 名 | 触发时机 |
|---|---|
| `PersonalCreateProposed` | propose_create 成功 |
| `PersonalCreated` | 投票通过 + 入金完成 + 状态 Active |
| `CreateExecutionFailed` | 投票通过但执行失败 |
| `PersonalCreateRejected` | 投票否决/超时清理 |
| `PersonalCloseProposed` | propose_close 成功 |
| `PersonalClosed` | 关闭投票通过 + 余额转出 |
| `CloseExecutionFailed` | 关闭投票通过但执行失败 |

## 类型(`src/types.rs`)

- `PersonalStatus { Pending, Active }`
- `PersonalAccount<AccountId, AccountName, BlockNumber>`：保存 `creator / account_name / created_at / status`
- `PersonalCreateAction<AccountId, Balance>`
- `PersonalCloseAction<AccountId>`

`PersonalCreateAction` 当前字段：

```text
account: AccountId
proposer: AccountId
amount: Balance
fee: Balance
```

`fee` 是创建提案发起时的手续费快照。执行、否决 cleanup、执行失败终态 cleanup 都必须按该快照处理 reserve,不得用当前 runtime 的 fee 公式重新计算。

## trait(对外暴露)

- `PersonalMultisigQuery<AccountId>`(`src/traits.rs`):暴露 `lookup_admin_config / is_active`,multisig-transfer 通过它 union 查询多签 admin 配置

## 派生公式

```
personal_account = Blake2b_256(
    GMB || OP_PERSONAL || SS58_PREFIX_LE || creator.encode() || account_name_utf8
)
```

账户只依赖 `creator + account_name`,与管理员列表无关 — 换管理员账户不变。
注销不会改变派生公式；同一创建者使用同一账户名再次注册时仍得到同一地址。

## 治理主体 ID(AccountId)

```
account_id = core_const::account_id_from_account(personal_account)
           = byte[0]=PersonalAccount AccountId + byte[1..33]=AccountId + byte[33..48]=zeros(15B)
```

详见 ADR-010。

## 与 organization-manage 的边界

| 关注点 | personal-manage | organization-manage |
|---|---|---|
| 主体来源 | 用户自定义 | CID 注册机构 |
| 账户派生 | creator + account_name | cid_number + account_name(主/费用/自创) |
| 账户表 | `PersonalAccounts`(单地址) | `Institutions`(CidNumber-keyed) + `InstitutionAccounts`(机构下多账户) |
| MODULE_TAG | `b"per-mgmt"` | `b"org-mgmt"` |
| pallet_index | 7 | 17 |
| 客户端 dispatch | `PersonalAccounts.has(addr)` 命中走此 pallet | `AccountRegisteredCid.has(addr)` 命中走 organization-manage |

## 客户端协议

- citizenapp `lib/personal-manage/*` 直接调 pallet=7 的 propose_create/propose_close。
- citizenapp `PersonalManageService.submitProposeCreatePersonal` 编码：
  `0x07 0x00 + account_name + admins + regular_threshold + amount`。
- citizenapp 查询个人多签时，状态读 `PersonalManage::PersonalAccounts`，
  `creator/account_name` 也读 `PersonalManage::PersonalAccounts`，管理员读
  `AdminsChange::AdminAccounts`，普通动态阈值读 `InternalVote.ActiveDynamicThresholds`。
- citizenapp 解码 `PersonalManage::CreateMultisigAction` 时必须读取 `amount + fee` 两个 u128 字段。
- 创建类交易入块后若未找到成功事件，客户端必须先解析 `System.ExtrinsicFailed` 并显示真实 `PersonalManage / AdminsChange` 模块错误，不能只提示“未找到成功事件”。
- citizenwallet `pallet_registry.dart` 注册 `personalManagePallet=7` + 3 call_index。
- citizenwallet `payload_decoder.dart` 解析 PersonalManage(7) 新编码，并拒绝旧
  `admins_len + threshold` 交易载荷。

## 测试

当前 personal-manage 自持单测 23 case：

```bash
cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib
```

2026-05-17 注销当前状态清理修复后已回归通过：23 passed。

联动回归：

```bash
cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib
cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote --lib
cargo test --manifest-path citizenchain/Cargo.toml -p multisig-transfer --lib
cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib
flutter test test/organization-manage/account_manage_service_test.dart test/organization-manage/multisig_storage_codec_test.dart test/organization-manage/institution_multisig_storage_test.dart
flutter test test/signer/payload_decoder_test.dart
```

当前结果：

- `personal-manage`:23 passed。
- `admins-change`:44 passed。
- `internal-vote`:86 passed。
- `multisig-transfer`:22 passed。
- `organization-manage`:24 passed。
- `citizenapp` 多签相关测试:10 passed。
- `citizenwallet` 公民钱包 payload decoder:30 passed。
- `citizencode/backend`:cargo check 通过。

## benchmarks

`src/benchmarks.rs` 当前为空骨架(D 阶段补);weights.rs 已使用保守非零权重。完整 benchmark 用例补齐留 follow-up 任务卡。

## follow-up debt

- benchmarks 补 propose_create / propose_close / cleanup_rejected_proposal 三个用例

## 已清的 follow-up(2026-05-07)

- ~~personal-manage 自持单测~~ → 初始 16 用例已补；重新创世前总审计修复后为 23 passed
- ~~organization-manage 单测重写~~ → 22 用例已补(`src/tests/{mod.rs(441 行), cases.rs(716 行)}`,24 passed)

## 2026-05-11 投票边界修复结果

- `propose_create` 接收 `regular_threshold`，但该字段只表示账户激活后的普通业务动态阈值，不是本次注册投票通过阈值。
- 创建流程校验管理员数量 `2..=64`、管理员去重、创建人必须在管理员集合内，并把动态阈值交给投票引擎按严格过半规则校验。
- 创建提案和关闭提案的投票阈值由投票引擎按管理员快照写成全员。
- `PersonalAccounts` 不保存管理员列表、管理员数量和阈值镜像字段。
- 提案通过执行时，同一事务内先完成入金，再激活 `admins-change` 主体，投票引擎随后把 pending 动态阈值激活为 active 动态阈值。
- `PersonalMultisigQuery` 从 `admins-change` 读取管理员配置，从 `internal-vote` 读取动态阈值。

## 重新创世前总审计修复结果(2026-05-08)

- `CreateMultisigAction` 新增 `fee` 快照字段,执行/清理不再按当前 fee policy 重算创建手续费。
- `cleanup_pending_create` 先检查 `PendingPersonalCreate` 是否存在,重复手动 cleanup 不再重复发 `MultisigCreateRejected`。
- 旧反向索引 storage 和 meta 类型已删除,`account_name` 合并进 `PersonalAccounts`。
- `remove_pending_admin_account` 不再吞掉 `admins-change` 错误,清理路径会向上返回失败。
- `execute_create_with_finalizer` / `execute_close_with_finalizer` 已删除死参数。
- `InternalVoteExecutor` 统一使用 `decode_module_action` 解码 `MODULE_TAG + ACTION + payload`。
- 创建/关闭执行失败事件改由 `on_execution_failed_terminal` 在终态清理后发出。
- `PersonalClosed` 事件补充 `admins_len / threshold`。
- `PersonalCreateProposed` 事件补充 `fee`。
- `weights.rs` 从 0 权重改为保守非零权重。
- citizenapp storage codec 和 ProposalData 解码同步新 SCALE 布局。
- CID indexer 已读取个人账户创建/关闭事件中的 `fee` 字段。
