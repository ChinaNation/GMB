# Phase 2 完整实施方案:Store 按省分片(v1 定稿)

- **任务 ID**: `20260410-sfid-store-shard-by-province`
- **版本**: v1 完整实施版
- **日期**: 2026-04-09
- **工作量**: 4 天(Day1 结构 + Day2 PG 后端 + Day3 handler 改造 + Day4 文档收官)
- **前置**: Phase 1 `sfid-sheng-admin-per-province-keyring` 已完成
- **架构上下文**: `memory/05-architecture/20260409-sfid-50k-concurrent-framework.md`
- **状态**: **已完成** (2026-04-09)

---

## 实施后实际状态总结

### 新建文件(5 个)

| 文件 | 说明 |
|---|---|
| `sfid/backend/src/store_shards/mod.rs` | 模块入口,StoreShard / GlobalShard / ShardedStore / ShardBackend trait 定义 + 8 单测 |
| `sfid/backend/src/store_shards/shard_types.rs` | StoreShard / GlobalShard 类型定义与序列化 |
| `sfid/backend/src/store_shards/backend.rs` | ShardBackend trait + ShardedStore 核心实现(DashMap 分片 + read/write/for_each_province) |
| `sfid/backend/src/store_shards/pg_backend.rs` | PostgresShardBackend 实现(store_shards 表 CRUD + 乐观并发 version) |
| `sfid/backend/src/store_shards/migration.rs` | 启动时幂等迁移:从 runtime_cache_entries 拆分灌入 store_shards 表 |

### 改动文件(7 个)

| 文件 | 改动内容 |
|---|---|
| `sfid/backend/src/main.rs` | AppState 接入 ShardedStore,启动时初始化 + 迁移 + 预加载 |
| `sfid/backend/Cargo.toml` | 新增依赖(dashmap 等) |
| `sfid/backend/src/institutions/handler.rs` | cpms_site_keys 读写迁移到 sharded_store |
| `sfid/backend/src/sheng-admins/institutions.rs` | multisig_institutions + multisig_accounts 读写迁移到 sharded_store |
| `sfid/backend/src/operate/status.rs` | 状态查询迁移到 sharded_store |
| `sfid/backend/src/app_core/runtime_ops.rs` | 启动路径保留 legacy store(迁移 + backfill + orphan cleanup) |
| `sfid/backend/src/main_tests.rs` | store_shards 8 个单测 |

### 迁移到 sharded_store 的三个字段

- `cpms_site_keys` -- institutions/handler.rs 9 处
- `multisig_institutions` -- sheng-admins/institutions.rs + operate/status.rs
- `multisig_accounts` -- sheng-admins/institutions.rs + operate/status.rs

### 有意保留走 legacy store 的路径(过渡期)

- `institutions/service.rs` -- reconcile 函数(`&mut Store` 签名,待后续重构)
- `institutions/store.rs` -- helpers(被 service.rs 复用)
- `app_core/runtime_ops.rs` -- 启动迁移 + backfill + orphan cleanup
- 审计日志 `append_audit_log` -- 继续用 legacy store 短锁

### 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `SFID_SHARD_SINGLE_WRITE` | false | 双写过渡期,设 true 切单写 |
| `SFID_SHARD_PRELOAD_ALL` | true | 设 false 跳过启动预加载(全部懒加载) |
| `SFID_SHARD_MIGRATION_SKIP` | false | 设 true 跳过迁移(紧急情况) |

### 编译与测试状态

- `cargo check` EXIT=0
- `cargo test store_shards` 8/8 绿

---

> 以下为 v1 方案设计原文,保留作为架构参考。

---

## 零、默认决策(已拍板)

所有 v1 草稿里的拍板项默认采用推荐组合:

| 决策 | 选择 |
|---|---|
| 分片维度 | **按 province 名**(UTF-8 字符串) |
| GlobalShard 形态 | **JSONB 统一存 store_shards 表** |
| 过渡期策略 | **双写 2 周 → 切单写** |
| 迁移时机 | **后端启动时自动检测 + 幂等迁移** |
| 压测目标 | **单省 100 并发 P99 <100ms + 10 省 500 并发 P99 <200ms** |
| 跨省查询 | **保留 `list_all_*` helpers,O(43) 遍历** |
| 并发控制 | **DashMap 分片级 Arc<RwLock<_>>,无分布式锁** |
| 持久化语义 | **写穿透 + 异步持久化,单分片事务** |

---

## 一、目标与范围

### 目标
- **解除后端 Store 全局锁 + 全量 JSON 反序列化瓶颈**
- 承载:Phase 1 的 ~500 并发 → **~5000 并发 SHI_ADMIN**
- 为 Phase 3(提交队列批处理)和 Phase 4(水平扩展)铺路

### 范围
- 只改 **sfid-backend 数据访问层**
- **不改**:链端代码、前端代码、HTTP 接口签名、业务逻辑语义
- **不引入**:Redis、分布式锁、PG 读副本、新中间件

---

## 二、现状分析

### 2.1 当前 Store 结构

`sfid/backend/src/models/mod.rs::BackendStore` 是一个 single big struct,包含:
- 43 省所有 citizen 记录
- 43 省所有机构/账户
- 全局 admin_users_by_pubkey(KEY + SHENG + SHI 混合)
- 全局 sessions / challenges / audit_log
- SFID 工具状态(sfid_meta / cities / 等)
- 链请求幂等 / RSA 密钥 / 等 20+ 个字段

### 2.2 当前持久化路径

