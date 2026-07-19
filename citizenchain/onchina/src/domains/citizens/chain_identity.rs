//! 公民身份上链准备 handler。
//!
//! 本模块只处理公民钱包签名与 `citizen-identity` call data 构造。
//! 本地建档不要求钱包;只有注册局准备推送链上投票身份时才录入钱包并验签。

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use codec::{Compact, Encode};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair};
use uuid::Uuid;

use crate::domains::citizens::admin_entry::{
    citizen_record_from_row, resolve_wallet_account, ResolvedWallet,
};
use crate::*;

const CITIZEN_IDENTITY_PALLET_INDEX: u8 = 10;
const REGISTER_VOTING_IDENTITY_CALL_INDEX: u8 = 0;
const UPGRADE_TO_CANDIDATE_IDENTITY_CALL_INDEX: u8 = 1;
const MIN_ONCHAIN_CITIZEN_AGE_YEARS: u8 = 16;

#[derive(Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CitizenOnchainIdentityLevel {
    Voting,
    Candidate,
}

impl CitizenOnchainIdentityLevel {
    fn as_str(self) -> &'static str {
        match self {
            CitizenOnchainIdentityLevel::Voting => "voting",
            CitizenOnchainIdentityLevel::Candidate => "candidate",
        }
    }

    fn call_index(self) -> u8 {
        match self {
            CitizenOnchainIdentityLevel::Voting => REGISTER_VOTING_IDENTITY_CALL_INDEX,
            CitizenOnchainIdentityLevel::Candidate => UPGRADE_TO_CANDIDATE_IDENTITY_CALL_INDEX,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct PrepareCitizenOnchainInput {
    pub(crate) wallet_account: String,
    pub(crate) identity_level: CitizenOnchainIdentityLevel,
}

#[derive(Serialize)]
pub(crate) struct PrepareCitizenOnchainOutput {
    pub(crate) cid_number: String,
    pub(crate) identity_level: CitizenOnchainIdentityLevel,
    pub(crate) wallet_address: String,
    pub(crate) wallet_pubkey: String,
    pub(crate) citizen_age_years: u8,
    pub(crate) payload_hex: String,
    pub(crate) sign_request: String,
    pub(crate) action_label_zh: String,
    pub(crate) expires_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct CompleteCitizenOnchainInput {
    pub(crate) wallet_account: String,
    pub(crate) identity_level: CitizenOnchainIdentityLevel,
    pub(crate) sign_response: String,
}

#[derive(Serialize)]
pub(crate) struct CompleteCitizenOnchainOutput {
    pub(crate) request_id: String,
    pub(crate) cid_number: String,
    pub(crate) identity_level: CitizenOnchainIdentityLevel,
    pub(crate) wallet_address: String,
    pub(crate) chain_action: u16,
    pub(crate) call_data_hex: String,
    pub(crate) citizen_signature: String,
    pub(crate) citizen_identity_chain_sign_request: String,
}

pub(crate) async fn prepare_citizen_onchain_signature(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(cid_number): Path<String>,
    Json(input): Json<PrepareCitizenOnchainInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_registry_admin(&ctx) {
        return resp;
    }
    // 整个业务操作只消费这一次 Passkey；后续公民回签通过 operation_id 绑定。
    if let Err(resp) = crate::auth::passkey::require_passkey_assertion(
        &state,
        &headers,
        ctx.admin_account.as_str(),
    ) {
        return resp;
    }
    let wallet = match resolve_wallet_account(input.wallet_account.as_str()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let record = match state.db.find_citizen_by_cid(cid_number.as_str()) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "公民档案不存在"),
        Err(err) => {
            tracing::error!(error = %err, "query citizen by cid failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "公民档案查询失败");
        }
    };
    if let Err(resp) = ensure_record_in_admin_scope(&ctx, &record) {
        return resp;
    }
    if let Err(resp) = ensure_wallet_available(&state, &record, &wallet) {
        return resp;
    }
    let payload = match build_citizen_identity_payload(&record, &wallet, input.identity_level) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(180);
    let request_id = format!("citizen-identity-{}", Uuid::new_v4());
    let sign_request = match crate::core::qr::build_sign_request_bytes(
        request_id.as_str(),
        issued_at.timestamp(),
        expires_at.timestamp(),
        wallet.pubkey.as_str(),
        &payload.payload_bytes,
        crate::core::qr::action_citizen_identity(),
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let operation = CitizenOnchainOperation {
        operation_id: request_id,
        admin_account: ctx.admin_account,
        institution_code: ctx.institution_code,
        cid_number: record.cid_number.clone(),
        wallet_pubkey: wallet.pubkey.clone(),
        wallet_address: wallet.address.clone(),
        identity_level: payload.identity_level.as_str().to_string(),
        payload_hex: hex::encode(&payload.payload_bytes),
        expires_at,
    };
    if let Err(err) = state.db.insert_citizen_onchain_operation(&operation) {
        tracing::error!(error = %err, "insert citizen onchain operation failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "公民签名操作落库失败",
        );
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PrepareCitizenOnchainOutput {
            cid_number: record.cid_number,
            identity_level: payload.identity_level,
            wallet_address: wallet.address,
            wallet_pubkey: wallet.pubkey,
            citizen_age_years: payload.citizen_age_years,
            payload_hex: format!("0x{}", hex::encode(payload.payload_bytes)),
            sign_request,
            action_label_zh: crate::core::qr::action_label_zh("citizen_identity"),
            expires_at: expires_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn complete_citizen_onchain_signature(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(cid_number): Path<String>,
    Json(input): Json<CompleteCitizenOnchainInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_registry_admin(&ctx) {
        return resp;
    }
    let wallet = match resolve_wallet_account(input.wallet_account.as_str()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let record = match state.db.find_citizen_by_cid(cid_number.as_str()) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "公民档案不存在"),
        Err(err) => {
            tracing::error!(error = %err, "query citizen by cid failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "公民档案查询失败");
        }
    };
    if let Err(resp) = ensure_record_in_admin_scope(&ctx, &record) {
        return resp;
    }
    if let Err(resp) = ensure_wallet_available(&state, &record, &wallet) {
        return resp;
    }
    let payload = match build_citizen_identity_payload(&record, &wallet, input.identity_level) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sign_response = match crate::core::qr::parse_sign_response(input.sign_response.as_str()) {
        Ok(v) => v,
        Err(err) => {
            let detail = format!("公民钱包签名响应无效: {err}");
            return api_error(StatusCode::BAD_REQUEST, 1001, detail.as_str());
        }
    };
    let operation_id = match sign_response
        .id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(value) => value,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "公民签名响应缺少操作编号"),
    };
    let operation = match state.db.find_citizen_onchain_operation(operation_id) {
        Ok(Some(value)) => value,
        Ok(None) => return api_error(StatusCode::GONE, 2003, "公民签名操作不存在或已失效"),
        Err(err) => {
            tracing::error!(error = %err, "query citizen onchain operation failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "公民签名操作查询失败",
            );
        }
    };
    if operation.admin_account != ctx.admin_account
        || operation.institution_code != ctx.institution_code
        || operation.cid_number != cid_number
        || operation.wallet_pubkey != wallet.pubkey
        || operation.wallet_address != wallet.address
        || operation.identity_level != input.identity_level.as_str()
        || operation.payload_hex != hex::encode(&payload.payload_bytes)
    {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "公民签名响应与当前业务操作不一致",
        );
    }
    let signer_pubkey = sign_response.body.pubkey;
    if !same_pubkey_hex(signer_pubkey.as_str(), wallet.pubkey.as_str()) {
        return api_error(StatusCode::FORBIDDEN, 1003, "签名钱包与录入钱包不一致");
    }
    let citizen_signature = sign_response.body.signature;
    if !verify_citizen_identity_signature(
        wallet.pubkey.as_str(),
        &payload.payload_bytes,
        citizen_signature.as_str(),
    ) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "公民钱包签名校验失败",
        );
    }
    match state.db.consume_citizen_onchain_operation(operation_id) {
        Ok(true) => {}
        Ok(false) => return api_error(StatusCode::CONFLICT, 2003, "公民签名操作已消费或已过期"),
        Err(err) => {
            tracing::error!(error = %err, "consume citizen onchain operation failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "公民签名操作消费失败",
            );
        }
    }

    let actor_cid_number = match active_registry_cid_number(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let signature_bytes = match parse_signature_bytes(citizen_signature.as_str()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "公民签名格式错误"),
    };
    let call = encode_citizen_identity_call(
        payload.identity_level,
        &actor_cid_number,
        &payload.payload_bytes,
        &signature_bytes,
    );
    let action = crate::core::institution_call::chain_action_code(
        CITIZEN_IDENTITY_PALLET_INDEX,
        payload.identity_level.call_index(),
    );
    // D7:QR 载荷 = 完整 runtime 签名载荷(与钱包解码器扩展尾规则对齐),
    // CitizenWallet 只签名一次并显示响应二维码；OnChina 回扫后经
    // /api/v1/admin/chain/submit 统一组装和提交。
    let prepared =
        match crate::core::chain_submit::prepare_signing(&call, ctx.admin_account.as_str()).await {
            Ok(v) => v,
            Err(err) => {
                tracing::error!(error = %err, "prepare identity push signing failed");
                return api_error(
                    StatusCode::BAD_GATEWAY,
                    1004,
                    "链签名载荷准备失败(链不可用)",
                );
            }
        };
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(600);
    let request_id = format!("citizen-chain-{}", Uuid::new_v4());
    let chain_sign_request = match crate::core::qr::build_sign_request_bytes(
        request_id.as_str(),
        issued_at.timestamp(),
        expires_at.timestamp(),
        ctx.admin_account.as_str(),
        &prepared.payload,
        action,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let session = crate::domains::citizens::occupy::ChainSignSession {
        request_id: request_id.clone(),
        purpose: crate::domains::citizens::occupy::PURPOSE_CITIZEN_IDENTITY_PUSH.to_string(),
        actor_pubkey: ctx.admin_account.clone(),
        call_data: call.clone(),
        nonce: prepared.nonce,
        signing_hash: prepared.signing_hash_hex.clone(),
        context: serde_json::json!({
            "cid_number": record.cid_number,
            "identity_level": payload.identity_level.as_str(),
            "wallet_pubkey": wallet.pubkey,
            "wallet_address": wallet.address,
        }),
        expires_at,
        consumed_at: None,
    };
    if let Err(err) = state.db.insert_chain_sign_session(&session) {
        tracing::error!(error = %err, "insert identity push session failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "冷签会话落库失败");
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CompleteCitizenOnchainOutput {
            request_id,
            cid_number: record.cid_number,
            identity_level: payload.identity_level,
            wallet_address: wallet.address,
            chain_action: action,
            call_data_hex: format!("0x{}", hex::encode(call)),
            citizen_signature,
            citizen_identity_chain_sign_request: chain_sign_request,
        },
    })
    .into_response()
}

#[derive(Clone)]
struct CitizenOnchainOperation {
    operation_id: String,
    admin_account: String,
    institution_code: String,
    cid_number: String,
    wallet_pubkey: String,
    wallet_address: String,
    identity_level: String,
    payload_hex: String,
    expires_at: chrono::DateTime<Utc>,
}

impl Db {
    fn insert_citizen_onchain_operation(
        &self,
        operation: &CitizenOnchainOperation,
    ) -> Result<(), String> {
        let operation = operation.clone();
        self.with_client(move |conn| {
            conn.execute(
                "DELETE FROM citizen_onchain_operations WHERE expires_at < now()",
                &[],
            )
            .map_err(|e| format!("delete expired citizen onchain operations failed: {e}"))?;
            conn.execute(
                "INSERT INTO citizen_onchain_operations
                 (operation_id, admin_account, institution_code, cid_number, wallet_pubkey,
                  wallet_address, identity_level, payload_hex, expires_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
                &[
                    &operation.operation_id,
                    &operation.admin_account,
                    &operation.institution_code,
                    &operation.cid_number,
                    &operation.wallet_pubkey,
                    &operation.wallet_address,
                    &operation.identity_level,
                    &operation.payload_hex,
                    &operation.expires_at,
                ],
            )
            .map_err(|e| format!("insert citizen onchain operation failed: {e}"))?;
            Ok(())
        })
    }

    fn find_citizen_onchain_operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<CitizenOnchainOperation>, String> {
        let operation_id = operation_id.to_string();
        self.with_client(move |conn| {
            let row = conn.query_opt(
                "SELECT operation_id, admin_account, institution_code, cid_number, wallet_pubkey,
                        wallet_address, identity_level, payload_hex, expires_at
                 FROM citizen_onchain_operations
                 WHERE operation_id = $1 AND citizen_signed_at IS NULL AND expires_at >= now()",
                &[&operation_id],
            ).map_err(|e| format!("query citizen onchain operation failed: {e}"))?;
            Ok(row.map(|row| CitizenOnchainOperation {
                operation_id: row.get(0),
                admin_account: row.get(1),
                institution_code: row.get(2),
                cid_number: row.get(3),
                wallet_pubkey: row.get(4),
                wallet_address: row.get(5),
                identity_level: row.get(6),
                payload_hex: row.get(7),
                expires_at: row.get(8),
            }))
        })
    }

    fn consume_citizen_onchain_operation(&self, operation_id: &str) -> Result<bool, String> {
        let operation_id = operation_id.to_string();
        self.with_client(move |conn| {
            conn.execute(
                "UPDATE citizen_onchain_operations SET citizen_signed_at = now()
             WHERE operation_id = $1 AND citizen_signed_at IS NULL AND expires_at >= now()",
                &[&operation_id],
            )
            .map(|count| count == 1)
            .map_err(|e| format!("consume citizen onchain operation failed: {e}"))
        })
    }

    pub(crate) fn find_citizen_by_cid(
        &self,
        cid_number: &str,
    ) -> Result<Option<CitizenRecord>, String> {
        let cid_number = cid_number.trim().to_string();
        if cid_number.is_empty() {
            return Ok(None);
        }
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT COALESCE(id, 0), cid_number, passport_no, citizen_family_name,
                            citizen_given_name, citizen_sex, citizen_birth_date, wallet_pubkey, wallet_address,
                            wallet_sig_alg, wallet_verified_at, citizen_status, voting_eligible,
                            passport_valid_from, passport_valid_until, status_updated_at,
                            province_code, city_code, town_code,
                            birth_province_code, birth_city_code, birth_town_code,
                            archive_hash, onchain_tx_hash, onchain_block_number, onchain_at,
                            created_by, created_at, updated_by, updated_at
                     FROM citizens
                     WHERE cid_number = $1
                     ORDER BY created_at DESC
                     LIMIT 1",
                    &[&cid_number],
                )
                .map_err(|e| format!("query citizen by cid failed: {e}"))?;
            Ok(row.as_ref().map(citizen_record_from_row))
        })
    }
}

struct CitizenIdentityPayloadBytes {
    payload_bytes: Vec<u8>,
    citizen_age_years: u8,
    identity_level: CitizenOnchainIdentityLevel,
}

pub(crate) fn ensure_registry_admin(
    ctx: &crate::auth::login::AdminAuthContext,
) -> Result<(), axum::response::Response> {
    if crate::core::chain_runtime::is_tier1_registry(&ctx.institution_code)
        || crate::core::chain_runtime::is_subordinate_registry(&ctx.institution_code)
    {
        Ok(())
    } else {
        Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "只有注册局管理员可以推送公民身份上链",
        ))
    }
}

