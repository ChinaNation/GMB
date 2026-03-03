use axum::{
    extract::Request,
    extract::{Query, State},
    http::{header::HeaderName, HeaderMap, HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use blake3;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use hex::FromHex;
use reqwest::Url;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::Hash,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex, RwLock},
    thread,
};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use uuid::Uuid;

mod business;
#[path = "key-admins/mod.rs"]
mod key_admins;
#[path = "operator-admins/mod.rs"]
mod operator_admins;
#[path = "sfid-tool/mod.rs"]
mod sfid_tool;
#[path = "super-admins/mod.rs"]
mod super_admins;
use business::scope::{in_scope, in_scope_cpms_site, in_scope_pending, province_scope_for_role};
use key_admins::chain_keyring::ChainKeyringState;
use key_admins::chain_proof::{build_public_key_output, SignatureEnvelope};
use sfid_tool::province::{provinces, super_admin_display_name};

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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Store {
    next_seq: u64,
    next_audit_seq: u64,
    pending_by_pubkey: HashMap<String, PendingRequest>,
    bindings_by_pubkey: HashMap<String, BindingRecord>,
    pubkey_by_archive_index: HashMap<String, String>,
    admin_users_by_pubkey: HashMap<String, AdminUser>,
    super_admin_province_by_pubkey: HashMap<String, String>,
    login_challenges: HashMap<String, LoginChallenge>,
    qr_login_results: HashMap<String, QrLoginResultRecord>,
    admin_sessions: HashMap<String, AdminSession>,
    cpms_site_keys: HashMap<String, CpmsSiteKeys>,
    consumed_cpms_register_tokens: HashMap<String, DateTime<Utc>>,
    consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    pending_status_by_archive_no: HashMap<String, CitizenStatus>,
    pending_bind_scan_by_qr_id: HashMap<String, PendingBindScan>,
    generated_sfid_by_pubkey: HashMap<String, String>,
    chain_keyring_state: Option<ChainKeyringState>,
    keyring_rotate_challenges: HashMap<String, KeyringRotateChallenge>,
    audit_logs: Vec<AuditLogEntry>,
    chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    chain_nonce_seen: HashMap<String, DateTime<Utc>>,
    bind_callback_jobs: Vec<BindCallbackJob>,
    reward_state_by_pubkey: HashMap<String, RewardStateRecord>,
    vote_verify_cache: HashMap<String, VoteVerifyCacheEntry>,
    metrics: ServiceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedRuntimeMeta {
    version: u32,
    signing_seed_hex: String,
    known_key_seeds: HashMap<String, String>,
    public_key_hex: String,
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
        client: Arc<Mutex<postgres::Client>>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AdminRole {
    KeyAdmin,
    SuperAdmin,
    OperatorAdmin,
    QueryOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AdminStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum CpmsSiteStatus {
    Pending,
    Active,
    Disabled,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum CitizenStatus {
    Normal,
    Abnormal,
}

fn default_cpms_site_status() -> CpmsSiteStatus {
    CpmsSiteStatus::Active
}

fn default_cpms_site_version() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdminUser {
    id: u64,
    admin_pubkey: String,
    #[serde(default)]
    admin_name: String,
    role: AdminRole,
    status: AdminStatus,
    built_in: bool,
    created_by: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoginChallenge {
    challenge_id: String,
    admin_pubkey: String,
    challenge_text: String,
    challenge_token: String,
    qr_aud: String,
    qr_origin: String,
    origin: String,
    domain: String,
    session_id: String,
    nonce: String,
    issued_at: DateTime<Utc>,
    expire_at: DateTime<Utc>,
    consumed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdminSession {
    token: String,
    admin_pubkey: String,
    role: AdminRole,
    expire_at: DateTime<Utc>,
    #[serde(default = "default_now_utc")]
    last_active_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QrLoginResultRecord {
    session_id: String,
    access_token: String,
    expire_at: DateTime<Utc>,
    admin_pubkey: String,
    role: AdminRole,
    status: AdminStatus,
    created_at: DateTime<Utc>,
}

fn default_now_utc() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpmsSiteKeys {
    site_sfid: String,
    pubkey_1: String,
    pubkey_2: String,
    pubkey_3: String,
    #[serde(default = "default_cpms_site_status")]
    status: CpmsSiteStatus,
    #[serde(default = "default_cpms_site_version")]
    version: u64,
    #[serde(default)]
    last_register_issued_at: i64,
    #[serde(default)]
    init_qr_payload: Option<String>,
    admin_province: String,
    created_by: String,
    created_at: DateTime<Utc>,
    #[serde(default)]
    updated_by: Option<String>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
struct AdminAuthContext {
    admin_pubkey: String,
    role: AdminRole,
    admin_name: String,
    admin_province: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingRequest {
    seq: u64,
    account_pubkey: String,
    admin_province: Option<String>,
    requested_at: DateTime<Utc>,
    callback_url: Option<String>,
    client_request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingBindScan {
    qr_id: String,
    archive_no: String,
    site_sfid: String,
    status: CitizenStatus,
    expire_at: i64,
    scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyringRotateChallenge {
    challenge_id: String,
    keyring_version: u64,
    initiator_pubkey: String,
    challenge_text: String,
    expire_at: DateTime<Utc>,
    verified_at: Option<DateTime<Utc>>,
    consumed: bool,
    created_by: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BindingRecord {
    seq: u64,
    account_pubkey: String,
    archive_index: String,
    birth_date: Option<NaiveDate>,
    citizen_status: CitizenStatus,
    sfid_code: String,
    sfid_signature: String,
    bound_at: DateTime<Utc>,
    bound_by: String,
    admin_province: Option<String>,
    client_request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditLogEntry {
    seq: u64,
    action: String,
    actor_pubkey: String,
    target_pubkey: Option<String>,
    target_archive_no: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    actor_ip: Option<String>,
    result: String,
    detail: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ServiceMetrics {
    chain_auth_failures: u64,
    chain_replay_rejects: u64,
    bind_requests_total: u64,
    bind_confirms_total: u64,
    vote_verify_total: u64,
    binding_validate_total: u64,
    voters_count_total: u64,
    bind_callback_success_total: u64,
    bind_callback_retry_total: u64,
    bind_callback_failed_total: u64,
    chain_request_total: u64,
    chain_request_failed_total: u64,
    chain_latency_samples: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChainRequestReceipt {
    route_key: String,
    request_id: String,
    nonce: String,
    fingerprint: String,
    received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BindCallbackJob {
    callback_id: String,
    callback_url: String,
    payload: BindCallbackPayload,
    attempts: u32,
    max_attempts: u32,
    next_attempt_at: DateTime<Utc>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RewardStatus {
    Pending,
    Rewarded,
    RetryWaiting,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RewardStateRecord {
    account_pubkey: String,
    archive_index: String,
    callback_id: String,
    reward_status: RewardStatus,
    retry_count: u32,
    max_retries: u32,
    reward_tx_hash: Option<String>,
    last_error: Option<String>,
    next_retry_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VoteVerifyCacheEntry {
    account_pubkey: String,
    proposal_id: Option<u64>,
    is_bound: bool,
    has_vote_eligibility: bool,
    sfid_code: Option<String>,
    archive_index: Option<String>,
    citizen_status: Option<CitizenStatus>,
    cached_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct ChainRequestAuth {
    request_id: String,
    nonce: String,
    timestamp: i64,
}

#[derive(Deserialize)]
struct AuditLogsQuery {
    action: Option<String>,
    actor_pubkey: Option<String>,
    keyword: Option<String>,
    limit: Option<usize>,
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
        client: &Arc<Mutex<postgres::Client>>,
        op: impl FnOnce(&mut postgres::Client) -> Result<R, String> + Send,
    ) -> Result<R, String>
    where
        R: Send,
    {
        thread::scope(|scope| {
            let handle = scope.spawn(|| {
                let mut conn = client
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
            Self::Postgres { client } => {
                Self::with_postgres_client(client, Self::load_store_postgres)
            }
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
            Self::Postgres { client } => {
                let snapshot = store.clone();
                Self::with_postgres_client(client, move |conn| {
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
        let handle = thread::spawn(move || {
            let mut client = postgres::Client::connect(db_url.as_str(), postgres::NoTls)
                .map_err(|e| format!("connect postgres failed: {e}"))?;
            client
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
                 ALTER TABLE IF EXISTS admins
                   ADD COLUMN IF NOT EXISTS admin_name TEXT NOT NULL DEFAULT '';",
                )
                .map_err(|e| format!("init runtime tables failed: {e}"))?;
            Ok::<postgres::Client, String>(client)
        });
        let client = match handle.join() {
            Ok(v) => v?,
            Err(_) => return Err("postgres init thread panicked".to_string()),
        };
        Ok(Self {
            backend: StoreBackend::Postgres {
                client: Arc::new(Mutex::new(client)),
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
        let write_guard = loop {
            if let Ok(guard) = self.write_gate.clone().try_lock_owned() {
                break guard;
            }
            std::thread::yield_now();
        };
        Ok(StoreWriteGuard {
            store: self.backend.load_store()?,
            backend: self.backend.clone(),
            _write_guard: write_guard,
        })
    }
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    code: u32,
    message: String,
    data: T,
}

#[derive(Serialize)]
struct ApiError {
    code: u32,
    message: String,
    trace_id: String,
}

#[derive(Serialize)]
struct HealthData {
    service: &'static str,
    status: &'static str,
    checked_at: i64,
}

#[derive(Serialize, Deserialize)]
struct BindRequestInput {
    account_pubkey: String,
    callback_url: Option<String>,
    client_request_id: Option<String>,
}

#[derive(Serialize)]
struct BindRequestOutput {
    account_pubkey: String,
    chain_request_id: String,
    status: &'static str,
    message: &'static str,
}

#[derive(Deserialize)]
struct AdminQueryInput {
    account_pubkey: String,
}

#[derive(Serialize)]
struct AdminQueryOutput {
    account_pubkey: String,
    found_pending: bool,
    found_binding: bool,
    archive_index: Option<String>,
    sfid_code: Option<String>,
}

#[derive(Deserialize)]
struct AdminBindInput {
    account_pubkey: String,
    archive_index: String,
    qr_id: String,
}

#[derive(Deserialize)]
struct AdminUnbindInput {
    account_pubkey: String,
}

#[derive(Deserialize)]
struct CitizensQuery {
    keyword: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct PublicIdentitySearchQuery {
    archive_no: Option<String>,
    identity_code: Option<String>,
    account_pubkey: Option<String>,
}

#[derive(Serialize)]
struct PublicIdentitySearchOutput {
    found: bool,
    archive_no: Option<String>,
    identity_code: Option<String>,
    account_pubkey: Option<String>,
}

#[derive(Serialize)]
struct CitizenRow {
    seq: u64,
    account_pubkey: String,
    archive_index: Option<String>,
    sfid_code: Option<String>,
    citizen_status: Option<CitizenStatus>,
    is_bound: bool,
}

#[derive(Serialize)]
struct AdminBindOutput {
    account_pubkey: String,
    archive_index: String,
    sfid_code: String,
    proof: SignatureEnvelope,
    status: &'static str,
    message: &'static str,
}

#[derive(Deserialize)]
struct AdminGenerateSfidInput {
    account_pubkey: String,
    a3: String,
    p1: Option<String>,
    province: String,
    city: String,
    institution: String,
}

#[derive(Serialize)]
struct AdminGenerateSfidOutput {
    account_pubkey: String,
    sfid_code: String,
}

#[derive(Serialize)]
struct SfidOptionItem {
    label: &'static str,
    value: &'static str,
}

#[derive(Serialize)]
struct SfidProvinceItem {
    name: String,
    code: String,
}

#[derive(Serialize)]
struct SfidCityItem {
    name: String,
    code: String,
}

#[derive(Serialize)]
struct AdminSfidMetaOutput {
    a3_options: Vec<SfidOptionItem>,
    institution_options: Vec<SfidOptionItem>,
    provinces: Vec<SfidProvinceItem>,
    scoped_province: Option<String>,
}

#[derive(Deserialize)]
struct AdminSfidCitiesQuery {
    province: String,
}

#[derive(Serialize)]
struct AdminAuthOutput {
    ok: bool,
    admin_pubkey: String,
    role: AdminRole,
    admin_name: String,
    admin_province: Option<String>,
}

#[derive(Deserialize)]
struct AdminIdentifyInput {
    identity_qr: String,
}

#[derive(Serialize)]
struct AdminIdentifyOutput {
    admin_pubkey: String,
    role: AdminRole,
    status: AdminStatus,
    admin_name: String,
    admin_province: Option<String>,
}

#[derive(Deserialize)]
struct AdminChallengeInput {
    admin_pubkey: String,
    origin: Option<String>,
    domain: Option<String>,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct AdminChallengeOutput {
    challenge_id: String,
    challenge_payload: String,
    origin: String,
    domain: String,
    session_id: String,
    nonce: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct AdminQrChallengeInput {
    origin: Option<String>,
    domain: Option<String>,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct AdminQrChallengeOutput {
    challenge_id: String,
    challenge_payload: String,
    login_qr_payload: String,
    origin: String,
    domain: String,
    session_id: String,
    nonce: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct AdminQrCompleteInput {
    #[serde(alias = "request_id")]
    challenge_id: String,
    session_id: Option<String>,
    admin_pubkey: String,
    #[serde(default, alias = "pubkey", alias = "public_key")]
    signer_pubkey: Option<String>,
    signature: String,
}

#[derive(Deserialize)]
struct AdminQrResultQuery {
    challenge_id: String,
    session_id: String,
}

#[derive(Serialize)]
struct AdminQrResultOutput {
    status: String,
    message: String,
    access_token: Option<String>,
    expire_at: Option<i64>,
    admin: Option<AdminIdentifyOutput>,
}

#[derive(Deserialize)]
struct AdminVerifyInput {
    challenge_id: String,
    origin: String,
    domain: Option<String>,
    session_id: String,
    nonce: String,
    signature: String,
}

#[derive(Serialize)]
struct AdminVerifyOutput {
    access_token: String,
    expire_at: i64,
    admin: AdminIdentifyOutput,
}

#[derive(Serialize)]
struct OperatorRow {
    id: u64,
    admin_pubkey: String,
    admin_name: String,
    role: AdminRole,
    status: AdminStatus,
    built_in: bool,
    created_by: String,
    created_by_name: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct SuperAdminRow {
    id: u64,
    province: String,
    admin_pubkey: String,
    status: AdminStatus,
    built_in: bool,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct CreateOperatorInput {
    admin_pubkey: String,
    admin_name: String,
}

#[derive(Deserialize)]
struct ReplaceSuperAdminInput {
    admin_pubkey: String,
}

#[derive(Deserialize)]
struct UpdateOperatorInput {
    admin_pubkey: Option<String>,
    admin_name: Option<String>,
}

#[derive(Deserialize)]
struct UpdateOperatorStatusInput {
    status: AdminStatus,
}

#[derive(Deserialize)]
struct CpmsRegisterScanInput {
    qr_payload: String,
}

#[derive(Deserialize)]
struct GenerateCpmsInstitutionSfidInput {
    province: Option<String>,
    city: String,
    institution: String,
}

#[derive(Deserialize)]
struct UpdateCpmsKeysInput {
    pubkey_1: String,
    pubkey_2: String,
    pubkey_3: String,
}

#[derive(Deserialize)]
struct UpdateCpmsSiteStatusInput {
    reason: Option<String>,
}

#[derive(Deserialize)]
struct BindScanInput {
    qr_payload: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CitizenQrPayload {
    ver: String,
    issuer_id: String,
    site_sfid: String,
    archive_no: String,
    issued_at: i64,
    expire_at: i64,
    qr_id: String,
    sig_alg: String,
    status: CitizenStatus,
    signature: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CitizenStatusQrPayload {
    ver: String,
    issuer_id: String,
    site_sfid: String,
    archive_no: String,
    status: CitizenStatus,
    issued_at: i64,
    expire_at: i64,
    qr_id: String,
    sig_alg: String,
    signature: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CpmsRegisterQrPayload {
    site_sfid: String,
    pubkey_1: String,
    pubkey_2: String,
    pubkey_3: String,
    issued_at: i64,
    checksum_or_signature: String,
    init_qr_payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpmsInstitutionInitQrPayload {
    ver: String,
    issuer_id: String,
    purpose: String,
    site_sfid: String,
    a3: String,
    p1: String,
    province: String,
    city: String,
    institution: String,
    issued_at: i64,
    expire_at: i64,
    qr_id: String,
    sig_alg: String,
    key_id: String,
    key_version: String,
    public_key: String,
    signature: String,
}

#[derive(Serialize)]
struct CpmsRegisterScanOutput {
    site_sfid: String,
    status: &'static str,
    message: &'static str,
}

#[derive(Serialize)]
struct GenerateCpmsInstitutionSfidOutput {
    site_sfid: String,
    issued_at: i64,
    expire_at: i64,
    qr_payload: String,
}

#[derive(Serialize)]
struct BindScanOutput {
    site_sfid: String,
    archive_no: String,
    qr_id: String,
    status: CitizenStatus,
    issued_at: i64,
    expire_at: i64,
}

#[derive(Serialize, Deserialize)]
struct BindResultQuery {
    account_pubkey: String,
}

#[derive(Serialize)]
struct BindResultOutput {
    account_pubkey: String,
    is_bound: bool,
    sfid_code: Option<String>,
    sfid_signature: Option<String>,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct VoteVerifyInput {
    account_pubkey: String,
    proposal_id: Option<u64>,
    challenge: Option<String>,
}

#[derive(Serialize)]
struct VoteVerifyOutput {
    account_pubkey: String,
    is_bound: bool,
    has_vote_eligibility: bool,
    sfid_code: Option<String>,
    vote_token: Option<SignatureEnvelope>,
    message: String,
}

#[derive(Serialize)]
struct ChainVotersCountOutput {
    total_voters: usize,
    as_of: i64,
}

#[derive(Serialize, Deserialize)]
struct ChainBindingValidateInput {
    archive_no: String,
    account_pubkey: String,
}

#[derive(Serialize)]
struct ChainBindingValidateOutput {
    is_bound: bool,
    is_voting_eligible: bool,
    citizen_status: Option<CitizenStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RewardAckStatusInput {
    Success,
    Failed,
}

#[derive(Serialize, Deserialize)]
struct RewardAckInput {
    account_pubkey: String,
    callback_id: String,
    status: RewardAckStatusInput,
    reward_tx_hash: Option<String>,
    error_message: Option<String>,
    retry_after_seconds: Option<u64>,
}

#[derive(Serialize)]
struct RewardAckOutput {
    account_pubkey: String,
    callback_id: String,
    reward_status: RewardStatus,
    retry_count: u32,
    next_retry_at: Option<i64>,
    message: String,
}

#[derive(Serialize)]
struct RewardStateOutput {
    account_pubkey: String,
    archive_index: String,
    callback_id: String,
    reward_status: RewardStatus,
    retry_count: u32,
    max_retries: u32,
    reward_tx_hash: Option<String>,
    last_error: Option<String>,
    next_retry_at: Option<i64>,
    updated_at: i64,
    created_at: i64,
}

#[derive(Serialize, Deserialize)]
struct RewardStateQuery {
    account_pubkey: String,
}

#[derive(Deserialize)]
struct CpmsStatusScanInput {
    qr_payload: String,
}

#[derive(Serialize)]
struct CpmsStatusScanOutput {
    archive_no: String,
    status: CitizenStatus,
    message: &'static str,
}

#[derive(Serialize)]
struct KeyringStateOutput {
    version: u64,
    main_pubkey: String,
    backup_a_pubkey: String,
    backup_b_pubkey: String,
    updated_at: i64,
}

#[derive(Deserialize)]
struct KeyringRotateChallengeInput {
    initiator_pubkey: String,
}

#[derive(Serialize)]
struct KeyringRotateChallengeOutput {
    challenge_id: String,
    keyring_version: u64,
    challenge_text: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct KeyringRotateCommitInput {
    challenge_id: String,
    signature: String,
    new_backup_pubkey: String,
    new_backup_seed_hex: Option<String>,
}

#[derive(Deserialize)]
struct KeyringRotateVerifyInput {
    challenge_id: String,
    signature: String,
}

#[derive(Serialize)]
struct KeyringRotateVerifyOutput {
    challenge_id: String,
    initiator_pubkey: String,
    keyring_version: u64,
    verified: bool,
    message: &'static str,
}

#[derive(Serialize)]
struct KeyringRotateCommitOutput {
    old_main_pubkey: String,
    promoted_slot: String,
    chain_tx_hash: String,
    chain_submit_ok: bool,
    chain_submit_error: Option<String>,
    version: u64,
    main_pubkey: String,
    backup_a_pubkey: String,
    backup_b_pubkey: String,
    updated_at: i64,
    message: String,
}

#[derive(Serialize)]
struct BindingPayload {
    kind: &'static str,
    version: &'static str,
    account_pubkey: String,
    archive_index: String,
    sfid_code: String,
    issued_at: i64,
}

#[derive(Serialize)]
struct VotePayload {
    kind: &'static str,
    version: &'static str,
    account_pubkey: String,
    sfid_code: String,
    proposal_id: Option<u64>,
    challenge: String,
    iat: i64,
    exp: i64,
    jti: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BindCallbackPayload {
    callback_id: String,
    event: String,
    account_pubkey: String,
    archive_index: String,
    sfid_code: String,
    status: String,
    bound_at: i64,
    proof: SignatureEnvelope,
    client_request_id: Option<String>,
    callback_attestation: SignatureEnvelope,
}

#[derive(Serialize)]
struct BindCallbackSignablePayload {
    callback_id: String,
    event: String,
    account_pubkey: String,
    archive_index: String,
    sfid_code: String,
    status: String,
    bound_at: i64,
    proof: SignatureEnvelope,
    client_request_id: Option<String>,
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
            .route("/api/v1/admin/auth/check", get(admin_auth_check))
            .route("/api/v1/admin/auth/identify", post(admin_auth_identify))
            .route("/api/v1/admin/auth/challenge", post(admin_auth_challenge))
            .route("/api/v1/admin/auth/verify", post(admin_auth_verify))
            .route(
                "/api/v1/admin/auth/qr/challenge",
                post(admin_auth_qr_challenge),
            )
            .route(
                "/api/v1/admin/auth/qr/complete",
                post(admin_auth_qr_complete),
            )
            .route("/api/v1/admin/auth/qr/result", get(admin_auth_qr_result));

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
                post(business::binding::admin_bind_scan),
            )
            .route(
                "/api/v1/admin/bind/query",
                get(business::query::admin_query_by_pubkey),
            )
            .route(
                "/api/v1/admin/bind/confirm",
                post(business::binding::admin_bind_confirm),
            )
            .route(
                "/api/v1/admin/bind/unbind",
                post(business::binding::admin_unbind),
            )
            .route(
                "/api/v1/admin/sfid/meta",
                get(business::sfid::admin_sfid_meta),
            )
            .route(
                "/api/v1/admin/sfid/cities",
                get(business::sfid::admin_sfid_cities),
            )
            .route(
                "/api/v1/admin/sfid/generate",
                post(business::sfid::admin_generate_sfid),
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
                require_admin_session_middleware,
            ));

        let chain_routes = Router::new()
            .route(
                "/api/v1/bind/request",
                post(business::binding::create_bind_request),
            )
            .route(
                "/api/v1/bind/result",
                get(business::binding::get_bind_result),
            )
            .route(
                "/api/v1/vote/verify",
                post(business::status::verify_vote_eligibility),
            )
            .route(
                "/api/v1/chain/voters/count",
                get(business::query::chain_voters_count),
            )
            .route(
                "/api/v1/chain/binding/validate",
                post(business::query::chain_binding_validate),
            )
            .route(
                "/api/v1/chain/reward/ack",
                post(business::query::chain_reward_ack),
            )
            .route(
                "/api/v1/chain/reward/state",
                get(business::query::chain_reward_state),
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
        axum::serve(listener, app)
            .await
            .expect("run sfid backend server");
    });
}

async fn require_admin_session_middleware(
    State(state): State<AppState>,
    request: Request,
    next: middleware::Next,
) -> Response {
    if let Err(resp) = admin_auth(&state, request.headers()) {
        return resp;
    }
    next.run(request).await
}

async fn global_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: middleware::Next,
) -> Response {
    let now = Utc::now();
    let limit_per_min = std::env::var("SFID_RATE_LIMIT_PER_MIN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(120);
    let actor = actor_ip_from_headers(request.headers())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    {
        let mut limits = match state.request_limits.lock() {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "rate limiter unavailable",
                );
            }
        };
        let bucket = limits.entry(actor.clone()).or_default();
        bucket.retain(|seen_at| *seen_at > now - Duration::minutes(1));
        if bucket.len() >= limit_per_min {
            return api_error(StatusCode::TOO_MANY_REQUESTS, 1029, "rate limit exceeded");
        }
        bucket.push(now);
        if limits.len() > 10_000 {
            limits.retain(|_, seen| !seen.is_empty());
        }
    }

    next.run(request).await
}

fn required_env(key: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => panic!("{key} is required and must be non-empty"),
    }
}

fn build_cors_layer() -> CorsLayer {
    let configured = std::env::var("SFID_CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .filter(|v| *v != "*")
                .filter_map(|v| HeaderValue::from_str(v).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let origins = if configured.is_empty() {
        vec![
            HeaderValue::from_static("http://127.0.0.1:5179"),
            HeaderValue::from_static("http://localhost:5179"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
            HeaderValue::from_static("http://localhost:5173"),
        ]
    } else {
        configured
    };
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-chain-token"),
            HeaderName::from_static("x-chain-request-id"),
            HeaderName::from_static("x-chain-nonce"),
            HeaderName::from_static("x-chain-timestamp"),
            HeaderName::from_static("x-chain-signature"),
            HeaderName::from_static("x-wallet-pubkey"),
            HeaderName::from_static("x-wallet-signature"),
            HeaderName::from_static("x-wallet-signature-message"),
        ])
}

async fn root() -> impl IntoResponse {
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "sfid backend is running",
    })
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let store = match state.store.read() {
        Ok(guard) => guard,
        Err(err) => {
            warn!("store read failed in /api/v1/health: {}", err);
            return Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: HealthData {
                    service: "sfid-backend",
                    status: "DEGRADED",
                    checked_at: Utc::now().timestamp(),
                },
            });
        }
    };
    let _ = latency_p95_p99_ms(&store.metrics.chain_latency_samples);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: HealthData {
            service: "sfid-backend",
            status: "UP",
            checked_at: Utc::now().timestamp(),
        },
    })
}

async fn attestor_public_key(State(state): State<AppState>) -> impl IntoResponse {
    let public_key_hex = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "public key unavailable",
            )
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: build_public_key_output(
            &state.key_id,
            &state.key_version,
            &state.key_alg,
            &public_key_hex,
        ),
    })
    .into_response()
}

async fn admin_auth_check(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let ctx = match admin_auth(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminAuthOutput {
            ok: true,
            admin_pubkey: ctx.admin_pubkey,
            role: ctx.role,
            admin_name: ctx.admin_name,
            admin_province: ctx.admin_province,
        },
    })
    .into_response()
}

async fn admin_auth_identify(
    State(state): State<AppState>,
    Json(input): Json<AdminIdentifyInput>,
) -> impl IntoResponse {
    let admin_pubkey = parse_admin_identity_qr(&input.identity_qr);
    if admin_pubkey.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "identity_qr is required");
    }

    let store = match state.store.read() {
        Ok(guard) => guard,
        Err(err) => {
            warn!("store read failed in /api/v1/admin/auth/identify: {}", err);
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, "store unavailable");
        }
    };
    let Some(admin) = store.admin_users_by_pubkey.get(&admin_pubkey) else {
        return api_error(StatusCode::FORBIDDEN, 2002, "admin not found");
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminIdentifyOutput {
            admin_pubkey: admin.admin_pubkey.clone(),
            role: admin.role.clone(),
            status: admin.status.clone(),
            admin_name: {
                let province = province_scope_for_role(&store, &admin.admin_pubkey, &admin.role);
                build_admin_display_name_from_user(admin, province.as_deref())
            },
            admin_province: province_scope_for_role(&store, &admin.admin_pubkey, &admin.role),
        },
    })
    .into_response()
}

async fn admin_auth_challenge(
    State(state): State<AppState>,
    Json(input): Json<AdminChallengeInput>,
) -> impl IntoResponse {
    if input.admin_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey is required");
    }
    let origin = input.origin.unwrap_or_default().trim().to_string();
    if origin.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "origin is required");
    }
    let session_id = input.session_id.unwrap_or_default().trim().to_string();
    if session_id.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "session_id is required");
    }
    let derived_domain = extract_domain_from_origin(&origin)
        .or_else(|| input.domain.clone())
        .unwrap_or_default();
    if derived_domain.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "domain is required");
    }

    let now = Utc::now();
    let expire_at = now + Duration::minutes(2);
    let challenge_id = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();
    let challenge_text = format!(
        "sfid-login|pubkey={}|origin={}|domain={}|session_id={}|nonce={}|iat={}|exp={}",
        input.admin_pubkey,
        origin,
        derived_domain,
        session_id,
        nonce,
        now.timestamp(),
        expire_at.timestamp()
    );

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);
    let Some(admin) = store.admin_users_by_pubkey.get(&input.admin_pubkey) else {
        return api_error(StatusCode::FORBIDDEN, 2002, "admin not found");
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }

    insert_bounded_map(
        &mut store.login_challenges,
        challenge_id.clone(),
        LoginChallenge {
            challenge_id: challenge_id.clone(),
            admin_pubkey: input.admin_pubkey,
            challenge_text: challenge_text.clone(),
            challenge_token: String::new(),
            qr_aud: String::new(),
            qr_origin: String::new(),
            origin: origin.clone(),
            domain: derived_domain.clone(),
            session_id: session_id.clone(),
            nonce: nonce.clone(),
            issued_at: now,
            expire_at,
            consumed: false,
        },
        bounded_cache_limit("SFID_LOGIN_CHALLENGE_MAX", 20_000),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminChallengeOutput {
            challenge_id,
            challenge_payload: challenge_text,
            origin,
            domain: derived_domain,
            session_id,
            nonce,
            expire_at: expire_at.timestamp(),
        },
    })
    .into_response()
}

async fn admin_auth_verify(
    State(state): State<AppState>,
    Json(input): Json<AdminVerifyInput>,
) -> impl IntoResponse {
    if input.challenge_id.trim().is_empty()
        || input.signature.trim().is_empty()
        || input.origin.trim().is_empty()
        || input.session_id.trim().is_empty()
        || input.nonce.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, origin, session_id, nonce, signature are required",
        );
    }
    let verify_domain = input
        .domain
        .clone()
        .or_else(|| extract_domain_from_origin(&input.origin))
        .unwrap_or_default();
    if verify_domain.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "domain is required");
    }

    let now = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);
    let admin_pubkey = {
        let Some(challenge) = store.login_challenges.get_mut(&input.challenge_id) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
        };
        if challenge.consumed {
            return api_error(StatusCode::CONFLICT, 1007, "challenge already consumed");
        }
        if now > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "challenge expired");
        }
        if challenge.origin != input.origin
            || challenge.domain != verify_domain
            || challenge.session_id != input.session_id
            || challenge.nonce != input.nonce
        {
            return api_error(StatusCode::UNAUTHORIZED, 2004, "challenge context mismatch");
        }

        if !verify_admin_signature(
            &challenge.admin_pubkey,
            &challenge.challenge_text,
            input.signature.trim(),
        ) {
            return api_error(StatusCode::UNAUTHORIZED, 2004, "signature verify failed");
        }
        challenge.consumed = true;
        challenge.admin_pubkey.clone()
    };

    let admin = match store.admin_users_by_pubkey.get(&admin_pubkey) {
        Some(v) => v,
        None => return api_error(StatusCode::FORBIDDEN, 2002, "admin not found"),
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }
    let admin_pubkey = admin.admin_pubkey.clone();
    let admin_role = admin.role.clone();
    let admin_status = admin.status.clone();
    let admin_province = province_scope_for_role(&store, &admin_pubkey, &admin_role);
    let admin_name = build_admin_display_name_from_user(admin, admin_province.as_deref());

    let access_token = Uuid::new_v4().to_string();
    let expire_at = now + Duration::hours(8);
    store.admin_sessions.insert(
        access_token.clone(),
        AdminSession {
            token: access_token.clone(),
            admin_pubkey: admin_pubkey.clone(),
            role: admin_role.clone(),
            expire_at,
            last_active_at: now,
        },
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminVerifyOutput {
            access_token,
            expire_at: expire_at.timestamp(),
            admin: AdminIdentifyOutput {
                admin_pubkey,
                role: admin_role,
                status: admin_status,
                admin_name,
                admin_province,
            },
        },
    })
    .into_response()
}

async fn admin_auth_qr_challenge(
    State(state): State<AppState>,
    Json(input): Json<AdminQrChallengeInput>,
) -> impl IntoResponse {
    let origin = input.origin.unwrap_or_default().trim().to_string();
    if origin.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "origin is required");
    }
    let session_id = input.session_id.unwrap_or_default().trim().to_string();
    if session_id.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "session_id is required");
    }
    let derived_domain = extract_domain_from_origin(&origin)
        .or_else(|| input.domain.clone())
        .unwrap_or_default();
    if derived_domain.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "domain is required");
    }

    let now = Utc::now();
    let expire_at = now + Duration::minutes(2);
    let challenge_id = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();
    let challenge_token = Uuid::new_v4().to_string();
    let qr_aud =
        std::env::var("SFID_LOGIN_QR_AUD").unwrap_or_else(|_| "sfid-local-app".to_string());
    let qr_origin =
        std::env::var("SFID_LOGIN_QR_ORIGIN").unwrap_or_else(|_| "sfid-device-id".to_string());
    let challenge_text = format!(
        "WUMINAPP_LOGIN_V1|{}|{}|{}|{}|{}|{}|{}",
        "sfid",
        qr_aud,
        qr_origin,
        challenge_id,
        challenge_token,
        nonce,
        expire_at.timestamp()
    );
    let login_qr_payload = serde_json::json!({
        "proto": "WUMINAPP_LOGIN_V1",
        "system": "sfid",
        "request_id": challenge_id,
        "challenge": challenge_token,
        "nonce": nonce,
        "issued_at": now.timestamp(),
        "expires_at": expire_at.timestamp(),
        "aud": qr_aud,
        "origin": qr_origin
    })
    .to_string();

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);
    insert_bounded_map(
        &mut store.login_challenges,
        challenge_id.clone(),
        LoginChallenge {
            challenge_id: challenge_id.clone(),
            admin_pubkey: String::new(),
            challenge_text: challenge_text.clone(),
            challenge_token,
            qr_aud,
            qr_origin,
            origin: origin.clone(),
            domain: derived_domain.clone(),
            session_id: session_id.clone(),
            nonce: nonce.clone(),
            issued_at: now,
            expire_at,
            consumed: false,
        },
        bounded_cache_limit("SFID_LOGIN_CHALLENGE_MAX", 20_000),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQrChallengeOutput {
            challenge_id,
            challenge_payload: challenge_text,
            login_qr_payload,
            origin,
            domain: derived_domain,
            session_id,
            nonce,
            expire_at: expire_at.timestamp(),
        },
    })
    .into_response()
}

