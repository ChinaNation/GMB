// 中文注释:Phase 2 Day 2 —— ShardBackend 的 Postgres 实现。
//
// 任务卡 `20260410-sfid-store-shard-by-province` Phase 2 Day 2:
//   - 所有分片都存在单表 `store_shards(shard_key TEXT PK, payload JSONB, updated_at, version)`
//   - 省分片 shard_key = 省名 UTF-8;GlobalShard shard_key = "global"
//   - UPSERT 语义保证幂等,version 由服务器端 +1
//
// 重要:现有后端(sfid-backend)复用了 `postgres = "*"` 同步 crate,
// 并把若干个 `postgres::Client` 放在 `Arc<Vec<Mutex<Client>>>` 池里轮询。
// 为了不引入新 runtime(tokio-postgres / sqlx 一律不加),
// PostgresShardBackend 直接复用同一个池,在 async 方法里用
// `tokio::task::spawn_blocking` 包装同步调用,保持 executor 不被阻塞。

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use async_trait::async_trait;

use crate::store_shards::{
    backend::ShardBackend,
    shard_types::{GlobalShard, StoreShard},
};

/// Postgres 后端。持有对现有连接池的共享引用,不自己管理连接生命周期。
pub(crate) struct PostgresShardBackend {
    clients: Arc<Vec<Mutex<postgres::Client>>>,
    next_idx: Arc<AtomicUsize>,
}

impl PostgresShardBackend {
    pub(crate) fn new(
        clients: Arc<Vec<Mutex<postgres::Client>>>,
        next_idx: Arc<AtomicUsize>,
    ) -> Self {
        Self { clients, next_idx }
    }

    /// 在阻塞线程里取一个池内连接执行闭包。
    async fn run_blocking<F, R>(&self, op: F) -> Result<R, String>
    where
        F: FnOnce(&mut postgres::Client) -> Result<R, String> + Send + 'static,
        R: Send + 'static,
    {
        let clients = self.clients.clone();
        let next_idx = self.next_idx.clone();
        tokio::task::spawn_blocking(move || {
            if clients.is_empty() {
                return Err("postgres client pool is empty".to_string());
            }
            let idx = next_idx.fetch_add(1, Ordering::Relaxed) % clients.len();
            let mut conn = clients[idx]
                .lock()
                .map_err(|_| "postgres client lock poisoned".to_string())?;
            op(&mut conn)
        })
        .await
        .map_err(|e| format!("shard backend join: {e}"))?
    }
}

#[async_trait]
impl ShardBackend for PostgresShardBackend {
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String> {
        let key = province.to_string();
        self.run_blocking(move |conn| {
            let row_opt = conn
                .query_opt(
                    "SELECT payload FROM store_shards WHERE shard_key = $1",
                    &[&key],
                )
                .map_err(|e| format!("load_shard sql: {e}"))?;
            match row_opt {
                Some(row) => {
                    let payload: serde_json::Value = row.get(0);
                    let shard: StoreShard = serde_json::from_value(payload)
                        .map_err(|e| format!("load_shard json: {e}"))?;
                    Ok(Some(shard))
                }
                None => Ok(None),
            }
        })
        .await
    }

    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String> {
        let payload =
            serde_json::to_value(shard).map_err(|e| format!("save_shard json: {e}"))?;
        let key = province.to_string();
        self.run_blocking(move |conn| {
            conn.execute(
                "INSERT INTO store_shards (shard_key, payload, updated_at, version)
                 VALUES ($1, $2, now(), 1)
                 ON CONFLICT (shard_key) DO UPDATE SET
                     payload = EXCLUDED.payload,
                     updated_at = now(),
                     version = store_shards.version + 1",
                &[&key, &payload],
            )
            .map_err(|e| format!("save_shard sql: {e}"))?;
            Ok(())
        })
        .await
    }

    async fn load_global(&self) -> Result<GlobalShard, String> {
        self.run_blocking(|conn| {
            let row_opt = conn
                .query_opt(
                    "SELECT payload FROM store_shards WHERE shard_key = 'global'",
                    &[],
                )
                .map_err(|e| format!("load_global sql: {e}"))?;
            match row_opt {
                Some(row) => {
                    let payload: serde_json::Value = row.get(0);
                    let global: GlobalShard = serde_json::from_value(payload)
                        .map_err(|e| format!("load_global json: {e}"))?;
                    Ok(global)
                }
                None => Ok(GlobalShard::default()),
            }
        })
        .await
    }

    async fn save_global(&self, global: &GlobalShard) -> Result<(), String> {
        let payload =
            serde_json::to_value(global).map_err(|e| format!("save_global json: {e}"))?;
        self.run_blocking(move |conn| {
            conn.execute(
                "INSERT INTO store_shards (shard_key, payload, updated_at, version)
                 VALUES ('global', $1, now(), 1)
                 ON CONFLICT (shard_key) DO UPDATE SET
                     payload = EXCLUDED.payload,
                     updated_at = now(),
                     version = store_shards.version + 1",
                &[&payload],
            )
            .map_err(|e| format!("save_global sql: {e}"))?;
            Ok(())
        })
        .await
    }

    async fn list_shard_keys(&self) -> Result<Vec<String>, String> {
        self.run_blocking(|conn| {
            let rows = conn
                .query(
                    "SELECT shard_key FROM store_shards ORDER BY shard_key",
                    &[],
                )
                .map_err(|e| format!("list_shard_keys sql: {e}"))?;
            Ok(rows.into_iter().map(|r| r.get::<_, String>(0)).collect())
        })
        .await
    }
}
