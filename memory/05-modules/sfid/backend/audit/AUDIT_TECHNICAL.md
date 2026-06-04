# AUDIT 模块技术文档

- 最后更新:2026-06-04
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-store.md`
  - `memory/08-tasks/done/20260604-sfid-core-number-store-refactor.md`

## 1. 模块定位

- 路径:`sfid/backend/audit.rs`
- 职责:承载后台审计日志查询 handler。
- 路由:`GET /api/v1/admin/audit-logs`

## 2. 边界

- 审计日志查询是后台独立能力,不属于 `scope` 权限范围规则。
- 审计数据结构 `AuditLogEntry` 归 `audit.rs`;审计记录持久化归 `store_ops` 模块快照表。
- 新增审计写入 helper 如跨模块复用,优先放 `core` 或专门 audit 模块,不得塞进 `scope`。

## 3. 目录规则

```text
sfid/backend/audit.rs
  # 审计日志查询 handler
```
