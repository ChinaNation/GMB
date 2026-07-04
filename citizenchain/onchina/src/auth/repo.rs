//! 管理员结构化表读写。
//!
//! 管理员、登录签名请求、会话和安全动作状态全部以数据库表为唯一持久化真源。

use chrono::{DateTime, Duration, Utc};
use postgres::Client;

use crate::auth::login::{
    AdminInstitutionCandidate, AdminSession, LoginSignRequest, NodeBindingChallenge,
    NodeInstitutionBinding, QrLoginResultRecord,
};
use crate::auth::model::AdminUser;
use crate::auth::security_model::{AdminActionChallenge, AdminSecurityGrant};
use crate::core::db::postgres_error_text;
use crate::Db;

fn admin_from_row(row: &postgres::Row) -> Result<AdminUser, String> {
    let id: i64 = row.get(0);
    Ok(AdminUser {
        id: u64::try_from(id).unwrap_or(0),
        admin_account: row.get(1),
        admin_name: row.get(2),
        institution_code: row.get(3),
        built_in: row.get(4),
        created_by: row.get(5),
        created_at: row.get(6),
        updated_at: row.get(7),
        city_name: row.get(8),
    })
}

fn binding_from_row(row: &postgres::Row) -> Result<NodeInstitutionBinding, String> {
    let binding_id: String = row.get(0);
    let institution_code: String = row.get(2);
    let candidate = AdminInstitutionCandidate {
        candidate_id: row.get(1),
        institution_code: institution_code.clone(),
        admin_level: crate::core::chain_runtime::admin_level_label_for(&institution_code),
        institution_cid_number: row.get(4),
        institution_main_account: row.get(5),
        frg_province_code: row.get(6),
        cid_full_name: row.get(7),
        cid_short_name: row.get(8),
        scope_province_name: row.get(9),
        scope_city_name: row.get(10),
        scope_town_name: row.get(11),
    };
    Ok(NodeInstitutionBinding {
        binding_id,
        candidate,
        bound_admin_pubkey: row.get(12),
        bound_at: row.get(13),
        status: row.get(14),
    })
}

fn strip_hex_prefix_text(value: &str) -> &str {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
}

fn hydrate_binding_candidate_metadata_conn(
    conn: &mut Client,
    binding: &mut NodeInstitutionBinding,
) -> Result<(), String> {
    if binding.candidate.frg_province_code.is_some()
        || binding
            .candidate
            .institution_cid_number
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .is_some()
    {
        return Ok(());
    }
    let Some(main_account) = binding.candidate.institution_main_account.clone() else {
        return Ok(());
    };
    let Some((cid_number, cid_full_name, cid_short_name, province_code, city_code, town_code)) =
        resolve_binding_candidate_metadata_conn(conn, main_account.as_str())?
    else {
        return Ok(());
    };
    let (province_name, city_name, town_name) = crate::cid::china::area_display_names(
        province_code.as_str(),
        Some(city_code.as_str()),
        Some(town_code.as_str()),
    );
    binding.candidate.institution_cid_number = Some(cid_number);
    binding.candidate.cid_full_name = cid_full_name;
    binding.candidate.cid_short_name = cid_short_name;
    binding.candidate.scope_province_name = (!province_name.is_empty()).then_some(province_name);
    binding.candidate.scope_city_name = (!city_name.is_empty()).then_some(city_name);
    binding.candidate.scope_town_name = (!town_name.is_empty()).then_some(town_name);
    Ok(())
}

// Tier1 创世注册局管理员「全走链读」(决策③):权威集合在链上
// `PublicAdmins::FederalRegistryProvinceGroups[省码]`,由链上读取并回填本地缓存;
// 省维度由链上省级组和节点 active binding 共同派生。

pub(crate) fn get_admin_by_id_and_registry_org_conn(
    conn: &mut Client,
    id: u64,
    institution_code: &str,
) -> Result<Option<AdminUser>, String> {
    let id = id as i64;
    let row = conn
        .query_opt(
            "SELECT admin_id, admin_account, admin_name, institution_code, built_in, created_by, created_at, updated_at, city_name
             FROM admins
             WHERE admin_id = $1 AND institution_code = $2",
            &[&id, &institution_code],
        )
        .map_err(|e| format!("query admin by id and institution_code failed: {e}"))?;
    row.as_ref().map(admin_from_row).transpose()
}

