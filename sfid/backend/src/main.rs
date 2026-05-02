use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use postgres::config::Host;
use redis::Client as RedisClient;
use sp_core::Pair;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
};
use tracing::{error, info, warn};
use uuid::Uuid;

mod app_core;
mod business;
mod chain;
mod indexer;
mod institutions;
#[path = "key-admins/mod.rs"]
mod key_admins;
mod login;
mod models;
mod operate;
#[allow(dead_code)]
mod qr;
mod scope;
mod sfid;
mod sheng_admins;
mod shi_admins;
mod store_shards;
use business::scope::in_scope_cpms_site;
use key_admins::chain_keyring::ChainKeyringState;

pub(crate) use app_core::http_security::*;
pub(crate) use app_core::runtime_ops::*;
pub(crate) use login::{
    build_admin_display_name, parse_sr25519_pubkey, parse_sr25519_pubkey_bytes, require_admin_any,
    require_admin_write, require_institution_or_key_admin, require_key_admin,
    verify_admin_signature,
};
pub(crate) use models::*;

#[derive(Clone)]
struct AppState {
    store: StoreHandle,
    signing_seed_hex: Arc<RwLock<SensitiveSeed>>,
    known_key_seeds: Arc<RwLock<HashMap<String, SensitiveSeed>>>,
    rate_limit_redis: Arc<RedisClient>,
    #[allow(dead_code)]
    cpms_register_inflight: Arc<Mutex<HashMap<String, std::time::Instant>>>,
    key_id: String,
    key_version: String,
    key_alg: String,
    public_key_hex: Arc<RwLock<String>>,
    /// 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B：
    /// 省级签名密钥内存缓存 + SFID MAIN 派生的 wrap key。
    pub(crate) sheng_signer_cache: Arc<key_admins::sheng_signer_cache::ShengSignerCache>,
    /// 任务卡 `20260410-sfid-store-shard-by-province` Phase 2 Day 2:
    /// 按省分片的新 Store。此轮只构造 + 迁移,handler 仍走 legacy `store`。
    #[allow(dead_code)]
    pub(crate) sharded_store: Arc<store_shards::ShardedStore>,
}

#[derive(Clone)]
struct StoreHandle {
    backend: StoreBackend,
    write_gate: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Clone)]
#[allow(dead_code)]
enum StoreBackend {
    Memory(Arc<RwLock<Store>>),
    Postgres {
        clients: Arc<Vec<Mutex<postgres::Client>>>,
        next_client_idx: Arc<AtomicUsize>,
    },
}

struct StoreReadGuard {
    store: Store,
}

struct StoreWriteGuard {
    store: Store,
    backend: StoreBackend,
    _write_guard: tokio::sync::OwnedMutexGuard<()>,
}

impl std::ops::Deref for StoreReadGuard {
    type Target = Store;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl std::ops::Deref for StoreWriteGuard {
    type Target = Store;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl std::ops::DerefMut for StoreWriteGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.store
    }
}

impl Drop for StoreWriteGuard {
    fn drop(&mut self) {
        if let Err(err) = self.backend.save_store(&self.store) {
            // 持久化失败是严重事件:数据可能丢失。升级为 error! 并计入 metrics。
            error!(error = %err, "CRITICAL: failed to persist store to database — data may be lost on restart");
            self.store.metrics.store_persist_failures += 1;
        }
    }
}

impl StoreBackend {
    fn with_postgres_client<R>(
        clients: &Arc<Vec<Mutex<postgres::Client>>>,
        next_client_idx: &Arc<AtomicUsize>,
        op: impl FnOnce(&mut postgres::Client) -> Result<R, String> + Send,
    ) -> Result<R, String>
    where
        R: Send,
    {
        if clients.is_empty() {
            return Err("postgres client pool is empty".to_string());
        }
        let idx = next_client_idx.fetch_add(1, Ordering::Relaxed) % clients.len();
        let selected = Arc::clone(clients);
        thread::scope(|scope| {
            let handle = scope.spawn(|| {
                let mut conn = selected[idx]
                    .lock()
                    .map_err(|_| "postgres client lock poisoned".to_string())?;
                op(&mut conn)
            });
            match handle.join() {
                Ok(v) => v,
                Err(_) => Err("postgres worker thread panicked".to_string()),
            }
        })
    }

    fn parse_admin_role(role: &str) -> AdminRole {
        // 中文注释：统一到 key/sheng/shi 三角色(见 feedback_sfid_three_roles_naming.md)。
        match role {
            "KEY_ADMIN" => AdminRole::KeyAdmin,
            "SHENG_ADMIN" => AdminRole::ShengAdmin,
            "SHI_ADMIN" => AdminRole::ShiAdmin,
            _ => AdminRole::ShiAdmin,
        }
    }

    fn parse_admin_status(status: &str) -> AdminStatus {
        match status {
            "DISABLED" => AdminStatus::Disabled,
            _ => AdminStatus::Active,
        }
    }

