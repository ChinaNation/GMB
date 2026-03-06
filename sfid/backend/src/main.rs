use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use blake3;
use chrono::{DateTime, NaiveDate, Utc};
use postgres::config::Host;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
};
use tracing::{info, warn};
use uuid::Uuid;

mod app_core;
mod business;
mod chain;
#[path = "key-admins/mod.rs"]
mod key_admins;
mod login;
mod models;
mod operate;
#[path = "operator-admins/mod.rs"]
mod operator_admins;
mod sfid;
#[path = "super-admins/mod.rs"]
mod super_admins;
use business::scope::{in_scope, in_scope_cpms_site, in_scope_pending};
use key_admins::chain_keyring::ChainKeyringState;

pub(crate) use app_core::http_security::*;
pub(crate) use app_core::runtime_ops::*;
pub(crate) use login::{
    build_admin_display_name, parse_sr25519_pubkey, parse_sr25519_pubkey_bytes, require_admin_any,
    require_admin_write, require_key_admin, require_super_admin, require_super_or_key_admin,
    require_super_or_operator_or_key_admin, verify_admin_signature,
};
pub(crate) use models::*;

#[derive(Clone)]
struct AppState {
    store: StoreHandle,
    signing_seed_hex: Arc<RwLock<String>>,
    known_key_seeds: Arc<RwLock<HashMap<String, String>>>,
    request_limits: Arc<Mutex<HashMap<String, Vec<DateTime<Utc>>>>>,
    key_id: String,
    key_version: String,
    key_alg: String,
    public_key_hex: Arc<RwLock<String>>,
}

