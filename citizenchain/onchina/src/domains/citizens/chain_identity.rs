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

use crate::auth::actions::require_admin_security_grant;
use crate::auth::operation_auth::AdminActionType;
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
    // 注册局上链操作最严档:passkey 断言 + 冷钱包扫码签名 grant,
    // grant 与 cid_number/wallet_account 载荷绑定,单次消费。
    let grant_payload = serde_json::json!({
        "cid_number": cid_number,
        "wallet_account": input.wallet_account,
        "identity_level": input.identity_level.as_str(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CitizenOnchainPush,
        cid_number.as_str(),
        Some(&grant_payload),
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
        crate::core::qr::ACTION_CITIZEN_IDENTITY,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

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
    // complete 同为上链操作,单独消费一次最严档 grant;载荷绑定不含
    // sign_response(公民回执在 grant 签发时尚不存在)。
    let grant_payload = serde_json::json!({
        "cid_number": cid_number,
        "wallet_account": input.wallet_account,
        "identity_level": input.identity_level.as_str(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CitizenOnchainPush,
        cid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let wallet = match resolve_wallet_account(input.wallet_account.as_str()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut record = match state.db.find_citizen_by_cid(cid_number.as_str()) {
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
    // 回签后经 /citizens/chain/submit 由 onchina 组装提交,QR 只签不提交。
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
        }),
        expires_at,
        consumed_at: None,
    };
    if let Err(err) = state.db.insert_chain_sign_session(&session) {
        tracing::error!(error = %err, "insert identity push session failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "冷签会话落库失败");
    }

    let now = Utc::now();
    record.wallet_pubkey = Some(wallet.pubkey.clone());
    record.wallet_address = Some(wallet.address.clone());
    record.wallet_sig_alg = Some("sr25519".to_string());
    record.wallet_verified_at = Some(now);
    record.updated_by = Some(ctx.admin_account.clone());
    record.updated_at = now;
    if let Err(err) = state.db.upsert_citizen_row(&record) {
        tracing::error!(error = %err, "update citizen wallet binding failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "公民钱包绑定落库失败",
        );
    }

    crate::core::runtime_ops::append_audit_log(
        &state,
        "CITIZEN_ONCHAIN_PREPARE",
        &ctx.admin_account,
        Some(record.cid_number.clone()),
        serde_json::json!({
            "cid_number": record.cid_number,
            "identity_level": payload.identity_level.as_str(),
            "wallet_address": wallet.address,
            "citizen_age_years": payload.citizen_age_years,
            "request_id": request_id_from_headers(&headers),
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );

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

impl Db {
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
    payload
        .payload_bytes
        .extend(birth_date_u32.to_le_bytes());
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
        .and_then(|binding| binding.candidate.institution_cid_number)
        .ok_or_else(|| {
            api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "当前注册局缺少机构 CID 绑定",
            )
        })?;
    if cid_number.is_empty() || cid_number.len() > 32 {
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