/// Tier2 下级注册局(CREG)管理员列表。
///
/// 每节点单省部署,本地 `admins` 缓存即本省数据,列表仅按机构码 + 可选市名过滤。
pub(crate) fn list_city_registry_admins_by_scope_conn(
    conn: &mut Client,
    city_name: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<(usize, Vec<AdminUser>), String> {
    let limit = i64::try_from(limit).unwrap_or(500);
    let offset = i64::try_from(offset).unwrap_or(0);
    let code = crate::core::chain_runtime::TIER2_REGISTRY_CODE;
    let (count_row, rows) = if let Some(city_name) = city_name {
        let count_row = conn
            .query_one(
                "SELECT COUNT(*) FROM admins
                 WHERE institution_code = $1 AND city_name = $2",
                &[&code, &city_name],
            )
            .map_err(|e| format!("count city registry admins by city failed: {e}"))?;
        let rows = conn
            .query(
                "SELECT admin_id, admin_account, admin_name, institution_code, built_in, created_by, created_at, updated_at, city_name
                 FROM admins
                 WHERE institution_code = $1 AND city_name = $2
                 ORDER BY admin_id DESC
                 LIMIT $3 OFFSET $4",
                &[&code, &city_name, &limit, &offset],
            )
            .map_err(|e| format!("query city registry admins by city failed: {e}"))?;
        (count_row, rows)
    } else {
        let count_row = conn
            .query_one(
                "SELECT COUNT(*) FROM admins WHERE institution_code = $1",
                &[&code],
            )
            .map_err(|e| format!("count city registry admins failed: {e}"))?;
        let rows = conn
            .query(
                "SELECT admin_id, admin_account, admin_name, institution_code, built_in, created_by, created_at, updated_at, city_name
                 FROM admins
                 WHERE institution_code = $1
                 ORDER BY admin_id DESC
                 LIMIT $2 OFFSET $3",
                &[&code, &limit, &offset],
            )
            .map_err(|e| format!("query city registry admins failed: {e}"))?;
        (count_row, rows)
    };
    let total: i64 = count_row.get(0);
    Ok((
        usize::try_from(total).unwrap_or(0),
        rows.iter()
            .map(admin_from_row)
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

pub(crate) fn count_city_registry_admins_by_city_conn(
    conn: &mut Client,
    city_name: &str,
) -> Result<usize, String> {
    let code = crate::core::chain_runtime::TIER2_REGISTRY_CODE;
    let row = conn
        .query_one(
            "SELECT COUNT(*) FROM admins WHERE institution_code = $1 AND city_name = $2",
            &[&code, &city_name],
        )
        .map_err(|e| format!("count city registry admins by city failed: {e}"))?;
    let count: i64 = row.get(0);
    Ok(usize::try_from(count).unwrap_or(0))
}

pub(crate) fn list_city_registry_admins_by_creator_conn(
    conn: &mut Client,
    creator_account: &str,
) -> Result<Vec<AdminUser>, String> {
    let code = crate::core::chain_runtime::TIER2_REGISTRY_CODE;
    let rows = conn
        .query(
            "SELECT admin_id, admin_account, admin_name, institution_code, built_in, created_by, created_at, updated_at, city_name
             FROM admins
             WHERE institution_code = $1 AND lower(created_by) = lower($2)
             ORDER BY admin_id ASC",
            &[&code, &creator_account],
        )
        .map_err(|e| format!("query city registry admins by creator failed: {e}"))?;
    rows.iter().map(admin_from_row).collect()
}

pub(crate) fn get_admin_by_account(
    db: &Db,
    admin_account: &str,
) -> Result<Option<AdminUser>, String> {
    let admin_account = admin_account.trim().to_string();
    db.with_client(move |conn| get_admin_by_account_conn(conn, admin_account.as_str()))
}

pub(crate) fn get_admin_by_account_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<Option<AdminUser>, String> {
    let row = conn
        .query_opt(
            "SELECT admin_id, admin_account, admin_name, institution_code, built_in, created_by, created_at, updated_at, city_name
             FROM admins
             WHERE lower(admin_account) = lower($1)",
            &[&admin_account],
        )
        .map_err(|e| format!("query admin by account failed: {e}"))?;
    row.as_ref().map(admin_from_row).transpose()
}

pub(crate) fn resolve_admin_account_key_conn(
    conn: &mut Client,
    candidate: &str,
) -> Result<Option<String>, String> {
    let row = conn
        .query_opt(
            "SELECT admin_account FROM admins WHERE lower(admin_account) = lower($1)",
            &[&candidate],
        )
        .map_err(|e| format!("query admin_account key failed: {e}"))?;
    Ok(row.map(|r| r.get(0)))
}

/// Tier1/Tier2 注册局管理员的省作用域。
///
/// 节点机构身份由首次链上 active admin 登录后绑定,省作用域取 active 绑定,
/// 不再读取节点 `ONCHAIN_CREDENTIAL_SCOPE_*` 环境变量。
pub(crate) fn province_scope_for_registry_org_conn(
    conn: &mut Client,
    admin_account: &str,
    institution_code: &str,
) -> Result<Option<String>, String> {
    if crate::core::chain_runtime::is_tier1_registry(institution_code) {
        if let Some(province_name) =
            federal_registry_admin_scope_conn(conn, admin_account)?.filter(|v| !v.trim().is_empty())
        {
            return Ok(Some(province_name));
        }
    }
    Ok(get_active_node_binding_conn(conn)?
        .and_then(|binding| binding.candidate.scope_province_name))
}

/// 写入联邦注册局管理员的链上省级组归属缓存。
///
/// 该缓存由 `FederalRegistryProvinceGroups[ProvinceCode]` 全量链读派生,只供列表显示
/// 和同省操作预检使用;管理员成员资格仍以链上 Active 集合为唯一真源。
pub(crate) fn upsert_federal_registry_admin_scope_conn(
    conn: &mut Client,
    admin_account: &str,
    province_name: &str,
) -> Result<(), String> {
    let admin_account = admin_account.trim();
    let province_name = province_name.trim();
    if admin_account.is_empty() || province_name.is_empty() {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO federal_registry_admin_scopes(admin_account, province_name, updated_at)
         VALUES ($1, $2, now())
         ON CONFLICT (admin_account) DO UPDATE SET
            province_name = EXCLUDED.province_name,
            updated_at = EXCLUDED.updated_at",
        &[&admin_account, &province_name],
    )
    .map_err(|e| format!("upsert federal registry admin scope failed: {e}"))?;
    Ok(())
}

pub(crate) fn federal_registry_admin_scope_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<Option<String>, String> {
    let row = conn
        .query_opt(
            "SELECT province_name
             FROM federal_registry_admin_scopes
             WHERE lower(admin_account) = lower($1)",
            &[&admin_account],
        )
        .map_err(|e| format!("query federal registry admin scope failed: {e}"))?;
    Ok(row.map(|r| r.get(0)))
}

pub(crate) fn replace_federal_registry_admin_scope_conn(
    conn: &mut Client,
    old_account: &str,
    new_account: &str,
    province_name: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM federal_registry_admin_scopes WHERE lower(admin_account) = lower($1)",
        &[&old_account],
    )
    .map_err(|e| format!("delete old federal registry admin scope failed: {e}"))?;
    upsert_federal_registry_admin_scope_conn(conn, new_account, province_name)
}

/// 派生管理员的省/市/镇作用域。**登录签发(onchain_gate)与会话重建(guards)共用此唯一来源**,
/// 保证两路口径逐字段一致(避免 login 与后续请求 scope 漂移)。维度按机构行政层级裁剪:
/// - 省/市/镇:统一取本节点 active 绑定,绑定来自链上 active admin 反查后的二次确认。
pub(crate) fn derive_admin_scope_conn(
    conn: &mut Client,
    _admin_account: &str,
    _institution_code: &str,
) -> Result<(Option<String>, Option<String>, Option<String>), String> {
    let Some(binding) = get_active_node_binding_conn(conn)? else {
        return Ok((None, None, None));
    };
    Ok((
        binding.candidate.scope_province_name,
        binding.candidate.scope_city_name,
        binding.candidate.scope_town_name,
    ))
}

/// 解析当前管理员所属机构的 cid_short_name 单一字段。
/// 联邦注册局管理员 → institution_code='FRG' 的全局唯一机构(总统府联邦注册局,简称=联邦注册局);
/// 市注册局管理员   → institution_code='CREG' AND province_name AND city_name 的本市机构(如 合肥市注册局)。
/// 无对应行返回 None(前端按空处理,绝不另造名字)。
pub(crate) fn resolve_home_cid_short_name_conn(
    conn: &mut Client,
    institution_code: &str,
    scope_province_name: Option<&str>,
    scope_city_name: Option<&str>,
) -> Result<Option<String>, String> {
    let row = if crate::core::chain_runtime::is_tier1_registry(institution_code) {
        // Tier1 创世注册局全局唯一,直接按机构码取其机构简称。
        conn.query_opt(
            "SELECT cid_short_name FROM subjects \
             WHERE institution_code = $1 AND status = 'ACTIVE' LIMIT 1",
            &[&institution_code],
        )
        .map_err(|e| format!("query federal registry short name failed: {e}"))?
    } else if crate::core::chain_runtime::admin_level_label_for(institution_code).as_deref()
        == Some("NATIONAL")
    {
        // NJD 等全国级机构没有省市作用域,按机构码直接解析本机构简称。
        conn.query_opt(
            "SELECT cid_short_name FROM subjects \
             WHERE institution_code = $1 AND status = 'ACTIVE' LIMIT 1",
            &[&institution_code],
        )
        .map_err(|e| format!("query national institution short name failed: {e}"))?
    } else {
        // 市级机构按本机构码 + 省 + 市定位机构简称。
        // subjects 已不存行政区名字,按 china.sqlite 把省/市名字派生成 code 再过滤(单源)。
        let (Some(province), Some(city)) = (scope_province_name, scope_city_name) else {
            return Ok(None);
        };
        let (Some(province_code), Some(city_code)) = (
            crate::cid::china::province_code_by_name(province),
            crate::cid::china::city_code_by_name(province, city),
        ) else {
            return Ok(None);
        };
        conn.query_opt(
            "SELECT cid_short_name FROM subjects \
             WHERE institution_code = $1 AND status = 'ACTIVE' \
               AND province_code = $2 AND city_code = $3 LIMIT 1",
            &[&institution_code, &province_code, &city_code],
        )
        .map_err(|e| format!("query institution short name failed: {e}"))?
    };
    Ok(row.and_then(|r| r.get::<usize, Option<String>>(0)))
}

pub(crate) fn resolve_home_cid_short_name(
    db: &Db,
    institution_code: &str,
    scope_province_name: Option<&str>,
    scope_city_name: Option<&str>,
) -> Result<Option<String>, String> {
    let institution_code = institution_code.to_string();
    let province = scope_province_name.map(str::to_string);
    let city = scope_city_name.map(str::to_string);
    db.with_client(move |conn| {
        resolve_home_cid_short_name_conn(
            conn,
            institution_code.as_str(),
            province.as_deref(),
            city.as_deref(),
        )
    })
}

pub(crate) fn resolve_binding_candidate_metadata_conn(
    conn: &mut Client,
    institution_main_account: &str,
) -> Result<
    Option<(
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        String,
    )>,
    String,
> {
    let normalized_main_account =
        strip_hex_prefix_text(institution_main_account).to_ascii_lowercase();
    let row = conn
        .query_opt(
            "SELECT s.cid_number, s.cid_full_name, s.cid_short_name,
                    s.province_code, COALESCE(s.city_code, ''), COALESCE(s.town_code, '')
             FROM accounts a
             JOIN subjects s
               ON s.province_code = a.province_code
              AND s.cid_number = a.cid_number
             WHERE lower(regexp_replace(a.account, '^0x', '', 'i')) = $1
               AND s.status = 'ACTIVE'
             ORDER BY s.updated_at DESC
             LIMIT 1",
            &[&normalized_main_account],
        )
        .map_err(|e| {
            format!(
                "query binding candidate metadata failed: {}",
                postgres_error_text(&e)
            )
        })?;
    Ok(row.map(|r| (r.get(0), r.get(1), r.get(2), r.get(3), r.get(4), r.get(5))))
}

pub(crate) fn get_active_node_binding_conn(
    conn: &mut Client,
) -> Result<Option<NodeInstitutionBinding>, String> {
    let row = conn
        .query_opt(
            "SELECT binding_id, candidate_id, institution_code, NULL::TEXT AS admin_level,
                    institution_cid_number, institution_main_account, frg_province_code,
                    cid_full_name, cid_short_name, scope_province_name, scope_city_name,
                    scope_town_name, bound_admin_pubkey, bound_at, status
             FROM node_institution_bindings
             WHERE status = 'ACTIVE'
             ORDER BY bound_at DESC
             LIMIT 1",
            &[],
        )
        .map_err(|e| {
            format!(
                "query active node binding failed: {}",
                postgres_error_text(&e)
            )
        })?;
    let Some(row) = row else {
        return Ok(None);
    };
    let mut binding = binding_from_row(&row)?;
    // 早期绑定行可能因账号 0x 前缀不一致缺少 CID 元数据;读取时补齐可让旧绑定继续按链上真源工作。
    hydrate_binding_candidate_metadata_conn(conn, &mut binding)?;
    Ok(Some(binding))
}

pub(crate) fn active_node_binding(db: &Db) -> Result<Option<NodeInstitutionBinding>, String> {
    db.with_client(get_active_node_binding_conn)
}

pub(crate) fn upsert_active_node_binding_conn(
    conn: &mut Client,
    binding: &NodeInstitutionBinding,
) -> Result<(), String> {
    conn.execute(
        "UPDATE node_institution_bindings
         SET status = 'INACTIVE'
         WHERE status = 'ACTIVE'",
        &[],
    )
    .map_err(|e| {
        format!(
            "deactivate old node binding failed: {}",
            postgres_error_text(&e)
        )
    })?;
    conn.execute(
        "INSERT INTO node_institution_bindings (
            binding_id, candidate_id, institution_code, institution_cid_number,
            institution_main_account, frg_province_code, cid_full_name, cid_short_name,
            scope_province_name, scope_city_name, scope_town_name, bound_admin_pubkey,
            bound_at, status
         )
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        &[
            &binding.binding_id,
            &binding.candidate.candidate_id,
            &binding.candidate.institution_code,
            &binding.candidate.institution_cid_number,
            &binding.candidate.institution_main_account,
            &binding.candidate.frg_province_code,
            &binding.candidate.cid_full_name,
            &binding.candidate.cid_short_name,
            &binding.candidate.scope_province_name,
            &binding.candidate.scope_city_name,
            &binding.candidate.scope_town_name,
            &binding.bound_admin_pubkey,
            &binding.bound_at,
            &binding.status,
        ],
    )
    .map_err(|e| format!("insert node binding failed: {}", postgres_error_text(&e)))?;
    Ok(())
}

pub(crate) fn deactivate_active_node_binding_conn(conn: &mut Client) -> Result<u64, String> {
    let changed = conn
        .execute(
            "UPDATE node_institution_bindings
             SET status = 'INACTIVE'
             WHERE status = 'ACTIVE'",
            &[],
        )
        .map_err(|e| {
            format!(
                "deactivate active node binding failed: {}",
                postgres_error_text(&e)
            )
        })?;
    Ok(changed)
}

pub(crate) fn insert_node_binding_challenge_conn(
    conn: &mut Client,
    challenge: &NodeBindingChallenge,
) -> Result<(), String> {
    let payload = serde_json::to_value(challenge)
        .map_err(|e| format!("encode node binding challenge failed: {e}"))?;
    conn.execute(
        "INSERT INTO node_binding_challenges(
            binding_challenge_id, admin_account, expires_at, consumed, payload
         )
         VALUES ($1,$2,$3,$4,$5)
         ON CONFLICT (binding_challenge_id) DO UPDATE SET
            admin_account = EXCLUDED.admin_account,
            expires_at = EXCLUDED.expires_at,
            consumed = EXCLUDED.consumed,
            payload = EXCLUDED.payload",
        &[
            &challenge.binding_challenge_id,
            &challenge.admin_account,
            &challenge.expire_at,
            &challenge.consumed,
            &payload,
        ],
    )
    .map_err(|e| {
        format!(
            "insert node binding challenge failed: {}",
            postgres_error_text(&e)
        )
    })?;
    Ok(())
}

pub(crate) fn get_node_binding_challenge_conn(
    conn: &mut Client,
    binding_challenge_id: &str,
) -> Result<Option<NodeBindingChallenge>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM node_binding_challenges WHERE binding_challenge_id = $1",
            &[&binding_challenge_id],
        )
        .map_err(|e| {
            format!(
                "query node binding challenge failed: {}",
                postgres_error_text(&e)
            )
        })?;
    row.map(|r| serde_json::from_value::<NodeBindingChallenge>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode node binding challenge failed: {e}"))
}

