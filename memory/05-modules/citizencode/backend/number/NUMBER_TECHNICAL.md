# cid/ — 身份 ID 编码协议

- 最后更新:2026-06-27
- 任务卡:
  - `memory/08-tasks/done/20260603-cid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260604-cid-core-number-store-refactor.md`
  - `memory/08-tasks/open/20260607-cid-number-protocol.md`
  - `memory/08-tasks/done/20260612-181650-重构-cid-私权机构架构-保留身份id格式-私权机构按个体经营-合伙企业-股权公司-股份公司-公益组织-注册协.md`
  - `memory/08-tasks/open/20260614-cid-education-classification.md`
  - `memory/08-tasks/open/20260620-unify-cid-fields.md`

## 定位

- 路径:`citizenchain/registry/src/cid/`
- 职责:提供 registry 运行态的 CID 号生成适配、动态种子、数据库查重、行政区 SQLite 查询、管理端元信息接口和按机构码派生的分类封装。
- 非职责:不维护国家码、省级行政区码、机构码第二真源;不保存市、镇和地址段数据。

国家码、省级行政区码、机构码的唯一常量真源在
`citizenchain/runtime/primitives/cid/code.rs`。CID 号解析、校验、核心生成规则和确定性种子协议在
`citizenchain/runtime/primitives/cid/{number,generator,seed}.rs`。registry 不再保存
`number/code.rs` 薄封装,不得恢复第二份机构码枚举、第二份 `ALL` 码表或 `label/value/name/code` 泛化字段。

市、镇和地址段数据仍由 `citizenchain/registry/src/cid/china/china.sqlite` 管理。`cid::generator`
生成号码时通过 `crate::cid::china::{province_code_by_name, city_code_by_name}` 取 R5 段,
再把省码、市码、名称和显式年份传入 runtime primitives 纯协议函数。

## 模块结构

```text
citizenchain/registry/src/cid/
├── mod.rs
├── category.rs
├── china/
├── generator.rs
├── model.rs
├── seed.rs
└── admin.rs
```

- `china/`:SQLite 行政区运行数据、只读查询和管理端城市接口。
- `category.rs`:机构分类枚举与分类函数,分类一律由机构码派生。
- `generator.rs`:registry 发号适配入口 `generate_cid_number`,负责查行政区和当前年份。
- `seed.rs`:动态 UUID、数据库查重和确定性种子调用入口。
- `model.rs`:管理端编码元信息 DTO。
- `admin.rs`:管理端编码元信息接口,路由为 `/api/v1/admin/cid/meta`。

## 生成规则摘要

- 编码段:`R5-SEG2-N9-D4`。
- `R5`:省码 + 市码;省码来自 primitives `ProvinceCodeInfo`,市码来自 `china.sqlite`。
- `SEG2`:恒 5 字符,按机构码长度分两种布局。
- 3 字符码布局:`机构码(3)+盈利位(1)+校验位(1,mod-36)`。
- 4 字符码布局:`机构码(4)+M1(1)`,M1 为盈利属性和校验位合一;数字表示盈利,字母表示非盈利。
- `N9`:稳定散列序列。
- `D4`:年份。
- 示例:`LN001-NRC0G-944805165-2026`。

规则:

- 国家/省级机构码和大学类机构使用 3 字符布局;市镇、公私权、个人和个人多签类机构码使用 4 字符布局。
- 机构码自身决定公法人、私法人、非法人、个人主体、教育机构、行政层级和盈利策略。
- `ProfitPolicy::NonProfit` 必须生成非盈利位;`ProfitPolicy::Profit` 必须生成盈利位;
  `ProfitPolicy::Variable` 由创建实例传入 `p1`;`ProfitPolicy::InheritParent` 继承父级法人盈利属性。
- 教育阶段、国家/市公民教育委员会分类由 `subjects.education_type` 表达,不进入
  `GenerateCidInput`,也不得改变机构码本身的 CID 生成语义。

私权目标类型映射:

| 类型 | 机构码 | 主体属性 |
|---|---|---|
| 个体经营 | `SFGT` | 非法人 |
| 无限合伙 | `SFGP` | 非法人 |
| 有限合伙 | `SFLP` | 私法人 |
| 股权公司 | `SFGQ` | 私法人 |
| 股份公司 | `SFGF` | 私法人 |
| 公益组织 | `SFGY` | 私法人 |
| 注册协会 | `SFAS` | 私法人 |

## 引用规则

- 编码协议统一通过 `crate::cid::*` 引用。
- 机构码分类、盈利策略、行政层级统一通过 `crate::cid::code::*` 引用,其内部必须引用
  `primitives::cid::code`。
- 行政区划运行数据统一通过 `crate::cid::china::*` 引用;省级代码不得在 `cid/china` 或 registry 其它目录内手写第二份。
- 不得恢复历史 CID 目录壳、旧 province/cities/city_codes 手写行政区文件、旧 registry number 模块或旧 registry 顶层 china 模块。

## 验收口径

```text
test ! -d citizencode/backend/cid
test ! -d citizencode/backend/cid_number
test -d citizenchain/registry/src/cid
test -d citizenchain/registry/src/cid/china
rg "历史主体属性字段|历史身份字段别名" citizencode/backend memory/05-modules/citizencode
rg "第二份机构码表|第二份省码表" citizenchain/registry/src/cid memory/05-modules/citizencode
cargo check -p registry --manifest-path citizenchain/Cargo.toml
```
