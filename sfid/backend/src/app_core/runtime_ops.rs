use chrono::{DateTime, Duration, Utc};
use reqwest::Url;
use serde::Serialize;
use std::{
    collections::HashMap,
    hash::Hash,
    net::{IpAddr, SocketAddr},
    time::Duration as StdDuration,
};
use tracing::warn;

use crate::key_admins;
use crate::key_admins::chain_proof::SignatureEnvelope;
use crate::sfid::province::provinces;
use crate::sfid::{generate_sfid_code, GenerateSfidInput};
use crate::*;

/// 首次初始化：从 province.rs 硬编码数据创建 43 个内置机构管理员
pub(crate) fn seed_sheng_admins(state: &AppState) {
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
                role: AdminRole::ShengAdmin,
                status: AdminStatus::Active,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: now,
                updated_at: Some(now),
                city: String::new(),
            },
        );
        store
            .sheng_admin_province_by_pubkey
            .insert(item.pubkey.to_string(), item.name.to_string());
    }
}

/// 从 DB 加载后，补充 province.rs 中新增的省份（DB 中缺失的）
/// - DB 是唯一真实数据源，已有省份的公钥不会被覆盖
/// - 只补缺：province.rs 中有但 DB 中没有的省份，用默认公钥创建
/// - 同时修正 role 字段（旧 DB 可能存的是 ShengAdmin）
pub(crate) fn sync_builtin_sheng_admins(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    let now = Utc::now();

    // 修正已有机构管理员的 role（从旧 DB 加载可能是错误的 role）
    let institution_pubkeys: Vec<String> = store
        .sheng_admin_province_by_pubkey
        .keys()
        .cloned()
        .collect();
    for pubkey in &institution_pubkeys {
        if let Some(user) = store.admin_users_by_pubkey.get_mut(pubkey) {
            if user.role != AdminRole::ShengAdmin {
                user.role = AdminRole::ShengAdmin;
            }
        }
    }

    // 补充 DB 中缺失的省份（province.rs 有但 DB 没有的）
    let existing_provinces: std::collections::HashSet<String> = store
        .sheng_admin_province_by_pubkey
        .values()
        .cloned()
        .collect();

    for item in provinces().iter() {
        let province = item.name.to_string();
        if existing_provinces.contains(&province) {
            continue; // DB 已有该省份，不覆盖
        }
        // DB 中缺失该省份，用 province.rs 的默认公钥创建
        let pubkey = item.pubkey.to_string();
        let max_id = store
            .admin_users_by_pubkey
            .values()
            .map(|u| u.id)
            .max()
            .unwrap_or(0);
        store.admin_users_by_pubkey.insert(
            pubkey.clone(),
            AdminUser {
                id: max_id + 1,
                admin_pubkey: pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::ShengAdmin,
                status: AdminStatus::Active,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: now,
                updated_at: Some(now),
                city: String::new(),
            },
        );
        store
            .sheng_admin_province_by_pubkey
            .insert(pubkey, province);
    }
}

pub(crate) fn cleanup_consumed_qr_ids(store: &mut Store, now: DateTime<Utc>) {
    store
        .consumed_qr_ids
        .retain(|_, consumed_at| *consumed_at > now - Duration::hours(24));
}

pub(crate) fn cleanup_pending_bind_scans(store: &mut Store, now: DateTime<Utc>) {
    let now_ts = now.timestamp();
    store.pending_bind_scan_by_qr_id.retain(|_, pending| {
        pending.scanned_at > now - Duration::hours(24) && pending.expire_at >= now_ts
    });
}

pub(crate) fn cleanup_pending_bind_requests(store: &mut Store, now: DateTime<Utc>) {
    store
        .pending_by_pubkey
        .retain(|_, pending| pending.requested_at > now - Duration::hours(24));
}

