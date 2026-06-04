use chrono::Utc;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::admins::province_admins::sheng_admin_mains;
use crate::crypto::pubkey::{normalize_admin_pubkey, same_admin_pubkey};
use crate::*;

/// 中文注释:启动时以 `admins/province_admins.rs` 为初始省级管理员唯一真源。
pub(crate) fn ensure_builtin_province_admins(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "ensure_builtin_province_admins: store write failed");
            return;
        }
    };
    let before = admin_bootstrap_signature(&store);
    ensure_builtin_province_admins_in_store(&mut store);
    if before == admin_bootstrap_signature(&store) {
        store.skip_persist_on_drop();
    }
}

type AdminBootstrapSignature = (
    Vec<(
        u64,
        String,
        String,
        String,
        bool,
        String,
        DateTimeSeconds,
        String,
    )>,
    Vec<(String, String)>,
    u64,
);

type DateTimeSeconds = i64;

fn admin_bootstrap_signature(store: &Store) -> AdminBootstrapSignature {
    let mut admins = store
        .admin_users_by_pubkey
        .values()
        .map(|admin| {
            let role = match admin.role {
                AdminRole::ShengAdmin => "SHENG_ADMIN",
                AdminRole::ShiAdmin => "SHI_ADMIN",
            };
            (
                admin.id,
                normalized_admin_pubkey_key(&admin.admin_pubkey),
                admin.admin_name.clone(),
                role.to_string(),
                admin.built_in,
                normalized_admin_pubkey_key(&admin.created_by),
                admin.created_at.timestamp(),
                admin.city.clone(),
            )
        })
        .collect::<Vec<_>>();
    admins.sort();
    let mut scopes = store
        .sheng_admin_province_by_pubkey
        .iter()
        .map(|(pubkey, province)| (normalized_admin_pubkey_key(pubkey), province.clone()))
        .collect::<Vec<_>>();
    scopes.sort();
    (admins, scopes, store.next_admin_user_id)
}

fn ensure_builtin_province_admins_in_store(store: &mut Store) {
    let now = Utc::now();
    let source_pubkeys: HashSet<String> = sheng_admin_mains()
        .iter()
        .map(|item| normalized_admin_pubkey_key(item.pubkey))
        .collect();

    let stale_builtin_pubkeys = store
        .admin_users_by_pubkey
        .iter()
        .filter_map(|(pubkey, user)| {
            let is_stale = user.role == AdminRole::ShengAdmin
                && user.built_in
                && !source_pubkeys.contains(&normalized_admin_pubkey_key(pubkey));
            is_stale.then(|| {
                let province = store
                    .sheng_admin_province_by_pubkey
                    .iter()
                    .find(|(candidate, _)| same_admin_pubkey(candidate.as_str(), pubkey.as_str()))
                    .map(|(_, province)| province.clone());
                (pubkey.clone(), province)
            })
        })
        .collect::<Vec<_>>();
    for (pubkey, province) in stale_builtin_pubkeys {
        let source_pubkey = province.as_deref().and_then(source_pubkey_for_province);
        store.admin_users_by_pubkey.remove(&pubkey);
        store
            .sheng_admin_province_by_pubkey
            .retain(|candidate, _| !same_admin_pubkey(candidate, pubkey.as_str()));
        remove_admin_runtime_state(store, pubkey.as_str());
        if let Some(source_pubkey) = source_pubkey {
            for user in store.admin_users_by_pubkey.values_mut() {
                if user.role == AdminRole::ShiAdmin
                    && same_admin_pubkey(user.created_by.as_str(), pubkey.as_str())
                {
                    user.created_by = source_pubkey.to_string();
                    user.updated_at = Some(now);
                }
            }
        }
    }

    for (idx, item) in sheng_admin_mains().iter().enumerate() {
        let pubkey = item.pubkey.to_string();
        let existing_key = store
            .admin_users_by_pubkey
            .keys()
            .find(|candidate| same_admin_pubkey(candidate.as_str(), item.pubkey))
            .cloned();
        let mut user = existing_key
            .and_then(|key| store.admin_users_by_pubkey.remove(&key))
            .unwrap_or_else(|| AdminUser {
                id: (idx as u64) + 1,
                admin_pubkey: pubkey.clone(),
                admin_name: format!("{}省级管理员", item.province),
                role: AdminRole::ShengAdmin,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: now,
                updated_at: Some(now),
                city: String::new(),
            });
        user.admin_pubkey = pubkey.clone();
        if user.admin_name.trim().is_empty() {
            user.admin_name = format!("{}省级管理员", item.province);
        }
        user.role = AdminRole::ShengAdmin;
        user.built_in = true;
        user.created_by = "SYSTEM".to_string();
        user.updated_at = Some(now);
        user.city.clear();
        store.admin_users_by_pubkey.insert(pubkey.clone(), user);
        store
            .sheng_admin_province_by_pubkey
            .retain(|candidate, _| !same_admin_pubkey(candidate.as_str(), item.pubkey));
        store
            .sheng_admin_province_by_pubkey
            .insert(pubkey, item.province.to_string());
    }

    let active_sheng_pubkeys: HashSet<String> = store
        .admin_users_by_pubkey
        .iter()
        .filter_map(|(pubkey, user)| {
            (user.role == AdminRole::ShengAdmin).then(|| normalized_admin_pubkey_key(pubkey))
        })
        .collect();
    store
        .sheng_admin_province_by_pubkey
        .retain(|pubkey, _| active_sheng_pubkeys.contains(&normalized_admin_pubkey_key(pubkey)));

    let max_id = store
        .admin_users_by_pubkey
        .values()
        .map(|user| user.id)
        .max()
        .unwrap_or(0);
    if store.next_admin_user_id <= max_id {
        store.next_admin_user_id = max_id + 1;
    }
}

