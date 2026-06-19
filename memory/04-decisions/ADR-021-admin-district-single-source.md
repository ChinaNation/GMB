# ADR-021:行政区唯一真源 —— 开发库 china.sqlite 随包只读

状态:Accepted(2026-06-18)
关联:[[ADR-018]](wuminapp 混合模式)、reference_wuminapp_public_institution_bundle、feedback_no_compatibility

## 背景 / 问题

行政区唯一真源入口 = `sfid/backend/china/`。行政区数据文件只有一份:

```text
sfid/backend/china/china.sqlite
```

此前讨论过“SFID 运行库可管理 + wuminapp 在线拉新版 + CPMS 离线导入”的方案。该方案会形成开发库、运行库、客户端包三条数据线,一旦管理员运行中修改或代码升级替换种子,就会出现数据漂移。

当前决策改为:行政区以开发库 SQLite 为准。重新创世基线版本为 1;此后每次变更必须修改 `sfid/backend/china/china.sqlite`,递增 `metadata.admin_division_version`,再由发布流程生成各系统随包只读数据。

## 决策

- SFID 后端运行时只读 `SFID_CHINA_DB` 指向的随包 SQLite。正式部署固定为 `/opt/sfid/china/china.sqlite`。
- SFID 不提供行政区管理 tab,也不提供运行中新增、改名、删除行政区 API。
- wuminapp 安装包内置 `assets/admin_divisions/` 行政区字典,启动走**版本驱动增量 reconcile**(见下「客户端增量同步」),不向 SFID 联网拉行政区新版。
- CPMS 安装包内置同源 `china.sqlite` 只读快照,运行中不得联网更新行政区。
- SFID 运行库中的自动公权机构必须由同一 `china.sqlite` 对账生成,`gov_manifest` 必须记录当前 SQLite hash 和目录 hash。
- 公权机构包 `assets/public_institutions/` 必须由对账并通过严格校验后的 SFID 真实接口导出,避免旧行政区 code 残留。

铁律:

- 省 code 固定,不维护 `province_tombstones`。
- 市、镇 code 不可变、不复用。删除的市/镇 code 写入 `city_tombstones` / `town_tombstones` 永久占位。
- 名称允许在原 code 上修改；不得用新 code 表达同一行政区改名。
- 省名和市名必须全国唯一。

## 不触及(红线)

`citizenchain/runtime/` 和 `/primitives/china/` 属于链端保护常量;行政区或保护机构常量变化必须走 runtime 升级和二次确认。SFID 开发库变更不会自动修改 runtime 常量。

## 当前行政区版本

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

本版本收口:

- 旧省命名已统一为 `YL/伊犁省`。
- `HI/071 龙感湖市` 保留真实镇,不把工业园折算为镇。
- `HU/106 洪江市` 为唯一活跃洪江市,`HU/107` 已删除;重新创世基线清空 city/town tombstones。
- `HU/072 大通湖市` 保留 `南湾湖镇`。
- `HB/097 察北市` 的管理处名称折算为镇名,不保留“管理处”字样。
- 描述性民族名已清理:`LJ/025 梅里斯达斡尔族市` 改 `梅里斯市`;`红旗满族镇` 合并为 `红旗镇`;`章党汉族村/章党朝鲜族村` 类同父重复村只保留一个规范名。`胡族铺镇`、`四族镇` 等本名含“族”的条目不清理。
- 岭南省已拆分 `香港市`、`九龙市`,新增 `香洲市`;广东省 `中山市` 已重建为一个市,原中山镇级伪市壳合并回 `中山市`。
- 广东省东莞区域不恢复为单一 `东莞市`,而是按重新创世口径拆为 `中堂市`、`莞城市`、`石龙市`、`万江市`、`虎门市`、`常平市`、`塘厦市` 七个市,完整承载原东莞 4 个街道和 28 个镇。
- `松山湖市`、`东莞港市`、`东莞生态园市` 功能区壳已删除。
- 重新创世后,开发库审计记录中的旧省命名残留已清理;2026-06-19 东莞拆分后已重新生成行政区包、公权机构运行库和 wuminapp 公权机构包。