async fn admin_auth_qr_complete(
    State(state): State<AppState>,
    Json(input): Json<AdminQrCompleteInput>,
) -> impl IntoResponse {
    if input.challenge_id.trim().is_empty()
        || input.admin_pubkey.trim().is_empty()
        || input.signature.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, admin_pubkey, signature are required",
        );
    }

    let now = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);

    let (challenge_text, session_id) = {
        let Some(challenge) = store.login_challenges.get_mut(&input.challenge_id) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
        };
        if challenge.consumed {
            return api_error(StatusCode::CONFLICT, 1007, "challenge already consumed");
        }
        if let Some(client_sid) = input.session_id.as_ref() {
            if challenge.session_id != client_sid.trim() {
                return api_error(StatusCode::FORBIDDEN, 1003, "challenge session mismatch");
            }
        }
        if now > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "challenge expired");
        }
        let verify_message = if !challenge.challenge_token.is_empty()
            && !challenge.qr_aud.is_empty()
            && !challenge.qr_origin.is_empty()
        {
            format!(
                "WUMINAPP_LOGIN_V1|{}|{}|{}|{}|{}|{}|{}",
                "sfid",
                challenge.qr_aud,
                challenge.qr_origin,
                challenge.challenge_id,
                challenge.challenge_token,
                challenge.nonce,
                challenge.expire_at.timestamp()
            )
        } else {
            challenge.challenge_text.clone()
        };
        (verify_message, challenge.session_id.clone())
    };

    let login_pubkey_raw = input.admin_pubkey.trim().to_string();
    let signer_pubkey = input
        .signer_pubkey
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let verify_pubkey = signer_pubkey
        .clone()
        .unwrap_or_else(|| login_pubkey_raw.clone());
    let login_pubkey = if store.admin_users_by_pubkey.contains_key(&login_pubkey_raw) {
        login_pubkey_raw.clone()
    } else if let Some(spk) = signer_pubkey.clone() {
        if store.admin_users_by_pubkey.contains_key(&spk) {
            spk
        } else {
            login_pubkey_raw.clone()
        }
    } else {
        login_pubkey_raw.clone()
    };
    if !verify_admin_signature(&verify_pubkey, &challenge_text, input.signature.trim()) {
        warn!(
            request_id = %input.challenge_id,
            admin_pubkey = %login_pubkey_raw,
            signer_pubkey = %verify_pubkey,
            "qr login signature verify failed"
        );
        return api_error(StatusCode::UNAUTHORIZED, 2004, "signature verify failed");
    }
    let Some(admin) = store.admin_users_by_pubkey.get(&login_pubkey) else {
        return api_error(StatusCode::FORBIDDEN, 2002, "admin not found");
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }
    let login_role = admin.role.clone();
    let login_status = admin.status.clone();

    if let Some(challenge) = store.login_challenges.get_mut(&input.challenge_id) {
        challenge.consumed = true;
        challenge.admin_pubkey = login_pubkey.clone();
    }

    let access_token = Uuid::new_v4().to_string();
    let expire_at = now + Duration::hours(8);
    store.admin_sessions.insert(
        access_token.clone(),
        AdminSession {
            token: access_token.clone(),
            admin_pubkey: login_pubkey.clone(),
            role: login_role.clone(),
            expire_at,
            last_active_at: now,
        },
    );
    store.qr_login_results.insert(
        input.challenge_id.clone(),
        QrLoginResultRecord {
            session_id,
            access_token: access_token.clone(),
            expire_at,
            admin_pubkey: login_pubkey,
            role: login_role,
            status: login_status,
            created_at: now,
        },
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "qr login complete",
    })
    .into_response()
}

