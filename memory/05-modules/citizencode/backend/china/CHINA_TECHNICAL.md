# china/ — 行政区划开发库权威源

- 最后更新:2026-06-20
- 任务卡:
  - `memory/08-tasks/open/20260618-admin-district-dev-db-authority.md`
  - `memory/08-tasks/done/20260618-cid-gov-admin-division-reconcile.md`
  - `memory/08-tasks/done/20260618-fresh-genesis-yl-cleanup.md`
  - `memory/08-tasks/open/20260618-version-reset-v1.md`
  - `memory/08-tasks/done/20260619-admin-district-fresh-genesis-hk-zs.md`
  - `memory/08-tasks/open/20260620-address-units.md`

## 定位

- 模块路径:`citizencode/backend/china/`
- 权威数据:`citizencode/backend/china/china.sqlite`
- 生产读取:`CID_CHINA_DB=/opt/citizencode/china/china.sqlite`
- 职责:提供省、市、镇和镇下地址段数据,以及市/镇 tombstones 和只读查询能力。

`china.sqlite` 是行政区唯一权威源。CID 后端只读打开该 SQLite;CitizenApp 和公权机构资产包都从这份开发库派生随包只读快照。系统运行中不修改行政区,也不提供行政区管理 tab。

## 数据规模

当前行政区版本:

```text
version:   2
provinces: 43
cities:    2872
towns:     39087
address_units: 598655
source_code_filled: 598655
official_source_codes: 535084
local_source_codes:    63571
sha256:    0a7bbe497fa04a084cebe37cef24b8683a27e8c618727f4f4ca07f5b83c7853c
city_tombstones: 0
town_tombstones: 140
```

重点修正:

- 旧省命名已统一为 `YL/伊犁省`。
- `洪江市` 是唯一活跃市,当前为 `HU/105`;旧重复洪江壳已删除,重新创世基线清空 tombstones。
- `HI/071 龙感湖市` 不包含“龙感湖工业园镇”。
- `HU/071 大通湖市` 包含 `南湾湖镇`。
- `HB/097 察北市` 下的管理处名称已折算为镇名。
- `LJ/025 梅里斯达斡尔族市` 已改为 `梅里斯市`。
- 民族描述清理:镇和地址段中属于“某某民族/某某族村/某某族镇”的描述性前缀已按同 code 改名或按同父重复合并;`红旗满族镇` 合并为 `红旗镇`;`章党汉族村/章党朝鲜族村` 类同父重复只保留一个规范名。
- 保留本名含“族”或通用地名的条目,例如 `胡族铺镇`、`四族镇`、`大族村`、`望族苑路`、`哈族新村`;这些不是“民族描述”清理对象。
- 岭南省已拆分 `香港市`、`九龙市`,并新增 `香洲市`;`香洲市` 包含 `坦洲镇`、`神湾镇`、`三乡镇`、`唐家湾镇`。
- 广东省 `中山市` 已重建为一个市,原中山镇级伪市壳合并回 `中山市`;原 `东升镇` 并入 `小榄镇`。
- 广东省东莞区域不恢复为单一旧市,而是按重新创世口径拆为 `中堂市`、`莞城市`、`石龙市`、`万江市`、`虎门市`、`常平市`、`塘厦市` 七个市,完整承载原东莞 4 个街道和 28 个镇;功能区壳已删除。
- 2026-06-19 行政区名称归属清理仅更新 `china.sqlite`,未生成 CitizenApp 数据包,未生成 CID 公权机构。
- 河南 `人和市` 保留为现实平顶山市石龙区的系统唯一名,并补齐 `龙兴镇/北郎店社区村`;广东 `石龙市` 保留,全国只存在一个 `石龙市`。
- 海南儋州区域从一镇一市收敛为 `儋州市`、`兰洋市`、`白马井市`、`新州市` 四个市。
- 矿区按重新创世规则清理:5 个及以上镇的正式矿区改为普通市名,少于 5 个镇的矿区壳删除并入相邻市。
- 已确认的截断名、`县市` 后缀名、跨省错挂名和重复壳已清理,保留的行政区全部使用当前规范名。最终审计发现的 42 个镇级伪行政区已删除,同步删除其下 235 条地址段;镇级伪行政区关键词命中归零。
- 河北平乡数据从错挂壳中拆出为 `平乡市`;重复错挂壳已删除。
- 内蒙古旗类后缀已按系统三层模型去掉“旗”;`社旗市` 是河南地名本身,不属于该清理对象。
- 镇下面第四层已改为地址段 `address_units`,不再是行政区,也不再强制补固定后缀。地址段名称保留当地地址名,`社区`、`村`、`路`、`巷`、`生活区` 等可以作为地址段;只剥离 `居委会`、`居民委员会`、`村委会`、`村民委员会`、`委员会`、`办事处`、`管理处`、`管委会` 等组织或管理机构词。地址段与注册局录入的详细地址组合为公民详细地址。2026-06-20 删除原金门市,补齐 10 个非港澳台空地址段市,并为 160 个高置信市补齐 19226 条 `source_code/raw_name`;随后合并原 `HN/006 天涯市` 到崖州市,并补齐崖州区 33 条来源码。剩余无法高置信绑定官方统计局来源的地址段已补入本地稳定来源码 `LOCAL-省市镇-地址段ID`,已有地址段 `source_code` 空值归零。最终审计已重排福建、海南删除/合并后的市 code 空洞,删除 42 个镇级伪行政区及其下 235 条地址段,清理 154 条地址段名称中的组织或管理机构词,将 568 条 `xx虚拟路` 归一为 `xx`,并删除 3 条纯 `虚拟路` 及其中 2 个功能区壳镇。随后对纯功能词地址段做二次收口:46 条原始名含 `社区` 的 `开发区/新区/农场/工业园` 等地址段恢复为 `xx社区`,26 条 `LOCAL-*` 来源的 `xx虚拟路` 合成占位地址段删除,同步删除因此空掉的 24 个镇并重排受影响市的镇 code。`raw_name` 保留原始来源名称用于审计,不作为前端展示名。当前 `FJ/043` 为 `石狮市`,当前 `HN/006` 为 `崖州市`;`LN/001`、`LN/002`、`LN/003`、`LN/004` 与 5 个台湾同名自建市当前无地址段行,按港澳台豁免不产生地址段来源码。

`address_units.source_code` 的口径:

- 纯数字值为官方统计局镜像来源码。
- `LOCAL-*` 为开发库本地稳定来源码,只用于标注已纳入开发库唯一真源但无法高置信绑定官方统计局来源的地址段;该值不参与行政区 code、CID 号或公权机构生成。

## 运行时接口

- `provinces()`:从只读 SQLite 读取并缓存省、市、镇树。
- `province_code_by_name(name)`:省名转省码。
- `city_code_by_name(province_name, city_name)`:省名 + 市名转市码。
- `province_name_by_code(code)`:省码转省名。
- `area_name_by_codes(...)`:按 code 还原省、市、镇名称。
- `town_exists(pc, cc, tc)`:判断活跃镇 code 是否存在,供孤儿机构清理和目录校验使用。
- `china_sqlite_hash()`:返回 SQLite 文件 SHA-256,供公权目录 hash 与资产包校验。
- `china::admin::admin_china_cities`:管理端只读省内城市列表,供机构创建页面选择城市。
- `address_units`:由注册局公民录入和 CitizenApp 行政区包消费,不进入 CID 公权机构目录。

权限边界:

- 联邦注册局机构管理员和市注册局机构管理员只能读取自己作用域内的省市元数据。
- 没有任何运行中写行政区入口。

## 公权机构联动

行政区变更后,`china.sqlite` 的 SHA-256 会进入 `gov_manifest.china_hash`。发布完整数据包和公权机构时,CID 运行库必须先执行:

```text
citizencode-backend reconcile-gov --changed-only
citizencode-backend check-gov --strict
```

