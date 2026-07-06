# 管理员任职字段与机构岗位制度模型

## 任务需求

按当前仓库架构重构管理员与岗位的边界：

- 管理员是具体的人，拥有账户、姓名、个人 CID、任期和来源；第 2 步先沿用管理员集合状态，不给单个管理员另建状态真源。
- 岗位属于机构侧制度，定义某机构有哪些岗位、岗位名称、岗位权限、产生方式和人数约束。
- 公权机构管理员唯一链上真源仍在 `public-admins`。
- 私权机构管理员唯一链上真源仍在 `private-admins`。
- 机构岗位制度由 entity 模块承载，公权放 `public-manage`，私权放 `private-manage`。
- OnChina 本地库只保存链下私密资料、草稿、操作审计和链投影，不作为管理员真源。

用户要求先实现管理员侧，再实现机构侧。每一步开始前必须先输出完整技术方案，得到用户明确同意后才能执行。

## 核心边界

### entity 模块负责

- 机构本体真源。
- 机构岗位定义。
- 岗位权限定义。
- 岗位产生方式。
- 岗位人数和任期规则。

### admins 模块负责

- 管理员任职事实。
- 管理员账户资格。
- 管理员姓名、个人 CID、任期和来源。
- 管理员是否属于某机构某岗位。
- 管理员集合 active 真源。

### OnChina 负责

- 录入、展示、二维码、冷签流程和链上提交辅助。
- 本地私密资料、操作日志和链投影。
- 不得把本地 `admins` 或 `institution_admins` 当作管理员真源。

## 统一字段方案

### 管理员任职字段

管理员字段放在对应 admins 模块的链上管理员集合中。公权机构写 `public-admins`，私权机构写 `private-admins`。

| 字段 | 注释 |
|---|---|
| `admin_account` | 管理员链上账户。登录、签名、投票和多签资格都认这个字段。 |
| `admin_cid_number` | 管理员个人 CID 号，作为实名锚。 |
| `admin_name` | 管理员姓名快照。目标语义应统一使用 `admin_name`，避免继续扩散泛化 `name`。 |
| `role_code` | 管理员担任的岗位代码，引用 entity 模块中的机构岗位定义。 |
| `role_name` | 管理员担任的岗位名称快照，用于展示和历史留痕。 |
| `term_start` | 任期开始。 |
| `term_end` | 任期结束；无任期为 0。 |
| `admin_source` | 该管理员任职事实的产生方式。 |
| `admin_source_ref` | 来源追溯 ID，如创世批次、注册局操作、内部投票提案、选举记录或提名任免记录。 |

管理员集合级字段仍由 `AdminAccount` 承载：

| 字段 | 注释 |
|---|---|
| `cid_number` | 管理员集合所属机构 CID 号；个人多签为空。 |
| `institution_code` | 管理员集合所属机构码，如 `CREG`、`NLG` 或私权机构码。 |
| `kind` | 管理员集合类型：公权机构、私权机构或个人多签。 |
| `admins` | 完整管理员任职列表。 |
| `creator` | 创建或提交该集合记录的账户。 |
| `created_at` | 创建区块。 |
| `updated_at` | 最近更新区块。 |
| `status` | 集合状态：`Pending`、`Active`、`Closed`。 |

### 管理员来源枚举

| 值 | 注释 |
|---|---|
| `Genesis` | 创世写入。 |
| `Registry` | 注册局设置。 |
| `InternalVote` | 机构内部管理员投票产生。 |
| `MutualElection` | 互选产生。 |
| `PopularElection` | 普选产生。 |
| `NominationAppointment` | 提名任免产生。当前仓库尚未实现，需要在管理员侧方案中设计。 |

### 机构岗位字段

岗位字段由 entity 模块定义，不属于管理员模块。公权机构岗位放 `public-manage`，私权机构岗位放 `private-manage`。

