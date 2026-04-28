// 中文注释:Phase 2 Day 2 —— legacy Store → 分片结构首次迁移。
//
// 任务卡 `20260410-sfid-store-shard-by-province` 第 6 节:
//   - 幂等:`store_shards` 表非空则跳过(说明已经迁过了)
//   - 空库首次启动:从内存中的 legacy `Store` 快照按省拆分
//   - 单事务:44 行(43 省 + 1 global)一次性写入
//
// 归属原则(impl.md 3.3 / 3.4 节,字段不存在的按实际代码为准):
//   - 省分片(StoreShard):本省机构、账户、CPMS 站点、citizen 记录、
//     档案导入、绑定流程、ShiAdmin、SFID 生成历史、奖励状态、回调任务
//   - 全局(GlobalShard):KeyAdmin/ShengAdmin 本体、登录 challenge/session、
//     幂等池、审计日志、链请求幂等、RSA 密钥、keyring、服务指标
//
// **归属不明确的字段一律保守下沉到 GlobalShard**,写锁开销可接受,
// 等 Day 3 handler 改造时再下沉到省分片。
//
// 省份识别规则:
//   - MultisigInstitution:直接读 `.province` 字段
//   - MultisigAccount:通过 key `sfid_id|account_name` 反查 institution.province
//   - CpmsSiteKeys:直接读 `.admin_province` 字段
//   - AdminUser(ShiAdmin):通过 `created_by → sheng_admin_province_by_pubkey` 反查
//   - CitizenRecord:通过 province_code(2 字符)→ province_name 反查,
//     反查表从 multisig_institutions 中就地构造

use std::collections::HashMap;
use std::sync::{atomic::AtomicUsize, Arc, Mutex};

use crate::models::{AdminRole, Store};
use crate::store_shards::shard_types::{GlobalShard, StoreShard};

