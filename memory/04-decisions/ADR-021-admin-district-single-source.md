# ADR-021:行政区代码与市镇数据唯一真源

状态:Accepted(2026-06-18),修订(2026-07-03)
关联:[[ADR-018]](citizenapp 混合模式)、reference_citizenapp_public_institution_bundle、feedback_no_compatibility

## 背景 / 问题

2026-06-25 修订后,行政区分两层真源:

- 国家码、省级行政区码和 CID 机构码的常量唯一真源 =
  `citizenchain/runtime/primitives/cid/code.rs`。
- 市、镇和地址段运行数据唯一真源入口 = `citizenchain/onchina/src/cid/china/`。行政区数据文件只有一份:

```text
citizenchain/onchina/src/cid/china/china.sqlite
```


当前决策改为:国家/省级代码先固化到 runtime primitives;OnChina 开发库 SQLite 保留省表并在加载时
断言与 primitives `PROVINCE_CODE_INFOS` 完全一致,但不再作为省码第二真源。市、镇和镇下地址仍以
开发库 SQLite 为准。开发库随安装包发布,运行态只读打开。链上 `AddressRegistry` 只记录地址库版本、
单条地址当前哈希和地址变更事件,不全量保存地址库。国家码、省码或机构码变更属于 runtime primitives
变更,必须单独走 runtime 二次确认。

不参与 CID 号生成、公权机构目录或链上治理边界。档案完整地址由
`省 + 市 + 镇 + address_name + address_local_no + address_detail` 组成。

## 决策

- 国家码、省级行政区码、CID 机构码只在 `citizenchain/runtime/primitives/cid/code.rs` 维护。
- OnChina 运行时只读 `ONCHINA_CHINA_DB` 指向的随包 SQLite;桌面端打包资源映射为 `china.sqlite`。
- OnChina 后端加载 SQLite 时必须校验 SQLite `provinces` 表与 primitives `PROVINCE_CODE_INFOS`
  的省名、省码、顺序和数量一致。
- CID 不提供行政区管理 tab,也不提供运行中新增、改名、删除行政区 API。
- citizenapp 安装包内置 `assets/admin_divisions/` 行政区字典,启动走版本驱动 reconcile(见下「客户端同步」),不向 OnChina 联网拉行政区新版。
- 公权机构不再由 OnChina 本地启动或命令从 `china.sqlite` 生成。所有公权机构信息唯一真源是链上
  `PublicManage::Institutions` / `PublicManage::InstitutionAccounts`;OnChina 只把链上存在的机构同步为本地查询投影。
- OnChina 本地投影状态记录在 `chain_projection_state(projection_key='public-gov')`;旧 `gov_manifest` 必须删除。
- 公权机构包 `assets/public_institutions/` 必须由完成 `sync-gov` 后的 OnChina 真实接口导出,避免本地派生目录或旧行政区 code 残留。
- 镇下地址只通过 `addresses` 单表保存当前有效数据。链上变更以 `AddressRegistry` 事件为同步事实,每次只更新对应地址名称或完整地址。

铁律:

- 省 code 固定在 runtime primitives,不维护 `province_tombstones`,不得在 OnChina `cid/china` 或旧 number
  模块手写第二份省码表。
- 市、镇 code 按当前创世基线确定。地址库不保留旧数据、旧表、墓碑或变更日志。
- 名称允许在原 code 上修改；不得用新 code 表达同一行政区改名。
- 省名和市名必须全国唯一。
- 镇下 `address_name_code` 在同一镇内按 `001..999` 编号;`address_local_no` 在同一 `address_name_code`
  下按 `0001..9999` 编号,可为空;`address_detail` 可为空。
- 地址名称保留当地地址名,`社区`、`村`、`路`、`巷`、`生活区` 等可以作为地址名称。

## 不触及(红线)

`citizenchain/runtime/`、`runtime/primitives/cid/code.rs` 和 `runtime/primitives/cid/china/` 属于链端保护常量;
国家码、省级行政区码、机构码或保护机构常量变化必须走 runtime 升级和二次确认。CID 市镇地址段
开发库变更不会自动修改 runtime 常量。

