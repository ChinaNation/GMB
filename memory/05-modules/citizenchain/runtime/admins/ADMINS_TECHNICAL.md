# runtime admins 技术文档

最新更新：2026-06-26。`citizenchain/runtime/admins/` 由五个 crate 组成，管理员链上状态按管理员类型分别保存在各自 pallet。

## 模块边界

| 模块 | 职责 |
|------|------|
| `admin-primitives` | 管理员共用类型、生命周期 trait、统一查询 trait 和机构码分类策略；不放业务 storage。 |
| `genesis-admins` | 创世管理员：国储会、43 个省储会、43 个省储行、联邦注册局。负责创世写入、固定治理管理员更换、创世封存保护。 |
| `public-admins` | 非创世公权机构管理员：立法院、政府、学校、公立机构等。 |
| `private-admins` | 私权法人和非法人机构管理员：公司、协会、私立学校、个体经营、无限合伙等。 |
| `personal-admins` | 个人多签管理员、个人多签账户生命周期和个人多签管理员集合变更。 |

## 唯一真源

- 管理员共用字段统一由 `admin-primitives::AdminAccount` 表达：`institution_code`、`kind`、`admins`、`creator`、`created_at`、`updated_at`、`status`。
- 各类管理员的链上管理员集合分别保存在各自 pallet 的 `AdminAccounts`。
- runtime 只通过 `RuntimeAdminAccountQuery` 聚合读取各管理员模块，业务 pallet 不直接扫多个 storage。
- 动态阈值仍由 `votingengine/internal-vote` 的动态阈值表保存，管理员模块只保存管理员集合。

## 生命周期

- 机构注册由 `organization-manage` 发起，按 `institution_code` 路由到 `public-admins` 或 `private-admins` 写入 Pending、激活、清理或关闭。
- 个人多签由 `personal-admins` 自己完成创建、关闭、Pending 清理和管理员集合变更；管理员更换 call 为 `PersonalAdmins(7).propose_admin_set_change(3)`。
- 创世管理员只由 `genesis-admins` 维护；国储会、省储会、省储行固定人数，联邦注册局为创世内置注册局但人数走动态上限。
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
| `genesis-admins` | `12.0 propose_admin_set_change` |
| `public-admins` | `29.0 propose_admin_set_change` |
| `private-admins` | `30.0 propose_admin_set_change` |
| `personal-admins` | `7.3 propose_admin_set_change` |

## 验证命令

```bash
cargo check --manifest-path citizenchain/Cargo.toml -p node
cargo test --manifest-path citizenchain/Cargo.toml -p genesis-admins -p public-admins -p private-admins -p personal-admins -p organization-manage -p multisig-transfer --lib
```
