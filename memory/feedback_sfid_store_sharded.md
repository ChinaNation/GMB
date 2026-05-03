# SFID Store 按省分片铁律

任务卡 `20260410-sfid-store-shard-by-province` 完成后的强制约定:

## 新数据路径

- `multisig_institutions` / `multisig_accounts` / `cpms_site_keys` 三个字段的 HTTP handler 层 **全部走 `state.sharded_store`(ShardedStore)**
- 读:用 `sharded_store.read_province(&province, |shard| {...}).await`
- 写:用 `sharded_store.write_province(&province, |shard| {...}).await`
- **禁止**在 handler 层直接走 `state.store.read()` / `state.store.write()` 访问这三个字段

## 省份定位

- 从 `ctx.admin_province`(handler 已有的 AdminAuthContext)
- 或从 sfid_id 字符串解析:第二段省代码(如 `AH`)→ `province_name_by_code`

## legacy store 保留路径(过渡期)

以下路径继续走 legacy `state.store`,不是 bug:
- `institutions/service.rs` reconcile 函数(`&mut Store` 签名)
- `institutions/store.rs` helpers(被 service.rs 复用)
- `app_core/runtime_ops.rs` 启动迁移 + backfill + orphan cleanup
- 审计日志 `append_audit_log`(继续用 legacy store 短锁)

## Postgres 存储

- 新表 `store_shards(shard_key TEXT PK, payload JSONB, updated_at, version)`
- 省分片 shard_key = 省名 UTF-8;全局分片 shard_key = "global"
- 启动时自动迁移(幂等):检测 `store_shards` 表为空则从 `runtime_cache_entries` 拆分灌入

## 环境变量

- `SFID_SHARD_SINGLE_WRITE`:默认 false(双写过渡期),设 true 切单写
- `SFID_SHARD_PRELOAD_ALL`:默认 true,设 false 跳过启动预加载(全部走懒加载)
- `SFID_SHARD_MIGRATION_SKIP`:默认 false,设 true 跳过迁移(紧急情况)

## 后续待清理

- 把 service.rs reconcile 也改走 sharded_store(需要重构 reconcile_public_security_for_province)
- 把 store.rs helpers 删除或改签名
- 把 runtime_ops.rs 启动路径切到 sharded_store
- 切断双写:设 `SFID_SHARD_SINGLE_WRITE=true`
- 可选:删除 `runtime_cache_entries` 表里的 `sfid_store` 行(冷归档后)