```
AppState.store: Arc<RwLock<BackendStore>>
    ↕ serde_json
runtime_cache_entries(key='sfid_store', payload JSONB)  ← 1.2MB 大 JSON
                                                        ← 每请求 load 一次

+ admins(结构表,10 列)     ← load_store_postgres 后补充填充
+ operators(结构表)
+ audit_logs / chain_requests / ...(其他小表)
```

### 2.3 瓶颈点

1. **单锁**:所有 handler 通过 `state.store.read()` / `.write()` 排队
2. **全量 load**:`load_store_postgres()` 每次 SELECT 1.2MB JSON + 5 次结构表 SELECT,耗时 50~100ms
3. **全量 persist**:写入时 DELETE `runtime_cache_entries` WHERE key='sfid_store' + INSERT 全量 JSON

50K 并发 → 这条路径直接崩溃。

---

## 三、目标架构

### 3.1 AppState 新结构

```rust
pub(crate) struct AppState {
    // ── 保留的字段(Phase 1) ──
    pub(crate) pg_pool: PgPool,
    pub(crate) sheng_signer_cache: Arc<ShengSignerCache>,
    pub(crate) signing_seed_hex: Arc<RwLock<SensitiveSeed>>,
    pub(crate) signing_public_key_hex: Arc<RwLock<String>>,
    // ...其他辅助字段...

    // ── 新增(Phase 2) ──
    pub(crate) store_shards: Arc<ShardedStore>,

    // ── 过渡期保留(Phase 2 中)──
    // 2 周过渡期结束后删除
    pub(crate) legacy_store: Arc<RwLock<BackendStore>>,  // 双写兜底,仅写不读
}
```

### 3.2 ShardedStore 结构

```rust
// src/store_shards/mod.rs
pub(crate) struct ShardedStore {
    shards: DashMap<String, Arc<RwLock<StoreShard>>>,    // key = province 名
    global: Arc<RwLock<GlobalShard>>,
    backend: Arc<dyn ShardBackend>,
    double_write: bool,                                   // 过渡期双写开关
}
```

### 3.3 StoreShard(每省一份)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct StoreShard {
    pub(crate) province: String,

    // ── 本省管理员(ShengAdmin + ShiAdmin,按 province 分散)──
    // Key: admin_pubkey
    pub(crate) local_admins: HashMap<String, AdminUser>,

    // ── 本省机构(两层模型)──
    pub(crate) multisig_institutions: HashMap<String, MultisigInstitution>,
    pub(crate) multisig_accounts: HashMap<String, MultisigAccount>,

    // ── 本省 CPMS 站点 ──
    pub(crate) cpms_site_keys: HashMap<String, CpmsSiteKeys>,
    pub(crate) cpms_pending_registrations: HashMap<String, PendingCpmsRegistration>,

    // ── 本省 citizen 记录(未来最大数据量)──
    pub(crate) next_citizen_id: u64,
    pub(crate) citizen_records: HashMap<u64, CitizenRecord>,
    pub(crate) citizen_id_by_pubkey: HashMap<String, u64>,
    pub(crate) citizen_id_by_archive_no: HashMap<String, u64>,
    pub(crate) pubkey_by_archive_index: HashMap<String, String>,

    // ── 本省 citizen 绑定流程 ──
    pub(crate) citizen_bind_challenges: HashMap<String, CitizenBindChallenge>,
    pub(crate) pending_bind_scan_by_qr_id: HashMap<String, PendingBindScan>,

    // ── 本省档案导入 ──
    pub(crate) imported_archives: HashMap<String, ImportedArchive>,
    pub(crate) pending_status_by_archive_no: HashMap<String, CitizenStatus>,

    // ── 本省 SFID 生成历史 ──
    pub(crate) generated_sfid_by_pubkey: HashMap<String, String>,

    // ── 本省回调任务 ──
    pub(crate) bind_callback_jobs: Vec<BindCallbackJob>,

    // ── 本省奖励状态 ──
    pub(crate) reward_state_by_pubkey: HashMap<String, RewardStateRecord>,

    // ── 版本号(冲突检测)──
    pub(crate) version: u64,
}
```

### 3.4 GlobalShard(跨省共享)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GlobalShard {
    // KEY_ADMIN 密钥环
    pub(crate) chain_keyring_state: Option<ChainKeyringState>,
    pub(crate) keyring_rotate_challenges: HashMap<String, KeyringRotateChallenge>,

    // 全局管理员索引(KeyAdmin + ShengAdmin 本身,不含 ShiAdmin)
    // KeyAdmin 不归属任何省,ShengAdmin 的本体也在这里(登录路由需要快速查)
    // 注意:ShengAdmin 的详细字段(含 encrypted_signing_privkey)在 GlobalShard 也存一份
    // 省分片 local_admins 只包含 ShiAdmin
    pub(crate) global_admins: HashMap<String, AdminUser>,

    // 省份路由索引
    pub(crate) sheng_admin_province_by_pubkey: HashMap<String, String>,

    // 登录 challenge + session
    pub(crate) login_challenges: HashMap<String, LoginChallenge>,
    pub(crate) qr_login_results: HashMap<String, QrLoginResultRecord>,
    pub(crate) admin_sessions: HashMap<String, AdminSession>,

    // 全局幂等池
    pub(crate) consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    pub(crate) consumed_cpms_register_tokens: HashMap<String, DateTime<Utc>>,

    // 审计(大表,可能将来移到 ClickHouse)
    pub(crate) audit_logs: Vec<AuditLogEntry>,

    // 链请求幂等
    pub(crate) chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    pub(crate) chain_nonce_seen: HashMap<String, DateTime<Utc>>,

    // RSA 匿名证书密钥
    pub(crate) anon_rsa_private_key_pem: Option<String>,

    // 清理时间戳
    pub(crate) chain_auth_last_cleanup_at: Option<DateTime<Utc>>,
    pub(crate) pending_bind_last_cleanup_at: Option<DateTime<Utc>>,

    // 服务指标
    pub(crate) metrics: ServiceMetrics,

    // 版本号
    pub(crate) version: u64,
}
```

