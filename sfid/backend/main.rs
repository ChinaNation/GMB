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
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
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

mod admins;
mod app_core;
mod audit;
mod citizens;
mod cpms;
mod crypto;
mod indexer;
mod institutions;
mod login;
mod models;
#[allow(dead_code)]
mod qr;
mod scope;
mod sfid;
mod store_shards;

pub(crate) use app_core::http_security::*;
pub(crate) use app_core::runtime_ops::*;
pub(crate) use citizens::model::*;
pub(crate) use cpms::model::*;
pub(crate) use cpms::scope::in_scope_cpms_site;
pub(crate) use login::{
    build_admin_display_name, parse_sr25519_pubkey, parse_sr25519_pubkey_bytes, require_admin_any,
    require_sheng_admin, AdminSession, LoginChallenge, QrLoginResultRecord,
};
pub(crate) use models::*;
pub(crate) use sfid::model::*;

#[derive(Clone)]
struct AppState {
    store: StoreHandle,
    rate_limit_redis: Arc<RedisClient>,
    /// 中文注释:按省分片的进程内缓存。Postgres 持久化已改为模块 Store 表,
    /// 这里不再写旧 `store_shards` JSONB 表。
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

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct CitizenStoreSnapshot {
    next_citizen_id: u64,
    citizen_records: HashMap<u64, CitizenRecord>,
    citizen_id_by_wallet_pubkey: HashMap<String, u64>,
    citizen_id_by_archive_no: HashMap<String, u64>,
    citizen_id_by_sfid_code: HashMap<String, u64>,
    /// 中文注释:绑定 challenge 属于公民绑定短期状态,必须跨请求可读,
    /// 但不再写入旧 runtime 整包 JSON。
    citizen_bind_challenges: HashMap<String, CitizenBindChallenge>,
    /// 中文注释:CPMS 年度报告导入幂等记录,随公民模块快照持久化。
    cpms_status_export_imports: HashMap<String, CpmsStatusExportImportRecord>,
    consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    reward_state_by_pubkey: HashMap<String, RewardStateRecord>,
    vote_verify_cache: HashMap<String, VoteVerifyCacheEntry>,
}

impl CitizenStoreSnapshot {
    fn from_store(store: &Store) -> Self {
        Self {
            next_citizen_id: store.next_citizen_id,
            citizen_records: store.citizen_records.clone(),
            citizen_id_by_wallet_pubkey: store.citizen_id_by_wallet_pubkey.clone(),
            citizen_id_by_archive_no: store.citizen_id_by_archive_no.clone(),
            citizen_id_by_sfid_code: store.citizen_id_by_sfid_code.clone(),
            citizen_bind_challenges: store.citizen_bind_challenges.clone(),
            cpms_status_export_imports: store.cpms_status_export_imports.clone(),
            consumed_qr_ids: store.consumed_qr_ids.clone(),
            reward_state_by_pubkey: store.reward_state_by_pubkey.clone(),
            vote_verify_cache: store.vote_verify_cache.clone(),
        }
    }

