# runtime admins 技术文档

最新更新：2026-07-12。`citizenchain/runtime/admins/` 由四个 crate 组成，管理员链上状态按管理员类型分别保存在各自 pallet。

## 模块边界

| 模块 | 职责 |
|------|------|
| `admin-primitives` | 管理员共用类型、生命周期 trait、统一查询 trait 和机构码分类策略；不放业务 storage。 |
| `public-admins` | 公权机构管理员：立法院、政府、学校、公立机构，以及 NRC/PRC/PRB/FRG/NJD 固定治理机构。FRG 省级组也在本模块保存和治理。 |
| `private-admins` | 私法人及私权侧/独立非法人机构管理员：公司、协会、私立学校、个体经营、无限合伙等；公法人下属非法人不得被描述为私法人附属类型。 |
| `personal-admins` | 个人多签管理员和个人多签管理员集合变更。个人多签账户生命周期归 `runtime/entity/personal-manage`。 |

## 唯一真源

- 管理员集合目标字段统一由 `admin-primitives::AdminAccount` 表达：`cid_number`、`institution_code`、`kind`、`admins`、`status`。当前实现中的 `creator`、`created_at`、`updated_at` 随第二步 runtime 改造删除；管理员或任职来源不得用创建人替代。
- 机构管理员真源只管理钱包账户集合 `admins`及集合生命周期，不再保存管理员姓名、公民 CID、岗位、任期和任职来源。
- 机构岗位定义、岗位权限和机构管理员任职关系归 `entity`；`admins` 只向下游提供当前有效管理员账户。
- 个人多签管理员保持 `personal-admins` 独立模型，不使用机构岗位或机构任职关系。
- 各类管理员的链上管理员集合分别保存在各自 pallet 的 `AdminAccounts`。
- runtime 只通过 `RuntimeAdminAccountQuery` 聚合读取各管理员模块，业务 pallet 不直接扫多个 storage。
- 注册个人多签和注册机构账户的动态阈值由 `votingengine/internal-vote` 的动态阈值表保存；NRC/PRC/PRB/FRG/NJD 固定治理阈值来自代码级固定阈值。
- 创世机构本体、主账户、费用账户和不可注销封存表由 `genesis-pallet/src/institution.rs` 在创世时写入 `public-manage`；固定治理机构初始管理员由同一文件写入 `public-admins`。
- 旧创世机构/管理员运行期模块已删除，不允许恢复为运行期治理模块或影子真源。

## 管理员集合目标字段

| 字段 | 说明 |
|------|------|
| `cid_number` | 管理员集合所属机构 CID 号；个人多签没有机构 CID 时为空。 |
| `institution_code` | 管理员集合所属机构码。 |
| `admins` | 当前有效管理员钱包账户集合，机构类不再内嵌 `AdminProfile`。 |
| `status` | 管理员集合生命周期状态。 |

机构岗位、任期、权限和任职来源不属于本表，统一读取 entity 的 `InstitutionRole` 与 `InstitutionAdminAssignment`。

## 机构任职来源边界

| 来源 | 写入语义 |
|------|----------|
| `Genesis` | 创世写入的固定治理机构管理员。 |
| `Registry` | 注册局或机构生命周期直接设置的管理员。 |
| `MutualElection` | 互选流程产生；流程尚未在本步骤实现。 |
| `PopularElection` | 普选流程产生；流程尚未在本步骤实现。 |
| `NominationAppointment` | 提名任免流程产生；本步骤只补齐枚举，不实现流程。 |

这些来源由 entity 的 `assignment_source` 保存；admins 不复制来源字段。

## 生命周期

- 公权机构生命周期由 `public-manage` 发起，只写 `public-admins`；固定治理机构由创世写入，同样只在 `public-admins` 承担运行期管理员治理。
- 私权机构生命周期由 `private-manage` 发起，只写 `private-admins`。
- 个人多签账户生命周期由 `personal-manage` 发起，只写 `personal-admins`；管理员更换 call 为 `PersonalAdmins(29).propose_admin_set_change(0)`。
- 公权/私权机构创建时，entity 模块只把机构 `cid_number` 和管理员钱包账户集合交给对应 admins 模块；对应的 `Registry` 任职来源由 entity 任职关系保存。
- 公权/私权管理员集合更换由投票引擎产出结果；admins 更新账户集合，entity 写入或终止对应岗位任职，admins 不复制投票或任职来源。
- 国家储委会、省储委会、省储行、国家司法院固定人数；国家司法院固定 15 人、阈值 8/15，其中 7 名护宪大法官用于修宪终审 4/7 表决；联邦注册局按 43 个省级 5 人组治理。
- 联邦注册局管理员更换必须走省级组入口：目标省 5 人组内部投票，阈值来自代码级固定阈值 `FRG=3`；不允许再用全 FRG 215 人平铺集合发起换届。
- FRG 主机构账户在读侧可聚合 43 个省级组，用于验签和身份展示；管理员更换投票根账户是链端按省码派生的省级组虚拟账户。
- 所有管理员集合变更仍经 `votingengine` 内部投票；各管理员模块用自己的 `MODULE_TAG` 绑定提案 owner。
- **固定治理骨架加锚（档 A，ADR-027 §6.4）**：NJD 护宪恰 7 席由 `public-admins::ensure_court_composition` runtime 强制（变更 + 执行终态，Error `InvalidCourtComposition`），并由节点 `core/governance_skeleton.rs` 守卫逐块背书（I1..I7：固定机构存在性/机构码/类型/Active/名额/NJD 护宪 7，setCode 改不动）。护宪 role 字面量 `ADMIN_ROLE_CONSTITUTION_GUARD` re-export 自 `primitives::governance_skeleton::ROLE_CONSTITUTION_GUARD`（与创世 role-by-index、节点守卫逐字节共用）。**只冻席位数不冻成员**：等长换人保持 7 席即放行。

## MODULE_TAG

| 模块 | MODULE_TAG |
|------|------------|
| `public-admins` | `b"pub-adm1"` |
| `private-admins` | `b"pri-adm1"` |
| `personal-admins` | `b"per-mgmt"` |

## Call Index

| 模块 | 管理员更换 call |
|------|----------------|
| `public-admins` | `27.0 propose_admin_set_change`（NRC/PRC/PRB/NJD 与普通公权机构；FRG 主账户禁止走本入口） |
| `public-admins` | `27.2 propose_federal_registry_province_admin_set_change`（FRG 省级组） |
| `private-admins` | `28.0 propose_admin_set_change` |
| `personal-admins` | `29.0 propose_admin_set_change` |

## 验证命令

```bash
cargo check --manifest-path citizenchain/Cargo.toml -p node
cargo test --manifest-path citizenchain/Cargo.toml -p public-admins -p private-admins -p personal-admins -p public-manage -p private-manage -p personal-manage -p multisig-transfer --lib
cargo test --manifest-path citizenchain/Cargo.toml -p citizenchain --lib
```