fn pending_bind_cleanup_interval_seconds() -> i64 {
    std::env::var("SFID_PENDING_BIND_CLEANUP_INTERVAL_SECONDS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(60)
}

pub(crate) fn maybe_cleanup_pending_bind_requests(store: &mut Store, now: DateTime<Utc>) {
    let interval = Duration::seconds(pending_bind_cleanup_interval_seconds());
    let should_cleanup = store
        .pending_bind_last_cleanup_at
        .map(|last| now - last >= interval)
        .unwrap_or(true);
    if should_cleanup {
        cleanup_pending_bind_requests(store, now);
        store.pending_bind_last_cleanup_at = Some(now);
    }
}

pub(crate) fn vote_cache_key(account_pubkey: &str, proposal_id: Option<u64>) -> String {
    match proposal_id {
        Some(id) => format!("{account_pubkey}:{id}"),
        None => format!("{account_pubkey}:none"),
    }
}

pub(crate) fn cleanup_vote_cache(store: &mut Store, now: DateTime<Utc>) {
    store
        .vote_verify_cache
        .retain(|_, entry| entry.cached_at > now - Duration::seconds(5));
}

pub(crate) fn invalidate_vote_cache_for_pubkey(store: &mut Store, account_pubkey: &str) {
    store
        .vote_verify_cache
        .retain(|_, entry| entry.account_pubkey != account_pubkey);
}

pub(crate) fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn normalize_account_pubkey(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let hex_body = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .or_else(|| {
            if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                Some(trimmed)
            } else {
                None
            }
        });
    if let Some(body) = hex_body {
        if body.len() != 64 || !body.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        return Some(format!("0x{}", body.to_ascii_lowercase()));
    }

    None
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

pub(crate) fn default_bind_callback_url() -> Option<String> {
    normalize_optional(std::env::var("SFID_BIND_CALLBACK_URL").ok())
}

pub(crate) fn default_bind_callback_auth_token() -> Option<String> {
    normalize_optional(std::env::var("SFID_BIND_CALLBACK_AUTH_TOKEN").ok())
}

pub(crate) fn enqueue_bind_callback_job(
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

#[derive(Debug, Clone)]
pub(crate) struct ResolvedCallbackTarget {
    pub(crate) url: Url,
    pub(crate) host: String,
    pub(crate) resolved_addrs: Vec<SocketAddr>,
    pub(crate) host_is_ip: bool,
}

fn retry_or_fail_callback_job(store: &mut Store, mut job: BindCallbackJob, err: String) {
    job.attempts += 1;
    if job.attempts >= job.max_attempts {
        store.metrics.bind_callback_failed_total += 1;
        append_audit_log(
            store,
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
        job.last_error = Some(err);
        store.bind_callback_jobs.push(job);
    }
}

pub(crate) async fn ensure_callback_delivery_target_safe(
    callback_url: &str,
) -> Result<ResolvedCallbackTarget, String> {
    let parsed = Url::parse(callback_url).map_err(|_| "invalid callback url".to_string())?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "callback url host is missing".to_string())?
        .to_ascii_lowercase();
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| "callback url port is missing".to_string())?;
    let host_is_ip = host.parse::<IpAddr>().is_ok();
    let mut resolved_addrs = Vec::new();
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_callback_ip(ip) {
            return Err("callback target resolves to private/local address".to_string());
        }
        resolved_addrs.push(SocketAddr::new(ip, port));
    } else {
        let resolved = tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|e| format!("callback dns resolve failed: {e}"))?;
        for addr in resolved {
            if is_blocked_callback_ip(addr.ip()) {
                return Err("callback target resolves to private/local address".to_string());
            }
            resolved_addrs.push(addr);
        }
    }
    if resolved_addrs.is_empty() {
        return Err("callback dns resolve returned no addresses".to_string());
    }
    Ok(ResolvedCallbackTarget {
        url: parsed,
        host,
        resolved_addrs,
        host_is_ip,
    })
}

