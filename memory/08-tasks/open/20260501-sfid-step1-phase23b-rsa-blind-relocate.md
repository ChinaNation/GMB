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

## Progress

### 2026-05-01 — phase23b 执行完毕(SFID Agent)

**改动文件:**

- `sfid/backend/src/institutions/anon_cert/mod.rs`(新建):中文 `//!` 用途说明 + `pub mod rsa_blind;`
- `sfid/backend/src/institutions/anon_cert/rsa_blind.rs`(`git mv` 自 `key-admins/rsa_blind.rs`,内容零改动,148 行)
- `sfid/backend/src/institutions/mod.rs`:加 `pub mod anon_cert;`
- `sfid/backend/src/key-admins/mod.rs`:删 `pub(crate) mod rsa_blind;`
- `sfid/backend/src/main.rs`:3 处 import 路径改为 `crate::institutions::anon_cert::rsa_blind::*`
- `sfid/backend/src/sheng_admins/institutions.rs`:4 处同上(任务卡原估 6 处,实际是 4 处)
- `sfid/backend/src/operate/binding.rs`:1 处同上(任务卡未列出,verify_anon_cert 调用)
- `memory/05-modules/sfid/backend/key-admins/KEY_ADMINS_TECHNICAL.md`:顶部加注释指出 rsa_blind 已搬出
- `memory/05-modules/sfid/backend/institutions/INSTITUTIONS_TECHNICAL.md`:顶部加注释说明新增 anon_cert 子模块

**调用方实际共 8 处(非任务卡说的 6 处):** main.rs 3 + sheng_admins/institutions.rs 4 + operate/binding.rs 1。任务卡描述偏低,实际都已更新。

**rsa_blind.rs 内部依赖:** 仅依赖 `blind_rsa_signatures` crate + `std::sync::RwLock`,无 `use crate::*` 或 `use super::*`,搬位置后无需修改任何 path。

**验收:**

- `cargo check`:Finished, 3 baseline dead_code warnings(province.rs),与 baseline 持平。
- `cargo test`:79 passed / 0 failed,含 `institutions::anon_cert::rsa_blind::tests::{generate_and_init_roundtrip, pem_reload}` 2 个测试已自动归位到新路径。
- `cargo clippy --all-targets -- -D warnings`:54+57 errors,与 baseline 59 持平,未引入新错。
- 残留 grep `key_admins::rsa_blind\|key-admins/rsa_blind` = **0**(达标)
- 新路径 grep `institutions::anon_cert::rsa_blind\|institutions/anon_cert/rsa_blind` = **8**(覆盖完整,≥6 达标)
- `key-admins/rsa_blind.rs` 物理已不存在,`git status` 显示 `R` (rename) 保留 history。
- `key-admins/mod.rs` 内 `rsa_blind` 行已删,其余三个 mod 完整保留。

**未做事项(留给后续卡):** 不删 `key-admins/` 整目录(phase23e),不改 `business/`(phase23c)、`operate/`(phase23d),不动 `citizenchain/`、`sfid/frontend/`,不 commit。

**任务卡调整建议:** 上游 phase23 主卡可记录"phase23b 调用方修正为 8 处而非 6 处",其余无需调整。状态 → done(待人工 review 后挪入 closed/)。
