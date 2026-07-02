# CID 后端 src 目录平移到 backend 根目录

- 创建时间:2026-05-02
- 状态：done

## 需求

删除旧后端源码壳这一层目录,把其中所有后端源码模块平移到
`citizencode/backend/` 根目录,让后端源码布局与前端一样直接按功能目录展开。

## 边界规则

- 只移动 CID 后端 Rust 源码目录和入口文件。
- 不移动 `citizencode/backend/db/`、`citizencode/backend/scripts/`、`citizencode/backend/tests/`、`citizencode/backend/target/`。
- Cargo 入口改为显式 `[[bin]] path = "main.rs"`。
- 删除空旧后端源码壳,不得留下兼容壳或影子目录。
- CID 后端不得恢复独立 `backend/src/` 或 `backend/chain/` 业务目录。

## 预计修改目录

- `citizencode/backend/`
  - 后端源码根目录;承接原 `src/main.rs` 和所有功能模块目录,代码路径从 `backend/src/...` 改为 `backend/...`。
- `citizencode/backend/Cargo.toml`
  - Rust 构建配置;显式声明二进制入口 `main.rs`,保证删除 `src/main.rs` 后仍可构建。
- `citizencode/frontend/`
  - 只更新指向后端源码的中文注释或路径说明,不调整前端业务逻辑。
- `memory/05-modules/citizencode/`
  - 更新 CID 模块技术文档中的后端路径和目录规则。
- `memory/07-ai/`、`memory/AGENTS.md`
  - 更新 AI 强制规则,禁止恢复旧后端源码壳和独立 `backend/chain/`。

## 验收

- 旧后端源码壳不存在。
- `citizencode/backend/main.rs` 存在,`Cargo.toml` 显式配置 `[[bin]] path = "main.rs"`。
- `cargo fmt && cargo check` 通过。
- 新路径下无 `backend/src` 活跃代码引用。
- 文档、中文注释、残留清理完成。

## 完成记录

- 已将旧源码壳入口平移为 `citizencode/backend/main.rs`。
- 已将 `app_core`、`citizens`、`crypto`、`indexer`、`institutions`、`login`、`models`、`qr`、`scope`、`cid`、`sheng_admins`、`shi_admins`、`store_shards` 平移到 `citizencode/backend/` 根目录。
- 已删除空旧后端源码壳。
- 已在 `citizencode/backend/Cargo.toml` 显式配置 `[[bin]] path = "main.rs"`。
- 已更新 CID 后端/前端注释、模块文档、repo-map、AI 规则与上下文加载脚本。
- 已新增 `memory/05-modules/citizencode/backend/BACKEND_LAYOUT.md` 固化新目录规则。
- 已执行 `cd citizencode/backend && cargo fmt && cargo check`,检查通过;仅保留既有 `citizencode/province.rs` 未读字段 warning。

## 完成信息

- 完成时间：2026-05-02 14:57:17
- 完成摘要：CID 后端源码已从 backend/src 平移到 backend 根目录,Cargo 显式入口 main.rs,旧 src 目录已删除,文档和规则已更新,cargo fmt/check 通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
