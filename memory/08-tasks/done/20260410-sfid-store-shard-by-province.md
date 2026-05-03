# Phase 2 技术方案:Store 按省分片

- **任务 ID**: `20260410-sfid-store-shard-by-province`
- **版本**: v1 初版
- **日期**: 2026-04-09
- **工作量**: 3~4 天
- **优先级**: 高(解除 50K 并发路径上第二个瓶颈)
- **前置**: Phase 1 `sfid-sheng-admin-per-province-keyring` 已完成
- **架构上下文**: `memory/05-architecture/20260409-sfid-50k-concurrent-framework.md`
- **状态**: 待开工

---

## 一、目标与范围

### 目标
- **解除后端 Store 全局锁 + 全量 JSON 反序列化的瓶颈**
- 承载能力:从 Phase 1 的 ~500 并发 → **~5000 并发 SHI_ADMIN**
- 为 Phase 3(提交队列批处理)和 Phase 4(水平扩展)铺路

### 范围
- **只改 sfid-backend 数据访问层**
- **不改**:链端、前端、HTTP 接口签名、业务逻辑、任务卡/前端现有功能
- **不引入**:Redis、分布式锁、pg 读副本、新中间件

---

## 二、现状分析

### 现状瓶颈

当前 `sfid/backend/src/main.rs` 的 `load_store_postgres`:

```
每次 HTTP 请求中 store 状态变化时触发:
  1. SELECT payload FROM runtime_cache_entries WHERE key='sfid_store'  (~1.2MB JSONB)
  2. serde_json::from_str → BackendStore (含 43 省所有数据)
  3. 再从 admins / operators / 其他关系表做 5 次 SELECT 补全
  4. 返回完整 Store
```

**问题**:
- 单次 load ~50~100ms(1.2MB JSON + 5 张表 SELECT)
- 50K 并发请求 → 每请求触发一次 full load → 串行化 + CPU 爆
- `Arc<RwLock<BackendStore>>` 是单锁,读写互斥

### 现有持久化路径(不改,只做增量)

- `runtime_cache_entries(key TEXT, payload JSONB)` —— 单行大 JSON 快照,key = `sfid_store`
- `admins(admin_id, admin_pubkey, ...)` —— 管理员结构表
- `operators(id, ...)` —— 操作员结构表
- 其他小表(audit_log 等)

---

## 三、架构设计

### 3.1 目标结构

```
┌──────────────────────────────────────────────────────────┐
│ AppState                                                 │
│                                                          │
│ store_shards: Arc<DashMap<String, Arc<RwLock<StoreShard>>>> │
│   ├── "辽宁省" → StoreShard { province: 辽宁, ... }     │
│   ├── "安徽省" → StoreShard { ... }                      │
│   └── ...(43 省,懒加载)                                │
│                                                          │
│ global_shard: Arc<RwLock<GlobalShard>>                   │
│                                                          │
│ sheng_signer_cache: Arc<ShengSignerCache>(Phase 1 已有)  │
└──────────────────────────────────────────────────────────┘
            │
            ↓ write-through
┌──────────────────────────────────────────────────────────┐
│ PostgreSQL                                               │
│                                                          │
│ store_shards(province TEXT PK, payload JSONB, updated_at)│
│   ├── "global" → 全局数据                                 │
│   ├── "辽宁省" → 辽宁分片                                 │
│   └── ...                                                 │
│                                                          │
│ admins / operators / audit_logs / ...(现有表保留)         │
└──────────────────────────────────────────────────────────┘
```

### 3.2 分片维度

按 **province**(省名 UTF-8 字符串)切分。特殊分片 `"global"` 存跨省数据。