    fn admin_role_text(role: &AdminRole) -> &'static str {
        match role {
            AdminRole::KeyAdmin => "KEY_ADMIN",
            AdminRole::ShengAdmin => "SHENG_ADMIN",
            AdminRole::ShiAdmin => "SHI_ADMIN",
        }
    }

    fn admin_status_text(status: &AdminStatus) -> &'static str {
        match status {
            AdminStatus::Active => "ACTIVE",
            AdminStatus::Disabled => "DISABLED",
        }
    }

    fn load_store_postgres(conn: &mut postgres::Client) -> Result<Store, String> {
        let mut store = {
            let cache_rows = conn
                .query("SELECT entry_key, payload FROM runtime_cache_entries", &[])
                .map_err(|e| format!("load runtime_cache_entries failed: {e}"))?;
            if !cache_rows.is_empty() {
                let mut payload_map = serde_json::Map::new();
                for row in cache_rows {
                    let entry_key: String = row.get(0);
                    let payload: serde_json::Value = row.get(1);
                    payload_map.insert(entry_key, payload);
                }
                serde_json::from_value(serde_json::Value::Object(payload_map))
                    .map_err(|e| format!("decode runtime_cache_entries failed: {e}"))?
            } else {
                let row = conn
                    .query_opt("SELECT payload FROM runtime_misc WHERE id=1", &[])
                    .map_err(|e| format!("load runtime_misc failed: {e}"))?;
                if let Some(row) = row {
                    let payload: serde_json::Value = row.get(0);
                    serde_json::from_value(payload)
                        .map_err(|e| format!("decode runtime_misc failed: {e}"))?
                } else {
                    Store::default()
                }
            }
        };

        store.admin_users_by_pubkey.clear();
        store.sheng_admin_province_by_pubkey.clear();
        store.chain_keyring_state = None;

        let admin_rows = conn
            .query(
                "SELECT admin_id, admin_pubkey, admin_name, role, status, built_in, created_by, created_at, updated_at, city, encrypted_signing_privkey, signing_pubkey, signing_created_at
                 FROM admins",
                &[],
            )
            .map_err(|e| format!("load admins failed: {e}"))?;
        for row in admin_rows {
            let id: i64 = row.get(0);
            let admin_pubkey: String = row.get(1);
            let admin_name: String = row.get(2);
            let role_text: String = row.get(3);
            let status_text: String = row.get(4);
            let built_in: bool = row.get(5);
            let created_by: String = row.get(6);
            let created_at: DateTime<Utc> = row.get(7);
            let updated_at: Option<DateTime<Utc>> = row.get(8);
            let city: String = row.get(9);
            let encrypted_signing_privkey: Option<String> = row.get(10);
            let signing_pubkey: Option<String> = row.get(11);
            let signing_created_at: Option<DateTime<Utc>> = row.get(12);
            store.admin_users_by_pubkey.insert(
                admin_pubkey.clone(),
                AdminUser {
                    id: u64::try_from(id).unwrap_or(0),
                    admin_pubkey,
                    admin_name,
                    role: Self::parse_admin_role(role_text.as_str()),
                    status: Self::parse_admin_status(status_text.as_str()),
                    built_in,
                    created_by,
                    created_at,
                    updated_at,
                    city,
                    encrypted_signing_privkey,
                    signing_pubkey,
                    signing_created_at,
                },
            );
        }

        let super_rows = conn
            .query(
                "SELECT a.admin_pubkey, s.province_name
                 FROM sheng_admin_scope s
                 JOIN admins a ON a.admin_id=s.admin_id",
                &[],
            )
            .map_err(|e| format!("load sheng_admin_scope failed: {e}"))?;
        for row in super_rows {
            let pubkey: String = row.get(0);
            let province: String = row.get(1);
            store
                .sheng_admin_province_by_pubkey
                .insert(pubkey, province);
        }

        let key_rows = conn
            .query(
                "SELECT slot, admin_pubkey, keyring_version, updated_at
                 FROM key_admin_keyring",
                &[],
            )
            .map_err(|e| format!("load key_admin_keyring failed: {e}"))?;
        let mut main_pubkey = String::new();
        let mut backup_a_pubkey = String::new();
        let mut backup_b_pubkey = String::new();
        let mut version = 1_u64;
        let mut latest_updated_at: Option<DateTime<Utc>> = None;
        for row in key_rows {
            let slot: String = row.get(0);
            let pubkey: String = row.get(1);
            let keyring_version: i64 = row.get(2);
            let updated_at: DateTime<Utc> = row.get(3);
            if keyring_version > 0 {
                version = u64::try_from(keyring_version).unwrap_or(version);
            }
            if latest_updated_at.map(|v| updated_at > v).unwrap_or(true) {
                latest_updated_at = Some(updated_at);
            }
            match slot.as_str() {
                "MAIN" => main_pubkey = pubkey,
                "BACKUP_A" => backup_a_pubkey = pubkey,
                "BACKUP_B" => backup_b_pubkey = pubkey,
                _ => {}
            }
        }
        if !main_pubkey.is_empty() && !backup_a_pubkey.is_empty() && !backup_b_pubkey.is_empty() {
            let mut kr = ChainKeyringState::new(main_pubkey, backup_a_pubkey, backup_b_pubkey);
            kr.version = version;
            if let Some(updated_at) = latest_updated_at {
                kr.updated_at = updated_at.timestamp();
            }
            store.chain_keyring_state = Some(kr);
        }

        Ok(store)
    }

    fn save_store_postgres(conn: &mut postgres::Client, store: &Store) -> Result<(), String> {
        let mut misc = store.clone();
        misc.admin_users_by_pubkey.clear();
        misc.sheng_admin_province_by_pubkey.clear();
        misc.chain_keyring_state = None;
        let payload =
            serde_json::to_value(&misc).map_err(|e| format!("encode runtime cache failed: {e}"))?;
        let payload_obj = payload
            .as_object()
            .ok_or_else(|| "runtime cache payload is not an object".to_string())?;
        let mut tx = conn
            .transaction()
            .map_err(|e| format!("begin runtime cache transaction failed: {e}"))?;
        tx.execute("DELETE FROM runtime_cache_entries", &[])
            .map_err(|e| format!("clear runtime_cache_entries failed: {e}"))?;
        for (entry_key, entry_payload) in payload_obj {
            tx.execute(
                "INSERT INTO runtime_cache_entries(entry_key, payload, updated_at)
                 VALUES ($1, $2, now())",
                &[entry_key, entry_payload],
            )
            .map_err(|e| format!("save runtime cache entry {entry_key} failed: {e}"))?;
        }
        tx.execute(
            "INSERT INTO runtime_misc(id, payload, updated_at) VALUES (1, $1, now())
             ON CONFLICT (id) DO UPDATE SET payload=excluded.payload, updated_at=now()",
            &[&payload],
        )
        .map_err(|e| format!("save runtime_misc compatibility snapshot failed: {e}"))?;
        tx.commit()
            .map_err(|e| format!("commit runtime cache transaction failed: {e}"))?;

        let mut tx = conn
            .transaction()
            .map_err(|e| format!("begin admin sync transaction failed: {e}"))?;
        tx.execute("DELETE FROM key_admin_keyring", &[])
            .map_err(|e| format!("clear key_admin_keyring failed: {e}"))?;
        tx.execute("DELETE FROM shi_admin_scope", &[])
            .map_err(|e| format!("clear shi_admin_scope failed: {e}"))?;
        tx.execute("DELETE FROM sheng_admin_scope", &[])
            .map_err(|e| format!("clear sheng_admin_scope failed: {e}"))?;
        tx.execute("DELETE FROM admins", &[])
            .map_err(|e| format!("clear admins failed: {e}"))?;

        let mut admin_id_by_pubkey: HashMap<String, i64> = HashMap::new();
        for admin in store.admin_users_by_pubkey.values() {
            let row = tx
                .query_one(
                    "INSERT INTO admins(admin_id, admin_pubkey, admin_name, role, status, built_in, created_by, created_at, updated_at, city, encrypted_signing_privkey, signing_pubkey, signing_created_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                     RETURNING admin_id",
                    &[
                        &(admin.id as i64),
                        &admin.admin_pubkey,
                        &admin.admin_name,
                        &Self::admin_role_text(&admin.role),
                        &Self::admin_status_text(&admin.status),
                        &admin.built_in,
                        &admin.created_by,
                        &admin.created_at,
                        &admin.updated_at.unwrap_or(admin.created_at),
                        &admin.city,
                        &admin.encrypted_signing_privkey,
                        &admin.signing_pubkey,
                        &admin.signing_created_at,
                    ],
                )
                .map_err(|e| format!("insert admins failed: {e}"))?;
            let admin_id: i64 = row.get(0);
            admin_id_by_pubkey.insert(admin.admin_pubkey.clone(), admin_id);
        }

        for province in store.sheng_admin_province_by_pubkey.values() {
            tx.execute(
                "INSERT INTO provinces(province_name) VALUES ($1)
                 ON CONFLICT (province_name) DO NOTHING",
                &[province],
            )
            .map_err(|e| format!("upsert provinces failed: {e}"))?;
        }

        for (pubkey, province) in &store.sheng_admin_province_by_pubkey {
            let Some(admin_id) = admin_id_by_pubkey.get(pubkey) else {
                continue;
            };
            tx.execute(
                "INSERT INTO sheng_admin_scope(admin_id, province_name) VALUES ($1, $2)",
                &[admin_id, province],
            )
            .map_err(|e| format!("insert sheng_admin_scope failed: {e}"))?;
        }

        for admin in store.admin_users_by_pubkey.values() {
            if admin.role != AdminRole::ShiAdmin {
                continue;
            }
            let Some(admin_id) = admin_id_by_pubkey.get(&admin.admin_pubkey) else {
                continue;
            };
            let Some(sheng_admin_id) = admin_id_by_pubkey.get(&admin.created_by) else {
                continue;
            };
            let province = store
                .sheng_admin_province_by_pubkey
                .get(&admin.created_by)
                .cloned();
            tx.execute(
                "INSERT INTO shi_admin_scope(admin_id, sheng_admin_id, province_name)
                 VALUES ($1, $2, $3)",
                &[admin_id, sheng_admin_id, &province],
            )
            .map_err(|e| format!("insert shi_admin_scope failed: {e}"))?;
        }

        if let Some(kr) = &store.chain_keyring_state {
            let slots = [
                ("MAIN", kr.main_pubkey.as_str()),
                ("BACKUP_A", kr.backup_a_pubkey.as_str()),
                ("BACKUP_B", kr.backup_b_pubkey.as_str()),
            ];
            for (slot, pubkey) in slots {
                let Some(admin_id) = admin_id_by_pubkey.get(pubkey) else {
                    continue;
                };
                tx.execute(
                    "INSERT INTO key_admin_keyring(slot, admin_id, admin_pubkey, keyring_version, updated_at)
                     VALUES ($1, $2, $3, $4, to_timestamp($5))
                     ON CONFLICT (slot) DO UPDATE SET
                       admin_id=excluded.admin_id,
                       admin_pubkey=excluded.admin_pubkey,
                       keyring_version=excluded.keyring_version,
                       updated_at=excluded.updated_at",
                    &[&slot, admin_id, &pubkey, &(kr.version as i64), &(kr.updated_at as f64)],
                )
                .map_err(|e| format!("upsert key_admin_keyring failed: {e}"))?;
            }
        }

        tx.commit()
            .map_err(|e| format!("commit admin sync transaction failed: {e}"))?;
        Ok(())
    }

    fn load_store(&self) -> Result<Store, String> {
        match self {
            Self::Memory(mem) => mem
                .read()
                .map(|v| v.clone())
                .map_err(|_| "memory store read lock poisoned".to_string()),
            Self::Postgres {
                clients,
                next_client_idx,
            } => Self::with_postgres_client(clients, next_client_idx, Self::load_store_postgres),
        }
    }

    fn save_store(&self, store: &Store) -> Result<(), String> {
        match self {
            Self::Memory(mem) => {
                let mut guard = mem
                    .write()
                    .map_err(|_| "memory store write lock poisoned".to_string())?;
                *guard = store.clone();
                Ok(())
            }
            Self::Postgres {
                clients,
                next_client_idx,
            } => {
                let snapshot = store.clone();
                Self::with_postgres_client(clients, next_client_idx, move |conn| {
                    Self::save_store_postgres(conn, &snapshot)
                })?;
                Ok(())
            }
        }
    }
}