**关键设计**:`ShengAdmin` 本体在 `GlobalShard.global_admins`(登录路由需要),但每个 ShengAdmin 管理的 `StoreShard.local_admins` 只含 ShiAdmin。这样登录路由快(查全局),业务读写快(按省分片)。

### 3.5 ShardBackend trait

```rust
// src/store_shards/backend.rs
#[async_trait::async_trait]
pub(crate) trait ShardBackend: Send + Sync {
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String>;
    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String>;

    async fn load_global(&self) -> Result<GlobalShard, String>;
    async fn save_global(&self, global: &GlobalShard) -> Result<(), String>;

    /// 列出所有已持久化的 shard key(包括 "global"),用于迁移 / 启动遍历
    async fn list_shard_keys(&self) -> Result<Vec<String>, String>;
}
```

### 3.6 PostgresShardBackend 实现

```rust
// src/store_shards/pg_backend.rs
pub(crate) struct PostgresShardBackend {
    pool: PgPool,
}

#[async_trait::async_trait]
impl ShardBackend for PostgresShardBackend {
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String> {
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT payload FROM store_shards WHERE shard_key = $1"
        )
        .bind(province)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("load_shard sql: {e}"))?;

        match row {
            Some((json,)) => {
                let shard: StoreShard = serde_json::from_value(json)
                    .map_err(|e| format!("load_shard json: {e}"))?;
                Ok(Some(shard))
            }
            None => Ok(None),
        }
    }

    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String> {
        let payload = serde_json::to_value(shard)
            .map_err(|e| format!("save_shard json: {e}"))?;
        sqlx::query(
            "INSERT INTO store_shards (shard_key, payload, updated_at, version)
             VALUES ($1, $2, now(), $3)
             ON CONFLICT (shard_key) DO UPDATE SET
                 payload = EXCLUDED.payload,
                 updated_at = EXCLUDED.updated_at,
                 version = store_shards.version + 1"
        )
        .bind(province)
        .bind(payload)
        .bind(shard.version as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("save_shard sql: {e}"))?;
        Ok(())
    }

    async fn load_global(&self) -> Result<GlobalShard, String> {
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT payload FROM store_shards WHERE shard_key = 'global'"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("load_global sql: {e}"))?;

        match row {
            Some((json,)) => serde_json::from_value(json)
                .map_err(|e| format!("load_global json: {e}")),
            None => Ok(GlobalShard::default()),
        }
    }

    async fn save_global(&self, global: &GlobalShard) -> Result<(), String> {
        let payload = serde_json::to_value(global)
            .map_err(|e| format!("save_global json: {e}"))?;
        sqlx::query(
            "INSERT INTO store_shards (shard_key, payload, updated_at, version)
             VALUES ('global', $1, now(), $2)
             ON CONFLICT (shard_key) DO UPDATE SET
                 payload = EXCLUDED.payload,
                 updated_at = EXCLUDED.updated_at,
                 version = store_shards.version + 1"
        )
        .bind(payload)
        .bind(global.version as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("save_global sql: {e}"))?;
        Ok(())
    }

    async fn list_shard_keys(&self) -> Result<Vec<String>, String> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT shard_key FROM store_shards ORDER BY shard_key"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("list_shard_keys sql: {e}"))?;
        Ok(rows.into_iter().map(|(k,)| k).collect())
    }
}
```

### 3.7 Postgres Schema

```sql
-- 在 main.rs 启动迁移里追加(幂等 CREATE)
CREATE TABLE IF NOT EXISTS store_shards (
    shard_key TEXT PRIMARY KEY,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_store_shards_updated_at
    ON store_shards(updated_at);
```

---

## 四、访问 API

### 4.1 ShardedStore 核心方法

