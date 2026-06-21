# CID Store 按省分片铁律

任务卡 `20260410-cid-store-shard-by-province` 完成后的强制约定:

## 新数据路径

- `multisig_institutions` / `multisig_accounts` / `cpms_site_keys` 三个字段的 HTTP handler 层 **全部走 `state.sharded_store`(ShardedStore)**
- 读:用 `sharded_store.read_province(&province, |shard| {...}).await`
- 写:用 `sharded_store.write_province(&province, |shard| {...}).await`
- handler 不得为了省市列表查询扫描 `state.store` 全量数据;需要持久化主数据时,必须走目标分区表或模块 Store 表,然后同步更新 `sharded_store`。

## 省份定位

- 从 `ctx.admin_province`(handler 已有的 AdminAuthContext)
- 或从 cid_number 字符串解析:第二段省代码(如 `AH`)→ `province_name_by_code`

## Store 与分片缓存边界

- `store_citizens / store_cpms / store_subjects / store_ops` 是模块 Store 快照表。
- `ids / subjects / citizens / gov / private / accounts / docs / audit` 是按 `province_code` 分区的目标行表。
- `sharded_store` 是运行期按省检索缓存,不把分片整包写入 PostgreSQL。
- `core/runtime_ops.rs` 只负责启动期同步和确定性目录对账,不得在 GET 列表接口里做 backfill 或写库。

## Postgres 存储

- 不再建立 `store_shards` 表。
- 不再从 `runtime_cache_entries` 做分片迁移。
- 不再保留 `CID_SHARD_*` 双写/迁移环境变量。

## 后续待清理

- 继续把联邦/市管理员高频查询改成目标分区表精确查询或 `sharded_store` 单省读取。
- 保持 `gov/private/accounts/docs/subjects` 主写入走目标表,不得退回内存全量过滤。
- 新增 Store 字段时优先归入对应业务模块模型,不得恢复 `models/` 或 `store_shards/` 目录。
