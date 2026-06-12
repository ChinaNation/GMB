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
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tracing::{info, warn};
use uuid::Uuid;

mod accounts;
mod admins;
mod audit;
mod china;
mod citizens;
mod core;
mod cpms;
mod crypto;
mod docs;
mod gov;
mod indexer;
mod number;
mod private;
mod scope;
mod subjects;

#[cfg(test)]
mod genesis {
    // 中文注释:SFID 测试编译会加载 citizenchain 的 china_ch 常量测试,
    // 该测试只需要创世人口常量来校验省储行人口总和。
    pub const GENESIS_CITIZEN_MAX: u64 = 1_443_497_378;
}

pub(crate) use crate::core::http_security::*;
pub(crate) use crate::core::response::*;
pub(crate) use crate::core::runtime_ops::*;
pub(crate) use crate::core::{db::Db, secret::SensitiveSeed};
pub(crate) use admins::login::{
    build_admin_display_name, parse_sr25519_pubkey, parse_sr25519_pubkey_bytes, require_admin_any,
    require_sheng_admin,
};
pub(crate) use admins::model::*;
pub(crate) use citizens::model::*;
pub(crate) use cpms::model::*;
pub(crate) use cpms::scope::in_scope_cpms_site;
pub(crate) use number::model::*;

#[derive(Clone)]
struct AppState {
    db: Db,
    rate_limit_redis: Arc<RedisClient>,
}

#[derive(Clone, Copy)]
struct DbPageCursor {
    created_at: DateTime<Utc>,
    id: i64,
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

fn institution_category_text(category: crate::number::InstitutionCategory) -> &'static str {
    match category {
        crate::number::InstitutionCategory::PublicSecurity => "PUBLIC_SECURITY",
        crate::number::InstitutionCategory::GovInstitution => "GOV_INSTITUTION",
        crate::number::InstitutionCategory::PrivateInstitution => "PRIVATE_INSTITUTION",
    }
}

fn institution_category_from_text(category: &str) -> Option<crate::number::InstitutionCategory> {
    match category {
        "PUBLIC_SECURITY" => Some(crate::number::InstitutionCategory::PublicSecurity),
        "GOV_INSTITUTION" => Some(crate::number::InstitutionCategory::GovInstitution),
        "PRIVATE_INSTITUTION" => Some(crate::number::InstitutionCategory::PrivateInstitution),
        _ => None,
    }
}

fn multisig_chain_status_text(status: &crate::subjects::MultisigChainStatus) -> &'static str {
    match status {
        crate::subjects::MultisigChainStatus::NotOnChain => "NOT_ON_CHAIN",
        crate::subjects::MultisigChainStatus::PendingOnChain => "PENDING_ON_CHAIN",
        crate::subjects::MultisigChainStatus::ActiveOnChain => "ACTIVE_ON_CHAIN",
        crate::subjects::MultisigChainStatus::RevokedOnChain => "REVOKED_ON_CHAIN",
    }
}

fn multisig_chain_status_from_text(status: &str) -> crate::subjects::MultisigChainStatus {
    match status {
        "PENDING_ON_CHAIN" => crate::subjects::MultisigChainStatus::PendingOnChain,
        "ACTIVE_ON_CHAIN" => crate::subjects::MultisigChainStatus::ActiveOnChain,
        "REVOKED_ON_CHAIN" => crate::subjects::MultisigChainStatus::RevokedOnChain,
        _ => crate::subjects::MultisigChainStatus::NotOnChain,
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
        manifest_version: None,
        catalog_status: None,
    }
}