async fn admin_auth_qr_result(
    State(state): State<AppState>,
    Query(query): Query<AdminQrResultQuery>,
) -> impl IntoResponse {
    if query.challenge_id.trim().is_empty() || query.session_id.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id and session_id are required",
        );
    }

    let now = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);

    if let Some(result) = store.qr_login_results.get(query.challenge_id.trim()) {
        if result.session_id != query.session_id.trim() {
            return api_error(StatusCode::FORBIDDEN, 1003, "challenge session mismatch");
        }
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: AdminQrResultOutput {
                status: "SUCCESS".to_string(),
                message: "login success".to_string(),
                access_token: Some(result.access_token.clone()),
                expire_at: Some(result.expire_at.timestamp()),
                admin: Some(AdminIdentifyOutput {
                    admin_pubkey: result.admin_pubkey.clone(),
                    role: result.role.clone(),
                    status: result.status.clone(),
                    admin_name: {
                        if let Some(admin_user) =
                            store.admin_users_by_pubkey.get(&result.admin_pubkey)
                        {
                            let province =
                                province_scope_for_role(&store, &result.admin_pubkey, &result.role);
                            build_admin_display_name_from_user(admin_user, province.as_deref())
                        } else {
                            let province =
                                province_scope_for_role(&store, &result.admin_pubkey, &result.role);
                            build_admin_display_name(
                                &result.admin_pubkey,
                                &result.role,
                                province.as_deref(),
                            )
                        }
                    },
                    admin_province: province_scope_for_role(
                        &store,
                        &result.admin_pubkey,
                        &result.role,
                    ),
                }),
            },
        })
        .into_response();
    }

    let Some(challenge) = store.login_challenges.get(query.challenge_id.trim()) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
    };
    if challenge.session_id != query.session_id.trim() {
        return api_error(StatusCode::FORBIDDEN, 1003, "challenge session mismatch");
    }
    if now > challenge.expire_at {
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: AdminQrResultOutput {
                status: "EXPIRED".to_string(),
                message: "challenge expired".to_string(),
                access_token: None,
                expire_at: None,
                admin: None,
            },
        })
        .into_response();
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQrResultOutput {
            status: "PENDING".to_string(),
            message: "waiting mobile scan".to_string(),
            access_token: None,
            expire_at: None,
            admin: None,
        },
    })
    .into_response()
}

