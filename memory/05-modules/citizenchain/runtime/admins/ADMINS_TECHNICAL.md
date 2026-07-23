# runtime admins 技术文档

最新更新：2026-07-20。`citizenchain/runtime/admins/` 由四个 crate 组成，管理员链上状态按管理员类型分别保存在各自 pallet。

## 模块边界

| 模块 | 职责 |
|------|------|
| `admin-primitives` | 管理员共用类型、生命周期 trait、统一查询 trait 和机构码分类策略；不放业务 storage。 |
| `public-admins` | 公权机构管理员钱包集合，包括 NRC/PRC/PRB/FRG/NJD 固定治理机构；不保存岗位或 FRG 虚拟省组。 |
| `private-admins` | 私法人及私权侧/独立非法人机构管理员：公司、协会、私立学校、个体经营、无限合伙等；公法人下属非法人不得被描述为私法人附属类型。 |
| `personal-admins` | 个人多签管理员和个人多签管理员集合变更。个人多签账户生命周期归 `runtime/entity/personal-manage`。 |

## 唯一真源

- 机构管理员集合由 `admin-primitives::InstitutionAdmins` 表达；storage key 是 `cid_number`，value 只含 `institution_code + admins`。
- 公权、私权机构和个人多签每项统一为 `Admin { account_id, cid_number, family_name, given_name }`。非空 CID 必须是 CTZN 且与 `citizen-identity` 的 CID↔账户双向索引完全一致。admins pallet 不复制公民身份真源，也不用占位姓名伪造真实资料。
- 姓、名只展示，账户用于人员识别与签名，但账户本身没有机构业务权限；统一结构不保存岗位、权限、任期或任职来源。字段完整性由机构类型、岗位和个人多签规则分别判定。
- 机构岗位定义和任职关系归 `entity`，与管理员人员集合独立：管理员可以没有岗位，岗位可以空缺；岗位变化不得反向生成、删除或覆盖 admins。
- 个人多签继续由 `personal-admins` 独立管理业务和 storage，但管理员项与机构使用同一个 `Admin` 三字段结构；不使用机构岗位或机构任职关系。
- 各类管理员的链上管理员集合分别保存在各自 pallet 的 `AdminAccounts`。
- runtime 当前通过 `RuntimeAdminAccountQuery` 聚合读取各管理员模块；ADR-039 落地后它只能证明人员属于 admins，业务 pallet 还必须通过 entity 解析完整 `RoleSubject` 的权限和有效任职。
- 机构治理阈值由 public/private entity 的 `InstitutionGovernanceThresholds[cid_number]` 独立保存，与 admins 人数、岗位数解耦；admins pallet 不接收阈值。个人多签继续使用 `internal-vote::ActivePersonalThresholds[personal_account]`。任何路径都不建立岗位阈值。
- 创世机构本体、协议账户、机构阈值、默认法定代表人岗位、固定岗位、固定权限和创世任职由 `runtime/genesis/src/institution/seeder.rs` 唯一写入：既有公权机构写 `public-manage` / `public-admins`，公民链技术发展基金会写 `private-manage` / `private-admins`；不得跨命名空间或从岗位伪造钱包。
- 旧创世机构/管理员运行期模块已删除，不允许恢复为运行期治理模块或影子真源。

## 管理员集合目标字段

| 字段 | 说明 |
|------|------|
| `cid_number` | 管理员集合所属机构 CID 号；个人多签没有机构 CID 时为空。 |
| `institution_code` | 管理员集合所属机构码。 |
| `admins` | 当前管理员人员集合；公权、私权机构和个人多签每项字段顺序统一为 `account_id + cid_number + family_name + given_name`。 |

私权机构和个人多签姓名缺失时，runtime 分别补为 `family_name="管理"`、`given_name="员"`。公权身份字段可保持空值，不填占位值。人员去重读取 `admin_account`；机构业务授权和投票资格必须继续校验完整 `RoleSubject`、岗位权限与有效任职。