fn ensure_record_in_admin_scope(
    ctx: &crate::auth::login::AdminAuthContext,
    record: &CitizenRecord,
) -> Result<(), axum::response::Response> {
    let scope = crate::scope::get_visible_scope(ctx);
    let province_name =
        crate::cid::china::area_name_by_codes(record.province_code.as_str(), None, None)
            .map(|(province, _, _)| province.to_string())
            .unwrap_or_default();
    let city_name = crate::cid::china::area_name_by_codes(
        record.province_code.as_str(),
        Some(record.city_code.as_str()),
        None,
    )
    .and_then(|(_, city, _)| city.map(str::to_string))
    .unwrap_or_default();
    if !scope.includes_province(province_name.as_str()) || !scope.includes_city(city_name.as_str())
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "公民档案不在当前注册局办理范围内",
        ));
    }
    Ok(())
}

fn ensure_wallet_available(
    state: &AppState,
    record: &CitizenRecord,
    wallet: &ResolvedWallet,
) -> Result<(), axum::response::Response> {
    if let Some(existing) = record.wallet_pubkey.as_deref() {
        if !same_pubkey_hex(existing, wallet.pubkey.as_str()) {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "该公民已绑定其他钱包账户",
            ));
        }
    }
    match state.db.find_citizen_by_wallet(wallet.pubkey.as_str()) {
        Ok(Some(existing)) if existing.cid_number != record.cid_number => Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "该钱包账户已绑定其他公民档案",
        )),
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::error!(error = %err, "query citizen wallet duplicate failed");
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "钱包账户查重失败",
            ))
        }
    }
}