fn normalized_admin_pubkey_key(pubkey: &str) -> String {
    normalize_admin_pubkey(pubkey)
        .unwrap_or_else(|| pubkey.trim().to_string())
        .to_ascii_lowercase()
}

fn source_pubkey_for_province(province: &str) -> Option<&'static str> {
    sheng_admin_mains()
        .iter()
        .find(|item| item.province == province)
        .map(|item| item.pubkey)
}

fn remove_admin_runtime_state(store: &mut Store, pubkey: &str) {
    store
        .admin_sessions
        .retain(|_, session| !same_admin_pubkey(session.admin_pubkey.as_str(), pubkey));
    store
        .admin_passkeys_by_credential_id
        .retain(|_, record| !same_admin_pubkey(record.admin_pubkey.as_str(), pubkey));
    store
        .admin_passkey_registration_challenges
        .retain(|_, challenge| !same_admin_pubkey(challenge.admin_pubkey.as_str(), pubkey));
    store
        .admin_action_challenges
        .retain(|_, challenge| !same_admin_pubkey(challenge.actor_pubkey.as_str(), pubkey));
    store
        .admin_security_grants
        .retain(|_, grant| !same_admin_pubkey(grant.actor_pubkey.as_str(), pubkey));
    store
        .login_challenges
        .retain(|_, challenge| !same_admin_pubkey(challenge.admin_pubkey.as_str(), pubkey));
    store
        .qr_login_results
        .retain(|_, result| !same_admin_pubkey(result.admin_pubkey.as_str(), pubkey));
}

#[allow(dead_code)]
// 中文注释:vote_cache_key / cleanup_vote_cache 配套于已下架的 verify_vote_eligibility
// dead route(`POST /api/v1/vote/verify`),2026-05-01 一并移除。当前 chain pull 的
// /api/v1/app/vote/credential 不依赖 cache,vote_verify_cache 只在投票账户绑定状态
// 变化时才被 invalidate_vote_cache_for_pubkey 清理。
pub(crate) fn cleanup_stale_citizen_bind_records(state: &AppState) -> usize {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "cleanup_stale_citizen_bind_records: store write failed");
            return 0;
        }
    };
    let stale_ids = store
        .citizen_records
        .values()
        .filter(|record| record.bind_status() != CitizenBindStatus::Bound)
        .map(|record| record.id)
        .collect::<Vec<_>>();
    if stale_ids.is_empty() {
        return 0;
    }
    for citizen_id in &stale_ids {
        store.citizen_records.remove(citizen_id);
    }
    let bound_ids = store
        .citizen_records
        .iter()
        .filter(|(_, record)| record.bind_status() == CitizenBindStatus::Bound)
        .map(|(citizen_id, _)| *citizen_id)
        .collect::<std::collections::HashSet<_>>();
    store
        .citizen_id_by_archive_no
        .retain(|_, citizen_id| bound_ids.contains(citizen_id));
    store
        .citizen_id_by_wallet_pubkey
        .retain(|_, citizen_id| bound_ids.contains(citizen_id));
    store
        .citizen_id_by_sfid_code
        .retain(|_, citizen_id| bound_ids.contains(citizen_id));
    tracing::info!(
        count = stale_ids.len(),
        "cleaned stale citizen bind records at startup"
    );
    stale_ids.len()
}