pub(crate) fn consume_node_binding_challenge_conn(
    conn: &mut Client,
    challenge: &NodeBindingChallenge,
) -> Result<(), String> {
    let mut consumed = challenge.clone();
    consumed.consumed = true;
    let payload = serde_json::to_value(&consumed)
        .map_err(|e| format!("encode consumed node binding challenge failed: {e}"))?;
    conn.execute(
        "UPDATE node_binding_challenges
         SET consumed = true, payload = $2
         WHERE binding_challenge_id = $1",
        &[&challenge.binding_challenge_id, &payload],
    )
    .map_err(|e| {
        format!(
            "consume node binding challenge failed: {}",
            postgres_error_text(&e)
        )
    })?;
    Ok(())
}

pub(crate) fn next_admin_id_conn(conn: &mut Client) -> Result<u64, String> {
    let row = conn
        .query_one("SELECT COALESCE(MAX(admin_id), 0) + 1 FROM admins", &[])
        .map_err(|e| format!("allocate admin id failed: {e}"))?;
    let id: i64 = row.get(0);
    Ok(u64::try_from(id).unwrap_or(1))
}

/// 写入 / 更新本地管理员缓存行(`admin_account` 为冲突键,幂等)。
///
/// 管理员成员资格与节点机构归属以链上 active 集合 + 本节点 active binding 为准。
/// 本函数只维护 `admins` 登录元数据缓存本身。
pub(crate) fn upsert_admin_conn(conn: &mut Client, admin: &AdminUser) -> Result<(), String> {
    conn.execute(
        "INSERT INTO admins(admin_id, admin_account, admin_name, institution_code, built_in, created_by, created_at, updated_at, city_name)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (admin_account) DO UPDATE SET
            admin_name = CASE
                WHEN trim(EXCLUDED.admin_name) <> '' THEN EXCLUDED.admin_name
                ELSE admins.admin_name
            END,
            institution_code = EXCLUDED.institution_code,
            built_in = EXCLUDED.built_in,
            created_by = EXCLUDED.created_by,
            updated_at = EXCLUDED.updated_at,
            city_name = EXCLUDED.city_name",
        &[
            &(admin.id as i64),
            &admin.admin_account,
            &admin.admin_name,
            &admin.institution_code,
            &admin.built_in,
            &admin.created_by,
            &admin.created_at,
            &admin.updated_at,
            &admin.city_name,
        ],
    )
    .map_err(|e| format!("upsert admin failed: {e}"))?;
    Ok(())
}

