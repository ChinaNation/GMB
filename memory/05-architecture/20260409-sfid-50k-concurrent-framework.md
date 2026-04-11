# SFID 50K 并发架构框架(5000 市 × 10 SHI_ADMIN)

- **文档类型**: 架构设计
- **日期**: 2026-04-09
- **触发**: 用户明确目标 —— 5000 市 × 10 市级管理员 = 50,000 同时在线
- **前置**: 任务卡 `20260409-sfid-sheng-admin-per-province-keyring`(省级密钥拆分)
- **相关铁律**:
  - `feedback_chain_in_dev.md` / `feedback_no_chain_restart.md`
  - `feedback_sfid_pow_chain_recipe.md` / `feedback_chainspec_frozen.md`
  - `feedback_sfid_three_roles_naming.md` / `feedback_scope_auto_filter.md`

## 规模假设

| 维度 | 量级 | 备注 |
|---|---|---|
| 省 | 43 | 固定 |
| 市 | ~5000 | 平均每省 116 市 |
| 每市 SHI_ADMIN | 10 | 均值 |
| SHI_ADMIN 总数 | ~50,000 | 同时在线上限 |
| 每 SHI_ADMIN 平均链操作频率 | 1 次 / 分钟 | 业务节奏估算 |
| **全局峰值 TPS(链写)** | **~833 tx/s** | 50000 / 60 |
| 机构数量 | ~5000(公安局)+ X(公权/私权) | |
| citizen 记录(未来) | 亿级 | 由 archive 导入产生 |

## 瓶颈分析(按层次从低到高)

### 1. 链端吞吐(**最硬的天花板**)

- Substrate 单块 ~1000 extrinsics,PoW 出块 ~15 秒 → **理论峰值 ~66 TPS**
- 实际瓶颈通常在 weight(每个 `register_sfid_institution` 大致 ~10M weight)→ 实际可能 30~50 TPS
- **833 TPS 目标直接超链 10~25 倍** —— 必须在应用层做批处理/削峰

### 2. 后端进程内 nonce 并行

- 当前单 KEY_ADMIN signer:任何时刻只有一个 "待上链 nonce",串行
- 43 省独立 signer 后:43 条并行 nonce 通道
- 每省平均 ~1160 并发 SHI_ADMIN,每 1 分钟 ~19 tx/秒/省 → 单省 **~19 TPS**,
  在链端接受度内但依然是省内串行瓶颈

### 3. 后端内存 / 磁盘 IO

- 当前 `Store` 单 `RwLock<Store>`,所有读写串行化
- 每次 HTTP 请求都可能触发 `load_store_postgres()` 全量反序列化(1.2MB JSON)
- 50K 并发 × N 请求/秒 × 1.2MB 全量读 = IO 直接炸

### 4. HTTP 层

- axum 异步 IO,单进程 50K 连接无压力
- 单进程 CPU 是瓶颈(大量 sr25519 验签 / SCALE 编解码)

### 5. 数据库

- 现在 `runtime_cache_entries` 是一张 JSON bloat 表,单行几 MB
- 真正的高频读写必须落到结构化表 + 索引

---

## 分层解决方案

### 层 0:链端 —— 批量交易(batch extrinsic)

**方案**:
- 新增 runtime extrinsic `register_sfid_institution_batch(batch: Vec<Institution>)`
  一次注册 N 个机构,一次签名 + 一次 nonce 消耗
- 批大小建议 50~100(太大容易 weight 超限)
- 链端 pallet 内部循环 dispatch,共享 verifier 校验

**效果**:833 TPS / 50 = **17 批/秒**,落到 ~5 个省级 signer 的 nonce 容量内完全 OK

**不做**:
- 不改 PoW 出块速度(`feedback_chain_in_dev` / `feedback_no_chain_restart`)
- 不上 L2 / sharding(复杂度爆炸,当前体量不需要)

### 层 1:后端 —— 省级 signer 并行 + 提交队列

基于 `20260409-sfid-sheng-admin-per-province-keyring` 任务卡的 43 省独立 signer,再叠加:

**1.1 省级提交队列(submit queue per province)**

```rust
struct ProvinceSubmitQueue {
    province: String,
    pending: SegQueue<SubmitJob>,              // 无锁 MPSC
    signer: PairSigner<...>,
    batch_buffer: Mutex<Vec<SubmitJob>>,       // 累积批次
    last_flush_at: AtomicU64,
}

// 后台 task per province(共 43 个 tokio task)
async fn province_flusher(queue: Arc<ProvinceSubmitQueue>) {
    loop {
        // 收集条件:buffer 满(50 条)OR 超时(200ms)
        // 满足任一即 flush:构造 batch extrinsic → 签名 → submit → InBestBlock
        // 每次 flush 是一次 nonce 推进
    }
}
```