机构岗位、任期、权限和任职来源不属于本表，统一读取 entity 的 `InstitutionRole` 与 `InstitutionAdminAssignment`。

## 机构任职来源边界

| 来源 | 写入语义 |
|------|----------|
| `Genesis` | 创世直接写入的固定岗位任职。 |
| `Registry` | 注册局登记或后续任免流程形成的岗位任职。 |
| `MutualElection` | 互选结果经选举业务模块复核后形成 entity 任职结果。 |
| `PopularElection` | 普选结果经选举业务模块复核后形成 entity 任职结果。 |
| `NominationAppointment` | 提名任免最终结果；当前只有强类型来源，尚无合法流程生产者。 |

这些来源由 entity 的 `assignment_source` 保存；admins 不复制来源字段。

## 生命周期

- 公权机构生命周期由 `public-manage` 发起，只写 `public-admins`；固定治理机构由创世写入，同样只在 `public-admins` 承担运行期管理员治理。
- 私权机构生命周期由 `private-manage` 发起，只写 `private-admins`。
- 个人多签账户生命周期由 `personal-manage` 发起，只写 `personal-admins`；管理员更换 call 为 `PersonalAdmins(29).propose_admin_set_change(0)`。
- ADR-039 目标中，公权/私权机构创建必须在同一原子操作中建立 admins、强制 LR、至少一个初始治理岗位及权限、初始任职和初始投票规则；禁止只创建人员名单和空 LR 后赋予管理员临时启动权限。
- 已完成业务把通用机构治理结果交给 entity；entity 只更新岗位、任职和法定代表人，且任职目标必须是该机构既有管理员。
- admins 不接收岗位结果、不解释岗位变化，也不保存任职来源；管理员的新增、删除、换人和姓名更新由独立管理员维护流程处理。
- 机构管理员名册更换不得附带阈值变更；机构阈值只由对应 entity 配置，任职结果同样不能隐式改阈值。
- 国家储委会、省储委会、省储行、国家司法院固定人数；国家司法院岗位为 7 护宪、1 首席、2 次席、5 大法官。
- FRG 在 `public-admins` 只有一个含 215 个钱包的机构管理员集合；43 个省专员岗位、每岗5人的分组真源在 entity 任职 storage，不存在虚拟省组账户。省域业务只能授权对应省专员 `RoleSubject`，不能授权 FRG 全体 admins。
- 岗位任职不得驱动管理员更换；public/private admins 当前不暴露对外管理员集合变更 extrinsic，管理员维护入口在第2步单独实现。
- ADR-039 目标 NodeGuard 保护强制 LR 和创世固定岗位的码、名、固定权限及制度席位；创世机构可以依法增加普通动态岗位，不能再因存在额外岗位而拒绝。固定岗位成员可依法原子轮换。
- 私权创世的公民链基金会固定为一名程伟管理员、三个固定岗位各一席和机构阈值 2；同一程伟钱包同时任职三岗。固定岗位不能改名、停用、增删或扩席，但不能为了满足阈值而伪造多个钱包或降低阈值。
- `public-admins`、`private-admins` 没有 `WeightInfo` 和 `weights.rs`；其写入仅由 entity 生命周期内部接口调用。
- 正式创世只接受统一四字段 `Admin` SCALE 布局；旧纯账户、旧三字段记录和 runtime storage migration 均已删除，不保留兼容路径。

## MODULE_TAG

| 模块 | MODULE_TAG |
|------|------------|
| `personal-admins` | `b"per-mgmt"` |

## Call Index

| 模块 | 管理员更换 call |
|------|----------------|
| `personal-admins` | `29.0 propose_admin_set_change` |

## 验证命令

```bash
cargo check --manifest-path citizenchain/Cargo.toml -p node
cargo test --manifest-path citizenchain/Cargo.toml -p public-admins -p private-admins -p personal-admins -p public-manage -p private-manage -p personal-manage -p multisig --lib
cargo test --manifest-path citizenchain/Cargo.toml -p citizenchain --lib
```