fn citizen_row_from_record(record: &CitizenRecord) -> CitizenRow {
    CitizenRow {
        id: record.id,
        wallet_pubkey: record.wallet_pubkey.clone(),
        wallet_address: record.wallet_address.clone(),
        archive_no: record.archive_no.clone(),
        sfid_number: record.sfid_number.clone(),
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
    inst: &crate::subjects::Institution,
    account_count: usize,
    created_by_name: Option<String>,
    created_by_role: Option<String>,
) -> crate::subjects::InstitutionListRow {
    crate::subjects::InstitutionListRow {
        sfid_number: inst.sfid_number.clone(),
        institution_name: inst.institution_name.clone(),
        sfid_name: inst.sfid_name.clone(),
        short_name: inst.short_name.clone(),
        status: inst.status.clone(),
        category: inst.category,
        subject_property: inst.subject_property.clone(),
        p1: inst.p1.clone(),
        province: inst.province.clone(),
        city: inst.city.clone(),
        town: inst.town.clone(),
        institution_code: inst.institution_code.clone(),
        org_code: inst.org_code.clone(),
        sub_type: inst.sub_type.clone(),
        parent_sfid_number: inst.parent_sfid_number.clone(),
        account_count,
        cpms_status: None,
        install_token_status: None,
        identity_service_status: None,
        created_at: inst.created_at,
        created_by_name,
        created_by_role,
    }
}

fn institution_row_from_pg_row(
    row: &postgres::Row,
) -> Result<crate::subjects::InstitutionListRow, String> {
    let category_text: String = row.get(2);
    let category = institution_category_from_text(category_text.as_str())
        .ok_or_else(|| format!("invalid institution category: {category_text}"))?;
    let account_count_i64: i64 = row.get(14);
    let created_by_name: Option<String> = row.get(15);
    let created_by_role: Option<String> = row.get(16);
    let sfid_name: Option<String> = row.get(17);
    let short_name: Option<String> = row.get(18);
    let town: Option<String> = row.get(19);
    let town_code: Option<String> = row.get(20);
    let org_code: Option<String> = row.get(21);
    let status: String = row.get(22);
    let cpms_status: Option<String> = row.get(23);
    let install_token_status: Option<String> = row.get(24);
    let cpms_pubkey_bound: Option<bool> = row.get(25);
    // 中文注释:公安局列表唯一的"业务状态"单轴(前端只显示这一列):
    // 待生成安装码 → 待安装 → 待绑定身份码 → 可办理,外加 已禁用/已吊销 两个管理态。
    // CPMS 站点状态/安装码状态是它的派生输入,不再单列展示。
    let identity_service_status = if category == crate::number::InstitutionCategory::PublicSecurity
    {
        Some(
            match cpms_status.as_deref() {
                Some("ACTIVE") if cpms_pubkey_bound.unwrap_or(false) => "READY",
                Some("ACTIVE") => "WAITING_IDENTITY_CODE",
                Some("DISABLED") => "DISABLED",
                Some("REVOKED") => "REVOKED",
                // PENDING = 安装码已生成,等现场扫码安装
                Some(_) => "WAITING_INSTALL",
                // 无 CPMS 站点记录 = 还没生成安装码
                None => "WAITING_INSTALL_CODE",
            }
            .to_string(),
        )
    } else {
        None
    };
    let inst = crate::subjects::Institution {
        sfid_number: row.get(0),
        institution_name: row.get(1),
        sfid_name,
        short_name,
        status,
        category,
        subject_property: row.get(3),
        p1: row.get(4),
        province: row.get(5),
        city: row.get(6),
        town: town.unwrap_or_default(),
        province_code: row.get(7),
        city_code: row.get(8),
        town_code: town_code.unwrap_or_default(),
        institution_code: row.get(9),
        org_code,
        sub_type: row.get(10),
        parent_sfid_number: row.get(11),
        legal_rep_name: None,
        legal_rep_sfid_number: None,
        legal_rep_photo_path: None,
        legal_rep_photo_name: None,
        legal_rep_photo_mime: None,
        legal_rep_photo_size: None,
        created_by: row.get(12),
        created_at: row.get(13),
    };
    let mut item = institution_row_from_record(
        &inst,
        usize::try_from(account_count_i64).unwrap_or(0),
        created_by_name,
        created_by_role,
    );
    item.cpms_status = cpms_status;
    item.install_token_status = install_token_status;
    item.identity_service_status = identity_service_status;
    Ok(item)
}

fn institution_from_subject_row(
    row: &postgres::Row,
) -> Result<crate::subjects::Institution, String> {
    let category_text: String = row.get(2);
    let category = institution_category_from_text(category_text.as_str())
        .ok_or_else(|| format!("invalid institution category: {category_text}"))?;
    let sfid_name: Option<String> = row.get(14);
    let short_name: Option<String> = row.get(15);
    let town: Option<String> = row.get(16);
    let town_code: Option<String> = row.get(17);
    let org_code: Option<String> = row.get(18);
    let status: String = row.get(19);
    let legal_rep_photo_size_i64: Option<i64> = row.get(25);
    Ok(crate::subjects::Institution {
        sfid_number: row.get(0),
        institution_name: row.get(1),
        sfid_name,
        short_name,
        status,
        category,
        subject_property: row.get(3),
        p1: row.get(4),
        province: row.get(5),
        city: row.get(6),
        town: town.unwrap_or_default(),
        province_code: row.get(7),
        city_code: row.get(8),
        town_code: town_code.unwrap_or_default(),
        institution_code: row.get(9),
        org_code,
        sub_type: row.get(10),
        parent_sfid_number: row.get(11),
        legal_rep_name: row.get(20),
        legal_rep_sfid_number: row.get(21),
        legal_rep_photo_path: row.get(22),
        legal_rep_photo_name: row.get(23),
        legal_rep_photo_mime: row.get(24),
        legal_rep_photo_size: legal_rep_photo_size_i64.and_then(|v| u64::try_from(v).ok()),
        created_by: row.get(12),
        created_at: row.get(13),
    })
}

fn offset_page_from_window<T: Serialize>(
    mut rows: Vec<T>,
    offset: usize,
    page_size: usize,
) -> PageResult<T> {
    let has_more = rows.len() > page_size;
    if has_more {
        rows.truncate(page_size);
    }
    let next_offset = offset.saturating_add(rows.len());
    PageResult {
        items: rows,
        page_size,
        next_cursor: has_more.then(|| next_offset.to_string()),
        has_more,
        manifest_version: None,
        catalog_status: None,
    }
}

impl Db {
    pub(crate) fn institution_name_exists(
        &self,
        name: &str,
        province_code: Option<&str>,
        city_code: Option<&str>,
        exclude_sfid_number: Option<&str>,
    ) -> Result<bool, String> {
        let name = name.trim().to_string();
        let province_code = province_code.map(str::to_string);
        let city_code = city_code.map(str::to_string);
        let exclude_sfid_number = exclude_sfid_number.map(str::to_string);
        self.with_client(move |conn| {
            let row = conn
                .query_one(
                    "SELECT EXISTS (
                        SELECT 1 FROM subjects
                        WHERE kind IN ('PUBLIC', 'PRIVATE')
                          AND name = $1
                          AND ($2::text IS NULL OR p_code = $2)
                          AND ($3::text IS NULL OR c_code = $3)
                          AND ($4::text IS NULL OR sfid_number <> $4)
                     )",
                    &[&name, &province_code, &city_code, &exclude_sfid_number],
                )
                .map_err(|e| format!("query institution name conflict failed: {e}"))?;
            Ok(row.get(0))
        })
    }

    pub(crate) fn get_institution_with_accounts(
        &self,
        sfid_number: &str,
    ) -> Result<
        Option<(
            crate::subjects::Institution,
            Vec<crate::subjects::InstitutionAccount>,
        )>,
        String,
    > {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT s.sfid_number, s.name, s.category,
                            s.subject_property, s.p1, s.province,
                            s.city, s.province_code, s.city_code, s.institution_code,
		                            s.sub_type, s.parent_sfid_number,
		                            s.created_by, s.created_at, s.sfid_name, s.short_name,
	                            COALESCE(s.town, ''), COALESCE(s.town_code, ''), s.org_code,
	                            s.status, s.legal_rep_name, s.legal_rep_sfid_number,
	                            s.legal_rep_photo_path, s.legal_rep_photo_name,
	                            s.legal_rep_photo_mime, s.legal_rep_photo_size
		                     FROM subjects s
		                     LEFT JOIN gov g ON g.p_code = s.p_code AND g.sfid_number = s.sfid_number
	                     WHERE s.kind IN ('PUBLIC', 'PRIVATE') AND s.sfid_number = $1
	                     LIMIT 1",
                    &[&sfid_number],
                )
                .map_err(|e| format!("query institution failed: {e}"))?;
            let Some(row) = row else {
                return Ok(None);
            };
            let inst = institution_from_subject_row(&row)?;
            let account_rows = conn
                .query(
                    "SELECT sfid_number, account_name, duoqian_address, chain_status, created_at
                     FROM accounts
                     WHERE sfid_number = $1
                     ORDER BY account_name ASC",
                    &[&sfid_number],
                )
                .map_err(|e| format!("query institution accounts failed: {e}"))?;
            let mut accounts = Vec::with_capacity(account_rows.len());
            for row in account_rows {
                let status_text: String = row.get(3);
                accounts.push(crate::subjects::InstitutionAccount {
                    sfid_number: row.get(0),
                    account_name: row.get(1),
                    duoqian_address: row.get(2),
                    chain_status: multisig_chain_status_from_text(status_text.as_str()),
                    chain_synced_at: None,
                    chain_tx_hash: None,
                    chain_block_number: None,
                    created_by: String::new(),
                    created_at: row.get(4),
                });
            }
            Ok(Some((inst, accounts)))
        })
    }

    pub(crate) fn upsert_citizen_row(&self, record: &CitizenRecord) -> Result<(), String> {
        let record = record.clone();
        self.with_client(move |conn| {
            Self::upsert_target_citizen_rows(conn, &record)?;
            Ok(())
        })
    }

    fn upsert_target_citizen_rows(
        conn: &mut postgres::Client,
        record: &CitizenRecord,
    ) -> Result<(), String> {
        let Some(sfid_number) = record
            .sfid_number
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        else {
            return Ok(());
        };
        let p_code = record
            .province_code
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| Self::scope_codes_from_sfid(sfid_number.as_str()).0);
        let c_code = record
            .city_code
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .or_else(|| Self::scope_codes_from_sfid(sfid_number.as_str()).1)
            .unwrap_or_else(|| "000".to_string());
        let status = if matches!(record.computed_identity_status(), CitizenStatus::Normal) {
            "ACTIVE"
        } else {
            "REVOKED"
        };
        let citizen_status = record
            .citizen_status
            .as_ref()
            .map(citizen_status_text)
            .unwrap_or("REVOKED");
        let bind_status = citizen_bind_status_text(&record.bind_status());
        let id = i64::try_from(record.id).map_err(|_| "citizen id exceeds i64".to_string())?;
        Self::upsert_target_id_row(
            conn,
            sfid_number.as_str(),
            "CITIZEN",
            p_code.as_str(),
            Some(c_code.as_str()),
        )?;
        Self::delete_target_rows_outside_scope(
            conn,
            "subjects",
            sfid_number.as_str(),
            p_code.as_str(),
        )?;
        Self::delete_target_rows_outside_scope(
            conn,
            "citizens",
            sfid_number.as_str(),
            p_code.as_str(),
        )?;
        conn.execute(
            "INSERT INTO subjects (
                sfid_number, kind, name, p_code, c_code, status, created_at, updated_at
             ) VALUES ($1, 'CITIZEN', NULL, $2, $3, $4, $5, now())
             ON CONFLICT (p_code, sfid_number) DO UPDATE SET
                c_code = EXCLUDED.c_code,
                status = EXCLUDED.status,
                updated_at = now()",
            &[&sfid_number, &p_code, &c_code, &status, &record.created_at],
        )
        .map_err(|e| format!("upsert citizen subject failed: {e}"))?;
        conn.execute(
            "INSERT INTO citizens (
                sfid_number, p_code, c_code, id, archive_no, wallet_pubkey, wallet_address,
                citizen_status, voting_eligible, valid_from, valid_until, status_updated_at,
                bind_status, bound_at, bound_by, created_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
             ON CONFLICT (p_code, sfid_number) DO UPDATE SET
                c_code = EXCLUDED.c_code,
                id = EXCLUDED.id,
                archive_no = EXCLUDED.archive_no,
                wallet_pubkey = EXCLUDED.wallet_pubkey,
                wallet_address = EXCLUDED.wallet_address,
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
                &sfid_number,
                &p_code,
                &c_code,
                &id,
                &record.archive_no,
                &record.wallet_pubkey,
                &record.wallet_address,
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
        .map_err(|e| format!("upsert citizens failed: {e}"))?;
        Ok(())
    }

    fn upsert_target_id_row(
        conn: &mut postgres::Client,
        sfid_number: &str,
        kind: &str,
        p_code: &str,
        c_code: Option<&str>,
    ) -> Result<(), String> {
        // 中文注释:ids 是 sfid_number 全局唯一索引,同号不能在身份大类之间静默改义。
        let existing = conn
            .query_opt(
                "SELECT kind FROM ids WHERE sfid_number = $1",
                &[&sfid_number],
            )
            .map_err(|e| format!("query target id failed: {e}"))?;
        if let Some(row) = existing {
            let existing_kind: String = row.get(0);
            if existing_kind != kind {
                return Err(format!(
                    "sfid_number {sfid_number} already belongs to {existing_kind}, cannot write {kind}"
                ));
            }
            conn.execute(
                "UPDATE ids SET p_code = $2, c_code = $3 WHERE sfid_number = $1",
                &[&sfid_number, &p_code, &c_code],
            )
            .map_err(|e| format!("update target id failed: {e}"))?;
        } else {
            conn.execute(
                "INSERT INTO ids (sfid_number, kind, p_code, c_code)
                 VALUES ($1, $2, $3, $4)",
                &[&sfid_number, &kind, &p_code, &c_code],
            )
            .map_err(|e| format!("insert target id failed: {e}"))?;
        }
        Ok(())
    }

    fn delete_target_rows_outside_scope(
        conn: &mut postgres::Client,
        table: &str,
        sfid_number: &str,
        p_code: &str,
    ) -> Result<(), String> {
        // 中文注释:分区键按行政区划真源修正时,清掉同一 sfid 留在原分区的查询行。
        let sql = format!("DELETE FROM {table} WHERE sfid_number = $1 AND p_code <> $2");
        conn.execute(sql.as_str(), &[&sfid_number, &p_code])
            .map_err(|e| format!("delete {table} rows outside scope failed: {e}"))?;
        Ok(())
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
                manifest_version: None,
                catalog_status: None,
            });
        }
        let cursor = decode_db_page_cursor(cursor)?;
        let keyword = keyword.to_string();
        let province_code = province_code.map(str::to_string);
        let city_code = city_code.map(str::to_string);
        self.with_client(move |conn| {
            let cursor_created_at = cursor.map(|c| c.created_at);
            let cursor_id = cursor.map(|c| c.id).unwrap_or(i64::MAX);
            let fetch_limit = i64::try_from(page_size.saturating_add(1))
                .map_err(|_| "page_size too large".to_string())?;
            let rows = conn
                .query(
                    "SELECT COALESCE(c.id, 0), c.wallet_pubkey, c.wallet_address,
                                    c.archive_no, c.sfid_number, c.citizen_status,
                                    c.voting_eligible, c.valid_from, c.valid_until,
                                    c.status_updated_at, c.bind_status, c.p_code, c.c_code,
                                    c.bound_at, c.bound_by, c.created_at
                             FROM citizens c
                             JOIN subjects s
                               ON s.p_code = c.p_code
                              AND s.sfid_number = c.sfid_number
                              AND s.kind = 'CITIZEN'
                             WHERE c.bind_status = 'BOUND'
                               AND ($1::text IS NULL OR c.p_code = $1)
                               AND ($2::text IS NULL OR c.c_code = $2)
                               AND (
                                    c.archive_no = $3 OR c.sfid_number = $3
                                    OR lower(c.wallet_pubkey) = lower($3)
                                    OR lower(c.wallet_address) = lower($3)
                               )
                               AND (
                                    $4::timestamptz IS NULL
                                    OR c.created_at < $4
                                    OR (c.created_at = $4 AND COALESCE(c.id, 0) < $5)
                               )
                             ORDER BY c.created_at DESC, COALESCE(c.id, 0) DESC
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
                .map_err(|e| format!("query citizens failed: {e}"))?;
            let mut output = Vec::with_capacity(rows.len());
            for row in rows {
                let id_i64: i64 = row.get(0);
                let created_at: DateTime<Utc> = row.get(15);
                let record = CitizenRecord {
                    id: u64::try_from(id_i64).unwrap_or(0),
                    wallet_pubkey: row.get(1),
                    wallet_address: row.get(2),
                    archive_no: row.get(3),
                    sfid_number: row.get(4),
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

    pub(crate) fn upsert_institution_row(
        &self,
        inst: &crate::subjects::Institution,
    ) -> Result<(), String> {
        let inst = inst.clone();
        self.with_client(move |conn| {
            Self::upsert_target_subject_rows(conn, &inst)?;
            Ok(())
        })
    }

    pub(crate) fn legal_representative_citizen_exists(
        &self,
        sfid_number: &str,
    ) -> Result<bool, String> {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_one(
                    "SELECT EXISTS (
                        SELECT 1 FROM citizens
                        WHERE sfid_number = $1
                          AND citizen_status = 'NORMAL'
                     )",
                    &[&sfid_number],
                )
                .map_err(|e| format!("query legal representative citizen failed: {e}"))?;
            Ok(row.get(0))
        })
    }

    fn upsert_target_subject_rows(
        conn: &mut postgres::Client,
        inst: &crate::subjects::Institution,
    ) -> Result<(), String> {
        let kind = match inst.category {
            crate::number::InstitutionCategory::PrivateInstitution => "PRIVATE",
            crate::number::InstitutionCategory::PublicSecurity
            | crate::number::InstitutionCategory::GovInstitution => "PUBLIC",
        };
        let p_code = inst.province_code.clone();
        let c_code = if inst.city_code == "000" || inst.city_code.is_empty() {
            None
        } else {
            Some(inst.city_code.clone())
        };
        let t_code = if inst.town_code.trim().is_empty() {
            None
        } else {
            Some(inst.town_code.clone())
        };
        let status = inst.status.trim().to_string();
        Self::upsert_target_id_row(
            conn,
            inst.sfid_number.as_str(),
            kind,
            p_code.as_str(),
            c_code.as_deref(),
        )?;
        Self::delete_target_rows_outside_scope(
            conn,
            "subjects",
            inst.sfid_number.as_str(),
            p_code.as_str(),
        )?;
        let category = institution_category_text(inst.category);
        let legal_rep_photo_size = inst
            .legal_rep_photo_size
            .and_then(|v| i64::try_from(v).ok());
        conn.execute(
            "INSERT INTO subjects (
		                sfid_number, kind, name, sfid_name, short_name, p_code, c_code, t_code,
		                status, category, subject_property, p1, province, city, town,
		                province_code, city_code, town_code, institution_code, org_code, sub_type,
		                parent_sfid_number, legal_rep_name, legal_rep_sfid_number,
		                legal_rep_photo_path, legal_rep_photo_name, legal_rep_photo_mime,
		                legal_rep_photo_size, created_by, created_at, updated_at
		             ) VALUES (
		                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
		                $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
		                $25, $26, $27, $28, $29, $30, now()
		             )
		             ON CONFLICT (p_code, sfid_number) DO UPDATE SET
		                kind = EXCLUDED.kind,
	                name = EXCLUDED.name,
	                sfid_name = EXCLUDED.sfid_name,
	                short_name = EXCLUDED.short_name,
	                c_code = EXCLUDED.c_code,
		                t_code = EXCLUDED.t_code,
		                status = EXCLUDED.status,
	                category = EXCLUDED.category,
	                subject_property = EXCLUDED.subject_property,
	                p1 = EXCLUDED.p1,
	                province = EXCLUDED.province,
	                city = EXCLUDED.city,
	                town = EXCLUDED.town,
	                province_code = EXCLUDED.province_code,
	                city_code = EXCLUDED.city_code,
	                town_code = EXCLUDED.town_code,
	                institution_code = EXCLUDED.institution_code,
	                org_code = EXCLUDED.org_code,
	                sub_type = EXCLUDED.sub_type,
                parent_sfid_number = EXCLUDED.parent_sfid_number,
                legal_rep_name = EXCLUDED.legal_rep_name,
                legal_rep_sfid_number = EXCLUDED.legal_rep_sfid_number,
                legal_rep_photo_path = EXCLUDED.legal_rep_photo_path,
                legal_rep_photo_name = EXCLUDED.legal_rep_photo_name,
                legal_rep_photo_mime = EXCLUDED.legal_rep_photo_mime,
                legal_rep_photo_size = EXCLUDED.legal_rep_photo_size,
                created_by = EXCLUDED.created_by,
                updated_at = now()",
            &[
                &inst.sfid_number,
                &kind,
                &inst.institution_name,
                &inst.sfid_name,
                &inst.short_name,
                &p_code,
                &c_code,
                &t_code,
                &status,
                &category,
                &inst.subject_property,
                &inst.p1,
                &inst.province,
                &inst.city,
                &inst.town,
                &inst.province_code,
                &inst.city_code,
                &inst.town_code,
                &inst.institution_code,
                &inst.org_code,
                &inst.sub_type,
                &inst.parent_sfid_number,
                &inst.legal_rep_name,
                &inst.legal_rep_sfid_number,
                &inst.legal_rep_photo_path,
                &inst.legal_rep_photo_name,
                &inst.legal_rep_photo_mime,
                &legal_rep_photo_size,
                &inst.created_by,
                &inst.created_at,
            ],
        )
        .map_err(|e| format!("upsert subjects failed: {e}"))?;

        match inst.category {
            crate::number::InstitutionCategory::PrivateInstitution => {
                Self::delete_target_rows_outside_scope(
                    conn,
                    "private",
                    inst.sfid_number.as_str(),
                    p_code.as_str(),
                )?;
                conn.execute(
                    "DELETE FROM gov WHERE sfid_number = $1",
                    &[&inst.sfid_number],
                )
                .map_err(|e| format!("delete gov row for private subject failed: {e}"))?;
                let private_kind = if inst.institution_code == "JY" {
                    "SCHOOL"
                } else if inst.subject_property == "F" {
                    "BRANCH"
                } else if inst.p1 == "0" {
                    "NONPROFIT"
                } else {
                    "PROFIT"
                };
                conn.execute(
                    "INSERT INTO private (
                        sfid_number, p_code, c_code, code, kind, subject_property, p1, sub_type, parent_sfid_number
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     ON CONFLICT (p_code, sfid_number) DO UPDATE SET
                        c_code = EXCLUDED.c_code,
                        code = EXCLUDED.code,
                        kind = EXCLUDED.kind,
                        subject_property = EXCLUDED.subject_property,
                        p1 = EXCLUDED.p1,
                        sub_type = EXCLUDED.sub_type,
                        parent_sfid_number = EXCLUDED.parent_sfid_number",
                    &[
                        &inst.sfid_number,
                        &p_code,
                        &c_code,
                        &inst.institution_code,
                        &private_kind,
                        &inst.subject_property,
                        &inst.p1,
                        &inst.sub_type,
                        &inst.parent_sfid_number,
                    ],
                )
                .map_err(|e| format!("upsert private failed: {e}"))?;
            }
            crate::number::InstitutionCategory::PublicSecurity
            | crate::number::InstitutionCategory::GovInstitution => {
                Self::delete_target_rows_outside_scope(
                    conn,
                    "gov",
                    inst.sfid_number.as_str(),
                    p_code.as_str(),
                )?;
                conn.execute(
                    "DELETE FROM private WHERE sfid_number = $1",
                    &[&inst.sfid_number],
                )
                .map_err(|e| format!("delete private row for public subject failed: {e}"))?;
                let home_p: Option<String> = None;
                let home_c: Option<String> = None;
                conn.execute(
                    "INSERT INTO gov (
		                        sfid_number, p_code, c_code, t_code, institution_code, org_code,
		                        home_p, home_c
		                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
		                     ON CONFLICT (p_code, sfid_number) DO UPDATE SET
		                        c_code = EXCLUDED.c_code,
		                        t_code = EXCLUDED.t_code,
		                        institution_code = EXCLUDED.institution_code,
		                        org_code = EXCLUDED.org_code,
		                        home_p = EXCLUDED.home_p,
		                        home_c = EXCLUDED.home_c",
                    &[
                        &inst.sfid_number,
                        &p_code,
                        &c_code,
                        &t_code,
                        &inst.institution_code,
                        &inst.org_code,
                        &home_p,
                        &home_c,
                    ],
                )
                .map_err(|e| format!("upsert gov failed: {e}"))?;
            }
        }
        Ok(())
    }

    pub(crate) fn upsert_institution_account_row(
        &self,
        account: &crate::subjects::InstitutionAccount,
    ) -> Result<(), String> {
        let account = account.clone();
        self.with_client(move |conn| {
            Self::upsert_target_account_row(conn, &account)?;
            Ok(())
        })
    }

    fn upsert_target_account_row(
        conn: &mut postgres::Client,
        account: &crate::subjects::InstitutionAccount,
    ) -> Result<(), String> {
        let scope_row = conn
            .query_opt(
                "SELECT p_code, c_code FROM ids WHERE sfid_number = $1",
                &[&account.sfid_number],
            )
            .map_err(|e| format!("query id scope for account failed: {e}"))?;
        let (fallback_p, fallback_c) = Self::scope_codes_from_sfid(account.sfid_number.as_str());
        let (p_code, c_code): (String, Option<String>) = match scope_row {
            Some(row) => (row.get(0), row.get(1)),
            None => (fallback_p, fallback_c),
        };
        let chain_status = multisig_chain_status_text(&account.chain_status);
        Self::delete_target_rows_outside_scope(
            conn,
            "accounts",
            account.sfid_number.as_str(),
            p_code.as_str(),
        )?;
        conn.execute(
            "INSERT INTO accounts (
                sfid_number, p_code, c_code, account_name, duoqian_address, chain_status, created_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (p_code, sfid_number, account_name) DO UPDATE SET
                c_code = EXCLUDED.c_code,
                duoqian_address = EXCLUDED.duoqian_address,
                chain_status = EXCLUDED.chain_status,
                created_at = EXCLUDED.created_at",
            &[
                &account.sfid_number,
                &p_code,
                &c_code,
                &account.account_name,
                &account.duoqian_address,
                &chain_status,
                &account.created_at,
            ],
        )
        .map_err(|e| format!("upsert accounts failed: {e}"))?;
        Ok(())
    }

    fn scope_codes_from_sfid(sfid_number: &str) -> (String, Option<String>) {
        let Some(r5) = sfid_number.split('-').next() else {
            return ("ZS".to_string(), None);
        };
        if r5.len() < 5 {
            return ("ZS".to_string(), None);
        }
        let p_code = r5[0..2].to_string();
        let c_part = &r5[2..5];
        let c_code = if c_part == "000" {
            None
        } else {
            Some(c_part.to_string())
        };
        (p_code, c_code)
    }

    pub(crate) fn delete_institution_account_row(
        &self,
        sfid_number: &str,
        account_name: &str,
    ) -> Result<(), String> {
        let sfid_number = sfid_number.to_string();
        let account_name = account_name.to_string();
        self.with_client(move |conn| {
            conn.execute(
                "DELETE FROM accounts
                 WHERE sfid_number = $1 AND account_name = $2",
                &[&sfid_number, &account_name],
            )
            .map_err(|e| format!("delete accounts failed: {e}"))?;
            Ok(())
        })
    }

    pub(crate) fn revoke_institution_rows_by_sfids(&self, sfids: &[String]) -> Result<(), String> {
        if sfids.is_empty() {
            return Ok(());
        }
        let sfids = sfids.to_vec();
        self.with_client(move |conn| {
            let mut tx = conn
                .transaction()
                .map_err(|e| format!("begin revoke subject rows failed: {e}"))?;
            for sfid in &sfids {
                tx.execute(
                    "UPDATE subjects
                     SET status = 'REVOKED', updated_at = now()
                     WHERE sfid_number = $1",
                    &[sfid],
                )
                .map_err(|e| format!("revoke subject row failed: {e}"))?;
            }
            tx.commit()
                .map_err(|e| format!("commit revoke subject rows failed: {e}"))?;
            Ok(())
        })
    }

    // 中文注释:删除全部旧格式 SFID 号在各号承载表里的残留行。旧号判定唯一标准 =
    // 过不了 `crate::number::validate_sfid_number_format`(新版 4 段 + checksum)。
    // dry_run 时在事务内删完即回滚,只回报计数,不改库。
    pub(crate) fn purge_legacy_sfid_rows(&self, dry_run: bool) -> Result<PurgeReport, String> {
        // 中文注释:号承载表清单,无外键约束,删除顺序无关;主登记表 ids 放最后。
        const SFID_TABLES: [&str; 9] = [
            "subjects",
            "citizens",
            "gov",
            "private",
            "accounts",
            "docs",
            "cpms_sites",
            "citizen_status_imports",
            "ids",
        ];
        self.with_client(move |conn| {
            // 1. 收集号全集与 kind(ids 为准,subjects 补孤儿,cpms_sites 兜底)。
            let mut kind_by_sfid: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for row in conn
                .query("SELECT sfid_number, kind FROM ids", &[])
                .map_err(|e| format!("scan ids failed: {e}"))?
            {
                kind_by_sfid.entry(row.get(0)).or_insert_with(|| row.get(1));
            }
            for row in conn
                .query("SELECT DISTINCT sfid_number, kind FROM subjects", &[])
                .map_err(|e| format!("scan subjects failed: {e}"))?
            {
                kind_by_sfid.entry(row.get(0)).or_insert_with(|| row.get(1));
            }
            for row in conn
                .query("SELECT DISTINCT sfid_number FROM cpms_sites", &[])
                .map_err(|e| format!("scan cpms_sites failed: {e}"))?
            {
                kind_by_sfid
                    .entry(row.get(0))
                    .or_insert_with(|| "PUBLIC".to_string());
            }

            // 2. 筛旧号:过不了新格式校验的即旧号。
            let legacy: Vec<String> = kind_by_sfid
                .keys()
                .filter(|sfid| crate::number::validate_sfid_number_format(sfid).is_err())
                .cloned()
                .collect();
            let private_count = legacy
                .iter()
                .filter(|sfid| kind_by_sfid.get(*sfid).map(String::as_str) == Some("PRIVATE"))
                .count();
            let citizen_count = legacy
                .iter()
                .filter(|sfid| kind_by_sfid.get(*sfid).map(String::as_str) == Some("CITIZEN"))
                .count();

            if legacy.is_empty() {
                return Ok(PurgeReport {
                    legacy_count: 0,
                    private_count: 0,
                    citizen_count: 0,
                    per_table_deleted: SFID_TABLES.iter().map(|table| (*table, 0)).collect(),
                    dry_run,
                });
            }

            // 3. 一事务内逐表删除,记录各表行数。
            let mut tx = conn
                .transaction()
                .map_err(|e| format!("begin purge legacy sfid tx failed: {e}"))?;
            let mut per_table_deleted = Vec::with_capacity(SFID_TABLES.len());
            for table in SFID_TABLES {
                let sql = format!("DELETE FROM {table} WHERE sfid_number = ANY($1)");
                let deleted = tx
                    .execute(sql.as_str(), &[&legacy])
                    .map_err(|e| format!("delete legacy sfid from {table} failed: {e}"))?;
                per_table_deleted.push((table, deleted));
            }
            if dry_run {
                tx.rollback()
                    .map_err(|e| format!("rollback purge legacy sfid dry-run failed: {e}"))?;
            } else {
                tx.commit()
                    .map_err(|e| format!("commit purge legacy sfid failed: {e}"))?;
            }

            Ok(PurgeReport {
                legacy_count: legacy.len(),
                private_count,
                citizen_count,
                per_table_deleted,
                dry_run,
            })
        })
    }

    pub(crate) fn list_institutions_exact(
        &self,
        filter: crate::subjects::InstitutionListFilter,
        p_code: &str,
        c_code: Option<&str>,
        keyword: &str,
        cursor: Option<&str>,
        page_size: usize,
    ) -> Result<PageResult<crate::subjects::InstitutionListRow>, String> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(PageResult {
                items: Vec::new(),
                page_size,
                next_cursor: None,
                has_more: false,
                manifest_version: None,
                catalog_status: None,
            });
        }
        let cursor = decode_db_page_cursor(cursor)?;
        let p_code = p_code.to_string();
        let c_code = c_code.map(str::to_string);
        let keyword = keyword.to_string();
        self.with_client(move |conn| {
            let cursor_created_at = cursor.map(|c| c.created_at);
            let fetch_limit = i64::try_from(page_size.saturating_add(1))
                .map_err(|_| "page_size too large".to_string())?;
            // 中文注释:过滤子句来自 InstitutionListFilter 的静态字面量(教育=手动 JY 行,
            // 非法人按父级属性分流到公权/私权),无注入面;par 是子句依赖的父级别名,
            // 父级允许跨省(私法人全国),故只按 sfid_number 关联不限定 p_code。
            let sql = format!(
		                    "SELECT s.sfid_number, s.name, s.category,
			                                    s.subject_property, s.p1, s.province,
			                                    s.city, s.province_code, s.city_code, s.institution_code,
				                                    s.sub_type, s.parent_sfid_number,
					                                    s.created_by, s.created_at, COALESCE(ac.account_count, 0),
				                                    a.admin_name, a.role, s.sfid_name, s.short_name,
				                                    COALESCE(s.town, ''), COALESCE(s.town_code, ''), s.org_code,
				                                    s.status
		                             FROM subjects s
		                             LEFT JOIN gov g ON g.p_code = s.p_code AND g.sfid_number = s.sfid_number
		                             LEFT JOIN subjects par ON par.sfid_number = s.parent_sfid_number
		                             LEFT JOIN (
	                                SELECT p_code, sfid_number, COUNT(*)::BIGINT AS account_count
	                                FROM accounts
	                                WHERE p_code = $1
	                                  AND ($2::text IS NULL OR c_code = $2)
	                                GROUP BY p_code, sfid_number
	                             ) ac ON ac.p_code = s.p_code AND ac.sfid_number = s.sfid_number
	                             LEFT JOIN admins a ON lower(a.admin_pubkey) = lower(s.created_by)
	                             WHERE s.kind IN ('PUBLIC', 'PRIVATE')
	                               {filter_clause}
	                               AND s.p_code = $1
	                               AND ($2::text IS NULL OR s.c_code = $2)
	                               AND (
	                                    s.sfid_number = $3
	                                    OR lower(COALESCE(s.name, '')) = lower($3)
	                               )
	                               AND (
	                                    $4::timestamptz IS NULL
	                                    OR s.created_at < $4
	                               )
	                             ORDER BY s.created_at DESC, s.sfid_number DESC
	                             LIMIT $5",
                filter_clause = filter.sql_clause(),
            );
            let rows = conn
                .query(
                    sql.as_str(),
                    &[
                        &p_code,
                        &c_code,
                        &keyword,
                        &cursor_created_at,
                        &fetch_limit,
                    ],
                )
                .map_err(|e| format!("query subjects failed: {e}"))?;
            let mut output = Vec::with_capacity(rows.len());
            for row in rows {
                let category_text: String = row.get(2);
                let category = institution_category_from_text(category_text.as_str())
                    .ok_or_else(|| format!("invalid institution category: {category_text}"))?;
			                let account_count_i64: i64 = row.get(14);
			                let created_by_name: Option<String> = row.get(15);
			                let created_by_role: Option<String> = row.get(16);
			                let sfid_name: Option<String> = row.get(17);
			                let short_name: Option<String> = row.get(18);
				                let town: Option<String> = row.get(19);
				                let town_code: Option<String> = row.get(20);
				                let org_code: Option<String> = row.get(21);
				                let status: String = row.get(22);
		                let inst = crate::subjects::Institution {
		                    sfid_number: row.get(0),
		                    institution_name: row.get(1),
		                    sfid_name,
			                    short_name,
			                    status,
			                    category,
		                    subject_property: row.get(3),
		                    p1: row.get(4),
			                    province: row.get(5),
			                    city: row.get(6),
			                    town: town.unwrap_or_default(),
			                    province_code: row.get(7),
			                    city_code: row.get(8),
			                    town_code: town_code.unwrap_or_default(),
			                    institution_code: row.get(9),
			                    org_code,
			                    sub_type: row.get(10),
		                    parent_sfid_number: row.get(11),
                    legal_rep_name: None,
                    legal_rep_sfid_number: None,
                    legal_rep_photo_path: None,
                    legal_rep_photo_name: None,
                    legal_rep_photo_mime: None,
                    legal_rep_photo_size: None,
		                    created_by: row.get(12),
		                    created_at: row.get(13),
                };
                let id = stable_institution_cursor_id(inst.sfid_number.as_str());
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

    pub(crate) fn list_public_security_scope(
        &self,
        p_code: &str,
        c_code: Option<&str>,
        offset: usize,
        page_size: usize,
    ) -> Result<PageResult<crate::subjects::InstitutionListRow>, String> {
        let p_code = p_code.to_string();
        let c_code = c_code.map(str::to_string);
        self.with_client(move |conn| {
            let limit = i64::try_from(page_size.saturating_add(1))
                .map_err(|_| "page_size too large".to_string())?;
            let offset_i64 =
                i64::try_from(offset).map_err(|_| "page offset too large".to_string())?;
            let rows = conn
                .query(
		                    "SELECT s.sfid_number, s.name, s.category,
			                                    s.subject_property, s.p1, s.province,
			                                    s.city, s.province_code, s.city_code, s.institution_code,
				                                    s.sub_type, s.parent_sfid_number,
					                                    s.created_by, s.created_at, COALESCE(ac.account_count, 0),
			                                    a.admin_name, a.role, s.sfid_name, s.short_name,
			                                    COALESCE(s.town, ''), COALESCE(s.town_code, ''), s.org_code,
			                                    s.status, cs.status, cs.install_token_status,
			                                    CASE WHEN cs.sfid_number IS NULL THEN NULL ELSE (cs.cpms_pubkey_hash IS NOT NULL) END
	                             FROM subjects s
	                             JOIN gov g ON g.p_code = s.p_code AND g.sfid_number = s.sfid_number
		                             LEFT JOIN (
		                                SELECT p_code, sfid_number, COUNT(*)::BIGINT AS account_count
	                                FROM accounts
	                                WHERE p_code = $1
	                                  AND ($2::text IS NULL OR c_code = $2)
	                                GROUP BY p_code, sfid_number
		                             ) ac ON ac.p_code = s.p_code AND ac.sfid_number = s.sfid_number
		                             LEFT JOIN admins a ON lower(a.admin_pubkey) = lower(s.created_by)
		                             LEFT JOIN cpms_sites cs ON cs.sfid_number = s.sfid_number
	                             WHERE s.kind = 'PUBLIC'
	                               AND s.category = 'PUBLIC_SECURITY'
	                               AND s.status = 'ACTIVE'
	                               AND g.org_code = 'CITY_POLICE'
	                               AND s.c_code IS NOT NULL
	                               AND s.p_code = $1
	                               AND ($2::text IS NULL OR s.c_code = $2)
	                             ORDER BY s.c_code ASC NULLS LAST, s.sfid_number ASC
	                             LIMIT $3 OFFSET $4",
                    &[&p_code, &c_code, &limit, &offset_i64],
                )
                .map_err(|e| format!("query public security failed: {e}"))?;
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                items.push(institution_row_from_pg_row(&row)?);
            }
            Ok(offset_page_from_window(items, offset, page_size))
        })
    }

    pub(crate) fn list_official_institutions_scope(
        &self,
        p_code: &str,
        c_code: Option<&str>,
        keyword: &str,
        offset: usize,
        page_size: usize,
    ) -> Result<PageResult<crate::subjects::InstitutionListRow>, String> {
        let keyword = keyword.trim().to_ascii_lowercase();
        let p_code = p_code.to_string();
        let c_code = c_code.map(str::to_string);
        self.with_client(move |conn| {
            let limit = i64::try_from(page_size.saturating_add(1))
                .map_err(|_| "page_size too large".to_string())?;
            let offset_i64 =
                i64::try_from(offset).map_err(|_| "page offset too large".to_string())?;
            // 中文注释:公权目录 = 自动生成目录(gov 表,排公安局) + 手动公权机构
            // (category=GOV,org_code 空,非 JY 学校) + 公权下属非法人(F,父级为公法人)。
            // 父级只按 sfid_number 关联(sfid 全局唯一,父级不限定本省分区)。
            let rows = conn
                .query(
                    "SELECT s.sfid_number, s.name, s.category,
			                                    s.subject_property, s.p1, s.province,
			                                    s.city, s.province_code, s.city_code, s.institution_code,
				                                    s.sub_type, s.parent_sfid_number,
				                                    s.created_by, s.created_at, COALESCE(ac.account_count, 0),
			                                    a.admin_name, a.role, s.sfid_name, s.short_name,
			                                    COALESCE(s.town, ''), COALESCE(s.town_code, ''), s.org_code,
			                                    s.status, NULL::text, NULL::text, NULL::boolean
	                             FROM subjects s
	                             LEFT JOIN gov g ON g.p_code = s.p_code AND g.sfid_number = s.sfid_number
	                             LEFT JOIN subjects par ON par.sfid_number = s.parent_sfid_number
		                             LEFT JOIN (
	                                SELECT p_code, sfid_number, COUNT(*)::BIGINT AS account_count
	                                FROM accounts
	                                WHERE p_code = $1
	                                  AND ($2::text IS NULL OR c_code = $2)
	                                GROUP BY p_code, sfid_number
	                             ) ac ON ac.p_code = s.p_code AND ac.sfid_number = s.sfid_number
	                             LEFT JOIN admins a ON lower(a.admin_pubkey) = lower(s.created_by)
	                             WHERE s.kind IN ('PUBLIC', 'PRIVATE')
	                               AND s.status = 'ACTIVE'
	                               AND (
	                                    (s.category = 'GOV_INSTITUTION'
	                                     AND g.sfid_number IS NOT NULL
	                                     AND COALESCE(g.org_code, '') <> 'CITY_POLICE')
	                                    OR (s.category = 'GOV_INSTITUTION'
	                                        AND g.sfid_number IS NULL
	                                        AND s.org_code IS NULL
	                                        AND s.institution_code <> 'JY')
	                                    OR (s.subject_property = 'F'
	                                        AND s.institution_code <> 'JY'
	                                        AND par.subject_property = 'G')
	                               )
	                               AND s.p_code = $1
	                               AND ($2::text IS NULL OR s.c_code = $2)
	                               AND (
	                                    $3::text = ''
	                                    OR lower(s.sfid_number) LIKE '%' || $3 || '%'
	                                    OR lower(COALESCE(s.name, '')) LIKE '%' || $3 || '%'
	                               )
	                             ORDER BY
		                                s.c_code ASC NULLS LAST,
		                                s.t_code ASC NULLS LAST,
		                                CASE s.institution_code
	                                    WHEN 'ZF' THEN 0
	                                    WHEN 'LF' THEN 1
	                                    WHEN 'SF' THEN 2
	                                    WHEN 'JC' THEN 3
	                                    WHEN 'JY' THEN 4
	                                    ELSE 9
	                                END ASC,
	                                COALESCE(s.name, '') ASC,
	                                s.sfid_number ASC
	                             LIMIT $4 OFFSET $5",
                    &[&p_code, &c_code, &keyword, &limit, &offset_i64],
                )
                .map_err(|e| format!("query official institutions failed: {e}"))?;
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                items.push(institution_row_from_pg_row(&row)?);
            }
            Ok(offset_page_from_window(items, offset, page_size))
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

#[derive(Debug, Clone)]
enum BackendCommand {
    Serve,
    EnsureGov,
    InitGov,
    CheckGov {
        strict: bool,
    },
    ReconcileGovChanged,
    ReconcileGovProvince {
        province_code: String,
    },
    ReconcileGovCity {
        province_code: String,
        city_code: String,
    },
    PurgeLegacySfid {
        dry_run: bool,
    },
}

fn parse_backend_command() -> BackendCommand {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        return BackendCommand::Serve;
    };
    match command {
        "serve" => BackendCommand::Serve,
        "ensure-gov" => BackendCommand::EnsureGov,
        "init-gov" => BackendCommand::InitGov,
        "check-gov" => BackendCommand::CheckGov {
            strict: args.iter().any(|arg| arg == "--strict"),
        },
        "reconcile-gov" => {
            if args.iter().any(|arg| arg == "--changed-only") {
                BackendCommand::ReconcileGovChanged
            } else {
                panic!("reconcile-gov requires --changed-only");
            }
        }
        "reconcile-gov-province" => {
            let province_code = parse_cli_option(&args, "--province")
                .unwrap_or_else(|| panic!("reconcile-gov-province requires --province <code>"));
            BackendCommand::ReconcileGovProvince { province_code }
        }
        "reconcile-gov-city" => {
            let province_code = parse_cli_option(&args, "--province")
                .unwrap_or_else(|| panic!("reconcile-gov-city requires --province <code>"));
            let city_code = parse_cli_option(&args, "--city")
                .unwrap_or_else(|| panic!("reconcile-gov-city requires --city <code>"));
            BackendCommand::ReconcileGovCity {
                province_code,
                city_code,
            }
        }
        "purge-legacy-sfid" => BackendCommand::PurgeLegacySfid {
            dry_run: args.iter().any(|arg| arg == "--dry-run"),
        },
        other => panic!("unknown sfid-backend command: {other}"),
    }
}

fn parse_cli_option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].trim().to_string())
        .filter(|v| !v.is_empty())
}