```rust
// src/store_shards/mod.rs
impl ShardedStore {
    pub(crate) fn new(backend: Arc<dyn ShardBackend>, double_write: bool) -> Self {
        Self {
            shards: DashMap::new(),
            global: Arc::new(RwLock::new(GlobalShard::default())),
            backend,
            double_write,
        }
    }

    /// 启动时一次性加载 GlobalShard(全局状态必须就绪)
    pub(crate) async fn bootstrap_global(&self) -> Result<(), String> {
        let global = self.backend.load_global().await?;
        *self.global.write().map_err(|_| "global poisoned")? = global;
        Ok(())
    }

    /// 启动时预加载所有省份分片(可选优化,也可留给懒加载)
    pub(crate) async fn preload_all_shards(&self) -> Result<usize, String> {
        let keys = self.backend.list_shard_keys().await?;
        let mut count = 0;
        for key in keys {
            if key == "global" {
                continue;
            }
            if let Some(shard) = self.backend.load_shard(&key).await? {
                self.shards.insert(key.clone(), Arc::new(RwLock::new(shard)));
                count += 1;
            }
        }
        Ok(count)
    }

    /// 读本省(懒加载)
    pub(crate) async fn read_province<F, R>(&self, province: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&StoreShard) -> R,
    {
        let shard = self.get_or_load_shard(province).await?;
        let guard = shard.read().map_err(|_| "shard poisoned")?;
        Ok(f(&*guard))
    }

    /// 写本省 + 写穿透
    pub(crate) async fn write_province<F, R>(&self, province: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut StoreShard) -> R,
    {
        let shard = self.get_or_load_shard(province).await?;
        let result = {
            let mut guard = shard.write().map_err(|_| "shard poisoned")?;
            guard.version += 1;
            f(&mut *guard)
        };
        // 写穿透:快照拷贝 → 异步持久化
        self.persist_shard(province).await?;
        Ok(result)
    }

    /// 读全局
    pub(crate) fn read_global<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&GlobalShard) -> R,
    {
        let guard = self.global.read().map_err(|_| "global poisoned")?;
        Ok(f(&*guard))
    }

    /// 写全局 + 写穿透
    pub(crate) async fn write_global<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut GlobalShard) -> R,
    {
        let result = {
            let mut guard = self.global.write().map_err(|_| "global poisoned")?;
            guard.version += 1;
            f(&mut *guard)
        };
        self.persist_global().await?;
        Ok(result)
    }

    /// 遍历所有已加载分片(跨省查询 helper)
    pub(crate) async fn for_each_province<F>(&self, mut f: F) -> Result<(), String>
    where
        F: FnMut(&str, &StoreShard),
    {
        // 先遍历已加载,再补齐未加载的
        let loaded_keys: Vec<String> = self.shards.iter()
            .map(|e| e.key().clone())
            .collect();
        for key in &loaded_keys {
            if let Some(arc) = self.shards.get(key) {
                let guard = arc.read().map_err(|_| "shard poisoned")?;
                f(key, &*guard);
            }
        }
        // 未加载的从 PG 加载一次
        let all_keys = self.backend.list_shard_keys().await?;
        for key in all_keys {
            if key == "global" || loaded_keys.contains(&key) {
                continue;
            }
            let shard = self.get_or_load_shard(&key).await?;
            let guard = shard.read().map_err(|_| "shard poisoned")?;
            f(&key, &*guard);
        }
        Ok(())
    }

    async fn get_or_load_shard(
        &self,
        province: &str,
    ) -> Result<Arc<RwLock<StoreShard>>, String> {
        if let Some(s) = self.shards.get(province) {
            return Ok(s.clone());
        }
        // 懒加载
        let loaded = self.backend.load_shard(province).await?
            .unwrap_or_else(|| StoreShard {
                province: province.to_string(),
                ..Default::default()
            });
        let arc = Arc::new(RwLock::new(loaded));
        // 并发安全:or_insert 只插入一次
        let entry = self.shards.entry(province.to_string()).or_insert(arc);
        Ok(entry.clone())
    }

    async fn persist_shard(&self, province: &str) -> Result<(), String> {
        let snapshot = {
            let arc = self.shards.get(province)
                .ok_or("shard not loaded")?
                .clone();
            let guard = arc.read().map_err(|_| "shard poisoned")?;
            guard.clone()
        };
        self.backend.save_shard(province, &snapshot).await
    }

    async fn persist_global(&self) -> Result<(), String> {
        let snapshot = {
            let guard = self.global.read().map_err(|_| "global poisoned")?;
            guard.clone()
        };
        self.backend.save_global(&snapshot).await
    }
}
```

### 4.2 双写过渡期(legacy 同步)

`write_province` 和 `write_global` 在过渡期 **同时** 更新 `legacy_store`:

```rust
// 伪代码:双写
if self.double_write {
    // 同时更新 AppState.legacy_store,触发老的 persist_store 路径
    crate::double_write::sync_to_legacy(province, &snapshot).await?;
}
```

过渡期 `SFID_SHARD_SINGLE_WRITE=false`(默认),完全切换后设置为 `true`,跳过 legacy 同步。

---

## 五、Handler 改造模式

### 5.1 按 province 路由

**旧代码**:
```rust
let store = state.store.read().map_err(|_| ...)?;
let institution = store.multisig_institutions.get(&sfid_id).cloned();
```

**新代码**:
```rust
// 从 ctx 拿 province
let province = ctx.admin_province.as_deref().ok_or(...)?;
let institution = state.store_shards.read_province(province, |shard| {
    shard.multisig_institutions.get(&sfid_id).cloned()
}).await?;
```

### 5.2 写路径

**旧代码**:
```rust
let mut store = state.store.write().map_err(|_| ...)?;
store.multisig_institutions.insert(sfid_id.clone(), inst);
drop(store);
persist_store(&state).await?;
```

**新代码**:
```rust
let province = ctx.admin_province.as_deref().ok_or(...)?;
state.store_shards.write_province(province, |shard| {
    shard.multisig_institutions.insert(sfid_id.clone(), inst);
}).await?;
// persist 已在 write_province 内部完成
```

### 5.3 全局读写

**旧代码**(登录验签):
```rust
let store = state.store.read()?;
let user = store.admin_users_by_pubkey.get(&pubkey).cloned();
```

**新代码**:
```rust
let user = state.store_shards.read_global(|g| {
    g.global_admins.get(&pubkey).cloned()
})?;
```

### 5.4 跨省查询(KEY_ADMIN 场景)

```rust
let mut all_institutions = Vec::new();
state.store_shards.for_each_province(|_province, shard| {
    all_institutions.extend(shard.multisig_institutions.values().cloned());
}).await?;
```

---

## 六、启动迁移逻辑

### 6.1 自动检测 + 幂等迁移

