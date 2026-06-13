//! 中文注释:管理员结构化表读写。
//!
//! 管理员、登录挑战、会话和 Passkey 状态全部以数据库表为唯一持久化真源。

use chrono::{DateTime, Duration, Utc};
use postgres::Client;

use crate::admins::federal_admins::federal_admin_province;
use crate::admins::login::{AdminSession, LoginChallenge, QrLoginResultRecord};
use crate::admins::model::{AdminRole, AdminUser};
use crate::admins::security_model::{
    AdminActionChallenge, AdminPasskeyCredential, AdminPasskeyRegistrationChallenge,
    AdminPasskeyStatus, AdminSecurityGrant,
};
use crate::core::db::postgres_error_text;
use crate::crypto::pubkey::same_admin_pubkey;
use crate::Db;

pub(crate) fn admin_role_text(role: &AdminRole) -> &'static str {
    match role {
        AdminRole::FederalAdmin => "FEDERAL_ADMIN",
        AdminRole::CityAdmin => "CITY_ADMIN",
    }
}

pub(crate) fn parse_admin_role(role: &str) -> Result<AdminRole, String> {
    match role {
        "FEDERAL_ADMIN" => Ok(AdminRole::FederalAdmin),
        "CITY_ADMIN" => Ok(AdminRole::CityAdmin),
        _ => Err(format!("invalid admin role in database: {role}")),
    }
}

fn admin_from_row(row: &postgres::Row) -> Result<AdminUser, String> {
    let id: i64 = row.get(0);
    let role_text: String = row.get(3);
    Ok(AdminUser {
        id: u64::try_from(id).unwrap_or(0),
        admin_pubkey: row.get(1),
        admin_name: row.get(2),
        role: parse_admin_role(role_text.as_str())?,
        built_in: row.get(4),
        created_by: row.get(5),
        created_at: row.get(6),
        updated_at: row.get(7),
        city: row.get(8),
    })
}

pub(crate) fn list_federal_admins_by_province_conn(
    conn: &mut Client,
    province: Option<&str>,
) -> Result<Vec<(AdminUser, String)>, String> {
    let rows = if let Some(province) = province {
        conn.query(
            "SELECT a.admin_id, a.admin_pubkey, a.admin_name, a.role, a.built_in, a.created_by, a.created_at, a.updated_at, a.city,
                    s.province_name
             FROM admins a
             JOIN federal_admin_scope s ON s.admin_id = a.admin_id
             WHERE a.role = 'FEDERAL_ADMIN' AND s.province_name = $1
             ORDER BY s.province_name ASC, a.built_in DESC, a.admin_id ASC",
            &[&province],
        )
    } else {
        conn.query(
            "SELECT a.admin_id, a.admin_pubkey, a.admin_name, a.role, a.built_in, a.created_by, a.created_at, a.updated_at, a.city,
                    s.province_name
             FROM admins a
             JOIN federal_admin_scope s ON s.admin_id = a.admin_id
             WHERE a.role = 'FEDERAL_ADMIN'
             ORDER BY s.province_name ASC, a.built_in DESC, a.admin_id ASC",
            &[],
        )
    }
    .map_err(|e| format!("query federal admins by province failed: {e}"))?;
    rows.iter()
        .map(|row| {
            let admin = admin_from_row(row)?;
            let province: String = row.get(9);
            Ok((admin, province))
        })
        .collect()
}

pub(crate) fn count_federal_admins_by_province_conn(
    conn: &mut Client,
    province: &str,
) -> Result<usize, String> {
    let row = conn
        .query_one(
            "SELECT COUNT(*)
             FROM admins a
             JOIN federal_admin_scope s ON s.admin_id = a.admin_id
             WHERE a.role = 'FEDERAL_ADMIN' AND s.province_name = $1",
            &[&province],
        )
        .map_err(|e| format!("count federal admins by province failed: {e}"))?;
    let count: i64 = row.get(0);
    Ok(usize::try_from(count).unwrap_or(0))
}

