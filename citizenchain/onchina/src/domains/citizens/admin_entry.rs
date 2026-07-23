//! 注册局直接录入公民 handler。
//!
//! 公民由注册局管理员在办理市先录入本地档案。请求只提交公民档案字段;
//! 身份 CID、护照号、护照有效期由服务端确定性生成并落库。
//! 链账户留到链上身份推送阶段录入，并由该账户签名确认。

use axum::http::{HeaderMap, StatusCode};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cid::{generate_cid_number, GenerateCidInput};
use crate::crypto::pubkey::{account_id_to_ss58, normalize_account_id};
use crate::domains::citizens::passport_no::{
    generate_passport_no_with_retry, is_voting_age_at, passport_valid_from, passport_valid_until,
    passport_validity_years,
};
use crate::*;

/// 直接录入公民请求 DTO。
///
/// 居住省市由当前注册局办理上下文校验,前端只负责回传当前选择。
/// 本地建档不得要求链账户；儿童或暂未开户公民同样能先发放电子护照。
#[derive(Deserialize)]
pub(crate) struct AdminCreateCitizenInput {
    /// 当前注册局内的任职岗位码；与机构 CID、管理员签名钱包共同构成权限主体。
    pub(crate) actor_role_code: String,
    pub(crate) family_name: String,
    pub(crate) given_name: String,
    pub(crate) citizen_sex: String,
    pub(crate) citizen_birth_date: String,
    pub(crate) province_name: String,
    pub(crate) city_name: String,
    pub(crate) town_code: String,
    pub(crate) birth_province_code: String,
    pub(crate) birth_city_code: String,
    pub(crate) birth_town_code: String,
    pub(crate) voting_eligible: bool,
}

/// 直接录入公民返回 DTO。
#[derive(Serialize)]
pub(crate) struct AdminCreateCitizenOutput {
    pub(crate) id: u64,
    pub(crate) cid_number: String,
    pub(crate) passport_no: String,
    pub(crate) family_name: String,
    pub(crate) given_name: String,
    pub(crate) citizen_sex: String,
    pub(crate) citizen_birth_date: String,
    pub(crate) citizen_status: CitizenStatus,
    pub(crate) voting_eligible: bool,
    pub(crate) account_id: Option<String>,
    pub(crate) ss58_address: Option<String>,
    pub(crate) passport_valid_from: String,
    pub(crate) passport_valid_until: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) town_code: String,
    pub(crate) birth_province_code: String,
    pub(crate) birth_city_code: String,
    pub(crate) birth_town_code: String,
    pub(crate) archive_hash: Option<String>,
}

/// 建档输入校验产物:两阶段占号流程经会话 JSON 往返(ADR-031 D6)。
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ValidatedCitizenInput {
    pub(crate) family_name: String,
    pub(crate) given_name: String,
    pub(crate) citizen_sex: String,
    /// YYYY-MM-DD(已校验)。
    pub(crate) citizen_birth_date: String,
    pub(crate) province_name: String,
    pub(crate) city_name: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) town_code: String,
    pub(crate) birth_province_code: String,
    pub(crate) birth_city_code: String,
    pub(crate) birth_town_code: String,
    pub(crate) voting_eligible: bool,
}