版本 1 发布验收:

```text
china.sqlite sha256:
  0db5080d05c1dcb184c6c8f9b4ad0d6fc4fbef17f88c645ba256e86ab450161d
gov reconcile:
  scopes=43 inserted=3543 updated=245100 account_inserted=497329 removed=4243
gov strict:
  ok=true manifest_current=true target_count=248643 active_count=248643
  missing=0 mismatched=0 missing_accounts=0 obsolete=0
  catalog_hash=856b48488086cabda027d16d47df352642377c31c41aeabcf927caa8187758ac
public_institutions bundle:
  version=1 provinces=43 total=248643
  公民端完整公权目录包含 CITY_POLICE=2898、CITY_EDU=2898、NATIONAL_EDU=1、PROVINCE_RESERVE_BANK=43。
```

## 发布流程

1. 修改 `sfid/backend/china/china.sqlite`。
2. 更新 `metadata.admin_division_version` 和 `admin_division_versions`。
3. 运行 `python3 sfid/backend/china/check_code_immutable.py`。
4. 运行 `node wuminapp/tools/generate_admin_division_bundle.mjs` 生成行政区字典包。
5. 用指向同一 `china.sqlite` 的 SFID 后端执行 `sfid-backend reconcile-gov --changed-only`。
6. 执行 `sfid-backend check-gov --strict`,确认 `gov_manifest.china_hash` 等于当前 `china.sqlite` SHA-256,且缺失、错配、缺账户、废弃残留均为 0。
7. 运行 `node wuminapp/tools/generate_public_institution_bundle.mjs --version <行政区版本>` 生成公权机构包。
8. SFID、wuminapp、CPMS 发布安装包时只携带上述同源快照。

## 客户端增量同步(wuminapp,2026-06-18)

wuminapp 无服务端,数据靠 assets 包随版本分发。包版本变了就**增量刷新:变的换、删的清、没变的不动**,零旧数据残留(行政区/公权机构都是只读派生数据,无用户数据)。

- **包内版本表**:两个 manifest 都带 `version`(全局,= `admin_division_version`)+ `provinces:[{code/name, ver}]`(省级内容版本)。行政区 `ver` = 该省市/镇分片内容 sha256;公权机构 `ver` = 该省目录 `manifest_version`。省内容(改名/删码/重排)一变 `ver` 即变。
- **客户端 `ensureSynced()`**(`*_bundle_loader.dart`):
  1. 先同步行政区字典,公权机构名称只允许从已对账的行政区 code 生成。
  2. 全局 `version` 只作完成标记,不得短路省级检查;即使全局相等,也必须读取本地省级 `ver` 游标。
  3. 逐省比 `ver`,**只 reconcile `ver` 变了或本地缺游标的省**,没变的省连分片都不读。
  4. reconcile 单省(事务/分块内):按主键(行政区 `divisionKey` / 机构 `sfidNumber`)做行级 diff,只 upsert 新增/字段变化的行,再删「包里已没有、本地还在」的废键。
  5. 逐省落 `ver`、最后落全局 `version`,中断可续。
- **版本游标**存于 `AppKvEntity`(`DataVersionKv`),**与 Isar `schemaVersion` 解耦**:`schemaVersion` 管 app 代码结构迁移,`data_version` 独立管数据新鲜度。旧格式包(无省级版本表)不得因为本地已有数据而跳过,必须按包 reconcile 以清理历史残留。
- **生效前提**:发布流程第 4/7 步重跑生成器产出带 `provinces:[{code/name,ver}]` 的新 manifest;客户端读到新版本即自动增量刷新,无需清 app 数据。

## 单源审查收口

1. 不得恢复 `sfid/backend/china/data/`。
2. 不得恢复 SFID 行政区管理 tab 或 `/api/v1/app/admin-divisions/*`。
3. 不得在 wuminapp/CPMS 内维护第二套行政区名字。
4. 任何 `HU/107`、`龙感湖工业园镇`、`xx管理市` 残留都必须在同一任务中清理。
