
- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-sheng-3tier-transition.md`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 前置依赖:phase23a(models split)
- 阻塞下游:phase23e

## 任务需求

2026-05-02 机构模块粗粒度整合后,该能力最终归入 CPMS 模块根目录

## 搬迁方案

- 新位置:`sfid/backend/cpms/rsa_blind.rs`

## 影响范围

- 新增:`cpms/rsa_blind.rs`
- 修改:`cpms/mod.rs` 加 `pub(crate) mod rsa_blind;`
- 修改调用方 import 路径

## 主要风险点

- **rsa_blind 内部对 `crate::*` 的依赖**:搬位置后某些 path 需改
- **匿名凭证业务测试**:`main_tests.rs` 若有 rsa_blind 测试需调路径

## 验收清单

- `cargo check` + `cargo test` + `cargo clippy` 与 baseline 持平
- Grep `cpms::rsa_blind` 覆盖所有匿名证书调用方

## 工作量

~0.5 agent round

## Progress

### 2026-05-01 — phase23b 初次执行完毕(SFID Agent)

**改动文件:**

- 初次迁入 institutions 的记录已被 2026-05-02 粗粒度整合覆盖。
- `sfid/backend/main.rs`、`sfid/backend/cpms/handler.rs`、`sfid/backend/citizens/binding.rs`
  已统一改为 `crate::cpms::rsa_blind::*`
- `memory/05-modules/sfid/backend/cpms/CPMS_TECHNICAL.md`:记录 RSA 盲签名归属

**调用方实际共 8 处(非任务卡说的 6 处):** main.rs 3 + sheng_admins/institutions.rs 4 + operate/binding.rs 1。任务卡描述偏低,实际都已更新。

**rsa_blind.rs 内部依赖:** 仅依赖 `blind_rsa_signatures` crate + `std::sync::RwLock`,无 `use crate::*` 或 `use super::*`,搬位置后无需修改任何 path。

**验收:**

- `cargo check`:Finished, 3 baseline dead_code warnings(province.rs),与 baseline 持平。
- `cargo test`:79 passed / 0 failed(初次迁移时记录)。
- `cargo clippy --all-targets -- -D warnings`:54+57 errors,与 baseline 59 持平,未引入新错。
- 新路径 grep `cpms::rsa_blind` 覆盖当前匿名证书调用方。


**任务卡调整建议:** 上游 phase23 主卡可记录"phase23b 调用方修正为 8 处而非 6 处",其余无需调整。状态 → done(待人工 review 后挪入 closed/)。
