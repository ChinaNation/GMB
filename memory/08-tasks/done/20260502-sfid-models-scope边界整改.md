# 20260502 SFID models/scope 边界整改

## 任务目标

- `models` 只保留全局共享模型，不再承载公民、CPMS、SFID 元信息、机构链状态等业务 DTO。
- `scope` 只保留权限范围规则，不再承载 HTTP handler、CPMS 专用判断和 pubkey 工具。
- 更新路由、引用、文档和任务索引，清理残留。

## 改动范围

- `sfid/backend/models`
- `sfid/backend/scope`
- `sfid/backend/citizens`
- `sfid/backend/cpms`
- `sfid/backend/sfid`
- `sfid/backend/crypto`
- `sfid/backend/main.rs`
- `memory/05-modules/sfid/backend`

## 验收标准

- `scope` 目录只保留 `mod.rs / rules.rs / filter.rs / admin_province.rs`。
- `models` 目录只保留全局共享模型文件。
- 公民、CPMS、SFID 元信息、机构链状态类型归属到对应功能模块。
- 后端格式化、编译检查通过；前端构建通过。
- 文档与目录结构一致，旧路径无活跃引用。

## 状态

- 已完成。

## 完成记录

- `models` 收敛为 `mod.rs / error.rs / role.rs / store.rs`。
- 公民 DTO 迁入 `citizens/model.rs`,CPMS DTO 迁入 `cpms/model.rs`,SFID 元信息 DTO 迁入 `sfid/model.rs`。
- 机构链状态迁入 `institutions/model.rs`。
- `scope` 收敛为 `mod.rs / rules.rs / filter.rs / admin_province.rs`。
- 审计查询迁入 `audit.rs`,公民查询迁入 `citizens/handler.rs`,CPMS scope 谓词迁入 `cpms/scope.rs`,pubkey 工具迁入 `crypto/pubkey.rs`。
- 已更新 backend layout、models、scope、citizens、cpms、sfid、audit、crypto 文档。
- 已执行 `cargo fmt --manifest-path sfid/backend/Cargo.toml`、`cargo check --manifest-path sfid/backend/Cargo.toml` 和 `npm run build`。
