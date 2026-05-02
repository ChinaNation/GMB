// 中文注释:Phase 2 Day 1 —— ShardBackend 持久化抽象。
//
// 定义 ShardedStore 使用的持久化接口。Day 1 只提供 trait 和一个
// 内存 MockShardBackend(仅 #[cfg(test)]),用于驱动 ShardedStore 的
// 单元测试。真正的 PostgresShardBackend 在 Day 2 再接入,不在本次
// 范围内。
//
// trait 的所有方法都是 async,因为将来 Postgres 后端要走 tokio-postgres
// 异步查询。MockShardBackend 通过 serde_json 往返一次,既验证结构体
// 的 Serialize/Deserialize 能对称,也模拟真实后端的 JSONB 行为。

use async_trait::async_trait;

use super::shard_types::{GlobalShard, StoreShard};

#[async_trait]
pub(crate) trait ShardBackend: Send + Sync {
    /// 加载指定省份的分片;不存在返回 Ok(None)。
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String>;

    /// 持久化指定省份的分片(upsert 语义)。
    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String>;

    /// 加载 GlobalShard;不存在返回 `GlobalShard::default()`。
    async fn load_global(&self) -> Result<GlobalShard, String>;

    /// 持久化 GlobalShard(upsert)。
    async fn save_global(&self, global: &GlobalShard) -> Result<(), String>;

    /// 列出所有分片 key(含 "global"),用于启动预加载。
    async fn list_shard_keys(&self) -> Result<Vec<String>, String>;
}

// ─────────────────────────────────────────────────────────────
// MockShardBackend:内存实现,仅测试用。
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) struct MockShardBackend {
    inner: std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>,
}

#[cfg(test)]
impl MockShardBackend {
    pub(crate) fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl ShardBackend for MockShardBackend {
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String> {
        let guard = self.inner.lock().map_err(|_| "mock poisoned".to_string())?;
        match guard.get(province) {
            Some(v) => {
                let shard: StoreShard = serde_json::from_value(v.clone())
                    .map_err(|e| format!("mock load_shard json: {e}"))?;
                Ok(Some(shard))
            }
            None => Ok(None),
        }
    }

    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String> {
        let payload =
            serde_json::to_value(shard).map_err(|e| format!("mock save_shard json: {e}"))?;
        let mut guard = self.inner.lock().map_err(|_| "mock poisoned".to_string())?;
        guard.insert(province.to_string(), payload);
        Ok(())
    }

    async fn load_global(&self) -> Result<GlobalShard, String> {
        let guard = self.inner.lock().map_err(|_| "mock poisoned".to_string())?;
        match guard.get("global") {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| format!("mock load_global json: {e}"))
            }
            None => Ok(GlobalShard::default()),
        }
    }

    async fn save_global(&self, global: &GlobalShard) -> Result<(), String> {
        let payload =
            serde_json::to_value(global).map_err(|e| format!("mock save_global json: {e}"))?;
        let mut guard = self.inner.lock().map_err(|_| "mock poisoned".to_string())?;
        guard.insert("global".to_string(), payload);
        Ok(())
    }

    async fn list_shard_keys(&self) -> Result<Vec<String>, String> {
        let guard = self.inner.lock().map_err(|_| "mock poisoned".to_string())?;
        let mut keys: Vec<String> = guard.keys().cloned().collect();
        keys.sort();
        Ok(keys)
    }
}
