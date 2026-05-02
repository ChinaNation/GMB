# SFID Step 1 / Phase 23c:`business/` 内容并入 `scope/` + 删 `business/`

- 状态:done(2026-05-02,与 phase23b 合并 commit 400dcdd 落地;Progress 详见末尾)
- 创建日期:2026-05-01
- 模块:`sfid/backend/{business,scope}/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-delete-key-admin-and-sheng-3tier.md`
- 前置依赖:phase23a
- 阻塞下游:phase23e

## 任务需求

`sfid/backend/business/`(5 文件:`audit.rs / mod.rs / pubkey.rs / query.rs / scope.rs`)与 `src/scope/`(3 文件:`mod / filter / rules`)职责重叠。`business::scope::in_scope_cpms_site` 在 `main.rs:43` 被引入。本卡按职责合并,删除 `business/`。

## 搬迁方案(按文件归位)

| 原 | 新 | 说明 |
|---|---|---|
| `business/scope.rs::in_scope_cpms_site` | `scope/cpms.rs`(新) | CPMS 站点 scope 过滤 |
| `business/audit.rs` | `scope/audit.rs`(新) | scope 决策审计 |
| `business/pubkey.rs` | `scope/pubkey.rs`(新)或 `crypto/pubkey.rs` 视内容 | pubkey 校验工具 |
| `business/query.rs` | `scope/query.rs`(新)或就近合并 `scope/filter.rs` | 查询过滤辅助 |
| `business/mod.rs` | 删除 | facade 不需要 |

具体归类先 read 5 文件确认语义。

## 影响范围

- 新增:`scope/{cpms,audit,pubkey,query}.rs`(按需)
- 修改:`scope/mod.rs` 加 sub-module 导出
- 修改:`main.rs:43` `use business::scope::in_scope_cpms_site` → `use scope::cpms::in_scope_cpms_site`
- 修改:`main.rs:25 mod business;` 删除
- 删除:`src/business/`(整目录)

## 主要风险点

- **`business::pubkey` 与 `crypto::sr25519`(若存在)重叠**:先确认有无重复实现
- **`audit.rs` 可能含跨切面 logging**:要保业务凭证签发链路审计不掉

## 验收清单

- `cargo check` + `cargo test` + `cargo clippy` baseline 持平
- Grep `crate::business::|business::|src/business/` 零结果
- 整目录 `business/` 物理删除

## 工作量

~0.5-1 agent round

## Progress

### 2026-05-02 — phase23b 越界 + phase23c 同步完成(合并 commit)

phase23b 派工时 SFID Agent 越界推进 phase23c。修复后两卡合并落地:

**phase23c 改动文件:**
- 新建 `sfid/backend/scope/{pubkey,admin_province,cpms,audit,query}.rs`(从 `business/{pubkey,scope::province_scope_for_role,scope::in_scope_cpms_site,audit,query}` 迁入)
- `scope/mod.rs` 加 `pub mod admin_province; pub mod audit; pub mod cpms; pub mod pubkey; pub mod query;` + Phase 23c 注释
- `main.rs`:删 `mod business;` + `use business::scope::in_scope_cpms_site;` → `use scope::cpms::in_scope_cpms_site;`;两处 audit/query 路由调用从 `business::` → `scope::`(public_identity_search、admin_list_audit_logs、admin_list_citizens 共 3 路由)
- `key-admins/mod.rs`:`business::pubkey::same_admin_pubkey` → `scope::pubkey::same_admin_pubkey`
- `login/mod.rs` / `sheng_admins/{catalog,operators}.rs` / `institutions/handler.rs`:`use crate::business::{pubkey, scope as adminscope}` → `use crate::scope::{pubkey, admin_province as adminscope}`(共多处,具体 grep `crate::scope::pubkey|crate::scope::admin_province` = 13 处)
- `git rm -r sfid/backend/business/`(整目录 5 文件删除)

**验收终态:**
- `cargo check`:全绿,3 baseline province dead_code 警告
- `cargo test`:**79 passed / 0 failed**
- `cargo clippy --all-targets -- -D warnings`:**59 errors,与 baseline 持平**
- `grep -rn "crate::business::|business::scope::|src/business/" sfid/backend/` = **0**(达标)
- `grep -rn "crate::scope::pubkey|crate::scope::audit|crate::scope::query|crate::scope::cpms|crate::scope::admin_province" sfid/backend/` = **13**

**与 phase23b 合并 commit:** business 整改与 rsa_blind 搬迁原本两张卡,因 SFID Agent 越界已混合,统一作为一个 commit 落地。

### 主入口补充(2026-05-02)

- 任务卡默认把 audit/query handler 归到 `scope/audit.rs` / `scope/query.rs`,语义上 handler 不属 scope 过滤入口,但本轮严格按任务卡字面字段对齐(用户铁律 feedback_no_scope_expansion.md)。如后续判定需 admin_handlers 顶层目录,留给 phase23d operate→citizens 整体评估
- `business/pubkey.rs` 入 `scope/pubkey.rs`(任务卡第一选项,因项目无独立 `crypto/` 模块);保留所有原 `#[allow(dead_code)]` 标注,clippy baseline 不污染
- AdminRole::KeyAdmin 分支保留:`scope::admin_province::province_scope_for_role` 仍 `match KeyAdmin => None`,phase23e KEY_ADMIN final removal 子卡再清理