### 3.3 StoreShard 内容(每省一份)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct StoreShard {
    pub(crate) province: String,

    // ── 本省管理员(SHENG_ADMIN + SHI_ADMIN,从 admins 表关联)──
    pub(crate) local_admins: HashMap<String, AdminUser>,

    // ── 本省机构(两层模型)──
    pub(crate) multisig_institutions: HashMap<String, MultisigInstitution>,
    pub(crate) multisig_accounts: HashMap<String, MultisigAccount>,

    // ── 本省 CPMS 站点 ──
    pub(crate) cpms_site_keys: HashMap<String, CpmsSiteKeys>,

    // ── 本省 citizen 记录(未来最大数据量)──
    pub(crate) citizen_records: HashMap<u64, CitizenRecord>,
    pub(crate) citizen_id_by_pubkey: HashMap<String, u64>,
    pub(crate) citizen_id_by_archive_no: HashMap<String, u64>,

    // ── 本省 pending 操作 ──
    pub(crate) pending_bind_scan_by_qr_id: HashMap<String, PendingBindScan>,
    pub(crate) pending_status_by_archive_no: HashMap<String, CitizenStatus>,

    // ── 其他本省独占状态 ──
    // ...按现有 Store 字段分类
}
```

### 3.4 GlobalShard 内容(跨省共享)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GlobalShard {


    // 登录 challenge(全局会话)
    pub(crate) login_challenges: HashMap<String, LoginChallenge>,
    pub(crate) qr_login_results: HashMap<String, QrLoginResultRecord>,
    pub(crate) admin_sessions: HashMap<String, AdminSession>,

    // 省份索引(pubkey → province,用于路由)
    pub(crate) sheng_admin_province_by_pubkey: HashMap<String, String>,

    // 全局 SFID 生成历史
    pub(crate) generated_sfid_by_pubkey: HashMap<String, String>,
    pub(crate) consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    pub(crate) consumed_cpms_register_tokens: HashMap<String, DateTime<Utc>>,

    // 审计日志(可能移到独立 ClickHouse,这里先留 PG)
    pub(crate) audit_logs: Vec<AuditLogEntry>,

    // 链请求幂等
    pub(crate) chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    pub(crate) chain_nonce_seen: HashMap<String, DateTime<Utc>>,

    // RSA 匿名证书密钥
    pub(crate) anon_rsa_private_key_pem: Option<String>,

    // 其他全局设置
    pub(crate) chain_auth_last_cleanup_at: Option<DateTime<Utc>>,
    pub(crate) pending_bind_last_cleanup_at: Option<DateTime<Utc>>,
    pub(crate) metrics: ServiceMetrics,
}
```

### 3.5 访问 API(新封装)