pub(crate) async fn bind_callback_worker(state: AppState) {
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

        for job in due_jobs {
            let target = match ensure_callback_delivery_target_safe(job.callback_url.as_str()).await
            {
                Ok(v) => v,
                Err(err) => {
                    let mut store = match state.store.write() {
                        Ok(guard) => guard,
                        Err(lock_err) => {
                            warn!(error = %lock_err, "bind callback worker lock failed on dns validation");
                            continue;
                        }
                    };
                    retry_or_fail_callback_job(&mut store, job, err);
                    continue;
                }
            };
            let mut client_builder = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(StdDuration::from_secs(10));
            if !target.host_is_ip {
                for addr in &target.resolved_addrs {
                    client_builder = client_builder.resolve(target.host.as_str(), *addr);
                }
            }
            let client = match client_builder.build() {
                Ok(v) => v,
                Err(err) => {
                    let mut store = match state.store.write() {
                        Ok(guard) => guard,
                        Err(lock_err) => {
                            warn!(error = %lock_err, "bind callback worker lock failed on client build");
                            continue;
                        }
                    };
                    retry_or_fail_callback_job(
                        &mut store,
                        job,
                        format!("build callback client failed: {err}"),
                    );
                    continue;
                }
            };
            let mut request = client
                .post(target.url.clone())
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
                    retry_or_fail_callback_job(
                        &mut store,
                        job,
                        format!("http status {}", resp.status().as_u16()),
                    );
                }
                Err(err) => {
                    retry_or_fail_callback_job(&mut store, job, err.to_string());
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

pub(crate) fn append_audit_log(
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

pub(crate) fn append_audit_log_with_meta(
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

pub(crate) fn seed_demo_record(state: &AppState) {
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
    let demo_province = provinces()
        .first()
        .map(|item| item.name)
        .unwrap_or("中原省");
    let demo_city = "省辖市";

    for seq in 1..=total {
        let pubkey = format!("0xDEMO_PUBKEY_{seq:04}");
        if seq <= bound_total {
            let archive = format!("CIV-DEMO-{seq:04}");
            // 中文注释：演示数据也统一走正式 SFID 生成工具，避免启动时再产出旧格式 SFID 演示码。
            let sfid = match generate_sfid_code(GenerateSfidInput {
                account_pubkey: &pubkey,
                a3: "GMR",
                p1: "",
                province: demo_province,
                city: demo_city,
                institution: "ZG",
            }) {
                Ok(v) => v,
                Err(err) => {
                    warn!(error = %err, "failed to generate demo sfid");
                    return;
                }
            };
            let binding_payload = BindingPayload {
                kind: "bind",
                version: "v1",
                account_pubkey: pubkey.clone(),
                archive_index: archive.clone(),
                sfid_code: sfid.clone(),
                issued_at: now.timestamp(),
            };
            let proof = match make_signature_envelope(state, &binding_payload) {
                Ok(v) => v,
                Err(err) => {
                    warn!(error = %err, "failed to sign demo binding payload");
                    return;
                }
            };
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
                    runtime_bind_binding_id: None,
                    runtime_bind_bind_nonce: None,
                    runtime_bind_signature: None,
                    runtime_bind_key_id: None,
                    runtime_bind_key_version: None,
                    runtime_bind_alg: None,
                    runtime_bind_signer_pubkey: None,
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

pub(crate) fn make_signature_envelope<T: Serialize>(
    state: &AppState,
    payload: &T,
) -> Result<SignatureEnvelope, String> {
    let seed = state
        .signing_seed_hex
        .read()
        .map(|v| v.clone())
        .map_err(|_| "signing seed read lock poisoned".to_string())?;
    let signing_key =
        key_admins::chain_keyring::try_load_signing_key_from_seed(seed.expose_secret())?;
    key_admins::chain_proof::make_signature_envelope(
        &state.key_id,
        &state.key_version,
        &state.key_alg,
        &signing_key,
        payload,
    )
}

/// 任务卡 2:把 legacy `multisig_sfid_records` 拆成 `multisig_institutions` + `multisig_accounts`。
///
/// 中文注释:
/// - 链端 `(sfid_id, name) → duoqian_address` 是 DoubleMap,一个 sfid 下可挂多个 name。
/// - 老结构把"机构"和"账户"混在一张表里,`site_sfid` 当主键,`institution_name` 被当作链上 name。
/// - 新结构按两层拆:sfid_id → Institution(每 sfid 唯一),(sfid_id, account_name) → Account(链上 name)。
///
/// 这个迁移**幂等**:每次启动后端都会运行一次,已迁移过的记录会跳过。
/// 老结构 `multisig_sfid_records` **不删除**,作为只读兜底便于回滚。
pub(crate) fn migrate_legacy_multisig_to_two_layer(state: &AppState) {
    use crate::institutions::model::{MultisigAccount, MultisigInstitution};
    use crate::institutions::store as inst_store;
    use crate::sfid::{classify, A3, InstitutionCode};

    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    if store.multisig_sfid_records.is_empty() {
        return;
    }
    let legacy: Vec<_> = store.multisig_sfid_records.values().cloned().collect();
    let mut migrated = 0usize;
    for rec in legacy {
        // 幂等:机构已存在则跳过(避免覆盖任务卡 2 之后新建的机构)
        if inst_store::contains_institution(&store, &rec.site_sfid) {
            continue;
        }

        // 分类:按 a3 + institution_code + institution_name 推导
        let a3 = match A3::from_str(&rec.a3) {
            Some(v) => v,
            None => {
                warn!(sfid = %rec.site_sfid, a3 = %rec.a3, "migrate: unknown a3, skip");
                continue;
            }
        };
        let code = match InstitutionCode::from_str(&rec.institution_code) {
            Some(v) => v,
            None => {
                warn!(
                    sfid = %rec.site_sfid,
                    code = %rec.institution_code,
                    "migrate: unknown institution_code, skip"
                );
                continue;
            }
        };
        let category = match classify(a3, code, &rec.institution_name) {
            Some(v) => v,
            None => {
                warn!(
                    sfid = %rec.site_sfid,
                    "migrate: not a valid institution (citizen a3?), skip"
                );
                continue;
            }
        };

        // 中文注释:从 sfid_id r5 段后 3 字符解析 city_code(任务卡 6 新增字段)
        let city_code = rec
            .site_sfid
            .split('-')
            .nth(1)
            .and_then(|r5| if r5.len() >= 5 { Some(r5[2..5].to_string()) } else { None })
            .unwrap_or_default();
        let inst = MultisigInstitution {
            sfid_id: rec.site_sfid.clone(),
            // 中文注释:老结构里 institution_name 实际被当作"账户名"发上链,
            // 这里拆两层后保留为机构展示名 + 同时作为默认账户名。
            institution_name: rec.institution_name.clone(),
            category,
            a3: rec.a3.clone(),
            p1: rec.p1.clone(),
            province: rec.province.clone(),
            city: rec.city.clone(),
            province_code: rec.province_code.clone(),
            city_code,
            institution_code: rec.institution_code.clone(),
            created_by: rec.created_by.clone(),
            created_at: rec.created_at,
        };
        inst_store::insert_institution(&mut store, inst);

        let account = MultisigAccount {
            sfid_id: rec.site_sfid.clone(),
            // 账户名沿用 institution_name(因为链上就是用它派生的 duoqian_address)
            account_name: rec.institution_name.clone(),
            duoqian_address: None,
            chain_status: rec.chain_status.clone(),
            chain_tx_hash: rec.chain_tx_hash.clone(),
            chain_block_number: rec.chain_block_number,
            created_by: rec.created_by.clone(),
            created_at: rec.created_at,
        };
        inst_store::insert_account(&mut store, account);
        migrated += 1;
    }
    if migrated > 0 {
        tracing::info!(
            count = migrated,
            "migrated legacy multisig_sfid_records → multisig_institutions + multisig_accounts"
        );
    }
}

/// 任务卡 6:后端启动时 backfill + 对 43 省全量对账公安局机构。
///
/// 中文注释:
/// 1. 先调 `backfill_public_security_city_codes` 给老公安局记录补 `city_code`
///    (任务卡 6 新增字段),否则 reconcile 会按 city_code 误删。
/// 2. 然后按 sfid 工具权威清单 reconcile 每个省:
///    增加缺失的市公安局、删除已从市清单剔除的、改名同步。
pub(crate) fn backfill_and_reconcile_public_security(state: &AppState) {
    use crate::institutions::service::{
        backfill_public_security_city_codes, reconcile_public_security_for_province,
    };
    use crate::sfid::province::PROVINCES;

    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    let fixed = backfill_public_security_city_codes(&mut store);
    if fixed > 0 {
        tracing::info!(count = fixed, "backfilled city_code for legacy public security institutions");
    }

    let mut total_inserted = 0usize;
    let mut total_updated = 0usize;
    let mut total_removed = 0usize;
    for p in PROVINCES.iter() {
        let r = reconcile_public_security_for_province(&mut store, p.name, "SYSTEM");
        total_inserted += r.inserted;
        total_updated += r.updated;
        total_removed += r.removed;
    }
    tracing::info!(
        inserted = total_inserted,
        updated = total_updated,
        removed = total_removed,
        "public security reconcile finished for 43 provinces"
    );
}

/// 任务卡 `20260408-sfid-public-security-cpms-embed`:
/// 启动时清理孤儿 CPMS 站点。
///
/// 中文注释:`cpms_site_keys` 里的记录通过
/// `(admin_province, city_name, institution_code)` 元组关联到
/// `multisig_institutions`。开发期如果某个公安局机构被 reconcile 删掉了,
/// 对应的 CPMS 站点就成了孤儿——老 UI 能看见,新详情页入口看不见。
/// 直接硬删,不留数据包袱(`feedback_chain_in_dev.md`)。
pub(crate) fn cleanup_orphan_cpms_sites(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    // 构建 (province, city, institution_code) 合法三元组集合:取自所有机构
    // (不限 PUBLIC_SECURITY,因为 CPMS 站点理论上可挂任意机构;目前业务只有公安局用,
    //  但收紧范围无益)。
    let valid: std::collections::HashSet<(String, String, String)> = store
        .multisig_institutions
        .values()
        .map(|inst| {
            (
                inst.province.clone(),
                inst.city.clone(),
                inst.institution_code.clone(),
            )
        })
        .collect();
    let orphans: Vec<String> = store
        .cpms_site_keys
        .values()
        .filter(|site| {
            !valid.contains(&(
                site.admin_province.clone(),
                site.city_name.clone(),
                site.institution_code.clone(),
            ))
        })
        .map(|site| site.site_sfid.clone())
        .collect();
    if orphans.is_empty() {
        return;
    }
    let sample: Vec<String> = orphans.iter().take(10).cloned().collect();
    for site_sfid in &orphans {
        store.cpms_site_keys.remove(site_sfid);
    }
    tracing::info!(
        count = orphans.len(),
        sample = ?sample,
        "cleaned up orphan CPMS sites (no matching institution)"
    );
}
