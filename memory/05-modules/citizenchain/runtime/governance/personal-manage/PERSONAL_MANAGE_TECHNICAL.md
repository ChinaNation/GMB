# personal-manage 模块技术文档

- pallet 名:`PersonalManage`
- pallet_index:**7**
- crate 路径:`citizenchain/runtime/governance/personal-manage/`
- MODULE_TAG:`b"per-mgmt"`(8 字节)
- 创建日期:2026-05-06(任务卡 B 拆分)
- 最新更新:2026-05-08(ADR-015 第 3 步破坏式改造)
- 关联 ADR:ADR-009(personal-manage 拆分)、ADR-010(SubjectId 协议)、ADR-015(账户级内部投票管理员模型)

## 模块定位

**个人多签账户的注册/创建/关闭生命周期入口**。用户自定义多签账户,无 SFID 归属,由 `creator + account_name` 派生地址。

与 `organization-manage`(机构多签)完全独立 — storage / event / error / extrinsic 命名空间完全隔离。

ADR-015 后，个人多签按“注册个人账户”治理：

- 个人多签只有一个账户，该账户独立持有管理员集合。
- 管理员数量范围为 `2..=64`。
- 创建和关闭必须全员投票通过。
- 普通业务提案按动态阈值通过。
- 管理员集合变更使用统一管理员集合变更提案，不拆分增加/删除/更换/改阈值。
- 阈值不再由用户自由输入，而是由链端按管理员数量派生：`2 -> 2`，`>=3 -> ceil(admin_count / 2)`。

## 协议参数

| 项 | 值 |
|---|---|
| pallet_index | 7 |
| MODULE_TAG | `b"per-mgmt"`(8 字节,与 `b"org-mgmt"` 长度对仗) |
| ACTION_CREATE | 0(独立命名空间,从 0 起) |
| ACTION_CLOSE | 1 |
| SubjectKind | `0x03 PersonalDuoqian`(D 阶段 ADR-010) |

## storage

| 名 | key/value | 用途 |
|---|---|---|
| `PersonalDuoqians` | `StorageMap<address, DuoqianAccount>` | 个人多签账户生命周期状态,仅保存 `creator / created_at / status` |
| `PersonalDuoqianInfo` | `StorageMap<address, PersonalDuoqianMeta>` | 反向索引(creator + account_name) |
| `PendingPersonalCreate` | `StorageMap<proposal_id, CreateDuoqianAction>` | 创建提案投票期 reserve 资金记录 |
| `PendingCloseProposal` | `StorageMap<address, proposal_id>` | 防并发关闭提案 |

管理员、管理员数量和普通阈值不再存储或镜像在 `PersonalDuoqians`。
唯一真源为 `admins-change::Subjects[subject_id_from_account(personal_address)]`。

## extrinsic

| call_index | 名 | 入参 | 业务 |
|---|---|---|---|
| 0 | `propose_create` | `account_name, duoqian_admins, amount` | 发起创建提案；`admin_count` 由管理员列表长度派生，普通阈值由 `admins-change` 派生，创建投票阈值为全员 |
| 1 | `propose_close` | `personal_address, beneficiary` | 发起关闭提案(仅个人地址) |
| 2 | `cleanup_rejected_proposal` | `proposal_id` | 清理被否决/超时的 Pending 残留 |

## Event

| 名 | 触发时机 |
|---|---|
| `PersonalDuoqianProposed` | propose_create 成功 |
| `DuoqianCreated` | 投票通过 + 入金完成 + 状态 Active |
| `CreateExecutionFailed` | 投票通过但执行失败 |
| `DuoqianCreateRejected` | 投票否决/超时清理 |
| `CloseDuoqianProposed` | propose_close 成功 |
| `DuoqianClosed` | 关闭投票通过 + 余额转出 |
| `CloseExecutionFailed` | 关闭投票通过但执行失败 |

## 类型(`src/types.rs`)

- `DuoqianStatus { Pending, Active }`
- `DuoqianAccount<AccountId, BlockNumber>`：只保存 `creator / created_at / status`
- `CreateDuoqianAction<AccountId, Balance>`
- `CloseDuoqianAction<AccountId>`
- `PersonalDuoqianMeta<AccountId, AccountName>`

`CreateDuoqianAction` 当前字段：

```text
duoqian_address: AccountId
proposer: AccountId
amount: Balance
```

## trait(对外暴露)

- `PersonalMultisigQuery<AccountId>`(`src/traits.rs`):暴露 `lookup_admin_config / is_active`,duoqian-transfer 通过它 union 查询多签 admin 配置