/// 校验建档输入(占号 prepare 阶段调用;不生成号、不落库,ADR-031 占号先行)。
pub(crate) fn validate_citizen_input(
    ctx: &crate::auth::login::AdminAuthContext,
    input: &AdminCreateCitizenInput,
) -> Result<ValidatedCitizenInput, axum::response::Response> {
    let family_name = match required_trimmed(&input.family_name, "family_name") {
        Ok(v) => v,
        Err(resp) => return Err(resp),
    };
    let given_name = match required_trimmed(&input.given_name, "given_name") {
        Ok(v) => v,
        Err(resp) => return Err(resp),
    };
    let citizen_sex = match normalize_citizen_sex(input.citizen_sex.as_str()) {
        Ok(v) => v,
        Err(resp) => return Err(resp),
    };
    let citizen_birth_date =
        match parse_required_date(input.citizen_birth_date.as_str(), "citizen_birth_date") {
            Ok(v) => v,
            Err(resp) => return Err(resp),
        };
    let today = Utc::now().date_naive();
    if citizen_birth_date > today {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "出生日期不能晚于今天",
        ));
    }
    if input.voting_eligible && !is_voting_age_at(today, citizen_birth_date) {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "未满16周岁不能设置选举资格",
        ));
    }

    let (province_name, city_name) =
        match resolve_citizen_scope(&ctx, &input.province_name, &input.city_name) {
            Ok(v) => v,
            Err(resp) => return Err(resp),
        };
    let province_code = match crate::cid::china::province_code_by_name(province_name.as_str()) {
        Some(v) => v.to_string(),
        None => return Err(api_error(StatusCode::BAD_REQUEST, 1001, "未知的办理省份")),
    };
    let city_code =
        match crate::cid::china::city_code_by_name(province_name.as_str(), city_name.as_str()) {
            Some(v) => v.to_string(),
            None => return Err(api_error(StatusCode::BAD_REQUEST, 1001, "未知的办理城市")),
        };
    let town_code = match required_trimmed(&input.town_code, "town_code") {
        Ok(v) => v,
        Err(resp) => return Err(resp),
    };
    if !crate::cid::china::town_exists(
        province_code.as_str(),
        city_code.as_str(),
        town_code.as_str(),
    ) {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "未知的镇代码"));
    }

    let birth_province_code =
        match required_trimmed(&input.birth_province_code, "birth_province_code") {
            Ok(v) => v,
            Err(resp) => return Err(resp),
        };
    let birth_city_code = match required_trimmed(&input.birth_city_code, "birth_city_code") {
        Ok(v) => v,
        Err(resp) => return Err(resp),
    };
    let birth_town_code = match required_trimmed(&input.birth_town_code, "birth_town_code") {
        Ok(v) => v,
        Err(resp) => return Err(resp),
    };
    let Some((birth_province_name, Some(_birth_city_name), Some(_birth_town_name))) =
        crate::cid::china::area_name_by_codes(
            birth_province_code.as_str(),
            Some(birth_city_code.as_str()),
            Some(birth_town_code.as_str()),
        )
    else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "未知的出生省市镇代码",
        ));
    };
    if birth_province_name.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "未知的出生省份代码",
        ));
    }

    Ok(ValidatedCitizenInput {
        family_name,
        given_name,
        citizen_sex,
        citizen_birth_date: citizen_birth_date.format("%Y-%m-%d").to_string(),
        province_name,
        city_name,
        province_code,
        city_code,
        town_code,
        birth_province_code,
        birth_city_code,
        birth_town_code,
        voting_eligible: input.voting_eligible,
    })
}

/// 建档种子:档案稳定字段确定性派生;发号种子与链上承诺哈希同源。
pub(crate) fn citizen_cid_seed(v: &ValidatedCitizenInput) -> String {
    let birth = NaiveDate::parse_from_str(v.citizen_birth_date.as_str(), "%Y-%m-%d")
        .expect("validated birth date");
    local_citizen_cid_seed(
        &v.family_name,
        &v.given_name,
        &v.citizen_sex,
        birth,
        &v.province_code,
        &v.city_code,
        &v.town_code,
        &v.birth_province_code,
        &v.birth_city_code,
        &v.birth_town_code,
    )
}

/// 按种子 + nonce 后缀生成候选号(碰撞重试用;nonce=0 与历史种子字节一致)。
pub(crate) fn generate_citizen_cid_candidate(
    v: &ValidatedCitizenInput,
    seed: &str,
    nonce: u32,
) -> Result<String, axum::response::Response> {
    let seeded = if nonce == 0 {
        seed.to_string()
    } else {
        format!("{seed}|n{nonce}")
    };
    let cid_number = generate_cid_number(GenerateCidInput {
        account_id: seeded.as_str(),
        p1: "1",
        province_name: v.province_name.as_str(),
        city_name: v.city_name.as_str(),
        institution: "CTZN",
    })
    .map_err(|err| {
        let detail = format!("公民身份CID生成失败: {err}");
        api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str())
    })?;
    if crate::cid::validate_cid_number_format(cid_number.as_str()).is_err() {
        return Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "公民身份CID格式生成失败",
        ));
    }
    Ok(cid_number)
}