```rust
// src/store_shards/migration.rs
pub(crate) async fn migrate_legacy_store_if_needed(
    pool: &PgPool,
    legacy_store: &BackendStore,
) -> Result<(), String> {
    // 检查 store_shards 表是否为空
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM store_shards"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("count: {e}"))?;

    if count > 0 {
        tracing::info!("store_shards already populated, skip migration");
        return Ok(());
    }

    tracing::info!("migrating legacy store → sharded structure");
    let started = std::time::Instant::now();

    // 1. 拆分 43 省分片
    let mut shards_by_province: HashMap<String, StoreShard> = HashMap::new();

    // 1.1 按 province 分散 multisig_institutions
    for (sfid_id, inst) in &legacy_store.multisig_institutions {
        let shard = shards_by_province
            .entry(inst.province.clone())
            .or_insert_with(|| StoreShard {
                province: inst.province.clone(),
                ..Default::default()
            });
        shard.multisig_institutions.insert(sfid_id.clone(), inst.clone());
    }

    // 1.2 按 sfid_id 关联 multisig_accounts
    for (key, acc) in &legacy_store.multisig_accounts {
        // key 格式 "sfid_id|account_name"
        if let Some(sfid_id) = key.split('|').next() {
            if let Some(inst) = legacy_store.multisig_institutions.get(sfid_id) {
                let shard = shards_by_province.entry(inst.province.clone())
                    .or_insert_with(|| StoreShard {
                        province: inst.province.clone(),
                        ..Default::default()
                    });
                shard.multisig_accounts.insert(key.clone(), acc.clone());
            }
        }
    }

    // 1.3 按 province 分散 CPMS
    for (site_sfid, site) in &legacy_store.cpms_site_keys {
        let province = &site.admin_province;
        let shard = shards_by_province.entry(province.clone())
            .or_insert_with(|| StoreShard {
                province: province.clone(),
                ..Default::default()
            });
        shard.cpms_site_keys.insert(site_sfid.clone(), site.clone());
    }

    // 1.4 按 province 分散 citizen 记录(通过 archive_no / archive_index 拆分)
    // archive_no 里编码了 province 代码,从现有 archive_no 解析
    for (citizen_id, record) in &legacy_store.citizen_records {
        let province = derive_province_from_citizen(record);
        let shard = shards_by_province.entry(province.clone())
            .or_insert_with(|| StoreShard {
                province: province.clone(),
                ..Default::default()
            });
        shard.citizen_records.insert(*citizen_id, record.clone());
    }

    // 1.5 按 pubkey → province 索引分散 ShiAdmin
    for (pubkey, user) in &legacy_store.admin_users_by_pubkey {
        if user.role != AdminRole::ShiAdmin {
            continue;
        }
        let province = user.admin_province.clone().unwrap_or_default();
        if province.is_empty() {
            continue;
        }
        let shard = shards_by_province.entry(province.clone())
            .or_insert_with(|| StoreShard {
                province: province.clone(),
                ..Default::default()
            });
        shard.local_admins.insert(pubkey.clone(), user.clone());
    }

    // 2. 构建 GlobalShard
    let mut global = GlobalShard::default();
    global.chain_keyring_state = legacy_store.chain_keyring_state.clone();
    global.keyring_rotate_challenges = legacy_store.keyring_rotate_challenges.clone();
    for (pubkey, user) in &legacy_store.admin_users_by_pubkey {
        if matches!(user.role, AdminRole::KeyAdmin | AdminRole::ShengAdmin) {
            global.global_admins.insert(pubkey.clone(), user.clone());
        }
    }
    global.sheng_admin_province_by_pubkey = legacy_store.sheng_admin_province_by_pubkey.clone();
    global.login_challenges = legacy_store.login_challenges.clone();
    global.qr_login_results = legacy_store.qr_login_results.clone();
    global.admin_sessions = legacy_store.admin_sessions.clone();
    global.consumed_qr_ids = legacy_store.consumed_qr_ids.clone();
    global.consumed_cpms_register_tokens = legacy_store.consumed_cpms_register_tokens.clone();
    global.audit_logs = legacy_store.audit_logs.clone();
    global.chain_requests_by_key = legacy_store.chain_requests_by_key.clone();
    global.chain_nonce_seen = legacy_store.chain_nonce_seen.clone();
    global.anon_rsa_private_key_pem = legacy_store.anon_rsa_private_key_pem.clone();
    global.metrics = legacy_store.metrics.clone();

    // 3. 批量 UPSERT 到 store_shards 表(单事务)
    let mut tx = pool.begin().await.map_err(|e| format!("tx: {e}"))?;

    // 3.1 写全局
    let global_payload = serde_json::to_value(&global)
        .map_err(|e| format!("global json: {e}"))?;
    sqlx::query(
        "INSERT INTO store_shards (shard_key, payload, version) VALUES ('global', $1, 1)"
    )
    .bind(global_payload)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("insert global: {e}"))?;

    // 3.2 写 43 省
    for (province, shard) in &shards_by_province {
        let payload = serde_json::to_value(shard)
            .map_err(|e| format!("shard json {province}: {e}"))?;
        sqlx::query(
            "INSERT INTO store_shards (shard_key, payload, version) VALUES ($1, $2, 1)"
        )
        .bind(province)
        .bind(payload)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("insert shard {province}: {e}"))?;
    }

    tx.commit().await.map_err(|e| format!("commit: {e}"))?;

    let elapsed = started.elapsed();
    tracing::info!(
        provinces = shards_by_province.len(),
        elapsed_ms = elapsed.as_millis(),
        "legacy store migrated to sharded structure"
    );
    Ok(())
}

fn derive_province_from_citizen(record: &CitizenRecord) -> String {
    // 优先从 archive_no 解析省代码,回退到 "unknown"
    if let Some(archive_no) = &record.archive_no {
        if let Some(code) = archive_no.get(..2) {
            return crate::sfid::province::province_name_from_code(code)
                .unwrap_or_else(|| "unknown".to_string());
        }
    }
    "unknown".to_string()
}
```