`check-gov --strict` 通过时,全局目标数、活跃数、缺失、错配、缺账户、obsolete 和
manifest 都必须一致。自动目录只清理 `gov.source='GENERATED'` 的派生机构;管理员手动创建的
`MANUAL` 公权机构不由行政区同步删除。

CitizenApp 公权机构包通过真实 CID 公开接口生成,公民端“公权机构”显示完整公权目录。
完整目录包含 CID 自动公权目录(公安局、教育委员会、省储行等)、管理员手动创建的
公法人、以及上级为公法人的非法人。CID 管理端可以按“公权机构 / 市公安局 /
教育机构”等后台功能分区管理,但这些分区不得影响 CitizenApp 公民端公权列表。

2026-06-20 已按当前 `china.sqlite` 完成发布资产收口:重新生成 CitizenApp 行政区包、执行 CID 公权机构运行库对账和 strict check,并通过当前 CID 真实公开接口重新生成 CitizenApp 公权机构包。

2026-06-21 镇名残留清理后增量传播:`china.sqlite` 镇名残留清理(剥省/老地级市前缀、功能区企业名删除或并入驻地镇、对照国家统计局2024)使 towns 39227→39087(86 改名 + 17 并入已有镇 + 94 并入驻地镇删壳 + 29 删空壳,共 140 移除码进 `town_tombstones`),admin_division_version 1→2。下游只更新改动的:行政区字典包按省增量重生(33 省 towns 分片变);`reconcile-gov --changed-only` 把 430 个镇级公权机构改名(cid 不变,确定性派生)、清 700 个孤儿(移除镇),仅 33 省写库;公权机构包重生全 43 省(33 省按数据变,10 省仅 manifest_version 同步早前对账)。strict check 通过。残留清理:`generate_admin_division_bundle.mjs` 默认库路径 `cid/→citizencode/`;`generate_public_institution_bundle.mjs` 请求参数与输出键 `province→province_name`(与后端/客户端对齐);`public_institution.rs` 接口注释参数名同步。

当前公权目录收口结果:

```text
admin_divisions bundle:
  version=2 provinces=43 cities=2872 towns=39087
  china_sqlite_sha256=0a7bbe497fa04a084cebe37cef24b8683a27e8c618727f4f4ca07f5b83c7853c
reconcile-gov --changed-only:
  scopes=33 inserted=0 updated=211347 account_inserted=422727 removed=700
check-gov --strict:
  ok=true manifest_current=true target_count=245016 active_count=245016
  missing=0 mismatched=0 missing_accounts=0 obsolete=0
  catalog_hash=a3d7f875388543cec87088333c03a9df148d3dad60b8be1f528d8b073705b89f
CitizenApp public_institutions:
  version=2026-06-22 provinces=43 total=245016 YL=1697
  完整公权目录包含 CITY_POLICE=2872,CITY_EDU=2872,JY=2873
  target_count - public_institutions = 0
  code cross-check bad_count=0
```

## 文件结构

```text
citizencode/backend/china/
├── admin.rs                 # 管理端只读城市列表接口
├── check_code_immutable.py  # 行政区、地址段唯一性与 tombstone 校验
├── china.sqlite             # 唯一权威源
├── mod.rs
├── model.rs
└── store.rs                 # SQLite 只读读取层
```

## 验收口径

```text
test -f citizencode/backend/china/china.sqlite
test ! -d citizencode/backend/china/data
python3 citizencode/backend/china/check_code_immutable.py
sqlite3 citizencode/backend/china/china.sqlite "select value from metadata where key='admin_division_version'"
sqlite3 citizencode/backend/china/china.sqlite "select count(*) from address_units"
sqlite3 citizencode/backend/china/china.sqlite "select count(*) from sqlite_master where type='table' and name='province_tombstones'"
sqlite3 citizencode/backend/china/china.sqlite "PRAGMA integrity_check"
```

本轮地址段改造不生成 CitizenApp 公权机构包,不执行公权机构 reconcile。