**效果**:
- SHI_ADMIN 请求 → 后端入队 → 立即返回(链上 pending 状态)→ 批处理后台 flush
- 50K 并发被 43 × 5 批/秒 = 215 批/秒吸收(P99 延迟 ~200ms 队列等待)
- **关键:后端 HTTP 响应不阻塞链上确认**,前端轮询或 callback 获取最终 tx_hash

**代价**:
- SHI_ADMIN 点"注册机构"后 UI 看到的是"已提交,等待上链",几秒~几十秒后才确认
- 需要引入"任务状态"机制:`pending` / `batched` / `submitted` / `in_block` / `finalized` / `failed`

**1.2 签名并行度**

虽然 43 个省级 task 已经充分 CPU 并行(sr25519 签名是 CPU-bound),但每个 task
内部的 batch 构造 / SCALE 编码仍可以用 rayon worker pool 进一步加速。可选优化,
初版先不做。

### 层 2:后端 —— 内存缓存 + 写穿透

**根因**:当前每个 HTTP 请求都 `load_store_postgres()` → 1.2MB JSON 反序列化。

**方案**:
```rust
pub(crate) struct AppState {
    // 旧:store: Arc<RwLock<BackendStore>> + 每次 load_store_postgres()
    // 新:
    store: Arc<DashMap<ShardKey, StoreShard>>,   // 按 province 分片
    submit_queues: Arc<HashMap<String, Arc<ProvinceSubmitQueue>>>,
    pg_pool: PgPool,                              // 只写(write-through)
    replication_log: ChangeLog,                   // 可选:链下 CDC 到 ClickHouse
}
```

**StoreShard 结构**:
- 按 province 切成 43 片,每片独立 `RwLock` 或 `DashMap`
- 读:优先内存,miss 时从 PG 加载单片(~30KB),cache in memory
- 写:先改内存,同步 append-only 写 PG(WAL-like),**不做每次 JSON 全量 dump**
- 启动:并行从 PG 加载 43 个 shard 到内存

**效果**:
- 50K 并发读 → 全内存命中 → 单进程 >10K QPS 无压力
- 写入走 append log → 单次写 <1ms
- 进程崩溃恢复:从 PG WAL 重放最后 N 条

**迁移策略**:
- 不改现有 PG schema(`runtime_cache_entries`),而是**新增 `store_shards` 表**
  (province, key, value JSONB, updated_at)
- 老的 `runtime_cache_entries` 保留作全量快照,后台任务定期从 shards 聚合
- 新接口路径全走 shards,老接口逐步迁移

### 层 3:后端 —— 水平扩展(多进程 / 多实例)

**问题**:单进程 Rust 后端可能在 CPU 上被 5 万并发打穿(sr25519 验签 / 连接密度)。

**方案**:
- 前置 **nginx** 或 **haproxy**,按 `admin_province` 做 **sticky routing**
- N 个后端进程,每个负责 `43/N` 省的请求(通过 consistent hash by province)
- 同一省的请求永远落到同一进程 → 内存 shard 无冲突、nonce 无冲突、cache 无抖动

**实例数建议**:
- 起步 3 实例(每实例负责 ~14 省,~16K 并发 SHI_ADMIN)
- 压测后根据 CPU 使用率调整到 6~8 实例

**实例间通信**:**不需要**。每省数据强归属单实例,跨省请求极少(KEY_ADMIN 全局操作)。

**KEY_ADMIN 操作路由**:
- KEY_ADMIN 请求 sticky 到"leader 实例"(预设第一个)
- 修改 `sheng_admin_pubkey` 等操作由 leader 处理,写入自己负责的 shards + 调链
- 其他实例通过**链上事件订阅**(`ShengAdminPubkeyUpdated` event)感知变化并刷新自己的省级 signer cache

### 层 4:数据库 —— 读写分离

**方案**:
- PG 主库:只处理写(append log from shards + chain event log)
- PG 只读副本:处理审计查询、历史记录查询(老接口 `list_citizens` / `list_audit_log` 等)
- SHI_ADMIN 热路径 100% 走内存 shards,不碰 PG

**何时需要**:
- 单实例内存 cache 命中率 >95%
- PG 写入 QPS 超 500 时加读副本
- 初版不做,等压测数据再决定

### 层 5:监控 + 削峰

