# 任务卡：镇下地址段与详细地址改造

- 状态:完成
- 模块:SFID 行政区 / CPMS 地址
- 创建时间:2026-06-20

## 目标

把镇下面的第四层统一改为“地址段”。省、市、镇仍是行政区;镇下面不再是行政区,
只作为既有地名地址段。档案地址由“地址段 + 详细地址输入段”组成完整详细地址。

## 规则

- 镇下面第四层统一称为地址段,英文统一为 `address_unit`。
- 地址段不强制以“村”或“路”结尾。
- 地址段保留当地地址名,`社区`、`村`、`路`、`巷`、`生活区` 等可以作为地址段。
- 地址段不保留 `居委会`、`居民委员会`、`村委会`、`村民委员会`、`委员会`、`办事处`、`管理处`、`管委会` 等组织或管理机构词。
- 地址段归一示例:`xx办事处社区` 归一为 `xx社区`,`xx管委会路` 归一为 `xx路`,`大坪村民委员会` 归一为 `大坪村`。
- CPMS 档案地址拆成选择的 `address_unit_id` 与人工输入的 `address_detail`。
- 本任务不修改 `citizenchain/runtime/**`,不生成 wuminapp 公权机构包,不生成 SFID 公权机构。

## 预计修改目录

- `sfid/backend/china/`:修改行政区开发库 SQLite 第四层表结构、数据和校验脚本。
- `cpms/backend/address/`:修改 CPMS 地址同步和地址查询 API。
- `cpms/backend/db/`:修改 CPMS 当前开发基线 schema 与 migration。
- `cpms/backend/dangan/`:修改档案创建、编辑、查询和地址校验字段。
- `cpms/backend/common/`:同步共享档案 DTO 字段。
- `cpms/frontend/address/`:修改地址 API 类型。
- `cpms/frontend/dangan/`:修改档案创建/编辑页面字段和文案。
- `cpms/frontend/super_admin/`:修改系统设置地址查看文案。
- `memory/04-decisions/`:更新行政区唯一真源 ADR。
- `memory/05-modules/`:更新 SFID / CPMS 技术文档。

## 验收

- `python3 sfid/backend/china/check_code_immutable.py`:PASS,地址段唯一且无基层组织尾词残留。
- `sqlite3 sfid/backend/china/china.sqlite "PRAGMA integrity_check"`:ok。
- `address_units`:598655 条,旧第四层表不存在。
- `address_units.source_code`:598655 条全部非空;其中 535084 条为官方数字来源码,63571 条为 `LOCAL-*` 本地稳定来源码。
- `china.sqlite` SHA-256:`c477cb5a300eac9f56d53beaef235617a6fc64584a0f1cffd8c85b2537840bbb`。
- 福建、海南删除/合并后的市 code 空洞已按重新创世口径重排,省内市 code 连续。
- 42 个镇级伪行政区已删除,同步删除其下 235 条地址段;镇级伪行政区关键词命中归零。
- 154 条地址段名称中的组织或管理机构词已清理;`社区` 作为合法地址段保留。
- 568 条 `xx虚拟路` 已归一为 `xx`;3 条纯 `虚拟路` 已删除,其中 2 条对应的功能区壳镇同步删除。
- 46 条原始名含 `社区` 的纯功能词地址段已恢复为 `xx社区`;26 条 `LOCAL-*` 来源的 `xx虚拟路` 合成占位地址段已删除,同步删除因此空掉的 24 个镇并重排受影响市的镇 code。
- 2026-06-20:已基于当前 `china.sqlite` 重新生成 `wuminapp/assets/admin_divisions/`,manifest `version=1`,43 省、2872 市、39227 镇,`china_sqlite_sha256=c477cb5a300eac9f56d53beaef235617a6fc64584a0f1cffd8c85b2537840bbb`。
- 2026-06-20:已执行 SFID 公权机构运行库对账和 strict check;首次全量同步为 `scopes=43 inserted=55354 updated=190362 account_inserted=491475 removed=58281`,本轮复跑为 `scopes=0 inserted=0 updated=0 account_inserted=0 removed=0`;最终 strict 为 `ok=true manifest_current=true target_count=245716 active_count=245716 missing=0 mismatched=0 missing_accounts=0 obsolete=0 catalog_hash=499c1ee8af974f0a79affe6731883d491052da1767f4a99ae072ff29c1f42ea6`。
- 2026-06-20:已通过当前 SFID 真实公开接口重新生成 `wuminapp/assets/public_institutions/`,manifest `version=1`,43 省,共 245716 条公民端公权机构,包含 `CITY_POLICE=2872`、`CITY_EDU=2872`、`JY=2873`、`PUBLIC_SECURITY=2872`;资产包 code 交叉检查 `bad_count=0`。
- `cargo fmt --manifest-path cpms/backend/Cargo.toml`:完成。
- `cargo check --manifest-path cpms/backend/Cargo.toml`:通过。
- `cargo test --manifest-path cpms/backend/Cargo.toml`:32 passed。
- `cargo build --manifest-path cpms/backend/Cargo.toml`:通过,刷新当前 debug 二进制。
- `npm run build` in `cpms/frontend`:通过。
- 真实运行态验收:使用临时 PostgreSQL 库启动 CPMS 后端,迁移完成后写入 BP001 安装城市并重启,
  启动同步得到 `address_towns=17`,`address_units=168`,样例地址段为 `多福巷 / 银闸 / 东厂`;
  验收后已删除临时库。
- 残留扫描:当前代码和模块文档中旧第四层表、旧字段、旧说法仅剩
  `sfid/backend/china/check_code_immutable.py` 的禁止项,用于防止旧表复活。
