# runtime admins 技术文档

最新更新：2026-06-29。`citizenchain/runtime/admins/` 由五个 crate 组成，管理员链上状态按管理员类型分别保存在各自 pallet。

## 模块边界

| 模块 | 职责 |
|------|------|
| `admin-primitives` | 管理员共用类型、生命周期 trait、统一查询 trait 和机构码分类策略；不放业务 storage。 |
| `genesis-admins` | 创世管理员：国储会、43 个省储会、43 个省储行、联邦注册局管理员治理。只保存管理员集合与换届治理，不保存创世机构本体封存表。 |
| `public-admins` | 非创世公权机构管理员：立法院、政府、学校、公立机构等。 |
| `private-admins` | 私法人及私权侧/独立非法人机构管理员：公司、协会、私立学校、个体经营、无限合伙等；公法人下属非法人不得被描述为私法人附属类型。 |
| `personal-admins` | 个人多签管理员和个人多签管理员集合变更。个人多签账户生命周期归 `runtime/entity/personal-manage`。 |

## 唯一真源

- 管理员共用字段统一由 `admin-primitives::AdminAccount` 表达：`institution_code`、`kind`、`admins`、`creator`、`created_at`、`updated_at`、`status`。
- 各类管理员的链上管理员集合分别保存在各自 pallet 的 `AdminAccounts`。
- runtime 只通过 `RuntimeAdminAccountQuery` 聚合读取各管理员模块，业务 pallet 不直接扫多个 storage。
- 注册个人多签和注册机构账户的动态阈值由 `votingengine/internal-vote` 的动态阈值表保存；NRC/PRC/PRB/FRG 创世治理阈值来自代码级固定阈值。
- 创世机构本体、主账户、费用账户和不可注销封存表由 `runtime/entity/genesis-manage` 保存；`genesis-admins` 不再承担机构信息或封存保护。

## 生命周期

- 公权机构生命周期由 `public-manage` 发起，只写 `public-admins`。
- 私权机构生命周期由 `private-manage` 发起，只写 `private-admins`。
- 个人多签账户生命周期由 `personal-manage` 发起，只写 `personal-admins`；管理员更换 call 为 `PersonalAdmins(7).propose_admin_set_change(3)`。
- 创世管理员只由 `genesis-admins` 维护；国储会、省储会、省储行固定人数，联邦注册局按 43 个省级 5 人组治理。
- 联邦注册局管理员更换必须走省级组入口：目标省 5 人组内部投票，阈值来自代码级固定阈值 `FRG=3`；不允许再用全 FRG 215 人平铺集合发起换届。
- FRG 主机构账户在读侧可聚合 43 个省级组，用于验签和身份展示；管理员更换投票根账户是链端按省码派生的省级组虚拟账户。
- 所有管理员集合变更仍经 `votingengine` 内部投票；各管理员模块用自己的 `MODULE_TAG` 绑定提案 owner。

## MODULE_TAG

| 模块 | MODULE_TAG |
|------|------------|
| `genesis-admins` | `b"gen-adm1"` |
| `public-admins` | `b"pub-adm1"` |
| `private-admins` | `b"pri-adm1"` |
| `personal-admins` | `b"per-mgmt"` |

## Call Index

| 模块 | 管理员更换 call |
|------|----------------|
| `genesis-admins` | `12.0 propose_admin_set_change`（NRC/PRC/PRB） |
| `genesis-admins` | `12.2 propose_federal_registry_province_admin_set_change`（FRG 省级组） |
| `public-admins` | `29.0 propose_admin_set_change` |
| `private-admins` | `30.0 propose_admin_set_change` |
| `personal-admins` | `7.3 propose_admin_set_change` |

## 验证命令

```bash
cargo check --manifest-path citizenchain/Cargo.toml -p node
cargo test --manifest-path citizenchain/Cargo.toml -p genesis-admins -p public-admins -p private-admins -p personal-admins -p public-manage -p private-manage -p personal-manage -p multisig-transfer --lib
```
