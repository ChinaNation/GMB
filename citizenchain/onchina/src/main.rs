use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use base64::Engine as _;
use chrono::{DateTime, Utc};
use postgres::config::Host;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tracing::{info, warn};
use uuid::Uuid;

mod audit;
mod auth;
mod cid;
mod citizenapp;
mod core;
mod crypto;
mod domains;
mod indexer;
mod institution;
mod platform;
mod scope;
mod workspace;

#[cfg(test)]
mod genesis {
    // CID 测试编译会加载 citizenchain 的 china_ch 常量测试,
    // 该测试只需要创世人口常量来校验省储行人口总和。
    pub const GENESIS_CITIZEN_MAX: u64 = 1_443_497_378;
}

pub(crate) use crate::core::http_security::*;
pub(crate) use crate::core::response::*;
pub(crate) use crate::core::{db::Db, secret::SensitiveSeed};
pub(crate) use auth::login::{parse_sr25519_pubkey, parse_sr25519_pubkey_bytes, require_admin_any};
pub(crate) use auth::model::*;
pub(crate) use cid::model::*;
pub(crate) use domains::citizens::model::*;

#[derive(Clone)]
struct AppState {
    db: Db,
    rate_limiter: Arc<LocalRateLimiter>,
}

#[derive(Serialize)]
struct OrganizationCaCertificateInfoView {
    filename: &'static str,
    sha256: String,
    subject: &'static str,
    valid_until: String,
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

fn institution_category_text(category: crate::cid::InstitutionCategory) -> &'static str {
    match category {
        crate::cid::InstitutionCategory::GovInstitution => "GOV_INSTITUTION",
        crate::cid::InstitutionCategory::PrivateInstitution => "PRIVATE_INSTITUTION",
    }
}

fn institution_category_from_text(category: &str) -> Option<crate::cid::InstitutionCategory> {
    match category {
        "GOV_INSTITUTION" => Some(crate::cid::InstitutionCategory::GovInstitution),
        "PRIVATE_INSTITUTION" => Some(crate::cid::InstitutionCategory::PrivateInstitution),
        _ => None,
    }
}

fn multisig_chain_status_text(
    status: &crate::institution::subjects::MultisigChainStatus,
) -> &'static str {
    match status {
        crate::institution::subjects::MultisigChainStatus::NotOnChain => "NOT_ON_CHAIN",
        crate::institution::subjects::MultisigChainStatus::PendingOnChain => "PENDING_ON_CHAIN",
        crate::institution::subjects::MultisigChainStatus::ActiveOnChain => "ACTIVE_ON_CHAIN",
        crate::institution::subjects::MultisigChainStatus::RevokedOnChain => "REVOKED_ON_CHAIN",
    }
}