fn build_voting_identity_payload(
    record: &CitizenRecord,
    wallet: &ResolvedWallet,
) -> Result<CitizenIdentityPayloadBytes, axum::response::Response> {
    if record.citizen_status != CitizenStatus::Normal
        || record.computed_identity_status() != CitizenStatus::Normal
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "只有有效公民档案可以推送上链",
        ));
    }
    if !record.voting_eligible {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "无选举资格的公民不能推送链上投票身份",
        ));
    }
    let birth_date = NaiveDate::parse_from_str(record.citizen_birth_date.as_str(), "%Y-%m-%d")
        .map_err(|_| api_error(StatusCode::BAD_REQUEST, 1001, "公民出生日期格式错误"))?;
    let age = citizen_age_years(Utc::now().date_naive(), birth_date);
    if age < MIN_ONCHAIN_CITIZEN_AGE_YEARS {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "未满16周岁不能推送链上身份",
        ));
    }
    let wallet_account = parse_sr25519_pubkey_bytes(wallet.pubkey.as_str())
        .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, 1001, "钱包账户不是 32 字节公钥"))?;
    let valid_from = passport_date_u32(record.passport_valid_from.as_str(), "passport_valid_from")?;
    let valid_until =
        passport_date_u32(record.passport_valid_until.as_str(), "passport_valid_until")?;

    let mut out = Vec::new();
    append_bounded_bytes(&mut out, record.cid_number.as_bytes(), 32, "cid_number")?;
    out.extend_from_slice(&wallet_account);
    out.push(age);
    out.extend(valid_from.to_le_bytes());
    out.extend(valid_until.to_le_bytes());
    out.push(0); // CitizenStatus::Normal
    append_bounded_bytes(
        &mut out,
        record.province_code.as_bytes(),
        16,
        "province_code",
    )?;
    append_bounded_bytes(&mut out, record.city_code.as_bytes(), 16, "city_code")?;
    append_bounded_bytes(&mut out, record.town_code.as_bytes(), 16, "town_code")?;
    Ok(CitizenIdentityPayloadBytes {
        payload_bytes: out,
        citizen_age_years: age,
        identity_level: CitizenOnchainIdentityLevel::Voting,
    })
}

