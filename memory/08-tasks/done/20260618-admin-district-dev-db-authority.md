# 任务卡：行政区改为开发库权威源

## 任务目标

把行政区方案从“SFID 运行库可管理、可联网更新”改回“开发库 `sfid/backend/china/china.sqlite` 是唯一权威源”：

- 删除省级 tombstone 机制。
- 行政区每次变更只改开发库，版本号递增。
- SFID、CPMS、公民发布安装包时都从开发库派生本地只读数据包。
- 删除 SFID 行政区管理 tab、运行中增删改 API 和公民端联网行政区更新。
- 修正“管理市/管理处”数据：保留真实管理区折算出的市/镇，不保留“管理”二字。

## 本次确认口径

- 旧省命名已改为 `伊犁省/YL`，继续保留该方向，runtime 后续另行升级。
- `HU/072` 恢复为 `大通湖市`，下辖 `河坝镇、金盆镇、北洲子镇、千山红镇、南湾湖镇`。
- `HI/071` 恢复为 `龙感湖市`，只保留真实办事处折算镇，工业园不折算成镇。
- `HU/106` 保留并命名为 `洪江市`，删除/合并 `HU/107 洪江`。
- `HB/097` 恢复为 `察北市`，`黄山/石门/乌兰/金沙/白塔管理处` 折算为镇。

## 预计修改目录

- `sfid/backend/china/`：开发库 SQLite、只读查询层、校验脚本和行政区源说明。
- `sfid/backend/`：删除行政区管理写接口、运行库复制逻辑和联网发布接口残留。
- `sfid/frontend/`：删除行政区管理 tab、权限位和页面/API 残留。
- `wuminapp/`：删除行政区联网更新代码，重生本地行政区数据包。
- `cpms/`：保持本地只读行政区包，更新说明与路径注释。
- `.github/workflows/`：构建链路以开发库 `china.sqlite` 为输入。
- `memory/`：更新 ADR、技术文档、AI 规则和任务残留。

## 验收要求

- `province_tombstones` 不存在，省级不建 tombstone。
- `city_tombstones` / `town_tombstones` 只保留真正删除的市/镇，不包含本次恢复或合并后的活跃 code。
- `metadata.admin_division_source` 指向开发库口径，版本号本次递增。
- SFID 后端无行政区运行中写库 API；前端无行政区管理 tab。
- 公民端无向 SFID 拉取行政区新版的联网逻辑。
- CPMS、公民、SFID 安装包均使用本地只读行政区数据。
- 真实服务验收覆盖 SFID 只读接口/编码查询、CPMS 本地查询、公民本地包载入路径。

## 执行记录

- 2026-06-18：按用户最新确认口径创建任务卡，开始执行。
- 2026-06-18：完成 `china.sqlite` 数据修正：版本 `2`，`HU/106 洪江市` 唯一保留，`HU/107` 退役，龙感湖不保留工业园镇，增加 `南湾湖镇`。
- 2026-06-18：删除 SFID 行政区运行中写库 API、前端行政区管理 tab、wuminapp 行政区联网更新路径。
- 2026-06-18：重生 wuminapp 行政区字典包和公权机构包，资产包版本均为 `2`。
- 2026-06-18：真实临时 SFID 服务验收通过：`/api/v1/app/public-institutions?province=湖南省&city=洪江市` 返回 `HU/106`，旧 `/api/v1/app/admin-divisions/version` 返回 `404`。
- 2026-06-18：最终收口验收通过：`sfid/backend/china/data/` 实体目录已删除，`check_code_immutable.py`、`cargo check/test`、`npm run build`、`flutter analyze/test`、残留关键字扫描均完成。
