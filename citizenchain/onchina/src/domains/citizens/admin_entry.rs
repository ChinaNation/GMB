//! 注册局直接录入公民 handler。
//!
//! 公民由注册局管理员在办理市一次性录入。请求只提交公民档案字段和一个钱包账户;
//! 身份 CID、护照号、护照有效期由服务端确定性生成并落库。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cid::{generate_cid_number, GenerateCidInput};
use crate::crypto::pubkey::{pubkey_hex_to_ss58, ss58_to_pubkey_hex};
use crate::domains::citizens::passport_no::{
    generate_passport_no_with_retry, is_voting_age_at, passport_valid_from, passport_valid_until,
    passport_validity_years,
};
use crate::*;

/// 直接录入公民请求 DTO。
///
/// 中文注释:居住省市不由前端提交,固定来自当前注册局办理上下文。
/// `wallet_account` 可传 SS58 地址或 0x 公钥;前端只展示 SS58 地址。
#[derive(Deserialize)]
pub(crate) struct AdminCreateCitizenInput {
    pub(crate) citizen_full_name: String,
    pub(crate) citizen_sex: String,
    pub(crate) citizen_birth_date: String,
    pub(crate) residence_town_code: String,
    pub(crate) birth_province_code: String,
    pub(crate) birth_city_code: String,
    pub(crate) birth_town_code: String,
    pub(crate) voting_eligible: bool,
    pub(crate) wallet_account: String,
}

/// 直接录入公民返回 DTO。
#[derive(Serialize)]
pub(crate) struct AdminCreateCitizenOutput {
    pub(crate) id: u64,
    pub(crate) cid_number: String,
    pub(crate) passport_no: String,
    pub(crate) citizen_full_name: String,
    pub(crate) citizen_sex: String,
    pub(crate) citizen_birth_date: String,
    pub(crate) citizen_status: CitizenStatus,
    pub(crate) voting_eligible: bool,
    pub(crate) wallet_address: String,
    pub(crate) passport_valid_from: String,
    pub(crate) passport_valid_until: String,
    pub(crate) residence_province_code: String,
    pub(crate) residence_city_code: String,
    pub(crate) residence_town_code: String,
    pub(crate) birth_province_code: String,
    pub(crate) birth_city_code: String,
    pub(crate) birth_town_code: String,
    pub(crate) archive_hash: Option<String>,
}

