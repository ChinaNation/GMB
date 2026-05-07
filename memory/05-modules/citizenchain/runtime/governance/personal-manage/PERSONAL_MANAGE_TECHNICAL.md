# personal-manage 模块技术文档

- pallet 名:`PersonalManage`
- pallet_index:**7**
- crate 路径:`citizenchain/runtime/governance/personal-manage/`
- MODULE_TAG:`b"per-mgmt"`(8 字节)
- 创建日期:2026-05-06(任务卡 B 拆分)
- 关联 ADR:ADR-009(personal-manage 拆分)、ADR-010(SubjectId 协议)

## 模块定位

**个人多签账户的注册/创建/关闭生命周期入口**。用户自定义多签账户,无 SFID 归属,由 `creator + account_name` 派生地址。

与 `organization-manage`(机构多签)完全独立 — storage / event / error / extrinsic 命名空间完全隔离。

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
| `PersonalDuoqians` | `StorageMap<address, DuoqianAccount>` | 个人多签账户配置(替代旧 `DuoqianAccounts` 个人部分) |
| `PersonalDuoqianInfo` | `StorageMap<address, PersonalDuoqianMeta>` | 反向索引(creator + account_name) |
| `PendingPersonalCreate` | `StorageMap<proposal_id, CreateDuoqianAction>` | 创建提案投票期 reserve 资金记录 |
| `PendingCloseProposal` | `StorageMap<address, proposal_id>` | 防并发关闭提案 |

## extrinsic

| call_index | 名 | 入参 | 业务 |
|---|---|---|---|
| 0 | `propose_create` | `account_name, admin_count, duoqian_admins, threshold, amount` | 发起创建提案,reserve 资金 |
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
- `DuoqianAccount<AdminList, AccountId, BlockNumber>`
- `CreateDuoqianAction<AccountId, Balance>`
- `CloseDuoqianAction<AccountId>`
- `PersonalDuoqianMeta<AccountId, AccountName>`

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

- wuminapp `lib/duoqian/personal/*` 6 dart 文件直接调 pallet=7 的 propose_create/propose_close
- wumin `pallet_registry.dart` 注册 `personalManagePallet=7` + 3 call_index
- wumin `payload_decoder.dart` 解析 PersonalManage(7) 的 3 个 call + MODULE_TAG `per-mgmt`

## 测试

当前 personal-manage 自持单测 0 case(B 阶段 follow-up debt);集成测试通过 `cargo test -p citizenchain --lib`(37 case)+ `cargo test -p duoqian-transfer`(20 case)间接覆盖。

## benchmarks

`src/benchmarks.rs` 当前为空骨架(D 阶段补);weights.rs 走零权重占位。完整 benchmark 用例补齐留 follow-up 任务卡。

## follow-up debt

- benchmarks 补 propose_create / propose_close / cleanup_rejected_proposal 三个用例

## 已清的 follow-up(2026-05-07)

- ~~personal-manage 自持单测~~ → 14 用例已补(`src/tests/{mod.rs(423 行), cases.rs(460 行)}`,16 passed)
- ~~organization-manage 单测重写~~ → 22 用例已补(`src/tests/{mod.rs(441 行), cases.rs(716 行)}`,24 passed)
