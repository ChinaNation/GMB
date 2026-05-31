use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use base64::Engine as _;
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
    /// 中文注释:按省分片的进程内缓存。Postgres 持久化只写模块 Store 表。
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

#[derive(Clone, Copy)]
struct DbPageCursor {
    created_at: DateTime<Utc>,
    id: i64,
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

fn encode_db_page_cursor(created_at: DateTime<Utc>, id: i64) -> String {
    let raw = format!("{}|{}", created_at.timestamp_micros(), id);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
}

fn decode_db_page_cursor(cursor: Option<&str>) -> Result<Option<DbPageCursor>, String> {
    let Some(raw_cursor) = cursor.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(raw_cursor)
        .map_err(|_| "invalid page cursor".to_string())?;
    let text = String::from_utf8(decoded).map_err(|_| "invalid page cursor".to_string())?;
    let mut parts = text.splitn(2, '|');
    let ts_micros = parts
        .next()
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| "invalid page cursor".to_string())?;
    let id = parts
        .next()
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| "invalid page cursor".to_string())?;
    let created_at = DateTime::<Utc>::from_timestamp_micros(ts_micros)
        .ok_or_else(|| "invalid page cursor".to_string())?;
    Ok(Some(DbPageCursor { created_at, id }))
}

fn citizen_status_text(status: &CitizenStatus) -> &'static str {
    match status {
        CitizenStatus::Normal => "NORMAL",
        CitizenStatus::Revoked => "REVOKED",
    }
}

fn citizen_status_from_text(status: &str) -> CitizenStatus {
    match status {
        "NORMAL" => CitizenStatus::Normal,
        _ => CitizenStatus::Revoked,
    }
}

fn citizen_bind_status_text(status: &CitizenBindStatus) -> &'static str {
    match status {
        CitizenBindStatus::Pending => "PENDING",
        CitizenBindStatus::Bound => "BOUND",
    }
}

fn institution_category_text(category: crate::sfid::InstitutionCategory) -> &'static str {
    match category {
        crate::sfid::InstitutionCategory::PublicSecurity => "PUBLIC_SECURITY",
        crate::sfid::InstitutionCategory::GovInstitution => "GOV_INSTITUTION",
        crate::sfid::InstitutionCategory::PrivateInstitution => "PRIVATE_INSTITUTION",
    }
}

fn institution_category_from_text(category: &str) -> Option<crate::sfid::InstitutionCategory> {
    match category {
        "PUBLIC_SECURITY" => Some(crate::sfid::InstitutionCategory::PublicSecurity),
        "GOV_INSTITUTION" => Some(crate::sfid::InstitutionCategory::GovInstitution),
        "PRIVATE_INSTITUTION" => Some(crate::sfid::InstitutionCategory::PrivateInstitution),
        _ => None,
    }
}

fn institution_chain_status_text(
    status: &crate::institutions::InstitutionChainStatus,
) -> &'static str {
    match status {
        crate::institutions::InstitutionChainStatus::NotRegistered => "NOT_REGISTERED",
        crate::institutions::InstitutionChainStatus::PendingRegister => "PENDING_REGISTER",
        crate::institutions::InstitutionChainStatus::Registered => "REGISTERED",
        crate::institutions::InstitutionChainStatus::RevokedOnChain => "REVOKED_ON_CHAIN",
    }
}

fn institution_chain_status_from_text(status: &str) -> crate::institutions::InstitutionChainStatus {
    match status {
        "PENDING_REGISTER" => crate::institutions::InstitutionChainStatus::PendingRegister,
        "REGISTERED" => crate::institutions::InstitutionChainStatus::Registered,
        "REVOKED_ON_CHAIN" => crate::institutions::InstitutionChainStatus::RevokedOnChain,
        _ => crate::institutions::InstitutionChainStatus::NotRegistered,
    }
}

fn multisig_chain_status_text(status: &crate::institutions::MultisigChainStatus) -> &'static str {
    match status {
        crate::institutions::MultisigChainStatus::NotOnChain => "NOT_ON_CHAIN",
        crate::institutions::MultisigChainStatus::PendingOnChain => "PENDING_ON_CHAIN",
        crate::institutions::MultisigChainStatus::ActiveOnChain => "ACTIVE_ON_CHAIN",
        crate::institutions::MultisigChainStatus::RevokedOnChain => "REVOKED_ON_CHAIN",
    }
}

