//! 注册局直接录入公民 handler。
//!
//! 公民由注册局管理员在本省/本市范围内直接录入并直接发护照。
//! 一条 citizen 记录处于 NORMAL 且在有效期内即视为护照已签发。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::crypto::pubkey::{pubkey_hex_to_ss58, ss58_to_pubkey_hex};
use crate::*;

/// 直接录入公民请求 DTO。
///
/// 中文注释:档案号概念已删,护照身份即 `cid_number`;wallet 可选,
/// 有钱包即同时完成钱包绑定,没有则只发护照、绑定态留 PENDING。
#[derive(Deserialize)]
pub(crate) struct AdminCreateCitizenInput {
    /// 公民身份码(护照身份),由调用方决定;直接作为唯一身份号落库。
    pub(crate) cid_number: String,
    /// 居住地行政区(省必填,市/镇可选)。
    pub(crate) residence_province_code: String,
    #[serde(default)]
    pub(crate) residence_city_code: Option<String>,
    #[serde(default)]
    pub(crate) residence_town_code: Option<String>,
    /// 出生地行政区(省必填,市/镇可选)。
    pub(crate) birth_province_code: String,
    #[serde(default)]
    pub(crate) birth_city_code: Option<String>,
    #[serde(default)]
    pub(crate) birth_town_code: Option<String>,
    /// 选举资格。
    pub(crate) voting_eligible: bool,
    /// 选举范围层级(PROVINCE / CITY / TOWN)。
    pub(crate) election_scope_level: String,
    /// 护照有效期起,格式固定 YYYY-MM-DD。
    pub(crate) valid_from: String,
    /// 护照有效期止,格式固定 YYYY-MM-DD。
    pub(crate) valid_until: String,
    /// 可选钱包公钥(0x hex);与 wallet_address 二选一或同时给出。
    #[serde(default)]
    pub(crate) wallet_pubkey: Option<String>,
    /// 可选钱包地址(SS58,prefix=2027)。
    #[serde(default)]
    pub(crate) wallet_address: Option<String>,
}