### 6.2 启动调用点

`main.rs` 的启动序列:

```rust
// 1. 现有逻辑:建立 pg_pool,执行 CREATE TABLE / ALTER TABLE 等 schema 迁移
// 2. 加载老 Store(用于迁移源 + 双写兜底)
let legacy_store = load_store_postgres(&pg_pool).await?;

// 3. 启动 ShardedStore
let backend = Arc::new(PostgresShardBackend { pool: pg_pool.clone() });
let double_write = std::env::var("SFID_SHARD_SINGLE_WRITE")
    .map(|v| v != "true")
    .unwrap_or(true);  // 默认双写
let store_shards = Arc::new(ShardedStore::new(backend.clone(), double_write));

// 4. 执行迁移(幂等)
crate::store_shards::migration::migrate_legacy_store_if_needed(&pg_pool, &legacy_store).await?;

// 5. 加载全局分片
store_shards.bootstrap_global().await?;

// 6. 可选:预加载所有省分片(减少首次请求延迟)
let preloaded = store_shards.preload_all_shards().await?;
tracing::info!(preloaded, "sharded store ready");

// 7. 构建 AppState
let state = AppState {
    pg_pool,
    sheng_signer_cache,
    store_shards,
    legacy_store: Arc::new(RwLock::new(legacy_store)),  // 过渡期保留
    // ...其他字段...
};
```

---

## 七、执行步骤(4 天)

### Day 1:数据结构 + 核心 API

**创建文件**:
- `src/store_shards/mod.rs`(主模块)
- `src/store_shards/backend.rs`(trait)
- `src/store_shards/shard_types.rs`(StoreShard / GlobalShard)
- `tests/store_shards_tests.rs`

**任务**:
1. 定义 `StoreShard` / `GlobalShard` 结构(见第 3.3 / 3.4 节)
2. 实现 `ShardedStore` 的核心方法:`read_province` / `write_province` / `read_global` / `write_global` / `get_or_load_shard`(见第 4.1 节)
3. 实现 `MockShardBackend`(仅测试用,内存 HashMap)
4. 单元测试:
   - 基础读写往返
   - 懒加载:首次访问 triggers load
   - 并发读:多个 read 不互锁
   - 并发写:多个 write 同一分片串行,不同分片并行
   - 版本号自增
5. `cargo check` + `cargo test --lib store_shards` 绿

### Day 2:Postgres 后端 + 迁移

**创建文件**:
- `src/store_shards/pg_backend.rs`
- `src/store_shards/migration.rs`
- `db/migrations/020_store_shards.sql`(如果项目有 migration 目录)

**任务**:
1. 实现 `PostgresShardBackend`(见第 3.6 节)
2. `CREATE TABLE store_shards` 加入 `main.rs` 启动 schema(或独立 migration)
3. 实现 `migrate_legacy_store_if_needed`(见第 6.1 节)
4. 在 `main.rs` 启动序列接入:
   - 加载老 store → 执行迁移 → 构造 ShardedStore → bootstrap_global → preload_all_shards
5. 集成测试:
   - 空库首次启动 → `store_shards` 表为空 → 迁移触发 → 43 省 + global 写入
   - 再次启动 → 检测到已迁移 → skip
   - 迁移后数据完整性:抽样 10 条机构 + 10 个 ShiAdmin diff
6. `cargo check` + `cargo test` 绿

### Day 3:Handler 改造(最大一块)

**涉及文件**:
- 所有 `state.store.read()` / `state.store.write()` 调用点
- 按 Grep 清单逐个迁移

**任务**:
1. `Grep state.store.read()` + `state.store.write()` 列出所有调用点(~30~50 处)
2. 按 handler 分类:
   - ShiAdmin 业务(注册机构、citizen 绑定等)→ `write_province` / `read_province`
   - ShengAdmin 管理(列省内资源)→ `read_province`
   - KeyAdmin 全局(登录、轮换、审计)→ `read_global` / `write_global`
   - KeyAdmin 跨省(全国列表)→ `for_each_province`
3. 逐个改造,每改 10 处跑一次 `cargo check`
4. **特别处理**:
   - 登录 handler:`admin_users_by_pubkey` → `GlobalShard.global_admins`(登录路由)+ 本省 local_admins(业务路径)
   - `bootstrap_sheng_signer`:读 ShengAdmin 从 `global.global_admins`,加密私钥字段仍在 global
   - `cleanup_admin_sessions`:sessions 在 GlobalShard,遍历后驱逐 sheng_signer_cache
   - `replace_sheng_admin`:清 global.global_admins + global.sheng_admin_province_by_pubkey,级联动作不变
   - `set_active_main_signer`:级联重加密的 sheng 密文现在从 `global.global_admins` 读/写
5. 功能回归测试(手工):
   - 登录流程
   - 注册机构
   - citizen 绑定(如有)
   - 替换 sheng admin
   - SFID MAIN 轮换(级联重加密)
   - CPMS 激活
6. `cargo check` + `cargo test` 绿

### Day 4:压测 + 双写过渡 + 收官