## 当前行政区版本

```text
version:   v1.0.0
provinces: 43
cities:    2872
towns:     39087
addresses: 598654
sha256:    54709a0c1935a59593c690c4547c8b6fa1140de8ebec642ef5c7daacb6012a5f
```

本版本收口:

- 旧省命名已统一为 `YL/伊犁省`。
- `HI/071 龙感湖市` 保留真实镇,不把工业园折算为镇。
- `洪江市` 为唯一活跃市,当前为 `HU/105`;旧重复洪江壳已删除,重新创世基线清空 city/town tombstones。
- `HU/071 大通湖市` 保留 `南湾湖镇`。
- `HB/097 察北市` 的管理处名称折算为镇名,不保留“管理处”字样。
- 描述性民族名已清理:`LJ/025 梅里斯达斡尔族市` 改 `梅里斯市`;`红旗满族镇` 合并为 `红旗镇`;`章党汉族村/章党朝鲜族村` 类同父重复村只保留一个规范名。`胡族铺镇`、`四族镇` 等本名含“族”的条目不清理。
- 岭南省已拆分 `香港市`、`九龙市`,新增 `香洲市`;广东省 `中山市` 已重建为一个市,原中山镇级伪市壳合并回 `中山市`。
- 广东省东莞区域不恢复为单一旧市,而是按重新创世口径拆为 `中堂市`、`莞城市`、`石龙市`、`万江市`、`虎门市`、`常平市`、`塘厦市` 七个市,完整承载原东莞 4 个街道和 28 个镇。
- 东莞区域功能区壳已删除。
- 河南 `人和市` 对应现实平顶山市石龙区,用于避让广东 `石龙市`;广东 `石龙市` 保留,全国仅一个 `石龙市`。
- 海南儋州区域收敛为 `儋州市`、`兰洋市`、`白马井市`、`新州市` 四个市。
- 本轮已清理已确认的截断名、`县市` 后缀名、跨省错挂名、重复壳和伪镇,并按重新创世重排市镇 code。最终审计发现的 42 个镇级伪行政区已删除,同步删除其下 235 条地址段;镇级伪行政区关键词命中归零。
- 重新创世后,开发库审计记录中的旧省命名残留已清理;2026-07-03 起公权机构不再执行本地运行库生成/对账,必须先随创世写入链上,再由 OnChina `sync-gov` 从链上同步投影并通过真实公开接口导出 citizenapp 公权机构包。
- 镇下地址已统一迁移为 `addresses` 单表,字段为 `province_code/city_code/town_code/address_name_code/address_name/address_local_no/address_detail/sort_order`。旧镇下地址表、旧来源字段、墓碑和变更日志表均已清除。

当前发布验收:

```text
china.sqlite sha256:
  c477cb5a300eac9f56d53beaef235617a6fc64584a0f1cffd8c85b2537840bbb
admin_divisions bundle:
  version=1 provinces=43 cities=2872 towns=39227
gov chain projection:
  command=onchina sync-gov
  source=PublicManage::Institutions + PublicManage::InstitutionAccounts
  projection_state=chain_projection_state(public-gov,status=OK)
gov chain audit:
  command=onchina audit-chain-catalog
  expected=596799 public institutions from genesis
public_institutions bundle:
  source=OnChina real API after sync-gov
  code cross-check bad_count=0
```

## 发布流程

开发期重新创世只做行政区基线清理时:

1. 修改 `citizenchain/onchina/src/cid/china/china.sqlite`。
2. 更新地址库版本常量或相关发布说明。
3. 运行 `python3 citizenchain/onchina/src/cid/china/check_code_immutable.py` 和 `PRAGMA integrity_check`。

正式发布完整资产时:

