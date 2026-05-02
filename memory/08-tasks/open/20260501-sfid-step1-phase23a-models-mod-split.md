# SFID Step 1 / Phase 23a:`models/mod.rs` 1021 行拆 6 文件

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend/src/models/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-delete-key-admin-and-sheng-3tier.md`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
- 前置依赖:Phase 1(完成)+ Phase 3 增量基础设施(完成,见 phase23 progress)
- 阻塞下游:phase23b/c/d/e

## 任务需求

`sfid/backend/src/models/mod.rs` 当前 1021 行,把 5 类(role / slot / session / permission / error)定义下沉到独立子文件,`mod.rs` 保留为 re-export facade,所有 `pub use models::*` 调用方零感知。**纯重构,业务行为零变化**。

## 拆分方案

| 子文件 | 内容 |
|---|---|
| `models/mod.rs` | 仅 `pub mod role; pub use role::*;` 等 re-export + 公共顶层注释 |
| `models/role.rs` | `AdminRole`(暂保 KeyAdmin,phase23e 删)+ `AdminStatus` + Display/parse |
| `models/slot.rs` | `Slot { Main, Backup1, Backup2 }`(若 phase23 progress 已加在 `sfid/province.rs`,搬过来或 re-export) |
| `models/session.rs` | `SessionContext` / 登录态 DTO / `LoginPayload` 等 |
| `models/permission.rs` | 权限决策类型(若 mod.rs 中无,可空文件 + 注释占位) |
| `models/error.rs` | 业务错误类型 / `ApiError` |
| `models/store.rs` | 若 mod.rs 含 Store 相关 DTO(`AdminEntry` / `InstitutionMeta` 等),按需要单文件 |

实际拆分:**先 read `models/mod.rs` 全文**,按类型语义归类,再批量 split。`pub use` facade 必须保持,确保 `crate::models::AdminRole` 等路径不变。

## 影响范围

- 仅 `sfid/backend/src/models/`
- `main.rs:53 pub(crate) use models::*;` 不变
- 其他模块零感知

## 主要风险点

- **类型互引用**:Slot 可能依赖 `province::ProvinceCode`,split 后 import 路径需更新
- **`#[derive]` 宏完整性**:确保每个类型连同 derive 一起搬
- **doc 注释**:类型上方的中文 `//!` 注释一并迁移
- **`pub` 可见性**:`pub` / `pub(crate)` 保持原级别

## 验收清单

- `cd sfid/backend && cargo check` 全绿
- `cargo test` 79 passed / 0 failed(同 baseline)
- `cargo clippy --all-targets -- -D warnings` 与 baseline 一致(不引入新错)
- `models/mod.rs` ≤ 80 行(只剩 facade)
- 每个 sub-file 顶部 1-3 行中文 `//!` 模块注释
- Grep `crate::models::` 调用方零变化(必要时 sed 校验)

## 工作量

~0.5 agent round(纯机械)
