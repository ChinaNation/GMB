# 任务卡：wuminapp 公权列表显示全部公权机构

- 任务编号：20260619-wuminapp-public-all-gov
- 状态：done
- 所属模块：sfid,wuminapp
- 当前负责人：Codex
- 创建时间：2026-06-19

## 任务需求

wuminapp 公民端“公权机构”列表必须显示全部公权机构。公安局、教育委员会、主体属性为公法人的机构、上级为公法人的非法人都属于公权机构,不得因为 SFID 管理端拆分了“市公安局”“教育机构”等后台功能 tab 而从公民端公权列表中排除。

## 边界

- 只修改 SFID 面向 wuminapp 的匿名只读 BFF 过滤口径和由该接口导出的 wuminapp 公权机构包。
- SFID 管理端仍可保留公权机构、市公安局、教育机构等后台功能分区。
- wuminapp 前端不新增“普通公权机构 / 市公安局 / 教育委员会”分组,继续按省、市显示全部公权机构。
- 不涉及 `citizenchain/runtime/`。

## 预计修改目录

- `sfid/backend/wuminapp/`:调整公民端公开公权机构查询条件,把公安局和教育委员会纳入公权列表。
- `wuminapp/assets/public_institutions/`:按新公开口径重新生成版本 1 公权机构包。
- `memory/`:更新 ADR、行政区技术文档、wuminapp 技术文档和本任务卡,清理旧排除口径说明。

## 验收要求

- SFID 真实接口按省查询时,公开目录 count 覆盖公安局和教育委员会。
- wuminapp 公权机构包总数与 SFID 运行库公权目标一致。
- 公权机构资产包 code 交叉检查 `bad_count=0`。
- 相关 Rust/Flutter 测试通过。

## 实施记录

- 2026-06-19:修正 `sfid/backend/wuminapp/public_institution.rs` 的公民端公开过滤口径,将 `CITY_POLICE` 和 `JY` 纳入公民端公权列表;自动公权目录、手动公法人、公权下属非法人统一进入公民端公权列表。
- 2026-06-19:清理 `wuminapp/lib/citizen/public/city_institution_list_page.dart` 中按后台机构类型显示的文案,列表只展示机构名称和身份 ID。
- 2026-06-19:通过真实 SFID HTTP 接口生成过公权机构包;该批结果已在 2026-06-20 行政区重新创世清理后作废,不再作为当前验收口径。
- 2026-06-19:更新 ADR、SFID 行政区技术文档、wuminapp 技术文档和历史任务卡,清理“公民端公开包遗漏公安局/教育委员会”的旧口径。
- 2026-06-20:后续行政区重新创世清理再次覆盖本任务资产包结果:当前 `wuminapp/assets/public_institutions/` 为 manifest `version=1`,43 省,共 245716 条公民端公权机构,伊犁省 1697 条。

## 验收记录

- `cargo test --manifest-path sfid/backend/Cargo.toml citizen_public_filter_keeps_all_public_institutions`:通过。
- `cargo test --manifest-path sfid/backend/Cargo.toml public_dto_`:通过。
- `cargo check --manifest-path sfid/backend/Cargo.toml`:通过。
- `python3 sfid/backend/china/check_code_immutable.py`:通过。
- 2026-06-19 旧包接口抽样和统计已被 2026-06-20 重新创世后的当前包覆盖,不再作为验收口径。
- 2026-06-20 当前公权机构包统计:总数 245716,包含 `CITY_POLICE=2872`、`CITY_EDU=2872`、`JY=2873`、`PUBLIC_SECURITY=2872`;资产包 code 交叉检查 `bad_count=0`。
- `flutter test test/citizen/public/admin_division_test.dart test/citizen/public/public_institution_bundle_loader_test.dart test/citizen/public/public_provinces_test.dart test/citizen/public/public_page_test.dart`:通过。
- 旧公开包遗漏口径残留扫描:通过,未再命中本任务清理范围内的旧事实。
- `git diff --check`:通过。