pub(crate) fn delete_admin_runtime_state_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM admin_sessions WHERE lower(admin_account) = lower($1)",
        &[&admin_account],
    )
    .map_err(|e| format!("delete admin sessions failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_action_challenges WHERE lower(actor_account) = lower($1)",
        &[&admin_account],
    )
    .map_err(|e| format!("delete admin action challenges failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_security_grants WHERE lower(actor_account) = lower($1)",
        &[&admin_account],
    )
    .map_err(|e| format!("delete admin security grants failed: {e}"))?;
    Ok(())
}

pub(crate) fn delete_all_admin_sessions_conn(conn: &mut Client) -> Result<u64, String> {
    conn.execute("DELETE FROM admin_sessions", &[])
        .map_err(|e| format!("delete all admin sessions failed: {e}"))
}

pub(crate) fn cleanup_security_state_conn(
    conn: &mut Client,
    now: DateTime<Utc>,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM admin_action_challenges
         WHERE consumed = true OR expires_at < $1",
        &[&now],
    )
    .map_err(|e| format!("cleanup admin action challenges failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_security_grants
         WHERE consumed = true OR expires_at < $1",
        &[&now],
    )
    .map_err(|e| format!("cleanup admin security grants failed: {e}"))?;
    Ok(())
}

pub(crate) fn insert_action_challenge(
    db: &Db,
    challenge: &AdminActionChallenge,
) -> Result<(), String> {
    let challenge = challenge.clone();
    db.with_client(move |conn| {
        cleanup_security_state_conn(conn, Utc::now())?;
        upsert_action_challenge_conn(conn, &challenge)
    })
}

/// 按 action_id 取挑战,同时以 actor_account 做先验隔离——只能读自己发起的挑战。
/// 非本人 action_id 直接当作不存在(返回 None),不在 DB 层暴露他人挑战。
pub(crate) fn get_action_challenge_conn(
    conn: &mut Client,
    action_id: &str,
    actor_account: &str,
) -> Result<Option<AdminActionChallenge>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_action_challenges
             WHERE action_id = $1 AND lower(actor_account) = lower($2)",
            &[&action_id, &actor_account],
        )
        .map_err(|e| format!("query action challenge failed: {e}"))?;
    row.map(|r| serde_json::from_value::<AdminActionChallenge>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode action challenge failed: {e}"))
}