/// 入口:若 `store_shards` 表已有数据则跳过,否则从 `legacy_store` 拆出分片并写库。
///
/// 参数:
/// - `clients` / `next_idx`:复用现有 Postgres 连接池
/// - `legacy_store`:启动阶段从 runtime_cache_entries + admins 表重建的内存快照
pub(crate) async fn migrate_legacy_store_if_needed(
    clients: Arc<Vec<Mutex<postgres::Client>>>,
    next_idx: Arc<AtomicUsize>,
    legacy_store: &Store,
) -> Result<(), String> {
    // 双写过渡期:每次启动都从 legacy store 同步到 shard,
    // 确保 reconcile / seed 等启动阶段对 legacy store 的写入同步到 shard。
    // 使用 UPSERT(INSERT ... ON CONFLICT DO UPDATE)代替 DELETE+INSERT,
    // 避免每次启动清空 44 个分片再重新写入的性能浪费。

    let start = std::time::Instant::now();

    // 从 legacy_store 拆出省分片 map + global。
    let (shards, global) = split_legacy_store(legacy_store);
    let province_count = shards.len();

    // 单事务批量 UPSERT。
    let shards_payload: Vec<(String, serde_json::Value)> = {
        let mut out = Vec::with_capacity(shards.len() + 1);
        for (province, shard) in &shards {
            let v =
                serde_json::to_value(shard).map_err(|e| format!("encode shard {province}: {e}"))?;
            out.push((province.clone(), v));
        }
        let gv = serde_json::to_value(&global).map_err(|e| format!("encode global shard: {e}"))?;
        out.push(("global".to_string(), gv));
        out
    };

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        if clients.is_empty() {
            return Err("postgres client pool is empty".to_string());
        }
        let idx = next_idx.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % clients.len();
        let mut conn = clients[idx]
            .lock()
            .map_err(|_| "postgres client lock poisoned".to_string())?;
        let mut tx = conn
            .transaction()
            .map_err(|e| format!("begin migration tx: {e}"))?;
        for (key, payload) in &shards_payload {
            tx.execute(
                "INSERT INTO store_shards (shard_key, payload, updated_at, version)
                 VALUES ($1, $2, now(), 1)
                 ON CONFLICT (shard_key) DO UPDATE SET payload = $2, updated_at = now()",
                &[key, payload],
            )
            .map_err(|e| format!("upsert shard {key}: {e}"))?;
        }
        tx.commit()
            .map_err(|e| format!("commit migration tx: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("migration write join: {e}"))??;

    tracing::info!(
        provinces = province_count,
        elapsed_ms = start.elapsed().as_millis() as u64,
        "synced legacy store → sharded structure (upsert)"
    );
    Ok(())
}

/// 把 legacy Store 拆成 (province → StoreShard) + GlobalShard。
fn split_legacy_store(store: &Store) -> (HashMap<String, StoreShard>, GlobalShard) {
    let mut shards: HashMap<String, StoreShard> = HashMap::new();

    // province_code(2 字符)→ province_name 反查表,
    // 从 multisig_institutions 就地提取(没有独立 helper)。
    let mut code_to_province: HashMap<String, String> = HashMap::new();
    for inst in store.multisig_institutions.values() {
        if !inst.province_code.is_empty() && !inst.province.is_empty() {
            code_to_province
                .entry(inst.province_code.clone())
                .or_insert_with(|| inst.province.clone());
        }
    }

    // 小工具:按需创建分片。
    fn ensure<'a>(
        shards: &'a mut HashMap<String, StoreShard>,
        province: &str,
    ) -> &'a mut StoreShard {
        shards.entry(province.to_string()).or_insert_with(|| {
            let mut s = StoreShard::default();
            s.province = province.to_string();
            s.version = 1;
            s
        })
    }

    // ── 机构:按 institution.province 分散 ──
    for (sfid_id, inst) in &store.multisig_institutions {
        let province = if inst.province.is_empty() {
            "__unknown__".to_string()
        } else {
            inst.province.clone()
        };
        let shard = ensure(&mut shards, &province);
        shard
            .multisig_institutions
            .insert(sfid_id.clone(), inst.clone());
    }

    // ── 机构账户:通过 "sfid_id|account_name" key 的 sfid_id 前缀反查 ──
    for (key, account) in &store.multisig_accounts {
        // 先找 parent institution 的 province
        let parent_province = store
            .multisig_institutions
            .get(&account.sfid_id)
            .map(|i| i.province.clone())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| "__unknown__".to_string());
        let shard = ensure(&mut shards, &parent_province);
        shard.multisig_accounts.insert(key.clone(), account.clone());
    }

    // ── CPMS 站点:按 admin_province 分散 ──
    for (site_sfid, cpms) in &store.cpms_site_keys {
        let province = if cpms.admin_province.is_empty() {
            "__unknown__".to_string()
        } else {
            cpms.admin_province.clone()
        };
        let shard = ensure(&mut shards, &province);
        shard.cpms_site_keys.insert(site_sfid.clone(), cpms.clone());
    }

    // ── citizen 记录:按 province_code → province_name 反查分散 ──
    let mut citizen_province_by_id: HashMap<u64, String> = HashMap::new();
    for (cid, rec) in &store.citizen_records {
        let province = rec
            .province_code
            .as_deref()
            .and_then(|code| code_to_province.get(code).cloned())
            .unwrap_or_else(|| "__unknown__".to_string());
        let shard = ensure(&mut shards, &province);
        shard.citizen_records.insert(*cid, rec.clone());
        citizen_province_by_id.insert(*cid, province);
    }
    // citizen 反向索引:按 citizen_id 对应的省下沉
    for (pubkey, cid) in &store.citizen_id_by_pubkey {
        if let Some(province) = citizen_province_by_id.get(cid) {
            let shard = ensure(&mut shards, province);
            shard.citizen_id_by_pubkey.insert(pubkey.clone(), *cid);
        }
    }
    for (archive_no, cid) in &store.citizen_id_by_archive_no {
        if let Some(province) = citizen_province_by_id.get(cid) {
            let shard = ensure(&mut shards, province);
            shard
                .citizen_id_by_archive_no
                .insert(archive_no.clone(), *cid);
        }
    }
    // 中文注释:legacy pubkey_by_archive_index 已从 Store 删除,不再迁移。

    // ── ShiAdmin:通过 created_by(上级 ShengAdmin)→ sheng_admin_province_by_pubkey 反查 ──
    for (pubkey, admin) in &store.admin_users_by_pubkey {
        if admin.role != AdminRole::ShiAdmin {
            continue;
        }
        let province = store
            .sheng_admin_province_by_pubkey
            .get(&admin.created_by)
            .cloned()
            .unwrap_or_else(|| "__unknown__".to_string());
        let shard = ensure(&mut shards, &province);
        shard.local_admins.insert(pubkey.clone(), admin.clone());
    }

    // ── 档案导入:按 province_code → province 反查 ──
    for (archive_no, archive) in &store.imported_archives {
        let province = code_to_province
            .get(&archive.province_code)
            .cloned()
            .unwrap_or_else(|| "__unknown__".to_string());
        let shard = ensure(&mut shards, &province);
        shard
            .imported_archives
            .insert(archive_no.clone(), archive.clone());
    }

    // pending_status_by_archive_no:按 archive_no 对应的 imported_archives 记录再查,
    // 查不到就下沉到 __unknown__。
    for (archive_no, status) in &store.pending_status_by_archive_no {
        let province = store
            .imported_archives
            .get(archive_no)
            .and_then(|a| code_to_province.get(&a.province_code).cloned())
            .unwrap_or_else(|| "__unknown__".to_string());
        let shard = ensure(&mut shards, &province);
        shard
            .pending_status_by_archive_no
            .insert(archive_no.clone(), status.clone());
    }

    // generated_sfid_by_pubkey:查 citizen_id_by_pubkey 反查省份
    for (pubkey, sfid) in &store.generated_sfid_by_pubkey {
        let province = store
            .citizen_id_by_pubkey
            .get(pubkey)
            .and_then(|cid| citizen_province_by_id.get(cid).cloned())
            .unwrap_or_else(|| "__unknown__".to_string());
        let shard = ensure(&mut shards, &province);
        shard
            .generated_sfid_by_pubkey
            .insert(pubkey.clone(), sfid.clone());
    }

    // reward_state_by_pubkey:同上
    for (pubkey, reward) in &store.reward_state_by_pubkey {
        let province = store
            .citizen_id_by_pubkey
            .get(pubkey)
            .and_then(|cid| citizen_province_by_id.get(cid).cloned())
            .unwrap_or_else(|| "__unknown__".to_string());
        let shard = ensure(&mut shards, &province);
        shard
            .reward_state_by_pubkey
            .insert(pubkey.clone(), reward.clone());
    }

    // citizen_bind_challenges / pending_bind_scan_by_qr_id:运行期短生命周期,
    // 且没有稳定省份字段,一律进 __unknown__(后续自然过期)。
    if !store.citizen_bind_challenges.is_empty() {
        let shard = ensure(&mut shards, "__unknown__");
        shard.citizen_bind_challenges = store.citizen_bind_challenges.clone();
    }
    if !store.pending_bind_scan_by_qr_id.is_empty() {
        let shard = ensure(&mut shards, "__unknown__");
        shard.pending_bind_scan_by_qr_id = store.pending_bind_scan_by_qr_id.clone();
    }

    // bind_callback_jobs 是 Vec,没有省份字段,全部进 __unknown__。
    if !store.bind_callback_jobs.is_empty() {
        let shard = ensure(&mut shards, "__unknown__");
        shard.bind_callback_jobs = store.bind_callback_jobs.clone();
    }

    // ── GlobalShard ──
    let mut global = GlobalShard::default();
    global.version = 1;
    global.chain_keyring_state = store.chain_keyring_state.clone();
    global.keyring_rotate_challenges = store.keyring_rotate_challenges.clone();
    // KeyAdmin + ShengAdmin 进 global_admins
    for (pubkey, admin) in &store.admin_users_by_pubkey {
        if matches!(admin.role, AdminRole::KeyAdmin | AdminRole::ShengAdmin) {
            global.global_admins.insert(pubkey.clone(), admin.clone());
        }
    }
    global.sheng_admin_province_by_pubkey = store.sheng_admin_province_by_pubkey.clone();
    global.login_challenges = store.login_challenges.clone();
    global.qr_login_results = store.qr_login_results.clone();
    global.admin_sessions = store.admin_sessions.clone();
    global.consumed_qr_ids = store.consumed_qr_ids.clone();
    global.consumed_cpms_register_tokens = store.consumed_cpms_register_tokens.clone();
    global.audit_logs = store.audit_logs.clone();
    global.chain_requests_by_key = store.chain_requests_by_key.clone();
    global.chain_nonce_seen = store.chain_nonce_seen.clone();
    global.anon_rsa_private_key_pem = store.anon_rsa_private_key_pem.clone();
    global.chain_auth_last_cleanup_at = store.chain_auth_last_cleanup_at;
    global.pending_bind_last_cleanup_at = store.pending_bind_last_cleanup_at;
    global.metrics = store.metrics.clone();

    // ── 全局计数器 + 投票缓存 ──
    global.next_seq = store.next_seq;
    global.next_audit_seq = store.next_audit_seq;
    global.next_admin_user_id = store.next_admin_user_id;
    global.vote_verify_cache = store.vote_verify_cache.clone();

    (shards, global)
}