```rust
// src/store_shards/mod.rs(新建)

pub(crate) struct ShardedStore {
    shards: DashMap<String, Arc<RwLock<StoreShard>>>,
    global: Arc<RwLock<GlobalShard>>,
    backend: Arc<dyn ShardBackend>,    // PG 写穿透层
}

impl ShardedStore {
    /// 读本省分片(懒加载)
    pub(crate) async fn read_province<F, R>(
        &self,
        province: &str,
        f: F,
    ) -> Result<R, String>
    where
        F: FnOnce(&StoreShard) -> R,
    {
        // 1. DashMap get_or_insert: 没有则从 PG 加载
        let shard = self.get_or_load_shard(province).await?;
        let guard = shard.read().map_err(|_| "shard poisoned")?;
        Ok(f(&*guard))
    }

    /// 写本省分片 + 同步持久化
    pub(crate) async fn write_province<F, R>(
        &self,
        province: &str,
        f: F,
    ) -> Result<R, String>
    where
        F: FnOnce(&mut StoreShard) -> R,
    {
        let shard = self.get_or_load_shard(province).await?;
        let result = {
            let mut guard = shard.write().map_err(|_| "shard poisoned")?;
            f(&mut *guard)
        };
        // 写穿透 PG(异步)
        self.persist_shard_async(province).await?;
        Ok(result)
    }

    /// 读全局分片
    pub(crate) fn read_global<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&GlobalShard) -> R,
    {
        let guard = self.global.read().map_err(|_| "global poisoned")?;
        Ok(f(&*guard))
    }

    /// 写全局分片 + 同步持久化
    pub(crate) async fn write_global<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut GlobalShard) -> R,
    {
        let result = {
            let mut guard = self.global.write().map_err(|_| "global poisoned")?;
            f(&mut *guard)
        };
        self.persist_global_async().await?;
        Ok(result)
    }

    async fn get_or_load_shard(
        &self,
        province: &str,
    ) -> Result<Arc<RwLock<StoreShard>>, String> {
        if let Some(s) = self.shards.get(province) {
            return Ok(s.clone());
        }
        // 懒加载:从 PG 读
        let shard = self.backend.load_shard(province).await?
            .unwrap_or_else(|| StoreShard {
                province: province.to_string(),
                ..Default::default()
            });
        let arc = Arc::new(RwLock::new(shard));
        self.shards.insert(province.to_string(), arc.clone());
        Ok(arc)
    }

    async fn persist_shard_async(&self, province: &str) -> Result<(), String> {
        let shard = self.shards.get(province)
            .ok_or("shard not loaded")?
            .clone();
        let snapshot = {
            let guard = shard.read().map_err(|_| "shard poisoned")?;
            guard.clone()
        };
        self.backend.save_shard(province, &snapshot).await
    }

    async fn persist_global_async(&self) -> Result<(), String> {
        let snapshot = {
            let guard = self.global.read().map_err(|_| "global poisoned")?;
            guard.clone()
        };
        self.backend.save_global(&snapshot).await
    }
}

#[async_trait]
pub(crate) trait ShardBackend: Send + Sync {
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String>;
    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String>;
    async fn load_global(&self) -> Result<GlobalShard, String>;
    async fn save_global(&self, global: &GlobalShard) -> Result<(), String>;
}
```

### 3.6 Postgres Schema 改造

**新增表**:
```sql
CREATE TABLE IF NOT EXISTS store_shards (
    shard_key TEXT PRIMARY KEY,   -- "global" 或省名
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_store_shards_updated_at
    ON store_shards(updated_at);
```

**保留旧表**:`runtime_cache_entries` / `admins` / `operators` 等不动,**只做增量迁移**。

### 3.7 迁移策略

#### 3.7.1 启动时一次性迁移

后端启动:
1. 检查 `store_shards` 表是否有数据
2. 若有 → 正常加载分片路径
3. 若无(首次升级)→
   - 从 `runtime_cache_entries` 读旧 JSON 全量快照
   - 从 `admins` / `operators` 等关系表读补充数据
   - 按 province 拆分成 43 个 `StoreShard` + 1 个 `GlobalShard`
   - 批量 UPSERT 到 `store_shards` 表
   - 打印日志 "migrated legacy store → 43 shards"

#### 3.7.2 过渡期双写

在 `sfid-backend` 进程生命周期内:
- **读路径**:优先新分片(DashMap + PG shards 表)
- **写路径**:
  - 先写新分片(内存 + PG shards 表)
  - 再写老路径(`runtime_cache_entries` 全量快照 + 关系表)—— 确保回滚时老后端能正常启动

双写代价:每次写多 1 次 PG 操作(~5ms),但换来回滚安全。过渡期约 1~2 周。

#### 3.7.3 过渡期结束

切换开关环境变量 `SFID_SHARD_SINGLE_WRITE=true`:
- 只写新分片,不再更新 `runtime_cache_entries`
- 老关系表仍正常写(`admins` / `operators` 等不变)

完全切换后,老 `runtime_cache_entries` 表可以保留作为冷归档,不删除。

---

## 四、路由层改造

### 4.1 按 province 路由访问

**所有业务 handler 改造路径**:

```rust
// 旧:
let store = state.store.read()?;
let institution = store.multisig_institutions.get(&sfid_id)?;

// 新:
let institution = state.store_shards.read_province(&province, |shard| {
    shard.multisig_institutions.get(&sfid_id).cloned()
}).await?;
```

**province 从哪里来**:
- SHI_ADMIN / ShengAdmin 的请求 → `ctx.admin_province`(已有)
- 公开路径(citizen 绑定等)→ 从 sfid_id 里解析省份代码(见 `sfid/province.rs`)