## 派生公式

```
personal_duoqian_address = Blake2b_256(
    DUOQIAN_DOMAIN || OP_PERSONAL || SS58_PREFIX_LE || creator.encode() || account_name_utf8
)
```

地址只依赖 `creator + account_name`,与管理员列表无关 — 换管理员地址不变。

## 治理主体 ID(SubjectId)

```
subject_id = primitives::derive::subject_id_from_account(personal_address)
           = byte[0]=0x03 PersonalDuoqian + byte[1..33]=AccountId + byte[33..48]=zeros(15B)
```

详见 ADR-010。

## 与 organization-manage 的边界

| 关注点 | personal-manage | organization-manage |
|---|---|---|
| 主体来源 | 用户自定义 | SFID 注册机构 |
| 地址派生 | creator + account_name | sfid_number + account_name(主/费用/自创) |
| 账户表 | `PersonalDuoqians`(单地址) | `Institutions`(SfidNumber-keyed) + `InstitutionAccounts`(机构下多账户) |
| MODULE_TAG | `b"per-mgmt"` | `b"org-mgmt"` |
| pallet_index | 7 | 17 |
| 客户端 dispatch | `PersonalDuoqianInfo.has(addr)` 命中走此 pallet | `AddressRegisteredSfid.has(addr)` 命中走 organization-manage |

## 客户端协议

- wuminapp `lib/duoqian/personal/*` 直接调 pallet=7 的 propose_create/propose_close。
- wuminapp `DuoqianManageService.submitProposeCreatePersonal` 编码：
  `0x07 0x00 + account_name + duoqian_admins + amount`。
- wuminapp 查询个人多签时，状态读 `PersonalManage::PersonalDuoqians`，
  管理员和阈值读 `AdminsChange::Subjects`。
- wumin `pallet_registry.dart` 注册 `personalManagePallet=7` + 3 call_index。
- wumin `payload_decoder.dart` 解析 PersonalManage(7) 新编码，并拒绝旧
  `admin_count + threshold` 交易载荷。

## 测试

当前 personal-manage 自持单测 19 case：

```bash
cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib
```

第 3 步破坏式改造后已回归通过：19 passed。

联动回归：

```bash
cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib
cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote --lib
cargo test --manifest-path citizenchain/Cargo.toml -p duoqian-transfer --lib
cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib
flutter test test/duoqian/duoqian_manage_service_test.dart test/duoqian/duoqian_storage_codec_test.dart test/duoqian/duoqian_manage_storage_test.dart
flutter test test/signer/payload_decoder_test.dart
```

当前结果：

- `personal-manage`:19 passed。
- `admins-change`:39 passed。
- `internal-vote`:86 passed。
- `duoqian-transfer`:20 passed。
- `organization-manage`:24 passed。
- `wuminapp` 多签相关测试:10 passed。
- `wumin` 冷钱包 payload decoder:30 passed。

## benchmarks

`src/benchmarks.rs` 当前为空骨架(D 阶段补);weights.rs 走零权重占位。完整 benchmark 用例补齐留 follow-up 任务卡。

## follow-up debt

- benchmarks 补 propose_create / propose_close / cleanup_rejected_proposal 三个用例

## 已清的 follow-up(2026-05-07)

- ~~personal-manage 自持单测~~ → 初始 16 用例已补；第 3 步扩展后为 19 passed
- ~~organization-manage 单测重写~~ → 22 用例已补(`src/tests/{mod.rs(441 行), cases.rs(716 行)}`,24 passed)

## 第 3 步执行结果(2026-05-08)

- `propose_create` 已删除 `admin_count / threshold` 入参。
- 创建流程校验管理员数量 `2..=64`、管理员去重、创建人必须在管理员集合内。
- 创建提案的投票阈值为拟定管理员全员数量。
- 普通阈值统一由 `admins-change::derived_threshold(PersonalDuoqian, ORG_REN, admin_count)` 派生。
- `PersonalDuoqians` 已删除管理员列表、管理员数量和阈值镜像字段。
- `CreateDuoqianAction` 已删除管理员数量和阈值字段。
- 提案通过执行时，同一事务内先完成入金，再激活 `admins-change` 主体，最后激活个人账户状态。
- `PersonalMultisigQuery` 和 `duoqian-transfer` 均从 `admins-change` 读取管理员配置。
- wuminapp 创建页移除手填阈值，只展示派生日常阈值与创建全员阈值。
- wumin 冷钱包拒绝旧个人创建交易载荷。
- 本次未修改 `spec_version`。
