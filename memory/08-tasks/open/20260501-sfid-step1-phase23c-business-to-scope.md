# SFID Step 1 / Phase 23c:`business/` 内容并入 `scope/` + 删 `business/`

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend/src/{business,scope}/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-phase23-delete-key-admin-and-sheng-3tier.md`
- 前置依赖:phase23a
- 阻塞下游:phase23e

## 任务需求

`sfid/backend/src/business/`(5 文件:`audit.rs / mod.rs / pubkey.rs / query.rs / scope.rs`)与 `src/scope/`(3 文件:`mod / filter / rules`)职责重叠。`business::scope::in_scope_cpms_site` 在 `main.rs:43` 被引入。本卡按职责合并,删除 `business/`。

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