fn admin_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    if let Some(token) = bearer_token(headers) {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return Err(resp),
        };
        let now = Utc::now();
        let idle_timeout_minutes = std::env::var("SFID_ADMIN_IDLE_TIMEOUT_MINUTES")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(10);
        let (session_pubkey, session_role) = {
            let Some(session) = store.admin_sessions.get_mut(&token) else {
                return Err(api_error(
                    StatusCode::UNAUTHORIZED,
                    1002,
                    "invalid access token",
                ));
            };
            if now > session.expire_at
                || now > session.last_active_at + Duration::minutes(idle_timeout_minutes)
            {
                store.admin_sessions.remove(&token);
                return Err(api_error(
                    StatusCode::UNAUTHORIZED,
                    1002,
                    "access token expired",
                ));
            }
            session.last_active_at = now;
            (session.admin_pubkey.clone(), session.role.clone())
        };
        if session_role == AdminRole::QueryOnly {
            return Ok(AdminAuthContext {
                admin_pubkey: session_pubkey.clone(),
                role: session_role.clone(),
                admin_name: build_admin_display_name(&session_pubkey, &session_role, None),
                admin_province: None,
            });
        }
        let Some(admin_user) = store.admin_users_by_pubkey.get(&session_pubkey) else {
            return Err(api_error(StatusCode::FORBIDDEN, 2002, "admin not found"));
        };
        if admin_user.status != AdminStatus::Active {
            return Err(api_error(StatusCode::FORBIDDEN, 2003, "admin disabled"));
        }
        let admin_province =
            province_scope_for_role(&store, &admin_user.admin_pubkey, &admin_user.role);
        return Ok(AdminAuthContext {
            admin_pubkey: admin_user.admin_pubkey.clone(),
            role: admin_user.role.clone(),
            admin_name: build_admin_display_name_from_user(admin_user, admin_province.as_deref()),
            admin_province,
        });
    }
    let _ = state;
    Err(api_error(
        StatusCode::UNAUTHORIZED,
        1002,
        "admin auth required",
    ))
}

fn require_admin_any(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    admin_auth(state, headers)
}

fn require_admin_write(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if ctx.role == AdminRole::QueryOnly {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin role required",
        ));
    }
    Ok(ctx)
}

fn require_super_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if ctx.role != AdminRole::SuperAdmin {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "super admin required",
        ));
    }
    Ok(ctx)
}

fn require_super_or_key_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if !matches!(ctx.role, AdminRole::SuperAdmin | AdminRole::KeyAdmin) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "super admin or key admin required",
        ));
    }
    Ok(ctx)
}

fn require_key_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if ctx.role != AdminRole::KeyAdmin {
        return Err(api_error(StatusCode::FORBIDDEN, 1003, "key admin required"));
    }
    Ok(ctx)
}

fn require_super_or_operator_or_key_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if !matches!(
        ctx.role,
        AdminRole::SuperAdmin | AdminRole::OperatorAdmin | AdminRole::KeyAdmin
    ) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "super admin or operator admin or key admin required",
        ));
    }
    Ok(ctx)
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?.trim();
    let token = auth.strip_prefix("Bearer ")?;
    if token.trim().is_empty() {
        return None;
    }
    Some(token.trim().to_string())
}

pub(crate) fn require_public_search_auth(
    headers: &HeaderMap,
) -> Result<(), axum::response::Response> {
    let incoming = headers
        .get("x-public-search-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string();
    if incoming.is_empty() {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1002,
            "public search auth required",
        ));
    }
    let expected = required_env("SFID_PUBLIC_SEARCH_TOKEN");
    if incoming != expected {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1008,
            "public search auth invalid",
        ));
    }
    Ok(())
}

fn require_chain_auth(headers: &HeaderMap) -> Result<(), axum::response::Response> {
    let incoming = headers
        .get("x-chain-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string();
    if incoming.is_empty() {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1002,
            "chain auth required",
        ));
    }
    let expected = required_env("SFID_CHAIN_TOKEN");
    if incoming != expected {
        return Err(api_error(StatusCode::FORBIDDEN, 1008, "chain auth invalid"));
    }
    Ok(())
}

fn env_flag_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            let value = v.trim();
            value.eq_ignore_ascii_case("1")
                || value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("yes")
                || value.eq_ignore_ascii_case("on")
        })
        .unwrap_or(false)
}

fn parse_csv_env_set(key: &str) -> Vec<String> {
    std::env::var(key)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| v.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

fn callback_allowed_hosts() -> Vec<String> {
    parse_csv_env_set("SFID_CALLBACK_ALLOWED_HOSTS")
}

fn host_matches_rule(host: &str, rule: &str) -> bool {
    if let Some(suffix) = rule.strip_prefix("*.") {
        return host.ends_with(&format!(".{suffix}"));
    }
    if let Some(suffix) = rule.strip_prefix('.') {
        return host.ends_with(&format!(".{suffix}"));
    }
    host == rule
}

fn is_blocked_callback_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6.is_multicast()
        }
    }
}

fn validate_bind_callback_url(url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|_| "callback_url is not a valid URL".to_string())?;
    let insecure_http_allowed = env_flag_enabled("SFID_ALLOW_INSECURE_CALLBACK_HTTP");
    match parsed.scheme() {
        "https" => {}
        "http" if insecure_http_allowed => {}
        "http" => {
            return Err(
                "callback_url must use https (set SFID_ALLOW_INSECURE_CALLBACK_HTTP=true only for local dev)"
                    .to_string(),
            )
        }
        _ => return Err("callback_url scheme must be http or https".to_string()),
    }

    let Some(host) = parsed.host_str() else {
        return Err("callback_url host is required".to_string());
    };
    let host_lower = host.to_ascii_lowercase();
    if host_lower == "localhost" || host_lower.ends_with(".localhost") {
        return Err("callback_url localhost is not allowed".to_string());
    }
    if let Ok(ip) = host_lower.parse::<IpAddr>() {
        if is_blocked_callback_ip(ip) {
            return Err("callback_url private/local IP literals are not allowed".to_string());
        }
    }

    let allowlist = callback_allowed_hosts();
    if !allowlist.is_empty()
        && !allowlist
            .iter()
            .any(|rule| host_matches_rule(host_lower.as_str(), rule.as_str()))
    {
        return Err("callback_url host is not in SFID_CALLBACK_ALLOWED_HOSTS".to_string());
    }

    Ok(())
}