/// 注册局管理员直接录入公民并直接发护照。
pub(crate) async fn admin_create_citizen(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminCreateCitizenInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if !crate::core::chain_runtime::is_tier1_registry(&ctx.institution_code)
        && !crate::core::chain_runtime::is_subordinate_registry(&ctx.institution_code)
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "只有注册局管理员可以新增公民");
    }

    let citizen_full_name = match required_trimmed(&input.citizen_full_name, "citizen_full_name") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let citizen_sex = match normalize_citizen_sex(input.citizen_sex.as_str()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let citizen_birth_date =
        match parse_required_date(input.citizen_birth_date.as_str(), "citizen_birth_date") {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    let today = Utc::now().date_naive();
    if citizen_birth_date > today {
        return api_error(StatusCode::BAD_REQUEST, 1001, "出生日期不能晚于今天");
    }
    if input.voting_eligible && !is_voting_age_at(today, citizen_birth_date) {
        return api_error(StatusCode::BAD_REQUEST, 1001, "未满16周岁不能设置选举资格");
    }

    let residence_province_name = match ctx.scope_province_name.as_deref().map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => return api_error(StatusCode::FORBIDDEN, 1003, "当前登录缺少办理省份"),
    };
    let residence_city_name = match ctx.scope_city_name.as_deref().map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => return api_error(StatusCode::FORBIDDEN, 1003, "当前登录缺少办理城市"),
    };
    let residence_province_code =
        match crate::cid::china::province_code_by_name(residence_province_name.as_str()) {
            Some(v) => v.to_string(),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "未知的办理省份"),
        };
    let residence_city_code = match crate::cid::china::city_code_by_name(
        residence_province_name.as_str(),
        residence_city_name.as_str(),
    ) {
        Some(v) => v.to_string(),
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "未知的办理城市"),
    };
    let residence_town_code =
        match required_trimmed(&input.residence_town_code, "residence_town_code") {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    if !crate::cid::china::town_exists(
        residence_province_code.as_str(),
        residence_city_code.as_str(),
        residence_town_code.as_str(),
    ) {
        return api_error(StatusCode::BAD_REQUEST, 1001, "未知的居住镇代码");
    }

    let birth_province_code =
        match required_trimmed(&input.birth_province_code, "birth_province_code") {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    let birth_city_code = match required_trimmed(&input.birth_city_code, "birth_city_code") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let birth_town_code = match required_trimmed(&input.birth_town_code, "birth_town_code") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some((birth_province_name, Some(_birth_city_name), Some(_birth_town_name))) =
        crate::cid::china::area_name_by_codes(
            birth_province_code.as_str(),
            Some(birth_city_code.as_str()),
            Some(birth_town_code.as_str()),
        )
    else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "未知的出生省市镇代码");
    };
    if birth_province_name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "未知的出生省份代码");
    }

    let wallet = match resolve_wallet_account(input.wallet_account.as_str()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match state.db.find_citizen_by_wallet(wallet.pubkey.as_str()) {
        Ok(Some(_)) => return api_error(StatusCode::CONFLICT, 1005, "该钱包账户已存在公民档案"),
        Ok(None) => {}
        Err(err) => {
            tracing::error!(error = %err, "query citizen wallet duplicate failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "钱包账户查重失败");
        }
    }

    let cid_number = match generate_cid_number(GenerateCidInput {
        account_pubkey: wallet.pubkey.as_str(),
        p1: "1",
        province_name: residence_province_name.as_str(),
        city_name: residence_city_name.as_str(),
        institution: "CTZN",
    }) {
        Ok(v) => v,
        Err(err) => {
            let detail = format!("公民身份CID生成失败: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
        }
    };
    if crate::cid::validate_cid_number_format(cid_number.as_str()).is_err() {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "公民身份CID格式生成失败",
        );
    }

    let now = Utc::now();
    let passport_no = match state.db.allocate_passport_no(
        residence_province_code.as_str(),
        residence_city_code.as_str(),
        cid_number.as_str(),
    ) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "allocate passport no failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "护照号生成失败");
        }
    };
    let valid_from = passport_valid_from(now);
    let valid_until = passport_valid_until(now, passport_validity_years(now, citizen_birth_date));
    let id = match state.db.next_citizen_id() {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "allocate citizen id failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "公民序号分配失败");
        }
    };

    let mut record = CitizenRecord {
        id,
        cid_number: cid_number.clone(),
        passport_no: passport_no.clone(),
        citizen_full_name: citizen_full_name.clone(),
        citizen_sex: citizen_sex.clone(),
        citizen_birth_date: citizen_birth_date.format("%Y-%m-%d").to_string(),
        wallet_pubkey: wallet.pubkey.clone(),
        wallet_address: wallet.address.clone(),
        wallet_sig_alg: "sr25519".to_string(),
        wallet_verified_at: Some(now),
        citizen_status: CitizenStatus::Normal,
        voting_eligible: input.voting_eligible,
        passport_valid_from: valid_from.clone(),
        passport_valid_until: valid_until.clone(),
        status_updated_at: Some(now.timestamp()),
        province_code: residence_province_code.clone(),
        city_code: residence_city_code.clone(),
        residence_province_code: residence_province_code.clone(),
        residence_city_code: residence_city_code.clone(),
        residence_town_code: residence_town_code.clone(),
        birth_province_code: birth_province_code.clone(),
        birth_city_code: birth_city_code.clone(),
        birth_town_code: birth_town_code.clone(),
        archive_hash: None,
        onchain_tx_hash: None,
        onchain_block_number: None,
        onchain_at: None,
        created_by: ctx.admin_account.clone(),
        created_at: now,
        updated_by: None,
        updated_at: now,
    };
    record.archive_hash = Some(citizen_archive_hash(&record));

    if let Err(err) = state.db.upsert_citizen_row(&record) {
        tracing::error!(error = %err, "citizen row upsert failed");
        if err.contains("duplicate key") || err.contains("already belongs") {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "公民身份、护照号或钱包账户已存在",
            );
        }
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "公民落库失败");
    }

    crate::core::runtime_ops::append_audit_log(
        &state,
        "CITIZEN_CREATE",
        &ctx.admin_account,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number,
            "passport_no": passport_no,
            "wallet_address": record.wallet_address,
            "residence_province_code": residence_province_code,
            "residence_city_code": residence_city_code,
            "residence_town_code": residence_town_code,
            "birth_province_code": birth_province_code,
            "birth_city_code": birth_city_code,
            "birth_town_code": birth_town_code,
            "voting_eligible": record.voting_eligible,
            "request_id": request_id_from_headers(&headers),
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );

    let output = AdminCreateCitizenOutput {
        id: record.id,
        cid_number: record.cid_number,
        passport_no: record.passport_no,
        citizen_full_name: record.citizen_full_name,
        citizen_sex: record.citizen_sex,
        citizen_birth_date: record.citizen_birth_date,
        citizen_status: record.citizen_status,
        voting_eligible: record.voting_eligible,
        wallet_address: record.wallet_address,
        passport_valid_from: record.passport_valid_from,
        passport_valid_until: record.passport_valid_until,
        residence_province_code: record.residence_province_code,
        residence_city_code: record.residence_city_code,
        residence_town_code: record.residence_town_code,
        birth_province_code: record.birth_province_code,
        birth_city_code: record.birth_city_code,
        birth_town_code: record.birth_town_code,
        archive_hash: record.archive_hash,
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

struct ResolvedWallet {
    address: String,
    pubkey: String,
}

fn required_trimmed(value: &str, field: &str) -> Result<String, axum::response::Response> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        let detail = format!("{field} 不能为空");
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, detail.as_str()));
    }
    Ok(trimmed.to_string())
}

