# 任务卡:行政区唯一真源(机构存 code + 字典派生)+ 数据清理迁移

属 ADR-021。把 wuminapp 公权机构显示的行政区名字从"多份独立副本"收敛为"唯一从 china.sqlite 派生",并清理本轮区划重构产生的孤儿/复用码机构。

状态:**全部完成并端到端验证通过(2026-06-16)**。代码(A1/A2/A3/B5/B6)+ 上线切换(新后端部署/purge 删 1485 孤儿/机构包重生 0 孤儿)+ 单源链路核验全部落地。

## 执行进度(2026-06-16)
**已完成并验证(SFID 侧,deployed 系统未受影响——旧后端仍在跑、旧客户端仍工作):**
- ✅ B6 数据:`migration_2026_06_16_retire_ln_001.sql` 应用——中西镇 001→005/葵青 001→010/大堂 001→006,旧 001 退役进 `town_tombstones` 表;47574 镇零重复。
- ✅ A2 字典生成器:`wuminapp/tools/generate_admin_division_bundle.mjs`(node:sqlite 直 dump,零映射),已生成 `wuminapp/assets/admin_divisions/`(省43/市3185/镇47574,manifest 带 china_sqlite_sha256);中西镇=005 已反映。
- ✅ A1 后端:`public_institution.rs` SELECT 改吐 `p_code/c_code/t_code AS province_code/city_code/town_code`、**停吐 s.province/s.city/s.town 名字**;`from_pg_row` 全改**按列名取**(根治索引漂移);DTO province/city/town→code。cargo test 61 passed。
- ✅ B6 铁律:`store.rs::load_provinces` 加载即断言 (省,市,镇) code 无重复(panic);`agent-rules.md` 加死规则;CI `china/data/check_code_immutable.py`(PASS);`feedback_china_code_immutable` 记忆。
- ✅ pubspec 注册 `assets/admin_divisions/`(含 cities/towns 子目录)。

**待续(协调上线切换,需 flutter/postgres/重启后端):**
- ✅ A3 wuminapp Dart(2026-06-16 落地,flutter analyze 0 issue / 公权+治理 138 tests 全绿):
  - Isar `AdminDivisionEntity`(`lib/isar/wallet_isar.dart`,唯一键 divisionKey=`level|pc|cc|tc`、level/code/scopeKey/name/dictVersion 索引)+ schema 版本 6→7(<7 清 publicInstitution+adminDivision 表首启重灌)+ build_runner 重生 `.g.dart`。
  - `PublicInstitutionEntity` 字段彻底改名 province/city/town→provinceCode/cityCode/townCode(含 @Index)。
  - DTO `public_institution_dto.dart` 改吃 `json['province_code']/['city_code']/['town_code']`(**无名字 fallback**,死规则不留旧方案)。
  - 新字典层:`admin_division_dto.dart`(divisionKey/scopeKey 工具 + DTO)、`admin_division_store.dart`+`isar_admin_division_store.dart`(`divisionName` 命中回退 code、`divisionsByLevel`、分块 2000 upsert)、`admin_division_bundle_loader.dart`(读 assets 灌字典,库空才灌幂等)、`area_path_formatter.dart`(`formatAreaPath` 有 town 显省·市·镇/空 town 只到市/缺失回退 code)。
  - `public_institution_bundle_loader.dart` 末尾灌字典(机构库非空时字典仍独立判空补灌)。
  - `isar_public_institution_store.dart` `listCities` 改按 cityCode 去重 + provinceCodeEqualTo 查询。
  - repository 接字典:`cityName`/`areaPath`/`institutionAreaPath` 预 join view-model(不在 build await)。
  - 省名源:`public_provinces.dart` 重写为 `PublicProvinceItem(code,fullName,displayName)`,省 code 从省储会 sfidNumber 前缀派生(链上常量,认可省名源);`publicProvinceNamesSet()` 守卫断言「链上省名集合==字典 provinces.json」(`public_provinces_test.dart`)。
  - UI 三处:`public_page.dart`(省鸭按 code 选中/市名字典 join VM/关注所属地预 join)、`city_institution_list_page.dart`(收 provinceCode/cityCode/cityName)、`public_institution_detail_page.dart`(所属地 `institutionAreaPath` 预 join)。
  - 顺手:`sfid_directory_lookup.dart` 反查改 code→名 join(省名链上常量、市名字典),保 governance 详情 SfidDirectoryInfo 名字契约不变。
  - 新测试:`admin_division_test.dart`(键工具/divisionName 回退/formatAreaPath 三态/字典 loader/listCities code 去重)、`public_provinces_test.dart`(链上==字典守卫);fake 新增 `fake_admin_division_store.dart`,harness/各测试改吃 code+seed 字典。
  - **遗留环境失败(非本改动)**:`widget_test.dart` app bootstraps + `im/im_mls_native*` 3 个因缺 `libsmoldot.dylib` native 库报错,与公权/字典无关、文件未改。