impl StoreHandle {
    #[allow(dead_code)]
    fn in_memory() -> Self {
        Self {
            backend: StoreBackend::Memory(Arc::new(RwLock::new(Store::default()))),
            write_gate: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    fn from_database_url(database_url: &str) -> Result<Self, String> {
        let db_url = database_url.to_string();
        let pool_size = std::env::var("SFID_PG_POOL_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(4);
        let handle = thread::spawn(move || {
            let mut bootstrap = postgres::Client::connect(db_url.as_str(), postgres::NoTls)
                .map_err(|e| format!("connect postgres failed: {e}"))?;
            bootstrap
                .batch_execute(
                    "CREATE TABLE IF NOT EXISTS runtime_misc (
                    id INTEGER PRIMARY KEY,
                    payload JSONB NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 );
                 CREATE TABLE IF NOT EXISTS runtime_cache_entries (
                    entry_key TEXT PRIMARY KEY,
                    payload JSONB NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 );
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS admin_name TEXT NOT NULL DEFAULT '';
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS city TEXT NOT NULL DEFAULT '';
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS encrypted_signing_privkey TEXT;
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS signing_pubkey TEXT;
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS signing_created_at TIMESTAMPTZ;
                 CREATE TABLE IF NOT EXISTS store_shards (
                    shard_key TEXT PRIMARY KEY,
                    payload JSONB NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                    version BIGINT NOT NULL DEFAULT 0
                 );
                 CREATE INDEX IF NOT EXISTS idx_store_shards_updated_at ON store_shards(updated_at);",
                )
                .map_err(|e| format!("init runtime tables failed: {e}"))?;
            let mut clients = Vec::with_capacity(pool_size);
            clients.push(Mutex::new(bootstrap));
            for _ in 1..pool_size {
                let conn = postgres::Client::connect(db_url.as_str(), postgres::NoTls)
                    .map_err(|e| format!("connect postgres pool client failed: {e}"))?;
                clients.push(Mutex::new(conn));
            }
            Ok::<Vec<Mutex<postgres::Client>>, String>(clients)
        });
        let clients = match handle.join() {
            Ok(v) => v?,
            Err(_) => return Err("postgres init thread panicked".to_string()),
        };
        Ok(Self {
            backend: StoreBackend::Postgres {
                clients: Arc::new(clients),
                next_client_idx: Arc::new(AtomicUsize::new(0)),
            },
            write_gate: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    fn read(&self) -> Result<StoreReadGuard, String> {
        Ok(StoreReadGuard {
            store: self.backend.load_store()?,
        })
    }

    /// 任务卡 `20260410-sfid-store-shard-by-province` Phase 2 Day 2:
    /// 把内部 Postgres 连接池暴露给 store_shards::pg_backend / migration,
    /// 避免新开连接池、也避免把 Phase 1 的 StoreBackend 改成 pub 字段。
    fn postgres_pool(&self) -> Option<(Arc<Vec<Mutex<postgres::Client>>>, Arc<AtomicUsize>)> {
        match &self.backend {
            StoreBackend::Postgres {
                clients,
                next_client_idx,
            } => Some((clients.clone(), next_client_idx.clone())),
            StoreBackend::Memory(_) => None,
        }
    }

    fn write(&self) -> Result<StoreWriteGuard, String> {
        let gate = self.write_gate.clone();
        let write_guard = match tokio::runtime::Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::MultiThread => {
                    tokio::task::block_in_place(move || gate.blocking_lock_owned())
                }
                tokio::runtime::RuntimeFlavor::CurrentThread => gate
                    .try_lock_owned()
                    .map_err(|_| "store write gate busy in current-thread runtime".to_string())?,
                _ => gate.blocking_lock_owned(),
            },
            Err(_) => gate.blocking_lock_owned(),
        };
        Ok(StoreWriteGuard {
            store: self.backend.load_store()?,
            backend: self.backend.clone(),
            _write_guard: write_guard,
        })
    }
}

fn resolve_backend_bind_addr() -> Result<SocketAddr, String> {
    let raw = std::env::var("SFID_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8899".to_string());
    raw.parse::<SocketAddr>()
        .map_err(|e| format!("invalid SFID_BIND_ADDR `{raw}`: {e}"))
}

fn database_url_targets_local_host_only(database_url: &str) -> Result<bool, String> {
    let config = database_url
        .parse::<postgres::Config>()
        .map_err(|e| format!("invalid DATABASE_URL: {e}"))?;
    if config.get_hosts().is_empty() {
        return Ok(true);
    }
    Ok(config.get_hosts().iter().all(|host| match host {
        Host::Tcp(name) => {
            let lowered = name.to_ascii_lowercase();
            lowered == "localhost" || lowered == "127.0.0.1" || lowered == "::1"
        }
        Host::Unix(_) => true,
    }))
}

fn disable_core_dumps() {
    #[cfg(unix)]
    {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // Best-effort hardening: avoid leaking in-memory secrets through coredumps.
        let rc = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limit) };
        if rc != 0 {
            warn!(
                error = %std::io::Error::last_os_error(),
                "failed to disable core dumps"
            );
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();
    disable_core_dumps();

    let redis_url = required_env("SFID_REDIS_URL");
    let redis_client = RedisClient::open(redis_url.as_str())
        .unwrap_or_else(|e| panic!("invalid SFID_REDIS_URL: {e}"));

    let main_seed = SensitiveSeed::from(required_env("SFID_SIGNING_SEED_HEX"));
    let main_key = key_admins::chain_keyring::load_signing_key_from_seed(main_seed.expose_secret());
    let public_key_hex = format!("0x{}", hex::encode(main_key.public().0));
    let mut known_key_seeds = HashMap::new();
    known_key_seeds.insert(public_key_hex.clone(), main_seed.clone());
    let database_url = required_env("DATABASE_URL");
    if database_url
        .to_ascii_lowercase()
        .contains("sslmode=disable")
    {
        panic!("DATABASE_URL must not use sslmode=disable");
    }
    let db_is_local = database_url_targets_local_host_only(database_url.as_str())
        .unwrap_or_else(|e| panic!("{e}"));
    if !db_is_local && !env_flag_enabled("SFID_ALLOW_REMOTE_DB_WITHOUT_TLS") {
        panic!(
            "DATABASE_URL points to non-local host, but sync postgres client is running in NoTls mode; set SFID_ALLOW_REMOTE_DB_WITHOUT_TLS=true only if transport is protected externally"
        );
    }
    let store = StoreHandle::from_database_url(database_url.as_str()).expect("init store handle");
    // 中文注释：任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B：
    // 从 SFID MAIN seed 构造省级签名密钥缓存。seed 字节在构造后立即被 zeroize。
    let sheng_signer_cache = {
        let seed_hex_str = main_seed.expose_secret().to_string();
        let seed_bytes = hex::decode(seed_hex_str.trim_start_matches("0x"))
            .unwrap_or_else(|e| panic!("SFID_SIGNING_SEED_HEX invalid hex: {e}"));
        if seed_bytes.len() != 32 {
            panic!("SFID_SIGNING_SEED_HEX must decode to 32 bytes");
        }
        let mut seed_arr = [0u8; 32];
        seed_arr.copy_from_slice(&seed_bytes);
        let cache = key_admins::sheng_signer_cache::ShengSignerCache::new_from_seed(&mut seed_arr)
            .expect("init sheng signer cache failed");
        Arc::new(cache)
    };
    // 任务卡 `20260410-sfid-store-shard-by-province` Phase 2 Day 2:
    // 基于现有 Postgres 连接池构造 ShardBackend 和 ShardedStore。
    // 此时只构造空壳,迁移 / bootstrap / preload 等到 tokio runtime 起来后再做。
    let sharded_store: Arc<store_shards::ShardedStore> = {
        let (pool, next_idx) = store
            .postgres_pool()
            .expect("store backend must be Postgres for sharded store");
        let backend: Arc<dyn store_shards::backend::ShardBackend> = Arc::new(
            store_shards::pg_backend::PostgresShardBackend::new(pool, next_idx),
        );
        let double_write = std::env::var("SFID_SHARD_SINGLE_WRITE")
            .map(|v| v != "true")
            .unwrap_or(true);
        Arc::new(store_shards::ShardedStore::new(backend, double_write))
    };
    let state = AppState {
        store,
        signing_seed_hex: Arc::new(RwLock::new(main_seed)),
        known_key_seeds: Arc::new(RwLock::new(known_key_seeds)),
        rate_limit_redis: Arc::new(redis_client),
        cpms_register_inflight: Arc::new(Mutex::new(HashMap::new())),
        key_id: required_env("SFID_KEY_ID"),
        key_version: "v1".to_string(),
        key_alg: "sr25519".to_string(),
        public_key_hex: Arc::new(RwLock::new(public_key_hex)),
        sheng_signer_cache,
        sharded_store,
    };
    seed_sheng_admins(&state);
    sync_builtin_sheng_admins(&state);
    key_admins::seed_chain_keyring(&state);
    key_admins::seed_key_admins(&state);
    info!("initialized runtime state with defaults");
    // 中文注释:任务卡 6 启动对账:按 sfid 工具市清单对齐全部公安局机构。
    app_core::runtime_ops::backfill_and_reconcile_public_security(&state);

    // ── SFID-CPMS QR v1: 初始化 RSA 匿名证书密钥对 ──
    // 注意：store.write() 需要 tokio runtime，因此仅在此处做 read + init，
    // 如果需要生成新密钥则在 runtime 启动后再 write + persist。
    {
        let existing_pem = state
            .store
            .read()
            .ok()
            .and_then(|s| s.anon_rsa_private_key_pem.clone());
        if existing_pem
            .as_deref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
        {
            // 已有密钥，直接加载（不需要 tokio runtime）
            match key_admins::rsa_blind::init_from_pem(existing_pem.as_deref().unwrap()) {
                Ok(()) => info!("loaded existing RSA anon cert keypair from store"),
                Err(e) => warn!("RSA anon cert keypair load failed: {e}"),
            }
        } else {
            info!("no existing RSA anon cert keypair, will generate after runtime start");
        }
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(async move {
        // ── RSA 密钥生成（需要 tokio runtime 才能 store.write）──
        if key_admins::rsa_blind::get_public_key_pem().is_err() {
            info!("generating RSA anon cert keypair...");
            match key_admins::rsa_blind::generate_keypair_pem() {
                Ok(new_pem) => {
                    if let Ok(mut store) = state.store.write() {
                        store.anon_rsa_private_key_pem = Some(new_pem);
                    }
                    info!("generated and persisted new RSA anon cert keypair");
                }
                Err(e) => warn!("RSA keypair generation failed: {e}"),
            }
        }

        // 任务卡 `20260410-sfid-store-shard-by-province` Phase 2 Day 2:
        // 1) 从 legacy store 快照执行一次幂等迁移(空库首次启动才真正写入)
        // 2) 加载 GlobalShard
        // 3) 可选预加载所有省分片
        {
            let legacy_snapshot = match state.store.read() {
                Ok(guard) => (*guard).clone(),
                Err(e) => {
                    warn!(error = %e, "load legacy store snapshot for migration failed");
                    Store::default()
                }
            };
            if let Some((pool, next_idx)) = state.store.postgres_pool() {
                if let Err(e) = store_shards::migration::migrate_legacy_store_if_needed(
                    pool,
                    next_idx,
                    &legacy_snapshot,
                )
                .await
                {
                    warn!(error = %e, "legacy → sharded migration failed");
                }
            }
            if let Err(e) = state.sharded_store.bootstrap_global().await {
                warn!(error = %e, "sharded store bootstrap_global failed");
            }
            let preload = std::env::var("SFID_SHARD_PRELOAD_ALL")
                .map(|v| v != "false")
                .unwrap_or(true);
            if preload {
                match state.sharded_store.preload_all_shards().await {
                    Ok(n) => info!(provinces_loaded = n, "sharded store preloaded"),
                    Err(e) => warn!(error = %e, "sharded store preload failed"),
                }
            }
        }

        // Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store 后，清理孤儿需要 async
        app_core::runtime_ops::cleanup_orphan_cpms_sites(&state).await;
        app_core::runtime_ops::cleanup_stale_cpms_sites(&state).await;

        // 2026-04-21:reconcile 是同步调用,只写 legacy store,新增的公安局机构 +
        // 主账户/费用账户 不会自动落到 sharded_store,前端按省读分片看不到。
        // 启动 + preload 完成后,从 legacy 快照幂等同步一次,保证详情页/列表可见。
        app_core::runtime_ops::sync_public_security_to_sharded(&state).await;

        tokio::spawn(bind_callback_worker(state.clone()));
        tokio::spawn(indexer::indexer_worker(state.store.backend.clone()));

        let auth_routes = Router::new()
            .route("/api/v1/admin/auth/check", get(login::admin_auth_check))
            .route("/api/v1/admin/auth/logout", post(login::admin_logout))
            .route(
                "/api/v1/admin/auth/identify",
                post(login::admin_auth_identify),
            )
            .route(
                "/api/v1/admin/auth/challenge",
                post(login::admin_auth_challenge),
            )
            .route("/api/v1/admin/auth/verify", post(login::admin_auth_verify))
            .route(
                "/api/v1/admin/auth/qr/challenge",
                post(login::admin_auth_qr_challenge),
            )
            .route(
                "/api/v1/admin/auth/qr/complete",
                post(login::admin_auth_qr_complete),
            )
            .route(
                "/api/v1/admin/auth/qr/result",
                get(login::admin_auth_qr_result),
            );

        let admin_routes = Router::new()
            .route(
                "/api/v1/admin/operators",
                get(sheng_admins::list_operators).post(sheng_admins::create_operator),
            )
            .route(
                "/api/v1/admin/operators/:id",
                put(sheng_admins::update_operator).delete(sheng_admins::delete_operator),
            )
            .route(
                "/api/v1/admin/operators/:id/status",
                put(sheng_admins::update_operator_status),
            )
            .route(
                "/api/v1/admin/sheng-admins",
                get(sheng_admins::list_sheng_admins),
            )
            .route(
                "/api/v1/admin/sheng-admins/:province",
                put(sheng_admins::replace_sheng_admin),
            )
            .route("/api/v1/admin/cpms-keys", get(sheng_admins::list_cpms_keys))
            .route(
                "/api/v1/admin/cpms-keys/by-institution/:sfid_id",
                get(sheng_admins::get_cpms_site_by_institution),
            )
            .route(
                "/api/v1/admin/cpms-keys/sfid/generate",
                post(sheng_admins::generate_cpms_institution_sfid_qr),
            )
            .route(
                "/api/v1/admin/cpms/register",
                post(sheng_admins::register_cpms),
            )
            .route(
                "/api/v1/admin/cpms/archive/import",
                post(sheng_admins::archive_import),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid",
                delete(sheng_admins::delete_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid/revoke-token",
                post(sheng_admins::revoke_install_token),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid/reissue",
                post(sheng_admins::reissue_install_token),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid/disable",
                put(sheng_admins::disable_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid/enable",
                put(sheng_admins::enable_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid/revoke",
                put(sheng_admins::revoke_cpms_keys),
            )
            // ── 链上余额查询(admin keyring 视图主账户行) ──
            .route(
                "/api/v1/admin/chain/balance",
                get(chain::balance::admin_query_chain_balance),
            )
            // 中文注释:机构/账户两层模型的 API
            // - GET  /api/v1/institution/check-name                      — 机构名称全国查重
            // - POST /api/v1/institution/create                          — 生成机构(不上链)
            // - POST /api/v1/institution/:sfid_id/account/create         — 只登记账户名称,不上链
            // - GET  /api/v1/institution/list                            — 按 scope 过滤的机构列表
            // - GET  /api/v1/institution/:sfid_id                        — 机构详情
            // - GET  /api/v1/institution/:sfid_id/accounts               — 账户列表
            // - DELETE /api/v1/institution/:sfid_id/account/:account_name — 删除未上链/已注销新增账户
            .route(
                "/api/v1/institution/check-name",
                get(institutions::handler::check_institution_name),
            )
            // FFR 详情页"所属法人"搜索(全国范围 SFR/GFR 模糊匹配)
            .route(
                "/api/v1/institution/search-parents",
                get(institutions::handler::search_parent_institutions),
            )
            .route(
                "/api/v1/institution/create",
                post(institutions::handler::create_institution),
            )
            .route(
                "/api/v1/institution/:sfid_id/account/create",
                post(institutions::handler::create_account),
            )
            .route(
                "/api/v1/institution/list",
                get(institutions::handler::list_institutions),
            )
            .route(
                "/api/v1/institution/:sfid_id",
                get(institutions::handler::get_institution)
                    // 两步式第二步:详情页更新机构名称/企业类型
                    .patch(institutions::handler::update_institution),
            )
            .route(
                "/api/v1/institution/:sfid_id/accounts",
                get(institutions::handler::list_accounts),
            )
            .route(
                "/api/v1/institution/:sfid_id/account/:account_name",
                delete(institutions::handler::delete_account),
            )
            // 机构资料库文档 CRUD
            .route(
                "/api/v1/institution/:sfid_id/documents",
                get(institutions::handler::list_documents)
                    .post(institutions::handler::upload_document),
            )
            .route(
                "/api/v1/institution/:sfid_id/documents/:doc_id/download",
                get(institutions::handler::download_document),
            )
            .route(
                "/api/v1/institution/:sfid_id/documents/:doc_id",
                delete(institutions::handler::delete_document),
            )
            // 临时诊断:手动触发 bootstrap sheng signer
            .route(
                "/api/v1/admin/debug/bootstrap-signer",
                post(debug_bootstrap_signer),
            )
            // 任务卡 6:公安局跟 sfid 工具市清单对账
            .route(
                "/api/v1/public-security/reconcile",
                post(institutions::handler::reconcile_public_security),
            )
            .route(
                "/api/v1/admin/cpms-status/scan",
                post(shi_admins::admin_cpms_status_scan),
            )
            .route(
                "/api/v1/admin/audit-logs",
                get(business::audit::admin_list_audit_logs),
            )
            .route(
                "/api/v1/admin/citizens",
                get(business::query::admin_list_citizens),
            )
            // ── 公民身份绑定 ──
            .route(
                "/api/v1/admin/citizen/bind/challenge",
                post(operate::binding::citizen_bind_challenge),
            )
            .route(
                "/api/v1/admin/citizen/bind",
                post(operate::binding::citizen_bind),
            )
            .route(
                "/api/v1/admin/citizen/unbind",
                post(operate::binding::citizen_unbind),
            )
            // ── 投票账户推链 ──
            .route(
                "/api/v1/admin/citizen/bind/push-chain",
                post(operate::binding::citizen_push_chain_bind),
            )
            .route(
                "/api/v1/admin/citizen/unbind/push-chain",
                post(operate::binding::citizen_push_chain_unbind),
            )
            .route("/api/v1/admin/sfid/meta", get(sfid::admin::admin_sfid_meta))
            .route(
                "/api/v1/admin/sfid/cities",
                get(sfid::admin::admin_sfid_cities),
            )
            .route(
                "/api/v1/admin/attestor/keyring",
                get(key_admins::admin_get_chain_keyring),
            )
            .route(
                "/api/v1/admin/attestor/rotate/challenge",
                post(key_admins::admin_chain_keyring_rotate_challenge),
            )
            .route(
                "/api/v1/admin/attestor/rotate/verify",
                post(key_admins::admin_chain_keyring_rotate_verify),
            )
            .route(
                "/api/v1/admin/attestor/rotate/commit",
                post(key_admins::admin_chain_keyring_rotate_commit),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                login::require_admin_session_middleware,
            ));

        // 中文注释:历史 chain_routes(/vote/verify、/chain/voters/count、/chain/binding/validate、
        // /chain/reward/ack、/chain/reward/state、/attestor/public-key)0 caller,
        // 2026-05-01 chain/ 重构一并下架。链端 pull 通道全部走 app_routes 命名空间。

        let public_routes = Router::new()
            .route("/", get(root))
            .route("/api/v1/health", get(health))
            .route(
                "/api/v1/public/identity/search",
                get(business::query::public_identity_search),
            );

        // App routes:手机 App 与节点桌面 chain pull 用的统一命名空间。
        //
        // 全部端点都汇集在 chain/ 子目录(institution_info / joint_vote / citizen_vote)。
        // wuminapp 自有功能(钱包交易索引、投票账户绑定)继续留 indexer / operate 模块。
        let app_routes = Router::new()
            // ── 联合投票:获取公民人数快照凭证 ──
            .route(
                "/api/v1/app/voters/count",
                get(chain::joint_vote::app_voters_count),
            )
            // ── 公民投票凭证签发 ──
            .route(
                "/api/v1/app/vote/credential",
                post(chain::citizen_vote::app_vote_credential),
            )
            // ── 钱包交易索引(wuminapp 自有,与链交互无关) ──
            .route(
                "/api/v1/app/wallet/:address/transactions",
                get(indexer::api::wallet_transactions),
            )
            // ── wuminapp 投票账户注册/查询(wuminapp 自有) ──
            .route(
                "/api/v1/app/vote-account/register",
                post(operate::binding::app_vote_account_register),
            )
            .route(
                "/api/v1/app/vote-account/status",
                get(operate::binding::app_vote_account_status),
            )
            // ── 机构信息查询(链端/钱包 pull):机构搜索 / 详情 / 账户列表 ──
            .route(
                "/api/v1/app/institutions/search",
                get(chain::institution_info::app_search_institutions),
            )
            .route(
                "/api/v1/app/institutions/:sfid_id",
                get(chain::institution_info::app_get_institution),
            )
            .route(
                "/api/v1/app/institutions/:sfid_id/accounts",
                get(chain::institution_info::app_list_accounts),
            )
            // ── 清算行搜索(已激活,wuminapp 绑定清算行用):资格白名单 + 主账户 ACTIVE_ON_CHAIN ──
            .route(
                "/api/v1/app/clearing-banks/search",
                get(chain::institution_info::app_search_clearing_banks),
            )
            // ── 候选清算行搜索(可未激活,节点桌面"添加清算行"用):仅资格白名单过滤 ──
            .route(
                "/api/v1/app/clearing-banks/eligible-search",
                get(chain::institution_info::app_search_eligible_clearing_banks),
            );

        let app_state = state.clone();
        let app = Router::new()
            .merge(public_routes)
            .merge(auth_routes)
            .merge(admin_routes)
            .merge(app_routes)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                global_rate_limit_middleware,
            ))
            .layer(build_cors_layer())
            .with_state(app_state);

        // 中文注释：SFID 现在以“链上三把公钥 + 本地主私钥”作为唯一真相；
        // 启动前必须完成创世哈希初始化、同步链上 keyring，并确认本地主私钥
        // 派生出的公钥就是链上当前 main 公钥，否则拒绝提供签名服务。
        chain::runtime_align::init_genesis_hash_from_chain()
            .await
            .unwrap_or_else(|e| panic!("failed to initialize chain genesis hash: {e}"));
        info!("chain genesis hash initialized");

        key_admins::sync_chain_keyring_from_chain(&state)
            .await
            .unwrap_or_else(|e| panic!("failed to sync chain keyring from chain: {e}"));
        key_admins::seed_key_admins(&state);
        key_admins::validate_active_main_signer_with_keyring(&state)
            .unwrap_or_else(|e| panic!("active main signer validation failed: {e}"));

        // 本地手机联调时必须监听到与 App 可访问的一致地址，避免只绑定回环导致超时。
        let addr = resolve_backend_bind_addr().expect("resolve sfid backend bind address");
        info!("sfid-backend listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("bind sfid backend listener");
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("run sfid backend server");
    });
}

// 中文注释:历史 ensure_chain_request_db / prepare_chain_request 与已下架的
// /api/v1/chain/* + /api/v1/vote/verify dead routes 配套使用,2026-05-01 一并下架。
// 链端 chain pull 端点(institution_info / joint_vote / citizen_vote)无 attestor
// 鉴权需求,全局 rate limiter 已防滥用,凭证签名本身就是反伪造保护。

#[allow(dead_code)]
fn ensure_binding_lock_db(
    state: &AppState,
    account_pubkey: &str,
    archive_index: &str,
) -> Result<(), axum::response::Response> {
    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return Ok(());
    };
    let check = StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        let mut tx = conn.transaction().map_err(|e| e.to_string())?;
        let row_by_pub = tx
            .query_opt(
                "SELECT archive_index FROM binding_unique_locks WHERE account_pubkey=$1 FOR UPDATE",
                &[&account_pubkey],
            )
            .map_err(|e| e.to_string())?;
        if let Some(row) = row_by_pub {
            let existing_archive: String = row.get(0);
            if existing_archive != archive_index {
                return Err("pubkey_conflict".to_string());
            }
        }
        let row_by_archive = tx
            .query_opt(
                "SELECT account_pubkey FROM binding_unique_locks WHERE archive_index=$1 FOR UPDATE",
                &[&archive_index],
            )
            .map_err(|e| e.to_string())?;
        if let Some(row) = row_by_archive {
            let existing_pubkey: String = row.get(0);
            if existing_pubkey != account_pubkey {
                return Err("archive_conflict".to_string());
            }
        }
        tx.execute(
            "INSERT INTO binding_unique_locks(account_pubkey, archive_index, bound_at)
             VALUES ($1, $2, now())
             ON CONFLICT (account_pubkey) DO NOTHING",
            &[&account_pubkey, &archive_index],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    });
    match check {
        Ok(_) => Ok(()),
        Err(err) if err == "archive_conflict" => Err(api_error(
            StatusCode::CONFLICT,
            3001,
            "archive_index already bound",
        )),
        Err(err) if err == "pubkey_conflict" => Err(api_error(
            StatusCode::CONFLICT,
            3002,
            "pubkey already bound to another archive_index",
        )),
        Err(err) => {
            warn!("binding lock db check failed: {err}");
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1501,
                "binding consistency check failed",
            ))
        }
    }
}

#[allow(dead_code)]
fn release_binding_lock_db(state: &AppState, account_pubkey: &str) {
    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return;
    };
    let _ = StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        conn.execute(
            "DELETE FROM binding_unique_locks WHERE account_pubkey=$1",
            &[&account_pubkey],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
}

#[allow(dead_code)]
fn persist_reward_state_db(
    state: &AppState,
    reward: &RewardStateRecord,
    expected_updated_at: Option<DateTime<Utc>>,
) -> Result<bool, String> {
    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return Ok(true);
    };
    StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        let status_text = format!("{:?}", reward.reward_status).to_uppercase();
        if let Some(previous_updated_at) = expected_updated_at {
            let affected = conn
                .execute(
                    "UPDATE bind_reward_states SET
                       archive_index=$2,
                       callback_id=$3,
                       reward_status=$4,
                       retry_count=$5,
                       max_retries=$6,
                       reward_tx_hash=$7,
                       last_error=$8,
                       next_retry_at=$9,
                       updated_at=$10
                     WHERE account_pubkey=$1 AND updated_at=$11",
                    &[
                        &reward.account_pubkey,
                        &reward.archive_index,
                        &reward.callback_id,
                        &status_text,
                        &(reward.retry_count as i32),
                        &(reward.max_retries as i32),
                        &reward.reward_tx_hash,
                        &reward.last_error,
                        &reward.next_retry_at,
                        &reward.updated_at,
                        &previous_updated_at,
                    ],
                )
                .map_err(|e| e.to_string())?;
            return Ok(affected > 0);
        }