### 4.2 跨省查询


```rust
// 遍历所有分片收集
pub(crate) async fn list_all_institutions(
    store_shards: &ShardedStore,
) -> Result<Vec<MultisigInstitution>, String> {
    let mut result = Vec::new();
    for entry in store_shards.shards.iter() {
        let province = entry.key();
        let shard_institutions = store_shards.read_province(province, |s| {
            s.multisig_institutions.values().cloned().collect::<Vec<_>>()
        }).await?;
        result.extend(shard_institutions);
    }
    Ok(result)
}
```


### 4.3 全局 state 路径

涉及 `GlobalShard` 字段的 handler 改走 `read_global` / `write_global`:
- 登录 challenge / session
- 审计日志
- 链请求幂等

---

## 五、执行步骤(4 天)

### Day 1:Store 分片数据结构 + 访问 API

- [ ] 新建 `src/store_shards/mod.rs` 模块
- [ ] 定义 `StoreShard` / `GlobalShard` / `ShardedStore` / `ShardBackend` trait
- [ ] 实现 `read_province` / `write_province` / `read_global` / `write_global`
- [ ] 实现 `get_or_load_shard` 懒加载逻辑
- [ ] 写单元测试:基础读写 + 懒加载 + 并发安全
- [ ] `cargo check` 绿

### Day 2:Postgres 后端 + 迁移

- [ ] 新建 `src/store_shards/pg_backend.rs` 实现 `ShardBackend`
- [ ] `store_shards` 表 CREATE / ALTER / CRUD SQL
- [ ] 启动时迁移逻辑(`runtime_cache_entries` → 分片)
- [ ] 双写逻辑(新分片 + 老 `runtime_cache_entries`)
- [ ] 单元测试:save/load 往返 + 迁移幂等
- [ ] `cargo check` 绿

### Day 3:业务 handler 改造(最大一块)

- [ ] AppState 扩展 `store_shards: Arc<ShardedStore>`
- [ ] Grep 所有 `state.store.read()` / `state.store.write()` 调用点
- [ ] 按 province 分类路由到 `read_province` / `write_province`
- [ ] 全局状态改走 `read_global` / `write_global`
- [ ] 跨省查询场景保留 `list_all_*` helper
- [ ] 单元测试 + 集成测试
- [ ] `cargo check` 绿

### Day 4:性能验证 + 双写过渡 + 收官

- [ ] 本地起后端,跑压测脚本(`sfid-load-test`)
- [ ] 对比 baseline:Phase 1 vs Phase 2 的 P50/P99
- [ ] 目标:单省 100 并发时 P99 降低 >50%
- [ ] 环境变量 `SFID_SHARD_SINGLE_WRITE=false`(过渡期)/ `true`(完全切换)
- [ ] 更新 `feedback_sfid_store_sharded.md`
- [ ] 任务卡归档 done/

---

## 六、关键风险 + 缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| 迁移时数据丢失 | 🔴 高 | 双写 + 旧表保留 + 迁移后做 diff 校验 |
| 跨省事务一致性 | 🟡 中 | 避免设计跨省事务;少数全局操作走 GlobalShard |
| DashMap 懒加载时并发竞争 | 🟡 中 | `entry().or_insert_with` 原子化 |
| 分片间的脏读 | 🟡 中 | 写穿透保证单分片强一致,跨分片不需要强一致 |
| Phase 1 的 `admin_users_by_pubkey` 访问路径被破坏 | 🟡 中 | 保持向后兼容:`AdminUser` 按 province 分散到 `shard.local_admins`,`GlobalShard` 里保留全局 pubkey → province 索引供登录用 |
| 启动迁移耗时过长 | 🟢 低 | 43 省 +1 全局 = 44 次批量 UPSERT,预估 <5 秒 |
| 双写开销 | 🟢 低 | 每次写 +5ms,过渡期可接受 |

---