pub(crate) fn upsert_action_challenge_conn(
    conn: &mut Client,
    challenge: &AdminActionChallenge,
) -> Result<(), String> {
    let payload = serde_json::to_value(challenge)
        .map_err(|e| format!("encode action challenge failed: {e}"))?;
    conn.execute(
        "INSERT INTO admin_action_challenges(action_id, actor_account, action_type, expires_at, consumed, payload)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (action_id) DO UPDATE SET
            actor_account = EXCLUDED.actor_account,
            action_type = EXCLUDED.action_type,
            expires_at = EXCLUDED.expires_at,
            consumed = EXCLUDED.consumed,
            payload = EXCLUDED.payload",
        &[
            &challenge.action_id,
            &challenge.actor_account,
            &challenge.action_type,
            &challenge.expires_at,
            &challenge.consumed,
            &payload,
        ],
    )
    .map_err(|e| format!("upsert action challenge failed: {e}"))?;
    Ok(())
}

pub(crate) fn insert_security_grant_conn(
    conn: &mut Client,
    grant: &AdminSecurityGrant,
) -> Result<(), String> {
    let payload =
        serde_json::to_value(grant).map_err(|e| format!("encode security grant failed: {e}"))?;
    conn.execute(
        "INSERT INTO admin_security_grants(grant_id, actor_account, action_type, expires_at, consumed, payload)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (grant_id) DO UPDATE SET
            actor_account = EXCLUDED.actor_account,
            action_type = EXCLUDED.action_type,
            expires_at = EXCLUDED.expires_at,
            consumed = EXCLUDED.consumed,
            payload = EXCLUDED.payload",
        &[
            &grant.grant_id,
            &grant.actor_account,
            &grant.action_type,
            &grant.expires_at,
            &grant.consumed,
            &payload,
        ],
    )
    .map_err(|e| format!("insert security grant failed: {e}"))?;
    Ok(())
}