pub(crate) fn get_admin_by_id_and_role_conn(
    conn: &mut Client,
    id: u64,
    role: &AdminRole,
) -> Result<Option<AdminUser>, String> {
    let id = id as i64;
    let role = admin_role_text(role);
    let row = conn
        .query_opt(
            "SELECT admin_id, admin_pubkey, admin_name, role, built_in, created_by, created_at, updated_at, city
             FROM admins
             WHERE admin_id = $1 AND role = $2",
            &[&id, &role],
        )
        .map_err(|e| format!("query admin by id and role failed: {e}"))?;
    row.as_ref().map(admin_from_row).transpose()
}

pub(crate) fn list_city_admins_by_scope_conn(
    conn: &mut Client,
    province: &str,
    city: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<(usize, Vec<AdminUser>), String> {
    let limit = i64::try_from(limit).unwrap_or(500);
    let offset = i64::try_from(offset).unwrap_or(0);
    let (count_row, rows) = if let Some(city) = city {
        let count_row = conn
            .query_one(
                "SELECT COUNT(*)
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_pubkey) = lower(a.created_by)
                 JOIN federal_admin_scope s ON s.admin_id = creator.admin_id
                 WHERE a.role = 'CITY_ADMIN' AND s.province_name = $1 AND a.city = $2",
                &[&province, &city],
            )
            .map_err(|e| format!("count city admins by city failed: {e}"))?;
        let rows = conn
            .query(
                "SELECT a.admin_id, a.admin_pubkey, a.admin_name, a.role, a.built_in, a.created_by, a.created_at, a.updated_at, a.city
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_pubkey) = lower(a.created_by)
                 JOIN federal_admin_scope s ON s.admin_id = creator.admin_id
                 WHERE a.role = 'CITY_ADMIN' AND s.province_name = $1 AND a.city = $2
                 ORDER BY a.admin_id DESC
                 LIMIT $3 OFFSET $4",
                &[&province, &city, &limit, &offset],
            )
            .map_err(|e| format!("query city admins by city failed: {e}"))?;
        (count_row, rows)
    } else {
        let count_row = conn
            .query_one(
                "SELECT COUNT(*)
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_pubkey) = lower(a.created_by)
                 JOIN federal_admin_scope s ON s.admin_id = creator.admin_id
                 WHERE a.role = 'CITY_ADMIN' AND s.province_name = $1",
                &[&province],
            )
            .map_err(|e| format!("count city admins by province failed: {e}"))?;
        let rows = conn
            .query(
                "SELECT a.admin_id, a.admin_pubkey, a.admin_name, a.role, a.built_in, a.created_by, a.created_at, a.updated_at, a.city
                 FROM admins a
                 JOIN admins creator ON lower(creator.admin_pubkey) = lower(a.created_by)
                 JOIN federal_admin_scope s ON s.admin_id = creator.admin_id
                 WHERE a.role = 'CITY_ADMIN' AND s.province_name = $1
                 ORDER BY a.admin_id DESC
                 LIMIT $2 OFFSET $3",
                &[&province, &limit, &offset],
            )
            .map_err(|e| format!("query city admins by province failed: {e}"))?;
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

pub(crate) fn count_city_admins_by_city_conn(
    conn: &mut Client,
    province: &str,
    city: &str,
) -> Result<usize, String> {
    let row = conn
        .query_one(
            "SELECT COUNT(*)
             FROM admins a
             JOIN admins creator ON lower(creator.admin_pubkey) = lower(a.created_by)
             JOIN federal_admin_scope s ON s.admin_id = creator.admin_id
             WHERE a.role = 'CITY_ADMIN' AND s.province_name = $1 AND a.city = $2",
            &[&province, &city],
        )
        .map_err(|e| format!("count city admins by city failed: {e}"))?;
    let count: i64 = row.get(0);
    Ok(usize::try_from(count).unwrap_or(0))
}

