# number/ — 身份 ID 编码协议

- 最后更新:2026-06-25
- 任务卡:
  - `memory/08-tasks/done/20260603-cid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260604-cid-core-number-store-refactor.md`
  - `memory/08-tasks/open/20260607-cid-number-protocol.md`
  - `memory/08-tasks/done/20260612-181650-重构-cid-私权机构架构-保留身份id格式-私权机构按个体经营-合伙企业-股权公司-股份公司-公益组织-注册协.md`
  - `memory/08-tasks/open/20260614-cid-education-classification.md`
  - `memory/08-tasks/open/20260620-unify-cid-fields.md`

## 定位

- 路径:`citizencode/backend/number/`
- 职责:提供 CID 号生成、解析、格式校验和按机构码派生的分类封装。
- 非职责:不维护国家码、省级行政区码、机构码第二真源;不保存市、镇和地址段数据。

国家码、省级行政区码、机构码的唯一常量真源在
`citizenchain/runtime/primitives/src/code.rs`。`number/code.rs` 只做薄封装并继续服务
CID 号生成、解析和校验;不得恢复第二份机构码枚举、第二份 `ALL` 码表或
`label/value/name/code` 泛化字段。

市、镇和地址段数据仍由 `citizencode/backend/china/china.sqlite` 管理。`number::generator`
生成号码时通过 `crate::china::{province_code_by_name, city_code_by_name}` 取 R5 段;其中
`province_code_by_name` 最终引用 runtime primitives 的 `ProvinceCodeInfo`。

## 模块结构

```text
citizencode/backend/number/
├── mod.rs
├── code.rs
├── category.rs
├── generator.rs
├── validator.rs
├── model.rs
└── admin.rs
```

- `code.rs`:引用 runtime primitives 的国家/省/机构代码常量和机构码谓词,不保存第二份码表。
- `category.rs`:机构分类枚举与分类函数,分类一律由机构码派生。
- `generator.rs`:CID 号码生成入口 `generate_cid_number`。
- `validator.rs`:CID 号码格式校验、校验位计算与协议字段拆分。
- `model.rs`:管理端编码元信息 DTO。
- `admin.rs`:管理端编码元信息接口,路由为 `/api/v1/admin/number/meta`。

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

- 编码协议统一通过 `crate::number::*` 引用。
- 机构码分类、盈利策略、行政层级统一通过 `crate::number::code::*` 引用,其内部必须引用
  `primitives::code`。
- 行政区划运行数据统一通过 `crate::china::*` 引用;省级代码不得在 `china` 或 `number` 内手写第二份。
- 不得恢复 `citizencode/backend/cid_number/`、`citizencode/backend/citizencode/`、`province.rs`、
  `cities.rs`、`city_codes/*.rs` 或 `number/code.rs`。

## 验收口径

```text
test ! -d citizencode/backend/cid
test ! -d citizencode/backend/cid_number
test -d citizencode/backend/number
test -d citizencode/backend/china
rg "历史主体属性字段|历史身份字段别名" citizencode/backend memory/05-modules/citizencode
rg "第二份机构码表|第二份省码表" citizencode/backend/number memory/05-modules/citizencode
cd citizencode/backend && cargo check
```