fn parse_required_date(value: &str, field: &str) -> Result<NaiveDate, axum::response::Response> {
    let value = required_trimmed(value, field)?;
    NaiveDate::parse_from_str(value.as_str(), "%Y-%m-%d").map_err(|_| {
        let detail = format!("{field} 必须是 YYYY-MM-DD");
        api_error(StatusCode::BAD_REQUEST, 1001, detail.as_str())
    })
}

fn normalize_citizen_sex(value: &str) -> Result<String, axum::response::Response> {
    let value = required_trimmed(value, "citizen_sex")?;
    match value.as_str() {
        "MALE" | "FEMALE" => Ok(value),
        _ => Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "citizen_sex 仅支持 MALE / FEMALE",
        )),
    }
}

fn resolve_wallet_account(account: &str) -> Result<ResolvedWallet, axum::response::Response> {
    let account = account.trim();
    if account.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "wallet_account 不能为空",
        ));
    }
    if let Some(pubkey) = ss58_to_pubkey_hex(account) {
        let address = pubkey_hex_to_ss58(&pubkey).unwrap_or_else(|| account.to_string());
        return Ok(ResolvedWallet { address, pubkey });
    }
    let Some(pubkey) = normalize_pubkey_hex(account) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "wallet_account 不是合法 SS58 地址或 0x 公钥",
        ));
    };
    let Some(address) = pubkey_hex_to_ss58(&pubkey) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "wallet_account 无法推导地址",
        ));
    };
    Ok(ResolvedWallet { address, pubkey })
}

fn normalize_pubkey_hex(pubkey: &str) -> Option<String> {
    let bytes = parse_sr25519_pubkey_bytes(pubkey)?;
    Some(format!("0x{}", hex::encode(bytes)))
}

fn citizen_archive_hash(record: &CitizenRecord) -> String {
    let value = serde_json::json!({
        "cid_number": record.cid_number,
        "passport_no": record.passport_no,
        "citizen_full_name": record.citizen_full_name,
        "citizen_sex": record.citizen_sex,
        "citizen_birth_date": record.citizen_birth_date,
        "wallet_address": record.wallet_address,
        "residence_province_code": record.residence_province_code,
        "residence_city_code": record.residence_city_code,
        "residence_town_code": record.residence_town_code,
        "birth_province_code": record.birth_province_code,
        "birth_city_code": record.birth_city_code,
        "birth_town_code": record.birth_town_code,
        "passport_valid_from": record.passport_valid_from,
        "passport_valid_until": record.passport_valid_until,
        "voting_eligible": record.voting_eligible,
    });
    let mut hasher = Sha256::new();
    hasher.update(value.to_string().as_bytes());
    format!("0x{}", hex::encode(hasher.finalize()))
}

