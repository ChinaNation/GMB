// 中文注释:Phase 2 Day 1 —— ShardedStore 核心访问 API。
//
// 本模块提供按省分片的 Store 访问入口:
//   - `DashMap<province, Arc<RwLock<StoreShard>>>` 实现省级并发隔离;
//   - `Arc<RwLock<GlobalShard>>` 承载跨省共享状态;
//   - `ShardBackend` 抽象让持久化可插拔(Day 1 mock,Day 2 Postgres)。
//
// 关键并发约束(impl.md 4.1 节):
//   1. 闭包 `f` 一律是同步的,不能 await;锁持有时间必须尽量短。
//   2. 写入闭包执行完立即 drop RwLock guard,再 await 持久化,避免
//      持有锁跨 .await 造成死锁 / 卡死 tokio executor。
//   3. `get_or_load_shard` 不能在 DashMap `entry` 闭包里 `.await`;
//      改为先 `get` 试一次,miss 则先 await 加载,再 `entry().or_insert`
//      把加载结果填回(别的线程可能先填好,以它为准)。
//   4. `for_each_province` 先收集快照再回调,避免跨 await 拿 guard。

pub(crate) mod backend;
pub(crate) mod migration;
pub(crate) mod pg_backend;
pub(crate) mod shard_types;

use std::sync::{Arc, RwLock};

use dashmap::DashMap;

pub(crate) use backend::ShardBackend;
pub(crate) use shard_types::{GlobalShard, StoreShard};

pub(crate) struct ShardedStore {
    shards: DashMap<String, Arc<RwLock<StoreShard>>>,
    global: Arc<RwLock<GlobalShard>>,
    backend: Arc<dyn ShardBackend>,
    /// 过渡期双写开关(Phase 2 中);Day 1 只存,不用。
    #[allow(dead_code)]
    double_write: bool,
}

impl ShardedStore {
    pub(crate) fn new(backend: Arc<dyn ShardBackend>, double_write: bool) -> Self {
        Self {
            shards: DashMap::new(),
            global: Arc::new(RwLock::new(GlobalShard::default())),
            backend,
            double_write,
        }
    }

    /// 启动时加载 GlobalShard(必须成功,不存在则留空)。
    pub(crate) async fn bootstrap_global(&self) -> Result<(), String> {
        let global = self.backend.load_global().await?;
        let mut guard = self
            .global
            .write()
            .map_err(|_| "global poisoned".to_string())?;
        *guard = global;
        Ok(())
    }

    /// 启动时预加载所有省份分片。返回成功加载的分片数(不含 global)。
    pub(crate) async fn preload_all_shards(&self) -> Result<usize, String> {
        let keys = self.backend.list_shard_keys().await?;
        let mut count = 0usize;
        for key in keys {
            if key == "global" {
                continue;
            }
            if let Some(shard) = self.backend.load_shard(&key).await? {
                self.shards
                    .insert(key.clone(), Arc::new(RwLock::new(shard)));
                count += 1;
            }
        }
        Ok(count)
    }

    /// 读本省(懒加载)。
    pub(crate) async fn read_province<F, R>(&self, province: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&StoreShard) -> R,
    {
        let shard = self.get_or_load_shard(province).await?;
        let guard = shard
            .read()
            .map_err(|_| format!("shard {province} poisoned"))?;
        Ok(f(&*guard))
    }

    /// 写本省 + 写穿透持久化。
    pub(crate) async fn write_province<F, R>(&self, province: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut StoreShard) -> R,
    {
        let shard = self.get_or_load_shard(province).await?;
        // 锁内只做修改 + 版本号递增,立即释放。
        let result = {
            let mut guard = shard
                .write()
                .map_err(|_| format!("shard {province} poisoned"))?;
            if guard.province.is_empty() {
                guard.province = province.to_string();
            }
            guard.version += 1;
            f(&mut *guard)
        };
        // 释放锁后再持久化,避免跨 await 持锁。
        self.persist_shard(province).await?;
        Ok(result)
    }