/// 占号进块后落库(submit 阶段调用):护照签发 + 档案入库 + 审计。
pub(crate) fn persist_citizen_record(
    state: &AppState,
    headers: &HeaderMap,
    account_id: &str,
    v: &ValidatedCitizenInput,
    cid_number: &str,
    onchain_tx_hash: &str,
    onchain_block_number: Option<u64>,
) -> Result<CitizenRecord, axum::response::Response> {
    let citizen_birth_date = NaiveDate::parse_from_str(v.citizen_birth_date.as_str(), "%Y-%m-%d")
        .expect("validated birth date");
    let family_name = v.family_name.clone();
    let given_name = v.given_name.clone();
    let citizen_sex = v.citizen_sex.clone();
    let province_code = v.province_code.clone();
    let city_code = v.city_code.clone();
    let town_code = v.town_code.clone();
    let birth_province_code = v.birth_province_code.clone();
    let birth_city_code = v.birth_city_code.clone();
    let birth_town_code = v.birth_town_code.clone();
    let cid_number = cid_number.to_string();

    let now = Utc::now();
    let passport_no = match state.db.allocate_passport_no(
        province_code.as_str(),
        city_code.as_str(),
        cid_number.as_str(),
    ) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "allocate passport no failed");
            return Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "护照号生成失败",
            ));
        }
    };
    let valid_from = passport_valid_from(now);
    let valid_until = passport_valid_until(now, passport_validity_years(now, citizen_birth_date));
    let id = match state.db.next_citizen_id() {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "allocate citizen id failed");
            return Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "公民序号分配失败",
            ));
        }
    };

    let mut record = CitizenRecord {
        id,
        cid_number: cid_number.clone(),
        passport_no: passport_no.clone(),
        family_name: family_name.clone(),
        given_name: given_name.clone(),
        citizen_sex: citizen_sex.clone(),
        citizen_birth_date: citizen_birth_date.format("%Y-%m-%d").to_string(),
        account_id: None,
        account_verified_at: None,
        citizen_status: CitizenStatus::Normal,
        voting_eligible: v.voting_eligible,
        passport_valid_from: valid_from.clone(),
        passport_valid_until: valid_until.clone(),
        status_updated_at: Some(now.timestamp()),
        province_code: province_code.clone(),
        city_code: city_code.clone(),
        town_code: town_code.clone(),
        birth_province_code: birth_province_code.clone(),
        birth_city_code: birth_city_code.clone(),
        birth_town_code: birth_town_code.clone(),
        archive_hash: None,
        onchain_tx_hash: Some(onchain_tx_hash.to_string()),
        onchain_block_number: onchain_block_number.map(|n| n as i64),
        onchain_at: Some(now),
        creator_account_id: account_id.to_string(),
        created_at: now,
        updater_account_id: None,
        updated_at: now,
    };
    record.archive_hash = Some(citizen_archive_hash(&record));

    if let Err(err) = state.db.upsert_citizen_row(&record) {
        tracing::error!(error = %err, "citizen row upsert failed");
        if err.contains("duplicate key") || err.contains("already belongs") {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "公民身份或护照号已存在",
            ));
        }
        return Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "公民落库失败",
        ));
    }

    crate::core::runtime_ops::append_audit_log(
        state,
        "CITIZEN_CREATE",
        account_id,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number,
            "passport_no": passport_no,
            "family_name": record.family_name,
            "given_name": record.given_name,
            "province_code": province_code,
            "city_code": city_code,
            "town_code": town_code,
            "birth_province_code": birth_province_code,
            "birth_city_code": birth_city_code,
            "birth_town_code": birth_town_code,
            "voting_eligible": record.voting_eligible,
            "onchain_tx_hash": onchain_tx_hash,
            "request_id": request_id_from_headers(headers),
            "actor_ip": actor_ip_from_headers(headers),
        }),
    );
    Ok(record)
}

/// 建档返回 DTO(submit 阶段复用)。
pub(crate) fn create_output_from_record(record: CitizenRecord) -> AdminCreateCitizenOutput {
    AdminCreateCitizenOutput {
        id: record.id,
        cid_number: record.cid_number,
        passport_no: record.passport_no,
        family_name: record.family_name,
        given_name: record.given_name,
        citizen_sex: record.citizen_sex,
        citizen_birth_date: record.citizen_birth_date,
        citizen_status: record.citizen_status,
        voting_eligible: record.voting_eligible,
        ss58_address: record.account_id.as_deref().and_then(account_id_to_ss58),
        account_id: record.account_id,
        passport_valid_from: record.passport_valid_from,
        passport_valid_until: record.passport_valid_until,
        province_code: record.province_code,
        city_code: record.city_code,
        town_code: record.town_code,
        birth_province_code: record.birth_province_code,
        birth_city_code: record.birth_city_code,
        birth_town_code: record.birth_town_code,
        archive_hash: record.archive_hash,
    }
}

pub(crate) struct ResolvedCitizenAccount {
    pub(crate) account_id: String,
    pub(crate) ss58_address: String,
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

fn resolve_citizen_scope(
    ctx: &crate::auth::login::AdminAuthContext,
    requested_province_name: &str,
    requested_city_name: &str,
) -> Result<(String, String), axum::response::Response> {
    let province_name = required_trimmed(requested_province_name, "province_name")?;
    let city_name = required_trimmed(requested_city_name, "city_name")?;
    let scope = crate::scope::get_visible_scope(ctx);
    if !scope.can_write {
        return Err(api_error(StatusCode::FORBIDDEN, 1003, "当前登录无办理权限"));
    }
    if !scope.includes_province(province_name.as_str()) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "province_name out of current admin scope",
        ));
    }
    if !scope.includes_city(city_name.as_str()) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "city_name out of current admin scope",
        ));
    }
    let Some(province) = crate::cid::china::provinces()
        .iter()
        .find(|p| p.province_name == province_name)
    else {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "未知的办理省份"));
    };
    if !province.cities.iter().any(|c| c.city_name == city_name) {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "未知的办理城市"));
    }
    Ok((province_name, city_name))
}