fn chain_header_value(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn chain_request_signing_secret() -> Result<String, axum::response::Response> {
    let secret = std::env::var("SFID_CHAIN_SIGNING_SECRET")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "SFID_CHAIN_SIGNING_SECRET must be configured",
            )
        })?;
    if secret.len() < 32 {
        return Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "SFID_CHAIN_SIGNING_SECRET must be at least 32 chars",
        ));
    }
    Ok(secret)
}

fn chain_signature_payload(
    route_key: &str,
    request_id: &str,
    nonce: &str,
    timestamp: i64,
    fingerprint: &str,
) -> String {
    format!(
        "route={route_key}\nrequest_id={request_id}\nnonce={nonce}\ntimestamp={timestamp}\nfingerprint={fingerprint}"
    )
}

fn chain_signature_hex(secret: &str, payload: &str) -> String {
    let key_digest = blake3::hash(secret.as_bytes());
    let hash = blake3::keyed_hash(key_digest.as_bytes(), payload.as_bytes());
    hex::encode(hash.as_bytes())
}

fn constant_time_eq_hex(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (left, right) in a.bytes().zip(b.bytes()) {
        diff |= left ^ right;
    }
    diff == 0
}

fn require_chain_signature(
    headers: &HeaderMap,
    route_key: &str,
    request_id: &str,
    nonce: &str,
    timestamp: i64,
    fingerprint: &str,
) -> Result<(), axum::response::Response> {
    let secret = chain_request_signing_secret()?;
    let Some(incoming_sig) = chain_header_value(headers, "x-chain-signature") else {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1020,
            "x-chain-signature is required",
        ));
    };
    let payload = chain_signature_payload(route_key, request_id, nonce, timestamp, fingerprint);
    let expected = chain_signature_hex(secret.as_str(), payload.as_str());
    let incoming_norm = incoming_sig.to_ascii_lowercase();
    if !constant_time_eq_hex(incoming_norm.as_str(), expected.as_str()) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1021,
            "chain signature invalid",
        ));
    }
    Ok(())
}

fn request_fingerprint<T: Serialize>(input: &T) -> String {
    let payload = serde_json::to_vec(input).unwrap_or_default();
    hex::encode(blake3::hash(&payload).as_bytes())
}

fn cleanup_chain_auth_tracking(store: &mut Store, now: DateTime<Utc>) {
    store
        .chain_requests_by_key
        .retain(|_, record| record.received_at > now - Duration::hours(24));
    store
        .chain_nonce_seen
        .retain(|_, seen_at| *seen_at > now - Duration::hours(24));
}

fn require_chain_request(
    store: &mut Store,
    headers: &HeaderMap,
    route_key: &str,
    fingerprint: &str,
) -> Result<ChainRequestAuth, axum::response::Response> {
    store.metrics.chain_request_total += 1;
    if let Err(resp) = require_chain_auth(headers) {
        store.metrics.chain_auth_failures += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(resp);
    }
    let Some(request_id) = chain_header_value(headers, "x-chain-request-id") else {
        store.metrics.chain_auth_failures += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1011,
            "x-chain-request-id is required",
        ));
    };
    let Some(nonce) = chain_header_value(headers, "x-chain-nonce") else {
        store.metrics.chain_auth_failures += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1012,
            "x-chain-nonce is required",
        ));
    };
    let Some(ts_text) = chain_header_value(headers, "x-chain-timestamp") else {
        store.metrics.chain_auth_failures += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1013,
            "x-chain-timestamp is required",
        ));
    };
    let Ok(ts) = ts_text.parse::<i64>() else {
        store.metrics.chain_auth_failures += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1014,
            "x-chain-timestamp must be unix timestamp seconds",
        ));
    };
    let now = Utc::now();
    if (now.timestamp() - ts).abs() > 300 {
        store.metrics.chain_auth_failures += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1015,
            "chain request timestamp expired",
        ));
    }
    if let Err(resp) =
        require_chain_signature(headers, route_key, &request_id, &nonce, ts, fingerprint)
    {
        store.metrics.chain_auth_failures += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(resp);
    }

    cleanup_chain_auth_tracking(store, now);
    let nonce_key = format!("{route_key}:{nonce}");
    if store.chain_nonce_seen.contains_key(&nonce_key) {
        store.metrics.chain_replay_rejects += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(api_error(
            StatusCode::CONFLICT,
            1016,
            "duplicate chain nonce",
        ));
    }
    let request_key = format!("{route_key}:{request_id}");
    if let Some(existing) = store.chain_requests_by_key.get(&request_key) {
        store.metrics.chain_replay_rejects += 1;
        store.metrics.chain_request_failed_total += 1;
        if existing.fingerprint == fingerprint {
            return Err(api_error(
                StatusCode::CONFLICT,
                1017,
                "duplicate chain request",
            ));
        }
        return Err(api_error(
            StatusCode::CONFLICT,
            1018,
            "chain request id conflict",
        ));
    }

    insert_bounded_map(
        &mut store.chain_nonce_seen,
        nonce_key,
        now,
        bounded_cache_limit("SFID_CHAIN_NONCE_CACHE_MAX", 50_000),
    );
    insert_bounded_map(
        &mut store.chain_requests_by_key,
        request_key,
        ChainRequestReceipt {
            route_key: route_key.to_string(),
            request_id: request_id.clone(),
            nonce: nonce.clone(),
            fingerprint: fingerprint.to_string(),
            received_at: now,
        },
        bounded_cache_limit("SFID_CHAIN_REQUEST_CACHE_MAX", 50_000),
    );
    Ok(ChainRequestAuth {
        request_id,
        nonce,
        timestamp: ts,
    })
}

fn record_chain_latency(store: &mut Store, started_at: DateTime<Utc>) {
    let elapsed_ms = (Utc::now() - started_at).num_milliseconds().max(0) as u32;
    let samples = &mut store.metrics.chain_latency_samples;
    samples.push(elapsed_ms);
    if samples.len() > 1024 {
        let drop_count = samples.len() - 1024;
        samples.drain(0..drop_count);
    }
}

fn latency_p95_p99_ms(samples: &[u32]) -> (u32, u32) {
    if samples.is_empty() {
        return (0, 0);
    }
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let len = ordered.len();
    let p95 = ordered[((len as f64 * 0.95).ceil() as usize).saturating_sub(1)];
    let p99 = ordered[((len as f64 * 0.99).ceil() as usize).saturating_sub(1)];
    (p95, p99)
}

fn actor_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    let forwarded = chain_header_value(headers, "x-forwarded-for");
    if let Some(ff) = forwarded {
        return ff.split(',').next().map(|v| v.trim().to_string());
    }
    chain_header_value(headers, "x-real-ip")
}

fn request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    chain_header_value(headers, "x-chain-request-id")
        .or_else(|| chain_header_value(headers, "x-request-id"))
}