fn build_citizen_identity_payload(
    record: &CitizenRecord,
    wallet: &ResolvedWallet,
    identity_level: CitizenOnchainIdentityLevel,
) -> Result<CitizenIdentityPayloadBytes, axum::response::Response> {
    let mut payload = build_voting_identity_payload(record, wallet)?;
    if identity_level == CitizenOnchainIdentityLevel::Voting {
        return Ok(payload);
    }

    append_bounded_bytes(
        &mut payload.payload_bytes,
        record.birth_province_code.as_bytes(),
        16,
        "birth_province_code",
    )?;
    append_bounded_bytes(
        &mut payload.payload_bytes,
        record.birth_city_code.as_bytes(),
        16,
        "birth_city_code",
    )?;
    append_bounded_bytes(
        &mut payload.payload_bytes,
        record.birth_town_code.as_bytes(),
        16,
        "birth_town_code",
    )?;
    let citizen_full_name = format!(
        "{}{}",
        record.citizen_family_name.trim(),
        record.citizen_given_name.trim()
    );
    append_bounded_bytes(
        &mut payload.payload_bytes,
        citizen_full_name.as_bytes(),
        128,
        "citizen_full_name",
    )?;
    let sex = match record.citizen_sex.trim().to_ascii_uppercase().as_str() {
        "MALE" => 0,
        "FEMALE" => 1,
        _ => {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "公民性别不能编码为参选身份",
            ));
        }
    };
    payload.payload_bytes.push(sex);
    // birth_date: u32 YYYYMMDD(LE),CandidateIdentityPayload 末字段。
    // 出生日期是注册局新增公民时必填、写入后不可修改的档案字段(citizen_birth_date),
    // 链端凭此实时计算竞选公民年龄。SCALE 布局须与链端结构体逐字节一致。
    let birth_date = NaiveDate::parse_from_str(record.citizen_birth_date.as_str(), "%Y-%m-%d")
        .map_err(|_| api_error(StatusCode::BAD_REQUEST, 1001, "公民出生日期格式错误"))?;
    let birth_date_u32 =
        birth_date.year() as u32 * 10_000 + birth_date.month() * 100 + birth_date.day();
    payload.payload_bytes.extend(birth_date_u32.to_le_bytes());
    payload.identity_level = CitizenOnchainIdentityLevel::Candidate;
    Ok(payload)
}