#[derive(Clone)]
struct StoreHandle {
    backend: StoreBackend,
    write_gate: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Clone)]
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
            warn!(error = %err, "failed to persist store to database");
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
        match role {
            "KEY_ADMIN" => AdminRole::KeyAdmin,
            "SUPER_ADMIN" => AdminRole::SuperAdmin,
            "OPERATOR_ADMIN" => AdminRole::OperatorAdmin,
            "QUERY_ONLY" => AdminRole::QueryOnly,
            _ => AdminRole::OperatorAdmin,
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
            AdminRole::SuperAdmin => "SUPER_ADMIN",
            AdminRole::OperatorAdmin => "OPERATOR_ADMIN",
            AdminRole::QueryOnly => "QUERY_ONLY",
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
        store.super_admin_province_by_pubkey.clear();
        store.chain_keyring_state = None;

        let admin_rows = conn
            .query(
                "SELECT admin_id, admin_pubkey, admin_name, role, status, built_in, created_by, created_at
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
                },
            );
        }

        let super_rows = conn
            .query(
                "SELECT a.admin_pubkey, s.province_name
                 FROM super_admin_scope s
                 JOIN admins a ON a.admin_id=s.admin_id",
                &[],
            )
            .map_err(|e| format!("load super_admin_scope failed: {e}"))?;
        for row in super_rows {
            let pubkey: String = row.get(0);
            let province: String = row.get(1);
            store
                .super_admin_province_by_pubkey
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
        misc.super_admin_province_by_pubkey.clear();
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
        tx.execute("DELETE FROM operator_admin_scope", &[])
            .map_err(|e| format!("clear operator_admin_scope failed: {e}"))?;
        tx.execute("DELETE FROM super_admin_scope", &[])
            .map_err(|e| format!("clear super_admin_scope failed: {e}"))?;
        tx.execute("DELETE FROM admins", &[])
            .map_err(|e| format!("clear admins failed: {e}"))?;

        let mut admin_id_by_pubkey: HashMap<String, i64> = HashMap::new();
        for admin in store.admin_users_by_pubkey.values() {
            let row = tx
                .query_one(
                    "INSERT INTO admins(admin_id, admin_pubkey, admin_name, role, status, built_in, created_by, created_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
                    ],
                )
                .map_err(|e| format!("insert admins failed: {e}"))?;
            let admin_id: i64 = row.get(0);
            admin_id_by_pubkey.insert(admin.admin_pubkey.clone(), admin_id);
        }

        for province in store.super_admin_province_by_pubkey.values() {
            tx.execute(
                "INSERT INTO provinces(province_name) VALUES ($1)
                 ON CONFLICT (province_name) DO NOTHING",
                &[province],
            )
            .map_err(|e| format!("upsert provinces failed: {e}"))?;
        }

        for (pubkey, province) in &store.super_admin_province_by_pubkey {
            let Some(admin_id) = admin_id_by_pubkey.get(pubkey) else {
                continue;
            };
            tx.execute(
                "INSERT INTO super_admin_scope(admin_id, province_name) VALUES ($1, $2)",
                &[admin_id, province],
            )
            .map_err(|e| format!("insert super_admin_scope failed: {e}"))?;
        }

        for admin in store.admin_users_by_pubkey.values() {
            if admin.role != AdminRole::OperatorAdmin {
                continue;
            }
            let Some(admin_id) = admin_id_by_pubkey.get(&admin.admin_pubkey) else {
                continue;
            };
            let Some(super_admin_id) = admin_id_by_pubkey.get(&admin.created_by) else {
                continue;
            };
            let province = store
                .super_admin_province_by_pubkey
                .get(&admin.created_by)
                .cloned();
            tx.execute(
                "INSERT INTO operator_admin_scope(admin_id, super_admin_id, province_name)
                 VALUES ($1, $2, $3)",
                &[admin_id, super_admin_id, &province],
            )
            .map_err(|e| format!("insert operator_admin_scope failed: {e}"))?;
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
                 CREATE TABLE IF NOT EXISTS runtime_meta (
                    id INTEGER PRIMARY KEY,
                    payload JSONB NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 );
                 ALTER TABLE runtime_meta ADD COLUMN IF NOT EXISTS payload_enc BYTEA;
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS admin_name TEXT NOT NULL DEFAULT '';",
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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    let _ = required_env("SFID_CHAIN_TOKEN");
    let _ = required_env("SFID_CHAIN_SIGNING_SECRET");
    let _ = required_env("SFID_PUBLIC_SEARCH_TOKEN");
    let _ = required_env("SFID_RUNTIME_META_KEY");

    let main_seed = required_env("SFID_SIGNING_SEED_HEX");
    let main_key = key_admins::chain_keyring::load_signing_key_from_seed(main_seed.as_str());
    let public_key_hex = format!("0x{}", hex::encode(main_key.public.to_bytes()));
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
    let state = AppState {
        store,
        signing_seed_hex: Arc::new(RwLock::new(main_seed)),
        known_key_seeds: Arc::new(RwLock::new(known_key_seeds)),
        request_limits: Arc::new(Mutex::new(HashMap::new())),
        key_id: required_env("SFID_KEY_ID"),
        key_version: "v1".to_string(),
        key_alg: "sr25519".to_string(),
        public_key_hex: Arc::new(RwLock::new(public_key_hex)),
    };
    if load_runtime_state(&state) {
        key_admins::seed_key_admins(&state);
        persist_runtime_state(&state);
        info!("loaded persisted runtime state from database");
    } else {
        seed_super_admins(&state);
        key_admins::seed_chain_keyring(&state);
        key_admins::seed_key_admins(&state);
        persist_runtime_state(&state);
        info!("initialized runtime state with defaults");
    }
    seed_demo_record(&state);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(async move {
        tokio::spawn(bind_callback_worker(state.clone()));

        let auth_routes = Router::new()
            .route("/api/v1/admin/auth/check", get(login::admin_auth_check))
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
                get(super_admins::list_operators).post(super_admins::create_operator),
            )
            .route(
                "/api/v1/admin/operators/:id",
                put(super_admins::update_operator).delete(super_admins::delete_operator),
            )
            .route(
                "/api/v1/admin/operators/:id/status",
                put(super_admins::update_operator_status),
            )
            .route(
                "/api/v1/admin/super-admins",
                get(super_admins::list_super_admins),
            )
            .route(
                "/api/v1/admin/super-admins/:province",
                put(super_admins::replace_super_admin),
            )
            .route("/api/v1/admin/cpms-keys", get(super_admins::list_cpms_keys))
            .route(
                "/api/v1/admin/cpms-keys/sfid/generate",
                post(super_admins::generate_cpms_institution_sfid_qr),
            )
            .route(
                "/api/v1/admin/cpms-keys/register-scan",
                post(super_admins::register_cpms_keys_scan),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid",
                put(super_admins::update_cpms_keys).delete(super_admins::delete_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid/disable",
                put(super_admins::disable_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-keys/:site_sfid/revoke",
                put(super_admins::revoke_cpms_keys),
            )
            .route(
                "/api/v1/admin/cpms-status/scan",
                post(operator_admins::admin_cpms_status_scan),
            )
            .route(
                "/api/v1/admin/audit-logs",
                get(business::audit::admin_list_audit_logs),
            )
            .route(
                "/api/v1/admin/citizens",
                get(business::query::admin_list_citizens),
            )
            .route(
                "/api/v1/admin/bind/scan",
                post(operate::binding::admin_bind_scan),
            )
            .route(
                "/api/v1/admin/bind/query",
                get(business::query::admin_query_by_pubkey),
            )
            .route(
                "/api/v1/admin/bind/confirm",
                post(operate::binding::admin_bind_confirm),
            )
            .route(
                "/api/v1/admin/bind/unbind",
                post(operate::binding::admin_unbind),
            )
            .route("/api/v1/admin/sfid/meta", get(sfid::admin::admin_sfid_meta))
            .route(
                "/api/v1/admin/sfid/cities",
                get(sfid::admin::admin_sfid_cities),
            )
            .route(
                "/api/v1/admin/sfid/generate",
                post(sfid::admin::admin_generate_sfid),
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

        let chain_routes = Router::new()
            .route(
                "/api/v1/bind/request",
                post(chain::binding::create_bind_request),
            )
            .route("/api/v1/bind/result", get(chain::binding::get_bind_result))
            .route(
                "/api/v1/vote/verify",
                post(chain::vote::verify_vote_eligibility),
            )
            .route(
                "/api/v1/chain/voters/count",
                get(chain::voters::chain_voters_count),
            )
            .route(
                "/api/v1/chain/binding/validate",
                post(chain::binding::chain_binding_validate),
            )
            .route(
                "/api/v1/chain/reward/ack",
                post(chain::binding::chain_reward_ack),
            )
            .route(
                "/api/v1/chain/reward/state",
                get(chain::binding::chain_reward_state),
            )
            .route("/api/v1/attestor/public-key", get(attestor_public_key));

        let public_routes = Router::new()
            .route("/", get(root))
            .route("/api/v1/health", get(health))
            .route(
                "/api/v1/public/identity/search",
                get(business::query::public_identity_search),
            );

        let app = Router::new()
            .merge(public_routes)
            .merge(auth_routes)
            .merge(admin_routes)
            .merge(chain_routes)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                global_rate_limit_middleware,
            ))
            .layer(build_cors_layer())
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], 8899));
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

