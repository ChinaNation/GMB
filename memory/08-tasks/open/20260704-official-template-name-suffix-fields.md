# 任务卡：公权机构模板名称后缀字段统一

## 任务需求

将 `OfficialOrgTemplate` 中容易误解的短称后缀字段和全称后缀字段统一为业务字段名：

- 短称后缀字段 -> `cid_short_name_suffix`
- 全称后缀字段 -> `cid_full_name_suffix`

同时将模板组装方法统一为：

- 短称组装方法 -> `cid_short_name(...)`
- 全称组装方法 -> `cid_full_name(...)`

这些字段仍表示“模板后缀”，不是最终机构实体名称。最终生成出的机构名称继续使用 `cid_full_name` / `cid_short_name` 字段承载。

## 预计修改目录

- `citizenchain/runtime/primitives/cid/`
  - 用途：统一公权机构命名模板字段和组装方法命名。
  - 边界：只改字段名、方法名、注释和测试引用；不改模板值、CID 派生、创世数量、链上编码或 storage。
  - 类型：runtime 代码与注释；已获得用户二次确认。

- `memory/04-decisions/`
  - 用途：同步 ADR-031 中的模板字段说明。
  - 边界：只更新字段命名表述。
  - 类型：文档。

- `memory/08-tasks/`
  - 用途：记录本次任务，并清理旧任务卡中的旧字段名残留。
  - 边界：只更新相关任务卡说明。
  - 类型：文档与残留清理。

## 实施步骤

1. 创建任务卡。
2. 修改 `OfficialOrgTemplate` 字段名和组装方法名。
3. 同步派生逻辑调用点和单元测试。
4. 更新 ADR 与任务卡文档，清理旧字段名残留。
5. 执行残留扫描、编译检查和 diff 检查。

## 验收标准

- `OfficialOrgTemplate` 不再出现旧短称/全称后缀字段名。
- 模板命名文档统一使用 `cid_short_name_suffix` / `cid_full_name_suffix`。
- 最终派生字段仍为 `cid_full_name` / `cid_short_name`。
- `cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain` 通过。
- `git diff --check` 通过。

## 执行记录

- 2026-07-04：任务卡创建，用户已确认允许创建任务卡并修改 runtime 模板字段命名。
- 2026-07-04：已将 `OfficialOrgTemplate` 的模板后缀字段统一为 `cid_short_name_suffix` / `cid_full_name_suffix`，并将模板组装方法统一为 `cid_short_name` / `cid_full_name`。
- 2026-07-04：已同步 `official_derive.rs` 调用点、模板单元测试、ADR-031 与相关任务卡说明；复扫裸旧字段名和旧方法名无残留。
- 2026-07-04：`cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain` 通过；`cargo test --manifest-path citizenchain/Cargo.toml -p primitives` 通过；`cargo fmt --manifest-path citizenchain/Cargo.toml -p citizenchain -- --check` 通过；`git diff --check` 通过。