#[allow(clippy::too_many_arguments)]
fn local_citizen_cid_seed(
    family_name: &str,
    given_name: &str,
    citizen_sex: &str,
    citizen_birth_date: NaiveDate,
    province_code: &str,
    city_code: &str,
    town_code: &str,
    birth_province_code: &str,
    birth_city_code: &str,
    birth_town_code: &str,
) -> String {
    // 本地建档阶段没有账户 ID,因此 CID 种子只能来自档案自身的稳定字段。
    // 钱包绑定属于后续链上身份推送,不得回头改变本地身份号。
    let mut hasher = Sha256::new();
    let birth_date_text = citizen_birth_date.format("%Y-%m-%d").to_string();
    for part in [
        family_name,
        given_name,
        citizen_sex,
        birth_date_text.as_str(),
        province_code,
        city_code,
        town_code,
        birth_province_code,
        birth_city_code,
        birth_town_code,
    ] {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    format!("citizen-local-0x{}", hex::encode(hasher.finalize()))
}

pub(crate) fn resolve_citizen_account(
    account_id: &str,
) -> Result<ResolvedCitizenAccount, axum::response::Response> {
    let account_id = account_id.trim();
    if account_id.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "account_id 不能为空",
        ));
    }
    let Some(account_id) = normalize_account_id(account_id) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "account_id 必须是小写 0x 加 64 位十六进制",
        ));
    };
    let Some(ss58_address) = account_id_to_ss58(&account_id) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "account_id 无法派生 SS58 展示地址",
        ));
    };
    Ok(ResolvedCitizenAccount {
        account_id,
        ss58_address,
    })
}

fn citizen_archive_hash(record: &CitizenRecord) -> String {
    let value = serde_json::json!({
        "cid_number": record.cid_number,
        "passport_no": record.passport_no,
        "family_name": record.family_name,
        "given_name": record.given_name,
        "citizen_sex": record.citizen_sex,
        "citizen_birth_date": record.citizen_birth_date,
        "province_code": record.province_code,
        "city_code": record.city_code,
        "town_code": record.town_code,
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
    /// 按账户 ID 查公民档案。仅已完成账户绑定的公民会命中本查询。
    pub(crate) fn find_citizen_by_account_id(
        &self,
        account_id: &str,
    ) -> Result<Option<CitizenRecord>, String> {
        let account_id = account_id.trim().to_string();
        if account_id.is_empty() {
            return Ok(None);
        }
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT COALESCE(id, 0), cid_number, passport_no, family_name,
                            given_name, citizen_sex, citizen_birth_date, account_id,
                            account_verified_at, citizen_status, voting_eligible,
                            passport_valid_from, passport_valid_until, status_updated_at,
                            province_code, city_code, town_code,
                            birth_province_code, birth_city_code, birth_town_code,
                            archive_hash, onchain_tx_hash, onchain_block_number, onchain_at,
                            creator_account_id, created_at, updater_account_id, updated_at
                     FROM citizens
                     WHERE account_id = $1
                     ORDER BY created_at DESC
                     LIMIT 1",
                    &[&account_id],
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
        family_name: row.get(3),
        given_name: row.get(4),
        citizen_sex: row.get(5),
        citizen_birth_date: row.get(6),
        account_id: row.get(7),
        account_verified_at: row.get(8),
        citizen_status: citizen_status_from_db(row.get::<_, String>(9).as_str()),
        voting_eligible: row.get(10),
        passport_valid_from: row.get(11),
        passport_valid_until: row.get(12),
        status_updated_at: row.get(13),
        province_code: row.get(14),
        city_code: row.get(15),
        town_code: row.get(16),
        birth_province_code: row.get(17),
        birth_city_code: row.get(18),
        birth_town_code: row.get(19),
        archive_hash: row.get(20),
        onchain_tx_hash: row.get(21),
        onchain_block_number: row.get(22),
        onchain_at: row.get(23),
        creator_account_id: row.get(24),
        created_at: row.get(25),
        updater_account_id: row.get(26),
        updated_at: row.get(27),
    }
}