fn ensure_chain_request_db(
    state: &AppState,
    route_key: &str,
    auth: &ChainRequestAuth,
    fingerprint: &str,
) -> Result<(), axum::response::Response> {
    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return Ok(());
    };
    let insert = StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        conn.execute(
            "INSERT INTO chain_idempotency_requests(route_key, request_id, nonce, request_timestamp, fingerprint, created_at)
             VALUES ($1,$2,$3,$4,$5, now())",
            &[&route_key, &auth.request_id, &auth.nonce, &auth.timestamp, &fingerprint],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
    match insert {
        Ok(_) => Ok(()),
        Err(err) if err.contains("uq_chain_idempotency_route_nonce") => Err(api_error(
            StatusCode::CONFLICT,
            1016,
            "duplicate chain nonce",
        )),
        Err(err) if err.contains("uq_chain_idempotency_route_request") => Err(api_error(
            StatusCode::CONFLICT,
            1017,
            "duplicate chain request",
        )),
        Err(err) => {
            warn!("chain idempotency db insert failed: {err}");
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1500,
                "chain idempotency persistence failed",
            ))
        }
    }
}

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

fn persist_reward_state_db(state: &AppState, reward: &RewardStateRecord) {
    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return;
    };
    let _ = StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        conn.execute(
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
                &format!("{:?}", reward.reward_status).to_uppercase(),
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
        Ok(())
    });
}

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

fn parse_birth_date_from_archive_no(archive_no: &str) -> Option<NaiveDate> {
    let trimmed = archive_no.trim();
    if trimmed.len() < 8 {
        return None;
    }
    let birth_text = &trimmed[trimmed.len() - 8..];
    NaiveDate::parse_from_str(birth_text, "%Y%m%d").ok()
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
    state
        .store
        .read()
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed"))
}