#[derive(Debug, Clone, Copy)]
struct GovBootstrapState {
    manifest_present: bool,
    manifest_ready: bool,
    target_count: i64,
    subject_count: i64,
    gov_count: i64,
    account_count: i64,
}

fn gov_bootstrap_state_ready(state: &GovBootstrapState) -> bool {
    state.manifest_ready
        && state.target_count > 0
        && state.subject_count >= state.target_count
        && state.gov_count >= state.target_count
        && state.account_count >= state.target_count * crate::gov::service::DEFAULT_ACCOUNT_COUNT
}

fn gov_bootstrap_state_summary(state: &GovBootstrapState) -> String {
    format!(
        "manifest_present={}, manifest_ready={}, target_count={}, subjects={}, gov={}, accounts={}",
        state.manifest_present,
        state.manifest_ready,
        state.target_count,
        state.subject_count,
        state.gov_count,
        state.account_count
    )
}

fn load_gov_bootstrap_state(state: &AppState) -> Result<GovBootstrapState, String> {
    use crate::gov::service::{gov_manifest_key, GovTargetKind, OfficialReconcileScope};

    let scope_key = gov_manifest_key(&OfficialReconcileScope::All, GovTargetKind::All);
    state.db.with_client(move |conn| {
        let row = conn
            .query_one(
                "SELECT
                    EXISTS(SELECT 1 FROM gov_manifest WHERE scope_key = $1) AS manifest_present,
                    COALESCE((
                        SELECT status = 'OK' AND template_version = $2 AND target_count > 0
                        FROM gov_manifest
                        WHERE scope_key = $1
                        ORDER BY updated_at DESC
                        LIMIT 1
                    ), false) AS manifest_ready,
                    COALESCE((
                        SELECT target_count
                        FROM gov_manifest
                        WHERE scope_key = $1
                        ORDER BY updated_at DESC
                        LIMIT 1
                    ), 0)::BIGINT AS target_count,
                    (SELECT COUNT(*)::BIGINT FROM subjects WHERE kind = 'PUBLIC' AND status = 'ACTIVE') AS subject_count,
                    (SELECT COUNT(*)::BIGINT FROM gov) AS gov_count,
                    (
                        SELECT COUNT(*)::BIGINT
                        FROM accounts a
                        JOIN gov g ON g.p_code = a.p_code AND g.sfid_number = a.sfid_number
                    ) AS account_count",
                &[&scope_key, &crate::gov::service::GOV_TEMPLATE_VERSION],
            )
            .map_err(|e| {
                format!(
                    "query gov bootstrap state failed: {}",
                    crate::core::db::postgres_error_text(&e)
                )
            })?;
        Ok(GovBootstrapState {
            manifest_present: row.get(0),
            manifest_ready: row.get(1),
            target_count: row.get(2),
            subject_count: row.get(3),
            gov_count: row.get(4),
            account_count: row.get(5),
        })
    })
}

