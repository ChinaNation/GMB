# number/ — 身份 ID 编码协议

- 最后更新:2026-06-04
- 任务卡:
  - `memory/08-tasks/done/20260603-sfid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260604-sfid-core-number-store-refactor.md`

## 定位

- 路径:`sfid/backend/number/`
- 职责:提供 SFID 编码协议的 A3、机构码、分类、生成和格式校验。
- 非职责:不维护行政区划静态表,不保存省市镇村数据。

行政区划唯一真源在 `sfid/backend/china/`。`number::generator`
只在生成号码时调用 `crate::china::{province_code_by_name, city_code_by_name}`。

## 模块结构

```text
sfid/backend/number/
├── mod.rs
├── a3.rs
├── institution_code.rs
├── category.rs
├── generator.rs
├── validator.rs
├── model.rs
└── admin.rs
```

- `a3.rs`:A3 主体属性枚举。
- `institution_code.rs`:机构类型枚举。
- `category.rs`:机构分类枚举与分类函数。
- `generator.rs`:SFID 号码生成入口。
- `validator.rs`:SFID 号码格式校验与标准化。
- `model.rs`:管理端编码元信息 DTO。
- `admin.rs`:管理端编码元信息接口,路由为 `/api/v1/admin/number/meta`。

## 生成规则摘要

- 编码段:`A3-R5-T2P1C1-N9-D4`
- `A3`:主体类型。
- `R5`:省码 + 市码;省市代码来自 `china` 模块。
- `T2P1`:机构类型与盈利属性。
- `C1`:校验位。
- `N9`:稳定散列序列。
- `D4`:年份。

规则:

- `GMR / ZRR / ZNR` 使用省级占位市码 `000`。
- `GFR / SFR / FFR` 使用真实市码。
- `GFR` 机构码允许 `ZF/LF/SF/JC/JY/CB`。
- `SFR` 允许 `ZG/JY/CH/TG`。
- `FFR` 允许 `ZG/JY/TG`,并由 `subjects/uninorg` 校验从属关系。

## 引用规则

- 编码协议统一通过 `crate::number::*` 引用。
- 行政区划统一通过 `crate::china::*` 引用。
- 不得恢复 `sfid/backend/sfid_number/`、`sfid/backend/sfid/`、`province.rs`、
  `cities.rs` 或 `city_codes/*.rs`。

## 验收口径

```text
test ! -d sfid/backend/sfid
test ! -d sfid/backend/sfid_number
test -d sfid/backend/number
test -d sfid/backend/china
rg "crate::sfid_number|mod sfid_number|city_codes|province.rs" sfid/backend -g '*.rs'
cd sfid/backend && cargo check
```