/// 直接录入公民返回 DTO。
#[derive(Serialize)]
pub(crate) struct AdminCreateCitizenOutput {
    pub(crate) id: u64,
    pub(crate) cid_number: String,
    pub(crate) citizen_status: CitizenStatus,
    pub(crate) voting_eligible: bool,
    pub(crate) bind_status: CitizenBindStatus,
    pub(crate) wallet_pubkey: Option<String>,
    pub(crate) wallet_address: Option<String>,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) election_scope_level: String,
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

    let cid_number = input.cid_number.trim().to_string();
    if cid_number.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "cid_number 不能为空");
    }
    if crate::number::validate_cid_number_format(cid_number.as_str()).is_err() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "cid_number 格式不合法");
    }

    let residence_province_code = input.residence_province_code.trim().to_string();
    let birth_province_code = input.birth_province_code.trim().to_string();
    if residence_province_code.is_empty() || birth_province_code.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "居住地和出生地省级行政区均为必填",
        );
    }

    let election_scope_level = input.election_scope_level.trim().to_string();
    if !matches!(election_scope_level.as_str(), "PROVINCE" | "CITY" | "TOWN") {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "election_scope_level 仅支持 PROVINCE / CITY / TOWN",
        );
    }

    let valid_from = input.valid_from.trim().to_string();
    let valid_until = input.valid_until.trim().to_string();
    if !is_valid_archive_date(&valid_from) || !is_valid_archive_date(&valid_until) {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "护照有效期格式必须为 YYYY-MM-DD",
        );
    }

    // scope 校验:管理员只能在本省/本市范围内录入(居住地为准)。
    let residence_province_name =
        match crate::china::province_name_by_code(residence_province_code.as_str()) {
            Some(v) => v,
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "未知的居住地省级代码"),
        };
    let scope = crate::scope::get_visible_scope(&ctx);
    if !scope.includes_province(residence_province_name) {
        return api_error(StatusCode::FORBIDDEN, 1003, "居住地省份超出当前管理员范围");
    }
    let residence_city_code = input
        .residence_city_code
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    if let Some(city_code) = residence_city_code.as_deref() {
        match crate::china::area_name_by_codes(
            residence_province_code.as_str(),
            Some(city_code),
            None,
        ) {
            Some((_, Some(city_name), _)) => {
                if !scope.includes_city(city_name) {
                    return api_error(StatusCode::FORBIDDEN, 1003, "居住地城市超出当前管理员范围");
                }
            }
            _ => return api_error(StatusCode::BAD_REQUEST, 1001, "未知的居住地城市代码"),
        }
    }

    // 钱包可选:给了就规范化校验,决定绑定态。
    let wallet = match resolve_optional_wallet(
        input.wallet_address.as_deref(),
        input.wallet_pubkey.as_deref(),
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let now = Utc::now();
    let record = CitizenRecord {
        id: 0, // 由 next_citizen_id 分配,见下。
        wallet_pubkey: wallet.as_ref().map(|(_, pubkey)| pubkey.clone()),
        wallet_address: wallet.as_ref().map(|(address, _)| address.clone()),
        cid_number: Some(cid_number.clone()),
        citizen_status: Some(CitizenStatus::Normal),
        voting_eligible: input.voting_eligible,
        archive_valid_from: Some(valid_from.clone()),
        archive_valid_until: Some(valid_until.clone()),
        status_updated_at: Some(now.timestamp()),
        province_code: Some(residence_province_code.clone()),
        city_code: residence_city_code.clone(),
        residence_province_code: Some(residence_province_code.clone()),
        residence_city_code: residence_city_code.clone(),
        residence_town_code: input
            .residence_town_code
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        birth_province_code: Some(birth_province_code.clone()),
        birth_city_code: input
            .birth_city_code
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        birth_town_code: input
            .birth_town_code
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        election_scope_level: Some(election_scope_level.clone()),
        bound_at: Some(now),
        bound_by: Some(ctx.admin_account.clone()),
        created_at: now,
    };

    let id = match state.db.next_citizen_id() {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "allocate citizen id failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "公民序号分配失败");
        }
    };
    let record = CitizenRecord { id, ..record };

    if let Err(err) = state.db.upsert_citizen_row(&record) {
        tracing::error!(error = %err, "citizen row upsert failed");
        if err.contains("duplicate key") {
            return api_error(StatusCode::CONFLICT, 1005, "公民身份号已存在");
        }
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "公民落库失败");
    }

    crate::core::runtime_ops::append_audit_log(
        &state,
        "CITIZEN_CREATE",
        &ctx.admin_account,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number.clone(),
            "voting_eligible": record.voting_eligible,
            "bind_status": citizen_bind_status_value(&record.bind_status()),
            "request_id": request_id_from_headers(&headers),
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );

    let output = AdminCreateCitizenOutput {
        id: record.id,
        cid_number,
        citizen_status: CitizenStatus::Normal,
        voting_eligible: record.voting_eligible,
        bind_status: record.bind_status(),
        wallet_pubkey: record.wallet_pubkey.clone(),
        wallet_address: record.wallet_address.clone(),
        valid_from,
        valid_until,
        election_scope_level,
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

fn citizen_bind_status_value(status: &CitizenBindStatus) -> &'static str {
    match status {
        CitizenBindStatus::Pending => "PENDING",
        CitizenBindStatus::Bound => "BOUND",
    }
}

fn is_valid_archive_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