fn run_ensure_gov_command(state: &AppState) -> Result<(), String> {
    use crate::gov::service::{
        check_gov_catalog_db, upsert_gov_manifest_from_check_db, GovTargetKind,
        OfficialReconcileScope,
    };

    let lock_sql = "SELECT pg_advisory_lock(hashtext('sfid'), hashtext('ensure-gov'))";
    let unlock_sql = "SELECT pg_advisory_unlock(hashtext('sfid'), hashtext('ensure-gov'))";

    // 中文注释:部署脚本可能被多实例同时执行,PostgreSQL 会话锁保证只有一个进程做目录初始化。
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is required".to_string())?;
    let mut lock_conn =
        postgres::Client::connect(database_url.as_str(), postgres::NoTls).map_err(|e| {
            format!(
                "connect postgres for gov ensure lock failed: {}",
                crate::core::db::postgres_error_text(&e)
            )
        })?;
    lock_conn.batch_execute(lock_sql).map_err(|e| {
        format!(
            "acquire gov ensure lock failed: {}",
            crate::core::db::postgres_error_text(&e)
        )
    })?;

    let result = (|| {
        let before = load_gov_bootstrap_state(state)?;
        info!(
            manifest_present = before.manifest_present,
            manifest_ready = before.manifest_ready,
            target_count = before.target_count,
            subjects = before.subject_count,
            gov = before.gov_count,
            accounts = before.account_count,
            "sfid gov directory bootstrap state checked"
        );

        if gov_bootstrap_state_ready(&before) {
            info!("sfid gov directory already initialized; ensure-gov skipped");
            return Ok(());
        }

        if before.subject_count > 0 || before.gov_count > 0 || before.account_count > 0 {
            let check =
                check_gov_catalog_db(&state.db, OfficialReconcileScope::All, GovTargetKind::All)?;
            info!(
                ok = check.ok,
                target_count = check.target_count,
                active_count = check.active_count,
                missing = check.missing_sfids.len(),
                mismatched = check.mismatched_sfids.len(),
                missing_accounts = check.missing_account_sfids.len(),
                obsolete = check.obsolete_sfids.len(),
                "sfid gov directory existing data checked before ensure rewrite"
            );
            if check.ok {
                upsert_gov_manifest_from_check_db(&state.db, &check)?;
                info!("sfid gov directory manifest repaired by ensure-gov");
                return Ok(());
            }
        }

        let report = core::runtime_ops::reconcile_official_institutions_explicit(
            state,
            OfficialReconcileScope::All,
            true,
        )?;
        let after = load_gov_bootstrap_state(state)?;
        if !gov_bootstrap_state_ready(&after) {
            return Err(format!(
                "gov directory remains incomplete after ensure-gov: {}",
                gov_bootstrap_state_summary(&after)
            ));
        }

        info!(
            inserted = report.inserted,
            updated = report.updated,
            account_inserted = report.account_inserted,
            removed = report.removed,
            total_after = report.total_after,
            touched = report.touched_sfids.len(),
            targets = report.target_sfids.len(),
            "sfid gov directory initialized by ensure-gov"
        );
        Ok(())
    })();

    let unlock_result = lock_conn.batch_execute(unlock_sql).map_err(|e| {
        format!(
            "release gov ensure lock failed: {}",
            crate::core::db::postgres_error_text(&e)
        )
    });
    if result.is_ok() {
        unlock_result?;
    }
    result
}

