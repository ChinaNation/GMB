//! 中文注释:管理员结构化表读写。
//!
//! 管理员、登录签名请求、会话和 Passkey 状态全部以数据库表为唯一持久化真源。

use chrono::{DateTime, Duration, Utc};
use postgres::Client;

use crate::admins::login::{AdminSession, LoginSignRequest, QrLoginResultRecord};
use crate::admins::model::{AdminUser, RegistryOrgCode};
use crate::admins::security_model::{
    AdminActionChallenge, AdminPasskeyCredential, AdminPasskeyRegistrationChallenge,
    AdminPasskeyStatus, AdminSecurityGrant,
};
use crate::core::db::postgres_error_text;
use crate::crypto::pubkey::same_admin_account;
use crate::Db;

pub(crate) fn registry_org_code_text(registry_org_code: &RegistryOrgCode) -> &'static str {
    match registry_org_code {
        RegistryOrgCode::FederalRegistry => "FEDERAL_REGISTRY",
        RegistryOrgCode::CityRegistry => "CITY_REGISTRY",
    }
}

pub(crate) fn parse_registry_org_code(registry_org_code: &str) -> Result<RegistryOrgCode, String> {
    match registry_org_code {
        "FEDERAL_REGISTRY" => Ok(RegistryOrgCode::FederalRegistry),
        "CITY_REGISTRY" => Ok(RegistryOrgCode::CityRegistry),
        _ => Err(format!(
            "invalid admin registry_org_code in database: {registry_org_code}"
        )),
    }
}

fn admin_from_row(row: &postgres::Row) -> Result<AdminUser, String> {
    let id: i64 = row.get(0);
    let role_text: String = row.get(3);
    Ok(AdminUser {
        id: u64::try_from(id).unwrap_or(0),
        admin_account: row.get(1),
        admin_name: row.get(2),
        registry_org_code: parse_registry_org_code(role_text.as_str())?,
        built_in: row.get(4),
        created_by: row.get(5),
        created_at: row.get(6),
        updated_at: row.get(7),
        city_name: row.get(8),
    })
}

pub(crate) fn list_federal_registry_admins_by_province_conn(
    conn: &mut Client,
    province_name: Option<&str>,
) -> Result<Vec<(AdminUser, String)>, String> {
    let rows = if let Some(province_name) = province_name {
        conn.query(
            "SELECT a.admin_id, a.admin_account, a.admin_name, a.registry_org_code, a.built_in, a.created_by, a.created_at, a.updated_at, a.city_name,
                    s.province_name
             FROM admins a
             JOIN federal_registry_scope s ON s.admin_id = a.admin_id
             WHERE a.registry_org_code = 'FEDERAL_REGISTRY' AND s.province_name = $1
             ORDER BY s.province_name ASC, a.built_in DESC, a.admin_id ASC",
            &[&province_name],
        )
    } else {
        conn.query(
            "SELECT a.admin_id, a.admin_account, a.admin_name, a.registry_org_code, a.built_in, a.created_by, a.created_at, a.updated_at, a.city_name,
                    s.province_name
             FROM admins a
             JOIN federal_registry_scope s ON s.admin_id = a.admin_id
             WHERE a.registry_org_code = 'FEDERAL_REGISTRY'
             ORDER BY s.province_name ASC, a.built_in DESC, a.admin_id ASC",
            &[],
        )
    }
    .map_err(|e| format!("query federal registry admins by province failed: {e}"))?;
    rows.iter()
        .map(|row| {
            let admin = admin_from_row(row)?;
            let province_name: String = row.get(9);
            Ok((admin, province_name))
        })
        .collect()
}