fn multisig_chain_status_from_text(
    status: &str,
) -> crate::institution::subjects::MultisigChainStatus {
    match status {
        "PENDING_ON_CHAIN" => crate::institution::subjects::MultisigChainStatus::PendingOnChain,
        "ACTIVE_ON_CHAIN" => crate::institution::subjects::MultisigChainStatus::ActiveOnChain,
        "REVOKED_ON_CHAIN" => crate::institution::subjects::MultisigChainStatus::RevokedOnChain,
        _ => crate::institution::subjects::MultisigChainStatus::NotOnChain,
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

fn citizen_region_names(
    province_code: Option<&str>,
    city_code: Option<&str>,
    town_code: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(province_code) = province_code.map(str::trim).filter(|code| !code.is_empty()) else {
        return (None, None, None);
    };
    crate::cid::china::area_name_by_codes(province_code, city_code, town_code)
        .map(|(province, city, town)| {
            (
                Some(province.to_string()),
                city.map(str::to_string),
                town.map(str::to_string),
            )
        })
        .unwrap_or((None, None, None))
}

fn citizen_row_from_record(record: &CitizenRecord) -> CitizenRow {
    let (province_name, city_name, town_name) = citizen_region_names(
        Some(record.province_code.as_str()),
        Some(record.city_code.as_str()),
        Some(record.town_code.as_str()),
    );
    let (birth_province_name, birth_city_name, birth_town_name) = citizen_region_names(
        Some(record.birth_province_code.as_str()),
        Some(record.birth_city_code.as_str()),
        Some(record.birth_town_code.as_str()),
    );
    CitizenRow {
        id: record.id,
        cid_number: record.cid_number.clone(),
        passport_no: record.passport_no.clone(),
        citizen_family_name: record.citizen_family_name.clone(),
        citizen_given_name: record.citizen_given_name.clone(),
        citizen_sex: record.citizen_sex.clone(),
        citizen_birth_date: record.citizen_birth_date.clone(),
        wallet_address: record.wallet_address.clone(),
        citizen_status: record.citizen_status.clone(),
        voting_eligible: record.voting_eligible,
        vote_status: record.computed_vote_status(),
        identity_status: record.computed_identity_status(),
        passport_valid_from: record.passport_valid_from.clone(),
        passport_valid_until: record.passport_valid_until.clone(),
        status_updated_at: record.status_updated_at,
        province_code: record.province_code.clone(),
        city_code: record.city_code.clone(),
        town_code: record.town_code.clone(),
        province_name,
        city_name,
        town_name,
        birth_province_code: record.birth_province_code.clone(),
        birth_city_code: record.birth_city_code.clone(),
        birth_town_code: record.birth_town_code.clone(),
        birth_province_name,
        birth_city_name,
        birth_town_name,
        archive_hash: record.archive_hash.clone(),
        onchain_tx_hash: record.onchain_tx_hash.clone(),
        onchain_block_number: record.onchain_block_number,
        onchain_at: record.onchain_at,
    }
}

fn stable_institution_cursor_id(cid_number: &str) -> i64 {
    cid_number
        .as_bytes()
        .iter()
        .fold(0i64, |acc, byte| {
            acc.wrapping_mul(131).wrapping_add(*byte as i64)
        })
        .wrapping_abs()
}

fn institution_row_from_record(
    inst: &crate::institution::subjects::Institution,
    account_count: usize,
    created_by_name: Option<String>,
    created_by_role: Option<String>,
) -> crate::institution::subjects::InstitutionListRow {
    crate::institution::subjects::InstitutionListRow {
        cid_number: inst.cid_number.clone(),
        cid_full_name: inst.cid_full_name.clone(),
        cid_short_name: inst.cid_short_name.clone(),
        status: inst.status.clone(),
        category: inst.category,
        p1: inst.p1.clone(),
        province_name: inst.province_name.clone(),
        city_name: inst.city_name.clone(),
        town_name: inst.town_name.clone(),
        institution_code: inst.institution_code.clone(),
        education_type: inst.education_type.clone(),
        private_type: inst.private_type.clone(),
        partnership_kind: inst.partnership_kind.clone(),
        has_legal_personality: inst.has_legal_personality,
        parent_cid_number: inst.parent_cid_number.clone(),
        account_count,
        created_at: inst.created_at,
        created_by_name,
        created_by_role,
    }
}

fn institution_row_from_pg_row(
    row: &postgres::Row,
) -> Result<crate::institution::subjects::InstitutionListRow, String> {
    let category_text: String = row.get(2);
    let category = institution_category_from_text(category_text.as_str())
        .ok_or_else(|| format!("invalid institution category: {category_text}"))?;
    let account_count_i64: i64 = row.get(15);
    let created_by_name: Option<String> = row.get(16);
    let created_by_role: Option<String> = row.get(17);
    let cid_full_name: Option<String> = row.get(18);
    let cid_short_name: Option<String> = row.get(19);
    let town_code: Option<String> = row.get(21);
    let education_type: Option<String> = row.get(22);
    let status: String = row.get(23);
    // 省/市/镇名字按 code 现场从 china.sqlite 派生,库里不存名字副本(ADR-021)。
    let province_code: String = row.get(6);
    let city_code: Option<String> = row.get(7);
    let town_code_value = town_code.clone().unwrap_or_default();
    let (province_name, city_name, town_name) = crate::cid::china::area_display_names(
        province_code.as_str(),
        city_code.as_deref(),
        Some(town_code_value.as_str()),
    );
    let inst = crate::institution::subjects::Institution {
        cid_number: row.get(0),
        cid_full_name,
        cid_short_name,
        status,
        category,
        p1: row.get(3),
        province_name,
        city_name,
        town_name,
        province_code,
        city_code: city_code.unwrap_or_default(),
        town_code: town_code.unwrap_or_default(),
        institution_code: row.get(8),
        education_type,
        private_type: row.get(9),
        partnership_kind: row.get(10),
        has_legal_personality: row.get(11),
        parent_cid_number: row.get(12),
        legal_rep_name: None,
        legal_rep_cid_number: None,
        legal_rep_photo_path: None,
        legal_rep_photo_name: None,
        legal_rep_photo_mime: None,
        legal_rep_photo_size: None,
        created_by: row.get(13),
        created_at: row.get(14),
    };
    let item = institution_row_from_record(
        &inst,
        usize::try_from(account_count_i64).unwrap_or(0),
        created_by_name,
        created_by_role,
    );
    Ok(item)
}

fn institution_from_subject_row(
    row: &postgres::Row,
) -> Result<crate::institution::subjects::Institution, String> {
    let category_text: String = row.get(2);
    let category = institution_category_from_text(category_text.as_str())
        .ok_or_else(|| format!("invalid institution category: {category_text}"))?;
    let cid_full_name: Option<String> = row.get(15);
    let cid_short_name: Option<String> = row.get(16);
    let town_code: Option<String> = row.get(18);
    let education_type: Option<String> = row.get(19);
    let status: String = row.get(20);
    // 字段顺序必须与 get_institution_with_accounts 的 SELECT 保持一致;
    // legal_rep_photo_size 是第 27 列,下标为 26,越界会在持有数据库锁时 panic。
    let legal_rep_photo_size_i64: Option<i64> = row.get(26);
    // 省/市/镇名字按 code 现场从 china.sqlite 派生,DTO 仍带名字,库里不存名字副本(ADR-021)。
    let province_code: String = row.get(6);
    let city_code: Option<String> = row.get(7);
    let town_code_value = town_code.clone().unwrap_or_default();
    let (province_name, city_name, town_name) = crate::cid::china::area_display_names(
        province_code.as_str(),
        city_code.as_deref(),
        Some(town_code_value.as_str()),
    );
    Ok(crate::institution::subjects::Institution {
        cid_number: row.get(0),
        cid_full_name,
        cid_short_name,
        status,
        category,
        p1: row.get(3),
        province_name,
        city_name,
        town_name,
        province_code,
        city_code: city_code.unwrap_or_default(),
        town_code: town_code.unwrap_or_default(),
        institution_code: row.get(8),
        education_type,
        private_type: row.get(9),
        partnership_kind: row.get(10),
        has_legal_personality: row.get(11),
        parent_cid_number: row.get(12),
        legal_rep_name: row.get(21),
        legal_rep_cid_number: row.get(22),
        legal_rep_photo_path: row.get(23),
        legal_rep_photo_name: row.get(24),
        legal_rep_photo_mime: row.get(25),
        legal_rep_photo_size: legal_rep_photo_size_i64.and_then(|v| u64::try_from(v).ok()),
        created_by: row.get(13),
        created_at: row.get(14),
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
    pub(crate) fn cid_full_name_exists(
        &self,
        cid_full_name: &str,
        province_code: Option<&str>,
        city_code: Option<&str>,
        exclude_cid_number: Option<&str>,
    ) -> Result<bool, String> {
        let cid_full_name = cid_full_name.trim().to_string();
        let province_code = province_code.map(str::to_string);
        let city_code = city_code.map(str::to_string);
        let exclude_cid_number = exclude_cid_number.map(str::to_string);
        self.with_client(move |conn| {
            let row = conn
                .query_one(
                    "SELECT EXISTS (
                        SELECT 1 FROM subjects
                        WHERE kind IN ('PUBLIC', 'PRIVATE')
                          AND cid_full_name = $1
                          AND ($2::text IS NULL OR province_code = $2)
                          AND ($3::text IS NULL OR city_code = $3)
                          AND ($4::text IS NULL OR cid_number <> $4)
                     )",
                    &[
                        &cid_full_name,
                        &province_code,
                        &city_code,
                        &exclude_cid_number,
                    ],
                )
                .map_err(|e| format!("query cid_full_name conflict failed: {e}"))?;
            Ok(row.get(0))
        })
    }

    pub(crate) fn get_institution_with_accounts(
        &self,
        cid_number: &str,
    ) -> Result<
        Option<(
            crate::institution::subjects::Institution,
            Vec<crate::institution::subjects::InstitutionAccount>,
        )>,
        String,
    > {
        let cid_number = cid_number.trim().to_string();
        self.with_client(move |conn| Self::get_institution_with_accounts_conn(conn, &cid_number))
    }

    pub(crate) fn chain_public_institution_cid_by_code(
        &self,
        institution_code: &str,
    ) -> Result<Option<String>, String> {
        let institution_code = institution_code.trim().to_string();
        self.with_client(move |conn| {
            let rows = conn
                .query(
                    "SELECT cid_number
                     FROM gov
                     WHERE source = 'CHAIN'
                       AND institution_code = $1
                     ORDER BY cid_number ASC
                     LIMIT 2",
                    &[&institution_code],
                )
                .map_err(|e| {
                    format!(
                        "query chain public institution by code failed: {}",
                        crate::core::db::postgres_error_text(&e)
                    )
                })?;
            if rows.len() > 1 {
                return Err(format!(
                    "chain public institution code {institution_code} is not unique in local projection"
                ));
            }
            Ok(rows.first().map(|row| row.get::<_, String>(0)))
        })
    }

    /// `get_institution_with_accounts` 的 conn 级版本,供已持有连接的
    /// 注册局动作派发(admins/actions.rs 注销校验）直接复用,避免嵌套 with_client。
    pub(crate) fn get_institution_with_accounts_conn(
        conn: &mut postgres::Client,
        cid_number: &str,
    ) -> Result<
        Option<(
            crate::institution::subjects::Institution,
            Vec<crate::institution::subjects::InstitutionAccount>,
        )>,
        String,
    > {
        let cid_number = cid_number.trim().to_string();
        {
            let row = conn
                .query_opt(
                    "SELECT s.cid_number, s.cid_full_name, s.category,
                            s.p1, ''::text AS province_name,
                            ''::text AS city_name, s.province_code, s.city_code, s.institution_code,
                            s.private_type, s.partnership_kind, s.has_legal_personality,
                            s.parent_cid_number, s.created_by, s.created_at,
                            s.cid_full_name, s.cid_short_name,
                            ''::text AS town_name, COALESCE(s.town_code, ''),
                            s.education_type, s.status, s.legal_rep_name, s.legal_rep_cid_number,
	                            s.legal_rep_photo_path, s.legal_rep_photo_name,
	                            s.legal_rep_photo_mime, s.legal_rep_photo_size
		                     FROM subjects s
		                     LEFT JOIN gov g ON g.province_code = s.province_code AND g.cid_number = s.cid_number
	                     WHERE s.kind IN ('PUBLIC', 'PRIVATE') AND s.cid_number = $1
	                     LIMIT 1",
                    &[&cid_number],
                )
                .map_err(|e| format!("query institution failed: {e}"))?;
            let Some(row) = row else {
                return Ok(None);
            };
            let inst = institution_from_subject_row(&row)?;
            let account_rows = conn
                .query(
                    "SELECT cid_number, account_name, account, chain_status, created_at
                     FROM accounts
                     WHERE cid_number = $1
                     ORDER BY account_name ASC",
                    &[&cid_number],
                )
                .map_err(|e| format!("query institution accounts failed: {e}"))?;
            let mut accounts = Vec::with_capacity(account_rows.len());
            for row in account_rows {
                let status_text: String = row.get(3);
                accounts.push(crate::institution::subjects::InstitutionAccount {
                    cid_number: row.get(0),
                    account_name: row.get(1),
                    account: row.get(2),
                    chain_status: multisig_chain_status_from_text(status_text.as_str()),
                    chain_synced_at: None,
                    chain_tx_hash: None,
                    chain_block_number: None,
                    created_by: String::new(),
                    created_at: row.get(4),
                });
            }
            Ok(Some((inst, accounts)))
        }
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
        let cid_number = record.cid_number.trim().to_string();
        if cid_number.is_empty() {
            return Err("citizen cid_number is required".to_string());
        }
        let province_code = record.province_code.trim().to_string();
        let city_code = record.city_code.trim().to_string();
        if province_code.is_empty() || city_code.is_empty() {
            return Err("citizen province_code/city_code is required".to_string());
        }
        let status = if matches!(record.computed_identity_status(), CitizenStatus::Normal) {
            "ACTIVE"
        } else {
            "REVOKED"
        };
        let citizen_status = citizen_status_text(&record.citizen_status);
        let id = i64::try_from(record.id).map_err(|_| "citizen id exceeds i64".to_string())?;
        Self::upsert_target_id_row(
            conn,
            cid_number.as_str(),
            "CITIZEN",
            province_code.as_str(),
            Some(city_code.as_str()),
        )?;
        Self::delete_target_rows_outside_scope(
            conn,
            "subjects",
            cid_number.as_str(),
            province_code.as_str(),
        )?;
        Self::delete_target_rows_outside_scope(
            conn,
            "citizens",
            cid_number.as_str(),
            province_code.as_str(),
        )?;
        conn.execute(
            "INSERT INTO subjects (
                cid_number, kind, province_code, city_code, status, created_at, updated_at
             ) VALUES ($1, 'CITIZEN', $2, $3, $4, $5, now())
             ON CONFLICT (province_code, cid_number) DO UPDATE SET
                city_code = EXCLUDED.city_code,
                status = EXCLUDED.status,
                updated_at = now()",
            &[
                &cid_number,
                &province_code,
                &city_code,
                &status,
                &record.created_at,
            ],
        )
        .map_err(|e| format!("upsert citizen subject failed: {e}"))?;
        conn.execute(
            "INSERT INTO citizens (
                cid_number, passport_no, citizen_family_name, citizen_given_name,
                citizen_sex, citizen_birth_date, province_code, city_code, id,
                wallet_pubkey, wallet_address, wallet_sig_alg,
                wallet_verified_at, citizen_status, voting_eligible, passport_valid_from,
                passport_valid_until, status_updated_at, town_code, birth_province_code, birth_city_code,
                birth_town_code, archive_hash, onchain_tx_hash, onchain_block_number, onchain_at,
                created_by, created_at, updated_by, updated_at
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19,
                $20, $21, $22, $23, $24, $25, $26, $27, $28,
                $29, $30
             )
             ON CONFLICT (province_code, cid_number) DO UPDATE SET
                passport_no = EXCLUDED.passport_no,
                citizen_family_name = EXCLUDED.citizen_family_name,
                citizen_given_name = EXCLUDED.citizen_given_name,
                citizen_sex = EXCLUDED.citizen_sex,
                citizen_birth_date = EXCLUDED.citizen_birth_date,
                city_code = EXCLUDED.city_code,
                id = EXCLUDED.id,
                wallet_pubkey = EXCLUDED.wallet_pubkey,
                wallet_address = EXCLUDED.wallet_address,
                wallet_sig_alg = EXCLUDED.wallet_sig_alg,
                wallet_verified_at = EXCLUDED.wallet_verified_at,
                citizen_status = EXCLUDED.citizen_status,
                voting_eligible = EXCLUDED.voting_eligible,
                passport_valid_from = EXCLUDED.passport_valid_from,
                passport_valid_until = EXCLUDED.passport_valid_until,
                status_updated_at = EXCLUDED.status_updated_at,
                town_code = EXCLUDED.town_code,
                birth_province_code = EXCLUDED.birth_province_code,
                birth_city_code = EXCLUDED.birth_city_code,
                birth_town_code = EXCLUDED.birth_town_code,
                archive_hash = EXCLUDED.archive_hash,
                onchain_tx_hash = EXCLUDED.onchain_tx_hash,
                onchain_block_number = EXCLUDED.onchain_block_number,
                onchain_at = EXCLUDED.onchain_at,
                updated_by = EXCLUDED.updated_by,
                updated_at = EXCLUDED.updated_at",
            &[
                &cid_number,
                &record.passport_no,
                &record.citizen_family_name,
                &record.citizen_given_name,
                &record.citizen_sex,
                &record.citizen_birth_date,
                &province_code,
                &city_code,
                &id,
                &record.wallet_pubkey,
                &record.wallet_address,
                &record.wallet_sig_alg,
                &record.wallet_verified_at,
                &citizen_status,
                &record.voting_eligible,
                &record.passport_valid_from,
                &record.passport_valid_until,
                &record.status_updated_at,
                &record.town_code,
                &record.birth_province_code,
                &record.birth_city_code,
                &record.birth_town_code,
                &record.archive_hash,
                &record.onchain_tx_hash,
                &record.onchain_block_number,
                &record.onchain_at,
                &record.created_by,
                &record.created_at,
                &record.updated_by,
                &record.updated_at,
            ],
        )
        .map_err(|e| format!("upsert citizens failed: {e}"))?;
        conn.execute(
            "INSERT INTO passport_numbers (passport_no, cid_number, province_code, city_code, created_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (passport_no) DO UPDATE SET
                cid_number = EXCLUDED.cid_number,
                province_code = EXCLUDED.province_code,
                city_code = EXCLUDED.city_code",
            &[
                &record.passport_no,
                &cid_number,
                &province_code,
                &city_code,
                &record.created_at,
            ],
        )
        .map_err(|e| format!("upsert passport number failed: {e}"))?;
        Ok(())
    }

    fn upsert_target_id_row<C: postgres::GenericClient>(
        conn: &mut C,
        cid_number: &str,
        kind: &str,
        province_code: &str,
        city_code: Option<&str>,
    ) -> Result<(), String> {
        // ids 是 cid_number 全局唯一索引,同号不能在身份大类之间静默改义。
        let existing = conn
            .query_opt("SELECT kind FROM ids WHERE cid_number = $1", &[&cid_number])
            .map_err(|e| format!("query target id failed: {e}"))?;
        if let Some(row) = existing {
            let existing_kind: String = row.get(0);
            if existing_kind != kind {
                return Err(format!(
                    "cid_number {cid_number} already belongs to {existing_kind}, cannot write {kind}"
                ));
            }
            conn.execute(
                "UPDATE ids SET province_code = $2, city_code = $3 WHERE cid_number = $1",
                &[&cid_number, &province_code, &city_code],
            )
            .map_err(|e| format!("update target id failed: {e}"))?;
        } else {
            conn.execute(
                "INSERT INTO ids (cid_number, kind, province_code, city_code)
                 VALUES ($1, $2, $3, $4)",
                &[&cid_number, &kind, &province_code, &city_code],
            )
            .map_err(|e| format!("insert target id failed: {e}"))?;
        }
        Ok(())
    }

    fn delete_target_rows_outside_scope<C: postgres::GenericClient>(
        conn: &mut C,
        table: &str,
        cid_number: &str,
        province_code: &str,
    ) -> Result<(), String> {
        // 分区键按行政区划真源修正时,清掉同一 cid 留在原分区的查询行。
        let sql = format!("DELETE FROM {table} WHERE cid_number = $1 AND province_code <> $2");
        conn.execute(sql.as_str(), &[&cid_number, &province_code])
            .map_err(|e| format!("delete {table} rows outside scope failed: {e}"))?;
        Ok(())
    }

    pub(crate) fn list_citizens_page(
        &self,
        keyword: &str,
        province_code: Option<&str>,
        city_code: Option<&str>,
        cursor: Option<&str>,
        page_size: usize,
    ) -> Result<PageResult<CitizenRow>, String> {
        let keyword = keyword.trim();
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
                    "SELECT COALESCE(c.id, 0), c.cid_number, c.passport_no, c.citizen_family_name,
                                    c.citizen_given_name, c.citizen_sex, c.citizen_birth_date,
                                    c.wallet_pubkey, c.wallet_address,
                                    c.wallet_sig_alg, c.wallet_verified_at, c.citizen_status, c.voting_eligible,
                                    c.passport_valid_from, c.passport_valid_until, c.status_updated_at,
                                    c.province_code, c.city_code, c.town_code,
                                    c.birth_province_code, c.birth_city_code, c.birth_town_code,
                                    c.archive_hash, c.onchain_tx_hash, c.onchain_block_number, c.onchain_at,
                                    c.created_by, c.created_at, c.updated_by, c.updated_at
                             FROM citizens c
                             JOIN subjects s
                               ON s.province_code = c.province_code
                              AND s.cid_number = c.cid_number
                              AND s.kind = 'CITIZEN'
                             WHERE ($1::text IS NULL OR c.province_code = $1)
                               AND ($2::text IS NULL OR c.city_code = $2)
                               AND (
                                    $3::text = ''
                                    OR
                                    c.cid_number = $3
                                    OR c.passport_no = $3
                                    OR c.citizen_family_name || c.citizen_given_name = $3
                                    OR c.citizen_family_name = $3
                                    OR c.citizen_given_name = $3
                                    OR (c.wallet_pubkey IS NOT NULL AND lower(c.wallet_pubkey) = lower($3))
                                    OR (c.wallet_address IS NOT NULL AND lower(c.wallet_address) = lower($3))
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
                let created_at: DateTime<Utc> = row.get(29);
                let record = crate::domains::citizens::admin_entry::citizen_record_from_row(&row);
                output.push((citizen_row_from_record(&record), created_at, id_i64));
            }
            Ok(page_from_rows(output, page_size))
        })
    }

    pub(crate) fn upsert_institution_row(
        &self,
        inst: &crate::institution::subjects::Institution,
    ) -> Result<(), String> {
        let inst = inst.clone();
        self.with_client(move |conn| {
            Self::upsert_target_subject_rows(conn, &inst)?;
            Ok(())
        })
    }

    pub(crate) fn legal_representative_citizen_exists_in_scope(
        &self,
        cid_number: &str,
        scope: &crate::institution::subjects::service::LegalRepresentativeCitizenScope,
    ) -> Result<bool, String> {
        let cid_number = cid_number.trim().to_string();
        let province_code = scope.province_code().map(str::to_string);
        let city_code = scope.city_code().map(str::to_string);
        self.with_client(move |conn| {
            let row = conn
                .query_one(
                    "SELECT EXISTS (
                        SELECT 1 FROM citizens
                        WHERE cid_number = $1
                          AND citizen_status = 'NORMAL'
                          AND ($2::text IS NULL OR province_code = $2)
                          AND ($3::text IS NULL OR city_code = $3)
                     )",
                    &[&cid_number, &province_code, &city_code],
                )
                .map_err(|e| format!("query legal representative citizen failed: {e}"))?;
            Ok(row.get(0))
        })
    }

    pub(crate) fn search_legal_representative_citizens_in_scope(
        &self,
        q: &str,
        page_size: usize,
        scope: &crate::institution::subjects::service::LegalRepresentativeCitizenScope,
    ) -> Result<Vec<String>, String> {
        let q = q.trim().to_string();
        let province_code = scope.province_code().map(str::to_string);
        let city_code = scope.city_code().map(str::to_string);
        self.with_client(move |conn| {
            let limit = i64::try_from(page_size).map_err(|_| "page_size too large".to_string())?;
            let rows = conn
                .query(
                    "SELECT cid_number
                     FROM citizens
                     WHERE citizen_status = 'NORMAL'
                       AND ($1::text IS NULL OR province_code = $1)
                       AND ($2::text IS NULL OR city_code = $2)
                       AND cid_number ILIKE '%' || $3 || '%'
                     ORDER BY cid_number ASC
                     LIMIT $4",
                    &[&province_code, &city_code, &q, &limit],
                )
                .map_err(|e| format!("query legal representative citizens failed: {e}"))?;
            Ok(rows
                .iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>())
        })
    }

    fn upsert_target_subject_rows<C: postgres::GenericClient>(
        conn: &mut C,
        inst: &crate::institution::subjects::Institution,
    ) -> Result<(), String> {
        let kind = match inst.category {
            crate::cid::InstitutionCategory::PrivateInstitution => "PRIVATE",
            crate::cid::InstitutionCategory::GovInstitution => "PUBLIC",
        };
        let province_code = inst.province_code.clone();
        let city_code = if inst.city_code == "000" || inst.city_code.is_empty() {
            None
        } else {
            Some(inst.city_code.clone())
        };
        let town_code = if inst.town_code.trim().is_empty() {
            None
        } else {
            Some(inst.town_code.clone())
        };
        let status = inst.status.trim().to_string();
        Self::upsert_target_id_row(
            conn,
            inst.cid_number.as_str(),
            kind,
            province_code.as_str(),
            city_code.as_deref(),
        )?;
        Self::delete_target_rows_outside_scope(
            conn,
            "subjects",
            inst.cid_number.as_str(),
            province_code.as_str(),
        )?;
        let category = institution_category_text(inst.category);
        let legal_rep_photo_size = inst
            .legal_rep_photo_size
            .and_then(|v| i64::try_from(v).ok());
        // 行政区名字不入库(china.sqlite 单源),只写 province_code/city_code/town_code。
        conn.execute(
            "INSERT INTO subjects (
                cid_number, kind, cid_full_name, cid_short_name,
                status, category, p1,
                province_code, city_code, town_code, institution_code,
                education_type, private_type, partnership_kind, has_legal_personality,
                parent_cid_number, legal_rep_name, legal_rep_cid_number,
                legal_rep_photo_path, legal_rep_photo_name, legal_rep_photo_mime,
                legal_rep_photo_size, created_by, created_at, updated_at
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19,
                $20, $21, $22, $23, $24, now()
             )
             ON CONFLICT (province_code, cid_number) DO UPDATE SET
                kind = EXCLUDED.kind,
                cid_full_name = EXCLUDED.cid_full_name,
                cid_short_name = EXCLUDED.cid_short_name,
                status = EXCLUDED.status,
                category = EXCLUDED.category,
                p1 = EXCLUDED.p1,
                province_code = EXCLUDED.province_code,
                city_code = EXCLUDED.city_code,
                town_code = EXCLUDED.town_code,
                institution_code = EXCLUDED.institution_code,
                education_type = EXCLUDED.education_type,
                private_type = EXCLUDED.private_type,
                partnership_kind = EXCLUDED.partnership_kind,
                has_legal_personality = EXCLUDED.has_legal_personality,
                parent_cid_number = EXCLUDED.parent_cid_number,
                legal_rep_name = EXCLUDED.legal_rep_name,
                legal_rep_cid_number = EXCLUDED.legal_rep_cid_number,
                legal_rep_photo_path = EXCLUDED.legal_rep_photo_path,
                legal_rep_photo_name = EXCLUDED.legal_rep_photo_name,
                legal_rep_photo_mime = EXCLUDED.legal_rep_photo_mime,
                legal_rep_photo_size = EXCLUDED.legal_rep_photo_size,
                created_by = EXCLUDED.created_by,
                updated_at = now()",
            &[
                &inst.cid_number,
                &kind,
                &inst.cid_full_name,
                &inst.cid_short_name,
                &status,
                &category,
                &inst.p1,
                &inst.province_code,
                &inst.city_code,
                &inst.town_code,
                &inst.institution_code,
                &inst.education_type,
                &inst.private_type,
                &inst.partnership_kind,
                &inst.has_legal_personality,
                &inst.parent_cid_number,
                &inst.legal_rep_name,
                &inst.legal_rep_cid_number,
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
            crate::cid::InstitutionCategory::PrivateInstitution => {
                Self::delete_target_rows_outside_scope(
                    conn,
                    "private",
                    inst.cid_number.as_str(),
                    province_code.as_str(),
                )?;
                conn.execute("DELETE FROM gov WHERE cid_number = $1", &[&inst.cid_number])
                    .map_err(|e| format!("delete gov row for private subject failed: {e}"))?;
                if let Some(private_type) = &inst.private_type {
                    let has_legal_personality = inst.has_legal_personality.unwrap_or(false);
                    conn.execute(
                        "INSERT INTO private (
                            cid_number, province_code, city_code, code, private_type, partnership_kind,
                            has_legal_personality, p1, parent_cid_number
                         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                         ON CONFLICT (province_code, cid_number) DO UPDATE SET
                            city_code = EXCLUDED.city_code,
                            code = EXCLUDED.code,
                            private_type = EXCLUDED.private_type,
                            partnership_kind = EXCLUDED.partnership_kind,
                            has_legal_personality = EXCLUDED.has_legal_personality,
                            p1 = EXCLUDED.p1,
                            parent_cid_number = EXCLUDED.parent_cid_number",
                        &[
                            &inst.cid_number,
                            &province_code,
                            &city_code,
                            &inst.institution_code,
                            private_type,
                            &inst.partnership_kind,
                            &has_legal_personality,
                            &inst.p1,
                            &inst.parent_cid_number,
                        ],
                    )
                    .map_err(|e| format!("upsert private failed: {e}"))?;
                } else {
                    conn.execute(
                        "DELETE FROM private WHERE cid_number = $1",
                        &[&inst.cid_number],
                    )
                    .map_err(|e| format!("delete non-private typed row failed: {e}"))?;
                }
            }
            crate::cid::InstitutionCategory::GovInstitution => {
                Self::delete_target_rows_outside_scope(
                    conn,
                    "gov",
                    inst.cid_number.as_str(),
                    province_code.as_str(),
                )?;
                conn.execute(
                    "DELETE FROM private WHERE cid_number = $1",
                    &[&inst.cid_number],
                )
                .map_err(|e| format!("delete private row for public subject failed: {e}"))?;
                let home_p: Option<String> = None;
                let home_c: Option<String> = None;
                conn.execute(
                    "INSERT INTO gov (
		                        cid_number, province_code, city_code, town_code, institution_code,
		                        source, home_p, home_c
		                     ) VALUES ($1, $2, $3, $4, $5, 'MANUAL', $6, $7)
		                     ON CONFLICT (province_code, cid_number) DO UPDATE SET
		                        city_code = EXCLUDED.city_code,
		                        town_code = EXCLUDED.town_code,
		                        institution_code = EXCLUDED.institution_code,
		                        home_p = EXCLUDED.home_p,
		                        home_c = EXCLUDED.home_c",
                    &[
                        &inst.cid_number,
                        &province_code,
                        &city_code,
                        &town_code,
                        &inst.institution_code,
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
        account: &crate::institution::subjects::InstitutionAccount,
    ) -> Result<(), String> {
        let account = account.clone();
        self.with_client(move |conn| {
            Self::upsert_target_account_row(conn, &account)?;
            Ok(())
        })
    }

    fn upsert_target_account_row(
        conn: &mut postgres::Client,
        account: &crate::institution::subjects::InstitutionAccount,
    ) -> Result<(), String> {
        let scope_row = conn
            .query_opt(
                "SELECT province_code, city_code FROM ids WHERE cid_number = $1",
                &[&account.cid_number],
            )
            .map_err(|e| format!("query id scope for account failed: {e}"))?;
        let (fallback_p, fallback_c) = Self::scope_codes_from_cid(account.cid_number.as_str());
        let (province_code, city_code): (String, Option<String>) = match scope_row {
            Some(row) => (row.get(0), row.get(1)),
            None => (fallback_p, fallback_c),
        };
        let chain_status = multisig_chain_status_text(&account.chain_status);
        Self::delete_target_rows_outside_scope(
            conn,
            "accounts",
            account.cid_number.as_str(),
            province_code.as_str(),
        )?;
        conn.execute(
            "INSERT INTO accounts (
                cid_number, province_code, city_code, account_name, account, chain_status, created_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (province_code, cid_number, account_name) DO UPDATE SET
                city_code = EXCLUDED.city_code,
                account = EXCLUDED.account,
                chain_status = EXCLUDED.chain_status,
                created_at = EXCLUDED.created_at",
            &[
                &account.cid_number,
                &province_code,
                &city_code,
                &account.account_name,
                &account.account,
                &chain_status,
                &account.created_at,
            ],
        )
        .map_err(|e| format!("upsert accounts failed: {e}"))?;
        Ok(())
    }

    fn scope_codes_from_cid(cid_number: &str) -> (String, Option<String>) {
        let Some(r5) = cid_number.split('-').next() else {
            return ("ZS".to_string(), None);
        };
        if r5.len() < 5 {
            return ("ZS".to_string(), None);
        }
        let province_code = r5[0..2].to_string();
        let c_part = &r5[2..5];
        let city_code = if c_part == "000" {
            None
        } else {
            Some(c_part.to_string())
        };
        (province_code, city_code)
    }

    pub(crate) fn delete_institution_account_row(
        &self,
        cid_number: &str,
        account_name: &str,
    ) -> Result<(), String> {
        let cid_number = cid_number.to_string();
        let account_name = account_name.to_string();
        self.with_client(move |conn| {
            conn.execute(
                "DELETE FROM accounts
                 WHERE cid_number = $1 AND account_name = $2",
                &[&cid_number, &account_name],
            )
            .map_err(|e| format!("delete accounts failed: {e}"))?;
            Ok(())
        })
    }

    // 删除所有不合规 CID 号在各号承载表里的行。判定唯一标准 =
    // 过不了 `crate::cid::validate_cid_number_format`。
    // dry_run 时在事务内删完即回滚,只回报计数,不改库。
    pub(crate) fn purge_legacy_cid_rows(&self, dry_run: bool) -> Result<PurgeReport, String> {
        // 号承载表清单,无外键约束,删除顺序无关;主登记表 ids 放最后。
        const CID_TABLES: [&str; 8] = [
            "subjects",
            "citizens",
            "citizen_documents",
            "gov",
            "private",
            "accounts",
            "docs",
            "ids",
        ];
        self.with_client(move |conn| {
            // 1. 收集号全集与 kind(ids 为准,subjects 补孤儿)。
            let mut kind_by_cid: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for row in conn
                .query("SELECT cid_number, kind FROM ids", &[])
                .map_err(|e| format!("scan ids failed: {e}"))?
            {
                kind_by_cid.entry(row.get(0)).or_insert_with(|| row.get(1));
            }
            for row in conn
                .query("SELECT DISTINCT cid_number, kind FROM subjects", &[])
                .map_err(|e| format!("scan subjects failed: {e}"))?
            {
                kind_by_cid.entry(row.get(0)).or_insert_with(|| row.get(1));
            }

            // 2. 筛旧号:过不了新格式校验的即旧号。
            let legacy: Vec<String> = kind_by_cid
                .keys()
                .filter(|cid| crate::cid::validate_cid_number_format(cid).is_err())
                .cloned()
                .collect();
            let private_count = legacy
                .iter()
                .filter(|cid| kind_by_cid.get(*cid).map(String::as_str) == Some("PRIVATE"))
                .count();
            let citizen_count = legacy
                .iter()
                .filter(|cid| kind_by_cid.get(*cid).map(String::as_str) == Some("CITIZEN"))
                .count();

            if legacy.is_empty() {
                return Ok(PurgeReport {
                    legacy_count: 0,
                    private_count: 0,
                    citizen_count: 0,
                    per_table_deleted: CID_TABLES.iter().map(|table| (*table, 0)).collect(),
                    dry_run,
                });
            }

            // 3. 一事务内逐表删除,记录各表行数。
            let mut tx = conn
                .transaction()
                .map_err(|e| format!("begin purge legacy cid tx failed: {e}"))?;
            let mut per_table_deleted = Vec::with_capacity(CID_TABLES.len());
            for table in CID_TABLES {
                let sql = format!("DELETE FROM {table} WHERE cid_number = ANY($1)");
                let deleted = tx
                    .execute(sql.as_str(), &[&legacy])
                    .map_err(|e| format!("delete legacy cid from {table} failed: {e}"))?;
                per_table_deleted.push((table, deleted));
            }
            if dry_run {
                tx.rollback()
                    .map_err(|e| format!("rollback purge legacy cid dry-run failed: {e}"))?;
            } else {
                tx.commit()
                    .map_err(|e| format!("commit purge legacy cid failed: {e}"))?;
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

    // 扫出"孤儿机构"——subjects 中 town_code 非空、但该
    // (province_code,city_code,town_code) 三元组在行政区划真源 china.sqlite 里不存在
    // (town_code 指向已退役的镇)。判定只走
    // 进程内内存树(crate::cid::china::town_exists),不在 PG 里 join china 数据(PG 无 towns 表)。
    // 白名单:town_code 为空/NULL 的行(市级机构/储委会/部委合法态)永远不是孤儿,直接跳过。
    // 只读扫描,不改库;删除由调用方拿到 cid 列表后逐省级联删。
    pub(crate) fn scan_orphan_institutions(&self) -> Result<Vec<OrphanInstitution>, String> {
        self.with_client(move |conn| {
            let rows = conn
                .query(
                    "SELECT province_code, cid_number, kind,
                            COALESCE(city_code, ''), COALESCE(town_code, ''),
                            ''::text AS town_name, COALESCE(category, ''),
                            COALESCE(institution_code, '')
                     FROM subjects
                     WHERE town_code IS NOT NULL AND town_code <> ''",
                    &[],
                )
                .map_err(|e| {
                    format!(
                        "scan subjects for orphan institutions failed: {}",
                        crate::core::db::postgres_error_text(&e)
                    )
                })?;
            let mut orphans = Vec::new();
            for row in rows {
                let province_code: String = row.get(0);
                let cid_number: String = row.get(1);
                let kind: String = row.get(2);
                let city_code: String = row.get(3);
                let town_code: String = row.get(4);
                let town_name: String = row.get(5);
                let category: String = row.get(6);
                let institution_code: String = row.get(7);
                // 白名单:空 town_code 已在 SQL 过滤;此处再调 town_exists 内存树判定。
                if crate::cid::china::town_exists(&province_code, &city_code, &town_code) {
                    continue;
                }
                orphans.push(OrphanInstitution {
                    province_code,
                    cid_number,
                    kind,
                    city_code,
                    town_code,
                    town_name,
                    category,
                    institution_code,
                });
            }
            Ok(orphans)
        })
    }

    // 把待删孤儿行(subjects/gov/private/accounts)文本导出到备份文件,删除唯一回滚保证。
    // 用 COPY ... TO STDOUT 抓 TSV(不依赖 pg_dump 外部进程),每张表一段,带表名分隔头。
    // 仅命中传入 cid 集合内的行(逐省 province_code + cid ANY 过滤),不会导出无关数据。
    fn export_orphan_backup(
        &self,
        by_province: &std::collections::BTreeMap<String, Vec<String>>,
        backup_path: &str,
    ) -> Result<(), String> {
        use std::io::Write;
        let by_province = by_province.clone();
        let backup_path = backup_path.to_string();
        self.with_client(move |conn| {
            let mut file = std::fs::File::create(&backup_path)
                .map_err(|e| format!("create orphan backup file {backup_path} failed: {e}"))?;
            writeln!(
                file,
                "-- cid purge-orphan-institutions backup\n-- 删除前导出的待删孤儿行(TSV/COPY 格式),删除唯一回滚保证。"
            )
            .map_err(|e| format!("write orphan backup header failed: {e}"))?;
            // 按 (表, 主键关联列) 分别导出;subjects/gov/private/accounts 用 cid_number,
            // docs 用 cid_number,audit 用 target_cid。
            const EXPORTS: [(&str, &str); 6] = [
                ("subjects", "cid_number"),
                ("gov", "cid_number"),
                ("private", "cid_number"),
                ("accounts", "cid_number"),
                ("docs", "cid_number"),
                ("audit", "target_cid"),
            ];
            for (province, cids) in &by_province {
                for (table, key_col) in EXPORTS {
                    writeln!(file, "-- TABLE={table} P_CODE={province}")
                        .map_err(|e| format!("write orphan backup table header failed: {e}"))?;
                    let copy_sql = format!(
                        "COPY (SELECT * FROM {table} WHERE province_code = '{province}' AND {key_col} = ANY(ARRAY[{}])) TO STDOUT",
                        cids
                            .iter()
                            .map(|s| format!("'{}'", s.replace('\'', "''")))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    let mut reader = conn
                        .copy_out(copy_sql.as_str())
                        .map_err(|e| format!("copy out {table} for orphan backup failed: {e}"))?;
                    std::io::copy(&mut reader, &mut file)
                        .map_err(|e| format!("stream {table} copy to orphan backup failed: {e}"))?;
                }
            }
            file.flush()
                .map_err(|e| format!("flush orphan backup file failed: {e}"))?;
            Ok(())
        })
    }

    // 逐省单事务级联删孤儿机构。每省一条事务,WHERE province_code=$1 命中子分区,
    // cid_number = ANY($2) 限定本省孤儿集合;禁止跨省一条 SQL。级联删顺序遵守关联依赖:
    // accounts → docs → audit → gov|private → ids → subjects(子承载表先删,主登记表 ids
    // 与父表 subjects 最后)。gov/private 按 subjects.kind 区分(本方法传入已分好的两类 cid)。
    // audit 关联列为 target_cid。ids 表不分区,仅按 cid_number 删。
    pub(crate) fn delete_orphan_institutions_by_province(
        &self,
        province: &str,
        gov_cids: &[String],
        private_cids: &[String],
    ) -> Result<u64, String> {
        let province = province.to_string();
        let gov_cids = gov_cids.to_vec();
        let private_cids = private_cids.to_vec();
        self.with_client(move |conn| {
            // 本省全部孤儿 cid(gov + private 合集),用于按 province_code 分区命中的表。
            let all_cids: Vec<String> = gov_cids
                .iter()
                .chain(private_cids.iter())
                .cloned()
                .collect();
            if all_cids.is_empty() {
                return Ok(0);
            }
            let mut tx = conn
                .transaction()
                .map_err(|e| format!("begin orphan purge tx for {province} failed: {e}"))?;

            // 1. accounts(province_code 分区,按 cid_number)。
            tx.execute(
                "DELETE FROM accounts WHERE province_code = $1 AND cid_number = ANY($2)",
                &[&province, &all_cids],
            )
            .map_err(|e| format!("delete accounts for {province} failed: {e}"))?;

            // 2. docs(province_code 分区,按 cid_number)。
            tx.execute(
                "DELETE FROM docs WHERE province_code = $1 AND cid_number = ANY($2)",
                &[&province, &all_cids],
            )
            .map_err(|e| format!("delete docs for {province} failed: {e}"))?;

            // 3. audit(province_code 分区,关联列 target_cid)。
            tx.execute(
                "DELETE FROM audit WHERE province_code = $1 AND target_cid = ANY($2)",
                &[&province, &all_cids],
            )
            .map_err(|e| format!("delete audit for {province} failed: {e}"))?;

            // 4. gov / private(province_code 分区,按 kind 区分各自的 cid 集合)。
            if !gov_cids.is_empty() {
                tx.execute(
                    "DELETE FROM gov WHERE province_code = $1 AND cid_number = ANY($2)",
                    &[&province, &gov_cids],
                )
                .map_err(|e| format!("delete gov for {province} failed: {e}"))?;
            }
            if !private_cids.is_empty() {
                tx.execute(
                    "DELETE FROM private WHERE province_code = $1 AND cid_number = ANY($2)",
                    &[&province, &private_cids],
                )
                .map_err(|e| format!("delete private for {province} failed: {e}"))?;
            }

            // 5. ids(不分区,主登记表,仅按 cid_number)。
            tx.execute("DELETE FROM ids WHERE cid_number = ANY($1)", &[&all_cids])
                .map_err(|e| format!("delete ids for {province} failed: {e}"))?;

            // 6. subjects(province_code 分区,父表最后删)。
            let deleted = tx
                .execute(
                    "DELETE FROM subjects WHERE province_code = $1 AND cid_number = ANY($2)",
                    &[&province, &all_cids],
                )
                .map_err(|e| format!("delete subjects for {province} failed: {e}"))?;

            tx.commit()
                .map_err(|e| format!("commit orphan purge tx for {province} failed: {e}"))?;
            Ok(deleted)
        })
    }

    pub(crate) fn list_institutions_exact(
        &self,
        filter: crate::institution::subjects::InstitutionListFilter,
        private_type: Option<&str>,
        province_code: &str,
        city_code: Option<&str>,
        keyword: &str,
        cursor: Option<&str>,
        page_size: usize,
    ) -> Result<PageResult<crate::institution::subjects::InstitutionListRow>, String> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            if matches!(
                filter,
                crate::institution::subjects::InstitutionListFilter::Education
            ) {
                return self.list_education_committees_direct(province_code, city_code, page_size);
            }
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
        let private_type = private_type
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let province_code = province_code.to_string();
        let city_code = city_code.map(str::to_string);
        let keyword = keyword.to_string();
        self.with_client(move |conn| {
            let cursor_created_at = cursor.map(|c| c.created_at);
            let fetch_limit = i64::try_from(page_size.saturating_add(1))
                .map_err(|_| "page_size too large".to_string())?;
            // 过滤子句来自 InstitutionListFilter 的静态字面量(教育=手动 JY 行,
            // 非法人按父级属性分流到公权/私权),无注入面;par 是子句依赖的父级别名,
            // 父级允许跨省(私法人全国),故只按 cid_number 关联不限定 province_code。
            let sql = format!(
		                    "SELECT s.cid_number, s.cid_full_name, s.category,
			                                    s.p1, ''::text AS province_name,
			                                    ''::text AS city_name, s.province_code, s.city_code, s.institution_code,
				                                    s.private_type, s.partnership_kind, s.has_legal_personality,
				                                    s.parent_cid_number, s.created_by, s.created_at,
				                                    COALESCE(ac.account_count, 0),
				                                    a.admin_name, a.institution_code, s.cid_full_name, s.cid_short_name,
				                                    ''::text AS town_name, COALESCE(s.town_code, ''),
				                                    s.education_type,
				                                    s.status
		                             FROM subjects s
		                             LEFT JOIN gov g ON g.province_code = s.province_code AND g.cid_number = s.cid_number
		                             LEFT JOIN subjects par ON par.cid_number = s.parent_cid_number
		                             LEFT JOIN (
	                                SELECT province_code, cid_number, COUNT(*)::BIGINT AS account_count
	                                FROM accounts
	                                WHERE province_code = $1
	                                  AND ($2::text IS NULL OR city_code = $2)
	                                GROUP BY province_code, cid_number
	                             ) ac ON ac.province_code = s.province_code AND ac.cid_number = s.cid_number
	                             LEFT JOIN admins a ON lower(a.admin_account) = lower(s.created_by)
	                             WHERE s.kind IN ('PUBLIC', 'PRIVATE')
	                               {filter_clause}
	                               AND ($6::text IS NULL OR s.private_type = $6)
	                               AND s.province_code = $1
	                               AND ($2::text IS NULL OR s.city_code = $2)
	                               AND (
	                                    s.cid_number = $3
	                                    OR lower(COALESCE(s.cid_full_name, '')) = lower($3)
	                                    OR lower(COALESCE(s.cid_short_name, '')) = lower($3)
	                               )
	                               AND (
	                                    $4::timestamptz IS NULL
	                                    OR s.created_at < $4
	                               )
	                             ORDER BY s.created_at DESC, s.cid_number DESC
	                             LIMIT $5",
                filter_clause = filter.sql_clause(),
            );
            let rows = conn
                .query(
                    sql.as_str(),
                    &[
                        &province_code,
                        &city_code,
                        &keyword,
                        &cursor_created_at,
                        &fetch_limit,
                        &private_type,
                    ],
                )
                .map_err(|e| format!("query subjects failed: {e}"))?;
            let mut output = Vec::with_capacity(rows.len());
            for row in rows {
                // 列布局与 institution_row_from_pg_row 一致,统一走该 helper(含名字派生)。
                let created_at: DateTime<Utc> = row.get(14);
                let cid_number: String = row.get(0);
                let item = institution_row_from_pg_row(&row)?;
                let id = stable_institution_cursor_id(cid_number.as_str());
                output.push((item, created_at, id));
            }
            Ok(page_from_rows(output, page_size))
        })
    }

    fn list_education_committees_direct(
        &self,
        province_code: &str,
        city_code: Option<&str>,
        page_size: usize,
    ) -> Result<PageResult<crate::institution::subjects::InstitutionListRow>, String> {
        let province_code = province_code.to_string();
        let city_code = city_code.map(str::to_string);
        self.with_client(move |conn| {
            let city_type = crate::institution::subjects::EDUCATION_TYPE_CITY_CITIZEN_EDU_COMMITTEE;
            let limit = i64::try_from(page_size.saturating_add(1))
                .map_err(|_| "page_size too large".to_string())?;
            // 市详情只直接显示本市确定性市公民教育委员会;
            // 国家公民教育委员会不跨市铺开,学校和 F+JY 分支机构仍走精确搜索。
            let rows = conn
                .query(
                    "SELECT s.cid_number, s.cid_full_name, s.category,
                                    s.p1, ''::text AS province_name,
                                    ''::text AS city_name, s.province_code, s.city_code, s.institution_code,
                                    s.private_type, s.partnership_kind, s.has_legal_personality,
                                    s.parent_cid_number, s.created_by, s.created_at,
                                    COALESCE(ac.account_count, 0),
                                    a.admin_name, a.institution_code, s.cid_full_name, s.cid_short_name,
                                    ''::text AS town_name, COALESCE(s.town_code, ''),
                                    s.education_type, s.status
	                     FROM subjects s
	                     JOIN gov g ON g.province_code = s.province_code AND g.cid_number = s.cid_number
	                     LEFT JOIN (
                        SELECT cid_number, COUNT(*)::BIGINT AS account_count
                        FROM accounts
                        GROUP BY cid_number
                     ) ac ON ac.cid_number = s.cid_number
                     LEFT JOIN admins a ON lower(a.admin_account) = lower(s.created_by)
                     WHERE s.kind = 'PUBLIC'
		                       AND s.status = 'ACTIVE'
		                       AND g.source = 'CHAIN'
		                       AND s.institution_code IN ('NED', 'CEDU', 'GUN', 'GSCH')
	                       AND s.education_type = $3
	                       AND s.province_code = $1
	                       AND ($2::text IS NULL OR s.city_code = $2)
	                     ORDER BY
	                        s.province_code ASC,
	                        s.city_code ASC NULLS FIRST,
	                        s.cid_number ASC
	                     LIMIT $4",
                    &[&province_code, &city_code, &city_type, &limit],
                )
                .map_err(|e| format!("query direct education committees failed: {e}"))?;
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                items.push(institution_row_from_pg_row(&row)?);
            }
            Ok(offset_page_from_window(items, 0, page_size))
        })
    }

    pub(crate) fn list_official_institutions_scope(
        &self,
        province_code: &str,
        city_code: Option<&str>,
        town_code: Option<&str>,
        keyword: &str,
        institution_code_filter: Option<&str>,
        offset: usize,
        page_size: usize,
    ) -> Result<PageResult<crate::institution::subjects::InstitutionListRow>, String> {
        let keyword = keyword.trim().to_ascii_lowercase();
        let province_code = province_code.to_string();
        let city_code = city_code.map(str::to_string);
        let town_code = town_code.map(str::to_string);
        let institution_code_filter = institution_code_filter.map(str::to_string);
        self.with_client(move |conn| {
            let limit = i64::try_from(page_size.saturating_add(1))
                .map_err(|_| "page_size too large".to_string())?;
            let offset_i64 =
                i64::try_from(offset).map_err(|_| "page offset too large".to_string())?;
	            // 公权目录 = 链上 PublicManage 投影(gov.source=CHAIN)
	            // + 公权下属非法人(F,父级为公法人)。本地 pending/手工行不能作为公权机构真源。
            // 父级只按 cid_number 关联(cid 全局唯一,父级不限定本省分区)。
            let rows = conn
                .query(
                    "SELECT s.cid_number, s.cid_full_name, s.category,
			                                    s.p1, ''::text AS province_name,
			                                    ''::text AS city_name, s.province_code, s.city_code, s.institution_code,
				                                    s.private_type, s.partnership_kind, s.has_legal_personality,
				                                    s.parent_cid_number, s.created_by, s.created_at,
			                                    COALESCE(ac.account_count, 0),
			                                    a.admin_name, a.institution_code, s.cid_full_name, s.cid_short_name,
			                                    ''::text AS town_name, COALESCE(s.town_code, ''),
			                                    s.education_type,
			                                    s.status
	                             FROM subjects s
	                             LEFT JOIN gov g ON g.province_code = s.province_code AND g.cid_number = s.cid_number
	                             LEFT JOIN subjects par ON par.cid_number = s.parent_cid_number
		                             LEFT JOIN (
	                                SELECT province_code, cid_number, COUNT(*)::BIGINT AS account_count
	                                FROM accounts
	                                WHERE province_code = $1
	                                  AND ($2::text IS NULL OR city_code = $2)
	                                GROUP BY province_code, cid_number
	                             ) ac ON ac.province_code = s.province_code AND ac.cid_number = s.cid_number
	                             LEFT JOIN admins a ON lower(a.admin_account) = lower(s.created_by)
	                             WHERE s.kind IN ('PUBLIC', 'PRIVATE')
	                               AND s.status = 'ACTIVE'
			                               AND (
			                                    (s.category = 'GOV_INSTITUTION'
			                                     AND g.cid_number IS NOT NULL
			                                     AND g.source = 'CHAIN'
			                                     AND s.institution_code NOT IN ('NED', 'CEDU', 'GUN', 'SUN', 'JUN', 'GSCH', 'SFSC', 'JSCH'))
		                                    OR (s.institution_code IN ('SFGT', 'SFGP', 'UNIN')
	                                        AND s.institution_code NOT IN ('NED', 'CEDU', 'GUN', 'SUN', 'GSCH', 'SFSC')
	                                        AND par.category = 'GOV_INSTITUTION')
	                               )
	                               AND s.province_code = $1
	                               AND ($2::text IS NULL OR s.city_code = $2)
	                               AND ($7::text IS NULL OR s.town_code = $7 OR COALESCE(s.town_code, '') = '')
	                               AND ($6::text IS NULL OR s.institution_code = $6)
	                               AND (
	                                    $3::text = ''
	                                    OR lower(s.cid_number) LIKE '%' || $3 || '%'
	                                    OR lower(COALESCE(s.cid_full_name, '')) LIKE '%' || $3 || '%'
	                                    OR lower(COALESCE(s.cid_short_name, '')) LIKE '%' || $3 || '%'
	                               )
	                             ORDER BY
		                                s.city_code ASC NULLS LAST,
		                                s.town_code ASC NULLS LAST,
		                                CASE
                                        WHEN s.institution_code IN ('NLG','NSN','NRP','PLG','PSN','PRP','CLEG') THEN 1
                                        WHEN s.institution_code IN ('NJD','PJD','CJUD') THEN 2
                                        WHEN s.institution_code IN ('NSP','PSP','CSUP','FAC','FAU','FIV') THEN 3
                                        WHEN s.institution_code IN ('NRC','PRC','PRB') THEN 4
                                        ELSE 0
                                    END ASC,
	                                COALESCE(s.cid_short_name, '') ASC,
	                                COALESCE(s.cid_full_name, '') ASC,
	                                s.cid_number ASC
	                             LIMIT $4 OFFSET $5",
                    &[&province_code, &city_code, &keyword, &limit, &offset_i64, &institution_code_filter, &town_code],
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
    let raw = std::env::var("ONCHINA_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8964".to_string());
    raw.parse::<SocketAddr>()
        .map_err(|e| format!("invalid ONCHINA_BIND_ADDR `{raw}`: {e}"))
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
        // SAFETY: `limit` 是栈上有效的 `libc::rlimit`,指针在调用期间始终有效;
        // setrlimit 只读取该结构,不保存指针,失败仅返回非 0 不产生未定义行为。
        #[allow(unsafe_code)]
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
    SyncGov,
    PurgeLegacyCid {
        dry_run: bool,
    },
    PurgeOrphanInstitutions {
        dry_run: bool,
        backup_path: Option<String>,
    },
    /// 创世机构目录全量链上双向比对(部署验收,ADR-031 D9)。
    AuditChainCatalog,
}

fn parse_backend_command() -> BackendCommand {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        return BackendCommand::Serve;
    };
    match command {
        "serve" => BackendCommand::Serve,
        "sync-gov" => BackendCommand::SyncGov,
        "audit-chain-catalog" => BackendCommand::AuditChainCatalog,
        "purge-legacy-cid" => BackendCommand::PurgeLegacyCid {
            dry_run: args.iter().any(|arg| arg == "--dry-run"),
        },
        "purge-orphan-institutions" => {
            // 默认 dry-run(只回报不删);必须显式 --apply 才落库,--apply 与
            // --dry-run 同时出现按 dry-run 处理(更安全)。--backup <path> 可覆盖默认备份文件名。
            let apply = args.iter().any(|arg| arg == "--apply");
            let dry_run = !apply || args.iter().any(|arg| arg == "--dry-run");
            BackendCommand::PurgeOrphanInstitutions {
                dry_run,
                backup_path: parse_cli_option(&args, "--backup"),
            }
        }
        other => panic!("unknown onchina command: {other}"),
    }
}

fn parse_cli_option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].trim().to_string())
        .filter(|v| !v.is_empty())
}

fn init_chain_genesis_hash_blocking() -> Result<(), String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("build chain genesis init runtime failed: {e}"))?;
    rt.block_on(core::chain_runtime::init_genesis_hash_from_chain())
}

fn log_gov_projection_report(
    label: &'static str,
    report: &domains::gov::service::GovChainProjectionReport,
) {
    info!(
        chain_institutions = report.chain_institutions,
        chain_accounts = report.chain_accounts,
        local_institutions = report.local_institutions,
        local_accounts = report.local_accounts,
        institution_rows_changed = report.institution_rows_changed,
        account_rows_changed = report.account_rows_changed,
        obsolete_subjects_removed = report.obsolete_subjects_removed,
        obsolete_accounts_removed = report.obsolete_accounts_removed,
        chain_genesis_hash = %report.chain_genesis_hash,
        label,
        "cid gov chain projection synced"
    );
}

fn run_gov_chain_projection_command(state: &AppState, command: BackendCommand) -> bool {
    match command {
        BackendCommand::Serve => false,
        BackendCommand::AuditChainCatalog => {
            init_chain_genesis_hash_blocking()
                .unwrap_or_else(|e| panic!("init chain genesis hash failed: {e}"));
            crate::domains::gov::chain_audit::full_audit_blocking()
                .unwrap_or_else(|e| panic!("audit-chain-catalog failed: {e}"));
            true
        }
        BackendCommand::SyncGov => {
            init_chain_genesis_hash_blocking()
                .unwrap_or_else(|e| panic!("init chain genesis hash failed: {e}"));
            let report =
                crate::domains::gov::service::sync_gov_chain_projection_blocking(&state.db)
                    .unwrap_or_else(|e| panic!("sync-gov failed: {e}"));
            log_gov_projection_report("sync-gov", &report);
            true
        }
        BackendCommand::PurgeLegacyCid { dry_run } => {
            run_purge_legacy_cid(state, dry_run);
            true
        }
        BackendCommand::PurgeOrphanInstitutions {
            dry_run,
            backup_path,
        } => {
            run_purge_orphan_institutions(state, dry_run, backup_path.as_deref());
            true
        }
    }
}

#[derive(Debug)]
struct PurgeReport {
    legacy_count: usize,
    private_count: usize,
    citizen_count: usize,
    per_table_deleted: Vec<(&'static str, u64)>,
    dry_run: bool,
}

// 清掉所有不合规 CID 号。PUBLIC 公权机构删后只能从链上唯一真源重建投影;
// PRIVATE 私权机构与公民属注册局直接录入,删后无法自动重建,需由注册局重新录入。
fn run_purge_legacy_cid(state: &AppState, dry_run: bool) {
    let report = state
        .db
        .purge_legacy_cid_rows(dry_run)
        .unwrap_or_else(|e| panic!("purge-legacy-cid failed: {e}"));
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
        "cid legacy purge finished"
    );
    if report.dry_run {
        info!("purge-legacy-cid dry-run: no rows changed; re-run without --dry-run to apply");
        return;
    }
    if report.legacy_count == 0 {
        info!("purge-legacy-cid: no legacy cid rows; skip chain projection sync");
        return;
    }
    if report.private_count > 0 || report.citizen_count > 0 {
        warn!(
            private_permanently_deleted = report.private_count,
            citizen_permanently_deleted = report.citizen_count,
            "purge-legacy-cid removed user-created PRIVATE/CITIZEN rows; they must be re-created/re-bound to get new-scheme cid"
        );
    }
    init_chain_genesis_hash_blocking()
        .unwrap_or_else(|e| panic!("purge-legacy-cid init chain genesis hash failed: {e}"));
    let projection = crate::domains::gov::service::sync_gov_chain_projection_blocking(&state.db)
        .unwrap_or_else(|e| panic!("purge-legacy-cid chain projection sync failed: {e}"));
    log_gov_projection_report("purge-legacy-cid", &projection);
}

// 一条孤儿机构记录(subjects 行 town_code 指向 china.sqlite 已退役/不存在的镇)。
#[derive(Debug, Clone)]
struct OrphanInstitution {
    province_code: String,
    cid_number: String,
    kind: String,
    city_code: String,
    town_code: String,
    town_name: String,
    category: String,
    institution_code: String,
}

// 清理孤儿机构 CLI。
// `purge-orphan-institutions [--dry-run|--apply] [--backup <path>]`,默认 dry-run。
// 孤儿 = subjects.town_code 非空 + (province_code,city_code,town_code) 不在 china.sqlite 内存树。
//   - dry-run:只打印孤儿清单(onchina/town/town_code/category/institution_code/原因)+ 总数,
//     供人工核对无一命中冻结常量号(储委会/部委)。
//   - apply:先把待删行导出到 purge_orphan_backup_<...>.sql(删除唯一回滚保证),
//     再逐省单事务级联删(accounts→docs→audit→gov|private→ids→subjects)。
// 红线:绝不动 cid_number;绝不删空 town_code 行(已在扫描层白名单过滤);不碰号生成/链/省市码。
fn run_purge_orphan_institutions(state: &AppState, dry_run: bool, backup_path: Option<&str>) {
    let orphans = state
        .db
        .scan_orphan_institutions()
        .unwrap_or_else(|e| panic!("purge-orphan-institutions scan failed: {e}"));

    if orphans.is_empty() {
        info!(
            dry_run,
            "purge-orphan-institutions: no orphan institutions found; nothing to do"
        );
        return;
    }

    // 打印孤儿清单(dry-run 与 apply 都先打,apply 时即删除前留痕)。
    for o in &orphans {
        info!(
            cid_number = %o.cid_number,
            kind = %o.kind,
            province_code = %o.province_code,
            city_code = %o.city_code,
            town_code = %o.town_code,
            town_name = %o.town_name,
            category = %o.category,
            institution_code = %o.institution_code,
            reason = "town (province_code,city_code,town_code) not found in china.sqlite (retired/reused town code)",
            "orphan institution"
        );
    }
    info!(
        dry_run,
        orphan_total = orphans.len(),
        "purge-orphan-institutions scan finished"
    );

    if dry_run {
        info!(
            "purge-orphan-institutions dry-run: no rows changed; review the list above (verify no frozen-constant cid e.g. reserve-committee/ministry is hit) then re-run with --apply to delete"
        );
        return;
    }

    // 按省分组 + 按 kind 拆 gov/private,供逐省事务级联删与备份导出复用。
    let mut gov_by_province: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut private_by_province: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut all_by_province: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for o in &orphans {
        all_by_province
            .entry(o.province_code.clone())
            .or_default()
            .push(o.cid_number.clone());
        match o.kind.as_str() {
            "PRIVATE" => private_by_province
                .entry(o.province_code.clone())
                .or_default()
                .push(o.cid_number.clone()),
            // PUBLIC 及其它(默认按公权机构 gov 表处理)。
            _ => gov_by_province
                .entry(o.province_code.clone())
                .or_default()
                .push(o.cid_number.clone()),
        }
    }

    // 1. 删除前导出待删行(删除唯一回滚保证)。
    let resolved_backup = backup_path.map(str::to_string).unwrap_or_else(|| {
        format!(
            "purge_orphan_backup_{}.sql",
            Utc::now().format("%Y%m%d%H%M%S")
        )
    });
    state
        .db
        .export_orphan_backup(&all_by_province, &resolved_backup)
        .unwrap_or_else(|e| panic!("purge-orphan-institutions backup failed: {e}"));
    info!(
        backup = %resolved_backup,
        provinces = all_by_province.len(),
        orphan_total = orphans.len(),
        "purge-orphan-institutions backup written before delete"
    );

    // 2. 逐省单事务级联删(禁止跨省一条 SQL)。
    let mut deleted_total: u64 = 0;
    for (province, _cids) in &all_by_province {
        let gov_cids = gov_by_province.get(province).cloned().unwrap_or_default();
        let private_cids = private_by_province
            .get(province)
            .cloned()
            .unwrap_or_default();
        let deleted = state
            .db
            .delete_orphan_institutions_by_province(province, &gov_cids, &private_cids)
            .unwrap_or_else(|e| panic!("purge-orphan-institutions delete failed: {e}"));
        info!(
            province_code = %province,
            gov_deleted = gov_cids.len(),
            private_deleted = private_cids.len(),
            subjects_deleted = deleted,
            "purge-orphan-institutions province purged"
        );
        deleted_total += deleted;
    }
    warn!(
        orphan_total = orphans.len(),
        subjects_deleted = deleted_total,
        backup = %resolved_backup,
        "purge-orphan-institutions applied: orphan institutions removed (rollback only via backup file)"
    );
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();
    disable_core_dumps();
    let command = parse_backend_command();

    // `ONCHINA_SIGNING_SEED_HEX` 是链上中国平台系统签名钥的**可选**配置,
    // 不是节点/服务启动前提。任何区块链节点都可启动 OnChina;需要签登录 QR 挑战、
    // 人口快照或机构注册凭证的操作才按需读取它。鉴权真源是链上 Active 管理员集合。
    if let Ok(seed_hex) = std::env::var("ONCHINA_SIGNING_SEED_HEX") {
        if !seed_hex.trim().is_empty() {
            crypto::sr25519::try_load_signing_key_from_seed(seed_hex.as_str())
                .unwrap_or_else(|e| panic!("invalid ONCHINA_SIGNING_SEED_HEX: {e}"));
        }
    }
    // (Card 05):桌面/小市内嵌私有 PostgreSQL——onchina 自管 initdb/起停,
    // 自拼本机 DATABASE_URL;大市外部托管 PG 时关 ONCHINA_EMBEDDED_PG,直接给 DATABASE_URL。
    let database_url = if core::embedded_pg::is_enabled() {
        core::embedded_pg::ensure_started()
            .unwrap_or_else(|e| panic!("embedded postgres start failed: {e}"))
    } else {
        required_env("DATABASE_URL")
    };
    if database_url
        .to_ascii_lowercase()
        .contains("sslmode=disable")
    {
        panic!("DATABASE_URL must not use sslmode=disable");
    }
    let db_is_local = database_url_targets_local_host_only(database_url.as_str())
        .unwrap_or_else(|e| panic!("{e}"));
    if !db_is_local && !env_flag_enabled("ONCHINA_ALLOW_REMOTE_DB_WITHOUT_TLS") {
        panic!(
            "DATABASE_URL points to non-local host, but sync postgres client is running in NoTls mode; set ONCHINA_ALLOW_REMOTE_DB_WITHOUT_TLS=true only if transport is protected externally"
        );
    }
    let db = Db::from_database_url(database_url.as_str()).expect("init database");
    let state = AppState {
        db,
        rate_limiter: Arc::new(LocalRateLimiter::new()),
    };
    info!("initialized database state with defaults");
    if run_gov_chain_projection_command(&state, command.clone()) {
        return;
    }
    init_chain_genesis_hash_blocking()
        .unwrap_or_else(|e| panic!("init chain genesis hash failed: {e}"));
    // 公权机构唯一真源是链上 PublicManage。本地 PostgreSQL 只是查询缓存:
    // 启动时先比对链 genesis/finalized head,只有链有变化或本地无有效投影时才全量刷新。
    // 链不可达或投影无法确认时仍 fail-closed,避免回退到本地重新生成公权机构。
    let projection_current =
        crate::domains::gov::service::chain_projection_matches_current_head_blocking(&state.db)
            .unwrap_or_else(|e| panic!("check cid gov chain projection anchor failed: {e}"));
    if projection_current {
        info!("cid gov chain projection is current; skip startup full sync");
    } else {
        let projection =
            crate::domains::gov::service::sync_gov_chain_projection_blocking(&state.db)
                .unwrap_or_else(|e| panic!("cid gov chain projection sync failed: {e}"));
        log_gov_projection_report("serve-startup", &projection);
    }
    // 创世机构目录链上抽样对账(ADR-031 D9,fail-closed):只校验 runtime/onchina
    // 派生规则与链上创世写入仍一致,不得作为 OnChina 本地公权机构生成来源。
    // ONCHINA_GOV_CHAIN_AUDIT=0 为开发无链环境逃生门。
    if std::env::var("ONCHINA_GOV_CHAIN_AUDIT")
        .map(|v| v != "0")
        .unwrap_or(true)
    {
        crate::domains::gov::chain_audit::startup_sample_audit_blocking()
            .unwrap_or_else(|e| panic!("创世机构目录链上对账失败(fail-closed): {e}"));
    } else {
        warn!("ONCHINA_GOV_CHAIN_AUDIT=0,已跳过创世机构目录链上对账(仅限开发)");
    }

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
        // 链上集合鉴权(3b)——后台周期复查,清退已不在链上 Active 集合的管理员会话。
        if core::chain_url::chain_ws_url().is_ok() {
            tokio::spawn(auth::login::revoke_stale_admin_sessions_loop(
                state.db.clone(),
            ));
        }

        let auth_routes = Router::new()
            .route(
                "/api/v1/admin/auth/check",
                get(auth::login::admin_auth_check),
            )
            .route(
                "/api/v1/admin/auth/logout",
                post(auth::login::admin_logout),
            )
            .route(
                "/api/v1/admin/auth/identify",
                post(auth::login::admin_auth_identify),
            )
            .route(
                "/api/v1/admin/auth/challenge",
                post(auth::login::admin_auth_challenge),
            )
            .route(
                "/api/v1/admin/auth/verify",
                post(auth::login::admin_auth_verify),
            )
            .route(
                "/api/v1/admin/auth/qr/sign-request",
                post(auth::login::admin_auth_qr_sign_request),
            )
            .route(
                "/api/v1/admin/auth/qr/complete",
                post(auth::login::admin_auth_qr_complete),
            )
            .route(
                "/api/v1/admin/auth/qr/result",
                get(auth::login::admin_auth_qr_result),
            )
            .route(
                "/api/v1/admin/auth/node-binding/confirm",
                post(auth::login::admin_auth_confirm_node_binding),
            );

        let admin_routes = Router::new()
            .route(
                "/api/v1/admin/city-registry-admins",
                get(auth::list_city_registry_admins),
            )
            .route(
                "/api/v1/admin/actions/prepare",
                post(auth::actions::prepare_admin_action),
            )
            .route(
                "/api/v1/admin/actions/commit",
                post(auth::actions::commit_admin_action),
            )
            .route(
                "/api/v1/admin/auth/passkey/register/begin",
                post(auth::passkey::register_begin),
            )
            .route(
                "/api/v1/admin/auth/passkey/register/finish",
                post(auth::passkey::register_finish),
            )
            .route(
                "/api/v1/admin/auth/passkey/assert/begin",
                post(auth::passkey::assert_begin),
            )
            .route(
                "/api/v1/admin/auth/passkey/assert/finish",
                post(auth::passkey::assert_finish),
            )
            .route(
                "/api/v1/admin/auth/passkey/status",
                get(auth::passkey::passkey_status),
            )
            .route(
                "/api/v1/admin/federal-registry-admins",
                get(auth::list_federal_registry_admins),
            )
            .route(
                "/api/v1/admin/own-institution-admins",
                get(auth::list_own_institution_admins),
            )
            .route(
                "/api/v1/admin/own-institution",
                get(auth::get_own_institution),
            )
            // 机构相关 API 外部路径保持稳定,内部按 subjects/gov/private/accounts/docs 归属。
            // - GET  /api/v1/institution/check-cid-full-name             — cid_full_name 查重
            // - POST /api/v1/institution/create                          — 公权/教育通用机构生成(不上链)
            // - POST /api/v1/private/<type>                              — 六类私权机构专属生成入口
            // - POST /api/v1/institution/:cid_number/account/create         — 只登记账户名称,不上链
            // - GET  /api/v1/institution/list                            — 公权/教育按 scope 过滤的机构列表
            // - GET  /api/v1/private/<type>                              — 六类私权机构专属列表入口
            // - GET  /api/v1/institution/:cid_number                        — 机构详情
            // - GET  /api/v1/institution/:cid_number/accounts               — 账户列表
            // - DELETE /api/v1/institution/:cid_number/account/:account_name — 删除未上链/已注销新增账户
            .route(
                "/api/v1/institution/check-cid-full-name",
                get(institution::subjects::admin::check_cid_full_name),
            )
            // F 详情页"所属法人"搜索(全国范围 S/G 模糊匹配)
            .route(
                "/api/v1/institution/search-parents",
                get(institution::subjects::admin::search_parent_institutions),
            )
            .route(
                "/api/v1/institution/legal-representative/photo",
                post(institution::subjects::admin::upload_legal_representative_photo),
            )
            .route(
                "/api/v1/institution/create",
                post(institution::subjects::registration::create_institution),
            )
            .route(
                "/api/v1/private/sole",
                get(domains::private::sole::list).post(domains::private::sole::create),
            )
            .route(
                "/api/v1/private/partnership",
                get(domains::private::partnership::list).post(domains::private::partnership::create),
            )
            .route(
                "/api/v1/private/company",
                get(domains::private::company::list).post(domains::private::company::create),
            )
            .route(
                "/api/v1/private/corporation",
                get(domains::private::corporation::list).post(domains::private::corporation::create),
            )
            .route(
                "/api/v1/private/welfare",
                get(domains::private::welfare::list).post(domains::private::welfare::create),
            )
            .route(
                "/api/v1/private/association",
                get(domains::private::association::list).post(domains::private::association::create),
            )
            .route(
                "/api/v1/institution/:cid_number/account/create",
                post(institution::accounts::handler::create_account),
            )
            .route(
                "/api/v1/institution/list",
                get(institution::subjects::registration::list_institutions),
            )
            .route(
                "/api/v1/institution/:cid_number",
                get(institution::subjects::admin::get_institution)
                    // 详情页只更新机构资料;私权类型由创建时身份编码锁定,不可在此改。
                    .patch(institution::subjects::admin::update_institution),
            )
            .route(
                "/api/v1/institution/:cid_number/accounts",
                get(institution::accounts::handler::list_accounts),
            )
            .route(
                "/api/v1/institution/:cid_number/account/:account_name",
                delete(institution::accounts::handler::delete_account),
            )
            // 机构资料库文档 CRUD
            .route(
                "/api/v1/institution/:cid_number/documents",
                get(domains::docs::handler::list_documents).post(domains::docs::handler::upload_document),
            )
            .route(
                "/api/v1/institution/:cid_number/documents/:doc_id/download",
                get(domains::docs::handler::download_document),
            )
            .route(
                "/api/v1/institution/:cid_number/documents/:doc_id",
                delete(domains::docs::handler::delete_document),
            )
            .route(
                "/api/v1/institutions/official",
                get(domains::gov::handler::list_official_institutions),
            )
            // 立法与表决:发起/院内表决(返回扫码上链 sign_request)+ 读法律/读提案进度。
            .route(
                "/api/legislation/proposable",
                get(domains::legislation::handler::list_proposable),
            )
            .route(
                "/api/legislation/propose",
                post(domains::legislation::handler::propose_legislation),
            )
            .route(
                "/api/legislation/house-vote",
                post(domains::legislation::handler::cast_house_vote),
            )
            .route(
                "/api/legislation/laws",
                get(domains::legislation::handler::list_laws),
            )
            // 本节点绑定机构层级/辖区的法律(会话派生 scope);静态段先于 :law_id 匹配。
            .route(
                "/api/legislation/laws/mine",
                get(domains::legislation::handler::list_my_laws),
            )
            .route(
                "/api/legislation/laws/:law_id",
                get(domains::legislation::handler::get_law),
            )
            .route(
                "/api/legislation/proposals/:proposal_id",
                get(domains::legislation::handler::get_proposal_state),
            )
            // 联邦注册局机构详情(只读,绕过 scope,所有联邦注册局管理员可读)
            .route(
                "/api/v1/institutions/federal-registry",
                get(institution::subjects::admin::get_federal_registry),
            )
            .route(
                "/api/v1/admin/audit-logs",
                get(audit::admin_list_audit_logs),
            )
            // 建档占号先行(ADR-031):POST = 占号 prepare(返回冷签 QR),
            // 链上进块后经 chain/submit 才落档案;列表查询走 GET。
            .route(
                "/api/v1/admin/citizens",
                get(domains::citizens::handler::admin_list_citizens)
                    .post(domains::citizens::occupy::prepare_citizen_occupy),
            )
            .route(
                "/api/v1/admin/citizens/chain/submit",
                post(domains::citizens::occupy::submit_chain_sign),
            )
            .route(
                "/api/v1/admin/citizens/:cid_number/onchain/revoke/prepare",
                post(domains::citizens::occupy::prepare_citizen_revoke),
            )
            .route(
                "/api/v1/admin/citizens/legal-representatives",
                get(domains::citizens::handler::admin_search_legal_representative_citizens),
            )
            .route(
                "/api/v1/admin/citizens/:cid_number/documents",
                get(domains::citizens::handler::list_citizen_documents)
                    .post(domains::citizens::handler::upload_citizen_document),
            )
            .route(
                "/api/v1/admin/citizens/:cid_number/documents/:doc_id/download",
                get(domains::citizens::handler::download_citizen_document),
            )
            .route(
                "/api/v1/admin/citizens/:cid_number/documents/:doc_id",
                delete(domains::citizens::handler::delete_citizen_document),
            )
            .route(
                "/api/v1/admin/citizens/:cid_number/onchain/prepare",
                post(domains::citizens::chain_identity::prepare_citizen_onchain_signature),
            )
            .route(
                "/api/v1/admin/citizens/:cid_number/onchain/complete",
                post(domains::citizens::chain_identity::complete_citizen_onchain_signature),
            )
            .route("/api/v1/admin/cid/meta", get(cid::admin::admin_cid_meta))
            .route(
                "/api/v1/admin/cid/china/cities",
                get(cid::china::admin::admin_china_cities),
            )
            .route(
                "/api/v1/admin/cid/china/towns",
                get(cid::china::admin::admin_china_towns),
            )
            .route(
                "/api/v1/admin/address/names",
                get(domains::address::handler::list_address_names),
            )
            .route(
                "/api/v1/admin/address/items",
                get(domains::address::handler::list_addresses),
            )
            .route(
                "/api/v1/admin/address/chain-call",
                post(domains::address::handler::prepare_chain_call),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth::login::require_admin_session_middleware,
            ));

        // 链端 pull 通道全部走 app_routes 命名空间。

        let public_routes = Router::new()
            // 根路径 `/` 让给前端 SPA(经 fallback_service 的 ServeDir → index.html),
            // 健康检查走专用 `/api/v1/health`;否则浏览器访问注册局只会看到健康 JSON。
            .route("/api/v1/health", get(health))
            .route(
                "/api/v1/platform/ca-certificate",
                get(organization_ca_certificate),
            )
            .route(
                "/api/v1/platform/ca-certificate/info",
                get(organization_ca_certificate_info),
            )
            .route(
                "/api/v1/public/identity/search",
                get(domains::citizens::handler::public_identity_search),
            )
            // 立法大屏只读看板(免登录):机构由节点绑定确定,不接受请求参数(fail-closed)。
            .route(
                "/api/public/legislation/display/board",
                get(domains::legislation::display::handler::display_board),
            );

        // App routes:CitizenApp 与节点桌面链端 pull 用的统一命名空间。
        //
        // 链端公开查询按所属业务模块落位,例如 institution::subjects::chain_multisig_info、
        // domains::citizens::chain_joint_vote 和 domains::citizens::chain_vote。
        // CitizenApp 自有功能(钱包交易索引、电子护照状态查询)继续留 indexer / citizens 模块。
        let app_routes = Router::new()
            // ── 联合投票:查询本地公民人数,链端快照由 citizen-identity 负责 ──
            .route(
                "/api/v1/app/voters/count",
                get(domains::citizens::chain_joint_vote::app_voters_count),
            )
            // ── 投票资格提示:提交交易前查询本地档案资格,链端资格由 citizen-identity 负责 ──
            .route(
                "/api/v1/app/vote/eligibility",
                post(domains::citizens::chain_vote::app_vote_eligibility),
            )
            // ── 钱包交易索引(CitizenApp 自有,与链交互无关) ──
            .route(
                "/api/v1/app/wallet/:address/transactions",
                get(indexer::api::wallet_transactions),
            )
            // ── CitizenApp 电子护照状态查询 ──
            .route(
                "/api/v1/app/myid/status",
                get(domains::citizens::vote::app_myid_status),
            )
            // ── 机构信息查询(链端/钱包 pull):机构搜索 / 详情 / 注册信息凭证 / 账户列表 ──
            .route(
                "/api/v1/app/institutions/search",
                get(institution::subjects::chain_multisig_info::app_search_institutions),
            )
            .route(
                "/api/v1/app/institutions/:cid_number/registration-info",
                get(institution::subjects::chain_multisig_info::app_get_institution_registration_info),
            )
            .route(
                "/api/v1/app/institutions/:cid_number/deregistration-info",
                get(institution::subjects::chain_multisig_info::app_get_institution_deregistration_info),
            )
            .route(
                "/api/v1/app/institutions/:cid_number",
                get(institution::subjects::chain_multisig_info::app_get_institution),
            )
            .route(
                "/api/v1/app/institutions/:cid_number/accounts",
                get(institution::subjects::chain_multisig_info::app_list_accounts),
            )
            // ── 公权机构目录(CitizenApp BFF,匿名只读,数据来自链上公权机构投影)──
            .route(
                "/api/v1/app/public-institutions",
                get(citizenapp::public_institution::list_public_institutions),
            )
            .route(
                "/api/v1/app/public-institutions/version",
                get(citizenapp::public_institution::public_institutions_version),
            );

        let app_state = state.clone();
        // onchina 控制台同源托管管理员前端 dist;未匹配路径回退 index.html(SPA 路由)。
        // 静态服务置于限流层之后,静态资源不走全局限流;dist 路径可由 ONCHINA_FRONTEND_DIST 覆盖。
        let frontend_dist = std::env::var("ONCHINA_FRONTEND_DIST")
            .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist").to_string());
        let frontend_index = format!("{frontend_dist}/index.html");
        let frontend_service = tower_http::services::ServeDir::new(&frontend_dist)
            .not_found_service(tower_http::services::ServeFile::new(&frontend_index));
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
            .fallback_service(frontend_service)
            .with_state(app_state);

        // 联邦注册局管理员采用同级模型;省域映射只做 CID 管辖元数据,
        // 管理员成员资格仍以链上 Active 集合为准,本地只允许同省更换投影。

        // 本地手机联调时必须监听到与 App 可访问的一致地址，避免只绑定回环导致超时。
        let addr = resolve_backend_bind_addr().expect("resolve onchina backend bind address");

        // (Card 05):收退出信号(Ctrl-C / node 停子进程 SIGTERM)→ 优雅停内嵌 PG → 退出。
        // 内嵌关闭时无操作;daemon 化的 postgres 即便被强杀也会在下次 ensure_started 复用。
        tokio::spawn(async {
            shutdown_signal().await;
            core::embedded_pg::stop();
            std::process::exit(0);
        });

        serve_console(addr, app).await;
    });
}