fn run_gov_directory_command(state: &AppState, command: BackendCommand) -> bool {
    use crate::gov::service::{
        check_gov_catalog_db, reconcile_changed_gov_catalog_db, GovTargetKind,
        OfficialReconcileScope,
    };

    let (scope, force_row_sync, label) = match command {
        BackendCommand::Serve => return false,
        BackendCommand::EnsureGov => {
            run_ensure_gov_command(state).unwrap_or_else(|e| panic!("ensure-gov failed: {e}"));
            return true;
        }
        BackendCommand::InitGov => (OfficialReconcileScope::All, true, "init-gov"),
        BackendCommand::CheckGov { strict } => {
            let report =
                check_gov_catalog_db(&state.db, OfficialReconcileScope::All, GovTargetKind::All)
                    .unwrap_or_else(|e| panic!("check-gov failed: {e}"));
            info!(
                ok = report.ok,
                target_count = report.target_count,
                active_count = report.active_count,
                missing = report.missing_sfids.len(),
                mismatched = report.mismatched_sfids.len(),
                missing_accounts = report.missing_account_sfids.len(),
                obsolete = report.obsolete_sfids.len(),
                catalog_hash = report.catalog_hash,
                "sfid gov directory check finished"
            );
            if strict && !report.ok {
                panic!("check-gov --strict failed: deterministic gov directory is incomplete");
            }
            return true;
        }
        BackendCommand::ReconcileGovChanged => {
            let reports = reconcile_changed_gov_catalog_db(&state.db, "SYSTEM")
                .unwrap_or_else(|e| panic!("reconcile-gov --changed-only failed: {e}"));
            info!(
                scopes = reports.len(),
                inserted = reports.iter().map(|r| r.inserted).sum::<usize>(),
                updated = reports.iter().map(|r| r.updated).sum::<usize>(),
                account_inserted = reports.iter().map(|r| r.account_inserted).sum::<usize>(),
                removed = reports.iter().map(|r| r.removed).sum::<usize>(),
                "sfid changed gov directory reconcile finished"
            );
            return true;
        }
        BackendCommand::PurgeLegacySfid { dry_run } => {
            run_purge_legacy_sfid(state, dry_run);
            return true;
        }
        BackendCommand::ReconcileGovProvince { province_code } => (
            OfficialReconcileScope::Province { province_code },
            false,
            "reconcile-gov-province",
        ),
        BackendCommand::ReconcileGovCity {
            province_code,
            city_code,
        } => (
            OfficialReconcileScope::City {
                province_code,
                city_code,
            },
            false,
            "reconcile-gov-city",
        ),
    };
    let report =
        core::runtime_ops::reconcile_official_institutions_explicit(state, scope, force_row_sync)
            .unwrap_or_else(|e| panic!("{label} failed: {e}"));
    info!(
        command = label,
        inserted = report.inserted,
        updated = report.updated,
        account_inserted = report.account_inserted,
        removed = report.removed,
        total_after = report.total_after,
        touched = report.touched_sfids.len(),
        targets = report.target_sfids.len(),
        "sfid gov directory command finished"
    );
    true
}

