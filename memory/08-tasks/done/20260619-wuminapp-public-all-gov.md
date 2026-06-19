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
- 2026-06-19:通过真实 SFID HTTP 接口重新生成 `wuminapp/assets/public_institutions/`,manifest `version=1`,43 省,共 249343 条公民端公权机构,伊犁省 1737 条。
- 2026-06-19:更新 ADR、SFID 行政区技术文档、wuminapp 技术文档和历史任务卡,清理“公民端公开包遗漏公安局/教育委员会”的旧口径。

## 验收记录

- `cargo test --manifest-path sfid/backend/Cargo.toml citizen_public_filter_keeps_all_public_institutions`:通过。
- `cargo test --manifest-path sfid/backend/Cargo.toml public_dto_`:通过。
- `cargo check --manifest-path sfid/backend/Cargo.toml`:通过。
- `python3 sfid/backend/china/check_code_immutable.py`:通过。
- SFID 真实 HTTP 接口 `GET /api/v1/app/public-institutions/version?province=伊犁省`:返回 `count=1737`。
- 真实接口分页抽样确认包含 `博乐市公安局`、`博乐市公民教育委员会`、`伊犁省储行`。
- 公权机构包统计:总数 249343,包含 `CITY_POLICE=2938`、`JY=2939`、`PROVINCE_RESERVE_BANK=43`、`PUBLIC_SECURITY=2938`。
- 公权机构资产包 code 交叉检查:249343 条记录的省、市、镇 code 均能在行政区包中定位,`bad_count=0`。
- `flutter test test/citizen/public/admin_division_test.dart test/citizen/public/public_institution_bundle_loader_test.dart test/citizen/public/public_provinces_test.dart test/citizen/public/public_page_test.dart`:通过。
- 旧公开包遗漏口径残留扫描:通过,未再命中本任务清理范围内的旧事实。
- `git diff --check`:通过。
