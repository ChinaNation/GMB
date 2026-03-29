# SFID 模块技术文档

## 1. 模块定位

- 路径：`backend/src/sfid`
- 职责：统一提供 SFID 生成能力与行政区代码表能力。
- 产物：标准化 SFID 编码字符串（含结构段与校验位）。

## 2. 模块结构

- `mod.rs`
  - `generate_sfid_code`：SFID 生成主入口。
  - `GenerateSfidInput`：输入参数结构。
  - `resolve_a3/resolve_p1/resolve_org_type`：业务规则约束。
  - `checksum/hash_text`：摘要与校验位计算。
- `admin.rs`
  - 管理端 SFID 业务接口实现：
  - `admin_generate_sfid`
  - `admin_sfid_meta`
  - `admin_sfid_cities`
- `province.rs`
  - 省级/市级代码数据源。
  - 省份公钥映射（用于超管归属推断）。
  - `provinces/super_admin_province/super_admin_display_name` 等查询函数。
- `city_codes/*.rs`
  - 43 个省份城市代码表。

## 3. 生成规则摘要

- 编码段：`A3-R5-T2P1C1-N9-D8`
  - `A3`：主体类型（如 `GMR/ZRR/GFR/...`）
  - `R5`：省市编码
  - `T2P1`：机构类型与盈利属性
  - `C1`：校验位
  - `N9`：稳定散列序列
  - `D8`：日期（`YYYYMMDD`）
- 不同 `A3` 对 `T2/P1` 有严格组合约束，生成前强校验。

## 4. 主要调用方

- `main.rs` 路由将 `admin/sfid/*` 接口接入 `sfid::admin`。
- `super-admins/institutions.rs`：机构管理员生成机构 `site_sfid`。
- `business/scope.rs` 与 `login/mod.rs`：使用省份公钥映射能力做角色展示和归属推断。

## 5. 命名与引用

- 当前统一模块名为 `sfid`。
- 代码统一通过 `crate::sfid::*` 引用。
- 模块目录为 `backend/src/sfid`。
