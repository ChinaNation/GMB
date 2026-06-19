# 任务卡：SFID 行政区运行真源与管理后台（已废弃）

## 状态

本任务卡记录的“SFID 运行库唯一真源 + 行政区管理 tab + wuminapp 在线行政区更新”方案已废弃。

废弃原因:后续用户确认行政区应改为开发库 `sfid/backend/china/china.sqlite` 权威源,所有系统随包只读,不再运行中改行政区。

## 被替代任务

- `memory/08-tasks/open/20260618-admin-district-dev-db-authority.md`

## 保留的有效结论

- `sfid/backend/china/data/` 不得恢复。
- 旧省命名已改为 `YL/伊犁省`。
- 市、镇 code 不可复用；删除的市/镇写入 tombstones。
- `citizenchain/runtime/` 和 `/primitives/china/` 仍须另走 runtime 升级二次确认。

## 已删除的旧结论

- 不再新增行政区管理 tab。
- 不再把 `SFID_CHINA_DB` 作为可写运行库。
- 不再提供 `/api/v1/app/admin-divisions/*` 在线行政区更新接口。
- 不再要求 wuminapp 运行中向 SFID 拉取行政区新版。