pub(crate) fn count_federal_registry_admins_by_province_conn(
    conn: &mut Client,
    province_name: &str,
) -> Result<usize, String> {
    let row = conn
        .query_one(
            "SELECT COUNT(*)
             FROM admins a
             JOIN federal_registry_scope s ON s.admin_id = a.admin_id
             WHERE a.registry_org_code = 'FEDERAL_REGISTRY' AND s.province_name = $1",
            &[&province_name],
        )
        .map_err(|e| format!("count federal registry admins by province failed: {e}"))?;
    let count: i64 = row.get(0);
    Ok(usize::try_from(count).unwrap_or(0))
}

pub(crate) fn get_admin_by_id_and_registry_org_conn(
    conn: &mut Client,
    id: u64,
    registry_org_code: &RegistryOrgCode,
) -> Result<Option<AdminUser>, String> {
    let id = id as i64;
    let registry_org_code = registry_org_code_text(registry_org_code);
    let row = conn
        .query_opt(
            "SELECT admin_id, admin_account, admin_name, registry_org_code, built_in, created_by, created_at, updated_at, city_name
             FROM admins
             WHERE admin_id = $1 AND registry_org_code = $2",
            &[&id, &registry_org_code],
        )
        .map_err(|e| format!("query admin by id and registry_org_code failed: {e}"))?;
    row.as_ref().map(admin_from_row).transpose()
}

pub(crate) fn list_city_registry_admins_by_scope_conn(
    conn: &mut Client,
    province_name: &str,
    city_name: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<(usize, Vec<AdminUser>), String> {
    let limit = i64::try_from(limit).unwrap_or(500);
    let offset = i64::try_from(offset).unwrap_or(0);
    let (count_row, rows) = if let Some(city_name) = city_name {
        let count_row = conn
            .query_one(
                "SELECT COUNT(*)
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_account) = lower(a.created_by)
                 JOIN federal_registry_scope s ON s.admin_id = creator.admin_id
                 WHERE a.registry_org_code = 'CITY_REGISTRY' AND s.province_name = $1 AND a.city_name = $2",
                &[&province_name, &city_name],
            )
            .map_err(|e| format!("count city registry admins by city failed: {e}"))?;
        let rows = conn
            .query(
                "SELECT a.admin_id, a.admin_account, a.admin_name, a.registry_org_code, a.built_in, a.created_by, a.created_at, a.updated_at, a.city_name
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_account) = lower(a.created_by)
                 JOIN federal_registry_scope s ON s.admin_id = creator.admin_id
                 WHERE a.registry_org_code = 'CITY_REGISTRY' AND s.province_name = $1 AND a.city_name = $2
                 ORDER BY a.admin_id DESC
                 LIMIT $3 OFFSET $4",
                &[&province_name, &city_name, &limit, &offset],
            )
            .map_err(|e| format!("query city registry admins by city failed: {e}"))?;
        (count_row, rows)
    } else {
        let count_row = conn
            .query_one(
                "SELECT COUNT(*)
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_account) = lower(a.created_by)
                 JOIN federal_registry_scope s ON s.admin_id = creator.admin_id
                 WHERE a.registry_org_code = 'CITY_REGISTRY' AND s.province_name = $1",
                &[&province_name],
            )
            .map_err(|e| format!("count city registry admins by province failed: {e}"))?;
        let rows = conn
            .query(
                "SELECT a.admin_id, a.admin_account, a.admin_name, a.registry_org_code, a.built_in, a.created_by, a.created_at, a.updated_at, a.city_name
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_account) = lower(a.created_by)
                 JOIN federal_registry_scope s ON s.admin_id = creator.admin_id
                 WHERE a.registry_org_code = 'CITY_REGISTRY' AND s.province_name = $1
                 ORDER BY a.admin_id DESC
                 LIMIT $2 OFFSET $3",
                &[&province_name, &limit, &offset],
            )
            .map_err(|e| format!("query city registry admins by province failed: {e}"))?;
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
    province_name: &str,
    city_name: &str,
) -> Result<usize, String> {
    let row = conn
        .query_one(
            "SELECT COUNT(*)
             FROM admins a
             JOIN admins creator ON lower(creator.admin_account) = lower(a.created_by)
             JOIN federal_registry_scope s ON s.admin_id = creator.admin_id
             WHERE a.registry_org_code = 'CITY_REGISTRY' AND s.province_name = $1 AND a.city_name = $2",
            &[&province_name, &city_name],
        )
        .map_err(|e| format!("count city registry admins by city failed: {e}"))?;
    let count: i64 = row.get(0);
    Ok(usize::try_from(count).unwrap_or(0))
}