pub(crate) fn invalidate_vote_cache_for_pubkey(store: &mut Store, account_pubkey: &str) {
    store
        .vote_verify_cache
        .retain(|_, entry| entry.account_pubkey != account_pubkey);
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
/// 1. 先调 `backfill_public_security_city_code_fields` 给老公安局记录补 `city_code`
///    (任务卡 6 新增字段),否则 reconcile 会按 city_code 误删。
/// 2. 然后按 sfid 工具权威清单 reconcile 每个省:
///    增加缺失的市公安局、删除已从市清单剔除的、改名同步。
pub(crate) fn backfill_and_reconcile_public_security(state: &AppState) {
    use crate::gov::service::{
        backfill_public_security_city_code_fields, reconcile_public_security_for_province,
    };
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "store RwLock poisoned");
            return;
        }
    };
    let fixed = backfill_public_security_city_code_fields(&mut store);
    if fixed > 0 {
        tracing::info!(
            count = fixed,
            "backfilled city_code for public security institutions"
        );
    }

    let mut total_inserted = 0usize;
    let mut total_updated = 0usize;
    let mut total_removed = 0usize;
    for p in crate::china::provinces().iter() {
        let r = reconcile_public_security_for_province(&mut store, p.name, "SYSTEM");
        total_inserted += r.inserted;
        total_updated += r.updated;
        total_removed += r.removed;
    }
    if fixed == 0 && total_inserted == 0 && total_updated == 0 && total_removed == 0 {
        store.skip_persist_on_drop();
    }
    tracing::info!(
        inserted = total_inserted,
        updated = total_updated,
        removed = total_removed,
        "public security reconcile finished for 43 provinces"
    );
}

/// 显式对账普通公权/宪法机构目录。
///
/// 中文注释:正常后端启动不得调用该函数。自动公权机构是持久化目录,
/// 只允许首次初始化或行政区划变化时显式执行,避免健康检查前全量写库。
pub(crate) fn reconcile_official_institutions_explicit(
    state: &AppState,
    scope: crate::gov::service::OfficialReconcileScope,
    force_row_sync: bool,
) -> crate::gov::service::OfficialReconcileReport {
    use crate::gov::service::{reconcile_official_institutions_in_scope, OfficialReconcileReport};

    let (report, upsert_institutions, upsert_accounts) = {
        let mut store = match state.store.write() {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "store RwLock poisoned");
                return OfficialReconcileReport::default();
            }
        };
        let report = reconcile_official_institutions_in_scope(&mut store, "SYSTEM", scope);
        let row_sync_sfids = if force_row_sync {
            report.target_sfids.iter().cloned().collect::<HashSet<_>>()
        } else {
            report.touched_sfids.iter().cloned().collect::<HashSet<_>>()
        };
        let upsert_institutions = report
            .target_sfids
            .iter()
            .filter(|sfid| row_sync_sfids.contains(*sfid))
            .filter_map(|sfid| store.multisig_institutions.get(sfid).cloned())
            .collect::<Vec<_>>();
        let upsert_accounts = store
            .multisig_accounts
            .values()
            .filter(|account| row_sync_sfids.contains(&account.sfid_number))
            .cloned()
            .collect::<Vec<_>>();
        (report, upsert_institutions, upsert_accounts)
    };

    if !report.removed_sfids.is_empty() {
        if let Err(e) = state
            .store
            .delete_institution_rows_by_sfids(&report.removed_sfids)
        {
            tracing::error!(error = %e, "official institution stale row cleanup failed");
        }
    }
    for inst in &upsert_institutions {
        if let Err(e) = state.store.upsert_institution_row(inst) {
            tracing::error!(sfid = %inst.sfid_number, error = %e, "official institution row upsert failed");
        }
    }
    for account in &upsert_accounts {
        if let Err(e) = state.store.upsert_institution_account_row(account) {
            tracing::error!(sfid = %account.sfid_number, account = %account.account_name, error = %e, "official institution account row upsert failed");
        }
    }

    tracing::info!(
        inserted = report.inserted,
        updated = report.updated,
        account_inserted = report.account_inserted,
        removed = report.removed,
        total_after = report.total_after,
        row_sync_forced = force_row_sync,
        "official institution reconcile finished"
    );
    report
}