pub(crate) fn list_city_admins_by_creator_conn(
    conn: &mut Client,
    creator_pubkey: &str,
) -> Result<Vec<AdminUser>, String> {
    let rows = conn
        .query(
            "SELECT admin_id, admin_pubkey, admin_name, role, built_in, created_by, created_at, updated_at, city
             FROM admins
             WHERE role = 'CITY_ADMIN' AND lower(created_by) = lower($1)
             ORDER BY admin_id ASC",
            &[&creator_pubkey],
        )
        .map_err(|e| format!("query city admins by creator failed: {e}"))?;
    rows.iter().map(admin_from_row).collect()
}

pub(crate) fn get_admin_by_pubkey(db: &Db, pubkey: &str) -> Result<Option<AdminUser>, String> {
    let pubkey = pubkey.trim().to_string();
    db.with_client(move |conn| get_admin_by_pubkey_conn(conn, pubkey.as_str()))
}

pub(crate) fn get_admin_by_pubkey_conn(
    conn: &mut Client,
    pubkey: &str,
) -> Result<Option<AdminUser>, String> {
    let row = conn
        .query_opt(
            "SELECT admin_id, admin_pubkey, admin_name, role, built_in, created_by, created_at, updated_at, city
             FROM admins
             WHERE lower(admin_pubkey) = lower($1)",
            &[&pubkey],
        )
        .map_err(|e| format!("query admin by pubkey failed: {e}"))?;
    row.as_ref().map(admin_from_row).transpose()
}

pub(crate) fn resolve_admin_pubkey_key_conn(
    conn: &mut Client,
    candidate: &str,
) -> Result<Option<String>, String> {
    let row = conn
        .query_opt(
            "SELECT admin_pubkey FROM admins WHERE lower(admin_pubkey) = lower($1)",
            &[&candidate],
        )
        .map_err(|e| format!("query admin pubkey key failed: {e}"))?;
    Ok(row.map(|r| r.get(0)))
}

pub(crate) fn province_scope_for_role(
    db: &Db,
    admin_pubkey: &str,
    role: &AdminRole,
) -> Result<Option<String>, String> {
    let admin_pubkey = admin_pubkey.trim().to_string();
    let role = role.clone();
    db.with_client(move |conn| province_scope_for_role_conn(conn, admin_pubkey.as_str(), &role))
}

pub(crate) fn province_scope_for_role_conn(
    conn: &mut Client,
    admin_pubkey: &str,
    role: &AdminRole,
) -> Result<Option<String>, String> {
    match role {
        AdminRole::FederalAdmin => find_federal_admin_scope_conn(conn, admin_pubkey),
        AdminRole::CityAdmin => {
            let Some(admin) = get_admin_by_pubkey_conn(conn, admin_pubkey)? else {
                return Ok(None);
            };
            find_federal_admin_scope_conn(conn, admin.created_by.as_str())
        }
    }
}

pub(crate) fn find_federal_admin_scope_conn(
    conn: &mut Client,
    pubkey: &str,
) -> Result<Option<String>, String> {
    let row = conn
        .query_opt(
            "SELECT a.admin_pubkey, s.province_name
             FROM federal_admin_scope s
             JOIN admins a ON a.admin_id = s.admin_id
             WHERE lower(a.admin_pubkey) = lower($1)",
            &[&pubkey],
        )
        .map_err(|e| format!("query federal admin scope failed: {e}"))?;
    if let Some(row) = row {
        return Ok(Some(row.get(1)));
    }
    Ok(federal_admin_province(pubkey).map(str::to_string))
}

pub(crate) fn admin_has_active_passkey(db: &Db, admin_pubkey: &str) -> Result<bool, String> {
    let admin_pubkey = admin_pubkey.trim().to_string();
    db.with_client(move |conn| admin_has_active_passkey_conn(conn, admin_pubkey.as_str()))
}