/// 按 grant_id 取冷签授权,同时以 actor_account 做先验隔离——只能读自己持有的授权。
/// 非本人 grant_id 直接当作不存在(返回 None),不在 DB 层暴露他人授权。
pub(crate) fn get_security_grant_conn(
    conn: &mut Client,
    grant_id: &str,
    actor_account: &str,
) -> Result<Option<AdminSecurityGrant>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_security_grants
             WHERE grant_id = $1 AND lower(actor_account) = lower($2)",
            &[&grant_id, &actor_account],
        )
        .map_err(|e| format!("query security grant failed: {e}"))?;
    row.map(|r| serde_json::from_value::<AdminSecurityGrant>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode security grant failed: {e}"))
}

pub(crate) fn cleanup_login_state_conn(
    conn: &mut Client,
    now: DateTime<Utc>,
) -> Result<(), String> {
    let stale_login_before = now - Duration::minutes(10);
    let consumed_login_before = now;
    conn.execute(
        "DELETE FROM admin_login_sign_requests
         WHERE expires_at < $1
            OR (consumed = true AND expires_at < $2)",
        &[&stale_login_before, &consumed_login_before],
    )
    .map_err(|e| {
        format!(
            "cleanup login sign requests failed: {}",
            postgres_error_text(&e)
        )
    })?;
    let stale_qr_created_before = now - Duration::hours(1);
    let stale_qr_expired_before = now - Duration::minutes(10);
    conn.execute(
        "DELETE FROM admin_qr_login_results
         WHERE created_at < $1
            OR expires_at < $2",
        &[&stale_qr_created_before, &stale_qr_expired_before],
    )
    .map_err(|e| {
        format!(
            "cleanup qr login results failed: {}",
            postgres_error_text(&e)
        )
    })?;
    conn.execute(
        "DELETE FROM node_binding_challenges
         WHERE expires_at < $1
            OR (consumed = true AND expires_at < $2)",
        &[&stale_login_before, &consumed_login_before],
    )
    .map_err(|e| {
        format!(
            "cleanup node binding challenges failed: {}",
            postgres_error_text(&e)
        )
    })?;
    Ok(())
}

pub(crate) fn insert_login_sign_request(
    db: &Db,
    challenge: &LoginSignRequest,
) -> Result<(), String> {
    let challenge = challenge.clone();
    db.with_client(move |conn| {
        cleanup_login_state_conn(conn, Utc::now())?;
        let payload = serde_json::to_value(&challenge)
            .map_err(|e| format!("encode login sign request failed: {e}"))?;
        conn.execute(
            "INSERT INTO admin_login_sign_requests(challenge_id, session_id, admin_account, expires_at, consumed, payload)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (challenge_id) DO UPDATE SET
                session_id = EXCLUDED.session_id,
                admin_account = EXCLUDED.admin_account,
                expires_at = EXCLUDED.expires_at,
                consumed = EXCLUDED.consumed,
                payload = EXCLUDED.payload",
            &[
                &challenge.challenge_id,
                &challenge.session_id,
                &challenge.admin_account,
                &challenge.expire_at,
                &challenge.consumed,
                &payload,
            ],
        )
        .map_err(|e| format!("insert login sign request failed: {}", postgres_error_text(&e)))?;
        Ok(())
    })
}

