# number/ — 身份 ID 编码协议

- 最后更新:2026-06-14
- 任务卡:
  - `memory/08-tasks/done/20260603-cid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260604-cid-core-number-store-refactor.md`
  - `memory/08-tasks/open/20260607-cid-number-protocol.md`
  - `memory/08-tasks/done/20260612-181650-重构-cid-私权机构架构-保留身份id格式-私权机构按个体经营-合伙企业-股权公司-股份公司-公益组织-注册协.md`
  - `memory/08-tasks/open/20260614-cid-education-classification.md`

## 定位

- 路径:`citizencode/backend/number/`
- 职责:提供 CID 编码协议的主体属性、机构码、分类、生成和格式校验。
- 非职责:不维护行政区划静态表,不保存省、市、镇和地址段数据。

行政区划唯一真源在 `citizencode/backend/china/`。`number::generator`
只在生成号码时调用 `crate::china::{province_code_by_name, city_code_by_name}`。

## 模块结构

```text
citizencode/backend/number/
├── mod.rs
├── institution_code.rs
├── category.rs
├── generator.rs
├── validator.rs
├── model.rs
└── admin.rs
```

- `institution_code.rs`:机构类型枚举。
- `category.rs`:主体属性枚举、机构分类枚举与分类函数。
- `generator.rs`:CID 号码生成入口 `generate_cid_number`。
- `validator.rs`:CID 号码格式校验、校验位计算与协议字段拆分。
- `model.rs`:管理端编码元信息 DTO。
- `admin.rs`:管理端编码元信息接口,路由为 `/api/v1/admin/number/meta`。

## 生成规则摘要

- 编码段:`R5-K3P1C1-N9-D4`
- `R5`:省码 + 市码;省市代码来自 `china` 模块。
- `K3`:主体属性 `K1` + 机构类型 `T2`。
- `K1`:主体属性,取值为 `M/Z/N/G/S/F`。
- `P1`:盈利属性,取值为 `0/1`。
- `C1`:校验位,继续使用原校验算法,载荷为 `R5 + K3 + P1 + N9 + D4`。
- `N9`:稳定散列序列。
- `D4`:年份。
- 示例:`LN001-GCB05-944805165-2026`。

规则:

- `M / Z / N` 使用省级占位市码 `000`。
- `G / S / F` 使用真实市码。
- `G` 机构码允许 `ZF/LF/SF/JC/JY/CB`。
- `S` 允许私权法人 `LP/GQ/GF/GY/AS` 和私法人教育机构 `JY`。
- `F` 允许独立非法人私权机构 `GT/GP`、教育分校 `JY` 和现有公权附属非法人 `ZG`;
  是否需要所属法人由 `subjects/uninorg` 校验。
- 教育阶段、国家/市公民教育委员会分类由 `subjects.education_type` 表达,不进入
  `GenerateCidInput`,也不得改变 `G/S/F + JY` 的 CID 生成语义。
- `ZG/TG` 不再用于私权机构分类;它们保留给人类主体来源分类,其中 `ZG` 仍承担既有公权附属
  非法人代码,不得在私权新增入口暴露。

私权目标类型映射:

| 类型 | 机构码 | 主体属性 |
|---|---|---|
| 个体经营 | `GT` | `F` |
| 无限合伙 | `GP` | `F` |
| 有限合伙 | `LP` | `S` |
| 股权公司 | `GQ` | `S` |
| 股份公司 | `GF` | `S` |
| 公益组织 | `GY` | `S` |
| 注册协会 | `AS` | `S` |

## 引用规则

- 编码协议统一通过 `crate::number::*` 引用。
- 行政区划统一通过 `crate::china::*` 引用。
- 不得恢复 `citizencode/backend/cid_number/`、`citizencode/backend/citizencode/`、`province.rs`、
  `cities.rs` 或 `city_codes/*.rs`。

## 验收口径

```text
test ! -d citizencode/backend/cid
test ! -d citizencode/backend/cid_number
test -d citizencode/backend/number
test -d citizencode/backend/china
rg "历史主体属性字段|历史身份字段别名" citizencode/backend memory/05-modules/citizencode
cd citizencode/backend && cargo check
```
