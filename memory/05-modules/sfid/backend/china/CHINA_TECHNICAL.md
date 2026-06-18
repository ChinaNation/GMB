# china/ — 行政区划开发库权威源

- 最后更新:2026-06-18
- 任务卡:
  - `memory/08-tasks/open/20260618-admin-district-dev-db-authority.md`
  - `memory/08-tasks/done/20260618-sfid-gov-admin-division-reconcile.md`

## 定位

- 模块路径:`sfid/backend/china/`
- 权威数据:`sfid/backend/china/china.sqlite`
- 生产读取:`SFID_CHINA_DB=/opt/sfid/china/china.sqlite`
- 职责:提供省、市、镇、村/路数据,以及市/镇 tombstones 和只读查询能力。

`china.sqlite` 是行政区唯一权威源。SFID 后端只读打开该 SQLite;wuminapp、CPMS 和公权机构资产包都从这份开发库派生随包只读快照。系统运行中不修改行政区,也不提供行政区管理 tab。

## 数据规模

当前发布版本:

```text
version:   2
provinces: 43
cities:    2941
towns:     39733
villages:  603913
```

重点修正:

- `TS/天山省` 已改为 `YL/伊犁省`。
- `HU/106 洪江市` 是唯一活跃洪江市,`HU/107` 已删除并写入 tombstones。
- `HI/071 龙感湖市` 不包含“龙感湖工业园镇”。
- `HU/072 大通湖市` 包含 `南湾湖镇`。
- `HB/097 察北市` 下的管理处名称已折算为镇名。

## 运行时接口

- `provinces()`:从只读 SQLite 读取并缓存省、市、镇树。
- `province_code_by_name(name)`:省名转省码。
- `city_code_by_name(province_name, city_name)`:省名 + 市名转市码。
- `province_name_by_code(code)`:省码转省名。
- `area_name_by_codes(...)`:按 code 还原省、市、镇名称。
- `town_exists(pc, cc, tc)`:判断活跃镇 code 是否存在,供孤儿机构清理和目录校验使用。
- `china_sqlite_hash()`:返回 SQLite 文件 SHA-256,供公权目录 hash 与资产包校验。
- `china::admin::admin_china_cities`:管理端只读省内城市列表,供机构创建页面选择城市。

权限边界:

- 联邦管理员和市管理员只能读取自己作用域内的省市元数据。
- 没有任何运行中写行政区入口。

## 公权机构联动

行政区变更后,`china.sqlite` 的 SHA-256 会进入 `gov_manifest.china_hash`。SFID 运行库必须先执行:

```text
sfid-backend reconcile-gov --changed-only
sfid-backend check-gov --strict
```

`check-gov --strict` 通过时,全局目标数、活跃数、缺失、错配、缺账户、obsolete 和
manifest 都必须一致。自动目录只清理 `gov.source='GENERATED'` 的派生机构;管理员手动创建的
`MANUAL` 公权机构不由行政区同步删除。

## 文件结构

```text
sfid/backend/china/
├── admin.rs                 # 管理端只读城市列表接口
├── check_code_immutable.py  # 行政区唯一性与 tombstone 校验
├── china.sqlite             # 唯一权威源
├── mod.rs
├── model.rs
└── store.rs                 # SQLite 只读读取层
```

## 验收口径

```text
test -f sfid/backend/china/china.sqlite
test ! -d sfid/backend/china/data
python3 sfid/backend/china/check_code_immutable.py
sqlite3 sfid/backend/china/china.sqlite "select value from metadata where key='admin_division_version'"
sqlite3 sfid/backend/china/china.sqlite "select count(*) from sqlite_master where type='table' and name='province_tombstones'"
cd sfid/backend && cargo check
cd sfid/backend && sfid-backend reconcile-gov --changed-only
cd sfid/backend && sfid-backend check-gov --strict
```