    fn apply_to(self, store: &mut Store) {
        store.next_citizen_id = self.next_citizen_id;
        store.citizen_records = self.citizen_records;
        store.citizen_id_by_wallet_pubkey = self.citizen_id_by_wallet_pubkey;
        store.citizen_id_by_archive_no = self.citizen_id_by_archive_no;
        store.citizen_id_by_sfid_code = self.citizen_id_by_sfid_code;
        store.citizen_bind_challenges = self.citizen_bind_challenges;
        store.cpms_status_export_imports = self.cpms_status_export_imports;
        store.consumed_qr_ids = self.consumed_qr_ids;
        store.reward_state_by_pubkey = self.reward_state_by_pubkey;
        store.vote_verify_cache = self.vote_verify_cache;
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct CpmsStoreSnapshot {
    cpms_site_keys: HashMap<String, CpmsSiteKeys>,
}

impl CpmsStoreSnapshot {
    fn from_store(store: &Store) -> Self {
        Self {
            cpms_site_keys: store.cpms_site_keys.clone(),
        }
    }

    fn apply_to(self, store: &mut Store) {
        store.cpms_site_keys = self.cpms_site_keys;
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct InstStoreSnapshot {
    multisig_institutions: HashMap<String, crate::institutions::MultisigInstitution>,
    multisig_accounts: HashMap<String, crate::institutions::MultisigAccount>,
    institution_documents: HashMap<String, crate::institutions::InstitutionDocument>,
    next_document_id: u64,
}

impl InstStoreSnapshot {
    fn from_store(store: &Store) -> Self {
        Self {
            multisig_institutions: store.multisig_institutions.clone(),
            multisig_accounts: store.multisig_accounts.clone(),
            institution_documents: store.institution_documents.clone(),
            next_document_id: store.next_document_id,
        }
    }

    fn apply_to(self, store: &mut Store) {
        store.multisig_institutions = self.multisig_institutions;
        store.multisig_accounts = self.multisig_accounts;
        store.institution_documents = self.institution_documents;
        store.next_document_id = self.next_document_id;
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct OpsStoreSnapshot {
    next_seq: u64,
    next_audit_seq: u64,
    next_admin_user_id: u64,
    admin_passkeys_by_credential_id: HashMap<String, AdminPasskeyCredential>,
    admin_passkey_registration_challenges: HashMap<String, AdminPasskeyRegistrationChallenge>,
    admin_action_challenges: HashMap<String, AdminActionChallenge>,
    admin_security_grants: HashMap<String, AdminSecurityGrant>,
    /// 中文注释:登录 challenge/session/扫码结果必须跨请求可读;
    /// 本任务将它们收敛到 ops 模块快照,不再依赖旧 runtime cache 表。
    login_challenges: HashMap<String, LoginChallenge>,
    qr_login_results: HashMap<String, QrLoginResultRecord>,
    admin_sessions: HashMap<String, AdminSession>,
    audit_logs: Vec<AuditLogEntry>,
    chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    chain_nonce_seen: HashMap<String, DateTime<Utc>>,
    chain_auth_last_cleanup_at: Option<DateTime<Utc>>,
    metrics: ServiceMetrics,
}

impl OpsStoreSnapshot {
    fn from_store(store: &Store) -> Self {
        Self {
            next_seq: store.next_seq,
            next_audit_seq: store.next_audit_seq,
            next_admin_user_id: store.next_admin_user_id,
            admin_passkeys_by_credential_id: store.admin_passkeys_by_credential_id.clone(),
            admin_passkey_registration_challenges: store
                .admin_passkey_registration_challenges
                .clone(),
            admin_action_challenges: store.admin_action_challenges.clone(),
            admin_security_grants: store.admin_security_grants.clone(),
            login_challenges: store.login_challenges.clone(),
            qr_login_results: store.qr_login_results.clone(),
            admin_sessions: store.admin_sessions.clone(),
            audit_logs: store.audit_logs.clone(),
            chain_requests_by_key: store.chain_requests_by_key.clone(),
            chain_nonce_seen: store.chain_nonce_seen.clone(),
            chain_auth_last_cleanup_at: store.chain_auth_last_cleanup_at,
            metrics: store.metrics.clone(),
        }
    }

    fn apply_to(self, store: &mut Store) {
        store.next_seq = self.next_seq;
        store.next_audit_seq = self.next_audit_seq;
        store.next_admin_user_id = self.next_admin_user_id;
        store.admin_passkeys_by_credential_id = self.admin_passkeys_by_credential_id;
        store.admin_passkey_registration_challenges = self.admin_passkey_registration_challenges;
        store.admin_action_challenges = self.admin_action_challenges;
        store.admin_security_grants = self.admin_security_grants;
        store.login_challenges = self.login_challenges;
        store.qr_login_results = self.qr_login_results;
        store.admin_sessions = self.admin_sessions;
        store.audit_logs = self.audit_logs;
        store.chain_requests_by_key = self.chain_requests_by_key;
        store.chain_nonce_seen = self.chain_nonce_seen;
        store.chain_auth_last_cleanup_at = self.chain_auth_last_cleanup_at;
        store.metrics = self.metrics;
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
        // 中文注释:当前只接受 SHENG_ADMIN / SHI_ADMIN;无法识别时降级到最严范围。
        match role {
            "SHENG_ADMIN" => AdminRole::ShengAdmin,
            "SHI_ADMIN" => AdminRole::ShiAdmin,
            _ => AdminRole::ShiAdmin,
        }
    }

    fn admin_role_text(role: &AdminRole) -> &'static str {
        match role {
            AdminRole::ShengAdmin => "SHENG_ADMIN",
            AdminRole::ShiAdmin => "SHI_ADMIN",
        }
    }

    fn load_module_store<T>(conn: &mut postgres::Client, table: &str) -> Result<T, String>
    where
        T: Default + DeserializeOwned,
    {
        let sql = format!("SELECT payload FROM {table} WHERE id = 1");
        let row = conn
            .query_opt(sql.as_str(), &[])
            .map_err(|e| format!("load {table} failed: {e}"))?;
        let Some(row) = row else {
            return Ok(T::default());
        };
        let payload: serde_json::Value = row.get(0);
        match serde_json::from_value(payload) {
            Ok(snapshot) => Ok(snapshot),
            Err(err) => {
                // 中文注释:模块快照结构不匹配时直接丢弃该模块状态。
                // 用户已确认本次 Store 重构不做旧数据迁移和旧格式兼容。
                let delete_sql = format!("DELETE FROM {table} WHERE id = 1");
                conn.execute(delete_sql.as_str(), &[])
                    .map_err(|e| format!("delete invalid {table} snapshot failed: {e}"))?;
                warn!(table, error = %err, "dropped invalid module store snapshot");
                Ok(T::default())
            }
        }
    }

    fn save_module_store<T>(
        tx: &mut postgres::Transaction<'_>,
        table: &str,
        snapshot: &T,
    ) -> Result<(), String>
    where
        T: Serialize,
    {
        let payload = serde_json::to_value(snapshot)
            .map_err(|e| format!("encode {table} snapshot failed: {e}"))?;
        let sql = format!(
            "INSERT INTO {table}(id, payload, updated_at) VALUES (1, $1, now())
             ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload, updated_at = now()"
        );
        tx.execute(sql.as_str(), &[&payload])
            .map_err(|e| format!("save {table} snapshot failed: {e}"))?;
        Ok(())
    }

    fn load_store_postgres(conn: &mut postgres::Client) -> Result<Store, String> {
        let mut store = Store::default();
        Self::load_module_store::<CitizenStoreSnapshot>(conn, "store_citizens")?
            .apply_to(&mut store);
        Self::load_module_store::<CpmsStoreSnapshot>(conn, "store_cpms")?.apply_to(&mut store);
        Self::load_module_store::<InstStoreSnapshot>(conn, "store_institutions")?
            .apply_to(&mut store);
        Self::load_module_store::<OpsStoreSnapshot>(conn, "store_ops")?.apply_to(&mut store);

        store.admin_users_by_pubkey.clear();
        store.sheng_admin_province_by_pubkey.clear();

        let admin_rows = conn
            .query(
                "SELECT admin_id, admin_pubkey, admin_name, role, built_in, created_by, created_at, updated_at, city
                 FROM admins",
                &[],
            )
            .map_err(|e| format!("load admins failed: {e}"))?;
        for row in admin_rows {
            let id: i64 = row.get(0);
            let admin_pubkey: String = row.get(1);
            let admin_name: String = row.get(2);
            let role_text: String = row.get(3);
            let built_in: bool = row.get(4);
            let created_by: String = row.get(5);
            let created_at: DateTime<Utc> = row.get(6);
            let updated_at: Option<DateTime<Utc>> = row.get(7);
            let city: String = row.get(8);
            store.admin_users_by_pubkey.insert(
                admin_pubkey.clone(),
                AdminUser {
                    id: u64::try_from(id).unwrap_or(0),
                    admin_pubkey,
                    admin_name,
                    role: Self::parse_admin_role(role_text.as_str()),
                    built_in,
                    created_by,
                    created_at,
                    updated_at,
                    city,
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

        // ADR-008 Phase 23e(2026-05-01):旧全局密钥环表和内存状态已删除。

        Ok(store)
    }

    fn save_store_postgres(conn: &mut postgres::Client, store: &Store) -> Result<(), String> {
        let mut tx = conn
            .transaction()
            .map_err(|e| format!("begin module store transaction failed: {e}"))?;
        Self::save_module_store(
            &mut tx,
            "store_citizens",
            &CitizenStoreSnapshot::from_store(store),
        )?;
        Self::save_module_store(&mut tx, "store_cpms", &CpmsStoreSnapshot::from_store(store))?;
        Self::save_module_store(
            &mut tx,
            "store_institutions",
            &InstStoreSnapshot::from_store(store),
        )?;
        Self::save_module_store(&mut tx, "store_ops", &OpsStoreSnapshot::from_store(store))?;
        tx.commit()
            .map_err(|e| format!("commit module store transaction failed: {e}"))?;

        let mut tx = conn
            .transaction()
            .map_err(|e| format!("begin admin sync transaction failed: {e}"))?;
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
                    "INSERT INTO admins(admin_id, admin_pubkey, admin_name, role, built_in, created_by, created_at, updated_at, city)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     RETURNING admin_id",
                    &[
                        &(admin.id as i64),
                        &admin.admin_pubkey,
                        &admin.admin_name,
                        &Self::admin_role_text(&admin.role),
                        &admin.built_in,
                        &admin.created_by,
                        &admin.created_at,
                        &admin.updated_at.unwrap_or(admin.created_at),
                        &admin.city,
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

        // ADR-008 Phase 23e:旧全局密钥环状态已删,不再写兼容表。

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
                    "DROP TABLE IF EXISTS runtime_store;
                 DROP TABLE IF EXISTS runtime_misc;
                 DROP TABLE IF EXISTS runtime_cache_entries;
                 DROP TABLE IF EXISTS store_shards;
                 CREATE TABLE IF NOT EXISTS store_citizens (
                    id SMALLINT PRIMARY KEY CHECK (id = 1),
                    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 );
                 CREATE TABLE IF NOT EXISTS store_cpms (
                    id SMALLINT PRIMARY KEY CHECK (id = 1),
                    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 );
                 CREATE TABLE IF NOT EXISTS store_institutions (
                    id SMALLINT PRIMARY KEY CHECK (id = 1),
                    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 );
                 CREATE TABLE IF NOT EXISTS store_ops (
                    id SMALLINT PRIMARY KEY CHECK (id = 1),
                    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 );
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS admin_name TEXT NOT NULL DEFAULT '';
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS city TEXT NOT NULL DEFAULT '';",
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

    // 中文注释:启动期仅校验 SFID_SIGNING_SEED_HEX 可解码,供登录二维码系统签名使用。
    // 省管理员业务治理签名只走各自冷钱包,后端不再保存或缓存省级私钥。
    {
        let seed_hex = required_env("SFID_SIGNING_SEED_HEX");
        crypto::sr25519::try_load_signing_key_from_seed(seed_hex.as_str())
            .unwrap_or_else(|e| panic!("invalid SFID_SIGNING_SEED_HEX: {e}"));
    }
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
    // 中文注释:ShardedStore 现在只作为进程内分片缓存。
    // 主数据由模块 Store 表保存;旧 `store_shards` Postgres 整包分片已删除。
    let sharded_store: Arc<store_shards::ShardedStore> = {
        let backend: Arc<dyn store_shards::backend::ShardBackend> =
            Arc::new(store_shards::backend::MemoryShardBackend::new());
        Arc::new(store_shards::ShardedStore::new(backend))
    };
    let state = AppState {
        store,
        rate_limit_redis: Arc::new(redis_client),
        sharded_store,
    };
    seed_builtin_province_admins(&state);
    sync_builtin_province_admins(&state);
    info!("initialized runtime state with defaults");
    // 中文注释:任务卡 6 启动对账:按 sfid 工具市清单对齐全部公安局机构。
    app_core::runtime_ops::backfill_and_reconcile_public_security(&state);
    app_core::runtime_ops::cleanup_stale_citizen_bind_records(&state);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(async move {
        // 中文注释:启动时只从进程内 ShardedStore 后端加载空缓存。
        // 旧持久化分片迁移已删除;数据从模块 Store 快照装入后,
        // 通过 sync_public_security_to_sharded 等函数同步到进程内缓存。
        {
            if let Err(e) = state.sharded_store.bootstrap_global().await {
                warn!(error = %e, "sharded store bootstrap_global failed");
            }
            let admin_runtime = state.store.read().ok().map(|store| {
                (
                    store
                        .admin_users_by_pubkey
                        .iter()
                        .filter(|(_, admin)| admin.role == AdminRole::ShengAdmin)
                        .map(|(pubkey, admin)| (pubkey.clone(), admin.clone()))
                        .collect::<HashMap<_, _>>(),
                    store.sheng_admin_province_by_pubkey.clone(),
                    store.login_challenges.clone(),
                    store.qr_login_results.clone(),
                    store.admin_sessions.clone(),
                    store.metrics.clone(),
                    store.next_seq,
                    store.next_audit_seq,
                    store.next_admin_user_id,
                )
            });
            if let Some((
                admins,
                provinces,
                login_challenges,
                qr_login_results,
                admin_sessions,
                metrics,
                next_seq,
                next_audit_seq,
                next_admin_user_id,
            )) = admin_runtime
            {
                let _ = state
                    .sharded_store
                    .write_global(|g| {
                        g.global_admins = admins;
                        g.sheng_admin_province_by_pubkey = provinces;
                        g.login_challenges = login_challenges;
                        g.qr_login_results = qr_login_results;
                        g.admin_sessions = admin_sessions;
                        g.metrics = metrics;
                        g.next_seq = next_seq;
                        g.next_audit_seq = next_audit_seq;
                        g.next_admin_user_id = next_admin_user_id;
                    })
                    .await;
            }
        }

        // 中文注释:启动后把模块 Store 快照里的公安局机构同步到进程内分片缓存,
        // 保证详情页/列表按省读取时能看到最新机构和账户。
        app_core::runtime_ops::sync_public_security_to_sharded(&state).await;

        // 中文注释:启动后把 store_cpms 持久化授权恢复到分片缓存,供 ARCHIVE geo_seal 验真扫描。
        app_core::runtime_ops::sync_cpms_sites_to_sharded(&state).await;

        // 中文注释:CPMS 授权站点缓存为异步分片访问,恢复缓存后再清理孤儿授权。
        app_core::runtime_ops::cleanup_orphan_cpms_sites(&state).await;

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
            .route("/api/v1/admin/operators", get(admins::list_operators))
            .route(
                "/api/v1/admin/passkeys/register/start",
                post(admins::passkeys::start_passkey_registration),
            )
            .route(
                "/api/v1/admin/passkeys/register/confirm",
                post(admins::passkeys::confirm_passkey_registration),
            )
            .route(
                "/api/v1/admin/passkeys/register/complete",
                post(admins::passkeys::complete_passkey_registration),
            )
            .route(
                "/api/v1/admin/actions/prepare",
                post(admins::actions::prepare_admin_action),
            )
            .route(
                "/api/v1/admin/actions/commit",
                post(admins::actions::commit_admin_action),
            )
            .route(
                "/api/v1/admin/sheng-admins",
                get(admins::list_province_admins),
            )
            .route("/api/v1/admin/cpms-keys", get(cpms::list_cpms_keys))
            .route(
                "/api/v1/admin/cpms-keys/by-institution/:sfid_number",
                get(cpms::get_cpms_site_by_institution),
            )
            .route(
                "/api/v1/admin/cpms-keys/sfid/generate",
                post(cpms::generate_cpms_install_qr),
            )
            .route(
                "/api/v1/admin/cpms/archive/verify",
                post(cpms::archive_verify),
            )
            .route(
                "/api/v1/admin/cpms-keys/:sfid_number",
                delete(cpms::delete_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:sfid_number/revoke-token",
                post(cpms::revoke_install_token),
            )
            .route(
                "/api/v1/admin/cpms-keys/:sfid_number/reissue",
                post(cpms::reissue_install_token),
            )
            .route(
                "/api/v1/admin/cpms-keys/:sfid_number/disable",
                put(cpms::disable_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:sfid_number/enable",
                put(cpms::enable_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:sfid_number/revoke",
                put(cpms::revoke_cpms_keys),
            )
            // ADR-008 Phase 23e:`/api/v1/admin/chain/balance` 已下架(chain/balance 整目录删)。
            // 中文注释:机构/账户两层模型的 API
            // - GET  /api/v1/institution/check-name                      — 机构名称全国查重
            // - POST /api/v1/institution/create                          — 生成机构(不上链)
            // - POST /api/v1/institution/:sfid_number/account/create         — 只登记账户名称,不上链
            // - GET  /api/v1/institution/list                            — 按 scope 过滤的机构列表
            // - GET  /api/v1/institution/:sfid_number                        — 机构详情
            // - GET  /api/v1/institution/:sfid_number/accounts               — 账户列表
            // - DELETE /api/v1/institution/:sfid_number/account/:account_name — 删除未上链/已注销新增账户
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
                "/api/v1/institution/:sfid_number/account/create",
                post(institutions::handler::create_account),
            )
            .route(
                "/api/v1/institution/list",
                get(institutions::handler::list_institutions),
            )
            .route(
                "/api/v1/institution/:sfid_number",
                get(institutions::handler::get_institution)
                    // 两步式第二步:详情页更新机构名称/企业类型
                    .patch(institutions::handler::update_institution),
            )
            .route(
                "/api/v1/institution/:sfid_number/accounts",
                get(institutions::handler::list_accounts),
            )
            .route(
                "/api/v1/institution/:sfid_number/account/:account_name",
                delete(institutions::handler::delete_account),
            )
            // 机构资料库文档 CRUD
            .route(
                "/api/v1/institution/:sfid_number/documents",
                get(institutions::handler::list_documents)
                    .post(institutions::handler::upload_document),
            )
            .route(
                "/api/v1/institution/:sfid_number/documents/:doc_id/download",
                get(institutions::handler::download_document),
            )
            .route(
                "/api/v1/institution/:sfid_number/documents/:doc_id",
                delete(institutions::handler::delete_document),
            )
            // 任务卡 6:公安局跟 sfid 工具市清单对账
            .route(
                "/api/v1/public-security/reconcile",
                post(institutions::handler::reconcile_public_security),
            )
            .route(
                "/api/v1/admin/citizens/cpms-status-export/import",
                post(citizens::status_export_import::admin_import_cpms_status_export),
            )
            .route(
                "/api/v1/admin/audit-logs",
                get(audit::admin_list_audit_logs),
            )
            .route(
                "/api/v1/admin/citizens",
                get(citizens::handler::admin_list_citizens),
            )
            // ── 公民身份绑定 ──
            .route(
                "/api/v1/admin/citizen/bind/challenge",
                post(citizens::binding::citizen_bind_challenge),
            )
            .route(
                "/api/v1/admin/citizen/bind",
                post(citizens::binding::citizen_bind),
            )
            .route("/api/v1/admin/sfid/meta", get(sfid::admin::admin_sfid_meta))
            .route(
                "/api/v1/admin/sfid/cities",
                get(sfid::admin::admin_sfid_cities),
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
                get(citizens::handler::public_identity_search),
            );

        // App routes:手机 App 与节点桌面 chain pull 用的统一命名空间。
        //
        // 全部端点都汇集在 chain/ 子目录(duoqian_info / joint_vote / citizen_vote)。
        // wuminapp 自有功能(钱包交易索引、电子护照状态查询)继续留 indexer / citizens 模块。
        let app_routes = Router::new()
            // ── 联合投票:获取公民人数快照凭证 ──
            .route(
                "/api/v1/app/voters/count",
                get(citizens::chain_joint_vote::app_voters_count),
            )
            // ── 公民投票凭证签发 ──
            .route(
                "/api/v1/app/vote/credential",
                post(citizens::chain_vote::app_vote_credential),
            )
            // ── 钱包交易索引(wuminapp 自有,与链交互无关) ──
            .route(
                "/api/v1/app/wallet/:address/transactions",
                get(indexer::api::wallet_transactions),
            )
            // ── wuminapp 电子护照状态查询 ──
            .route(
                "/api/v1/app/myid/status",
                get(citizens::vote::app_myid_status),
            )
            // ── 机构信息查询(链端/钱包 pull):机构搜索 / 详情 / 注册信息凭证 / 账户列表 ──
            .route(
                "/api/v1/app/institutions/search",
                get(institutions::chain_duoqian_info::app_search_institutions),
            )
            .route(
                "/api/v1/app/institutions/:sfid_number/registration-info",
                get(institutions::chain_duoqian_info::app_get_institution_registration_info),
            )
            .route(
                "/api/v1/app/institutions/:sfid_number",
                get(institutions::chain_duoqian_info::app_get_institution),
            )
            .route(
                "/api/v1/app/institutions/:sfid_number/accounts",
                get(institutions::chain_duoqian_info::app_list_accounts),
            )
            // ── 清算行搜索(已激活,wuminapp 绑定清算行用):资格白名单 + 主账户 ACTIVE_ON_CHAIN ──
            .route(
                "/api/v1/app/clearing-banks/search",
                get(institutions::chain_duoqian_info::app_search_clearing_banks),
            )
            // ── 候选清算行搜索(可未激活,节点桌面"添加清算行"用):仅资格白名单过滤 ──
            .route(
                "/api/v1/app/clearing-banks/eligible-search",
                get(institutions::chain_duoqian_info::app_search_eligible_clearing_banks),
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

        // 中文注释:SFID 后端启动时只初始化链 genesis;管理员业务签名由
        // 各省/市管理员自己的冷钱包完成,后端不持有管理员业务私钥。
        app_core::chain_runtime::init_genesis_hash_from_chain()
            .await
            .unwrap_or_else(|e| panic!("failed to initialize chain genesis hash: {e}"));
        info!("chain genesis hash initialized");

        // 中文注释:Passkey 绑定必须受 WebAuthn RP ID / Origin 约束;
        // 生产环境在启动期强制校验为 sfid.crcfrcn.com。
        admins::passkeys::validate_passkey_configuration()
            .unwrap_or_else(|e| panic!("invalid SFID Passkey configuration: {e}"));
        info!("passkey webauthn configuration validated");

        // 中文注释:省级管理员采用同级模型;43 个初始省级管理员只作为
        // 不可删除安全根,新增省级管理员走 admins 安全动作落本地管理表。

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
// 链端 chain pull 端点(duoqian_info / joint_vote / citizen_vote)无 attestor
// 鉴权需求,全局 rate limiter 已防滥用,凭证签名本身就是反伪造保护。

fn api_error(status: StatusCode, code: u32, message: &str) -> axum::response::Response {
    (
        status,
        Json(ApiError {
            code,
            error_code: sfid_error_code(status, message),
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
        .into_response()
}

fn sfid_error_code(status: StatusCode, message: &str) -> &'static str {
    // 中文注释:HTTP 状态表达协议层含义,稳定 error_code 表达业务语义;前端不得解析 message。
    match message {
        "missing bearer token" => "SFID_AUTH_MISSING_TOKEN",
        "invalid access token" => "SFID_AUTH_INVALID_ACCESS_TOKEN",
        "access token expired" => "SFID_AUTH_ACCESS_TOKEN_EXPIRED",
        "admin disabled" => "SFID_AUTH_ADMIN_DISABLED",
        "permission denied" => "SFID_AUTH_PERMISSION_DENIED",
        "challenge not found" | "challenge not found or expired" => "SFID_BIND_CHALLENGE_NOT_FOUND",
        "challenge already consumed" => "SFID_BIND_CHALLENGE_CONSUMED",
        "challenge expired" => "SFID_BIND_CHALLENGE_EXPIRED",
        "challenge wallet mismatch" | "challenge context mismatch" => "SFID_BIND_WALLET_MISMATCH",
        "signature verify failed" => "SFID_BIND_SIGNATURE_VERIFY_FAILED",
        "invalid signature hex" => "SFID_BIND_SIGNATURE_FORMAT_INVALID",
        "archive_no already bound" => "SFID_BIND_ARCHIVE_ALREADY_BOUND",
        "archive_no immutable after binding" => "SFID_BIND_ARCHIVE_IMMUTABLE",
        "wallet_pubkey already bound" => "SFID_BIND_WALLET_ALREADY_BOUND",
        "archive signature invalid" => "SFID_CITIZEN_ARCHIVE_SIGNATURE_BAD",
        "geo_seal cannot be decrypted" => "SFID_CITIZEN_ARCHIVE_GEO_SEAL_INVALID",
        "geo_seal install scope mismatch" => "SFID_CITIZEN_ARCHIVE_SCOPE_MISMATCH",
        "cpms_pubkey does not match installed CPMS" => "SFID_CITIZEN_ARCHIVE_PUBKEY_MISMATCH",
        "qr expired" => "SFID_CITIZEN_QR_EXPIRED",
        "qr header invalid" => "SFID_CITIZEN_QR_HEADER_INVALID",
        _ if status == StatusCode::UNAUTHORIZED => "SFID_AUTH_UNAUTHORIZED",
        _ if status == StatusCode::FORBIDDEN => "SFID_AUTH_FORBIDDEN",
        _ if status == StatusCode::BAD_REQUEST => "SFID_REQUEST_INVALID",
        _ if status == StatusCode::NOT_FOUND => "SFID_RESOURCE_NOT_FOUND",
        _ if status == StatusCode::CONFLICT => "SFID_RESOURCE_CONFLICT",
        _ if status == StatusCode::GONE => "SFID_RESOURCE_EXPIRED",
        _ if status == StatusCode::UNPROCESSABLE_ENTITY => "SFID_BUSINESS_VALIDATION_FAILED",
        _ if status == StatusCode::TOO_MANY_REQUESTS => "SFID_RATE_LIMITED",
        _ if status == StatusCode::SERVICE_UNAVAILABLE => "SFID_SERVICE_UNAVAILABLE",
        _ => "SFID_INTERNAL_ERROR",
    }
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
