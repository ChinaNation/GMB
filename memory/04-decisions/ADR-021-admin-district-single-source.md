# ADR-021:行政区唯一真源 —— 机构存 code,名字一律派生自 china.sqlite

状态:Proposed(2026-06-16)
关联:[[ADR-018]](wuminapp 混合模式)、project_village_unify_cun_lu_2026_06_15、reference_wuminapp_public_institution_bundle、feedback_no_compatibility

## 背景 / 问题
行政区唯一真源 = `sfid/backend/china/data/china.sqlite`(省/市/镇/村/路)。但行政区**名字**目前被独立拷贝在多处:
- postgres `subjects.province/city/town`(机构注册时写入的名字字符串,从不回灌)
- wuminapp 机构数据包 `assets/public_institutions/*.json`(从 postgres 快照,带名字)
- wuminapp `public_provinces.dart`(省名来自链上 `kProvincialCouncils`,靠"逐字对齐"约定)

改 china.sqlite(如本轮把香港岛拆成中西/湾仔/东/南镇、联港工业区→联港镇)**不会**传导到这些副本 → 数据漂移。要求:**行政区只有唯一真源,别处零独立副本**。省名走链上常量(④⑤)已认可不动。

## 决策
机构记录只带 `(province_code, city_code, town_code)` 三元组;行政区**名字**唯一来自一份从 china.sqlite 直接 dump 的「行政区字典」,wuminapp 本地按三元组 join 显示。**同步停用所有名字副本**(不是新增 code 列、保留名字 —— 那只是多一份真源)。

铁律:**china.sqlite 行政区 code 不可变、不复用**;撤并的 code 进墓碑表永久占位,新镇只取该 (省,市) 内全新 code。

## 不触及(红线,已读码验证)
SFID 号生成(`number/` 零 town;储委会/部委号是 `china_*.rs` 冻结常量)、省码/市码、链上治理常量、chainspec/runtime。backfill 只 UPDATE town_code,**绝不动 sfid_number**。

## 方案要点(详见任务卡 20260616-admin-district-single-source)
**A 架构**
- A1 后端 `wuminapp/public_institution.rs`:SELECT 改吐 `province_code/c_code/town_code`、**停吐 `s.province/s.city/s.town` 名字**;`from_pg_row` 全改**按列名取**(消除裸索引 panic);查询入参由"名字→code 匹配"改为**直接收 code**;`registration.rs` 创建路径停写名字列。version 口径(MAX updated_at)不变。
- A2 行政区字典:**静态 assets 包**(否决后端增量接口),新生成器 `wuminapp/tools/generate_admin_division_bundle.mjs` 直接 dump china.sqlite 三表(零映射),按省分片,manifest 带 `china_sqlite_hash` + version,与机构包同源同批对齐。
- A3 wuminapp:新增 Isar `AdminDivisionEntity`(复合唯一键 `(pc,cc,tc)`,**非裸 code** —— LN 三市都有镇 code 001);机构实体字段改名 `provinceCode/cityCode/townCode`(不复用旧名字字段塞新语义);`bundle_loader` 分批灌字典;UI 三处统一走 `formatAreaPath(p,c,t)` 三元组 join,空 town 只显到市,缺失回退显 code 不崩。
- A4 单源不变量:字典唯一写入口、grep/hook 卡点(禁新增名字语义字段/禁硬编码中文区名/禁接口下发 s.town)、生成器零映射。

**B 迁移**
- B5 删孤儿/复用码旧机构:**Rust CLI**(不在 PG join china,china 在进程内 `china::store` 缓存);只圈 `town_code 非空且反查不到`,**显式白名单空 town_code**(市级/储委会合法);dry-run 出清单人工核常量号 → `pg_dump` 备份 → 逐省单事务级联删(accounts/docs/audit/gov|private/ids/subjects)。
- B6 退役 001 + 铁律 + 墓碑:china.sqlite 给中西/葵青/大堂镇换该市新 code,001 进墓碑永不复用;存量旧机构 town_code backfill 到新 code 或随 B5 删;铁律入 `agent-rules.md` + `china/store.rs` 头注释 + CI 校验脚本 + 运行期 `(pc,cc,code)` 去重 panic 断言;区划脚本加"新 code 不得命中墓碑"断言。
- B7 顺序:后端加 code→china 换码 LFS commit(先 tag baseline)→立铁律→purge dry-run/apply→重生字典包+机构包(版本对齐)→wuminapp 切 code+字典→后端删名字列。回滚:git-LFS checkout + 重启换 OnceLock 缓存;PG 靠 pg_dump 重放。
- B8 验收:端到端「改镇名(code 不变)→重生字典→app 显示更新、机构记录字节不变」;一一对应核验(字典条数=china 三表 count、purge 前后差=清单、二次 dry-run 零孤儿、省名仍来自链上常量)。

## 单源审查收口(四条硬约束)
1. 三处名字副本同步停用(postgres 名字列 / 机构包省全名 / wuminapp 链上省名作查询键)。
2. 字典严格 china.sqlite 单源 dump,带 `china_sqlite_hash` 做机构包↔字典版本耦合;缺失回退显 code + 日志。
3. 孤儿删除只圈"非空 town_code 且反查空",dry-run 人工核冻结常量号。
4. 立"code 不可变不复用 + 墓碑表"铁律,收口本次 001 复用。
