# SFID 后端 src 目录平移到 backend 根目录

- 创建时间:2026-05-02
- 状态：done

## 需求

删除 `sfid/backend/src/` 这一层目录,把 `src` 下所有后端源码模块平移到
`sfid/backend/` 根目录,让后端源码布局与前端一样直接按功能目录展开。

## 边界规则

- 只移动 SFID 后端 Rust 源码目录和入口文件。
- 不移动 `sfid/backend/db/`、`sfid/backend/scripts/`、`sfid/backend/tests/`、`sfid/backend/target/`。
- Cargo 入口改为显式 `[[bin]] path = "main.rs"`。
- 删除空 `sfid/backend/src/`,不得留下兼容壳或影子目录。
- SFID 后端不得恢复独立 `backend/src/` 或 `backend/chain/` 业务目录。

## 预计修改目录

- `sfid/backend/`
  - 中文注释:后端源码根目录;承接原 `src/main.rs` 和所有功能模块目录,代码路径从 `backend/src/...` 改为 `backend/...`。
- `sfid/backend/Cargo.toml`
  - 中文注释:Rust 构建配置;显式声明二进制入口 `main.rs`,保证删除 `src/main.rs` 后仍可构建。
- `sfid/frontend/`
  - 中文注释:只更新指向后端源码的中文注释或路径说明,不调整前端业务逻辑。
- `memory/05-modules/sfid/`
  - 中文注释:更新 SFID 模块技术文档中的后端路径和目录规则。
- `memory/07-ai/`、`memory/AGENTS.md`
  - 中文注释:更新 AI 强制规则,禁止恢复 `sfid/backend/src/` 和独立 `backend/chain/`。

## 验收

- `sfid/backend/src/` 不存在。
- `sfid/backend/main.rs` 存在,`Cargo.toml` 显式配置 `[[bin]] path = "main.rs"`。
- `cargo fmt && cargo check` 通过。
- 新路径下无 `backend/src` 活跃代码引用。
- 文档、中文注释、残留清理完成。

## 完成记录

- 已将 `sfid/backend/src/main.rs` 平移为 `sfid/backend/main.rs`。
- 已将 `app_core`、`citizens`、`crypto`、`indexer`、`institutions`、`login`、`models`、`qr`、`scope`、`sfid`、`sheng_admins`、`shi_admins`、`store_shards` 平移到 `sfid/backend/` 根目录。
- 已删除空 `sfid/backend/src/`。
- 已在 `sfid/backend/Cargo.toml` 显式配置 `[[bin]] path = "main.rs"`。
- 已更新 SFID 后端/前端注释、模块文档、repo-map、AI 规则与上下文加载脚本。
- 已新增 `memory/05-modules/sfid/backend/BACKEND_LAYOUT.md` 固化新目录规则。
- 已执行 `cd sfid/backend && cargo fmt && cargo check`,检查通过;仅保留既有 `sfid/province.rs` 未读字段 warning。

## 完成信息

- 完成时间：2026-05-02 14:57:17
- 完成摘要：SFID 后端源码已从 backend/src 平移到 backend 根目录,Cargo 显式入口 main.rs,旧 src 目录已删除,文档和规则已更新,cargo fmt/check 通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
