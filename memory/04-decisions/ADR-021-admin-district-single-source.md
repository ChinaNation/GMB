# ADR-021:行政区唯一真源 —— 开发库 china.sqlite 随包只读

状态:Accepted(2026-06-18)
关联:[[ADR-018]](wuminapp 混合模式)、reference_wuminapp_public_institution_bundle、feedback_no_compatibility

## 背景 / 问题

行政区唯一真源入口 = `sfid/backend/china/`。行政区数据文件只有一份:

```text
sfid/backend/china/china.sqlite
```

此前讨论过“SFID 运行库可管理 + wuminapp 在线拉新版 + CPMS 离线导入”的方案。该方案会形成开发库、运行库、客户端包三条数据线,一旦管理员运行中修改或代码升级替换种子,就会出现数据漂移。

当前决策改为:行政区以开发库 SQLite 为准。每次变更必须修改 `sfid/backend/china/china.sqlite`,递增 `metadata.admin_division_version`,再由发布流程生成各系统随包只读数据。

## 决策

- SFID 后端运行时只读 `SFID_CHINA_DB` 指向的随包 SQLite。正式部署固定为 `/opt/sfid/china/china.sqlite`。
- SFID 不提供行政区管理 tab,也不提供运行中新增、改名、删除行政区 API。
- wuminapp 安装包内置 `assets/admin_divisions/` 行政区字典,启动后只从本地包灌入 Isar,不向 SFID 联网拉行政区新版。
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
version:   2
provinces: 43
cities:    2941
towns:     39733
villages:  603913
```

本版本收口:

- `TS/天山省` 改为 `YL/伊犁省`。
- `HI/071 龙感湖市` 保留真实镇,不把工业园折算为镇。
- `HU/106 洪江市` 为唯一活跃洪江市,`HU/107` 已删除并写入 city/town tombstones。
- `HU/072 大通湖市` 保留 `南湾湖镇`。
- `HB/097 察北市` 的管理处名称折算为镇名,不保留“管理处”字样。

## 发布流程

1. 修改 `sfid/backend/china/china.sqlite`。
2. 更新 `metadata.admin_division_version` 和 `admin_division_versions`。
3. 运行 `python3 sfid/backend/china/check_code_immutable.py`。
4. 运行 `node wuminapp/tools/generate_admin_division_bundle.mjs` 生成行政区字典包。
5. 用指向同一 `china.sqlite` 的 SFID 后端执行 `sfid-backend reconcile-gov --changed-only`。
6. 执行 `sfid-backend check-gov --strict`,确认 `gov_manifest.china_hash` 等于当前 `china.sqlite` SHA-256,且缺失、错配、缺账户、废弃残留均为 0。
7. 运行 `node wuminapp/tools/generate_public_institution_bundle.mjs --version <行政区版本>` 生成公权机构包。
8. SFID、wuminapp、CPMS 发布安装包时只携带上述同源快照。

## 单源审查收口

1. 不得恢复 `sfid/backend/china/data/`。
2. 不得恢复 SFID 行政区管理 tab 或 `/api/v1/app/admin-divisions/*`。
3. 不得在 wuminapp/CPMS 内维护第二套行政区名字。
4. 任何 `HU/107`、`龙感湖工业园镇`、`xx管理市` 残留都必须在同一任务中清理。
