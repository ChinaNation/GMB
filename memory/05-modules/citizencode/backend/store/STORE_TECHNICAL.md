# store/ — Store 聚合体与省级分片缓存

- 最后更新:2026-06-04
- 任务卡:
  - `memory/08-tasks/done/20260410-cid-store-shard-by-province.md`
  - `memory/08-tasks/done/20260525-cid-cpms-store.md`
  - `memory/08-tasks/done/20260604-cid-core-number-store-refactor.md`

## 定位

- 路径:`citizencode/backend/store/`
- 职责:
  - `model.rs`:进程内 `Store` 聚合体、敏感种子、服务指标、链请求回执、公民奖励状态、投票验证缓存。
  - `mod.rs / shard_types.rs / backend.rs`:按省分片的进程内缓存访问 API。
- 非职责:
  - 不承载管理员权限模型,管理员模型归 `admins/model.rs`。
  - 不承载管理员 Passkey 和安全挑战模型,这些归 `admins/security_model.rs`。
  - 不承载审计日志行模型,审计模型归 `audit.rs`。
  - 不恢复 `models/` facade。

## 当前结构

```text
citizencode/backend/store/
├── mod.rs          # ShardedStore 访问 API + store 模块导出
├── backend.rs      # ShardBackend 抽象与内存实现
├── shard_types.rs  # StoreShard / GlobalShard
└── model.rs        # Store 聚合体与运行时状态模型
```

## 数据边界

- `Store` 是进程内短锁聚合体,不是一张数据库大表。
- PostgreSQL 持久化按模块快照表和目标行表拆分:
  - `store_citizens`
  - `store_cpms`
  - `store_subjects`
  - `store_ops`
  - `ids / subjects / citizens / gov / private / accounts / docs / audit`
- `ShardedStore` 是按省读取和写入的进程内缓存,用于避免联邦注册局机构管理员/市注册局机构管理员查询时扫描全量内存。
- 省级分片 key 使用省名;数据库真实过滤必须先落到 `province_code / city_code`。

## 引用规则

- Store 聚合体类型统一通过 `crate::store::Store` 或 `crate::store::model::Store` 引用。
- 省级分片类型统一通过 `crate::store::ShardedStore / StoreShard / GlobalShard` 引用。
- 新增业务 DTO 必须归属对应业务模块,不得塞入 `store/model.rs`。
- 跨模块但有明确领域归属的模型必须归属领域模块:
  - HTTP 响应包装:`core/response.rs`
  - 管理员模型:`admins/model.rs`
  - 管理员安全模型:`admins/security_model.rs`
  - 审计日志行:`audit.rs`

## 验收口径

```text
test -d citizencode/backend/store
test ! -d citizencode/backend/store_shards
test ! -d citizencode/backend/models
rg "crate::models|store_shards|mod models" citizencode/backend -g '*.rs'
cd citizencode/backend && cargo check
```
