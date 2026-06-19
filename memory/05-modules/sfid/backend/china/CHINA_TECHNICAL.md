# china/ — 行政区划开发库权威源

- 最后更新:2026-06-19
- 任务卡:
  - `memory/08-tasks/open/20260618-admin-district-dev-db-authority.md`
  - `memory/08-tasks/done/20260618-sfid-gov-admin-division-reconcile.md`
  - `memory/08-tasks/done/20260618-fresh-genesis-yl-cleanup.md`
  - `memory/08-tasks/open/20260618-version-reset-v1.md`
  - `memory/08-tasks/done/20260619-admin-district-fresh-genesis-hk-zs.md`

## 定位

- 模块路径:`sfid/backend/china/`
- 权威数据:`sfid/backend/china/china.sqlite`
- 生产读取:`SFID_CHINA_DB=/opt/sfid/china/china.sqlite`
- 职责:提供省、市、镇、村/路数据,以及市/镇 tombstones 和只读查询能力。

`china.sqlite` 是行政区唯一权威源。SFID 后端只读打开该 SQLite;wuminapp、CPMS 和公权机构资产包都从这份开发库派生随包只读快照。系统运行中不修改行政区,也不提供行政区管理 tab。

## 数据规模

重新创世基线当前发布版本:

```text
version:   1
provinces: 43
cities:    2898
towns:     39724
villages:  603901
sha256:    0db5080d05c1dcb184c6c8f9b4ad0d6fc4fbef17f88c645ba256e86ab450161d
city_tombstones: 0
town_tombstones: 0
```

重点修正:

- 旧省命名已统一为 `YL/伊犁省`。
- `HU/106 洪江市` 是唯一活跃洪江市,`HU/107` 已删除;重新创世基线清空 tombstones。
- `HI/071 龙感湖市` 不包含“龙感湖工业园镇”。
- `HU/072 大通湖市` 包含 `南湾湖镇`。
- `HB/097 察北市` 下的管理处名称已折算为镇名。
- `LJ/025 梅里斯达斡尔族市` 已改为 `梅里斯市`。
- 民族描述清理:镇/村/路中属于“某某民族/某某族村/某某族镇”的描述性前缀已按同 code 改名或按同父重复合并;`红旗满族镇` 合并为 `红旗镇`;`章党汉族村/章党朝鲜族村` 类同父重复只保留一个规范名。
- 保留本名含“族”或通用地名的条目,例如 `胡族铺镇`、`四族镇`、`大族村`、`望族苑路`、`哈族新村`;这些不是“民族描述”清理对象。
- 岭南省已拆分 `香港市`、`九龙市`,并新增 `香洲市`;`香洲市` 包含 `坦洲镇`、`神湾镇`、`三乡镇`、`唐家湾镇`。
- 广东省 `中山市` 已重建为一个市,原中山镇级伪市壳合并回 `中山市`;原 `东升镇` 并入 `小榄镇`。
- 广东省东莞区域不恢复为单一 `东莞市`,而是按重新创世口径拆为 `中堂市`、`莞城市`、`石龙市`、`万江市`、`虎门市`、`常平市`、`塘厦市` 七个市,完整承载原东莞 4 个街道和 28 个镇;`松山湖市`、`东莞港市`、`东莞生态园市` 功能区壳已删除。

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

wuminapp 公权机构包通过真实 SFID 公开接口生成,公民端“公权机构”显示完整公权目录。
完整目录包含 SFID 自动公权目录(公安局、教育委员会、省储行等)、管理员手动创建的
公法人、以及上级为公法人的非法人。SFID 管理端可以按“公权机构 / 市公安局 /
教育机构”等后台功能分区管理,但这些分区不得影响 wuminapp 公民端公权列表。

2026-06-19 重新创世版本 1 当前公权目录收口结果:

```text
reconcile-gov --changed-only:
  scopes=43 inserted=3543 updated=245100 account_inserted=497329 removed=4243
check-gov --strict:
  ok=true manifest_current=true target_count=248643 active_count=248643
  missing=0 mismatched=0 missing_accounts=0 obsolete=0
  catalog_hash=856b48488086cabda027d16d47df352642377c31c41aeabcf927caa8187758ac
wuminapp public_institutions:
  version=1 provinces=43 total=248643 YL=1737
  完整公权目录包含 CITY_POLICE=2898,CITY_EDU=2898,NATIONAL_EDU=1,PROVINCE_RESERVE_BANK=43,PUBLIC_SECURITY=2898
  target_count - public_institutions = 0
```

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
