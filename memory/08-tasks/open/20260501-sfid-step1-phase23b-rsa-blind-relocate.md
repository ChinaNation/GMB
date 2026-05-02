# SFID Step 1 / Phase 23b:`key_admins::rsa_blind` 搬到 `institutions/anon_cert/`

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend/src/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-delete-key-admin-and-sheng-3tier.md`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
- 前置依赖:phase23a(models split)
- 阻塞下游:phase23e

## 任务需求

`key-admins/rsa_blind.rs`(148 行)实现 RSA 盲签匿名凭证,被 `sheng_admins/institutions.rs` 6 处直接调用——**这跟 KEY_ADMIN 角色无关**,只是历史上代码放错位置。本卡把 rsa_blind 搬到正确位置,从 KEY_ADMIN 删除路径上摘出。

## 搬迁方案

- 新位置:`sfid/backend/src/institutions/anon_cert/`(新目录)
  - `mod.rs` re-export
  - `rsa_blind.rs`(从 `key-admins/` 整文件迁移,内容不动)
- 调用方更新:`sheng_admins/institutions.rs` 把 `use crate::key_admins::rsa_blind` → `use crate::institutions::anon_cert::rsa_blind`
- `key-admins/mod.rs` 删除 `pub mod rsa_blind;` 行(暂保留其他)

## 影响范围

- 新增:`institutions/anon_cert/{mod,rsa_blind}.rs`
- 修改:`institutions/mod.rs` 加 `pub mod anon_cert;`
- 修改:`key-admins/mod.rs` 去 `pub mod rsa_blind;`
- 修改:`sheng_admins/institutions.rs` 6 处 import 路径
- 删除:`key-admins/rsa_blind.rs`(物理 git mv 到新位置)

## 主要风险点

- **rsa_blind 内部对 `crate::*` 的依赖**:搬位置后某些 path 需改
- **匿名凭证业务测试**:`main_tests.rs` 若有 rsa_blind 测试需调路径

## 验收清单

- `cargo check` + `cargo test` + `cargo clippy` 与 baseline 持平
- Grep `key_admins::rsa_blind` 零结果
- Grep `institutions::anon_cert::rsa_blind` ≥ 6(迁移完整)

## 工作量

~0.5 agent round