fn page_from_rows<T: Serialize>(
    mut rows: Vec<(T, DateTime<Utc>, i64)>,
    page_size: usize,
) -> PageResult<T> {
    let has_more = rows.len() > page_size;
    if has_more {
        rows.truncate(page_size);
    }
    let next_cursor = if has_more {
        rows.last()
            .map(|(_, created_at, id)| encode_db_page_cursor(*created_at, *id))
    } else {
        None
    };
    PageResult {
        items: rows.into_iter().map(|(row, _, _)| row).collect(),
        page_size,
        next_cursor,
        has_more,
    }
}

fn citizen_record_exact_match(record: &CitizenRecord, keyword: &str) -> bool {
    record
        .archive_no
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case(keyword))
        .unwrap_or(false)
        || record
            .sfid_code
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case(keyword))
            .unwrap_or(false)
        || record
            .wallet_pubkey
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case(keyword))
            .unwrap_or(false)
        || record
            .wallet_address
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case(keyword))
            .unwrap_or(false)
}

fn citizen_row_from_record(record: &CitizenRecord) -> CitizenRow {
    CitizenRow {
        id: record.id,
        wallet_pubkey: record.wallet_pubkey.clone(),
        wallet_address: record.wallet_address.clone(),
        archive_no: record.archive_no.clone(),
        sfid_code: record.sfid_code.clone(),
        citizen_status: record.citizen_status.clone(),
        voting_eligible: record.voting_eligible,
        vote_status: record.computed_vote_status(),
        identity_status: record.computed_identity_status(),
        valid_from: record.archive_valid_from.clone(),
        valid_until: record.archive_valid_until.clone(),
        status_updated_at: record.status_updated_at,
        bind_status: record.bind_status(),
    }
}

fn institution_exact_match(inst: &crate::institutions::MultisigInstitution, keyword: &str) -> bool {
    inst.sfid_number.eq_ignore_ascii_case(keyword)
        || inst
            .institution_name
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case(keyword))
            .unwrap_or(false)
}

fn stable_institution_cursor_id(sfid_number: &str) -> i64 {
    sfid_number
        .as_bytes()
        .iter()
        .fold(0i64, |acc, byte| {
            acc.wrapping_mul(131).wrapping_add(*byte as i64)
        })
        .wrapping_abs()
}

