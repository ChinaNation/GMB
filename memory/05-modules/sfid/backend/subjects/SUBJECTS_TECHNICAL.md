# subjects/ — SFID 身份主体共享边界

- 最后更新:2026-06-12
- 任务卡:
  - `memory/08-tasks/done/20260603-sfid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260612-181650-重构-sfid-私权机构架构-保留身份id格式-私权机构按个体经营-合伙企业-股权公司-股份公司-公益组织-注册协.md`
  - `memory/08-tasks/done/20260612-194131-sfid-private-real-module-refactor.md`

## 定位

`sfid/backend/subjects/` 承接法人和非法人主体的共享模型、主体详情、名称检查、
链端公开查询和目标分区表结构。它不是“机构业务总目录”;公权机构业务归 `gov/`,
私权机构业务归 `private/`,账户归 `accounts/`,资料库归 `docs/`。

身份边界:

- 唯一且不可变身份只认 `sfid_number`。
- `ids` 表只做全局唯一约束,不是第二身份键。
- 不新增 `identity_key`、`generation_key` 或任何派生身份键。
- 自动公权/宪法机构由后端对账维护;手动教育委员会 `JY` 类型登记的是学校本体。

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

非法人机构能力统一放在 `sfid/backend/subjects/uninorg/`。

- `F` 不具备独立法人资格。
- `F+GT` 个体经营和 `F+GP` 无限合伙是独立非法人,不选择所属法人。
- 其它从属非法人必须从属于一个具备法人资格的主体。
- 公权机构和私权机构都可能拥有从属非法人机构,所以能力不能放在 `gov/` 或 `private/` 单侧目录。

## 自动目录

- 国家/省级政府、立法院、司法院、监察院、教育委员会、储备委员会、储备银行读取
  `citizenchain/runtime/primitives/china/china_*.rs` 常量中的 `sfid_number`。
- 市级自治政府、市立法会、市司法院、市监察院、市教育委员会按
  `sfid/backend/china/data/china.sqlite` 的行政区划生成。
- 行政区划唯一真源是 `sfid/backend/china/`;SFID 编码协议目录不再维护省市静态表。
- 市级自动机构对账匹配键只在内存中用于保持原 `sfid_number` 不变,不得落库为第二身份。

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
sfid/backend/subjects/
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
test ! -d sfid/backend/institutions
rg "crate::institutions|mod institutions" sfid/backend -g '*.rs'
cd sfid/backend && cargo check
```