/// (Card 05):内网 TLS——正式入口固定为 https://onchina.local:8964;
/// `ONCHINA_ENABLE_TLS` 关闭仅保留给底层开发调试。
async fn serve_console(addr: SocketAddr, app: axum::Router) {
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    // 广告 onchina.local mDNS:局域网内可经统一 HTTPS 域名访问本节点控制台(best-effort)。
    platform::mdns::advertise(addr.port());
    if core::tls::is_enabled() {
        let config = core::tls::load_or_generate_rustls_config()
            .await
            .unwrap_or_else(|e| panic!("onchina tls config failed: {e}"));
        info!("onchina console listening on https://{}", addr);
        axum_server::bind_rustls(addr, config)
            .serve(service)
            .await
            .expect("run onchina backend https server");
    } else {
        info!("onchina console listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("bind onchina backend listener");
        axum::serve(listener, service)
            .await
            .expect("run onchina backend server");
    }
}

/// 下载本机构节点私有 CA 公钥证书。
///
/// 该接口必须保持未登录可访问,否则员工首次访问不可信 HTTPS 时无法先下载证书。
/// 只返回 CA 公钥证书 PEM;CA 私钥只落在服务器本地 `ONCHINA_TLS_DIR`。
async fn organization_ca_certificate() -> impl IntoResponse {
    match core::tls::organization_ca_certificate_pem() {
        Ok(pem) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/x-x509-ca-cert"),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"onchina-org-root-ca.crt\""),
            );
            (headers, pem).into_response()
        }
        Err(err) => {
            warn!(error = %err, "onchina CA certificate download failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1500,
                "onchina ca certificate unavailable",
            )
        }
    }
}