    /// 读全局状态(同步,不需要 async)。
    pub(crate) fn read_global<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&GlobalShard) -> R,
    {
        let guard = self.global.read().map_err(|_| "global poisoned".to_string())?;
        Ok(f(&*guard))
    }

    /// 写全局状态 + 写穿透。
    pub(crate) async fn write_global<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut GlobalShard) -> R,
    {
        let result = {
            let mut guard = self
                .global
                .write()
                .map_err(|_| "global poisoned".to_string())?;
            guard.version += 1;
            f(&mut *guard)
        };
        self.persist_global().await?;
        Ok(result)
    }

    /// 遍历所有已加载省份分片;回调内不能再调用 ShardedStore 的写方法。
    /// 为了避免跨 await 持锁,这里先拷出快照再调用 callback。
    pub(crate) async fn for_each_province<F>(&self, mut f: F) -> Result<(), String>
    where
        F: FnMut(&str, &StoreShard),
    {
        // 先拿到所有 province key 的快照,避免迭代时 DashMap 被写入导致死锁。
        let provinces: Vec<String> = self
            .shards
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        for province in provinces {
            let arc = match self.shards.get(&province) {
                Some(e) => e.value().clone(),
                None => continue,
            };
            let guard = arc
                .read()
                .map_err(|_| format!("shard {province} poisoned"))?;
            f(&province, &*guard);
        }
        Ok(())
    }

    /// 获取或懒加载分片。
    ///
    /// 注意不能在 DashMap 的 entry 闭包里 .await。做法:
    ///   1. 先非阻塞 `get` 一次,命中直接返回;
    ///   2. miss 则 await backend.load_shard;
    ///   3. 不管加载结果有没有,最后 `entry().or_insert_with(...)`
    ///      给一个默认值(province 字段填好);
    ///   4. 如果 backend 里有数据,把它 overwrite 进 RwLock。
    async fn get_or_load_shard(
        &self,
        province: &str,
    ) -> Result<Arc<RwLock<StoreShard>>, String> {
        if let Some(entry) = self.shards.get(province) {
            return Ok(entry.value().clone());
        }

        // miss —— 先从 backend 加载(可能为 None)
        let loaded = self.backend.load_shard(province).await?;

        // 回填到 DashMap。竞争条件:别的线程可能已经插入,以它为准。
        let arc = self
            .shards
            .entry(province.to_string())
            .or_insert_with(|| {
                let mut init = StoreShard::default();
                init.province = province.to_string();
                Arc::new(RwLock::new(init))
            })
            .value()
            .clone();

        // 如果后端有数据,把它写进锁内(覆盖刚才 or_insert 的空壳)。
        if let Some(shard) = loaded {
            let mut guard = arc
                .write()
                .map_err(|_| format!("shard {province} poisoned"))?;
            // 只有版本为 0(刚 or_insert 的空壳)才覆盖,避免踩别的线程写入。
            if guard.version == 0 && guard.citizen_records.is_empty() {
                *guard = shard;
            }
        }

        Ok(arc)
    }

    async fn persist_shard(&self, province: &str) -> Result<(), String> {
        // 短暂拿 read lock 做克隆快照,立即释放,再走 await。
        let snapshot = {
            let arc = self
                .shards
                .get(province)
                .ok_or_else(|| format!("shard {province} not found"))?
                .value()
                .clone();
            let guard = arc
                .read()
                .map_err(|_| format!("shard {province} poisoned"))?;
            guard.clone()
        };
        self.backend.save_shard(province, &snapshot).await
    }

    async fn persist_global(&self) -> Result<(), String> {
        let snapshot = {
            let guard = self
                .global
                .read()
                .map_err(|_| "global poisoned".to_string())?;
            guard.clone()
        };
        self.backend.save_global(&snapshot).await
    }
}