- ✅ B5 purge CLI `purge-orphan-institutions`(2026-06-16,cargo check + test 60+5 过):`china/store.rs::town_exists(pc,cc,tc)`(空 tc 永真,大小写不敏感)+ 4 单测;`main.rs` 加 `BackendCommand::PurgeOrphanInstitutions{dry_run,backup_path}` 子命令(默认 dry-run);`scan_orphan_institutions`(SQL 预筛 t_code 非空 + china::town_exists 确认,白名单空 t_code)、`export_orphan_backup`(COPY TSV 落 purge_orphan_backup_<ts>.sql)、`delete_orphan_institutions_by_province`(逐省单事务级联删 accounts→docs→audit(target_sfid)→gov|private(按kind)→ids(无分区)→subjects)。绝不动 sfid_number、绝不删空 t_code。

**上线切换进度(2026-06-16):**
  - ✅ 新后端已部署(PID 32018):接口确认返回 province_code/city_code/town_code 不再名字。
  - ✅ 机构包已对新后端重生:`generate_public_institution_bundle.mjs` 写 287,790 机构全带 code(province=None/province_code='LN'…),manifest 新版本;字典包已就位。客户端三件套(机构包 code + 字典 code→名 + 客户端 join)现已一致,**新客户端可端到端工作**。
  - ✅ **purge 已执行**:用户用 `set -a;source sfid/.env.dev.local;set +a; ./backend/target/debug/sfid-backend purge-orphan-institutions --apply` 跑通(agent 因安全护栏不能加载 DB 凭据做批量删,由用户侧执行);删 **1,485 条孤儿**(subjects_deleted=1485),自动备份 `sfid/purge_orphan_backup_20260616042108.sql`。
  - ✅ **机构包再次重生**:286,305 条(287790−1485),**孤儿=0**;237,780 个镇级机构全部能在字典解析。
  - ✅ **端到端单源核验通过**:珠海市机构 town_code='030'(注册时"联港工业区")经字典显示为"联港镇"(china.sqlite 改名后显示自动跟变,town_code 字节不变);香港岛X科孤儿已删光;中西镇旧001退役(字典无)、新005→中西镇。
  - **遗留(可选 follow-up,非本次范围)**:postgres `subjects.province/city/town` 名字列仍被 registration.rs 写、仍可能被 SFID admin web 读——公权机构/wuminapp 路径已不依赖它们。彻底删名字列需先审计 admin web 读路径,**不在本次 wuminapp 单源范围**,未动以免破坏 admin。
  **硬约束**:新后端返 code 一上线,旧机构包(assets/public_institutions/*.json 仍是名字+null code)与旧客户端会显示空——后端+机构包重生+客户端 A3 必须同批上,不可单独部署后端。
  **端到端核验**:改一镇名(code 不变)→重生字典→app 显示新名、机构 town_code 字节不变。

## A 架构

### A1 后端接口 `sfid/backend/wuminapp/public_institution.rs`
- SELECT 改吐 `s.province_code, s.c_code, COALESCE(s.town_code,'')`,**停吐 `s.province/s.city/s.town`**(先核 `core/db.rs:327-363` 实名:市级是 `c_code` 不是 city_code)。
- `from_pg_row`(108-129)**全部改 `row.get("列名")`**,消除裸位置索引 panic(H-2)。
- 查询入参 `province/city`(157/166 `province_code_by_name`)改为**直接收 code**;加 `resolve_*` 仅做 code 存在性校验,不再"名字→code 匹配"(H-1)。
- `subjects/registration.rs:442-447` 创建路径**停写 province/city/town 名字列**,只写三 code。
- version(MAX updated_at)不动;backfill town_code 会 bump updated_at,客户端增量自然拉到。

### A2 行政区字典(静态 assets,新生成器)
- 新建 `wuminapp/tools/generate_admin_division_bundle.mjs`:**直接 dump china.sqlite 三表**(零映射,不逐条调 `area_name_by_codes`,L-3)。结构:`assets/admin_divisions/{manifest.json, provinces.json, cities/<pcode>.json, towns/<pcode>.json}`。
- manifest 带 `china_sqlite_hash`(store.rs:26 已有)+ version,与机构包同批对齐(H-3 版本耦合)。
- 体积:43+3185+47574≈5万条,raw ~1.7MB,gzip <500KB。

### A3 wuminapp 数据层
- 新增 Isar `AdminDivisionEntity`(`lib/isar/wallet_isar.dart`):唯一键 `divisionKey = level|pcode|ccode|tcode`(**复合,非裸 code**;LN 三市都有镇 001,M-3)、level、code、scopeKey、name、dictVersion。
- 机构实体字段**改名** `province→provinceCode/city→cityCode/town→townCode`(不复用旧字段塞新语义,L-2);首启检测 schema 版本,旧版清表重灌(公权目录是只读派生,无用户数据)。
- DTO `public_institution_dto.dart` 吃 code。
- `bundle_loader` 末尾分批灌字典(沿用 _upsertChunk≈2000,flutter test --concurrency=1)。
- `isar_public_institution_store.dart`:`listCities` 改 **code 去重**;加 `divisionName(level,scopeKey,code)`/`divisionsByLevel`。
- UI 三处(`public_page.dart:178`/`city_institution_list_page.dart:53`/`public_institution_detail_page.dart:246`)统一走 `formatAreaPath(p,c,t)`:**有 town 显到镇、空 town 只到市、缺失回退显 code**(M-3/H-3)。**不在 widget build 里 await**,repo 层预 join 成 view-model。
- 省名:左栏导航改遍历字典省 code,显示走链上常量 map(④认可)。**加启动/测试断言:`kProvincialCouncils` 省名集合==字典省名集合**,把"逐字对齐"变 CI 守卫(M-4)。

### A4 单源不变量
字典唯一写入口=bundle_loader;PR checklist + 可做 hookify:禁新增名字语义 Isar 字段、禁 `.dart` 硬编码镇/市中文名做映射、禁后端新接口 SELECT `s.province/s.city/s.town`。生成器零"修正名字"逻辑。

## B 迁移

### B5 删孤儿/复用码旧机构(新 Rust CLI `purge-orphan-institutions`,参照 purge-legacy-sfid)
- 走 `china::store` 内存树判定(**不在 PG join sqlite**)。
- 只圈 `town_code 非空且 (pc,cc,tc) 反查不到`;**显式排除空 town_code**(市级/储委会白名单,M-2)。
- `--dry-run` 出清单(sfid_number+town+town_code+原因+category/org_code/institution_code),人工核**无一命中冻结常量号**。
- `--apply`:先 `pg_dump` 被删行落 `purge_backup_<ts>.sql`(删除唯一回滚保证),再逐省单事务级联删 `accounts→docs→audit→gov|private→ids→subjects`(按 p_code 命中子分区,禁跨省一条 SQL)。
- 纯设施镇下机构:被删镇 code 已不在 towns 表 → 自动归入"反查空"。

### B6 退役 001 + code 铁律 + 墓碑
- china.sqlite:`UPDATE towns SET code=<该市新code>` 给中西(LN 001)/葵青(LN 002)/大堂(LN 003)三镇换号;001 在这三市**永久退役**。REINDEX + hash 重算。
- 存量指向 LN/00x/001 的机构:CLI `--remap` 改 town_code 到新 code(合法保留者)或随 B5 删(人工清单定)。
- 墓碑表:退役 code 进 tombstone,字典生成遇墓碑 code 输出"已撤销"不复用名;区划脚本(normalize_*.py)加"新 code 不得命中墓碑"断言。
- 铁律三处同源:`memory/07-ai/agent-rules.md` 条目 + `china/store.rs` 头注释 + 新 `feedback_china_code_immutable.md`;CI `check_code_immutable.py`(对比 git HEAD,断 code 不可复用改指)+ 运行期 `load_provinces` 后 `(pc,cc,code)` 去重 panic 断言。

### B7 顺序 / 回滚
顺序:A1 后端加 code(名字仍在)部署 → china 换码(先 `git tag` baseline)+ LFS commit → 立铁律 → purge dry-run/apply → 重生字典包+机构包(version+hash 对齐)→ wuminapp 切 code+字典 → 后端删名字列(第二 PR,客户端已全切;遵 feedback_no_compatibility 不长期双源)。
回滚:china.sqlite `git checkout <baseline> -- ... && git lfs checkout` + 重启 SFID(OnceLock 换库);PG 重放 pg_dump。

### B8 验收
- 后端 cargo test:from_pg_row 列名映射、`(pc,cc,code)` 无重复、三镇换码后新 code 命中旧 001 落空、purge dry-run 识别孤儿。
- wuminapp flutter test --concurrency=1:listCities code 去重(LN 三市 001 出 3 个不同)、字典三元组 join、缺失回退、bundle 分批。
- 端到端:改镇名(code 不变)→重生字典→app 显示新名、机构 town_code 字节不变。
- 一一对应:字典条数=china 三表 count;purge 前后差=清单;二次 dry-run 零孤儿;省名仍链上常量。

### B9 红线
不动 SFID 号生成 / 省码市码 / 链上治理常量 / chainspec。backfill 只改 town_code,绝不动 sfid_number。

## follow-up / 风险
- subjects 市级列名 `c_code` vs `city_code` 改 SQL 前必 grep core/db.rs 确认(唯一会让运行炸的点)。
- 镇 code 全国不唯一,字典键/去重一律带 (省,市) 前缀。
- 删除不可逆,CLI 必先 pg_dump。