fn append_bounded_bytes(
    out: &mut Vec<u8>,
    bytes: &[u8],
    max_len: usize,
    field: &str,
) -> Result<(), axum::response::Response> {
    if bytes.is_empty() || bytes.len() > max_len {
        let detail = format!("{field} 长度不合法");
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, detail.as_str()));
    }
    out.extend(Compact(bytes.len() as u32).encode());
    out.extend_from_slice(bytes);
    Ok(())
}

fn passport_date_u32(value: &str, field: &str) -> Result<u32, axum::response::Response> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        let detail = format!("{field} 必须是 YYYY-MM-DD");
        api_error(StatusCode::BAD_REQUEST, 1001, detail.as_str())
    })?;
    Ok((date.year() as u32) * 10_000 + date.month() * 100 + date.day())
}

fn citizen_age_years(today: NaiveDate, birth_date: NaiveDate) -> u8 {
    let mut age = today.year() - birth_date.year();
    if (today.month(), today.day()) < (birth_date.month(), birth_date.day()) {
        age -= 1;
    }
    u8::try_from(age.max(0)).unwrap_or(u8::MAX)
}

fn verify_citizen_identity_signature(
    wallet_pubkey: &str,
    payload: &[u8],
    signature_hex: &str,
) -> bool {
    let Some(pubkey) = parse_sr25519_pubkey_bytes(wallet_pubkey) else {
        return false;
    };
    let Some(signature) = parse_signature_bytes(signature_hex) else {
        return false;
    };
    let message =
        primitives::sign::signing_message(primitives::sign::OP_SIGN_CITIZEN_IDENTITY, payload);
    let public = sr25519::Public::from_raw(pubkey);
    let signature = sr25519::Signature::from_raw(signature);
    sr25519::Pair::verify(&signature, &message, &public)
}

