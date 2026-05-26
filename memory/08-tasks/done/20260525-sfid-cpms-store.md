# 任务卡：SFID/CPMS Store 架构重构

- 任务编号：20260525-161949
- 状态：done
- 所属模块：sfid-cpms
- 当前负责人：Codex
- 创建时间：2026-05-25 16:19:49

## 任务需求

重构 SFID/CPMS Store 架构：SFID 删除旧整包 JSON Store、旧 runtime cache 和持久化分片表，改为 PostgreSQL 模块快照表 + 进程内分片缓存；短期 challenge/session 进入对应模块快照，确保跨请求可读；CPMS 保持 DB-first 并整理轻量 store 边界；不迁移旧数据，可清空旧数据；命名精简、结构清晰、方便扩展；完成后更新文档、完善中文注释、清理残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/sfid/backend/BACKEND_LAYOUT.md
- memory/05-modules/sfid/backend/models/MODELS_TECHNICAL.md
- memory/05-modules/sfid/backend/login/LOGIN_TECHNICAL.md
- memory/05-modules/sfid/backend/citizens/CITIZENS_TECHNICAL.md
- memory/05-modules/sfid/backend/cpms/CPMS_TECHNICAL.md
- memory/05-modules/cpms/backend/login/LOGIN_TECHNICAL.md
- memory/05-modules/cpms/backend/initialize/INITIALIZE_TECHNICAL.md
- memory/05-modules/cpms/backend/dangan/DANGAN_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 2026-05-25：按用户确认，本任务不做旧数据迁移，不做旧 Store 兼容；旧短期状态和旧 JSON Store 数据允许直接删除。
- 2026-05-25：SFID 删除旧 `runtime_store / runtime_misc / runtime_cache_entries / store_shards` 持久化路径，新增 `store_citizens / store_cpms / store_institutions / store_ops` 模块快照。
- 2026-05-25：`store_shards` Rust 模块收敛为进程内分片缓存，删除旧 Postgres 分片后端和迁移辅助。
- 2026-05-25：CPMS 新增轻量 `StoreDb`，统一清理登录短期 PostgreSQL 状态。

## 完成信息

- 完成时间：2026-05-25 16:39:15
- 完成摘要：完成 SFID/CPMS Store 架构重构：SFID 旧 runtime JSON 与持久化分片清理，模块快照表落地，CPMS StoreDb 清理边界落地，文档和注释已更新。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
