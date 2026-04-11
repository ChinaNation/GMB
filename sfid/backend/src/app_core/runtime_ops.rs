use chrono::{DateTime, Duration, Utc};
use reqwest::Url;
use std::{
    collections::HashMap,
    hash::Hash,
    net::{IpAddr, SocketAddr},
    time::Duration as StdDuration,
};
use tracing::warn;

use crate::sfid::province::provinces;
use crate::*;

/// 首次初始化：从 province.rs 硬编码数据创建 43 个内置机构管理员
pub(crate) fn seed_sheng_admins(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "seed_sheng_admins: store RwLock poisoned — initialization skipped");
            return;
        }
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
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
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
        Err(e) => {
            tracing::error!(error = %e, "sync_builtin_sheng_admins: store RwLock poisoned — sync skipped");
            return;
        }
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
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
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

#[allow(dead_code)]
pub(crate) fn cleanup_pending_bind_scans(store: &mut Store, now: DateTime<Utc>) {
    let now_ts = now.timestamp();
    store.pending_bind_scan_by_qr_id.retain(|_, pending| {
        pending.scanned_at > now - Duration::hours(24) && pending.expire_at >= now_ts
    });
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


pub(crate) fn default_bind_callback_auth_token() -> Option<String> {
    normalize_optional(std::env::var("SFID_BIND_CALLBACK_AUTH_TOKEN").ok())
}

#[allow(dead_code)]
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
        Err(e) => { tracing::error!(error = %e, "store RwLock poisoned"); return; },
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
/// Phase 2 Day 3：cpms_site_keys 迁移到 sharded_store
///
/// 中文注释:`cpms_site_keys` 里的记录通过
/// `(admin_province, city_name, institution_code)` 元组关联到
/// `multisig_institutions`。开发期如果某个公安局机构被 reconcile 删掉了,
/// 对应的 CPMS 站点就成了孤儿——老 UI 能看见,新详情页入口看不见。
/// 直接硬删,不留数据包袱(`feedback_chain_in_dev.md`)。
pub(crate) async fn cleanup_orphan_cpms_sites(state: &AppState) {
    // 构建 (province, city, institution_code) 合法三元组集合:取自所有机构(legacy store)
    let valid: std::collections::HashSet<(String, String, String)> = match state.store.read() {
        Ok(store) => store
            .multisig_institutions
            .values()
            .map(|inst| {
                (
                    inst.province.clone(),
                    inst.city.clone(),
                    inst.institution_code.clone(),
                )
            })
            .collect(),
        Err(e) => { tracing::error!(error = %e, "store RwLock poisoned"); return; },
    };
    // 遍历所有省分片，收集孤儿 site_sfid 及其所在省
    let mut orphans_by_province: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let scan_result = state
        .sharded_store
        .for_each_province(|province, shard| {
            for site in shard.cpms_site_keys.values() {
                if !valid.contains(&(
                    site.admin_province.clone(),
                    site.city_name.clone(),
                    site.institution_code.clone(),
                )) {
                    orphans_by_province
                        .entry(province.to_string())
                        .or_default()
                        .push(site.site_sfid.clone());
                }
            }
        })
        .await;
    if let Err(e) = scan_result {
        tracing::warn!(error = %e, "cleanup_orphan_cpms_sites: shard scan failed");
        return;
    }
    if orphans_by_province.is_empty() {
        return;
    }
    let total: usize = orphans_by_province.values().map(|v| v.len()).sum();
    let sample: Vec<String> = orphans_by_province
        .values()
        .flatten()
        .take(10)
        .cloned()
        .collect();
    for (province, sfids) in &orphans_by_province {
        let sfids_owned = sfids.clone();
        if let Err(e) = state
            .sharded_store
            .write_province(province, move |shard| {
                for sfid in &sfids_owned {
                    shard.cpms_site_keys.remove(sfid);
                }
            })
            .await
        {
            tracing::warn!(province, error = %e, "cleanup_orphan_cpms_sites: write_province failed");
        }
    }

    // 双写过渡期:sharded_store + legacy store 同步写(清理孤儿 cpms)
    {
        match state.store.write() {
            Ok(mut store) => {
                for sfids in orphans_by_province.values() {
                    for sfid in sfids {
                        store.cpms_site_keys.remove(sfid);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (cleanup orphan cpms, shard already committed)");
            }
        }
    }

    tracing::info!(
        count = total,
        sample = ?sample,
        "cleaned up orphan CPMS sites (no matching institution)"
    );
}