/// 把持久化 Store 快照里的 CPMS 安装授权同步到进程内分片缓存。
///
/// 中文注释:`store_cpms` 是 CPMS 授权主数据；`sharded_store` 只是运行期按省检索缓存。
/// ARCHIVE 验真需要按省扫描授权并用 `install_secret` 解 `geo_seal`，所以 SFID 启动后
/// 必须先把持久化授权恢复进分片缓存，否则重启后会误报 `geo_seal cannot be decrypted`。
pub(crate) async fn sync_cpms_sites_to_sharded(state: &AppState) {
    let sites: Vec<CpmsSiteKeys> = match state.store.read() {
        Ok(store) => store.cpms_site_keys.values().cloned().collect(),
        Err(e) => {
            tracing::warn!(error = %e, "sync_cpms_sites_to_sharded: store read failed");
            return;
        }
    };
    if sites.is_empty() {
        return;
    }

    let mut by_province: HashMap<String, Vec<CpmsSiteKeys>> = HashMap::new();
    for site in sites {
        let province = if !site.admin_province.trim().is_empty() {
            site.admin_province.clone()
        } else {
            match crate::china::province_name_by_code(site.province_code.as_str()) {
                Some(name) => name.to_string(),
                None => {
                    tracing::warn!(
                        sfid_number = %site.site_sfid,
                        province_code = %site.province_code,
                        "sync_cpms_sites_to_sharded: cannot resolve site province"
                    );
                    continue;
                }
            }
        };
        by_province.entry(province).or_default().push(site);
    }

    let mut total_synced = 0usize;
    for (province, group) in by_province {
        let group_len = group.len();
        let write_result = state
            .sharded_store
            .write_province(&province, move |shard| {
                for site in group {
                    shard.cpms_site_keys.insert(site.site_sfid.clone(), site);
                }
            })
            .await;
        match write_result {
            Ok(()) => total_synced += group_len,
            Err(e) => {
                tracing::warn!(
                    province = %province,
                    error = %e,
                    "sync_cpms_sites_to_sharded: shard write failed"
                );
            }
        }
    }

    tracing::info!(
        count = total_synced,
        "CPMS site authorizations synced to sharded_store"
    );
}

/// 任务卡 `20260408-sfid-public-security-cpms-embed`:
/// 启动时清理孤儿 CPMS 站点。
///
/// 中文注释:清理 CPMS 授权缓存中已无对应机构的孤儿站点。
///
/// 中文注释:`cpms_site_keys` 里的记录通过
/// `(admin_province, city_name, institution_code)` 元组关联到
/// `multisig_institutions`。开发期如果某个公安局机构被 reconcile 删掉了,
/// 对应的 CPMS 站点就成了孤儿——老 UI 能看见,新详情页入口看不见。
/// 直接硬删,不留数据包袱(`feedback_chain_in_dev.md`)。
pub(crate) async fn cleanup_orphan_cpms_sites(state: &AppState) {
    // 构建 (province, city, institution_code) 合法三元组集合:取自所有机构模块快照。
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
        Err(e) => {
            tracing::error!(error = %e, "store RwLock poisoned");
            return;
        }
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

    // 同步写分片缓存 + 模块 Store 快照(清理孤儿 CPMS)。
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
                tracing::warn!(error = %e, "module store snapshot write failed (cleanup orphan cpms, shard already committed)");
            }
        }
    }

    tracing::info!(
        count = total,
        sample = ?sample,
        "cleaned up orphan CPMS sites (no matching institution)"
    );
}