/// 中文注释:钱包可选——两个字段都空则返回 None(护照仍照发,绑定态 PENDING);
/// 任一字段非空则要求 SS58 地址与公钥可互相规范化推导且一致,否则拒绝。
#[allow(clippy::type_complexity)]
fn resolve_optional_wallet(
    wallet_address: Option<&str>,
    wallet_pubkey: Option<&str>,
) -> Result<Option<(String, String)>, axum::response::Response> {
    let address = wallet_address.map(str::trim).filter(|v| !v.is_empty());
    let pubkey = wallet_pubkey.map(str::trim).filter(|v| !v.is_empty());
    match (address, pubkey) {
        (None, None) => Ok(None),
        (Some(address), maybe_pubkey) => {
            let Some(derived_pubkey) = ss58_to_pubkey_hex(address) else {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "wallet_address 不是合法 SS58 地址",
                ));
            };
            if let Some(pubkey) = maybe_pubkey {
                if !same_pubkey_hex(pubkey, &derived_pubkey) {
                    return Err(api_error(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        2004,
                        "wallet_pubkey 与 wallet_address 不一致",
                    ));
                }
            }
            let canonical_address = pubkey_hex_to_ss58(&derived_pubkey).unwrap_or_default();
            Ok(Some((canonical_address, derived_pubkey)))
        }
        (None, Some(pubkey)) => {
            let normalized = normalize_pubkey_hex(pubkey).ok_or_else(|| {
                api_error(StatusCode::BAD_REQUEST, 1001, "wallet_pubkey 格式不合法")
            })?;
            let Some(canonical_address) = pubkey_hex_to_ss58(&normalized) else {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "wallet_pubkey 无法推导地址",
                ));
            };
            Ok(Some((canonical_address, normalized)))
        }
    }
}

fn normalize_pubkey_hex(pubkey: &str) -> Option<String> {
    let bytes = parse_sr25519_pubkey_bytes(pubkey)?;
    Some(format!("0x{}", hex::encode(bytes)))
}

fn same_pubkey_hex(left: &str, right: &str) -> bool {
    left.trim_start_matches("0x")
        .eq_ignore_ascii_case(right.trim_start_matches("0x"))
}

impl Db {
    /// 按钱包公钥查已绑定公民(vote / chain_vote 的状态查询入口)。
    pub(crate) fn find_bound_citizen_by_wallet(
        &self,
        wallet_pubkey: &str,
    ) -> Result<Option<CitizenRecord>, String> {
        let wallet_pubkey = wallet_pubkey.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT COALESCE(id, 0), wallet_pubkey, wallet_address,
                            cid_number, citizen_status, voting_eligible, valid_from,
                            valid_until, status_updated_at, province_code, city_code,
                            residence_province_code, residence_city_code, residence_town_code,
                            birth_province_code, birth_city_code, birth_town_code, election_scope_level,
                            bound_at, bound_by, created_at
                     FROM citizens
                     WHERE lower(wallet_pubkey) = lower($1) AND bind_status = 'BOUND'
                     ORDER BY created_at DESC
                     LIMIT 1",
                    &[&wallet_pubkey],
                )
                .map_err(|e| format!("query citizen failed: {e}"))?;
            Ok(row.as_ref().map(citizen_record_from_row))
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

fn citizen_record_from_row(row: &postgres::Row) -> CitizenRecord {
    let id: i64 = row.get(0);
    CitizenRecord {
        id: u64::try_from(id).unwrap_or(0),
        wallet_pubkey: row.get(1),
        wallet_address: row.get(2),
        cid_number: Some(row.get(3)),
        citizen_status: Some(citizen_status_from_db(row.get::<_, String>(4).as_str())),
        voting_eligible: row.get(5),
        archive_valid_from: row.get(6),
        archive_valid_until: row.get(7),
        status_updated_at: row.get(8),
        province_code: Some(row.get(9)),
        city_code: row.get(10),
        residence_province_code: row.get(11),
        residence_city_code: row.get(12),
        residence_town_code: row.get(13),
        birth_province_code: row.get(14),
        birth_city_code: row.get(15),
        birth_town_code: row.get(16),
        election_scope_level: row.get(17),
        bound_at: row.get(18),
        bound_by: row.get(19),
        created_at: row.get(20),
    }
}
