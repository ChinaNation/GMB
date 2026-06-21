# 任务卡:citizenapp 行政区+公权机构 版本驱动增量同步

让 citizenapp「公民-公权」的行政区/机构数据随真源(china.sqlite)版本更新而增量刷新:**变的换、删的清、没变的不动**,零旧数据残留;citizenapp 无服务端,靠 assets 包分发。

## 现状(用户已建的源头侧,勿重造)
- `citizencode/backend/china/china.sqlite`(已从 china/data/ 上移到 china/):`metadata.admin_division_version`(全局单调版本,现=3)+ `admin_division_versions`(版本登记)+ `admin_division_change_log`(每版精确变更 MERGE/RESTORE/RENAME,带 old/new 名+码)+ city/town_tombstones。
- `generate_admin_division_bundle.mjs` 已读 `admin_division_version` 写进 manifest `version`。store.rs 路径已改 china/china.sqlite + 加省名/市名全国唯一断言。
- 缺口①(生成器):manifest 只有全局 version,**没有省级版本**(客户端跳过没变省要用)。
- 缺口②(客户端):loader 还是「库空才灌」,无版本逻辑;清表只挂 Isar schemaVersion<7。

## 要做(我的范围)
### A 生成器(mjs)
- 两个 manifest 都加:`data_version`(=admin_division_version,统一字段名)+ `provinces:[{code, ver}]`,ver=该省内容(市+镇 / 机构)hash。行政区包与机构包**同 version + 同 china_sqlite_sha256**(同源)。

### B 客户端(dart)版本驱动 reconcile
- AppKv 存 `admin_division.data_version`/`public_institution.data_version` + per-province ver map(**与 Isar schemaVersion 解耦**)。
- loader:`ensureDictionaryLoaded/ensureBaselineLoaded(库空才灌)` → `ensureSynced()`:全局 version 只作完成标记,逐省比 per-province ver,**只 reconcile 变了或缺游标的省**。
- reconcile(事务内,按主键集合):先读取本省现有实体,逐条比字段,只 upsert 新增/字段变化的行;删「包里没有、Isar 还在」的 key(divisionKey / cidNumber)。没变的行不动。
- store 补:`divisionsOfProvince/institutionsOfProvince` + `deleteByKeys/deleteByCids`。
- repository 进页调 ensureSynced 取代 ensureBaselineLoaded。

### C 收尾
- 更新文档/注释、清残留死代码/旧注释、检查遗漏。

## 验收
- 单测:reconcile 改名(同key覆盖)/删除(删absent)/新增/没变省跳过/首装全insert;version 相等秒过、不等只对账变的省;两包 sha 不一致拒绝。
- flutter analyze 0 + 公权相关 test 全绿;生成器跑通输出新 manifest。

## 红线
- 只读派生表(adminDivision/publicInstitution)可整理;**绝不碰用户数据表**。data_version 与 schemaVersion 解耦。china.sqlite 是唯一真源,生成器零修正名字。

## 执行结果(2026-06-18 完工)
- **生成器**:两个 mjs 的 manifest 都加了 `provinces:[{code/name, ver}]`(省级内容版本);行政区 ver=该省市/镇分片 sha256,机构 ver=后端 manifest_version。
- **客户端**:新 `DataVersionKv`(AppKv 存全局+省级版本游标,与 schemaVersion 解耦);两 loader 加 `ensureSynced()`=版本驱动增量 reconcile。2026-06-18 追加修正:全局 `version` 不再短路省级 `ver` 检查;旧格式 manifest 不再因本地已有数据跳过;单省 reconcile 改成行级 diff,只 upsert 新增/字段变化行并删除 absent。
- **验证**:`flutter analyze` 全项目 0 issue;`flutter test test/citizen/public/` 41 passed/0 failed(覆盖首装全量/版本相等秒过/改名/删除/新增/没变省不读分片/新旧 manifest 格式)。2026-06-18 追加防回归测试:全局 version 相等但省级 ver 变化仍 reconcile;旧格式 manifest 已有脏数据也 reconcile;同省仅变化行会进入 upsert。
- **文档**:ADR-021 加「客户端增量同步」节 + 更新 client 行为描述。
- **改动**:18 改 + 3 新(全在 `citizenapp/lib/citizen/public/data` + test + tools + 1 manifest),零外溢。
- **资产包**:已用真实 CID 本地接口重跑 `generate_public_institution_bundle.mjs --version 3`;公开包 43 省、243520 条可见公权机构。城市级 44115 条、镇级 198655 条名称前缀扫描错位 0。

状态:**完工(2026-06-18)。代码+测试+文档齐;机构包重跑待后端**。