pub(crate) fn store_write_or_500(
    state: &AppState,
) -> Result<StoreWriteGuard, axum::response::Response> {
    state.store.write().map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "store write failed",
        )
    })
}

fn load_runtime_state(state: &AppState) -> bool {
    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return false;
    };
    let cipher_key = runtime_meta_cipher_key();
    let row = match StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        conn.query_opt(
            "SELECT payload, payload_enc FROM runtime_meta WHERE id=1",
            &[],
        )
        .map_err(|e| format!("failed to load runtime_meta: {e}"))
    }) {
        Ok(v) => v,
        Err(err) => {
            warn!(error = %err, "failed to load runtime_meta");
            return false;
        }
    };
    let Some(row) = row else {
        return false;
    };
    let payload: serde_json::Value = row.get(0);
    let payload_enc: Option<Vec<u8>> = row.get(1);
    let snapshot: PersistedRuntimeMeta = match payload_enc {
        Some(ciphertext) if !ciphertext.is_empty() => {
            let decrypted_text =
                match StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
                    conn.query_one(
                        "SELECT pgp_sym_decrypt($1::bytea, $2)::text",
                        &[&ciphertext, &cipher_key],
                    )
                    .map(|row| row.get::<usize, String>(0))
                    .map_err(|e| format!("failed to decrypt runtime_meta payload: {e}"))
                }) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!(error = %err, "failed to decrypt runtime_meta");
                        return false;
                    }
                };
            match serde_json::from_str::<PersistedRuntimeMeta>(&decrypted_text) {
                Ok(v) => v,
                Err(err) => {
                    warn!(error = %err, "failed to decode decrypted runtime_meta");
                    return false;
                }
            }
        }
        _ => match serde_json::from_value(payload) {
            Ok(v) => v,
            Err(err) => {
                warn!(error = %err, "failed to decode runtime_meta");
                return false;
            }
        },
    };

    {
        let mut seed_guard = match state.signing_seed_hex.write() {
            Ok(v) => v,
            Err(_) => return false,
        };
        *seed_guard = snapshot.signing_seed_hex;
    }
    {
        let mut known_guard = match state.known_key_seeds.write() {
            Ok(v) => v,
            Err(_) => return false,
        };
        *known_guard = snapshot.known_key_seeds;
    }
    {
        let mut pubkey_guard = match state.public_key_hex.write() {
            Ok(v) => v,
            Err(_) => return false,
        };
        *pubkey_guard = snapshot.public_key_hex;
    }
    true
}

fn persist_runtime_state(state: &AppState) {
    let snapshot = PersistedRuntimeMeta {
        version: 1,
        signing_seed_hex: match state.signing_seed_hex.read() {
            Ok(v) => v.clone(),
            Err(_) => return,
        },
        known_key_seeds: match state.known_key_seeds.read() {
            Ok(v) => v.clone(),
            Err(_) => return,
        },
        public_key_hex: match state.public_key_hex.read() {
            Ok(v) => v.clone(),
            Err(_) => return,
        },
    };
    let payload_text = match serde_json::to_string(&snapshot) {
        Ok(v) => v,
        Err(err) => {
            warn!(error = %err, "failed to encode runtime_meta");
            return;
        }
    };
    let payload = serde_json::json!({"encrypted": true, "version": snapshot.version});
    let cipher_key = runtime_meta_cipher_key();

    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return;
    };
    if let Err(err) = StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
        conn.execute(
            "INSERT INTO runtime_meta(id, payload, payload_enc, updated_at)
             VALUES (1, $1, pgp_sym_encrypt($2, $3, 'cipher-algo=aes256,compress-algo=1'), now())
             ON CONFLICT (id) DO UPDATE SET
               payload=excluded.payload,
               payload_enc=excluded.payload_enc,
               updated_at=now()",
            &[&payload, &payload_text, &cipher_key],
        )
        .map(|_| ())
        .map_err(|e| format!("failed to persist runtime_meta: {e}"))
    }) {
        warn!(error = %err, "failed to persist runtime_meta");
    }
}

#[cfg(test)]
mod main_tests;
