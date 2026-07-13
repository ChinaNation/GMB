# 任务卡：机构、岗位与管理员链上模型收口

## 当前状态

- 状态：进行中
- 当前步骤：第一步已完成，等待用户确认第二步 runtime 技术方案
- 最新业务确认：2026-07-12
- 实施方式：逐步输出技术方案，用户确认后才执行；每步完成后立即更新文档、完善中文注释、删除残留，再输出下一步方案

## 任务目标

将机构信息、机构岗位、机构管理员任职与管理员集合彻底拆分到正确模块：

- 机构信息、机构岗位、岗位权限和机构管理员任职归 `entity`。
- 机构管理员钱包账户集合 `admins` 及其生命周期归 `admins`。
- 投票引擎只决定普选、互选、提名任免等任职结果，不保存第二份管理员或岗位真源。
- 所有机构都必须有链上公开法定代表人。
- 个人多签及 `personal-admins` 完全排除在本机构岗位模型之外。

## 强制领域关系

```text
公民 cid_number 1 ─── 1 钱包账户

机构 1 ─── N 机构岗位
机构 N ─── N 管理员钱包账户
机构 + 管理员钱包账户 + 机构岗位 = 机构管理员任职
```

- 一个公民只有一个 `cid_number`，且只能绑定一个钱包账户。
- 一个钱包账户只能绑定一个公民 CID。
- 一个机构管理员就是一个取得机构管理资格的钱包账户，不新建管理员身份 ID。
- 一个管理员钱包账户可在多个机构任职，同一机构可有多个管理员。
- 管理员的当前机构权限由“机构 + 有效岗位 + 有效任职 + 岗位权限”确定。

## 当前权威字段契约

### 机构信息

`InstitutionInfo` 保留当前机构公开信息，并对所有公权、私权、创世、非创世机构强制增加：

| 字段 | 中文注释 |
|---|---|
| `legal_representative_name` | 法定代表人公开姓名 |
| `legal_representative_cid_number` | 法定代表人唯一公民 CID |
| `legal_representative_account` | 法定代表人唯一钱包账户 |

目标结构废弃 `legal_rep_name` 和 `legal_rep_cid_number`，全仓统一使用 `legal_representative_*`。法定代表人照片、联系方式和原始身份档案不上链。

### 机构岗位

`InstitutionRole` 归 `entity`：

| 字段 | 中文注释 |
|---|---|
| `role_code` | 机构内唯一岗位代码 |
| `role_name` | 岗位名称 |
| `role_permissions` | 岗位权限代码集合 |
| `term_required` | 该岗位是否强制任期 |
| `role_status` | 岗位是否有效 |

### 机构管理员任职

`InstitutionAdminAssignment` 归 `entity`：

| 字段 | 中文注释 |
|---|---|
| `cid_number` | 任职机构 CID |
| `admin_account` | 管理员唯一钱包账户 |
| `role_code` | 在该机构担任的岗位 |
| `term_start` | 任期开始日期，自纪元以来天数 |
| `term_end` | 任期结束日期，自纪元以来天数 |
| `assignment_source` | 任职制度来源 |
| `assignment_source_ref` | 选举、投票、登记或任免记录 ID |
| `assignment_status` | 任职是否有效 |

`assignment_source` 只允许：

- `Genesis`
- `Registry`
- `PopularElection`
- `MutualElection`
- `NominationAppointment`

任职不保存 `creator`；来源由 `assignment_source + assignment_source_ref` 唯一表达。

### 管理员集合

`public-admins` 和 `private-admins` 中的目标字段为：

```text
admins: BoundedVec<AccountId>
```

- 不再内嵌 `AdminProfile`。
- 管理员集合目标记录不保存 `creator`、`created_at`、`updated_at`；链上来源和时间由对应任职关系、事件及区块确定。
- 不保存 `admin_name`、`admin_cid_number`、`role_code`、`role_name`、`term_start`、`term_end`、`admin_source`、`admin_source_ref`。
- 当前 runtime 中的 `AdminProfile` 是待拆除旧实现，不是目标契约。

## 信任与隐私边界

- 普通公民的原始实名档案、护照号、出生日期、住址等非公开信息不上链。
- 机构法定代表人、机构岗位任职和竞选资料属于依法公开或主动公开的身份事实，可以上链。
- 所有机构都必须有法定代表人，三个统一字段必须上链。

## 实施步骤

1. 冻结领域模型、字段命名、协议和任务边界。
2. 实现 runtime 基础数据结构。
3. 实现创世机构、岗位和任职。
4. 实现运行期机构注册和任职写入。
5. 改造 OnChina。
6. 改造 CitizenApp 和公民钱包。
7. 全仓残留清理、重新创世和真实运行态验收。

## 各步确认规则

- 每一步必须先输出完整技术方案和预计修改目录。
- 用户确认后才能执行该步骤。
- 涉及 `citizenchain/runtime/` 的每一步都必须单独获得 runtime 二次确认。
- 每步代码执行完成后，必须立即更新文档、完善中文注释、删除旧代码、旧字段、旧注释、旧协议和旧文档口径。

## 第一步执行记录

- [x] 修正“链上不保存真实身份”过宽口径，区分普通公民隐私与依法公开身份。
- [x] 确认所有机构的法定代表人三个字段必须上链。
- [x] 确认机构岗位和任职关系归 `entity`，管理员账户集合归 `admins`。
- [x] 确认任职只记录制度来源，不存在 `creator`。
- [x] 确认个人多签完全排除在本任务的机构岗位模型之外。
- [x] 统一登记新字段命名和目标协议。

## 历史实现事实

2026-06-28 至 2026-06-30 曾经实现机构 `AdminProfile`，将管理员姓名、CID、岗位、任期和来源内嵌到 `AdminAccounts.admins`，并同步实现 OnChina、CitizenApp 和公民钱包解码。该布局已被 2026-07-12 用户确认的目标模型取代，后续步骤必须彻底删除相关代码、协议、缓存、注释和展示残留。

## 完成标准

- runtime 中不再存在机构管理员 `AdminProfile` 内嵌布局。
- 所有机构都有可查询的链上法定代表人三字段。
- 机构岗位、岗位权限和机构管理员任职关系有唯一 entity 真源。
- `admins` 只保存管理员钱包账户集合。
- 一个管理员可在多个机构任职，一个机构可有多个管理员。
- 无有效岗位任职或任期失效的账户不具有对应机构权限。
- 个人多签行为和存储不受本机构岗位改造影响。
- OnChina、CitizenApp、公民钱包与 runtime SCALE 字节完全一致。
- 重新创世后通过真实节点、真实 PostgreSQL、真实 HTTP、真实页面和真实冷签验收。
