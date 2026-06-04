# china/ — 行政区划真源

- 最后更新:2026-06-03
- 任务卡:
  - `memory/08-tasks/done/20260603-sfid-remove-institutions-china-sqlite.md`

## 定位

- 路径:`sfid/backend/china/`
- 数据源:`sfid/backend/china/data/china.sqlite`
- 职责:提供中国行政区划省、市、镇、村数据和省市代码查询。

`china` 是行政区划唯一真源。SFID 编码协议、自动公权机构目录、公安局对账和前端城市选择
都必须从该模块读取行政区划,不得恢复 Rust 静态行政区常量。

## 数据规模

当前 SQLite 数据来自已删除的旧 Rust 静态表:

```text
provinces: 43
cities:    3185
towns:     47853
villages:  716219
```

## 运行时接口

- `provinces()`:读取并缓存省市层级。
- `province_code_by_name(name)`:省名转省码。
- `city_code_by_name(province_name, city_name)`:省名 + 市名转市码。
- `province_name_by_code(code)`:省码转省名。
- `china::admin::admin_sfid_cities`:管理端城市列表接口。

镇村数据保存在 SQLite 中,当前运行时只加载省市层级;后续需要镇村下钻时必须继续从
SQLite 查询,不得再把镇村生成回 Rust 源码。

## 文件结构

```text
sfid/backend/china/
├── mod.rs
├── model.rs
├── store.rs
├── admin.rs
└── data/
    └── china.sqlite
```

## 验收口径

```text
test -f sfid/backend/china/data/china.sqlite
test ! -d sfid/backend/sfid
rg "city_codes|PROVINCES" sfid/backend -g '*.rs'
cd sfid/backend && cargo check
```