#[derive(Debug)]
struct PurgeReport {
    legacy_count: usize,
    private_count: usize,
    citizen_count: usize,
    per_table_deleted: Vec<(&'static str, u64)>,
    dry_run: bool,
}

// 中文注释:清掉所有旧格式 SFID 号(身份ID系统重构前入库的残留),再把能确定性
// 自动重建的公权机构(含公安局)按新号重对账。PRIVATE 私权机构与公民属用户创建/
// 钱包绑定,删后无法自动重建,需由用户重建/重绑。链端与 CPMS host 不在本命令范围。
fn run_purge_legacy_sfid(state: &AppState, dry_run: bool) {
    let report = state
        .db
        .purge_legacy_sfid_rows(dry_run)
        .unwrap_or_else(|e| panic!("purge-legacy-sfid failed: {e}"));
    let per_table = report
        .per_table_deleted
        .iter()
        .map(|(table, count)| format!("{table}={count}"))
        .collect::<Vec<_>>()
        .join(", ");
    info!(
        dry_run = report.dry_run,
        legacy_count = report.legacy_count,
        private_permanently_deleted = report.private_count,
        citizen_permanently_deleted = report.citizen_count,
        per_table = %per_table,
        "sfid legacy purge finished"
    );
    if report.dry_run {
        info!("purge-legacy-sfid dry-run: no rows changed; re-run without --dry-run to apply");
        return;
    }
    if report.legacy_count == 0 {
        info!("purge-legacy-sfid: no legacy sfid rows; skip reconcile");
        return;
    }
    if report.private_count > 0 || report.citizen_count > 0 {
        warn!(
            private_permanently_deleted = report.private_count,
            citizen_permanently_deleted = report.citizen_count,
            "purge-legacy-sfid removed user-created PRIVATE/CITIZEN rows; they must be re-created/re-bound to get new-scheme sfid"
        );
    }
    // 中文注释:build_raw_targets(GovTargetKind::All) 同时含公权机构与公安局,
    // 一次全量重对账即把所有 PUBLIC 主体按新号重建。
    let recon = core::runtime_ops::reconcile_official_institutions_explicit(
        state,
        crate::gov::service::OfficialReconcileScope::All,
        true,
    )
    .unwrap_or_else(|e| panic!("purge-legacy-sfid reconcile failed: {e}"));
    info!(
        inserted = recon.inserted,
        updated = recon.updated,
        account_inserted = recon.account_inserted,
        removed = recon.removed,
        total_after = recon.total_after,
        "sfid public institutions reconciled with new scheme"
    );
}

fn chain_genesis_source_configured() -> bool {
    std::env::var("SFID_CHAIN_GENESIS_HASH")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        || core::chain_url::chain_ws_url().is_ok()
}

// 中文注释:链节点是 SFID 的外部联调依赖,不能阻塞后端、管理员和 CPMS 基础服务启动。
async fn cache_chain_genesis_hash_until_ready() {
    let mut retry_secs = 2u64;
    loop {
        match core::chain_runtime::init_genesis_hash_from_chain().await {
            Ok(()) => {
                info!("chain genesis hash initialized");
                return;
            }
            Err(err) => {
                warn!(
                    error = %err,
                    retry_in = retry_secs,
                    "chain genesis hash unavailable; sfid backend continues without chain"
                );
            }
        }
        tokio::time::sleep(Duration::from_secs(retry_secs)).await;
        retry_secs = (retry_secs * 2).min(60);
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();
    disable_core_dumps();
    let command = parse_backend_command();

    let redis_url = required_env("SFID_REDIS_URL");
    let redis_client = RedisClient::open(redis_url.as_str())
        .unwrap_or_else(|e| panic!("invalid SFID_REDIS_URL: {e}"));

    // 中文注释:启动期仅校验 SFID_SIGNING_SEED_HEX 可解码,供登录二维码系统签名使用。
    // 联邦管理员业务治理签名只走各自冷钱包,后端不再保存或缓存省级私钥。
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
    let db = Db::from_database_url(database_url.as_str()).expect("init database");
    let state = AppState {
        db,
        rate_limit_redis: Arc::new(redis_client),
    };
    ensure_builtin_province_admins(&state);
    info!("initialized database state with defaults");
    if run_gov_directory_command(&state, command.clone()) {
        return;
    }
    // 中文注释:普通公权/宪法机构目录是持久化数据,正常启动只读数据库,
    // 不再于健康检查前执行全量生成和逐条 upsert。
    core::runtime_ops::cleanup_stale_citizen_bind_records(&state);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    let indexer_db = match core::chain_url::chain_ws_url() {
        Ok(_) => Some(state.db.clone()),
        Err(err) => {
            warn!("indexer disabled: {err}");
            None
        }
    };
    runtime.block_on(async move {
        if let Some(db) = indexer_db {
            tokio::spawn(indexer::indexer_worker(db));
        }
        if chain_genesis_source_configured() {
            tokio::spawn(cache_chain_genesis_hash_until_ready());
        } else {
            warn!("chain genesis hash source not configured; sfid backend continues without chain");
        }

        let auth_routes = Router::new()
            .route(
                "/api/v1/admin/auth/check",
                get(admins::login::admin_auth_check),
            )
            .route(
                "/api/v1/admin/auth/logout",
                post(admins::login::admin_logout),
            )
            .route(
                "/api/v1/admin/auth/identify",
                post(admins::login::admin_auth_identify),
            )
            .route(
                "/api/v1/admin/auth/challenge",
                post(admins::login::admin_auth_challenge),
            )
            .route(
                "/api/v1/admin/auth/verify",
                post(admins::login::admin_auth_verify),
            )
            .route(
                "/api/v1/admin/auth/qr/challenge",
                post(admins::login::admin_auth_qr_challenge),
            )
            .route(
                "/api/v1/admin/auth/qr/complete",
                post(admins::login::admin_auth_qr_complete),
            )
            .route(
                "/api/v1/admin/auth/qr/result",
                get(admins::login::admin_auth_qr_result),
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
            // 中文注释:机构相关 API 外部路径保持稳定,内部按 subjects/gov/private/accounts/docs 归属。
            // - GET  /api/v1/institution/check-name                      — 机构名称全国查重
            // - POST /api/v1/institution/create                          — 生成机构(不上链)
            // - POST /api/v1/institution/:sfid_number/account/create         — 只登记账户名称,不上链
            // - GET  /api/v1/institution/list                            — 按 scope 过滤的机构列表
            // - GET  /api/v1/institution/:sfid_number                        — 机构详情
            // - GET  /api/v1/institution/:sfid_number/accounts               — 账户列表
            // - DELETE /api/v1/institution/:sfid_number/account/:account_name — 删除未上链/已注销新增账户
            .route(
                "/api/v1/institution/check-name",
                get(subjects::admin::check_institution_name),
            )
            // F 详情页"所属法人"搜索(全国范围 S/G 模糊匹配)
            .route(
                "/api/v1/institution/search-parents",
                get(subjects::admin::search_parent_institutions),
            )
            .route(
                "/api/v1/institution/legal-representative/photo",
                post(subjects::admin::upload_legal_representative_photo),
            )
            .route(
                "/api/v1/institution/create",
                post(private::handler::create_institution),
            )
            .route(
                "/api/v1/institution/:sfid_number/account/create",
                post(accounts::handler::create_account),
            )
            .route(
                "/api/v1/institution/list",
                get(private::handler::list_institutions),
            )
            .route(
                "/api/v1/institution/:sfid_number",
                get(subjects::admin::get_institution)
                    // 两步式第二步:详情页更新机构名称/企业类型
                    .patch(subjects::admin::update_institution),
            )
            .route(
                "/api/v1/institution/:sfid_number/accounts",
                get(accounts::handler::list_accounts),
            )
            .route(
                "/api/v1/institution/:sfid_number/account/:account_name",
                delete(accounts::handler::delete_account),
            )
            // 机构资料库文档 CRUD
            .route(
                "/api/v1/institution/:sfid_number/documents",
                get(docs::handler::list_documents).post(docs::handler::upload_document),
            )
            .route(
                "/api/v1/institution/:sfid_number/documents/:doc_id/download",
                get(docs::handler::download_document),
            )
            .route(
                "/api/v1/institution/:sfid_number/documents/:doc_id",
                delete(docs::handler::delete_document),
            )
            // 任务卡 6:公安局跟 sfid 工具市清单对账
            .route(
                "/api/v1/public-security/reconcile",
                post(gov::handler::reconcile_public_security),
            )
            .route(
                "/api/v1/institutions/public-security",
                get(gov::handler::list_public_security_institutions),
            )
            .route(
                "/api/v1/institutions/official",
                get(gov::handler::list_official_institutions),
            )
            // 联邦注册局机构详情(只读,绕过 scope,所有省管理员可读)
            .route(
                "/api/v1/institutions/federal-registry",
                get(subjects::admin::get_federal_registry),
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
            .route(
                "/api/v1/admin/citizens/legal-representatives",
                get(citizens::handler::admin_search_legal_representative_citizens),
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
            .route(
                "/api/v1/admin/number/meta",
                get(number::admin::admin_number_meta),
            )
            .route(
                "/api/v1/admin/china/cities",
                get(china::admin::admin_china_cities),
            )
            .route(
                "/api/v1/admin/china/towns",
                get(china::admin::admin_china_towns),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                admins::login::require_admin_session_middleware,
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
                get(subjects::chain_duoqian_info::app_search_institutions),
            )
            .route(
                "/api/v1/app/institutions/:sfid_number/registration-info",
                get(subjects::chain_duoqian_info::app_get_institution_registration_info),
            )
            .route(
                "/api/v1/app/institutions/:sfid_number",
                get(subjects::chain_duoqian_info::app_get_institution),
            )
            .route(
                "/api/v1/app/institutions/:sfid_number/accounts",
                get(subjects::chain_duoqian_info::app_list_accounts),
            )
            // ── 清算行搜索(已激活,wuminapp 绑定清算行用):资格白名单 + 主账户 ACTIVE_ON_CHAIN ──
            .route(
                "/api/v1/app/clearing-banks/search",
                get(subjects::chain_duoqian_info::app_search_clearing_banks),
            )
            // ── 候选清算行搜索(可未激活,节点桌面"添加清算行"用):仅资格白名单过滤 ──
            .route(
                "/api/v1/app/clearing-banks/eligible-search",
                get(subjects::chain_duoqian_info::app_search_eligible_clearing_banks),
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

        // 中文注释:Passkey 绑定必须受 WebAuthn RP ID / Origin 约束;
        // 生产环境在启动期强制校验为 sfid.crcfrcn.com。
        admins::passkeys::validate_passkey_configuration()
            .unwrap_or_else(|e| panic!("invalid SFID Passkey configuration: {e}"));
        info!("passkey webauthn configuration validated");

        // 中文注释:联邦管理员采用同级模型;43 个初始联邦管理员只作为
        // 不可删除安全根,新增联邦管理员走 admins 安全动作落本地管理表。

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
        "admin pubkey already exists as sheng admin" => "SFID_ADMIN_PUBKEY_EXISTS_AS_FEDERAL_ADMIN",
        "admin pubkey already exists as shi admin" => "SFID_ADMIN_PUBKEY_EXISTS_AS_SHI_ADMIN",
        "sheng admin province limit reached" => "SFID_ADMIN_FEDERAL_ADMIN_PROVINCE_LIMIT_REACHED",
        "shi admin city limit reached" => "SFID_ADMIN_SHI_ADMIN_CITY_LIMIT_REACHED",
        "passkey required" => "SFID_ADMIN_PASSKEY_REQUIRED",
        "security grant required" => "SFID_ADMIN_SECURITY_GRANT_REQUIRED",
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