1. 修改 `citizenchain/onchina/src/cid/china/china.sqlite`。
2. 运行 `python3 citizenchain/onchina/src/cid/china/check_code_immutable.py`。
3. 运行 `node citizenapp/tools/generate_admin_division_bundle.mjs` 生成行政区字典包。
4. 重新创世并确认所有一府两会三院等公权机构已写入链上 `PublicManage`。
5. 用连接该链的 OnChina 后端执行 `onchina sync-gov`,确认 `chain_projection_state.public-gov` 为 `OK`。
6. 执行 `onchina audit-chain-catalog`,确认链上公权机构与创世清单全量一致。
7. 运行 `node citizenapp/tools/generate_public_institution_bundle.mjs --version <行政区版本>` 通过 OnChina 真实接口生成公权机构包。

## 客户端增量同步(citizenapp,2026-06-18)

citizenapp 无服务端,数据靠 assets 包随版本分发。包版本变了就**增量刷新:变的换、删的清、没变的不动**,零旧数据残留(行政区/公权机构都是只读派生数据,无用户数据)。

- **包内版本表**:两个 manifest 都带 `version`(全局,= `admin_division_version`)+ `provinces:[{code/name, ver}]`(省级内容版本)。行政区 `ver` = 该省市/镇分片内容 sha256;公权机构 `ver` 来自 OnChina 链投影版本(`chain_projection_state`),不得再使用本地生成目录 manifest。
- **客户端 `ensureSynced()`**(`*_bundle_loader.dart`):
  1. 先同步行政区字典,再同步公权机构包;公权机构全称/简称来自链投影接口,行政区只用于 code→名称展示 join。
  2. 全局 `version` 只作完成标记,不得短路省级检查;即使全局相等,也必须读取本地省级 `ver` 游标。
  3. 逐省比 `ver`,**只 reconcile `ver` 变了或本地缺游标的省**,没变的省连分片都不读。
  4. reconcile 单省(事务/分块内):按主键(行政区 `divisionKey` / 机构 `cidNumber`)做行级 diff,只 upsert 新增/字段变化的行,再删「包里已没有、本地还在」的废键。
  5. 逐省落 `ver`、最后落全局 `version`,中断可续。
- **版本游标**存于 `AppKvEntity`(`DataVersionKv`),**与 Isar `schemaVersion` 解耦**:`schemaVersion` 管 app 代码结构迁移,`data_version` 独立管数据新鲜度。旧格式包(无省级版本表)不得因为本地已有数据而跳过,必须按包 reconcile 以清理历史残留。
- **生效前提**:发布流程第 4/7 步重跑生成器产出带 `provinces:[{code/name,ver}]` 的新 manifest;客户端读到新版本即自动增量刷新,无需清 app 数据。

## 单源审查收口

1. 不得恢复旧行政区数据目录或旧顶层 china/number 模块。
2. 不得恢复 CID 行政区管理 tab 或 `/api/v1/app/admin-divisions/*`。
4. 任何重复洪江旧壳、`龙感湖工业园镇`、`xx管理市` 残留都必须在同一任务中清理。

## 实现坑（2026-06-23 修复）

字典 join = 「后台灌库 + UI 实时查字典」异步模型,两个时序约束必须守(违反→市名永久显示 code 兜底,如公权机构页每省市卡片显示 001/002):

1. **`ensureSynced()` 不能 `unawaited` 发射后不管**:首装灌 4.2 万条进 Isar(秒级~十几秒),期间 `cityNameMap` 查空 → 市名回退 code。须在同步**完成后回刷当前视图**(`public_page._syncThenRefresh`:await 完成 → 清缓存 → 重 join)。
2. **字典未就绪的兜底 code 不得入缓存**:`dictReady = cities.any((c) => c.name != c.code)` 为假时不写市列表缓存,否则脏 `001` 缓存住、同步完成也不自愈。
3. 排错铁律:citizenapp(离线 assets+Isar) 与 citizencode(实时 API+localStorage) 同症状但**两套独立通路**,改一个不修另一个;`unawaited` 吞掉的同步失败静默无声,定位须真 assets+真 Isar 端到端复现。详见 [[feedback-unawaited-bg-sync-needs-completion-refresh]]。