pub(crate) fn list_city_registry_admins_by_creator_conn(
    conn: &mut Client,
    creator_account: &str,
) -> Result<Vec<AdminUser>, String> {
    let rows = conn
        .query(
            "SELECT admin_id, admin_account, admin_name, registry_org_code, built_in, created_by, created_at, updated_at, city_name
             FROM admins
             WHERE registry_org_code = 'CITY_REGISTRY' AND lower(created_by) = lower($1)
             ORDER BY admin_id ASC",
            &[&creator_account],
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
            "SELECT admin_id, admin_account, admin_name, registry_org_code, built_in, created_by, created_at, updated_at, city_name
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

pub(crate) fn province_scope_for_registry_org(
    db: &Db,
    admin_account: &str,
    registry_org_code: &RegistryOrgCode,
) -> Result<Option<String>, String> {
    let admin_account = admin_account.trim().to_string();
    let registry_org_code = registry_org_code.clone();
    db.with_client(move |conn| {
        province_scope_for_registry_org_conn(conn, admin_account.as_str(), &registry_org_code)
    })
}

pub(crate) fn province_scope_for_registry_org_conn(
    conn: &mut Client,
    admin_account: &str,
    registry_org_code: &RegistryOrgCode,
) -> Result<Option<String>, String> {
    match registry_org_code {
        RegistryOrgCode::FederalRegistry => find_federal_registry_scope_conn(conn, admin_account),
        RegistryOrgCode::CityRegistry => {
            let Some(admin) = get_admin_by_account_conn(conn, admin_account)? else {
                return Ok(None);
            };
            find_federal_registry_scope_conn(conn, admin.created_by.as_str())
        }
    }
}

/// 中文注释:解析当前管理员所属机构的 cid_short_name 单一字段。
/// 联邦注册局管理员 → institution_code='FRG' 的全局唯一机构(总统府联邦注册局,简称=联邦注册局);
/// 市注册局管理员   → institution_code='CREG' AND province_name AND city_name 的本市机构(如 合肥市注册局)。
/// 无对应行返回 None(前端按空处理,绝不另造名字)。
pub(crate) fn resolve_home_cid_short_name_conn(
    conn: &mut Client,
    registry_org_code: &RegistryOrgCode,
    scope_province_name: Option<&str>,
    scope_city_name: Option<&str>,
) -> Result<Option<String>, String> {
    let row = match registry_org_code {
        RegistryOrgCode::FederalRegistry => conn
            .query_opt(
                "SELECT cid_short_name FROM subjects \
                 WHERE institution_code = 'FRG' AND status = 'ACTIVE' LIMIT 1",
                &[],
            )
            .map_err(|e| format!("query federal registry short name failed: {e}"))?,
        RegistryOrgCode::CityRegistry => {
            let (Some(province), Some(city)) = (scope_province_name, scope_city_name) else {
                return Ok(None);
            };
            conn.query_opt(
                "SELECT cid_short_name FROM subjects \
                 WHERE institution_code = 'CREG' AND status = 'ACTIVE' \
                   AND province_name = $1 AND city_name = $2 LIMIT 1",
                &[&province, &city],
            )
            .map_err(|e| format!("query city registry short name failed: {e}"))?
        }
    };
    Ok(row.and_then(|r| r.get::<usize, Option<String>>(0)))
}

pub(crate) fn resolve_home_cid_short_name(
    db: &Db,
    registry_org_code: &RegistryOrgCode,
    scope_province_name: Option<&str>,
    scope_city_name: Option<&str>,
) -> Result<Option<String>, String> {
    let registry_org_code = registry_org_code.clone();
    let province = scope_province_name.map(str::to_string);
    let city = scope_city_name.map(str::to_string);
    db.with_client(move |conn| {
        resolve_home_cid_short_name_conn(
            conn,
            &registry_org_code,
            province.as_deref(),
            city.as_deref(),
        )
    })
}

pub(crate) fn find_federal_registry_scope_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<Option<String>, String> {
    let row = conn
        .query_opt(
            "SELECT a.admin_account, s.province_name
             FROM federal_registry_scope s
             JOIN admins a ON a.admin_id = s.admin_id
             WHERE lower(a.admin_account) = lower($1)",
            &[&admin_account],
        )
        .map_err(|e| format!("query federal admin scope failed: {e}"))?;
    if let Some(row) = row {
        return Ok(Some(row.get(1)));
    }
    // 中文注释:内置联邦注册局管理员真源为链上常量,本地不反查清单;
    // 省份归属仅以 postgres federal_registry_scope 表为准,无行即 None。
    Ok(None)
}

pub(crate) fn admin_has_active_passkey(db: &Db, admin_account: &str) -> Result<bool, String> {
    let admin_account = admin_account.trim().to_string();
    db.with_client(move |conn| admin_has_active_passkey_conn(conn, admin_account.as_str()))
}

pub(crate) fn admin_has_active_passkey_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<bool, String> {
    let row = conn
        .query_one(
            "SELECT COUNT(*)
             FROM admin_passkeys
             WHERE status = 'ACTIVE' AND lower(admin_account) = lower($1)",
            &[&admin_account],
        )
        .map_err(|e| format!("query active passkeys failed: {e}"))?;
    let count: i64 = row.get(0);
    Ok(count > 0)
}

pub(crate) fn next_admin_id_conn(conn: &mut Client) -> Result<u64, String> {
    let row = conn
        .query_one("SELECT COALESCE(MAX(admin_id), 0) + 1 FROM admins", &[])
        .map_err(|e| format!("allocate admin id failed: {e}"))?;
    let id: i64 = row.get(0);
    Ok(u64::try_from(id).unwrap_or(1))
}

pub(crate) fn upsert_admin_conn(
    conn: &mut Client,
    admin: &AdminUser,
    province_scope: Option<&str>,
) -> Result<(), String> {
    if let Some(scope) = province_scope.map(str::trim).filter(|v| !v.is_empty()) {
        conn.execute(
            "INSERT INTO provinces(province_name)
             VALUES ($1)
             ON CONFLICT (province_name) DO NOTHING",
            &[&scope],
        )
        .map_err(|e| format!("upsert province failed: {e}"))?;
    }
    conn.execute(
        "INSERT INTO admins(admin_id, admin_account, admin_name, registry_org_code, built_in, created_by, created_at, updated_at, city_name)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (admin_account) DO UPDATE SET
            admin_name = EXCLUDED.admin_name,
            registry_org_code = EXCLUDED.registry_org_code,
            built_in = EXCLUDED.built_in,
            created_by = EXCLUDED.created_by,
            updated_at = EXCLUDED.updated_at,
            city_name = EXCLUDED.city_name",
        &[
            &(admin.id as i64),
            &admin.admin_account,
            &admin.admin_name,
            &registry_org_code_text(&admin.registry_org_code),
            &admin.built_in,
            &admin.created_by,
            &admin.created_at,
            &admin.updated_at,
            &admin.city_name,
        ],
    )
    .map_err(|e| format!("upsert admin failed: {e}"))?;
    if admin.registry_org_code == RegistryOrgCode::FederalRegistry {
        let Some(scope) = province_scope else {
            return Err("federal admin province scope missing".to_string());
        };
        let row = conn
            .query_one(
                "SELECT admin_id FROM admins WHERE lower(admin_account) = lower($1)",
                &[&admin.admin_account],
            )
            .map_err(|e| format!("query federal admin id failed: {e}"))?;
        let id: i64 = row.get(0);
        conn.execute(
            "INSERT INTO provinces(province_name) VALUES ($1)
             ON CONFLICT (province_name) DO NOTHING",
            &[&scope],
        )
        .map_err(|e| format!("upsert federal admin province failed: {e}"))?;
        conn.execute(
            "INSERT INTO federal_registry_scope(admin_id, province_name)
             VALUES ($1, $2)
             ON CONFLICT (admin_id) DO UPDATE SET province_name = EXCLUDED.province_name",
            &[&id, &scope],
        )
        .map_err(|e| format!("upsert federal admin scope failed: {e}"))?;
    }
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
        "DELETE FROM admin_passkeys WHERE lower(admin_account) = lower($1)",
        &[&admin_account],
    )
    .map_err(|e| format!("delete admin passkeys failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_passkey_challenges WHERE lower(admin_account) = lower($1)",
        &[&admin_account],
    )
    .map_err(|e| format!("delete admin passkey challenges failed: {e}"))?;
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

pub(crate) fn cleanup_security_state_conn(
    conn: &mut Client,
    now: DateTime<Utc>,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM admin_passkey_challenges
         WHERE consumed = true OR expires_at < $1",
        &[&now],
    )
    .map_err(|e| format!("cleanup passkey challenges failed: {e}"))?;
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

pub(crate) fn insert_passkey_challenge(
    db: &Db,
    challenge: &AdminPasskeyRegistrationChallenge,
) -> Result<(), String> {
    let challenge = challenge.clone();
    db.with_client(move |conn| {
        cleanup_security_state_conn(conn, Utc::now())?;
        upsert_passkey_challenge_conn(conn, &challenge)
    })
}

pub(crate) fn get_passkey_challenge_conn(
    conn: &mut Client,
    registration_id: &str,
) -> Result<Option<AdminPasskeyRegistrationChallenge>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_passkey_challenges WHERE registration_id = $1",
            &[&registration_id],
        )
        .map_err(|e| format!("query passkey challenge failed: {e}"))?;
    row.map(|r| serde_json::from_value::<AdminPasskeyRegistrationChallenge>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode passkey challenge failed: {e}"))
}

pub(crate) fn upsert_passkey_challenge_conn(
    conn: &mut Client,
    challenge: &AdminPasskeyRegistrationChallenge,
) -> Result<(), String> {
    let payload = serde_json::to_value(challenge)
        .map_err(|e| format!("encode passkey challenge failed: {e}"))?;
    conn.execute(
        "INSERT INTO admin_passkey_challenges(registration_id, admin_account, expires_at, consumed, payload)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (registration_id) DO UPDATE SET
            admin_account = EXCLUDED.admin_account,
            expires_at = EXCLUDED.expires_at,
            consumed = EXCLUDED.consumed,
            payload = EXCLUDED.payload",
        &[
            &challenge.registration_id,
            &challenge.admin_account,
            &challenge.expires_at,
            &challenge.consumed,
            &payload,
        ],
    )
    .map_err(|e| format!("upsert passkey challenge failed: {e}"))?;
    Ok(())
}

pub(crate) fn active_passkey_credentials_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<Vec<AdminPasskeyCredential>, String> {
    let rows = conn
        .query(
            "SELECT payload FROM admin_passkeys WHERE status = 'ACTIVE'",
            &[],
        )
        .map_err(|e| format!("query passkey credentials failed: {e}"))?;
    let mut output = Vec::new();
    for row in rows {
        let record: AdminPasskeyCredential = serde_json::from_value(row.get(0))
            .map_err(|e| format!("decode passkey credential failed: {e}"))?;
        if record.status == AdminPasskeyStatus::Active
            && same_admin_account(record.admin_account.as_str(), admin_account)
        {
            output.push(record);
        }
    }
    Ok(output)
}

pub(crate) fn upsert_passkey_credential_conn(
    conn: &mut Client,
    credential: &AdminPasskeyCredential,
) -> Result<(), String> {
    let payload = serde_json::to_value(credential)
        .map_err(|e| format!("encode passkey credential failed: {e}"))?;
    let status = passkey_status_text(&credential.status);
    conn.execute(
        "INSERT INTO admin_passkeys(credential_id, admin_account, label, status, payload, created_at, last_used_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (credential_id) DO UPDATE SET
            admin_account = EXCLUDED.admin_account,
            label = EXCLUDED.label,
            status = EXCLUDED.status,
            payload = EXCLUDED.payload,
            last_used_at = EXCLUDED.last_used_at",
        &[
            &credential.credential_id,
            &credential.admin_account,
            &credential.label,
            &status,
            &payload,
            &credential.created_at,
            &credential.last_used_at,
        ],
    )
    .map_err(|e| format!("upsert passkey credential failed: {e}"))?;
    Ok(())
}

pub(crate) fn revoke_active_passkeys_for_admin_conn(
    conn: &mut Client,
    admin_account: &str,
) -> Result<(), String> {
    let rows = active_passkey_credentials_conn(conn, admin_account)?;
    for mut record in rows {
        record.status = AdminPasskeyStatus::Revoked;
        upsert_passkey_credential_conn(conn, &record)?;
    }
    Ok(())
}

pub(crate) fn get_passkey_credential_conn(
    conn: &mut Client,
    credential_id: &str,
) -> Result<Option<AdminPasskeyCredential>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_passkeys WHERE credential_id = $1",
            &[&credential_id],
        )
        .map_err(|e| format!("query passkey credential failed: {e}"))?;
    row.map(|r| serde_json::from_value::<AdminPasskeyCredential>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode passkey credential failed: {e}"))
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

pub(crate) fn get_action_challenge_conn(
    conn: &mut Client,
    action_id: &str,
) -> Result<Option<AdminActionChallenge>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_action_challenges WHERE action_id = $1",
            &[&action_id],
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

pub(crate) fn get_security_grant_conn(
    conn: &mut Client,
    grant_id: &str,
) -> Result<Option<AdminSecurityGrant>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_security_grants WHERE grant_id = $1",
            &[&grant_id],
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
        "INSERT INTO admin_sessions(token, admin_account, registry_org_code, expires_at, last_active_at, payload)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (token) DO UPDATE SET
            admin_account = EXCLUDED.admin_account,
            registry_org_code = EXCLUDED.registry_org_code,
            expires_at = EXCLUDED.expires_at,
            last_active_at = EXCLUDED.last_active_at,
            payload = EXCLUDED.payload",
        &[
            &session.token,
            &session.admin_account,
            &registry_org_code_text(&session.registry_org_code),
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
         WHERE registry_org_code = 'CITY_REGISTRY' AND last_active_at < $1",
        &[&idle_cutoff],
    )
    .map_err(|e| format!("cleanup idle city admin sessions failed: {e}"))?;
    Ok(())
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

pub(crate) fn passkey_status_text(status: &AdminPasskeyStatus) -> &'static str {
    match status {
        AdminPasskeyStatus::Active => "ACTIVE",
        AdminPasskeyStatus::Revoked => "REVOKED",
    }
}

/// 中文注释:已签发注销凭证行(下发 /deregistration-info 用)。
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

/// 中文注释:写入注册局域注销态(ISSUED,signature 待 commit 层回填)。
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

/// 中文注释:commit 层签发成功后回填签名 + issuer(issuer 来自 env runtime_signing_context,
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

/// 中文注释:签发失败时清掉无签名的 ISSUED 行,保持一致(不留无签名残行)。
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

/// 中文注释:取该机构当前已签发(ISSUED 且已回填签名)的注销凭证,供机构管理员下发。
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
