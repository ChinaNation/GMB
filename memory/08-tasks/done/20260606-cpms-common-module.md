# 任务卡：CPMS 后端横切工具收敛到 common/（对齐前端）

## 任务需求

后端共享工具散在顶层（`ss58.rs`、`rate_limit.rs`），前端却已有 `common/` 共享家。建后端 `common/` 镜像前端 `common/`，把 ss58、rate_limit 收进去，让前后端各有一个 `common/` 对称。main.rs 的共享 DTO 本轮不动。

## 背景结论（分析已确认）

- ss58(42 行,3 模块用)、rate_limit(73 行,5 模块用)是横切工具，非单消费方，不能像 china 折进某功能，只能归共享命名空间。
- main.rs 是强制 bin 入口，不能收敛；本轮不拆它的 DTO/helper。
- 前端已全目录化且有 `common/`；china 收敛是纯后端、无前端对应。用户选：建后端 `common/`（命名对齐前端）只移 ss58+rate_limit。
- 关键：消费方多为 `use crate::{... rate_limit, ss58 ...}`，改 `common::rate_limit`/`common::ss58` 后调用点 `rate_limit::check`/`ss58::foo` 不变。
- `rate_limit.rs` 自带 `use crate::{err, ApiError, AppState}`（crate 根绝对路径），移动后不变；ss58.rs 无 crate 依赖。

## 影响范围

- `cpms/backend/ss58.rs` → `common/ss58.rs`；`rate_limit.rs` → `common/rate_limit.rs`（git mv）。
- 新建 `cpms/backend/common/mod.rs`：`pub mod rate_limit; pub mod ss58;`。
- `main.rs`：`mod rate_limit;`+`mod ss58;` → `mod common;`；`rate_limit::RateLimiter` → `common::rate_limit::RateLimiter`（2 处）。
- `initialize/mod.rs`：use 组 `rate_limit`→`common::rate_limit`；`crate::ss58::`→`crate::common::ss58::`。
- `dangan/materials.rs`：use 组 `rate_limit`→`common::rate_limit`。
- `dangan/routes.rs`：use 组 `rate_limit, ss58`→`common::rate_limit, common::ss58`。
- `super_admin/mod.rs`：use 组 `ss58`→`common::ss58`。
- `login/mod.rs`：use 组 `rate_limit`→`common::rate_limit`。
- `CPMS_TECHNICAL.md`：模块表 ss58/rate_limit 行改 common/。

## 主要风险点

- 纯结构搬迁、0 行为变化；引用改名靠 `cargo check` 全覆盖兜底。
- 不动 main.rs 的 DTO，避免大面积 `crate::ApiResponse` 改名。

## 是否需要先沟通

- 幅度与命名已确认：建 common/ 只移 ss58+rate_limit，命名 common 对齐前端。

## 执行清单

- [x] git mv ss58.rs/rate_limit.rs 入 common/（git 识别 rename 保历史），建 `common/mod.rs`（`pub mod rate_limit; pub mod ss58;`）。
- [x] main.rs：`mod rate_limit;`+`mod ss58;` → `mod common;`；2 处 `RateLimiter` 改 `common::rate_limit::`。
- [x] 5 个消费方 use 组/限定路径更新（initialize/dangan-materials/dangan-routes/super_admin/login + initialize 的 `crate::common::ss58::`）；调用点 `rate_limit::check`/`ss58::foo` 不变。
- [x] CPMS_TECHNICAL.md 模块表加 common / common::rate_limit / common::ss58。
- [x] cargo fmt --check OK + clippy --all-targets 零警告 + test 32 passed。
- [x] 残留扫描：顶层无 `mod ss58;`/`mod rate_limit;`、无 `crate::ss58`/`crate::rate_limit`（唯一命中是 common/mod.rs 的 `pub mod` 声明）。

## 完成记录

- 2026-06-06：创建任务卡，开始执行。
- 2026-06-06：执行完成。ss58.rs + rate_limit.rs 收进 `cpms/backend/common/`（对齐前端 `common/`），main.rs 顶层两个 `mod` 合为 `mod common;`。纯结构搬迁、0 行为变化；`use crate::{... rate_limit, ss58}` 改 `common::rate_limit`/`common::ss58` 后调用点零改。main.rs 的共享 DTO 本轮按确认未动。fmt/clippy/test 全绿（32 passed）。