fn parse_signature_bytes(signature_hex: &str) -> Option<[u8; 64]> {
    let raw = hex::decode(signature_hex.trim_start_matches("0x")).ok()?;
    raw.try_into().ok()
}

pub(crate) fn same_pubkey_hex(left: &str, right: &str) -> bool {
    normalize_prefixed_hex(left).eq_ignore_ascii_case(normalize_prefixed_hex(right))
}

fn normalize_prefixed_hex(value: &str) -> &str {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value)
}

pub(crate) fn active_registry_cid_number(
    state: &AppState,
) -> Result<String, axum::response::Response> {
    let binding = crate::auth::repo::active_node_binding(&state.db).map_err(|err| {
        tracing::error!(error = %err, "query active registry binding failed");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "注册局绑定查询失败",
        )
    })?;
    let cid_number = binding
        .map(|binding| binding.institution_cid_number)
        .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, 1001, "当前注册局缺少机构 CID 绑定"))?;
    if cid_number.is_empty()
        || cid_number.len() > primitives::core_const::CID_NUMBER_MAX_BYTES as usize
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "注册局机构 CID 格式错误",
        ));
    }
    Ok(cid_number)
}

fn encode_citizen_identity_call(
    identity_level: CitizenOnchainIdentityLevel,
    actor_cid_number: &str,
    payload_bytes: &[u8],
    citizen_signature: &[u8; 64],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(CITIZEN_IDENTITY_PALLET_INDEX);
    out.push(identity_level.call_index());
    out.extend(Compact(actor_cid_number.len() as u32).encode());
    out.extend_from_slice(actor_cid_number.as_bytes());
    out.extend_from_slice(payload_bytes);
    out.extend(Compact(citizen_signature.len() as u32).encode());
    out.extend_from_slice(citizen_signature);
    out
}