async fn organization_ca_certificate_info() -> impl IntoResponse {
    match core::tls::organization_ca_certificate_info() {
        Ok(info) => Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: OrganizationCaCertificateInfoView {
                filename: info.filename,
                sha256: info.sha256,
                subject: info.subject,
                valid_until: info.valid_until,
            },
        })
        .into_response(),
        Err(err) => {
            warn!(error = %err, "onchina CA certificate info failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1500,
                "onchina ca certificate unavailable",
            )
        }
    }
}

/// 退出信号:Ctrl-C 全平台;Unix 额外捕获 SIGTERM(node 停子进程默认信号)。
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    {
        let mut term =
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(s) => s,
                Err(_) => {
                    ctrl_c.await;
                    return;
                }
            };
        tokio::select! {
            _ = ctrl_c => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        ctrl_c.await;
    }
}

// chain pull 端点(multisig_info / joint_vote / vote_eligibility)无 attestor 鉴权需求,
// 全局 rate limiter 防滥用,凭证签名本身就是反伪造保护。

fn api_error(status: StatusCode, code: u32, message: &str) -> axum::response::Response {
    (
        status,
        Json(ApiError {
            code,
            error_code: onchina_error_code(status, message),
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
        .into_response()
}

fn onchina_error_code(status: StatusCode, message: &str) -> &'static str {
    // HTTP 状态表达协议层含义,稳定 error_code 表达业务语义;前端不得解析 message。
    match message {
        "missing bearer token" => "ONCHINA_AUTH_MISSING_TOKEN",
        "invalid access token" => "ONCHINA_AUTH_INVALID_ACCESS_TOKEN",
        "access token expired" => "ONCHINA_AUTH_ACCESS_TOKEN_EXPIRED",
        "admin disabled" => "ONCHINA_AUTH_ADMIN_DISABLED",
        "permission denied" => "ONCHINA_AUTH_PERMISSION_DENIED",
        "identity_qr is required" => "ONCHINA_LOGIN_IDENTITY_QR_REQUIRED",
        "admin_account is required" => "ONCHINA_LOGIN_ADMIN_ACCOUNT_REQUIRED",
        "origin is required" => "ONCHINA_LOGIN_ORIGIN_REQUIRED",
        "session_id is required" => "ONCHINA_LOGIN_SESSION_REQUIRED",
        "domain is required" => "ONCHINA_LOGIN_DOMAIN_REQUIRED",
        "challenge_id, origin, session_id, nonce, signature are required" => {
            "ONCHINA_LOGIN_REQUEST_INVALID"
        }
        "challenge_id, admin_account, signature are required" => "ONCHINA_LOGIN_REQUEST_INVALID",
        "challenge_id and session_id are required" => "ONCHINA_LOGIN_RESULT_PARAM_REQUIRED",
        "admin not found" => "ONCHINA_LOGIN_ADMIN_NOT_FOUND",
        "admin province scope missing" => "ONCHINA_LOGIN_ADMIN_SCOPE_MISSING",
        "sign request not found" => "ONCHINA_LOGIN_CHALLENGE_NOT_FOUND",
        "sign request already consumed" => "ONCHINA_LOGIN_CHALLENGE_CONSUMED",
        "sign request session mismatch" => "ONCHINA_LOGIN_SESSION_MISMATCH",
        "sign request expired" => "ONCHINA_LOGIN_CHALLENGE_EXPIRED",
        "signer_pubkey must match admin_account" => "ONCHINA_LOGIN_SIGNER_MISMATCH",
        "login signature verify failed" => "ONCHINA_LOGIN_SIGNATURE_VERIFY_FAILED",
        "challenge not found" | "challenge not found or expired" => {
            "ONCHINA_LOGIN_CHALLENGE_NOT_FOUND"
        }
        "challenge already consumed" => "ONCHINA_LOGIN_CHALLENGE_CONSUMED",
        "challenge expired" => "ONCHINA_LOGIN_CHALLENGE_EXPIRED",
        "challenge context mismatch" => "ONCHINA_LOGIN_CONTEXT_MISMATCH",
        "chain unreachable" => "ONCHINA_LOGIN_CHAIN_UNREACHABLE",
        "desktop governance institution is not supported by OnChina" => {
            "ONCHINA_LOGIN_DESKTOP_GOVERNANCE_UNSUPPORTED"
        }
        "personal multisig is not supported by OnChina" => {
            "ONCHINA_LOGIN_PERSONAL_MULTISIG_UNSUPPORTED"
        }
        "node binding required" => "ONCHINA_LOGIN_NODE_BINDING_REQUIRED",
        "node binding missing" => "ONCHINA_LOGIN_NODE_BINDING_MISSING",
        "node binding invalid" => "ONCHINA_LOGIN_NODE_BINDING_INVALID",
        "node binding query failed" => "ONCHINA_LOGIN_NODE_BINDING_QUERY_FAILED",
        "node binding already inactive" => "ONCHINA_LOGIN_NODE_BINDING_ALREADY_INACTIVE",
        "node binding challenge not found" => "ONCHINA_LOGIN_NODE_BINDING_CHALLENGE_NOT_FOUND",
        "node binding challenge already consumed" => {
            "ONCHINA_LOGIN_NODE_BINDING_CHALLENGE_CONSUMED"
        }
        "node binding challenge expired" => "ONCHINA_LOGIN_NODE_BINDING_CHALLENGE_EXPIRED",
        "binding_challenge_id and candidate_id are required" => {
            "ONCHINA_LOGIN_NODE_BINDING_REQUEST_INVALID"
        }
        "selected institution candidate not found" => {
            "ONCHINA_LOGIN_NODE_BINDING_CANDIDATE_NOT_FOUND"
        }
        "admin no longer belongs to selected institution" => {
            "ONCHINA_LOGIN_NODE_BINDING_ADMIN_MISMATCH"
        }
        "login persist failed" => "ONCHINA_LOGIN_PERSIST_FAILED",
        "challenge wallet mismatch" => "ONCHINA_BIND_WALLET_MISMATCH",
        "signature verify failed" => "ONCHINA_BIND_SIGNATURE_VERIFY_FAILED",
        "onchina ca certificate unavailable" => "ONCHINA_TLS_CA_UNAVAILABLE",
        "admin admin_account already exists as federal admin" => {
            "ONCHINA_ADMIN_ACCOUNT_EXISTS_AS_FEDERAL_REGISTRY"
        }
        "admin admin_account already exists as city admin" => {
            "ONCHINA_ADMIN_ACCOUNT_EXISTS_AS_CITY_REGISTRY"
        }
        "city admin city limit reached" => "ONCHINA_ADMIN_CITY_REGISTRY_CITY_LIMIT_REACHED",
        "replacement admin is not an on-chain admin" => "ONCHINA_ADMIN_REPLACEMENT_NOT_ONCHAIN",
        "not an on-chain admin" => "ONCHINA_LOGIN_ADMIN_NOT_ONCHAIN",
        "security grant required" => "ONCHINA_ADMIN_SECURITY_GRANT_REQUIRED",
        _ if message.starts_with("insert qr sign request failed") => {
            "ONCHINA_LOGIN_CHALLENGE_CREATE_FAILED"
        }
        _ if message.starts_with("query admin failed") => "ONCHINA_LOGIN_ADMIN_QUERY_FAILED",
        _ if message.starts_with("query admin scope failed") => "ONCHINA_LOGIN_ADMIN_QUERY_FAILED",
        _ if message.starts_with("build login qr signature failed") => {
            "ONCHINA_LOGIN_SYSTEM_SIGN_FAILED"
        }
        _ if message.starts_with("complete qr login failed") => "ONCHINA_LOGIN_COMPLETE_FAILED",
        _ if message.starts_with("persist qr login result failed") => {
            "ONCHINA_LOGIN_RESULT_SAVE_FAILED"
        }
        _ if message.starts_with("query qr login result failed") => {
            "ONCHINA_LOGIN_RESULT_QUERY_FAILED"
        }
        _ if message.starts_with("insert challenge failed") => {
            "ONCHINA_LOGIN_CHALLENGE_CREATE_FAILED"
        }
        _ if message.starts_with("verify login failed") => "ONCHINA_LOGIN_VERIFY_FAILED",
        _ if status == StatusCode::UNAUTHORIZED => "ONCHINA_AUTH_UNAUTHORIZED",
        _ if status == StatusCode::FORBIDDEN => "ONCHINA_AUTH_FORBIDDEN",
        _ if status == StatusCode::BAD_REQUEST => "ONCHINA_REQUEST_INVALID",
        _ if status == StatusCode::NOT_FOUND => "ONCHINA_RESOURCE_NOT_FOUND",
        _ if status == StatusCode::CONFLICT => "ONCHINA_RESOURCE_CONFLICT",
        _ if status == StatusCode::GONE => "ONCHINA_RESOURCE_EXPIRED",
        _ if status == StatusCode::UNPROCESSABLE_ENTITY => "ONCHINA_BUSINESS_VALIDATION_FAILED",
        _ if status == StatusCode::TOO_MANY_REQUESTS => "ONCHINA_RATE_LIMITED",
        _ if status == StatusCode::SERVICE_UNAVAILABLE => "ONCHINA_SERVICE_UNAVAILABLE",
        _ => "ONCHINA_INTERNAL_ERROR",
    }
}