**5.1 Prometheus 指标**:
- `sfid_submit_queue_depth{province}` gauge
- `sfid_submit_queue_wait_ms{province}` histogram
- `sfid_chain_submit_duration_ms{province}` histogram
- `sfid_chain_submit_errors{province, error_type}` counter
- `sfid_store_shard_cache_hit{shard}` counter
- `sfid_http_inflight{role}` gauge

**5.2 Grafana 面板**:
- 每省 queue depth 实时图
- 每省 TPS / 错误率
- 全局 P50 / P95 / P99 端到端延迟

**5.3 软限流**:
- 单省 queue depth >500 时返回 `429 Too Many Requests`,前端 exponential backoff
- KEY_ADMIN 白名单不被限流

**5.4 熔断**:
- 某省链端连续失败 3 次 → 该省队列暂停 30 秒 → 告警 → 自动重试
- 熔断期间前端显示"本省暂时繁忙,请稍后重试"

---

## 分阶段落地路线图

**不要试图一次把所有层都上**。按优先级分期落地:

### Phase 0(当前状态)
- 单 KEY_ADMIN signer,单 Store RwLock,单进程,每请求全量 load
- 可承载:~50 并发 SHI_ADMIN
- **距 50K 目标:** 1000 倍缺口

### Phase 1(本次任务卡 sheng-admin-per-province-keyring)
- 43 省独立 signer,链上 verifier 扩展
- 消除 nonce 串行瓶颈
- 可承载:**~500 并发 SHI_ADMIN**(Store 内存瓶颈尚未解决)
- **工作量**:2.75 天(已在任务卡里)

### Phase 2(下一步:Store 内存分片)
- 按 province 切 43 shards,内存 DashMap,写穿透 PG
- 读走内存,写 append 日志
- 可承载:**~5000 并发 SHI_ADMIN**
- **工作量**:3~4 天
- 新任务卡 ID 建议:`20260410-sfid-store-shard-by-province`

### Phase 3(关键优化:省级提交队列 + batch extrinsic)
- 链端新增 `register_sfid_institution_batch` extrinsic
- 后端每省一个 flush task,200ms 或 50 条聚合 flush
- 前端引入"任务状态轮询"(pending → in_block)
- 可承载:**~30K 并发 SHI_ADMIN**
- **工作量**:5~7 天
- 新任务卡 ID 建议:`20260411-sfid-submit-queue-batching`

### Phase 4(水平扩展:多实例 sticky routing)
- 前置 nginx,consistent hash by province
- 3 实例起步,链事件订阅同步省级 signer cache
- 可承载:**50K+ 并发 SHI_ADMIN**(目标达成)
- **工作量**:2~3 天(主要是运维 + 链事件订阅实现)
- 新任务卡 ID 建议:`20260412-sfid-horizontal-scale`

### Phase 5(可选:PG 读写分离 + ClickHouse 审计)
- 根据 Phase 4 压测结果决定是否做
- **工作量**:2~3 天

**总工作量预估**:Phase 1~4 约 **13~18 天**,不含运维和压测迭代。

---

## 架构决策记录(ADR 摘要)

| 决策 | 选择 | 原因 |
|---|---|---|
| 签名密钥隔离粒度 | **按省(43 份)** | 匹配业务行政边界 + 链上审计可追溯 + 运维成本合理 |
| 密钥冗余 | **无冗余(1 key per province)** | 用户明确简化 + 轮换 = 替换,不需要备份 |
| 密钥存储位置 | **后端加密存储** | 信任模型下 KEY_ADMIN 本就是最高权威 + 运维单体 |
| 加密算法 | **AES-256-GCM + HKDF-SHA256** | 用户拍板统一 |
| 链端升级方式 | **on-chain setCode** | `feedback_chainspec_frozen` 铁律 |
| 批处理粒度 | **50 tx / batch** | 链 weight 上限约束下的最大聚合量 |
| Store 分片维度 | **按 province** | 天然业务边界 + 无跨分片事务 |
| 水平扩展策略 | **sticky routing by province** | 避免分布式锁 / 共享状态 |
| 数据库角色 | **持久化 + 审计,非热路径** | 所有热路径走内存 |
| 压测工具 | **自写 Rust** | 用户拍板,精确控制分省并发分布 |
| 跨实例同步 | **链事件订阅 only** | 链本身是 source of truth,不额外引入 pub/sub |

---