## 七、与 Phase 1 的兼容

Phase 1 的改动需要迁移到新分片结构:

1. **`AdminUser.encrypted_signing_privkey` / `signing_pubkey`**
   - ShengAdmin 记录的 province 字段决定它属于哪个 `StoreShard.local_admins`
   - 登录 handler 里 `bootstrap_sheng_signer` 访问路径改为 `store_shards.write_province(&province, |s| s.local_admins.get_mut(&pubkey))`

2. **`sheng_admin_province_by_pubkey` 索引**
   - 留在 `GlobalShard`(登录时需要全局查 pubkey → province)
   - 写路径:replace_sheng_admin 时同时更新 GlobalShard 索引 + 新 ShardX.local_admins

3. **`sheng_signer_cache`**
   - 独立于 store_shards,继续在 AppState 里作为单独的 Arc
   - 数据源依然是解密后的 seed,不经过 PG

4. **`cleanup_admin_sessions`**
   - sessions 在 `GlobalShard.admin_sessions`
   - idle 清理时从全局分片驱逐,同步驱逐 cache

---

## 八、不做的事

- **不引入 Redis / Memcached / Kafka / gRPC**(保持单体 Rust + PG)
- **不拆微服务**(不按 province 拆后端进程,那是 Phase 4)
- **不改链端**(Phase 1 已定型)
- **不改前端**(API 接口签名不变)
- **不改现有 HTTP 路径**(只改访问层)
- **不删老 `runtime_cache_entries` 表**(保留作兜底归档)
- **不改 `admins` / `operators` 表结构**(它们继续是"快速索引"+关系查询源)

---

## 九、验收标准

- [ ] `cargo check` 全绿
- [ ] 单元测试覆盖:ShardedStore 读写、懒加载、迁移、双写、跨省查询
- [ ] 启动时迁移日志:"migrated legacy store → 43 shards + 1 global, took X ms"
- [ ] 压测:单省 100 并发,P99 较 Phase 1 降低 >50%
- [ ] 压测:10 省 500 并发(每省 50 并发),P99 保持在 <200ms
- [ ] 数据完整性:迁移后做 diff,新老路径读取机构/citizen/操作员各 10 条抽样对比

---

## 十、开工前需要你拍板的事

### 拍板 1:分片维度

- [ ] 按 province_code 数字:索引更紧凑但 UTF-8 省名更可读
- [ ] 按 (province, category):更细粒度但跨 category 查询复杂

### 拍板 2:GlobalShard 是否拆进 PG 结构表

- [ ] **走 JSONB(推荐)**:和省分片一样 `store_shards` 表统一处理,简单
- [ ] 各字段拆进对应结构表:查询更快但 schema 复杂

### 拍板 3:过渡期双写

- [ ] **开启双写(推荐)**:2 周过渡期,支持无损回滚
- [ ] 直接单写:一刀切,快但风险高

### 拍板 4:迁移时机

- [ ] **后端启动时自动迁移**:首次检测到 `store_shards` 表空自动迁移
- [ ] 单独的 CLI 工具执行一次迁移:更可控但多一个运维步骤

### 拍板 5:压测目标数值

- [ ] **单省 100 并发 P99 <100ms + 10 省 500 并发 P99 <200ms(推荐)**
- [ ] 更激进:单省 500 并发
- [ ] 更保守:单省 50 并发

---

## 十一、相关参考

- Phase 1 任务卡:`memory/08-tasks/done/20260409-sfid-sheng-admin-per-province-keyring-impl.md`
- 架构框架:`memory/05-architecture/20260409-sfid-50k-concurrent-framework.md`
- 铁律:
  - `feedback_sfid_sheng_signing_keyring.md`(Phase 1 产出)
  - `feedback_scope_auto_filter.md`(省级 scope 过滤)
  - `feedback_sfid_module_is_single_entry.md`

## 十二、开工信号

回复 **"开工"** 按本方案进入 Day 1 实施。
回复 **"改 X"** 先修订方案再开工。