pub(crate) fn admin_has_active_passkey_conn(
    conn: &mut Client,
    admin_pubkey: &str,
) -> Result<bool, String> {
    let row = conn
        .query_one(
            "SELECT COUNT(*)
             FROM admin_passkeys
             WHERE status = 'ACTIVE' AND lower(admin_pubkey) = lower($1)",
            &[&admin_pubkey],
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
        "INSERT INTO admins(admin_id, admin_pubkey, admin_name, role, built_in, created_by, created_at, updated_at, city)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (admin_pubkey) DO UPDATE SET
            admin_name = EXCLUDED.admin_name,
            role = EXCLUDED.role,
            built_in = EXCLUDED.built_in,
            created_by = EXCLUDED.created_by,
            updated_at = EXCLUDED.updated_at,
            city = EXCLUDED.city",
        &[
            &(admin.id as i64),
            &admin.admin_pubkey,
            &admin.admin_name,
            &admin_role_text(&admin.role),
            &admin.built_in,
            &admin.created_by,
            &admin.created_at,
            &admin.updated_at,
            &admin.city,
        ],
    )
    .map_err(|e| format!("upsert admin failed: {e}"))?;
    if admin.role == AdminRole::FederalAdmin {
        let Some(scope) = province_scope else {
            return Err("federal admin province scope missing".to_string());
        };
        let row = conn
            .query_one(
                "SELECT admin_id FROM admins WHERE lower(admin_pubkey) = lower($1)",
                &[&admin.admin_pubkey],
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
            "INSERT INTO federal_admin_scope(admin_id, province_name)
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
    pubkey: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM admin_sessions WHERE lower(admin_pubkey) = lower($1)",
        &[&pubkey],
    )
    .map_err(|e| format!("delete admin sessions failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_passkeys WHERE lower(admin_pubkey) = lower($1)",
        &[&pubkey],
    )
    .map_err(|e| format!("delete admin passkeys failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_passkey_challenges WHERE lower(admin_pubkey) = lower($1)",
        &[&pubkey],
    )
    .map_err(|e| format!("delete admin passkey challenges failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_action_challenges WHERE lower(actor_pubkey) = lower($1)",
        &[&pubkey],
    )
    .map_err(|e| format!("delete admin action challenges failed: {e}"))?;
    conn.execute(
        "DELETE FROM admin_security_grants WHERE lower(actor_pubkey) = lower($1)",
        &[&pubkey],
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
        "INSERT INTO admin_passkey_challenges(registration_id, admin_pubkey, expires_at, consumed, payload)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (registration_id) DO UPDATE SET
            admin_pubkey = EXCLUDED.admin_pubkey,
            expires_at = EXCLUDED.expires_at,
            consumed = EXCLUDED.consumed,
            payload = EXCLUDED.payload",
        &[
            &challenge.registration_id,
            &challenge.admin_pubkey,
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
    admin_pubkey: &str,
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
            && same_admin_pubkey(record.admin_pubkey.as_str(), admin_pubkey)
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
        "INSERT INTO admin_passkeys(credential_id, admin_pubkey, label, status, payload, created_at, last_used_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (credential_id) DO UPDATE SET
            admin_pubkey = EXCLUDED.admin_pubkey,
            label = EXCLUDED.label,
            status = EXCLUDED.status,
            payload = EXCLUDED.payload,
            last_used_at = EXCLUDED.last_used_at",
        &[
            &credential.credential_id,
            &credential.admin_pubkey,
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
    admin_pubkey: &str,
) -> Result<(), String> {
    let rows = active_passkey_credentials_conn(conn, admin_pubkey)?;
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
        "INSERT INTO admin_action_challenges(action_id, actor_pubkey, action_type, expires_at, consumed, payload)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (action_id) DO UPDATE SET
            actor_pubkey = EXCLUDED.actor_pubkey,
            action_type = EXCLUDED.action_type,
            expires_at = EXCLUDED.expires_at,
            consumed = EXCLUDED.consumed,
            payload = EXCLUDED.payload",
        &[
            &challenge.action_id,
            &challenge.actor_pubkey,
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
        "INSERT INTO admin_security_grants(grant_id, actor_pubkey, action_type, expires_at, consumed, payload)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (grant_id) DO UPDATE SET
            actor_pubkey = EXCLUDED.actor_pubkey,
            action_type = EXCLUDED.action_type,
            expires_at = EXCLUDED.expires_at,
            consumed = EXCLUDED.consumed,
            payload = EXCLUDED.payload",
        &[
            &grant.grant_id,
            &grant.actor_pubkey,
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
        "DELETE FROM admin_login_challenges
         WHERE expires_at < $1
            OR (consumed = true AND expires_at < $2)",
        &[&stale_login_before, &consumed_login_before],
    )
    .map_err(|e| {
        format!(
            "cleanup login challenges failed: {}",
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

pub(crate) fn insert_login_challenge(db: &Db, challenge: &LoginChallenge) -> Result<(), String> {
    let challenge = challenge.clone();
    db.with_client(move |conn| {
        cleanup_login_state_conn(conn, Utc::now())?;
        let payload = serde_json::to_value(&challenge)
            .map_err(|e| format!("encode login challenge failed: {e}"))?;
        conn.execute(
            "INSERT INTO admin_login_challenges(challenge_id, session_id, admin_pubkey, expires_at, consumed, payload)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (challenge_id) DO UPDATE SET
                session_id = EXCLUDED.session_id,
                admin_pubkey = EXCLUDED.admin_pubkey,
                expires_at = EXCLUDED.expires_at,
                consumed = EXCLUDED.consumed,
                payload = EXCLUDED.payload",
            &[
                &challenge.challenge_id,
                &challenge.session_id,
                &challenge.admin_pubkey,
                &challenge.expire_at,
                &challenge.consumed,
                &payload,
            ],
        )
        .map_err(|e| format!("insert login challenge failed: {}", postgres_error_text(&e)))?;
        Ok(())
    })
}

pub(crate) fn get_login_challenge_conn(
    conn: &mut Client,
    challenge_id: &str,
) -> Result<Option<LoginChallenge>, String> {
    let row = conn
        .query_opt(
            "SELECT payload FROM admin_login_challenges WHERE challenge_id = $1",
            &[&challenge_id],
        )
        .map_err(|e| format!("query login challenge failed: {}", postgres_error_text(&e)))?;
    row.map(|r| serde_json::from_value::<LoginChallenge>(r.get(0)))
        .transpose()
        .map_err(|e| format!("decode login challenge failed: {e}"))
}

pub(crate) fn update_login_challenge_conn(
    conn: &mut Client,
    challenge: &LoginChallenge,
) -> Result<(), String> {
    let payload = serde_json::to_value(challenge)
        .map_err(|e| format!("encode login challenge failed: {e}"))?;
    conn.execute(
        "UPDATE admin_login_challenges
         SET admin_pubkey = $2, expires_at = $3, consumed = $4, payload = $5
         WHERE challenge_id = $1",
        &[
            &challenge.challenge_id,
            &challenge.admin_pubkey,
            &challenge.expire_at,
            &challenge.consumed,
            &payload,
        ],
    )
    .map_err(|e| format!("update login challenge failed: {}", postgres_error_text(&e)))?;
    Ok(())
}

pub(crate) fn insert_admin_session_conn(
    conn: &mut Client,
    session: &AdminSession,
) -> Result<(), String> {
    let payload =
        serde_json::to_value(session).map_err(|e| format!("encode admin session failed: {e}"))?;
    conn.execute(
        "INSERT INTO admin_sessions(token, admin_pubkey, role, expires_at, last_active_at, payload)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (token) DO UPDATE SET
            admin_pubkey = EXCLUDED.admin_pubkey,
            role = EXCLUDED.role,
            expires_at = EXCLUDED.expires_at,
            last_active_at = EXCLUDED.last_active_at,
            payload = EXCLUDED.payload",
        &[
            &session.token,
            &session.admin_pubkey,
            &admin_role_text(&session.role),
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
         WHERE role = 'CITY_ADMIN' AND last_active_at < $1",
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
