// 中文注释:ShardedStore 的后端抽象。
//
// 当前 ShardedStore 只作为进程内分片缓存使用,不把分片整包写入 Postgres。
// 主数据由 SFID 各模块 Store 独立落库;这里保留 trait 是为了让 handler
// 继续按省读写缓存。

use async_trait::async_trait;

use super::shard_types::{GlobalShard, StoreShard};

#[async_trait]
pub(crate) trait ShardBackend: Send + Sync {
    /// 加载指定省份的分片;不存在返回 Ok(None)。
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String>;

    /// 写回指定省份的进程内分片。
    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String>;

    /// 加载 GlobalShard;不存在返回 `GlobalShard::default()`。
    async fn load_global(&self) -> Result<GlobalShard, String>;

    /// 写回进程内 GlobalShard。
    async fn save_global(&self, global: &GlobalShard) -> Result<(), String>;
}

// ─────────────────────────────────────────────────────────────
// MemoryShardBackend:进程内实现。
// ─────────────────────────────────────────────────────────────

pub(crate) struct MemoryShardBackend {
    inner: std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>,
}

impl MemoryShardBackend {
    pub(crate) fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl ShardBackend for MemoryShardBackend {
    async fn load_shard(&self, province: &str) -> Result<Option<StoreShard>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "memory shard backend poisoned".to_string())?;
        match guard.get(province) {
            Some(v) => {
                let shard: StoreShard = serde_json::from_value(v.clone())
                    .map_err(|e| format!("memory load_shard json: {e}"))?;
                Ok(Some(shard))
            }
            None => Ok(None),
        }
    }

    async fn save_shard(&self, province: &str, shard: &StoreShard) -> Result<(), String> {
        let payload =
            serde_json::to_value(shard).map_err(|e| format!("memory save_shard json: {e}"))?;
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "memory shard backend poisoned".to_string())?;
        guard.insert(province.to_string(), payload);
        Ok(())
    }

    async fn load_global(&self) -> Result<GlobalShard, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "memory shard backend poisoned".to_string())?;
        match guard.get("global") {
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| format!("memory load_global json: {e}")),
            None => Ok(GlobalShard::default()),
        }
    }

    async fn save_global(&self, global: &GlobalShard) -> Result<(), String> {
        let payload =
            serde_json::to_value(global).map_err(|e| format!("memory save_global json: {e}"))?;
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "memory shard backend poisoned".to_string())?;
        guard.insert("global".to_string(), payload);
        Ok(())
    }
}

#[cfg(test)]
pub(crate) type MockShardBackend = MemoryShardBackend;