// ─────────────────────────────────────────────────────────────
// 单元测试
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::backend::MockShardBackend;
    use super::*;

    fn new_store() -> ShardedStore {
        let backend: Arc<dyn ShardBackend> = Arc::new(MockShardBackend::new());
        ShardedStore::new(backend, false)
    }

    #[tokio::test]
    async fn test_read_write_province_roundtrip() {
        let store = new_store();
        store
            .write_province("广东省", |s| {
                s.next_citizen_id = 42;
                s.generated_sfid_by_pubkey
                    .insert("pk1".into(), "sfid1".into());
            })
            .await
            .unwrap();

        let (id, sfid) = store
            .read_province("广东省", |s| {
                (
                    s.next_citizen_id,
                    s.generated_sfid_by_pubkey.get("pk1").cloned(),
                )
            })
            .await
            .unwrap();
        assert_eq!(id, 42);
        assert_eq!(sfid.as_deref(), Some("sfid1"));
    }

    #[tokio::test]
    async fn test_lazy_load_from_backend() {
        let backend = Arc::new(MockShardBackend::new());
        // 先直接往 backend 里塞一条
        {
            let mut shard = StoreShard::default();
            shard.province = "江苏省".into();
            shard.next_citizen_id = 7;
            shard.version = 1;
            backend.save_shard("江苏省", &shard).await.unwrap();
        }
        let store = ShardedStore::new(backend.clone() as Arc<dyn ShardBackend>, false);
        // 新 store 完全没加载,read_province 应能走懒加载拿到 7
        let id = store
            .read_province("江苏省", |s| s.next_citizen_id)
            .await
            .unwrap();
        assert_eq!(id, 7);
    }

    #[tokio::test]
    async fn test_concurrent_read_same_province() {
        let store = Arc::new(new_store());
        store
            .write_province("浙江省", |s| s.next_citizen_id = 100)
            .await
            .unwrap();

        let mut handles = Vec::new();
        for _ in 0..16 {
            let s = store.clone();
            handles.push(tokio::spawn(async move {
                s.read_province("浙江省", |sh| sh.next_citizen_id)
                    .await
                    .unwrap()
            }));
        }
        for h in handles {
            assert_eq!(h.await.unwrap(), 100);
        }
    }

    #[tokio::test]
    async fn test_concurrent_write_different_provinces() {
        let store = Arc::new(new_store());
        let a = {
            let s = store.clone();
            tokio::spawn(async move {
                s.write_province("A省", |sh| sh.next_citizen_id = 1)
                    .await
                    .unwrap();
            })
        };
        let b = {
            let s = store.clone();
            tokio::spawn(async move {
                s.write_province("B省", |sh| sh.next_citizen_id = 2)
                    .await
                    .unwrap();
            })
        };
        a.await.unwrap();
        b.await.unwrap();
        assert_eq!(
            store.read_province("A省", |s| s.next_citizen_id).await.unwrap(),
            1
        );
        assert_eq!(
            store.read_province("B省", |s| s.next_citizen_id).await.unwrap(),
            2
        );
    }

    #[tokio::test]
    async fn test_version_auto_increment() {
        let store = new_store();
        store.write_province("云南省", |_| {}).await.unwrap();
        store.write_province("云南省", |_| {}).await.unwrap();
        store.write_province("云南省", |_| {}).await.unwrap();
        let v = store
            .read_province("云南省", |s| s.version)
            .await
            .unwrap();
        assert_eq!(v, 3);

        store.write_global(|_| {}).await.unwrap();
        store.write_global(|_| {}).await.unwrap();
        let gv = store.read_global(|g| g.version).unwrap();
        assert_eq!(gv, 2);
    }

    #[tokio::test]
    async fn test_for_each_province() {
        let store = new_store();
        store.write_province("甲省", |s| s.next_citizen_id = 1).await.unwrap();
        store.write_province("乙省", |s| s.next_citizen_id = 2).await.unwrap();

        let mut seen: Vec<(String, u64)> = Vec::new();
        store
            .for_each_province(|p, s| seen.push((p.to_string(), s.next_citizen_id)))
            .await
            .unwrap();
        seen.sort();
        assert_eq!(seen, vec![("乙省".into(), 2), ("甲省".into(), 1)]);
    }

    #[tokio::test]
    async fn test_read_write_global_roundtrip() {
        let store = new_store();
        store
            .write_global(|g| {
                g.anon_rsa_private_key_pem = Some("pem-data".into());
            })
            .await
            .unwrap();
        let pem = store
            .read_global(|g| g.anon_rsa_private_key_pem.clone())
            .unwrap();
        assert_eq!(pem.as_deref(), Some("pem-data"));
    }

    #[tokio::test]
    async fn test_preload_all_shards() {
        let backend = Arc::new(MockShardBackend::new());
        for p in ["川省", "鄂省", "赣省"] {
            let mut s = StoreShard::default();
            s.province = p.into();
            s.version = 1;
            backend.save_shard(p, &s).await.unwrap();
        }
        let store = ShardedStore::new(backend as Arc<dyn ShardBackend>, false);
        let n = store.preload_all_shards().await.unwrap();
        assert_eq!(n, 3);
        // 再 read 一次确认已加载(不会再打 backend)
        let v = store.read_province("川省", |s| s.version).await.unwrap();
        assert_eq!(v, 1);
    }
}