fn institution_row_from_record(
    inst: &crate::institutions::MultisigInstitution,
    account_count: usize,
    created_by_name: Option<String>,
    created_by_role: Option<String>,
) -> crate::institutions::InstitutionListRow {
    crate::institutions::InstitutionListRow {
        sfid_number: inst.sfid_number.clone(),
        institution_name: inst.institution_name.clone(),
        category: inst.category,
        a3: inst.a3.clone(),
        p1: inst.p1.clone(),
        province: inst.province.clone(),
        city: inst.city.clone(),
        institution_code: inst.institution_code.clone(),
        sub_type: inst.sub_type.clone(),
        parent_sfid_number: inst.parent_sfid_number.clone(),
        chain_status: inst.chain_status.clone(),
        account_count,
        created_at: inst.created_at,
        created_by_name,
        created_by_role,
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

impl StoreWriteGuard {
    pub(crate) fn persist_or_500(&self) -> Result<(), axum::response::Response> {
        self.backend.save_store(&self.store).map_err(|err| {
            error!(error = %err, "store persist failed before response");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "store persist failed",
            )
        })
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

    fn parse_admin_role(role: &str) -> Result<AdminRole, String> {
        // 中文注释:当前只接受 SHENG_ADMIN / SHI_ADMIN;数据库出现未知角色直接拒绝启动。
        match role {
            "SHENG_ADMIN" => Ok(AdminRole::ShengAdmin),
            "SHI_ADMIN" => Ok(AdminRole::ShiAdmin),
            _ => Err(format!("invalid admin role in database: {role}")),
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
            Err(err) => Err(format!("decode {table} snapshot failed: {err}")),
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
                    role: Self::parse_admin_role(role_text.as_str())?,
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

        tx.commit()
            .map_err(|e| format!("commit admin sync transaction failed: {e}"))?;
        Ok(())
    }

    fn init_current_schema(conn: &mut postgres::Client) -> Result<(), String> {
        // 中文注释:SFID 还未发行正式版,启动时只创建当前目标结构;不执行历史 SQL 脚本。
        conn.batch_execute(
            "CREATE TABLE IF NOT EXISTS provinces (
                province_name TEXT PRIMARY KEY
             );

             CREATE TABLE IF NOT EXISTS admins (
                admin_id BIGINT PRIMARY KEY,
                admin_pubkey TEXT NOT NULL UNIQUE,
                admin_name TEXT NOT NULL,
                role TEXT NOT NULL CHECK (role IN ('SHENG_ADMIN', 'SHI_ADMIN')),
                built_in BOOLEAN NOT NULL DEFAULT FALSE,
                created_by TEXT NOT NULL DEFAULT 'SYSTEM',
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ,
                city TEXT NOT NULL DEFAULT ''
             );
             CREATE INDEX IF NOT EXISTS idx_admins_role ON admins(role);

             CREATE TABLE IF NOT EXISTS sheng_admin_scope (
                admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
                province_name TEXT NOT NULL REFERENCES provinces(province_name) ON DELETE RESTRICT
             );
             CREATE INDEX IF NOT EXISTS idx_sheng_admin_scope_province_name
                ON sheng_admin_scope(province_name);

             CREATE TABLE IF NOT EXISTS shi_admin_scope (
                admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
                sheng_admin_id BIGINT NOT NULL REFERENCES admins(admin_id) ON DELETE RESTRICT,
                province_name TEXT NULL REFERENCES provinces(province_name) ON DELETE RESTRICT
             );
             CREATE INDEX IF NOT EXISTS idx_shi_admin_scope_sheng
                ON shi_admin_scope(sheng_admin_id);

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

	             CREATE TABLE IF NOT EXISTS sfid_citizens (
	                id BIGINT PRIMARY KEY,
	                wallet_pubkey TEXT UNIQUE,
	                wallet_address TEXT UNIQUE,
	                archive_no TEXT UNIQUE,
	                sfid_code TEXT UNIQUE,
	                province_code TEXT NOT NULL,
	                city_code TEXT NOT NULL,
	                citizen_status TEXT NOT NULL CHECK (citizen_status IN ('NORMAL', 'REVOKED')),
	                voting_eligible BOOLEAN NOT NULL,
	                valid_from TEXT,
	                valid_until TEXT,
	                status_updated_at BIGINT,
	                bind_status TEXT NOT NULL CHECK (bind_status IN ('PENDING', 'BOUND')),
	                bound_at TIMESTAMPTZ,
	                bound_by TEXT,
	                created_at TIMESTAMPTZ NOT NULL
	             );
	             CREATE INDEX IF NOT EXISTS idx_sfid_citizens_scope_created
	                ON sfid_citizens(province_code, city_code, created_at DESC, id DESC);
	             CREATE INDEX IF NOT EXISTS idx_sfid_citizens_province_created
	                ON sfid_citizens(province_code, created_at DESC, id DESC);

	             CREATE TABLE IF NOT EXISTS sfid_institutions (
	                id BIGSERIAL PRIMARY KEY,
	                sfid_number TEXT NOT NULL UNIQUE,
	                institution_name TEXT,
	                category TEXT NOT NULL CHECK (category IN ('PUBLIC_SECURITY', 'GOV_INSTITUTION', 'PRIVATE_INSTITUTION')),
	                a3 TEXT NOT NULL,
	                p1 TEXT NOT NULL,
	                province TEXT NOT NULL,
	                city TEXT NOT NULL,
	                province_code TEXT NOT NULL,
	                city_code TEXT NOT NULL,
	                institution_code TEXT NOT NULL,
	                sub_type TEXT,
	                parent_sfid_number TEXT,
	                chain_status TEXT NOT NULL CHECK (chain_status IN ('NOT_REGISTERED', 'PENDING_REGISTER', 'REGISTERED', 'REVOKED_ON_CHAIN')),
	                chain_tx_hash TEXT,
	                chain_block_number BIGINT,
	                chain_synced_at TIMESTAMPTZ,
	                created_by TEXT NOT NULL,
	                created_at TIMESTAMPTZ NOT NULL
	             );
	             CREATE INDEX IF NOT EXISTS idx_sfid_institutions_scope_created
	                ON sfid_institutions(category, province, city, created_at DESC, id DESC);
	             CREATE INDEX IF NOT EXISTS idx_sfid_institutions_province_created
	                ON sfid_institutions(category, province, created_at DESC, id DESC);
	             CREATE INDEX IF NOT EXISTS idx_sfid_institutions_name
	                ON sfid_institutions(institution_name);

	             CREATE TABLE IF NOT EXISTS sfid_institution_accounts (
	                sfid_number TEXT NOT NULL REFERENCES sfid_institutions(sfid_number) ON DELETE CASCADE,
	                account_name TEXT NOT NULL,
	                duoqian_address TEXT,
	                chain_status TEXT NOT NULL CHECK (chain_status IN ('NOT_ON_CHAIN', 'PENDING_ON_CHAIN', 'ACTIVE_ON_CHAIN', 'REVOKED_ON_CHAIN')),
	                chain_tx_hash TEXT,
	                chain_block_number BIGINT,
	                chain_synced_at TIMESTAMPTZ,
	                created_by TEXT NOT NULL,
	                created_at TIMESTAMPTZ NOT NULL,
	                PRIMARY KEY (sfid_number, account_name)
	             );
	             CREATE INDEX IF NOT EXISTS idx_sfid_institution_accounts_sfid
	                ON sfid_institution_accounts(sfid_number);

	             CREATE TABLE IF NOT EXISTS tx_records (
                id BIGSERIAL PRIMARY KEY,
                block_number BIGINT NOT NULL,
                extrinsic_index SMALLINT,
                event_index SMALLINT NOT NULL,
                tx_type TEXT NOT NULL,
                from_address TEXT,
                to_address TEXT,
                amount_fen BIGINT NOT NULL,
                fee_fen BIGINT,
                block_timestamp TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             CREATE INDEX IF NOT EXISTS idx_tx_records_from
                ON tx_records (from_address, block_number DESC);
             CREATE INDEX IF NOT EXISTS idx_tx_records_to
                ON tx_records (to_address, block_number DESC);
             CREATE INDEX IF NOT EXISTS idx_tx_records_block
                ON tx_records (block_number DESC);
             CREATE INDEX IF NOT EXISTS idx_tx_records_type
                ON tx_records (tx_type);

             CREATE TABLE IF NOT EXISTS tx_indexer_state (
                id INT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
                last_indexed_block BIGINT NOT NULL DEFAULT 0,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             INSERT INTO tx_indexer_state (id, last_indexed_block)
             VALUES (1, 0)
             ON CONFLICT (id) DO NOTHING;",
        )
        .map_err(|e| format!("init current schema failed: {e}"))?;
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
            StoreBackend::init_current_schema(&mut bootstrap)?;
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

    pub(crate) fn upsert_citizen_row(&self, record: &CitizenRecord) -> Result<(), String> {
        match &self.backend {
            StoreBackend::Memory(_) => Ok(()),
            StoreBackend::Postgres {
                clients,
                next_client_idx,
            } => {
                let record = record.clone();
                StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
                    let citizen_status = record
                        .citizen_status
                        .as_ref()
                        .map(citizen_status_text)
                        .unwrap_or("REVOKED");
                    let bind_status = citizen_bind_status_text(&record.bind_status());
                    let id = i64::try_from(record.id)
                        .map_err(|_| "citizen id exceeds i64".to_string())?;
                    let province_code = record.province_code.clone().unwrap_or_default();
                    let city_code = record.city_code.clone().unwrap_or_default();
                    conn.execute(
                        "INSERT INTO sfid_citizens (
                            id, wallet_pubkey, wallet_address, archive_no, sfid_code,
                            province_code, city_code, citizen_status, voting_eligible,
                            valid_from, valid_until, status_updated_at, bind_status,
                            bound_at, bound_by, created_at
                         ) VALUES (
                            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
                         )
                         ON CONFLICT (id) DO UPDATE SET
                            wallet_pubkey = EXCLUDED.wallet_pubkey,
                            wallet_address = EXCLUDED.wallet_address,
                            archive_no = EXCLUDED.archive_no,
                            sfid_code = EXCLUDED.sfid_code,
                            province_code = EXCLUDED.province_code,
                            city_code = EXCLUDED.city_code,
                            citizen_status = EXCLUDED.citizen_status,
                            voting_eligible = EXCLUDED.voting_eligible,
                            valid_from = EXCLUDED.valid_from,
                            valid_until = EXCLUDED.valid_until,
                            status_updated_at = EXCLUDED.status_updated_at,
                            bind_status = EXCLUDED.bind_status,
                            bound_at = EXCLUDED.bound_at,
                            bound_by = EXCLUDED.bound_by,
                            created_at = EXCLUDED.created_at",
                        &[
                            &id,
                            &record.wallet_pubkey,
                            &record.wallet_address,
                            &record.archive_no,
                            &record.sfid_code,
                            &province_code,
                            &city_code,
                            &citizen_status,
                            &record.voting_eligible,
                            &record.archive_valid_from,
                            &record.archive_valid_until,
                            &record.status_updated_at,
                            &bind_status,
                            &record.bound_at,
                            &record.bound_by,
                            &record.created_at,
                        ],
                    )
                    .map_err(|e| format!("upsert sfid_citizens failed: {e}"))?;
                    Ok(())
                })
            }
        }
    }

    pub(crate) fn list_citizens_exact(
        &self,
        keyword: &str,
        province_code: Option<&str>,
        city_code: Option<&str>,
        cursor: Option<&str>,
        page_size: usize,
    ) -> Result<PageResult<CitizenRow>, String> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(PageResult {
                items: Vec::new(),
                page_size,
                next_cursor: None,
                has_more: false,
            });
        }
        let cursor = decode_db_page_cursor(cursor)?;
        match &self.backend {
            StoreBackend::Memory(inner) => {
                let store = inner
                    .read()
                    .map_err(|_| "memory store read lock poisoned".to_string())?;
                let mut rows = store
                    .citizen_records
                    .values()
                    .filter(|record| record.bind_status() == CitizenBindStatus::Bound)
                    .filter(|record| {
                        province_code
                            .map_or(true, |code| record.province_code.as_deref() == Some(code))
                            && city_code
                                .map_or(true, |code| record.city_code.as_deref() == Some(code))
                    })
                    .filter(|record| citizen_record_exact_match(record, keyword))
                    .filter(|record| {
                        cursor.map_or(true, |c| {
                            let id = i64::try_from(record.id).unwrap_or(i64::MAX);
                            record.created_at < c.created_at
                                || (record.created_at == c.created_at && id < c.id)
                        })
                    })
                    .map(|record| {
                        (
                            citizen_row_from_record(record),
                            record.created_at,
                            i64::try_from(record.id).unwrap_or(i64::MAX),
                        )
                    })
                    .collect::<Vec<_>>();
                rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));
                Ok(page_from_rows(rows, page_size))
            }
            StoreBackend::Postgres {
                clients,
                next_client_idx,
            } => {
                let keyword = keyword.to_string();
                let province_code = province_code.map(str::to_string);
                let city_code = city_code.map(str::to_string);
                StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
                    let cursor_created_at = cursor.map(|c| c.created_at);
                    let cursor_id = cursor.map(|c| c.id).unwrap_or(i64::MAX);
                    let fetch_limit = i64::try_from(page_size.saturating_add(1))
                        .map_err(|_| "page_size too large".to_string())?;
                    let rows = conn
                        .query(
                            "SELECT id, wallet_pubkey, wallet_address, archive_no, sfid_code,
                                    citizen_status, voting_eligible, valid_from, valid_until,
                                    status_updated_at, bind_status, province_code, city_code,
                                    bound_at, bound_by, created_at
                             FROM sfid_citizens
                             WHERE bind_status = 'BOUND'
                               AND ($1::text IS NULL OR province_code = $1)
                               AND ($2::text IS NULL OR city_code = $2)
                               AND (
                                    archive_no = $3 OR sfid_code = $3
                                    OR lower(wallet_pubkey) = lower($3)
                                    OR lower(wallet_address) = lower($3)
                               )
                               AND (
                                    $4::timestamptz IS NULL
                                    OR created_at < $4
                                    OR (created_at = $4 AND id < $5)
                               )
                             ORDER BY created_at DESC, id DESC
                             LIMIT $6",
                            &[
                                &province_code,
                                &city_code,
                                &keyword,
                                &cursor_created_at,
                                &cursor_id,
                                &fetch_limit,
                            ],
                        )
                        .map_err(|e| format!("query sfid_citizens failed: {e}"))?;
                    let mut output = Vec::with_capacity(rows.len());
                    for row in rows {
                        let id_i64: i64 = row.get(0);
                        let created_at: DateTime<Utc> = row.get(15);
                        let record = CitizenRecord {
                            id: u64::try_from(id_i64).unwrap_or(0),
                            wallet_pubkey: row.get(1),
                            wallet_address: row.get(2),
                            archive_no: row.get(3),
                            sfid_code: row.get(4),
                            citizen_status: Some(citizen_status_from_text(
                                row.get::<_, String>(5).as_str(),
                            )),
                            voting_eligible: row.get(6),
                            archive_valid_from: row.get(7),
                            archive_valid_until: row.get(8),
                            status_updated_at: row.get(9),
                            sfid_signature: None,
                            province_code: row.get(11),
                            city_code: row.get(12),
                            bound_at: row.get(13),
                            bound_by: row.get(14),
                            created_at,
                        };
                        output.push((citizen_row_from_record(&record), created_at, id_i64));
                    }
                    Ok(page_from_rows(output, page_size))
                })
            }
        }
    }

    pub(crate) fn upsert_institution_row(
        &self,
        inst: &crate::institutions::MultisigInstitution,
    ) -> Result<(), String> {
        match &self.backend {
            StoreBackend::Memory(_) => Ok(()),
            StoreBackend::Postgres {
                clients,
                next_client_idx,
            } => {
                let inst = inst.clone();
                StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
                    let category = institution_category_text(inst.category);
                    let chain_status = institution_chain_status_text(&inst.chain_status);
                    let chain_block_number = inst.chain_block_number.map(|v| v as i64);
                    conn.execute(
                        "INSERT INTO sfid_institutions (
                            sfid_number, institution_name, category, a3, p1, province, city,
                            province_code, city_code, institution_code, sub_type,
                            parent_sfid_number, chain_status, chain_tx_hash,
                            chain_block_number, chain_synced_at, created_by, created_at
                         ) VALUES (
                            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
                         )
                         ON CONFLICT (sfid_number) DO UPDATE SET
                            institution_name = EXCLUDED.institution_name,
                            category = EXCLUDED.category,
                            a3 = EXCLUDED.a3,
                            p1 = EXCLUDED.p1,
                            province = EXCLUDED.province,
                            city = EXCLUDED.city,
                            province_code = EXCLUDED.province_code,
                            city_code = EXCLUDED.city_code,
                            institution_code = EXCLUDED.institution_code,
                            sub_type = EXCLUDED.sub_type,
                            parent_sfid_number = EXCLUDED.parent_sfid_number,
                            chain_status = EXCLUDED.chain_status,
                            chain_tx_hash = EXCLUDED.chain_tx_hash,
                            chain_block_number = EXCLUDED.chain_block_number,
                            chain_synced_at = EXCLUDED.chain_synced_at,
                            created_by = EXCLUDED.created_by,
                            created_at = EXCLUDED.created_at",
                        &[
                            &inst.sfid_number,
                            &inst.institution_name,
                            &category,
                            &inst.a3,
                            &inst.p1,
                            &inst.province,
                            &inst.city,
                            &inst.province_code,
                            &inst.city_code,
                            &inst.institution_code,
                            &inst.sub_type,
                            &inst.parent_sfid_number,
                            &chain_status,
                            &inst.chain_tx_hash,
                            &chain_block_number,
                            &inst.chain_synced_at,
                            &inst.created_by,
                            &inst.created_at,
                        ],
                    )
                    .map_err(|e| format!("upsert sfid_institutions failed: {e}"))?;
                    Ok(())
                })
            }
        }
    }

    pub(crate) fn upsert_institution_account_row(
        &self,
        account: &crate::institutions::MultisigAccount,
    ) -> Result<(), String> {
        match &self.backend {
            StoreBackend::Memory(_) => Ok(()),
            StoreBackend::Postgres {
                clients,
                next_client_idx,
            } => {
                let account = account.clone();
                StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
                    let chain_status = multisig_chain_status_text(&account.chain_status);
                    let chain_block_number = account.chain_block_number.map(|v| v as i64);
                    conn.execute(
                        "INSERT INTO sfid_institution_accounts (
                            sfid_number, account_name, duoqian_address, chain_status,
                            chain_tx_hash, chain_block_number, chain_synced_at,
                            created_by, created_at
                         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                         ON CONFLICT (sfid_number, account_name) DO UPDATE SET
                            duoqian_address = EXCLUDED.duoqian_address,
                            chain_status = EXCLUDED.chain_status,
                            chain_tx_hash = EXCLUDED.chain_tx_hash,
                            chain_block_number = EXCLUDED.chain_block_number,
                            chain_synced_at = EXCLUDED.chain_synced_at,
                            created_by = EXCLUDED.created_by,
                            created_at = EXCLUDED.created_at",
                        &[
                            &account.sfid_number,
                            &account.account_name,
                            &account.duoqian_address,
                            &chain_status,
                            &account.chain_tx_hash,
                            &chain_block_number,
                            &account.chain_synced_at,
                            &account.created_by,
                            &account.created_at,
                        ],
                    )
                    .map_err(|e| format!("upsert sfid_institution_accounts failed: {e}"))?;
                    Ok(())
                })
            }
        }
    }

    pub(crate) fn delete_institution_account_row(
        &self,
        sfid_number: &str,
        account_name: &str,
    ) -> Result<(), String> {
        match &self.backend {
            StoreBackend::Memory(_) => Ok(()),
            StoreBackend::Postgres {
                clients,
                next_client_idx,
            } => {
                let sfid_number = sfid_number.to_string();
                let account_name = account_name.to_string();
                StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
                    conn.execute(
                        "DELETE FROM sfid_institution_accounts
                         WHERE sfid_number = $1 AND account_name = $2",
                        &[&sfid_number, &account_name],
                    )
                    .map_err(|e| format!("delete sfid_institution_accounts failed: {e}"))?;
                    Ok(())
                })
            }
        }
    }

    pub(crate) fn list_institutions_exact(
        &self,
        category: Option<&str>,
        province: Option<&str>,
        city: Option<&str>,
        keyword: &str,
        cursor: Option<&str>,
        page_size: usize,
    ) -> Result<PageResult<crate::institutions::InstitutionListRow>, String> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(PageResult {
                items: Vec::new(),
                page_size,
                next_cursor: None,
                has_more: false,
            });
        }
        let cursor = decode_db_page_cursor(cursor)?;
        match &self.backend {
            StoreBackend::Memory(inner) => {
                let store = inner
                    .read()
                    .map_err(|_| "memory store read lock poisoned".to_string())?;
                let mut rows = store
                    .multisig_institutions
                    .values()
                    .filter(|inst| {
                        category
                            .and_then(institution_category_from_text)
                            .map_or(true, |cat| inst.category == cat)
                    })
                    .filter(|inst| province.map_or(true, |v| inst.province == v))
                    .filter(|inst| city.map_or(true, |v| inst.city == v))
                    .filter(|inst| institution_exact_match(inst, keyword))
                    .filter(|inst| {
                        cursor.map_or(true, |c| {
                            let id = stable_institution_cursor_id(inst.sfid_number.as_str());
                            inst.created_at < c.created_at
                                || (inst.created_at == c.created_at && id < c.id)
                        })
                    })
                    .map(|inst| {
                        let account_count = store
                            .multisig_accounts
                            .values()
                            .filter(|acc| acc.sfid_number == inst.sfid_number)
                            .count();
                        (
                            institution_row_from_record(inst, account_count, None, None),
                            inst.created_at,
                            stable_institution_cursor_id(inst.sfid_number.as_str()),
                        )
                    })
                    .collect::<Vec<_>>();
                rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));
                Ok(page_from_rows(rows, page_size))
            }
            StoreBackend::Postgres {
                clients,
                next_client_idx,
            } => {
                let category = category.map(str::to_string);
                let province = province.map(str::to_string);
                let city = city.map(str::to_string);
                let keyword = keyword.to_string();
                StoreBackend::with_postgres_client(clients, next_client_idx, move |conn| {
                    let cursor_created_at = cursor.map(|c| c.created_at);
                    let cursor_id = cursor.map(|c| c.id).unwrap_or(i64::MAX);
                    let fetch_limit = i64::try_from(page_size.saturating_add(1))
                        .map_err(|_| "page_size too large".to_string())?;
                    let rows = conn
                        .query(
                            "SELECT i.id, i.sfid_number, i.institution_name, i.category,
                                    i.a3, i.p1, i.province, i.city, i.institution_code,
                                    i.sub_type, i.parent_sfid_number, i.chain_status,
                                    i.created_at, COALESCE(ac.account_count, 0),
                                    a.admin_name, a.role
                             FROM sfid_institutions i
                             LEFT JOIN (
                                SELECT sfid_number, COUNT(*)::BIGINT AS account_count
                                FROM sfid_institution_accounts
                                GROUP BY sfid_number
                             ) ac ON ac.sfid_number = i.sfid_number
                             LEFT JOIN admins a ON lower(a.admin_pubkey) = lower(i.created_by)
                             WHERE ($1::text IS NULL OR i.category = $1)
                               AND ($2::text IS NULL OR i.province = $2)
                               AND ($3::text IS NULL OR i.city = $3)
                               AND (
                                    i.sfid_number = $4
                                    OR lower(COALESCE(i.institution_name, '')) = lower($4)
                               )
                               AND (
                                    $5::timestamptz IS NULL
                                    OR i.created_at < $5
                                    OR (i.created_at = $5 AND i.id < $6)
                               )
                             ORDER BY i.created_at DESC, i.id DESC
                             LIMIT $7",
                            &[
                                &category,
                                &province,
                                &city,
                                &keyword,
                                &cursor_created_at,
                                &cursor_id,
                                &fetch_limit,
                            ],
                        )
                        .map_err(|e| format!("query sfid_institutions failed: {e}"))?;
                    let mut output = Vec::with_capacity(rows.len());
                    for row in rows {
                        let id: i64 = row.get(0);
                        let category_text: String = row.get(3);
                        let category = institution_category_from_text(category_text.as_str())
                            .ok_or_else(|| {
                                format!("invalid institution category: {category_text}")
                            })?;
                        let chain_status_text: String = row.get(11);
                        let account_count_i64: i64 = row.get(13);
                        let created_by_name: Option<String> = row.get(14);
                        let created_by_role: Option<String> = row.get(15);
                        let inst = crate::institutions::MultisigInstitution {
                            sfid_number: row.get(1),
                            institution_name: row.get(2),
                            category,
                            a3: row.get(4),
                            p1: row.get(5),
                            province: row.get(6),
                            city: row.get(7),
                            province_code: String::new(),
                            city_code: String::new(),
                            institution_code: row.get(8),
                            sub_type: row.get(9),
                            parent_sfid_number: row.get(10),
                            chain_status: institution_chain_status_from_text(
                                chain_status_text.as_str(),
                            ),
                            chain_tx_hash: None,
                            chain_block_number: None,
                            chain_synced_at: None,
                            created_by: String::new(),
                            created_at: row.get(12),
                        };
                        output.push((
                            institution_row_from_record(
                                &inst,
                                usize::try_from(account_count_i64).unwrap_or(0),
                                created_by_name,
                                created_by_role,
                            ),
                            inst.created_at,
                            id,
                        ));
                    }
                    Ok(page_from_rows(output, page_size))
                })
            }
        }
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
    // 中文注释:ShardedStore 只作为进程内分片缓存,主数据由模块 Store 表保存。
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
    ensure_builtin_province_admins(&state);
    info!("initialized runtime state with defaults");
    // 中文注释:任务卡 6 启动对账:按 sfid 工具市清单对齐全部公安局机构。
    app_core::runtime_ops::backfill_and_reconcile_public_security(&state);
    app_core::runtime_ops::cleanup_stale_citizen_bind_records(&state);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(async move {
        // 中文注释:启动时只从进程内 ShardedStore 后端加载空缓存;
        // 数据从模块 Store 快照装入后同步到进程内缓存。
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
                "/api/v1/admin/operators/:id",
                patch(admins::actions::update_operator_login_state),
            )
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
            .route(
                "/api/v1/admin/sheng-admins/:id",
                patch(admins::actions::update_sheng_admin_login_state),
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
                "/api/v1/institutions/public-security",
                get(institutions::handler::list_public_security_institutions),
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
        "admin pubkey already exists as sheng admin" => "SFID_ADMIN_PUBKEY_EXISTS_AS_SHENG_ADMIN",
        "admin pubkey already exists as shi admin" => "SFID_ADMIN_PUBKEY_EXISTS_AS_SHI_ADMIN",
        "sheng admin province limit reached" => "SFID_ADMIN_SHENG_ADMIN_PROVINCE_LIMIT_REACHED",
        "shi admin city limit reached" => "SFID_ADMIN_SHI_ADMIN_CITY_LIMIT_REACHED",
        "store persist failed" => "SFID_STORE_PERSIST_FAILED",
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