**任务**:
1. 自写 Rust 压测 `tools/load_test/src/main.rs`(沿用 Phase 1.D 未做的脚本):
   - 参数:`--concurrency N --provinces M --duration T --backend URL`
   - 行为:模拟 N 个虚拟 SHI_ADMIN,均分到 M 省,循环调 `register_sfid_institution`
   - 输出:P50/P95/P99 + 成功率 + 每省 TPS
2. 运行 baseline 对比:
   - Phase 1 状态(`SFID_SHARD_ENABLED=false`,走老 store):50 并发 × 1 省
   - Phase 2 状态(`SFID_SHARD_ENABLED=true`,走新分片):50/100/200 并发 × 1 省
   - Phase 2:10 省 × 50 并发 = 500 并发
3. 验证目标:
   - 单省 100 并发 P99 < 100ms(比 Phase 1 降 >50%)
   - 10 省 500 并发 P99 < 200ms
4. 更新运维文档:
   - 环境变量 `SFID_SHARD_SINGLE_WRITE`(默认 false 双写,过渡 2 周后切 true)
   - 启动迁移日志格式
   - 回滚步骤(环境变量切回 + 重启老二进制)
5. 创建 `feedback_sfid_store_sharded.md` 铁律
6. 任务卡归档 `done/`
7. 更新 `MEMORY.md` 索引

---

## 八、风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| 迁移时丢数据 | 🔴 高 | 双写 2 周 + 老表保留 + 迁移后 diff 抽样校验 |
| Handler 改造漏迁某个调用点 | 🔴 高 | Grep 清单 + 改造一个 handler 先跑 cargo check,逐步推进 |
| DashMap 懒加载竞争 | 🟡 中 | `entry().or_insert()` 原子化,并发测试覆盖 |
| 登录路径读 global 频繁 → GlobalShard 写锁争用 | 🟡 中 | GlobalShard 的写操作只在 session 变化时,频率低;读用 RwLock 并发读不阻塞 |
| Phase 1 的 sheng_signer_cache 数据源依赖 global | 🟡 中 | ShengAdmin 明确放 global,不拆到 province 分片 |
| 跨省 KEY_ADMIN 操作性能下降 | 🟢 低 | KEY_ADMIN 操作频率低,O(43) 可接受 |
| 迁移耗时过长导致启动卡 | 🟢 低 | 预估 <5 秒,实际压测后调整 preload 策略(可跳过预加载,全部走懒加载) |
| 双写失败导致新旧不一致 | 🟡 中 | 失败时日志 ERROR,但不阻塞请求(新路径优先);过渡期结束前发现的不一致手工补齐 |

---

## 九、与 Phase 1 的兼容

### 9.1 Phase 1 改动点的新路径

| Phase 1 改动 | Phase 2 新路径 |
|---|---|
| `AdminUser.encrypted_signing_privkey` / `signing_pubkey` | 留在 `GlobalShard.global_admins[sheng_pubkey]`,不分散到省分片 |
| `sheng_admin_province_by_pubkey` 索引 | `GlobalShard.sheng_admin_province_by_pubkey` |
| `sheng_signer_cache` | **独立于 store_shards**,仍在 AppState 单独 Arc |
| `cleanup_admin_sessions` | 从 `GlobalShard.admin_sessions` 驱逐 + 同步驱逐 cache |
| `bootstrap_sheng_signer` | 读/写走 `write_global` 更新 sheng admin 记录 + cache load_province |
| `replace_sheng_admin` | 级联写 global.global_admins + sheng_admin_province_by_pubkey + 省分片 local_admins(如有 shi 管理员影响)|
| `set_active_main_signer` | 级联重加密的密文从 `global.global_admins` 批量读/写 |
| `submit_register_sfid_institution_extrinsic` | 从 `write_province(province, |s| s.multisig_institutions / accounts)` 读写,签名 signer 继续从 sheng_signer_cache 按 province 取 |

### 9.2 不影响的 Phase 1 模块

- 链端(完全不动)
- sheng_signer_cache / signer_router / chain_sheng_signing(独立于分片)
- 4 个 payload DOMAIN 常量(`GMB_SFID_V1`，2026-04-20 彻底退役为 `DUOQIAN_DOMAIN + OP_SIGN_*`)
- runtime_align.rs 里 `build_institution_credential_with_province`

---

## 十、部署 Runbook

### 10.1 首次部署(Phase 2 上线)

```bash
# 1. 停老 sfid-backend
systemctl stop sfid-backend

# 2. 备份 PG
pg_dump -t runtime_cache_entries -t admins -t operators ... > backup.sql

# 3. 部署新二进制(含 store_shards 模块)
cp target/release/sfid-backend /usr/local/bin/
# 默认 SFID_SHARD_SINGLE_WRITE 未设置 → 双写开启

# 4. 启动
systemctl start sfid-backend

# 5. 观察日志
journalctl -u sfid-backend -f
# 期待看到:
#   "migrating legacy store → sharded structure"
#   "legacy store migrated to sharded structure, provinces=43, elapsed_ms=..."
#   "sharded store ready, preloaded=44"

# 6. 功能回归(手工):登录 / 注册机构 / 替换 sheng admin / 主密钥轮换
# 7. 压测:./sfid-load-test --concurrency 100 --provinces 1 --duration 60s
```

### 10.2 过渡期(2 周)

- 双写保持开启
- 监控 `persist_shard` 和 `legacy persist` 的错误率
- 抽样做 PG diff 检查一致性

### 10.3 切换单写