        let affected = conn
            .execute(
                "INSERT INTO bind_reward_states(
                   account_pubkey, archive_index, callback_id, reward_status, retry_count, max_retries,
                   reward_tx_hash, last_error, next_retry_at, updated_at, created_at
                 ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
                 ON CONFLICT (account_pubkey) DO UPDATE SET
                   archive_index=excluded.archive_index,
                   callback_id=excluded.callback_id,
                   reward_status=excluded.reward_status,
                   retry_count=excluded.retry_count,
                   max_retries=excluded.max_retries,
                   reward_tx_hash=excluded.reward_tx_hash,
                   last_error=excluded.last_error,
                   next_retry_at=excluded.next_retry_at,
                   updated_at=excluded.updated_at",
                &[
                    &reward.account_pubkey,
                    &reward.archive_index,
                    &reward.callback_id,
                    &status_text,
                    &(reward.retry_count as i32),
                    &(reward.max_retries as i32),
                    &reward.reward_tx_hash,
                    &reward.last_error,
                    &reward.next_retry_at,
                    &reward.updated_at,
                    &reward.created_at,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(affected > 0)
    })
}

#[allow(dead_code)]
fn remove_reward_state_db(state: &AppState, account_pubkey: &str) {
    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return;
    };
    let _ = StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        conn.execute(
            "DELETE FROM bind_reward_states WHERE account_pubkey=$1",
            &[&account_pubkey],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
}