pub(crate) fn get_login_sign_request_conn(
    conn: &mut Client,
    challenge_id: &str,
) -> Result<Option<LoginSignRequest>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_login_sign_requests WHERE challenge_id = $1",
            &[&challenge_id],
        )
        .map_err(|e| {
            format!(
                "query login sign request failed: {}",
                postgres_error_text(&e)
            )
        })?;
    row.map(|r| serde_json::from_value::<LoginSignRequest>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode login sign request failed: {e}"))
}

pub(crate) fn update_login_sign_request_conn(
    conn: &mut Client,
    challenge: &LoginSignRequest,
) -> Result<(), String> {
    let payload = serde_json::to_value(challenge)
        .map_err(|e| format!("encode login sign request failed: {e}"))?;
    conn.execute(
        "UPDATE admin_login_sign_requests
         SET admin_account = $2, expires_at = $3, consumed = $4, payload = $5
         WHERE challenge_id = $1",
        &[
            &challenge.challenge_id,
            &challenge.admin_account,
            &challenge.expire_at,
            &challenge.consumed,
            &payload,
        ],
    )
    .map_err(|e| {
        format!(
            "update login sign request failed: {}",
            postgres_error_text(&e)
        )
    })?;
    Ok(())
}

pub(crate) fn insert_admin_session_conn(
    conn: &mut Client,
    session: &AdminSession,
) -> Result<(), String> {
    let payload =
        serde_json::to_value(session).map_err(|e| format!("encode admin session failed: {e}"))?;
    conn.execute(
        "INSERT INTO admin_sessions(token, admin_account, institution_code, expires_at, last_active_at, payload)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (token) DO UPDATE SET
            admin_account = EXCLUDED.admin_account,
            institution_code = EXCLUDED.institution_code,
            expires_at = EXCLUDED.expires_at,
            last_active_at = EXCLUDED.last_active_at,
            payload = EXCLUDED.payload",
        &[
            &session.token,
            &session.admin_account,
            &session.institution_code,
            &session.expire_at,
            &session.last_active_at,
            &payload,
        ],
    )
    .map_err(|e| format!("insert admin session failed: {e}"))?;
    Ok(())
}

pub(crate) fn delete_admin_session(db: &Db, token: &str) -> Result<(), String> {
    let token = token.trim().to_string();
    db.with_client(move |conn| {
        conn.execute("DELETE FROM admin_sessions WHERE token = $1", &[&token])
            .map_err(|e| format!("delete admin session failed: {e}"))?;
        Ok(())
    })
}

pub(crate) fn get_admin_session_conn(
    conn: &mut Client,
    token: &str,
) -> Result<Option<AdminSession>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_sessions WHERE token = $1",
            &[&token],
        )
        .map_err(|e| format!("query admin session failed: {e}"))?;
    row.map(|r| serde_json::from_value::<AdminSession>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode admin session failed: {e}"))
}

pub(crate) fn touch_admin_session_conn(
    conn: &mut Client,
    session: &AdminSession,
) -> Result<(), String> {
    insert_admin_session_conn(conn, session)
}

pub(crate) fn cleanup_admin_sessions_conn(
    conn: &mut Client,
    now: DateTime<Utc>,
    city_idle_timeout_minutes: i64,
) -> Result<(), String> {
    conn.execute("DELETE FROM admin_sessions WHERE expires_at < $1", &[&now])
        .map_err(|e| format!("cleanup expired admin sessions failed: {e}"))?;
    let idle_cutoff = now - Duration::minutes(city_idle_timeout_minutes);
    conn.execute(
        "DELETE FROM admin_sessions
         WHERE institution_code = 'CREG' AND last_active_at < $1",
        &[&idle_cutoff],
    )
    .map_err(|e| format!("cleanup idle city admin sessions failed: {e}"))?;
    Ok(())
}

/// 列出某机构当前有会话的去重管理员账户(链上集合复查用)。
pub(crate) fn list_session_admin_accounts_conn(
    conn: &mut Client,
    institution_code: &str,
) -> Result<Vec<String>, String> {
    let rows = conn
        .query(
            "SELECT DISTINCT admin_account FROM admin_sessions WHERE institution_code = $1",
            &[&institution_code],
        )
        .map_err(|e| format!("list session admin accounts failed: {e}"))?;
    Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
}

/// 清退某管理员账户的全部会话,返回删除行数(链上移除后即时失效用)。
pub(crate) fn delete_admin_sessions_for_account_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<u64, String> {
    conn.execute(
        "DELETE FROM admin_sessions WHERE lower(admin_account) = lower($1)",
        &[&admin_account],
    )
    .map_err(|e| format!("delete admin sessions for account failed: {e}"))
}

