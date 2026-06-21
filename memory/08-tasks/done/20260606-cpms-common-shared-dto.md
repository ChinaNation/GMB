# 任务卡：CPMS main.rs 共享 DTO/helper 统一进 common/（前后端一致）

## 任务需求

把 main.rs 里跨模块共享的 DTO 和 helper 抽进 `common/`，让后端 `common/` 与前端 `common/`（http.ts 响应 + types.ts 类型）对称，main.rs 只剩入口+路由+AppState 装配。

## 切分决策（已确认方向：都统一进 common）

- **进 common/**（跨模块共享，有前端对应或纯共享）：
  - `common/response.rs` ← `ApiResponse`/`ApiError`/`ok`/`err`/`cpms_error_code`（≈ 前端 http.ts）
  - `common/types.rs` ← `AdminUser`/`Archive`（≈ 前端 types.ts，字段改 `pub(crate)`，因脱离 crate 根后子模块需可见）
  - `common/admin.rs` ← `find_admin_by_pubkey`/`find_admin_by_user_id`
  - `common/audit.rs` ← `write_audit`
  - `common/encoding.rs` ← `decode_bytes`
  - `common/mod.rs` 用 `pub(crate) use` 再导出，使 `crate::common::{err, ok, ApiError, ...}` 可用
- **留 main.rs**（装配根，无前端对应，不属共享 util）：`AppState`、`MIGRATOR`、`main()`、`health()`、`validate_frontend_dir()`、`security_headers()`、`mod tests`。AppState 字段 private 但定义在 crate 根=全 crate 可见，admin/audit 访问 `state.db` 不受影响。

## 影响范围

- 新建 `citizenpassport/backend/common/{response,types,admin,audit,encoding}.rs`；改 `common/mod.rs` 再导出。
- `main.rs`：删上述结构体/函数；`health()` 加 `use common::{ok, ApiResponse}`；清理失效 import（base64 STANDARD、serde、uuid、sqlx Row、axum StatusCode）。
- 13 个消费方 `use crate::{...}` 把 `err/ok/ApiError/ApiResponse/Archive/write_audit/find_admin_*` 改成 `common::{...}` 来源；`crate::decode_bytes`(dangan/routes + login×2)→`crate::common::decode_bytes`。调用点（err()/ok() 等 264+ 处）不变。
- `CPMS_TECHNICAL.md` 模块表补 common 子模块。

## 主要风险点

- DTO 移出 crate 根后字段可见性：`AdminUser`/`Archive` 字段必须 `pub(crate)`，否则子模块访问 `.field` 编译失败。
- main.rs 失效 import 要清干净（clippy 兜底）。
- 纯结构搬迁、0 行为变化；靠 `cargo check`/`clippy`/`test` 全覆盖。
- AppState 本轮按决策留 main，不动（前端无对应，移动高 churn 低收益）。

## 是否需要先沟通

- 方向已确认（都统一进 common）。AppState 留 main 为切分决策，如需一并移动再单开。

## 执行清单

- [x] 建 `common/{response,types,admin,audit,encoding}.rs` + `common/mod.rs` `pub(crate) use` 再导出。
- [x] main.rs 删 4 结构体 + 7 函数（497→224 行），修 import（删 base64/serde/uuid、trim sqlx Row/axum StatusCode），health 加 `use common::{ok, ApiResponse}`。
- [x] 13 消费方 use 块改 `common::{...}` 来源；3 处 `crate::decode_bytes`→`crate::common::decode_bytes`；漏网的 `crate::find_admin_by_user_id`(login)→`crate::common::...` 已补。
- [x] CPMS_TECHNICAL.md 模块表补 common::response/types/admin/audit/encoding。
- [x] cargo fmt --check OK + clippy --all-targets 零警告 + test 32 passed。
- [x] 残留扫描：无 `crate::decode_bytes`/`crate::write_audit`/`crate::find_admin`/`crate::ApiError`/`crate::Archive` 顶层引用；main.rs 无被移定义。

## 完成记录

- 2026-06-06：创建任务卡，开始执行。
- 2026-06-06：执行完成。main.rs 的共享 DTO(`AdminUser`/`Archive`/`ApiResponse`/`ApiError`)与 helper(`ok`/`err`/`cpms_error_code`/`find_admin_*`/`write_audit`/`decode_bytes`)全部抽进 `common/` 五个子模块，`common/mod.rs` 再导出使消费方统一 `use crate::common::{...}`，与前端从 `common/` 导入一致。`AdminUser`/`Archive` 字段标 `pub(crate)`（脱离 crate 根后子模块可见）。AppState/MIGRATOR/main/health/middleware 按决策留 main（装配根，无前端对应）。main.rs 497→224 行。纯结构搬迁 0 行为变化；fmt/clippy/test 全绿（32 passed）。
