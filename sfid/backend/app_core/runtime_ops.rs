use chrono::{DateTime, Duration, Utc};
use std::{collections::HashMap, hash::Hash};

use crate::sheng_admins::province_admins::sheng_admin_mains;
use crate::*;

/// 首次初始化：从 sheng_admins/province_admins.rs 硬编码数据创建 43 个内置省管理员。
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
    for (idx, item) in sheng_admin_mains().iter().enumerate() {
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
            .insert(item.pubkey.to_string(), item.province.to_string());
    }
}

/// 从 DB 加载后，补充 province_admins.rs 中新增的省管理员（DB 中缺失的）
/// - DB 是唯一真实数据源，已有省份的公钥不会被覆盖
/// - 只补缺：province_admins.rs 中有但 DB 中没有的省份，用默认公钥创建
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

    // 补充 DB 中缺失的省份（province_admins.rs 有但 DB 没有的）
    let existing_provinces: std::collections::HashSet<String> = store
        .sheng_admin_province_by_pubkey
        .values()
        .cloned()
        .collect();

    for item in sheng_admin_mains().iter() {
        let province = item.province.to_string();
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
        Err(e) => {
            tracing::error!(error = %e, "store RwLock poisoned");
            return;
        }
    };
    let fixed = backfill_public_security_city_codes(&mut store);
    if fixed > 0 {
        tracing::info!(
            count = fixed,
            "backfilled city_code for public security institutions"
        );
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

/// 把 `backfill_and_reconcile_public_security` 新写入的公安局机构 + 2 条默认账户
/// 从模块 Store 快照幂等同步到进程内分片缓存。
///
/// 为什么需要这一步:
/// - `reconcile_public_security_for_province` 是同步函数,只持全局 `&mut Store`,
///   无法写 sharded_store(sharded 接口是 async)
/// - ShardedStore 当前是进程内缓存,启动后需要从模块 Store 快照同步一次
///
/// 所以启动时 reconcile 新建的公安局机构和它们的主账户 / 费用账户,前端按省读
/// sharded_store 否则看不到。本函数在 tokio runtime 起来后跑一次,
/// 把模块 Store 快照里所有 `PublicSecurity` 机构及其默认账户写入分片缓存,**幂等**(or_insert)。
pub(crate) async fn sync_public_security_to_sharded(state: &AppState) {
    use crate::institutions::model::{account_key_to_string, MultisigAccount, MultisigInstitution};
    use crate::institutions::MultisigChainStatus;
    use crate::sfid::InstitutionCategory;

    // 1. 从模块 Store 快照取所有公安局机构 + 其 2 条默认账户
    let snapshot: Vec<(MultisigInstitution, Vec<MultisigAccount>)> = match state.store.read() {
        Ok(store) => {
            let institutions: Vec<MultisigInstitution> = store
                .multisig_institutions
                .values()
                .filter(|i| matches!(i.category, InstitutionCategory::PublicSecurity))
                .cloned()
                .collect();
            institutions
                .into_iter()
                .map(|inst| {
                    let accs: Vec<MultisigAccount> = store
                        .multisig_accounts
                        .values()
                        .filter(|a| a.sfid_number == inst.sfid_number)
                        .cloned()
                        .collect();
                    (inst, accs)
                })
                .collect()
        }
        Err(e) => {
            tracing::warn!(error = %e, "sync_public_security_to_sharded: store read failed");
            return;
        }
    };

    let mut inst_written = 0usize;
    let mut acc_written = 0usize;

    // 2. 按省分组 → 一次 write_province 写入
    use std::collections::HashMap;
    let mut by_province: HashMap<String, Vec<(MultisigInstitution, Vec<MultisigAccount>)>> =
        HashMap::new();
    for (inst, accs) in snapshot {
        by_province
            .entry(inst.province.clone())
            .or_default()
            .push((inst, accs));
    }
    for (province, group) in by_province {
        let group_len = group.len();
        let acc_count: usize = group.iter().map(|(_, a)| a.len()).sum();
        let write_result = state
            .sharded_store
            .write_province(&province, move |shard| {
                for (inst, accs) in group {
                    shard
                        .multisig_institutions
                        .entry(inst.sfid_number.clone())
                        .or_insert(inst.clone());
                    for acc in accs {
                        let key = account_key_to_string(&acc.sfid_number, &acc.account_name);
                        // 已存在账户保留链上状态/地址;不存在才补
                        shard.multisig_accounts.entry(key).or_insert(acc);
                    }
                }
            })
            .await;
        match write_result {
            Ok(()) => {
                inst_written += group_len;
                acc_written += acc_count;
            }
            Err(e) => {
                tracing::warn!(
                    province = %province,
                    error = %e,
                    "sync_public_security_to_sharded: shard write failed"
                );
            }
        }
    }

    // 3. 对模块 Store 快照里存在但缺默认账户的公安局,再补齐一次默认账户。
    //    走 DUOQIAN_V1 本地派生;同时写模块 Store 快照 + 分片缓存。
    let missing: Vec<(String, String)> = {
        let mut out: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
        if let Ok(store) = state.store.read() {
            for inst in store
                .multisig_institutions
                .values()
                .filter(|i| matches!(i.category, InstitutionCategory::PublicSecurity))
            {
                for name in crate::institutions::service::DEFAULT_ACCOUNT_NAMES {
                    let key = account_key_to_string(&inst.sfid_number, name);
                    if !store.multisig_accounts.contains_key(&key) {
                        out.insert((inst.province.clone(), inst.sfid_number.clone()));
                    }
                }
            }
        }
        out.into_iter().collect()
    };
    if !missing.is_empty() {
        if let Ok(mut store) = state.store.write() {
            for (_, sfid) in &missing {
                crate::institutions::service::insert_default_accounts_into_global_store(
                    &mut store, sfid, "SYSTEM",
                );
            }
        }
        // 同步到 sharded
        let missing_by_prov: HashMap<String, Vec<String>> =
            missing
                .into_iter()
                .fold(HashMap::new(), |mut acc, (prov, sfid)| {
                    acc.entry(prov).or_default().push(sfid);
                    acc
                });
        for (province, sfids) in missing_by_prov {
            let sfids_clone = sfids.clone();
            let backfilled =
                sfids.len() * crate::institutions::service::DEFAULT_ACCOUNT_NAMES.len();
            let write_result = state
                .sharded_store
                .write_province(&province, move |shard| {
                    let now = chrono::Utc::now();
                    for sfid in &sfids_clone {
                        for name in crate::institutions::service::DEFAULT_ACCOUNT_NAMES {
                            let key = account_key_to_string(sfid, name);
                            let addr =
                                crate::institutions::derive::derive_duoqian_address(sfid, name);
                            shard
                                .multisig_accounts
                                .entry(key)
                                .or_insert_with(|| MultisigAccount {
                                    sfid_number: sfid.clone(),
                                    account_name: (*name).to_string(),
                                    duoqian_address: addr,
                                    chain_status: MultisigChainStatus::NotOnChain,
                                    chain_synced_at: None,
                                    chain_tx_hash: None,
                                    chain_block_number: None,
                                    created_by: "SYSTEM".to_string(),
                                    created_at: now,
                                });
                        }
                    }
                })
                .await;
            if write_result.is_ok() {
                acc_written += backfilled;
            }
        }
    }

    tracing::info!(
        institutions_synced = inst_written,
        accounts_synced = acc_written,
        "public security sharded_store sync finished"
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