fn api_error(status: StatusCode, code: u32, message: &str) -> axum::response::Response {
    (
        status,
        Json(ApiError {
            code,
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
        .into_response()
}

pub(crate) fn store_read_or_500(
    state: &AppState,
) -> Result<StoreReadGuard, axum::response::Response> {
    state.store.read().map_err(|err| {
        warn!(error = %err, "store read failed");
        api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed")
    })
}

pub(crate) fn store_write_or_500(
    state: &AppState,
) -> Result<StoreWriteGuard, axum::response::Response> {
    state.store.write().map_err(|err| {
        warn!(error = %err, "store write failed");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "store write failed",
        )
    })
}

#[cfg(test)]
mod main_tests;

/// 临时诊断端点:手动触发 bootstrap sheng signer，返回具体错误信息。
async fn debug_bootstrap_signer(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl axum::response::IntoResponse {
    let ctx = match login::require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if ctx.role != models::AdminRole::ShengAdmin {
        return api_error(axum::http::StatusCode::FORBIDDEN, 1003, "only sheng admin");
    }
    let province = match ctx.admin_province.as_deref() {
        Some(v) => v.to_string(),
        None => return api_error(axum::http::StatusCode::BAD_REQUEST, 1001, "no province"),
    };
    // 先诊断 subxt metadata
    let ws_url = match chain::url::chain_ws_url() {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("chain_ws_url: {e}"),
            )
        }
    };
    let diag = match subxt::OnlineClient::<subxt::PolkadotConfig>::from_insecure_url(&ws_url).await
    {
        Ok(client) => {
            let metadata = client.metadata();
            let pallets: Vec<String> = metadata.pallets().map(|p| p.name().to_string()).collect();
            let sfid_calls: Vec<String> = metadata
                .pallets()
                .find(|p| p.name() == "SfidSystem")
                .and_then(|p| p.call_variants())
                .map(|calls| calls.iter().map(|c| c.name.clone()).collect())
                .unwrap_or_default();
            format!(
                "ws_url={} pallets={} sfid_calls={:?}",
                ws_url,
                pallets.len(),
                sfid_calls
            )
        }
        Err(e) => format!("ws_url={} connect_error={}", ws_url, e),
    };

    match key_admins::bootstrap_sheng_signer(&state, &ctx.admin_pubkey, &province).await {
        Ok(()) => axum::Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: format!("bootstrap success for {province} | diag: {diag}"),
        })
        .into_response(),
        Err(e) => api_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            &format!("{e} | diag: {diag}"),
        ),
    }
}