fn ensure_chain_request_db(
    state: &AppState,
    route_key: &str,
    auth: &ChainRequestAuth,
    fingerprint: &str,
) -> Result<(), axum::response::Response> {
    let StoreBackend::Postgres { client } = &state.store.backend else {
        return Ok(());
    };
    let insert = StoreBackend::with_postgres_client(client, |conn| {
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
    let StoreBackend::Postgres { client } = &state.store.backend else {
        return Ok(());
    };
    let check = StoreBackend::with_postgres_client(client, |conn| {
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
    let StoreBackend::Postgres { client } = &state.store.backend else {
        return;
    };
    let _ = StoreBackend::with_postgres_client(client, |conn| {
        conn.execute(
            "DELETE FROM binding_unique_locks WHERE account_pubkey=$1",
            &[&account_pubkey],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
}

fn persist_reward_state_db(state: &AppState, reward: &RewardStateRecord) {
    let StoreBackend::Postgres { client } = &state.store.backend else {
        return;
    };
    let _ = StoreBackend::with_postgres_client(client, |conn| {
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
    let StoreBackend::Postgres { client } = &state.store.backend else {
        return;
    };
    let _ = StoreBackend::with_postgres_client(client, |conn| {
        conn.execute(
            "DELETE FROM bind_reward_states WHERE account_pubkey=$1",
            &[&account_pubkey],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
}

pub(crate) fn verify_admin_signature(
    admin_pubkey: &str,
    message: &str,
    signature_text: &str,
) -> bool {
    let Some(pubkey_bytes) = parse_sr25519_pubkey_bytes(admin_pubkey) else {
        return false;
    };
    let normalized_sig = normalize_hex(signature_text);
    let sig_bytes = match Vec::from_hex(&normalized_sig) {
        Ok(v) if v.len() == 64 => v,
        _ => return false,
    };
    let sig_arr: [u8; 64] = match sig_bytes.as_slice().try_into() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let pubkey = match Sr25519PublicKey::from_bytes(&pubkey_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let signature = match Sr25519Signature::from_bytes(&sig_arr) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let ctx = signing_context(b"substrate");
    if pubkey
        .verify(ctx.bytes(message.as_bytes()), &signature)
        .is_ok()
    {
        return true;
    }
    // Some wallets sign wrapped bytes payload: "<Bytes>{message}</Bytes>".
    let wrapped = format!("<Bytes>{}</Bytes>", message);
    pubkey
        .verify(ctx.bytes(wrapped.as_bytes()), &signature)
        .is_ok()
}

fn parse_sr25519_pubkey(admin_pubkey: &str) -> Option<String> {
    let normalized = normalize_hex(admin_pubkey);
    if normalized.len() == 64 && normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(normalized);
    }
    None
}

fn parse_sr25519_pubkey_bytes(admin_pubkey: &str) -> Option<[u8; 32]> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(admin_pubkey) {
        let bytes = Vec::from_hex(&hex_pubkey).ok()?;
        let arr: [u8; 32] = bytes.as_slice().try_into().ok()?;
        return Some(arr);
    }
    None
}

fn normalize_hex(value: &str) -> String {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
        .to_string()
}

fn parse_admin_identity_qr(identity_qr: &str) -> String {
    let trimmed = identity_qr.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(v) = value
                .get("admin_pubkey")
                .or_else(|| value.get("pubkey"))
                .and_then(|v| v.as_str())
            {
                return v.trim().to_string();
            }
        }
    }
    trimmed.to_string()
}

fn extract_domain_from_origin(origin: &str) -> Option<String> {
    let trimmed = origin.trim();
    if trimmed.is_empty() {
        return None;
    }
    let no_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let host_port = no_scheme.split('/').next().unwrap_or("");
    if host_port.is_empty() {
        return None;
    }
    let domain = host_port.split(':').next().unwrap_or("");
    if domain.is_empty() {
        return None;
    }
    Some(domain.to_string())
}

fn cleanup_expired_challenges(store: &mut Store, now: DateTime<Utc>) {
    store.login_challenges.retain(|_, c| {
        c.expire_at > now - Duration::minutes(10) && (!c.consumed || c.expire_at > now)
    });
    store.qr_login_results.retain(|_, r| {
        r.created_at > now - Duration::hours(1) && r.expire_at > now - Duration::minutes(10)
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
    let StoreBackend::Postgres { client } = &state.store.backend else {
        return false;
    };
    let row = match StoreBackend::with_postgres_client(client, |conn| {
        conn.query_opt("SELECT payload FROM runtime_meta WHERE id=1", &[])
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
    let snapshot: PersistedRuntimeMeta = match serde_json::from_value(payload) {
        Ok(v) => v,
        Err(err) => {
            warn!(error = %err, "failed to decode runtime_meta");
            return false;
        }
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

    let StoreBackend::Postgres { client } = &state.store.backend else {
        return;
    };
    let payload = match serde_json::to_value(snapshot) {
        Ok(v) => v,
        Err(err) => {
            warn!(error = %err, "failed to encode runtime_meta");
            return;
        }
    };
    if let Err(err) = StoreBackend::with_postgres_client(client, move |conn| {
        conn.execute(
            "INSERT INTO runtime_meta(id, payload, updated_at) VALUES (1, $1, now())
             ON CONFLICT (id) DO UPDATE SET payload=excluded.payload, updated_at=now()",
            &[&payload],
        )
        .map(|_| ())
        .map_err(|e| format!("failed to persist runtime_meta: {e}"))
    }) {
        warn!(error = %err, "failed to persist runtime_meta");
    }
}

fn seed_super_admins(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    if !store.admin_users_by_pubkey.is_empty() {
        return;
    }
    let now = Utc::now();
    for (idx, item) in provinces().iter().enumerate() {
        let pubkey = item.pubkey.to_string();
        store.admin_users_by_pubkey.insert(
            pubkey.clone(),
            AdminUser {
                id: (idx as u64) + 1,
                admin_pubkey: pubkey,
                admin_name: String::new(),
                role: AdminRole::SuperAdmin,
                status: AdminStatus::Active,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: now,
            },
        );
        store
            .super_admin_province_by_pubkey
            .insert(item.pubkey.to_string(), item.name.to_string());
    }
}

fn build_admin_display_name(
    admin_pubkey: &str,
    role: &AdminRole,
    admin_province: Option<&str>,
) -> String {
    if *role == AdminRole::SuperAdmin {
        if let Some(province) = admin_province {
            return format!("{province}超级管理员");
        }
    }
    if let Some(name) = super_admin_display_name(admin_pubkey) {
        return name;
    }
    match role {
        AdminRole::KeyAdmin => "密钥管理员".to_string(),
        AdminRole::OperatorAdmin => "操作管理员".to_string(),
        AdminRole::QueryOnly => "查询管理员".to_string(),
        AdminRole::SuperAdmin => "超级管理员".to_string(),
    }
}

fn build_admin_display_name_from_user(admin: &AdminUser, admin_province: Option<&str>) -> String {
    if admin.role == AdminRole::OperatorAdmin {
        let name = admin.admin_name.trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }
    build_admin_display_name(&admin.admin_pubkey, &admin.role, admin_province)
}

fn cleanup_consumed_qr_ids(store: &mut Store, now: DateTime<Utc>) {
    store
        .consumed_qr_ids
        .retain(|_, consumed_at| *consumed_at > now - Duration::hours(24));
}

fn cleanup_pending_bind_scans(store: &mut Store, now: DateTime<Utc>) {
    let now_ts = now.timestamp();
    store.pending_bind_scan_by_qr_id.retain(|_, pending| {
        pending.scanned_at > now - Duration::hours(24) && pending.expire_at >= now_ts
    });
}

fn vote_cache_key(account_pubkey: &str, proposal_id: Option<u64>) -> String {
    match proposal_id {
        Some(id) => format!("{account_pubkey}:{id}"),
        None => format!("{account_pubkey}:none"),
    }
}

fn cleanup_vote_cache(store: &mut Store, now: DateTime<Utc>) {
    store
        .vote_verify_cache
        .retain(|_, entry| entry.cached_at > now - Duration::seconds(5));
}

fn invalidate_vote_cache_for_pubkey(store: &mut Store, account_pubkey: &str) {
    store
        .vote_verify_cache
        .retain(|_, entry| entry.account_pubkey != account_pubkey);
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn bounded_cache_limit(key: &str, default_value: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default_value)
}

pub(crate) fn insert_bounded_map<K, V>(map: &mut HashMap<K, V>, key: K, value: V, limit: usize)
where
    K: Eq + Hash + Clone,
{
    if !map.contains_key(&key) && map.len() >= limit {
        if let Some(first_key) = map.keys().next().cloned() {
            map.remove(&first_key);
        }
    }
    map.insert(key, value);
}

fn default_bind_callback_url() -> Option<String> {
    normalize_optional(std::env::var("SFID_BIND_CALLBACK_URL").ok())
}

fn default_bind_callback_auth_token() -> Option<String> {
    normalize_optional(std::env::var("SFID_BIND_CALLBACK_AUTH_TOKEN").ok())
}

fn enqueue_bind_callback_job(
    store: &mut Store,
    callback_url: Option<String>,
    payload: BindCallbackPayload,
) {
    let Some(url) = callback_url else {
        return;
    };
    store.bind_callback_jobs.push(BindCallbackJob {
        callback_id: payload.callback_id.clone(),
        callback_url: url,
        payload,
        attempts: 0,
        max_attempts: 5,
        next_attempt_at: Utc::now(),
        last_error: None,
    });
}

async fn bind_callback_worker(state: AppState) {
    let client = reqwest::Client::new();
    loop {
        let due_jobs = {
            let mut store = match state.store.write() {
                Ok(guard) => guard,
                Err(err) => {
                    warn!(error = %err, "bind callback worker failed to lock store");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            };
            let now = Utc::now();
            let mut due = Vec::new();
            let mut pending = Vec::new();
            for job in store.bind_callback_jobs.drain(..) {
                if job.next_attempt_at <= now {
                    due.push(job);
                } else {
                    pending.push(job);
                }
            }
            store.bind_callback_jobs = pending;
            due
        };

        for mut job in due_jobs {
            let mut request = client
                .post(job.callback_url.clone())
                .header("content-type", "application/json")
                .header("x-sfid-callback-id", job.callback_id.clone())
                .header(
                    "x-sfid-callback-signature",
                    job.payload.callback_attestation.signature_hex.clone(),
                )
                .header(
                    "x-sfid-callback-key-id",
                    job.payload.callback_attestation.key_id.clone(),
                )
                .json(&job.payload);
            if let Some(token) = default_bind_callback_auth_token().as_ref() {
                request = request.bearer_auth(token);
            }
            let delivery = request.send().await;
            let mut store = match state.store.write() {
                Ok(guard) => guard,
                Err(err) => {
                    warn!(error = %err, "bind callback worker failed to lock store after send");
                    continue;
                }
            };
            match delivery {
                Ok(resp) if resp.status().is_success() => {
                    store.metrics.bind_callback_success_total += 1;
                    append_audit_log(
                        &mut store,
                        "BIND_CALLBACK",
                        "system",
                        Some(job.payload.account_pubkey.clone()),
                        Some(job.payload.archive_index.clone()),
                        "SUCCESS",
                        format!(
                            "callback delivered callback_id={} url={}",
                            job.callback_id, job.callback_url
                        ),
                    );
                }
                Ok(resp) => {
                    job.attempts += 1;
                    let status = resp.status().as_u16();
                    if job.attempts >= job.max_attempts {
                        store.metrics.bind_callback_failed_total += 1;
                        append_audit_log(
                            &mut store,
                            "BIND_CALLBACK",
                            "system",
                            Some(job.payload.account_pubkey.clone()),
                            Some(job.payload.archive_index.clone()),
                            "FAILED",
                            format!(
                                "callback exhausted callback_id={} status={}",
                                job.callback_id, status
                            ),
                        );
                    } else {
                        store.metrics.bind_callback_retry_total += 1;
                        let backoff_secs = (2_i64.pow(job.attempts.min(6))).min(300);
                        job.next_attempt_at = Utc::now() + Duration::seconds(backoff_secs);
                        job.last_error = Some(format!("http status {status}"));
                        store.bind_callback_jobs.push(job);
                    }
                }
                Err(err) => {
                    job.attempts += 1;
                    if job.attempts >= job.max_attempts {
                        store.metrics.bind_callback_failed_total += 1;
                        append_audit_log(
                            &mut store,
                            "BIND_CALLBACK",
                            "system",
                            Some(job.payload.account_pubkey.clone()),
                            Some(job.payload.archive_index.clone()),
                            "FAILED",
                            format!(
                                "callback exhausted callback_id={} error={}",
                                job.callback_id, err
                            ),
                        );
                    } else {
                        store.metrics.bind_callback_retry_total += 1;
                        let backoff_secs = (2_i64.pow(job.attempts.min(6))).min(300);
                        job.next_attempt_at = Utc::now() + Duration::seconds(backoff_secs);
                        job.last_error = Some(err.to_string());
                        store.bind_callback_jobs.push(job);
                    }
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

fn append_audit_log(
    store: &mut Store,
    action: &str,
    actor_pubkey: &str,
    target_pubkey: Option<String>,
    target_archive_no: Option<String>,
    result: &'static str,
    detail: String,
) {
    append_audit_log_with_meta(
        store,
        action,
        actor_pubkey,
        target_pubkey,
        target_archive_no,
        None,
        None,
        result,
        detail,
    );
}

fn append_audit_log_with_meta(
    store: &mut Store,
    action: &str,
    actor_pubkey: &str,
    target_pubkey: Option<String>,
    target_archive_no: Option<String>,
    request_id: Option<String>,
    actor_ip: Option<String>,
    result: &'static str,
    detail: String,
) {
    let max_logs = bounded_cache_limit("SFID_AUDIT_LOG_MAX", 20_000);
    if store.audit_logs.len() >= max_logs {
        let overflow = store.audit_logs.len() - max_logs + 1;
        store.audit_logs.drain(0..overflow);
    }
    store.next_audit_seq += 1;
    store.audit_logs.push(AuditLogEntry {
        seq: store.next_audit_seq,
        action: action.to_string(),
        actor_pubkey: actor_pubkey.to_string(),
        target_pubkey,
        target_archive_no,
        request_id,
        actor_ip,
        result: result.to_string(),
        detail,
        created_at: Utc::now(),
    });
}

fn seed_demo_record(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    if !store.bindings_by_pubkey.is_empty() || !store.pending_by_pubkey.is_empty() {
        return;
    }
    let total = 50_u64;
    let bound_total = 30_u64;
    let now = Utc::now();

    for seq in 1..=total {
        let pubkey = format!("0xDEMO_PUBKEY_{seq:04}");
        if seq <= bound_total {
            let archive = format!("CIV-DEMO-{seq:04}");
            let sfid = deterministic_sfid_code(state, &archive, &pubkey);
            let binding_payload = BindingPayload {
                kind: "bind",
                version: "v1",
                account_pubkey: pubkey.clone(),
                archive_index: archive.clone(),
                sfid_code: sfid.clone(),
                issued_at: now.timestamp(),
            };
            let proof = make_signature_envelope(state, &binding_payload);
            store.bindings_by_pubkey.insert(
                pubkey.clone(),
                BindingRecord {
                    seq,
                    account_pubkey: pubkey.clone(),
                    archive_index: archive.clone(),
                    birth_date: parse_birth_date_from_archive_no(&archive),
                    citizen_status: CitizenStatus::Normal,
                    sfid_code: sfid,
                    sfid_signature: proof.signature_hex,
                    bound_at: now,
                    bound_by: "system-seed".to_string(),
                    admin_province: None,
                    client_request_id: None,
                },
            );
            store.pubkey_by_archive_index.insert(archive, pubkey);
        } else {
            store.pending_by_pubkey.insert(
                pubkey.clone(),
                PendingRequest {
                    seq,
                    account_pubkey: pubkey,
                    admin_province: None,
                    requested_at: now,
                    callback_url: None,
                    client_request_id: None,
                },
            );
        }
    }
    store.next_seq = total;
}

fn deterministic_sfid_code(state: &AppState, archive_index: &str, account_pubkey: &str) -> String {
    let public_key_hex = state
        .public_key_hex
        .read()
        .map(|v| v.clone())
        .unwrap_or_default();
    let mut payload = Vec::new();
    payload.extend_from_slice(b"sfid-code-v1|");
    payload.extend_from_slice(public_key_hex.as_bytes());
    payload.extend_from_slice(b"|");
    payload.extend_from_slice(archive_index.as_bytes());
    payload.extend_from_slice(b"|");
    payload.extend_from_slice(account_pubkey.as_bytes());
    let digest = blake3::hash(&payload);
    let digest_bytes = digest.as_bytes();

    let core = hex::encode_upper(&digest_bytes[..12]);
    let checksum = digest_bytes
        .iter()
        .fold(0_u32, |acc, b| (acc + u32::from(*b)) % 10_u32);
    format!("SFID-{core}{checksum}")
}

fn make_signature_envelope<T: Serialize>(state: &AppState, payload: &T) -> SignatureEnvelope {
    let seed = state
        .signing_seed_hex
        .read()
        .map(|v| v.clone())
        .unwrap_or_default();
    let signing_key = key_admins::chain_keyring::load_signing_key_from_seed(seed.as_str());
    key_admins::chain_proof::make_signature_envelope(
        &state.key_id,
        &state.key_version,
        &state.key_alg,
        &signing_key,
        payload,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::to_bytes,
        http::{HeaderValue, StatusCode},
        response::Response,
    };

    fn build_test_state() -> AppState {
        std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
        std::env::set_var(
            "SFID_CHAIN_SIGNING_SECRET",
            "test-chain-signing-secret-at-least-32",
        );
        std::env::set_var("SFID_PUBLIC_SEARCH_TOKEN", "test-public-search-token");
        let main_seed = "sfid-dev-master-seed-v1".to_string();
        let main_key = key_admins::chain_keyring::load_signing_key_from_seed(main_seed.as_str());
        let public_key_hex = format!("0x{}", hex::encode(main_key.public.to_bytes()));
        let mut known_key_seeds = HashMap::new();
        known_key_seeds.insert(public_key_hex.clone(), main_seed.clone());
        let state = AppState {
            store: StoreHandle::in_memory(),
            signing_seed_hex: Arc::new(RwLock::new(main_seed)),
            known_key_seeds: Arc::new(RwLock::new(known_key_seeds)),
            request_limits: Arc::new(Mutex::new(HashMap::new())),
            key_id: "sfid-master-v1".to_string(),
            key_version: "v1".to_string(),
            key_alg: "sr25519".to_string(),
            public_key_hex: Arc::new(RwLock::new(public_key_hex)),
        };
        seed_super_admins(&state);
        key_admins::seed_chain_keyring(&state);
        key_admins::seed_key_admins(&state);
        seed_demo_record(&state);
        state
    }

    async fn parse_json(resp: Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("response body bytes");
        serde_json::from_slice(&bytes).expect("json response")
    }

    fn sign_with_test_sr25519(seed_byte: u8, message: &str) -> (String, String) {
        let seed = [seed_byte; 32];
        let mini = schnorrkel::MiniSecretKey::from_bytes(&seed).expect("mini secret key");
        let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Uniform);
        let ctx = signing_context(b"substrate");
        let sig = keypair.sign(ctx.bytes(message.as_bytes()));
        (
            format!("0x{}", hex::encode(keypair.public.to_bytes())),
            format!("0x{}", hex::encode(sig.to_bytes())),
        )
    }

    fn sign_rotation_challenge(seed_hex: &str, message: &str) -> String {
        let keypair = key_admins::chain_keyring::load_signing_key_from_seed(seed_hex);
        let ctx = signing_context(b"substrate");
        let sig = keypair.sign(ctx.bytes(message.as_bytes()));
        format!("0x{}", hex::encode(sig.to_bytes()))
    }

    fn setup_rotation_test_state() -> (AppState, HeaderMap, String, String) {
        let state = build_test_state();
        let main_seed = "sfid-test-main-seed";
        let backup_a_seed = "sfid-test-backup-a-seed";
        let backup_b_seed = "sfid-test-backup-b-seed";
        let new_backup_seed = "sfid-test-backup-c-seed";
        let main_pubkey = key_admins::chain_keyring::derive_pubkey_hex_from_seed(main_seed);
        let backup_a_pubkey = key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_a_seed);
        let backup_b_pubkey = key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_b_seed);

        {
            let mut seed_guard = state
                .signing_seed_hex
                .write()
                .expect("signing seed write lock poisoned");
            *seed_guard = main_seed.to_string();
        }
        {
            let mut pubkey_guard = state
                .public_key_hex
                .write()
                .expect("public key write lock poisoned");
            *pubkey_guard = main_pubkey.clone();
        }
        {
            let mut known = state
                .known_key_seeds
                .write()
                .expect("known seeds write lock poisoned");
            known.insert(main_pubkey.clone(), main_seed.to_string());
            known.insert(backup_a_pubkey.clone(), backup_a_seed.to_string());
            known.insert(backup_b_pubkey.clone(), backup_b_seed.to_string());
            known.insert(
                key_admins::chain_keyring::derive_pubkey_hex_from_seed(new_backup_seed),
                new_backup_seed.to_string(),
            );
        }
        {
            let mut store = state.store.write().expect("store write lock poisoned");
            store.chain_keyring_state = Some(ChainKeyringState::new(
                main_pubkey,
                backup_a_pubkey.clone(),
                backup_b_pubkey,
            ));
            key_admins::sync_key_admin_users(&mut store);
            store.admin_sessions.insert(
                "tok-rotate".to_string(),
                AdminSession {
                    token: "tok-rotate".to_string(),
                    admin_pubkey: backup_a_pubkey.clone(),
                    role: AdminRole::KeyAdmin,
                    expire_at: Utc::now() + Duration::hours(1),
                    last_active_at: Utc::now(),
                },
            );
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str("Bearer tok-rotate").expect("header value"),
        );
        (
            state,
            headers,
            backup_a_seed.to_string(),
            new_backup_seed.to_string(),
        )
    }

    #[tokio::test]
    async fn keyring_rotate_commit_requires_prior_verify() {
        let (state, headers, backup_a_seed, new_backup_seed) = setup_rotation_test_state();
        let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
            State(state.clone()),
            headers.clone(),
            Json(KeyringRotateChallengeInput {
                initiator_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                    backup_a_seed.as_str(),
                ),
            }),
        )
        .await
        .into_response();
        assert_eq!(challenge_resp.status(), StatusCode::OK);
        let challenge_json = parse_json(challenge_resp).await;
        let challenge_id = challenge_json["data"]["challenge_id"]
            .as_str()
            .expect("challenge_id")
            .to_string();
        let challenge_text = challenge_json["data"]["challenge_text"]
            .as_str()
            .expect("challenge_text")
            .to_string();

        let commit_resp = key_admins::admin_chain_keyring_rotate_commit(
            State(state),
            headers,
            Json(KeyringRotateCommitInput {
                challenge_id,
                signature: sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str()),
                new_backup_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                    new_backup_seed.as_str(),
                ),
                new_backup_seed_hex: Some(new_backup_seed),
            }),
        )
        .await
        .into_response();
        assert_eq!(commit_resp.status(), StatusCode::CONFLICT);
        let body = parse_json(commit_resp).await;
        assert_eq!(
            body["message"].as_str(),
            Some("rotation challenge not verified")
        );
    }

    #[tokio::test]
    async fn keyring_rotate_commit_reports_chain_submit_failure_without_blocking_local_rotation() {
        let previous_rpc_url = std::env::var("SFID_CHAIN_RPC_URL").ok();
        std::env::remove_var("SFID_CHAIN_RPC_URL");
        let (state, headers, backup_a_seed, new_backup_seed) = setup_rotation_test_state();
        let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
            State(state.clone()),
            headers.clone(),
            Json(KeyringRotateChallengeInput {
                initiator_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                    backup_a_seed.as_str(),
                ),
            }),
        )
        .await
        .into_response();
        assert_eq!(challenge_resp.status(), StatusCode::OK);
        let challenge_json = parse_json(challenge_resp).await;
        let challenge_id = challenge_json["data"]["challenge_id"]
            .as_str()
            .expect("challenge_id")
            .to_string();
        let challenge_text = challenge_json["data"]["challenge_text"]
            .as_str()
            .expect("challenge_text")
            .to_string();
        let signature = sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str());

        let verify_resp = key_admins::admin_chain_keyring_rotate_verify(
            State(state.clone()),
            headers.clone(),
            Json(KeyringRotateVerifyInput {
                challenge_id: challenge_id.clone(),
                signature: signature.clone(),
            }),
        )
        .await
        .into_response();
        assert_eq!(verify_resp.status(), StatusCode::OK);

        let new_backup_pubkey =
            key_admins::chain_keyring::derive_pubkey_hex_from_seed(new_backup_seed.as_str());
        let backup_a_pubkey =
            key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_a_seed.as_str());
        let commit_resp = key_admins::admin_chain_keyring_rotate_commit(
            State(state),
            headers,
            Json(KeyringRotateCommitInput {
                challenge_id,
                signature,
                new_backup_pubkey,
                new_backup_seed_hex: Some(new_backup_seed),
            }),
        )
        .await
        .into_response();

        if let Some(value) = previous_rpc_url {
            std::env::set_var("SFID_CHAIN_RPC_URL", value);
        }

        assert_eq!(commit_resp.status(), StatusCode::OK);
        let body = parse_json(commit_resp).await;
        assert_eq!(body["data"]["chain_submit_ok"].as_bool(), Some(false));
        assert_eq!(
            body["data"]["main_pubkey"].as_str(),
            Some(backup_a_pubkey.as_str())
        );
    }

    #[tokio::test]
    async fn keyring_rotate_verify_rejects_expired_challenge() {
        let (state, headers, backup_a_seed, _) = setup_rotation_test_state();
        let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
            State(state.clone()),
            headers.clone(),
            Json(KeyringRotateChallengeInput {
                initiator_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                    backup_a_seed.as_str(),
                ),
            }),
        )
        .await
        .into_response();
        assert_eq!(challenge_resp.status(), StatusCode::OK);
        let challenge_json = parse_json(challenge_resp).await;
        let challenge_id = challenge_json["data"]["challenge_id"]
            .as_str()
            .expect("challenge_id")
            .to_string();
        let challenge_text = challenge_json["data"]["challenge_text"]
            .as_str()
            .expect("challenge_text")
            .to_string();
        {
            let mut store = state.store.write().expect("store write lock poisoned");
            let entry = store
                .keyring_rotate_challenges
                .get_mut(&challenge_id)
                .expect("challenge exists");
            entry.expire_at = Utc::now() - Duration::minutes(3);
        }

        let verify_resp = key_admins::admin_chain_keyring_rotate_verify(
            State(state),
            headers,
            Json(KeyringRotateVerifyInput {
                challenge_id,
                signature: sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str()),
            }),
        )
        .await
        .into_response();
        assert_eq!(verify_resp.status(), StatusCode::UNAUTHORIZED);
        let body = parse_json(verify_resp).await;
        assert_eq!(body["message"].as_str(), Some("rotation challenge expired"));
    }

    #[tokio::test]
    async fn qr_login_non_admin_should_be_rejected() {
        let state = build_test_state();

        let challenge_resp = admin_auth_qr_challenge(
            State(state.clone()),
            Json(AdminQrChallengeInput {
                origin: Some("http://127.0.0.1:5179".to_string()),
                domain: None,
                session_id: Some("sid-query-test".to_string()),
            }),
        )
        .await
        .into_response();
        assert_eq!(challenge_resp.status(), StatusCode::OK);
        let challenge_json = parse_json(challenge_resp).await;
        let challenge_id = challenge_json["data"]["challenge_id"]
            .as_str()
            .expect("challenge_id")
            .to_string();
        let session_id = challenge_json["data"]["session_id"]
            .as_str()
            .expect("session_id")
            .to_string();
        let challenge_payload = challenge_json["data"]["challenge_payload"]
            .as_str()
            .expect("challenge_payload")
            .to_string();

        let (query_pubkey, signature) = sign_with_test_sr25519(11, &challenge_payload);
        let complete_resp = admin_auth_qr_complete(
            State(state.clone()),
            Json(AdminQrCompleteInput {
                challenge_id: challenge_id.clone(),
                session_id: Some(session_id.clone()),
                admin_pubkey: query_pubkey,
                signer_pubkey: None,
                signature,
            }),
        )
        .await
        .into_response();
        assert_eq!(complete_resp.status(), StatusCode::FORBIDDEN);
        let body = parse_json(complete_resp).await;
        assert_eq!(body["message"].as_str(), Some("admin not found"));
    }

    #[tokio::test]
    async fn qr_login_super_admin_keeps_write_permission() {
        let state = build_test_state();

        let challenge_resp = admin_auth_qr_challenge(
            State(state.clone()),
            Json(AdminQrChallengeInput {
                origin: Some("http://127.0.0.1:5179".to_string()),
                domain: None,
                session_id: Some("sid-admin-test".to_string()),
            }),
        )
        .await
        .into_response();
        assert_eq!(challenge_resp.status(), StatusCode::OK);
        let challenge_json = parse_json(challenge_resp).await;
        let challenge_id = challenge_json["data"]["challenge_id"]
            .as_str()
            .expect("challenge_id")
            .to_string();
        let session_id = challenge_json["data"]["session_id"]
            .as_str()
            .expect("session_id")
            .to_string();
        let challenge_payload = challenge_json["data"]["challenge_payload"]
            .as_str()
            .expect("challenge_payload")
            .to_string();

        let (admin_pubkey, signature) = sign_with_test_sr25519(22, &challenge_payload);
        {
            let mut store = state.store.write().expect("store write lock poisoned");
            store.admin_users_by_pubkey.insert(
                admin_pubkey.clone(),
                AdminUser {
                    id: 999,
                    admin_pubkey: admin_pubkey.clone(),
                    admin_name: String::new(),
                    role: AdminRole::SuperAdmin,
                    status: AdminStatus::Active,
                    built_in: false,
                    created_by: "TEST".to_string(),
                    created_at: Utc::now(),
                },
            );
        }
        let complete_resp = admin_auth_qr_complete(
            State(state.clone()),
            Json(AdminQrCompleteInput {
                challenge_id: challenge_id.clone(),
                session_id: Some(session_id.clone()),
                admin_pubkey,
                signer_pubkey: None,
                signature,
            }),
        )
        .await
        .into_response();
        assert_eq!(complete_resp.status(), StatusCode::OK);

        let result_resp = admin_auth_qr_result(
            State(state.clone()),
            Query(AdminQrResultQuery {
                challenge_id,
                session_id,
            }),
        )
        .await
        .into_response();
        assert_eq!(result_resp.status(), StatusCode::OK);
        let result_json = parse_json(result_resp).await;
        let token = result_json["data"]["access_token"]
            .as_str()
            .expect("access token");
        assert_eq!(
            result_json["data"]["admin"]["role"].as_str(),
            Some("SUPER_ADMIN")
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {}", token)).expect("header value"),
        );
        assert!(require_admin_write(&state, &headers).is_ok());
    }

    #[test]
    fn require_super_or_operator_or_key_admin_should_allow_expected_roles() {
        let state = build_test_state();
        let (super_pubkey, key_pubkey) = {
            let store = state.store.read().expect("store read lock poisoned");
            let super_pubkey = store
                .admin_users_by_pubkey
                .values()
                .find(|u| u.role == AdminRole::SuperAdmin)
                .map(|u| u.admin_pubkey.clone())
                .expect("super admin exists");
            let key_pubkey = store
                .admin_users_by_pubkey
                .values()
                .find(|u| u.role == AdminRole::KeyAdmin)
                .map(|u| u.admin_pubkey.clone())
                .expect("key admin exists");
            (super_pubkey, key_pubkey)
        };
        let operator_pubkey = "0xTEST_OPERATOR_ADMIN".to_string();
        {
            let mut store = state.store.write().expect("store write lock poisoned");
            store.admin_users_by_pubkey.insert(
                operator_pubkey.clone(),
                AdminUser {
                    id: 9_999,
                    admin_pubkey: operator_pubkey.clone(),
                    admin_name: "测试操作员".to_string(),
                    role: AdminRole::OperatorAdmin,
                    status: AdminStatus::Active,
                    built_in: false,
                    created_by: super_pubkey.clone(),
                    created_at: Utc::now(),
                },
            );
            store.admin_sessions.insert(
                "tok-super".to_string(),
                AdminSession {
                    token: "tok-super".to_string(),
                    admin_pubkey: super_pubkey.clone(),
                    role: AdminRole::SuperAdmin,
                    expire_at: Utc::now() + Duration::hours(1),
                    last_active_at: Utc::now(),
                },
            );
            store.admin_sessions.insert(
                "tok-operator".to_string(),
                AdminSession {
                    token: "tok-operator".to_string(),
                    admin_pubkey: operator_pubkey.clone(),
                    role: AdminRole::OperatorAdmin,
                    expire_at: Utc::now() + Duration::hours(1),
                    last_active_at: Utc::now(),
                },
            );
            store.admin_sessions.insert(
                "tok-key".to_string(),
                AdminSession {
                    token: "tok-key".to_string(),
                    admin_pubkey: key_pubkey.clone(),
                    role: AdminRole::KeyAdmin,
                    expire_at: Utc::now() + Duration::hours(1),
                    last_active_at: Utc::now(),
                },
            );
            store.admin_sessions.insert(
                "tok-query".to_string(),
                AdminSession {
                    token: "tok-query".to_string(),
                    admin_pubkey: "query-only".to_string(),
                    role: AdminRole::QueryOnly,
                    expire_at: Utc::now() + Duration::hours(1),
                    last_active_at: Utc::now(),
                },
            );
        }

        for token in ["tok-super", "tok-operator", "tok-key"] {
            let mut headers = HeaderMap::new();
            headers.insert(
                "authorization",
                HeaderValue::from_str(&format!("Bearer {token}")).expect("header value"),
            );
            assert!(require_super_or_operator_or_key_admin(&state, &headers).is_ok());
        }

        let mut query_headers = HeaderMap::new();
        query_headers.insert(
            "authorization",
            HeaderValue::from_str("Bearer tok-query").expect("header value"),
        );
        assert!(require_super_or_operator_or_key_admin(&state, &query_headers).is_err());
    }

    #[test]
    fn parse_sr25519_pubkey_accepts_0x_prefix() {
        let key = "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let parsed = parse_sr25519_pubkey(key).expect("parse pubkey");
        assert_eq!(
            parsed,
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
        );
    }

    #[test]
    fn verify_admin_signature_accepts_0x_signature_prefix_for_sr25519() {
        let message = "sfid-qr-login|origin=http://127.0.0.1:5179|domain=127.0.0.1|session_id=sid|nonce=n|iat=1|exp=2";
        let (pubkey, signature) = sign_with_test_sr25519(33, message);
        assert!(verify_admin_signature(&pubkey, message, &signature));
    }

    #[test]
    fn parse_sr25519_pubkey_bytes_rejects_non_hex_pubkey() {
        assert!(
            parse_sr25519_pubkey_bytes("5D4Y9fP2U8NDDw7X9W7N6wA6ZwZP3oYfgho2dQ4q8W35bLoA")
                .is_none()
        );
    }

    #[test]
    fn pending_scope_requires_province_when_admin_is_scoped() {
        let pending = PendingRequest {
            seq: 1,
            account_pubkey: "0xP".to_string(),
            admin_province: None,
            requested_at: Utc::now(),
            callback_url: None,
            client_request_id: None,
        };
        assert!(!in_scope_pending(&pending, Some("中枢省")));

        let claimed = PendingRequest {
            admin_province: Some("中枢省".to_string()),
            ..pending
        };
        assert!(in_scope_pending(&claimed, Some("中枢省")));
        assert!(!in_scope_pending(&claimed, Some("岭南省")));
    }

    #[test]
    fn cpms_site_scope_must_match_admin_province() {
        let site = CpmsSiteKeys {
            site_sfid: "SFID-SITE-001".to_string(),
            pubkey_1: "0x1".to_string(),
            pubkey_2: "0x2".to_string(),
            pubkey_3: "0x3".to_string(),
            status: CpmsSiteStatus::Active,
            version: 1,
            last_register_issued_at: Utc::now().timestamp(),
            init_qr_payload: None,
            admin_province: "贵州省".to_string(),
            created_by: "0xSUPER".to_string(),
            created_at: Utc::now(),
            updated_by: None,
            updated_at: None,
        };
        assert!(in_scope_cpms_site(&site, Some("贵州省")));
        assert!(!in_scope_cpms_site(&site, Some("中枢省")));
    }

    #[test]
    fn validate_bind_callback_url_rejects_localhost_and_private_literals() {
        let localhost = validate_bind_callback_url("https://localhost/callback");
        assert!(localhost.is_err());
        let private_ip = validate_bind_callback_url("https://192.168.1.8/callback");
        assert!(private_ip.is_err());
    }

    #[test]
    fn chain_signature_payload_and_hash_are_deterministic() {
        let payload =
            chain_signature_payload("vote_verify", "req-1", "nonce-1", 1731000000, "fp-123");
        let sig_a = chain_signature_hex("secret-a", payload.as_str());
        let sig_b = chain_signature_hex("secret-a", payload.as_str());
        let sig_c = chain_signature_hex("secret-b", payload.as_str());
        assert!(constant_time_eq_hex(sig_a.as_str(), sig_b.as_str()));
        assert!(!constant_time_eq_hex(sig_a.as_str(), sig_c.as_str()));
    }

    #[test]
    fn chain_request_requires_replay_headers() {
        std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
        std::env::set_var(
            "SFID_CHAIN_SIGNING_SECRET",
            "test-chain-signing-secret-at-least-32",
        );
        let mut store = Store::default();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-chain-token",
            HeaderValue::from_static("test-chain-token"),
        );
        assert!(require_chain_request(&mut store, &headers, "vote_verify", "fp").is_err());
    }

    #[test]
    fn chain_request_rejects_duplicate_nonce() {
        std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
        std::env::set_var(
            "SFID_CHAIN_SIGNING_SECRET",
            "test-chain-signing-secret-at-least-32",
        );
        let mut store = Store::default();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-chain-token",
            HeaderValue::from_static("test-chain-token"),
        );
        headers.insert("x-chain-request-id", HeaderValue::from_static("req-1"));
        headers.insert("x-chain-nonce", HeaderValue::from_static("nonce-1"));
        let ts = Utc::now().timestamp();
        headers.insert(
            "x-chain-timestamp",
            HeaderValue::from_str(&ts.to_string()).expect("header value"),
        );
        let sig_payload = chain_signature_payload("vote_verify", "req-1", "nonce-1", ts, "fp-1");
        let sig = chain_signature_hex(
            "test-chain-signing-secret-at-least-32",
            sig_payload.as_str(),
        );
        headers.insert(
            "x-chain-signature",
            HeaderValue::from_str(sig.as_str()).expect("header value"),
        );
        assert!(require_chain_request(&mut store, &headers, "vote_verify", "fp-1").is_ok());

        let mut second_headers = headers.clone();
        second_headers.insert("x-chain-request-id", HeaderValue::from_static("req-2"));
        let sig_payload_2 = chain_signature_payload("vote_verify", "req-2", "nonce-1", ts, "fp-2");
        let sig2 = chain_signature_hex(
            "test-chain-signing-secret-at-least-32",
            sig_payload_2.as_str(),
        );
        second_headers.insert(
            "x-chain-signature",
            HeaderValue::from_str(sig2.as_str()).expect("header value"),
        );
        assert!(require_chain_request(&mut store, &second_headers, "vote_verify", "fp-2").is_err());
    }
}