## 关键风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| 链端 batch extrinsic weight 估算错误 → block 被拒 | 高 | 压测时用多种 batch size 找到真实上限,代码里硬编码保守值 |
| 省级 signer 私钥在 3 个后端实例之间不同步(密钥轮换后) | 高 | 链事件订阅 + 轮询 fallback,最终一致性窗口 <10 秒 |
| 某省 queue 堆积 >10K 条 → 内存爆 | 中 | 软限流 + 熔断 + 持久化 queue(可选) |
| 后端实例重启 → queue 里未 flush 的 jobs 丢失 | 中 | 入队前先 WAL 到 PG,实例启动时恢复 |
| 链上 nonce 与后端预期 nonce 不一致(重启后) | 中 | 启动时拉链上 nonce 作为初始值,不信本地 cache |
| sticky routing 的 province 偏斜(某省 SHI_ADMIN 特别多) | 低 | 预留手动 override 路由表,紧急情况直接拉偏 |
| KEY_ADMIN 下线后无法 unlock 新启动的实例 | 中 | 实例启动后 fallback 到"仅 KEY_ADMIN 兜底"模式(慢但不断服务) |
| 链停摆(PoW 算力不足) | 高 | 超出本框架范围,属于链端运维 |

---

## 不做的事(严格)

- **不上分片链**:单条 Substrate 链够用,别引入 Polkadot parachain 复杂度
- **不引入 Redis / Memcached**:Rust 进程内 DashMap 已经足够,省一层网络 hop
- **不引入 Kafka / RabbitMQ**:省级提交队列是进程内 SegQueue,不跨进程
- **不做分布式锁**:sticky routing 天然避免跨进程冲突
- **不做实时 CDC**:审计需求可延后 1 分钟到 PG 读副本即可
- **不引入 gRPC / 微服务拆分**:单体 Rust 后端 + 水平复制足够
- **不引入 TiDB / YugabyteDB**:PG 加读副本够用
- **不做跨省事务**:业务上不需要,每省强隔离

---

## 对现有代码的侵入点汇总

| 改动文件 | Phase | 动作 |
|---|---|---|
| `citizenchain/frame/sfid-code-auth/src/lib.rs` | 1 | 新增 ShengAdminPubkey storage + extrinsic + verifier |
| `citizenchain/frame/sfid-code-auth/src/lib.rs` | 3 | 新增 `register_sfid_institution_batch` extrinsic |
| `sfid/backend/src/models/mod.rs` | 1 | AdminUser 扩展 encrypted_privkey + chain_version |
| `sfid/backend/src/key-admins/` | 1 | 新 `signer_cache.rs`(42 省 cache) |
| `sfid/backend/src/sheng-admins/` | 1 | `replace_sheng_admin` handler 扩展级联轮换 |
| `sfid/backend/src/app_core/runtime_ops.rs` | 1 | 启动钩子加"等待 unlock"状态 |
| `sfid/backend/src/models/store.rs`(新建) | 2 | 43 shard 数据结构 |
| `sfid/backend/src/pg_backend.rs` | 2 | append-only WAL 写 shard changes |
| `sfid/backend/src/sheng-admins/institutions.rs` | 3 | 改 submit 为入队 |
| `sfid/backend/src/submit_queue/`(新建) | 3 | 省级 flush task + batch 聚合 |
| `sfid/backend/Cargo.toml` | 3 | 加 `crossbeam-queue` 依赖 |
| `sfid/frontend/src/api/` | 3 | 增加任务状态轮询接口 |
| `sfid/frontend/src/views/institutions/` | 3 | UI 改"已提交,等待上链"状态显示 |
| 部署:`nginx.conf` | 4 | sticky routing by province header |
| `sfid/backend/src/chain/events.rs`(新建) | 4 | 链事件订阅,同步 sheng signer cache |
| `sfid/backend/tools/load_test/`(新建) | 4 | 自写 Rust 压测脚本 |

---

## 下一步行动

1. **本任务卡**(`sheng-admin-per-province-keyring`)= Phase 1,先做
2. 完成后起 Phase 2 任务卡:`20260410-sfid-store-shard-by-province`
3. 按节奏推进 Phase 3 / Phase 4
4. Phase 5 根据实际压测数据决定

**建议起始信号**:本任务卡完成后,先用自写压测脚本跑一次 Phase 1 → 实测并发上限,
对照 Phase 2 的预期收益决定节奏(是否需要加急 Phase 2/3)。

## 参考

- 任务卡 `20260409-sfid-sheng-admin-per-province-keyring`(Phase 1 落地细节)
- 铁律:`feedback_chain_in_dev.md` / `feedback_no_chain_restart.md` /
  `feedback_sfid_pow_chain_recipe.md` / `feedback_chainspec_frozen.md` /
  `feedback_sfid_three_roles_naming.md` / `feedback_scope_auto_filter.md`