pub(crate) fn insert_qr_login_result_conn(
    conn: &mut Client,
    challenge_id: &str,
    result: &QrLoginResultRecord,
) -> Result<(), String> {
    let payload =
        serde_json::to_value(result).map_err(|e| format!("encode qr login result failed: {e}"))?;
    conn.execute(
        "INSERT INTO admin_qr_login_results(challenge_id, session_id, access_token, expires_at, payload, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (challenge_id) DO UPDATE SET
            session_id = EXCLUDED.session_id,
            access_token = EXCLUDED.access_token,
            expires_at = EXCLUDED.expires_at,
            payload = EXCLUDED.payload,
            created_at = EXCLUDED.created_at",
        &[
            &challenge_id,
            &result.session_id,
            &result.access_token,
            &result.expire_at,
            &payload,
            &result.created_at,
        ],
    )
    .map_err(|e| format!("insert qr login result failed: {e}"))?;
    Ok(())
}

pub(crate) fn get_qr_login_result_conn(
    conn: &mut Client,
    challenge_id: &str,
) -> Result<Option<QrLoginResultRecord>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_qr_login_results WHERE challenge_id = $1",
            &[&challenge_id],
        )
        .map_err(|e| format!("query qr login result failed: {e}"))?;
    row.map(|r| serde_json::from_value::<QrLoginResultRecord>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode qr login result failed: {e}"))
}

/// 已签发注销凭证行(下发 /deregistration-info 用)。
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct DeregistrationCredentialRow {
    pub(crate) scope: i16,
    pub(crate) account_name: String,
    pub(crate) target_account: String,
    pub(crate) deregister_nonce: String,
    pub(crate) signature: Option<String>,
    pub(crate) issuer_cid_number: String,
    pub(crate) issuer_main_account: String,
    pub(crate) signer_pubkey: String,
}

/// 写入注册局域注销态(ISSUED,signature 待 commit 层回填)。
/// 同账户已有活跃 ISSUED(唯一索引)或 nonce 重复时返回 conflict。
#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_deregistration_issued_conn(
    conn: &mut Client,
    cid_number: &str,
    account_name: &str,
    scope: u8,
    target_account: &str,
    deregister_nonce: &str,
    issuer_cid_number: &str,
    issuer_main_account: &str,
    signer_pubkey: &str,
    issued_by: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO institution_deregistrations
            (cid_number, account_name, scope, target_account, deregister_nonce,
             issuer_cid_number, issuer_main_account, signer_pubkey, issued_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &[
            &cid_number,
            &account_name,
            &(scope as i16),
            &target_account,
            &deregister_nonce,
            &issuer_cid_number,
            &issuer_main_account,
            &signer_pubkey,
            &issued_by,
        ],
    )
    .map_err(|e| {
        if e.code() == Some(&postgres::error::SqlState::UNIQUE_VIOLATION) {
            "http:conflict:deregistration already pending for this account".to_string()
        } else {
            format!("insert deregistration failed: {}", postgres_error_text(&e))
        }
    })?;
    Ok(())
}

/// commit 层签发成功后回填签名 + issuer(issuer 来自 env runtime_signing_context,
/// 与签名同源,下发时直读)。
pub(crate) fn set_deregistration_credential_conn(
    conn: &mut Client,
    deregister_nonce: &str,
    signature: &str,
    issuer_cid_number: &str,
    issuer_main_account: &str,
    signer_pubkey: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE institution_deregistrations
            SET signature = $2, issuer_cid_number = $3,
                issuer_main_account = $4, signer_pubkey = $5
         WHERE deregister_nonce = $1 AND status = 'ISSUED'",
        &[
            &deregister_nonce,
            &signature,
            &issuer_cid_number,
            &issuer_main_account,
            &signer_pubkey,
        ],
    )
    .map_err(|e| {
        format!(
            "set deregistration credential failed: {}",
            postgres_error_text(&e)
        )
    })?;
    Ok(())
}

/// 签发失败时清掉无签名的 ISSUED 行,保持一致(不留无签名残行)。
pub(crate) fn delete_deregistration_by_nonce_conn(
    conn: &mut Client,
    deregister_nonce: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM institution_deregistrations WHERE deregister_nonce = $1",
        &[&deregister_nonce],
    )
    .map_err(|e| format!("delete deregistration failed: {}", postgres_error_text(&e)))?;
    Ok(())
}

/// 取该机构当前已签发(ISSUED 且已回填签名)的注销凭证,供机构管理员下发。
pub(crate) fn get_active_deregistration_by_cid_conn(
    conn: &mut Client,
    cid_number: &str,
) -> Result<Option<DeregistrationCredentialRow>, String> {
    let row = conn
        .query_opt(
            "SELECT scope, account_name, target_account, deregister_nonce, signature,
                    issuer_cid_number, issuer_main_account, signer_pubkey
             FROM institution_deregistrations
             WHERE cid_number = $1 AND status = 'ISSUED' AND signature IS NOT NULL
             ORDER BY issued_at DESC
             LIMIT 1",
            &[&cid_number],
        )
        .map_err(|e| format!("query deregistration failed: {}", postgres_error_text(&e)))?;
    Ok(row.map(|r| DeregistrationCredentialRow {
        scope: r.get(0),
        account_name: r.get(1),
        target_account: r.get(2),
        deregister_nonce: r.get(3),
        signature: r.get(4),
        issuer_cid_number: r.get(5),
        issuer_main_account: r.get(6),
        signer_pubkey: r.get(7),
    }))
}
