# SFID 系统模块残留与文档一致性整改

## 任务来源

用户要求再次检查 SFID 系统及其文档和注释，确认是否存在需要继续整合、拆分、遗漏或残留的问题，并执行整改。

## 本次目标

- 清理 SFID 后端仍然存在的空壳转发模块和错位 handler。
- 将 wuminapp 投票账户接口从公民绑定文件中拆到投票账户文件。
- 修正 SFID 文档中已过期的目录、文件名和模块说明。
- 补齐缺失的中文模块注释，清理与当前结构不一致的残留说明。

## 影响范围

- `sfid/backend/citizens/`
- `sfid/backend/main.rs`
- `sfid/backend/sheng_admins/`
- `sfid/backend/shi_admins/`
- `memory/05-modules/sfid/`

## 验收标准

- 后端路由不再依赖仅做转发的 `shi_admins` 后端空壳模块。
- 投票账户接口位于 `citizens/vote.rs`，公民绑定接口留在 `citizens/binding.rs`。
- 文档中的旧路径、旧模块说明、旧残留声明更新为当前真实结构。
- 代码补齐必要中文注释。
- `cargo check` 与前端构建通过，残留扫描无本次新增旧路径。

## 完成记录

- 已拆分 `login/mod.rs` 为 `model.rs`、`handler.rs`、`qr_login.rs`、`guards.rs`、`signature.rs`，对外 `login::...` API 保持不变。
- 已将 wuminapp 投票账户登记/查询从 `citizens/binding.rs` 迁入 `citizens/vote.rs`。
- 已删除后端 `shi_admins` 空壳转发目录，CPMS 状态扫码路由直接指向 `citizens/status.rs`。
- 已更新 `memory/05-modules/sfid/` 下相关技术文档和旧路径。
- 已执行 `cargo fmt`、`cargo check --manifest-path sfid/backend/Cargo.toml`、`npm run build` 和残留扫描。