impl Db {
    /// 按钱包公钥查公民档案。钱包必填后,存在即代表该钱包已有档案。
    pub(crate) fn find_citizen_by_wallet(
        &self,
        wallet_pubkey: &str,
    ) -> Result<Option<CitizenRecord>, String> {
        let wallet_pubkey = wallet_pubkey.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT COALESCE(id, 0), cid_number, passport_no, citizen_full_name,
                            citizen_sex, citizen_birth_date, wallet_pubkey, wallet_address,
                            wallet_sig_alg, wallet_verified_at, citizen_status, voting_eligible,
                            passport_valid_from, passport_valid_until, status_updated_at,
                            province_code, city_code, residence_province_code, residence_city_code,
                            residence_town_code, birth_province_code, birth_city_code, birth_town_code,
                            archive_hash, onchain_tx_hash, onchain_block_number, onchain_at,
                            created_by, created_at, updated_by, updated_at
                     FROM citizens
                     WHERE lower(wallet_pubkey) = lower($1)
                     ORDER BY created_at DESC
                     LIMIT 1",
                    &[&wallet_pubkey],
                )
                .map_err(|e| format!("query citizen failed: {e}"))?;
            Ok(row.as_ref().map(citizen_record_from_row))
        })
    }

    pub(crate) fn allocate_passport_no(
        &self,
        province_code: &str,
        city_code: &str,
        cid_number: &str,
    ) -> Result<String, String> {
        let province_code = province_code.to_string();
        let city_code = city_code.to_string();
        let cid_number = cid_number.to_string();
        self.with_client(move |conn| {
            generate_passport_no_with_retry(conn, &province_code, &city_code, &cid_number)
        })
    }

    /// 分配下一个公民自增序号。
    pub(crate) fn next_citizen_id(&self) -> Result<u64, String> {
        self.with_client(|conn| {
            let row = conn
                .query_one("SELECT COALESCE(MAX(id), 0) + 1 FROM citizens", &[])
                .map_err(|e| format!("allocate citizen id failed: {e}"))?;
            let id: i64 = row.get(0);
            Ok(u64::try_from(id).unwrap_or(1))
        })
    }
}

fn citizen_status_from_db(status: &str) -> CitizenStatus {
    match status {
        "NORMAL" => CitizenStatus::Normal,
        _ => CitizenStatus::Revoked,
    }
}

pub(crate) fn citizen_record_from_row(row: &postgres::Row) -> CitizenRecord {
    let id: i64 = row.get(0);
    CitizenRecord {
        id: u64::try_from(id).unwrap_or(0),
        cid_number: row.get(1),
        passport_no: row.get(2),
        citizen_full_name: row.get(3),
        citizen_sex: row.get(4),
        citizen_birth_date: row.get(5),
        wallet_pubkey: row.get(6),
        wallet_address: row.get(7),
        wallet_sig_alg: row.get(8),
        wallet_verified_at: row.get(9),
        citizen_status: citizen_status_from_db(row.get::<_, String>(10).as_str()),
        voting_eligible: row.get(11),
        passport_valid_from: row.get(12),
        passport_valid_until: row.get(13),
        status_updated_at: row.get(14),
        province_code: row.get(15),
        city_code: row.get(16),
        residence_province_code: row.get(17),
        residence_city_code: row.get(18),
        residence_town_code: row.get(19),
        birth_province_code: row.get(20),
        birth_city_code: row.get(21),
        birth_town_code: row.get(22),
        archive_hash: row.get(23),
        onchain_tx_hash: row.get(24),
        onchain_block_number: row.get(25),
        onchain_at: row.get(26),
        created_by: row.get(27),
        created_at: row.get(28),
        updated_by: row.get(29),
        updated_at: row.get(30),
    }
}