```bash
# 修改 env
echo "SFID_SHARD_SINGLE_WRITE=true" >> /etc/sfid-backend.env

# 重启
systemctl restart sfid-backend

# 验证日志:"double write disabled, shard-only persistence"
```

### 10.4 回滚(紧急)

```bash
# 1. 停新二进制
systemctl stop sfid-backend

# 2. 回滚到老二进制(双写期间 runtime_cache_entries 仍是全量快照)
cp /backup/sfid-backend-phase1 /usr/local/bin/sfid-backend

# 3. 启动
systemctl start sfid-backend

# 4. 验证登录 + 业务回归
```

**不能回滚的场景**:如果已经切到 `SFID_SHARD_SINGLE_WRITE=true` 超过 24 小时,`runtime_cache_entries` 过期 → 无法回滚。这种情况下只能前滚修复。

---

## 十一、验收标准

### 11.1 功能验收
- [ ] 启动迁移成功,日志显示 `provinces=43, elapsed_ms<5000`
- [ ] 登录流程(KeyAdmin / ShengAdmin / ShiAdmin)三角色全通
- [ ] 注册机构、替换 sheng admin、SFID MAIN 轮换、CPMS 激活 — 手工各走一遍
- [ ] `cargo check` + `cargo test` + `npx tsc --noEmit` + `npm run build` 全绿
- [ ] PG 里 `store_shards` 表有 44 行(43 省 + 1 global),`legacy runtime_cache_entries` 内容仍同步更新(双写期)

### 11.2 性能验收
- [ ] 单省 100 并发 SHI_ADMIN 推 `register_sfid_institution`,**P99 < 100ms**
- [ ] 10 省 × 50 = 500 并发,**P99 < 200ms**
- [ ] 对比 Phase 1 baseline,P99 降低 **> 50%**
- [ ] 成功率 > 99.9%
- [ ] 后端进程 CPU < 200%(单核峰值),内存 < 1 GB

### 11.3 数据完整性
- [ ] 迁移后抽样:10 个机构 / 10 个 ShiAdmin / 10 条 CPMS 记录,新旧路径读出字段全等
- [ ] 双写期间 24 小时后 diff PG `store_shards` vs `runtime_cache_entries`,差异 = 0

---

## 十二、不做的事(严格)

- **不引入** Redis / Memcached / Kafka / gRPC / 分布式锁 / 服务发现
- **不拆微服务**(每省独立进程留给 Phase 4)
- **不改链端**(Phase 1 已定型)
- **不改前端**(API 接口签名不变)
- **不删老表** `runtime_cache_entries`
- **不改 admins / operators 表结构**
- **不改 Phase 1 sheng_signer_cache 的实现**
- **不优化 citizen_records 查询**(留给未来 ClickHouse 方案)

---

## 十三、相关参考

- Phase 1 任务卡(完成):`memory/08-tasks/done/20260409-sfid-sheng-admin-per-province-keyring-impl.md`
- 架构框架:`memory/05-architecture/20260409-sfid-50k-concurrent-framework.md`
- 铁律:
  - `feedback_sfid_sheng_signing_keyring.md`(Phase 1 产出)
  - `feedback_scope_auto_filter.md`(省级 scope 过滤)
  - `feedback_sfid_module_is_single_entry.md`
  - `feedback_sfid_three_roles_naming.md`

---

## 十四、工作量汇总

| 阶段 | 工作量 |
|---|---|
| Day 1 数据结构 + 核心 API | 6 小时 |
| Day 2 PG 后端 + 迁移 | 8 小时 |
| Day 3 Handler 改造(最大块) | 10 小时 |
| Day 4 压测 + 过渡 + 收官 | 6 小时 |
| **合计** | **30 小时 ≈ 4 天** |

---

## 十五、开工信号

回复 **"开工"** 按本方案进入 Day 1 实施(新建 `src/store_shards/mod.rs` 和数据结构定义)。

回复 **"改 X"** 先修订方案再开工。

---

## 附录 A:Grep 清单(Handler 改造参考)

```bash
# Day 3 开始时先执行这组 Grep,生成改造 todo 清单
cd /Users/rhett/GMB/sfid/backend

# 所有 store 读
grep -rn "state\.store\.read\(\)" src/ | wc -l
# 预估 ~30 处

# 所有 store 写
grep -rn "state\.store\.write\(\)" src/ | wc -l
# 预估 ~15 处

# 所有 AdminUser 访问
grep -rn "admin_users_by_pubkey" src/ | wc -l

# 所有 multisig_institutions / multisig_accounts 访问
grep -rn "multisig_institutions\|multisig_accounts" src/ | wc -l

# 所有 cpms_site_keys 访问
grep -rn "cpms_site_keys" src/ | wc -l

# 所有 citizen_records 访问
grep -rn "citizen_records" src/ | wc -l
```

每一项都需要判断:
- 属于哪个 shard(global / province)
- 如果属于 province,province 怎么从 handler ctx 拿
- 读还是写,改成对应 API

---

## 附录 B:环境变量清单

| 变量 | 默认 | 作用 |
|---|---|---|
| `SFID_SHARD_SINGLE_WRITE` | false(双写)| true 时跳过 legacy 同步,只写新分片 |
| `SFID_SHARD_PRELOAD_ALL` | true | false 时跳过启动预加载,全部走懒加载 |
| `SFID_SHARD_MIGRATION_SKIP` | false | true 时跳过迁移(紧急情况,不推荐) |

---

**本方案已定稿,不再留拍板项。回复 "开工" 直接进入 Day 1。**
