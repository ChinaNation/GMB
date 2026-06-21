# subjects/ — CID 身份主体共享边界

- 最后更新:2026-06-18
- 任务卡:
  - `memory/08-tasks/done/20260603-cid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260612-181650-重构-cid-私权机构架构-保留身份id格式-私权机构按个体经营-合伙企业-股权公司-股份公司-公益组织-注册协.md`
  - `memory/08-tasks/done/20260612-194131-cid-private-real-module-refactor.md`
  - `memory/08-tasks/open/20260613-cid-institution-list-audit-accounts.md`
  - `memory/08-tasks/open/20260614-cid-education-classification.md`
  - `memory/08-tasks/done/20260618-cid-gov-admin-division-reconcile.md`

## 定位

`citizencode/backend/subjects/` 承接法人和非法人主体的共享模型、主体详情、名称检查、
链端公开查询和目标分区表结构。它不是“机构业务总目录”;公权机构业务归 `gov/`,
私权机构业务归 `private/`,账户归 `accounts/`,资料库归 `docs/`。

身份边界:

- 唯一且不可变身份只认 `cid_number`。
- `ids` 表只做全局唯一约束,不是第二身份键。
- 不新增 `identity_key`、`generation_key` 或任何派生身份键。
- 自动公权/宪法机构由后端对账维护;`JY` 教育机构统一归教育机构分类展示。
- `education_type` 只表达教育业务分类,不参与 `cid_number` 生成。
- `parent_cid_number` 只是指向另一个机构 `cid_number` 的从属关系引用,不得理解为第二套身份ID。
- `legal_rep_cid_number` 只能保存正常状态公民的 `cid_number`;法定代表人没有第二套身份ID规则。

## 私权机构分类

私权机构的身份号码结构不变,但私权类型不再借用旧的企业细分字段或 `ZG/TG` 表达。
目标分类统一由 `private_type` 表达:

| private_type | 中文 | subject_property + T2 | 法人资格 |
|---|---|---|---|
| `SOLE` | 个体经营 | `F+GT` | 无 |
| `PARTNERSHIP` + `GENERAL` | 无限合伙 | `F+GP` | 无 |
| `PARTNERSHIP` + `LIMITED` | 有限合伙 | `S+LP` | 有 |
| `COMPANY` | 股权公司 | `S+GQ` | 有 |
| `CORPORATION` | 股份公司 | `S+GF` | 有 |
| `WELFARE` | 公益组织 | `S+GY` | 有 |
| `ASSOCIATION` | 注册协会 | `S+AS` | 有 |

`private/common` 是规则单一来源,创建私权机构时后端根据 `private_type` 与
`partnership_kind` 锁定 `subject_property / institution_code / p1 / has_legal_personality`。

## 非法人

非法人机构能力统一放在 `citizencode/backend/subjects/uninorg/`。

- `F` 不具备独立法人资格。
- `F+GT` 个体经营和 `F+GP` 无限合伙是独立非法人,不选择所属法人。
- 其它从属非法人必须从属于一个具备法人资格的主体。
- 公权机构和私权机构都可能拥有从属非法人机构,所以能力不能放在 `gov/` 或 `private/` 单侧目录。

## 教育机构分类

教育机构统一使用 `institution_code=JY` 进入教育机构 tab,`subject_property/p1/JY`
继续按既有 CID 号码规则生成身份 ID,不得为了教育阶段修改号码协议。

`subjects.education_type` 是教育业务分类:

| education_type | 含义 | 创建来源 |
|---|---|---|
| `NATIONAL_CITIZEN_EDU_COMMITTEE` | 国家公民教育委员会 | 确定性目录 |
| `CITY_CITIZEN_EDU_COMMITTEE` | 市公民教育委员会 | 确定性目录 |
| `EARLY_SCHOOL` | 初学 | 新增 G/S 学校时选择 |
| `PRIMARY_SCHOOL` | 小学 | 新增 G/S 学校时选择 |
| `SECONDARY_SCHOOL` | 中学 | 新增 G/S 学校时选择 |
| `UNIVERSITY` | 大学 | 新增 G/S 学校时选择 |

规则:

- 国家/市公民教育委员会从公权目录移出;教育机构市详情空搜索只直接显示本市市公民教育委员会,国家公民教育委员会不得跨市铺开。
- `G+JY`/`S+JY` 是法人教育机构;`F+JY` 是挂靠法人教育机构的非法人教育分支,按名称或身份ID精确搜索后显示。
- G/S/F 教育机构创建统一按管理员省市 scope 控制:联邦注册局机构管理员可在本省任意市创建,市注册局机构管理员只能在本市创建。
- 学校内部部门不写入 `subjects`,不生成 `cid_number`,不创建账户,不参与法定代表人校验。

## 法定代表人公民范围

法定代表人选择范围由 `subjects::service::resolve_legal_representative_scope_*`
实时推导,不落库新字段:

| 目标机构 | 法定代表人范围 |
|---|---|
| 普通私法人机构、私法人学校 `S+JY`、挂靠私法人的非法人机构/分校 | 全国正常状态公民 |
| 公法人机构 `G`、公安局、市注册局等公权机构 | 按机构行政层级限制 |
| 挂靠公法人的非法人机构/分校 `F` | 按该非法人机构自身落位省市限制 |
| 国家级/部级/联邦级公权机构(`NATIONAL_`/`MINISTRY_`/`FEDERAL_`) | 全国正常状态公民 |
| 省级公权机构(`PROVINCE_`) | 本省正常状态公民 |
| 市/镇级、手动公权机构和未知公权前缀 | 本市正常状态公民;无市码时退为本省 |

执行口径:

- 后台创建机构和更新机构资料时,必须调用同一套 scope 做最终校验;前端搜索结果不能作为可信依据。
- `citizens` 模块只按传入 scope 查询正常公民,不自行决定机构规则。
- `target_cid_number` 模式必须以数据库中现有机构为准;创建模式必须提交目标省、市、主体属性、机构代码和所属法人。

## 自动目录

- 国家/省级政府、立法院、司法院、监察院、教育委员会、储备委员会、储备银行读取
  `citizenchain/runtime/primitives/china/china_*.rs` 常量中的 `cid_number`。
- 市级自治政府、市立法会、市司法院、市监察院、市教育委员会按
  `citizencode/backend/china/china.sqlite` 的行政区划生成。
- 行政区划唯一真源是 `citizencode/backend/china/`;CID 编码协议目录不再维护省市静态表。
- 市级自动机构对账匹配键只在内存中用于保持原 `cid_number` 不变,不得落库为第二身份。
- 自动目录写入 `gov.source='GENERATED'`;手动公权机构写入 `MANUAL`。行政区对账清理
  obsolete 时只允许删除 `GENERATED` 派生行及其账户、资料、索引和审计残留,不得删除手工公权机构。
- 确定性公权目录简称必须写入规范短名,例如住建部、国储会、省储会、省储行;
  不得把全称重复写入 `cid_short_name`。

## 默认账户

- `subjects::service` 是机构默认账户名称规则的后端单一来源。
- 普通机构默认账户为“主账户 / 费用账户”。
- 省公民储备银行默认账户为“主账户 / 费用账户 / 永久质押”。
- 国储会默认账户为“主账户 / 费用账户 / 安全基金 / 两和基金”。
- 上述默认账户都属于制度保留账户,账户列表展示但不可按普通自定义账户删除。

## 路由归属

后台管理外部路径保持稳定,内部源码归属已经拆分:

| 内部模块 | 职责 |
|---|---|
| `subjects::admin` | 主体详情、名称检查、链状态同步入口 |
| `subjects::registration` | 公权/教育通用机构注册和列表内核;私权六类模块只调用其私权专用内核 |
| `gov::handler` | 公安局和公权机构确定性列表 |
| `private::sole` / `private::partnership` / `private::company` / `private::corporation` / `private::welfare` / `private::association` | 六类私权机构创建、校验和精确查询 |
| `accounts::handler` | 机构账户 CRUD |
| `docs::handler` | 机构资料库 CRUD |
| `subjects::chain_duoqian_info` | 区块链软件和钱包公开查询 |

外部公开查询路径仍保留 `/api/v1/app/institutions/...` 作为协议路径,不得据此恢复
`backend/institutions/` 源码目录。

## 文件结构

```text
citizencode/backend/subjects/
├── mod.rs
├── model.rs
├── schema.rs
├── store.rs
├── service.rs
├── admin.rs
├── chain_duoqian_info.rs
└── uninorg/
    └── mod.rs
```

## 验收口径

```text
test ! -d citizencode/backend/institutions
rg "crate::institutions|mod institutions" citizencode/backend -g '*.rs'
cd citizencode/backend && cargo check
```
