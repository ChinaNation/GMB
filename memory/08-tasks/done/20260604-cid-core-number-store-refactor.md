# 20260604 CID core/number/store 目录整改

## 任务目标

- 后端统一目录命名：`app_core` 改为 `core`，`cid_number` 改为 `number`，`store_shards` 改为 `store`。
- 后端 `login` 并入 `admins/login`，`qr` 并入 `core/qr`。
- 检查并拆分 `models`，把通用响应、管理员模型、Store 聚合体、管理员安全模型、审计模型迁回对应模块，删除 `models`。
- 删除空的 `backend/scripts`，把 `backend/db` 的存储边界合并进 `store`。
- 前端同步整改公共目录：公共组件和 QR 能力归入 `frontend/core`，行政区元数据归入 `frontend/china`。
- 更新文档、补齐中文注释、清理旧目录名和旧引用残留。

## 边界要求

- `cid_number` 字段名和业务含义不变；只改编码协议目录名为 `number`。
- 行政区划归 `china`，身份 ID 编码协议归 `number`，不得混用。
- 公权机构归 `gov`，私权机构归 `private`，主体公共能力归 `subjects`。
- 不保留 `models`、`backend/login`、`backend/qr`、`backend/app_core`、`backend/cid_number`、`backend/store_shards`、`frontend/qr`、`frontend/cid` 等旧目录残留。

## 验证要求

- `cargo fmt`
- `cargo check --manifest-path citizencode/backend/Cargo.toml`
- `npm run build`（`citizencode/frontend`）
- 使用 `rg` 扫描旧目录名、旧 import、旧文档路径残留。

## 完成记录

- 后端已改为 `core`、`number`、`store`、`admins/login`、`core/qr` 目标结构。
- 前端已改为 `core`、`core/qr`、`china` 目标结构。
- `models` 已拆分到 `core`、`admins`、`store`、`audit` 所属模块。
- 空目录和旧目录已删除，根目录 `AGENTS.md` 已恢复为指向 `memory/AGENTS.md` 的符号链接。
- 已更新 CID 架构文档、模块文档、任务引用和相关注释。
- 已同步 `agent-rules`、`repo-map`、`unified-naming` 和 Store 分片铁律等活跃规则文档。
- 已通过 `cargo fmt --manifest-path citizencode/backend/Cargo.toml`。
- 已通过 `cargo check --manifest-path citizencode/backend/Cargo.toml`。
- 已通过 `npm run build`（Vite 仅输出大 chunk 提示，不影响构建结果）。
