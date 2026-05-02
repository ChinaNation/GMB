# SFID 模块技术文档

## 1. 模块定位

- 路径：`backend/sfid`
- 职责：统一提供 SFID 生成能力与行政区代码表能力。
- 产物：标准化 SFID 编码字符串（含结构段与校验位）。

## 2. 模块结构(任务卡 1 重组后)

铁律:`feedback_sfid_module_is_single_entry.md` — sfid 系统所有 SFID 相关常量、枚举、生成、校验
**必须**归位本模块,不能散在 sheng-admins / shi-admins / citizens / business / chain 等业务模块里。

- `mod.rs`
  - 纯 `pub mod` + `pub use` 聚合,不放实现。
- `generator.rs`
  - `generate_sfid_code`:SFID 生成主入口(任务卡 1 从老 mod.rs 拆出)
  - `GenerateSfidInput`:输入参数结构
  - `checksum/hash_text/resolve_p1`:私有辅助
- `validator.rs`
  - `validate_sfid_id_format`:格式校验 + 标准化
  - `SFID_ID_MAX_BYTES` / `SFID_ID_SEGMENT_*` 常量(任务卡 1 从 sheng-admins/institutions.rs 搬回)
- `a3.rs`
  - `A3` 枚举(GMR/ZRR/ZNR/GFR/SFR/FFR)+ `from_str` / `as_code` / `label_zh` / `all_a3`
  - 兼容 legacy `resolve_a3(&str) -> Result<&'static str>`
- `institution_code.rs`
  - `InstitutionCode` 枚举(ZG/ZF/LF/SF/JC/JY/CB/CH/TG)
  - 兼容 legacy `resolve_org_type`
- `category.rs`
  - `InstitutionCategory` 枚举(PublicSecurity / GovInstitution / PrivateInstitution)
  - `classify(a3, code, name)`:机构分类函数(任务卡 2 使用)
  - `PUBLIC_SECURITY_INSTITUTION_NAME` 常量("公民安全局")
- `cities.rs`
  - `cities_of(province)` / `real_cities_of(province)`:城市清单高层 API
- `province.rs`
  - 43 省 `PROVINCES` 常量表,只保存 SFID 号码生成需要的省市代码
  - `province_code_by_name` / `city_code_by_name` / `province_name_by_code`
- 省管理员 main 公钥、Slot、三槽名册等已移出本模块,统一位于
  `backend/sheng_admins/province_admins.rs`。
- `admin.rs`
  - 管理端 SFID 业务接口实现(legacy,`admin_generate_sfid` / `admin_sfid_meta` / `admin_sfid_cities`)
- `city_codes/*.rs`
  - 43 个省份城市代码表(数据)

## 3. 生成规则摘要

- 编码段：`A3-R5-T2P1C1-N9-D8`
  - `A3`：主体类型（如 `GMR/ZRR/GFR/...`）
  - `R5`：省码 + 市码 / 省级占位码
  - `T2P1`：机构类型与盈利属性
  - `C1`：校验位
  - `N9`：稳定散列序列
  - `D8`：日期（`YYYYMMDD`）
- 不同 `A3` 对 `T2/P1` 有严格组合约束，生成前强校验。
- `GMR / ZRR / ZNR` 当前固定使用省级占位市码 `000`；真实市码从 `001` 起排。
- `GFR / SFR / FFR` 与机构 `site_sfid` 继续使用真实市码。
- 管理端生成后的 `sfid_code` 先进入 `generated_sfid_by_pubkey` 暂存，绑定确认阶段只能消费这份结果，不再允许绑定时兜底生成第二套口径。

## 4. 主要调用方

- `main.rs` 路由将 `admin/sfid/*` 接口接入 `sfid::admin`。
- `app_core/runtime_ops.rs` 的 `seed_demo_record` 也复用同一套生成工具，不再生成旧格式演示 `sfid`。
- `sheng_admins/institutions.rs`：省级管理员生成机构 `site_sfid`。
- `login/mod.rs` 与 `scope/admin_province.rs`：通过
  `sheng_admins::province_admins` 做角色展示和归属推断。

## 5. 命名与引用

- 当前统一模块名为 `sfid`。
- 代码统一通过 `crate::sfid::*` 引用。
- 模块目录为 `backend/sfid`。

## 6. 历史

- 2026-04-08 任务卡 1(`08-tasks/done/20260408-sfid-模块补全-任务卡1.md`):补全 a3/institution_code/cities/category/validator/generator 子文件,所有 SFID 工具归位此模块
- 2026-04-08 任务卡 0.5:`super_admin_*` → `sheng_admin_*` 相关函数重命名
