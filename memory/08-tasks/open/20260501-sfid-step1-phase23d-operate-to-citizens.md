# SFID Step 1 / Phase 23d:`operate/` 迁入 `citizens/` + 删 `operate/`

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend/{operate,citizens}/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-delete-key-admin-and-sheng-3tier.md`
- 前置依赖:phase23a
- 阻塞下游:phase23e

## 任务需求

`sfid/backend/operate/{binding.rs, cpms_qr.rs, status.rs, mod.rs}` 实际是公民身份业务,本卡迁入新建的 `citizens/` 目录,删 `operate/`。

## 搬迁方案

| 原 | 新 | 说明 |
|---|---|---|
| `operate/binding.rs` | `citizens/binding.rs` | 公民身份绑定凭证 |
| `operate/status.rs` | `citizens/status.rs`(或并入 `citizens/binding.rs`) | 公民状态查询 |
| `operate/cpms_qr.rs` | `citizens/cpms_qr.rs` | **边界讨论**:cpms_qr 涉及 CPMS 站点扫码,业务上接近 citizens(用户身份扫码)。**本卡先 conservatively 迁入 citizens/,phase7 可再评是否拆 `cpms/` 子模块** |
| `operate/mod.rs` | 删除 | |
| 新建 `citizens/{mod,handler,vote}.rs` | (handler/vote 留空骨架,后续 Phase 业务补) | 仅注册 mod |

## 影响范围

- 新增:`citizens/{mod,binding,status,cpms_qr,handler,vote}.rs`
- 修改:`main.rs` `mod operate;` → `mod citizens;`,所有 `use operate::` → `use citizens::`
- 修改:调用方 import 路径(grep 全量替换)
- 删除:`src/operate/`(整目录)

## 主要风险点

- **`bootstrap_sheng_signer` 在 binding 路径中调用?**:phase23 progress 显示 `login/mod.rs` 才是 bootstrap 调用方,operate 只用 signing pair。验证 binding/status 不直接依赖 KEY_ADMIN
- **cpms_qr 归属**:本卡先按 citizens 迁入,边界争议留 phase7

## 验收清单

- `cargo check` + `cargo test` + `cargo clippy` baseline 持平
- Grep `crate::operate::|src/operate/` 零结果
- 整目录 `operate/` 物理删除

## 工作量

~1 agent round

## Progress

### 2026-05-01 phase23d 子卡完工

**搬迁动作**:

- `git mv sfid/backend/operate/{binding,cpms_qr,status}.rs` → `sfid/backend/citizens/`
- 删 `sfid/backend/operate/mod.rs` + `rmdir operate/`
- 新建 `sfid/backend/citizens/{mod,handler,vote}.rs`(handler/vote 空骨架 + `//!` 中文用途说明)
- 三个迁入文件顶部补 `//!` 模块级文档注释

**调用方更新**(13 处):

- `main.rs:32` `mod operate;` → `mod citizens;`(同 mod 列字母序就位)
- `main.rs:1005-1089` 7 处路由 `operate::binding::*` → `citizens::binding::*`
- `main.rs:1065` 注释 "indexer / operate 模块" → "indexer / citizens 模块"
- `shi_admins/mod.rs:10` `crate::operate::status` → `crate::citizens::status`
- `citizens/status.rs:99` 自引用 `crate::operate::cpms_qr` → `crate::citizens::cpms_qr`
- `chain/citizen_binding/mod.rs:13` 注释 `crate::operate::binding` → `crate::citizens::binding`
- `sfid/generator.rs:7` 注释 `operate::binding` → `citizens::binding`(去掉过渡说明)
- `main_tests.rs:201,204` 注释 `operate::binding` → `citizens::binding`(2 处)

**文档同步**:

- `git mv memory/05-modules/sfid/backend/operate/OPERATE_TECHNICAL.md` →
  `memory/05-modules/sfid/backend/citizens/CITIZENS_TECHNICAL.md` 并改写正文(模块定位/路由表/审计事件全部对齐 phase23d 后形态)
- `shi_admins/SHI_ADMINS_TECHNICAL.md`:`operate::status` → `citizens::status`(2 处)
- `sheng_admins/SHENG_ADMINS_TECHNICAL.md`:`backend/src/operate/status.rs` → `backend/src/citizens/status.rs`
- `business/BUSINESS_TECHNICAL.md`:"操作业务在 backend/src/operate" → "公民身份业务在 backend/src/citizens"
- `models/MODELS_TECHNICAL.md`:列出业务模块时 `operate(操作业务)` → `citizens(公民身份业务)`
- `sfid/SFID_TECHNICAL.md`:铁律说明里 `operate` → `citizens`

**验收**:

- `cargo check` 全绿(3 baseline province dead_code warnings)
- `cargo test` **79 passed / 0 failed / 0 ignored**
- `cargo clippy --all-targets -- -D warnings` 59 errors = baseline(零新增)
- `grep -rn "crate::operate\|operate::\|src/operate/" sfid/backend/` = **0 hit**
- `grep -rn "crate::citizens\|citizens::" sfid/backend/` = **14 hit**(≥8 阈值)
- `ls sfid/backend/operate/` = 不存在
- `citizens/{handler,vote}.rs` 空骨架 + `//!` 中文用途说明

**后续提醒**:

- handler/vote 拆分留给后续 Phase(任务卡里已说明,phase23d 不在范围)
- cpms_qr 边界讨论(是否拆 `cpms/` 子模块)留 phase7 评估
- phase23e 可继续拆 `key-admins/`、`chain/key_admins/`、`chain/balance/`