| 字段 | 注释 |
|---|---|
| `cid_number` | 岗位所属机构 CID 号。 |
| `role_code` | 岗位代码，机构内唯一。 |
| `role_name` | 岗位名称，如参议员、经理、财务、局长。 |
| `role_source` | 岗位制度上的产生方式。 |
| `role_permissions` | 岗位权限集合，只定义该岗位能做什么，不定义谁担任。 |
| `min_count` | 该岗位最少人数。 |
| `max_count` | 该岗位最多人数。 |
| `term_days` | 标准任期天数；无固定任期为 0。 |
| `role_status` | 岗位状态，如 `Active`、`Disabled`。 |

## 分步实现建议

### 第 1 步：管理员侧技术方案

只输出完整技术方案，不改代码。

需要明确：

- 当前 `AdminProfile` 字段如何迁移到统一管理员任职字段。
- `name` 是否一次性改为 `admin_name`。
- `admin_role` 是否改为 `role_name`，以及是否新增 `role_code`。
- `source` 是否改为 `admin_source`。
- 是否新增 `admin_source_ref` 和 `admin_status`。
- `NominationAppointment` 枚举如何加入并同步 SCALE 编码。
- `public-admins`、`private-admins`、OnChina、CitizenWallet、CitizenApp 的协议影响。

确认结论：

- 第 2 步新增 `admin_source_ref`。
- 第 2 步不新增单管理员 `admin_status`；状态仍保持在 `AdminAccount.status` 集合级。
- 第 2 步新增 `NominationAppointment` 枚举但不实现提名任免流程。

### 第 2 步：管理员侧 runtime 实现

用户确认第 1 步方案后执行。

预计涉及：

- `citizenchain/runtime/admins/admin-primitives/`
- `citizenchain/runtime/admins/public-admins/`
- `citizenchain/runtime/admins/private-admins/`
- 必要的 runtime 聚合查询适配。

目标：

- 完成管理员任职字段的链上结构调整。
- 保持公权管理员真源只在 `public-admins`。
- 保持私权管理员真源只在 `private-admins`。
- 个人多签只做 trait 签名兼容和集合级 `cid_number` 空值，不扩展为机构岗位模型。

执行记录：

- 已完成 `AdminProfile` 字段统一：`admin_account`、`admin_cid_number`、`admin_name`、`role_code`、`role_name`、`term_start`、`term_end`、`admin_source`、`admin_source_ref`。
- 已完成 `AdminAccount` 集合级机构 CID：新增 `cid_number`。
- 已补齐 `AdminSource::NominationAppointment` 枚举；本步骤不实现提名任免流程。
- 创世写入来源为 `Genesis`。
- 注册局/机构生命周期直写来源由 admins 模块强制落为 `Registry`。
- 管理员集合更换提案来源由 admins 模块强制落为 `InternalVote`。
- 公权机构管理员 active 真源仍只在 `public-admins`。
- 私权机构管理员 active 真源仍只在 `private-admins`。
- 本步骤未实现岗位制度和注册局越权强校验，留到第 4-6 步。

验证记录：

- `cargo test --manifest-path citizenchain/Cargo.toml -p public-admins -p private-admins -p personal-admins -p public-manage -p private-manage -p personal-manage --lib`
- `cargo test --manifest-path citizenchain/Cargo.toml -p citizenchain --lib`

### 第 3 步：管理员侧客户端和 OnChina 同步

用户确认第 3 步方案后执行。

预计涉及：

- `citizenchain/onchina/`
- `citizenwallet/`
- `citizenapp/`

目标：

- OnChina 按新字段组装和解码管理员任职资料。
- CitizenWallet 冷签展示字段与 SCALE 解码同步。
- CitizenApp 展示管理员账户、姓名、个人 CID、岗位、任期和来源。
- 本地表只保留链下私密资料和链投影。

### 第 4 步：机构岗位侧技术方案

只输出完整技术方案，不改代码。

需要明确：

- 公权和私权岗位定义是否采用相同字段结构。
- 岗位权限 `role_permissions` 如何表达。
- 不同机构的岗位模板如何管理。
- 市注册局管理员上限 30 是机构级约束还是岗位级约束。
- 哪些岗位允许注册局设置，哪些必须走普选、互选或提名任免。

### 第 5 步：机构岗位侧 runtime 实现

用户确认第 4 步方案后执行。

预计涉及：

- `citizenchain/runtime/entity/public-manage/`
- `citizenchain/runtime/entity/private-manage/`

