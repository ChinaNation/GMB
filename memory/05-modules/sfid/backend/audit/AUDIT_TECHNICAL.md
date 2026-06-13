# AUDIT 模块技术文档

- 最后更新:2026-06-13
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-store.md`
  - `memory/08-tasks/done/20260604-sfid-core-number-store-refactor.md`
  - `memory/08-tasks/open/20260613-sfid-institution-list-audit-accounts.md`

## 1. 模块定位

- 路径:`sfid/backend/audit.rs`
- 职责:承载后台审计日志查询 handler,为机构详情操作记录提供精确目标查询。
- 路由:`GET /api/v1/admin/audit-logs`
- 机构操作记录查询参数:`target_sfid=<机构身份ID>&limit=1000`;该参数必须走精确等值过滤,
  不得退回关键字模糊搜索。

## 2. 边界

- 审计日志查询是后台独立能力,不属于 `scope` 权限范围规则。
- 审计数据结构 `AuditLogEntry` 归 `audit.rs`;审计写入统一走 `core/runtime_ops.rs`
  的 `append_audit_log`,持久化目标为 `audit` 表。
- 机构详情操作记录覆盖机构创建、详情编辑、账户创建/删除、资料上传/下载/删除和
  CPMS 安装码吊销。新增机构相关写操作时,必须同步写入带 `target_sfid` 的审计记录。
- 新增审计写入 helper 如跨模块复用,优先放 `core` 或专门 audit 模块,不得塞进 `scope`。

## 3. 目录规则

```text
sfid/backend/audit.rs
  # 审计日志查询 handler
```