目标：

- 在 entity 模块保存机构岗位制度。
- 不在 entity 模块保存管理员本人。
- 提供 admins 模块可查询的岗位规则接口。

### 第 6 步：管理员写入强校验

用户确认第 6 步方案后执行。

预计涉及：

- `citizenchain/runtime/admins/public-admins/`
- `citizenchain/runtime/admins/private-admins/`
- 必要的 entity 查询 trait。

目标：

- admins 模块写管理员时读取 entity 岗位规则。
- `role_source = Registry` 的岗位允许注册局按辖区维护。
- `role_source = PopularElection`、`MutualElection`、`NominationAppointment` 的岗位拒绝注册局直接新增、删除、修改。
- `role_source = Genesis` 的岗位拒绝普通维护入口。
- FRG 只能维护自己分管省内允许注册局设置的公权岗位。
- CREG 只能维护本市允许注册局设置的公权岗位。
- CREG 管理员最多 30 人。

### 第 7 步：文档、残留清理和真实验收

用户确认第 7 步方案后执行。

目标：

- 更新模块技术文档和协议文档。
- 清理旧字段、旧注释、旧前端文案和旧本地真源残留。
- 执行 runtime、OnChina、CitizenWallet、CitizenApp 的真实验收。
- 用真实本地服务和链上查询验证管理员写入、展示、登录和拒绝越权。

## 预计修改目录

- `citizenchain/runtime/admins/admin-primitives/`
  - 管理员共用字段、来源枚举和查询 trait。
- `citizenchain/runtime/admins/public-admins/`
  - 公权机构管理员任职事实真源和写入强校验。
- `citizenchain/runtime/admins/private-admins/`
  - 私权机构管理员任职事实真源和写入强校验。
- `citizenchain/runtime/entity/public-manage/`
  - 公权机构岗位制度定义。
- `citizenchain/runtime/entity/private-manage/`
  - 私权机构岗位制度定义。
- `citizenchain/onchina/`
  - 管理员录入、展示、链上 call data、链投影和本地私密资料。
- `citizenwallet/`
  - 冷签二维码解码和展示。
- `citizenapp/`
  - 管理员资料展示和链上读取。
- `memory/`
  - 任务卡、协议文档、模块技术文档和残留清理记录。

## 硬性执行规则

- 每一步开始前必须先输出完整技术方案。
- 技术方案必须得到用户明确同意后才能执行代码修改。
- 涉及 `citizenchain/runtime/` 的任何改动，必须再次单独列出完整路径、预计改动内容和原因，并得到 runtime 二次确认。
- 不允许通过 OnChina 本地表绕过链上管理员真源。
- 不允许把岗位权限和岗位产生制度塞进管理员字段。
- 不允许把具体管理员本人放进 entity 岗位定义。
- 不允许保留旧字段双轨兼容，除非用户在当前任务中明确要求。

## 验收标准

- 公权机构管理员 active 真源只在 `public-admins`。
- 私权机构管理员 active 真源只在 `private-admins`。
- entity 模块只定义岗位制度，不保存具体管理员本人。
- admins 模块保存具体管理员任职事实，并引用 entity 岗位定义。
- FRG/CREG 无法通过注册局维护入口修改普选、互选、提名任免或创世保护岗位。
- 市注册局管理员人数最多 30 的规则由链上强校验保证。
- OnChina、CitizenWallet、CitizenApp 字段命名和 SCALE 编码与 runtime 一致。
- 链上查询、登录、冷签、管理员列表展示和越权拒绝均完成真实验收。

## 进度

- [x] 需求边界确认：岗位属于机构侧，管理员属于任职人侧。
- [x] 创建任务卡。
- [x] 第 1 步：管理员侧技术方案。
- [x] 第 2 步：管理员侧 runtime 实现。
- [ ] 第 3 步：管理员侧客户端和 OnChina 同步。
- [ ] 第 4 步：机构岗位侧技术方案。
- [ ] 第 5 步：机构岗位侧 runtime 实现。
- [ ] 第 6 步：管理员写入强校验。
- [ ] 第 7 步：文档、残留清理和真实验收。
