//! 公民电子护照绑定 handler。
//!
//! SFID 只接受 CPMS 档案码中的钱包信息，并要求 wuminapp 对 SFID 下发的
//! `sign_request` 完成 sr25519 签名；验签通过后，SFID 本地写入绑定结果。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sha2::Digest;
use uuid::Uuid;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::login::AdminAuthContext;
use crate::admins::operation_auth::AdminActionType;
use crate::cpms::CpmsArchiveCodePayload;
use crate::*;

const BIND_CHALLENGE_TTL_SECONDS: i64 = 300;

/// 生成电子护照绑定 challenge。
pub(crate) async fn citizen_bind_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CitizenBindChallengeInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mode = input.mode.trim();
    if mode != "create" && mode != "replace" {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "mode must be create or replace",
        );
    }
    if mode == "replace" && input.citizen_id.is_none() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "citizen_id is required for replace",
        );
    }

    let archive_code: CpmsArchiveCodePayload =
        match serde_json::from_str(input.archive_code_payload.trim()) {
            Ok(v) => v,
            Err(_) => {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "invalid archive code payload",
                )
            }
        };
    let verified = match crate::cpms::verify_cpms_archive_qr(
        &state,
        &archive_code,
        ctx.admin_province.as_deref(),
    )
    .await
    {
        Ok(v) => v,
        Err((status, code, msg)) => return api_error(status, code, msg.as_str()),
    };
    if let Err(resp) =
        ensure_verified_archive_in_admin_scope(&state, &ctx, verified.sfid_number.as_str())
    {
        return resp;
    }
    if verified.wallet_sig_alg != "sr25519" {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "wallet_sig_alg must be sr25519",
        );
    }
    let (wallet_address, wallet_pubkey) = match resolve_bind_wallet(verified.wallet_address.trim())
    {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid wallet_address"),
    };
    if !same_pubkey_hex(&wallet_pubkey, &verified.wallet_pubkey) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "archive wallet pubkey mismatch",
        );
    }

    {
        if mode == "create" {
            match state.db.find_bound_citizen_by_archive(&verified.archive_no) {
                Ok(Some(_)) => {
                    return api_error(StatusCode::CONFLICT, 1005, "archive_no already bound")
                }
                Ok(None) => {}
                Err(err) => {
                    tracing::error!(error = %err, "query citizen archive owner failed");
                    return api_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        1004,
                        "citizen query failed",
                    );
                }
            }
        } else {
            let citizen_id = input.citizen_id.unwrap();
            let record = match state.db.find_bound_citizen_by_id(citizen_id) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found")
                }
                Err(err) => {
                    tracing::error!(error = %err, "query citizen record failed");
                    return api_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        1004,
                        "citizen query failed",
                    );
                }
            };
            if record.bind_status() != CitizenBindStatus::Bound {
                return api_error(StatusCode::CONFLICT, 1005, "citizen record is not bound");
            }
            if record.archive_no.as_deref() != Some(verified.archive_no.as_str()) {
                return api_error(
                    StatusCode::CONFLICT,
                    1005,
                    "archive_no immutable after binding",
                );
            }
            if let Some(current_updated_at) = record.status_updated_at {
                if verified.status_updated_at < current_updated_at {
                    return api_error(StatusCode::CONFLICT, 1005, "citizen status is stale");
                }
            }
            match state.db.find_bound_citizen_by_archive(&verified.archive_no) {
                Ok(Some(owner)) if owner.id != citizen_id => {
                    return api_error(StatusCode::CONFLICT, 1005, "archive_no already bound")
                }
                Ok(_) => {}
                Err(err) => {
                    tracing::error!(error = %err, "query archive owner failed");
                    return api_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        1004,
                        "citizen query failed",
                    );
                }
            }
        }
        match state.db.find_bound_citizen_by_wallet(&wallet_pubkey) {
            Ok(Some(owner)) if mode == "create" || Some(owner.id) != input.citizen_id => {
                return api_error(StatusCode::CONFLICT, 1005, "wallet_pubkey already bound");
            }
            Ok(_) => {}
            Err(err) => {
                tracing::error!(error = %err, "query wallet owner failed");
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "citizen query failed",
                );
            }
        }
    }

    let challenge_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let challenge_text = build_challenge_text(
        &challenge_id,
        mode,
        &verified.archive_no,
        &verified.citizen_status,
        verified.voting_eligible,
        &verified.valid_from,
        &verified.valid_until,
        verified.status_updated_at,
        &wallet_pubkey,
        now,
    );
    let expire_at = now + chrono::Duration::seconds(BIND_CHALLENGE_TTL_SECONDS);
    let sign_request_str = build_citizen_bind_sign_request(
        &challenge_id,
        now,
        expire_at,
        &challenge_text,
        &wallet_address,
        &wallet_pubkey,
        &verified.archive_no,
        &verified.citizen_status,
        verified.voting_eligible,
        mode,
    );

    let challenge = CitizenBindChallenge {
        challenge_id: challenge_id.clone(),
        challenge_text: challenge_text.clone(),
        mode: mode.to_string(),
        citizen_id: input.citizen_id,
        archive_no: verified.archive_no.clone(),
        wallet_address: wallet_address.clone(),
        wallet_pubkey: wallet_pubkey.clone(),
        wallet_sig_alg: verified.wallet_sig_alg.clone(),
        citizen_status: verified.citizen_status.clone(),
        voting_eligible: verified.voting_eligible,
        archive_valid_from: verified.valid_from.clone(),
        archive_valid_until: verified.valid_until.clone(),
        status_updated_at: verified.status_updated_at,
        province_code: verified.province_code.clone(),
        city_code: verified.city_code.clone(),
        expire_at,
        created_at: now,
    };
    if let Err(err) = state.db.insert_citizen_bind_challenge(&challenge) {
        tracing::error!(error = %err, "insert citizen bind challenge failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "citizen bind challenge write failed",
        );
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CitizenBindChallengeOutput {
            challenge_id,
            challenge_text,
            mode: mode.to_string(),
            archive_no: verified.archive_no,
            wallet_address,
            wallet_pubkey,
            citizen_status: verified.citizen_status,
            voting_eligible: verified.voting_eligible,
            valid_from: verified.valid_from,
            valid_until: verified.valid_until,
            status_updated_at: verified.status_updated_at,
            sign_request: sign_request_str,
            expire_at: expire_at.timestamp(),
        },
    })
    .into_response()
}

/// 完成电子护照绑定。
pub(crate) async fn citizen_bind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CitizenBindInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.challenge_id.trim().is_empty()
        || input.pubkey.trim().is_empty()
        || input.signature.trim().is_empty()
        || input.payload_hash.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, pubkey, signature, payload_hash are required",
        );
    }
    let challenge_id = input.challenge_id.trim().to_string();
    let grant_payload = serde_json::json!({
        "target": challenge_id.clone(),
        "challenge_id": challenge_id.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::CitizenBindCommit,
        challenge_id.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }

    let challenge = match state.db.take_citizen_bind_challenge(&challenge_id) {
        Ok(Some(v)) => {
            if Utc::now() > v.expire_at {
                return api_error(StatusCode::GONE, 1007, "challenge expired");
            }
            v
        }
        Ok(None) => {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "challenge not found or expired",
            )
        }
        Err(err) => {
            tracing::error!(error = %err, "take citizen bind challenge failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "challenge query failed",
            );
        }
    };

    let wallet_pubkey = input.pubkey.trim().to_lowercase();
    if !same_pubkey_hex(&wallet_pubkey, &challenge.wallet_pubkey) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "challenge wallet mismatch",
        );
    }
    let expected_hash = payload_hash_for_text(&challenge.challenge_text);
    if input.payload_hash.trim().to_lowercase() != expected_hash {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "payload hash mismatch",
        );
    }

    let pubkey_bytes = match crate::admins::login::parse_sr25519_pubkey_bytes(&wallet_pubkey) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid wallet_pubkey"),
    };
    let sig_bytes = match hex::decode(input.signature.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid signature hex"),
    };
    if !verify_citizen_bind_signature(&pubkey_bytes, &challenge.challenge_text, &sig_bytes) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "signature verify failed",
        );
    }

    let record = if challenge.mode == "create" {
        match create_citizen_record(&state, &ctx, &challenge) {
            Ok(v) => v,
            Err(resp) => return resp,
        }
    } else {
        match replace_citizen_record(&state, &ctx, &challenge) {
            Ok(v) => v,
            Err(resp) => return resp,
        }
    };
    if let Err(e) = state.db.upsert_citizen_row(&record) {
        tracing::error!(error = %e, "citizen row upsert failed");
        if e.contains("duplicate key") {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "citizen unique key already exists",
            );
        }
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "citizen row write failed",
        );
    }
    crate::core::runtime_ops::append_audit_log(
        &state,
        "CITIZEN_BIND",
        &ctx.admin_pubkey,
        record.sfid_number.clone(),
        serde_json::json!({
            "mode": challenge.mode.clone(),
            "sfid_number": record.sfid_number.clone(),
            "archive_no": record.archive_no.clone(),
            "request_id": request_id_from_headers(&headers),
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );
    let output = citizen_bind_output(&record);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

fn ensure_verified_archive_in_admin_scope(
    state: &AppState,
    ctx: &AdminAuthContext,
    sfid_number: &str,
) -> Result<(), axum::response::Response> {
    let site = state
        .db
        .get_cpms_site(sfid_number)
        .map_err(|err| {
            tracing::error!(error = %err, "query cpms site failed");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "cpms query failed")
        })?
        .ok_or_else(|| {
            api_error(
                StatusCode::NOT_FOUND,
                1004,
                "cpms install authorization not found",
            )
        })?;
    if let Some(province) = ctx.admin_province.as_deref() {
        if site.admin_province != province {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                1003,
                "province out of current admin scope",
            ));
        }
    }
    if let Some(city) = ctx.admin_city.as_deref() {
        if site.city_name != city {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                1003,
                "city out of current admin scope",
            ));
        }
    }
    Ok(())
}

fn create_citizen_record(
    state: &AppState,
    ctx: &AdminAuthContext,
    challenge: &CitizenBindChallenge,
) -> Result<CitizenRecord, axum::response::Response> {
    if let Some(_) = state
        .db
        .find_bound_citizen_by_archive(&challenge.archive_no)
        .map_err(|err| {
            tracing::error!(error = %err, "query archive owner failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "citizen query failed",
            )
        })?
    {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "archive_no already bound",
        ));
    }
    if let Some(_) = state
        .db
        .find_bound_citizen_by_wallet(&challenge.wallet_pubkey)
        .map_err(|err| {
            tracing::error!(error = %err, "query wallet owner failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "citizen query failed",
            )
        })?
    {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "wallet_pubkey already bound",
        ));
    }

    let province_name = crate::china::province_name_by_code(&challenge.province_code).unwrap_or("");
    let sfid_number = generate_unique_citizen_sfid(state, province_name, &challenge.wallet_pubkey)?;
    let cid = state.db.next_citizen_id().map_err(|err| {
        tracing::error!(error = %err, "allocate citizen id failed");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "citizen id allocate failed",
        )
    })?;
    Ok(CitizenRecord {
        id: cid,
        wallet_pubkey: Some(challenge.wallet_pubkey.clone()),
        wallet_address: Some(challenge.wallet_address.clone()),
        archive_no: Some(challenge.archive_no.clone()),
        sfid_number: Some(sfid_number.clone()),
        citizen_status: Some(challenge.citizen_status.clone()),
        voting_eligible: challenge.voting_eligible,
        archive_valid_from: Some(challenge.archive_valid_from.clone()),
        archive_valid_until: Some(challenge.archive_valid_until.clone()),
        status_updated_at: Some(challenge.status_updated_at),
        sfid_signature: None,
        province_code: Some(challenge.province_code.clone()),
        city_code: Some(challenge.city_code.clone()),
        bound_at: Some(Utc::now()),
        bound_by: Some(ctx.admin_pubkey.clone()),
        created_at: Utc::now(),
    })
}

fn replace_citizen_record(
    state: &AppState,
    ctx: &AdminAuthContext,
    challenge: &CitizenBindChallenge,
) -> Result<CitizenRecord, axum::response::Response> {
    let citizen_id = challenge.citizen_id.unwrap_or_default();
    let Some(existing) = state
        .db
        .find_bound_citizen_by_id(citizen_id)
        .map_err(|err| {
            tracing::error!(error = %err, "query existing citizen failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "citizen query failed",
            )
        })?
    else {
        return Err(api_error(
            StatusCode::NOT_FOUND,
            1004,
            "citizen record not found",
        ));
    };
    if existing.bind_status() != CitizenBindStatus::Bound {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "citizen record is not bound",
        ));
    }
    let existing_archive_no = existing
        .archive_no
        .clone()
        .ok_or_else(|| api_error(StatusCode::CONFLICT, 1005, "citizen record is not bound"))?;
    if existing_archive_no != challenge.archive_no {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "archive_no immutable after binding",
        ));
    }
    if let Some(current_updated_at) = existing.status_updated_at {
        if challenge.status_updated_at < current_updated_at {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "citizen status is stale",
            ));
        }
    }
    if let Some(owner) = state
        .db
        .find_bound_citizen_by_archive(&challenge.archive_no)
        .map_err(|err| {
            tracing::error!(error = %err, "query archive owner failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "citizen query failed",
            )
        })?
    {
        if owner.id != citizen_id {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "archive_no already bound",
            ));
        }
    }
    if let Some(owner) = state
        .db
        .find_bound_citizen_by_wallet(&challenge.wallet_pubkey)
        .map_err(|err| {
            tracing::error!(error = %err, "query wallet owner failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "citizen query failed",
            )
        })?
    {
        if owner.id != citizen_id {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "wallet_pubkey already bound",
            ));
        }
    }
    let mut record = existing;
    record.wallet_pubkey = Some(challenge.wallet_pubkey.clone());
    record.wallet_address = Some(challenge.wallet_address.clone());
    record.citizen_status = Some(challenge.citizen_status.clone());
    record.voting_eligible = challenge.voting_eligible;
    record.archive_valid_from = Some(challenge.archive_valid_from.clone());
    record.archive_valid_until = Some(challenge.archive_valid_until.clone());
    record.status_updated_at = Some(challenge.status_updated_at);
    record.province_code = Some(challenge.province_code.clone());
    record.city_code = Some(challenge.city_code.clone());
    record.bound_at = Some(Utc::now());
    record.bound_by = Some(ctx.admin_pubkey.clone());
    Ok(record)
}

fn citizen_bind_output(record: &CitizenRecord) -> CitizenBindOutput {
    CitizenBindOutput {
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

fn build_challenge_text(
    challenge_id: &str,
    mode: &str,
    archive_no: &str,
    citizen_status: &CitizenStatus,
    voting_eligible: bool,
    valid_from: &str,
    valid_until: &str,
    status_updated_at: i64,
    wallet_pubkey: &str,
    issued_at: DateTime<Utc>,
) -> String {
    let citizen_status_text = match citizen_status {
        CitizenStatus::Normal => "NORMAL",
        CitizenStatus::Revoked => "REVOKED",
    };
    format!(
        "sfid-citizen-bind-v1|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        challenge_id,
        mode,
        archive_no,
        citizen_status_text,
        voting_eligible,
        valid_from,
        valid_until,
        status_updated_at,
        wallet_pubkey,
        issued_at.timestamp()
    )
}

fn build_citizen_bind_sign_request(
    challenge_id: &str,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    challenge_text: &str,
    wallet_address: &str,
    wallet_pubkey: &str,
    archive_no: &str,
    citizen_status: &CitizenStatus,
    voting_eligible: bool,
    mode: &str,
) -> String {
    let citizen_status_text = match citizen_status {
        CitizenStatus::Normal => "正常",
        CitizenStatus::Revoked => "注销",
    };
    let voting_eligible_text = if voting_eligible { "有" } else { "无" };
    let mode_label = if mode == "replace" {
        "更换绑定"
    } else {
        "新增身份ID绑定"
    };
    let summary = if mode == "replace" {
        "确认更换绑定"
    } else {
        "确认新增身份ID绑定"
    };
    let sign_request = serde_json::json!({
        "proto": crate::core::qr::WUMIN_QR_V1,
        "kind": "sign_request",
        "id": challenge_id,
        "issued_at": issued_at.timestamp(),
        "expires_at": expires_at.timestamp(),
        "body": {
            "address": wallet_address,
            "pubkey": wallet_pubkey,
            "sig_alg": "sr25519",
            "payload_hex": format!("0x{}", hex::encode(challenge_text.as_bytes())),
            "display": {
                "action": "citizen_bind",
                "summary": summary,
                "fields": [
                    { "key": "mode", "label": "操作", "value": mode_label },
                    { "key": "archive_no", "label": "档案号", "value": archive_no },
                    { "key": "voting_eligible", "label": "选举权利", "value": voting_eligible_text },
                    { "key": "citizen_status", "label": "公民状态", "value": citizen_status_text },
                    { "key": "wallet_address", "label": "投票账户", "value": wallet_address }
                ]
            }
        }
    });
    serde_json::to_string(&sign_request).unwrap_or_default()
}

fn generate_unique_citizen_sfid(
    state: &AppState,
    province_name: &str,
    wallet_pubkey: &str,
) -> Result<String, axum::response::Response> {
    for retry in 0..1000u32 {
        let attempt_pubkey = if retry == 0 {
            wallet_pubkey.to_string()
        } else {
            format!("{}#{retry}", wallet_pubkey)
        };
        let candidate =
            match crate::number::generate_sfid_number(crate::number::GenerateSfidInput {
                account_pubkey: attempt_pubkey.as_str(),
                subject_property: "M",
                p1: "1",
                province: province_name,
                city: "省辖市",
                institution: "ZG",
            }) {
                Ok(v) => v,
                Err(msg) => return Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg)),
            };
        let exists = state.db.sfid_exists(&candidate).map_err(|err| {
            tracing::error!(error = %err, "query sfid exists failed");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "sfid query failed")
        })?;
        if !exists {
            return Ok(candidate);
        }
    }
    Err(api_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        1099,
        "SFID 桶饱和(N/10⁹>99.9%),协议需扩容",
    ))
}

/// 验证公民电子护照绑定签名（sr25519，substrate context）。
pub(crate) fn verify_citizen_bind_signature(
    pubkey_bytes: &[u8; 32],
    message: &str,
    signature: &[u8],
) -> bool {
    use schnorrkel::{
        signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature,
    };
    let pk = match Sr25519PublicKey::from_bytes(pubkey_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let sig = match Sr25519Signature::from_bytes(signature) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let ctx = signing_context(b"substrate");
    pk.verify(ctx.bytes(message.as_bytes()), &sig).is_ok()
}

fn resolve_bind_wallet(address: &str) -> Option<(String, String)> {
    let wallet_pubkey = ss58_to_pubkey_hex(address)?;
    let canonical_address = pubkey_hex_to_ss58(&wallet_pubkey)?;
    if canonical_address != address.trim() {
        return None;
    }
    Some((canonical_address, wallet_pubkey))
}

fn payload_hash_for_text(text: &str) -> String {
    format!(
        "0x{}",
        sha2::Sha256::digest(text.as_bytes())
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    )
}

fn same_pubkey_hex(left: &str, right: &str) -> bool {
    left.trim_start_matches("0x")
        .eq_ignore_ascii_case(right.trim_start_matches("0x"))
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
        archive_no: row.get(3),
        sfid_number: Some(row.get(4)),
        citizen_status: Some(citizen_status_from_db(row.get::<_, String>(5).as_str())),
        voting_eligible: row.get(6),
        archive_valid_from: row.get(7),
        archive_valid_until: row.get(8),
        status_updated_at: row.get(9),
        sfid_signature: None,
        province_code: Some(row.get(10)),
        city_code: Some(row.get(11)),
        bound_at: row.get(12),
        bound_by: row.get(13),
        created_at: row.get(14),
    }
}

impl Db {
    pub(crate) fn insert_citizen_bind_challenge(
        &self,
        challenge: &CitizenBindChallenge,
    ) -> Result<(), String> {
        let challenge = challenge.clone();
        self.with_client(move |conn| {
            conn.execute(
                "DELETE FROM citizen_bind_challenges WHERE expires_at < now()",
                &[],
            )
            .map_err(|e| format!("cleanup citizen challenges failed: {e}"))?;
            let payload = serde_json::to_value(&challenge)
                .map_err(|e| format!("serialize citizen challenge failed: {e}"))?;
            conn.execute(
                "INSERT INTO citizen_bind_challenges (
                    challenge_id, p_code, c_code, wallet_pubkey, archive_no, expires_at, consumed, payload
                 ) VALUES ($1, $2, $3, $4, $5, $6, false, $7)
                 ON CONFLICT (challenge_id) DO UPDATE SET
                    p_code = EXCLUDED.p_code,
                    c_code = EXCLUDED.c_code,
                    wallet_pubkey = EXCLUDED.wallet_pubkey,
                    archive_no = EXCLUDED.archive_no,
                    expires_at = EXCLUDED.expires_at,
                    consumed = false,
                    payload = EXCLUDED.payload",
                &[
                    &challenge.challenge_id,
                    &challenge.province_code,
                    &challenge.city_code,
                    &challenge.wallet_pubkey,
                    &challenge.archive_no,
                    &challenge.expire_at,
                    &payload,
                ],
            )
            .map_err(|e| format!("insert citizen challenge failed: {e}"))?;
            Ok(())
        })
    }

    pub(crate) fn take_citizen_bind_challenge(
        &self,
        challenge_id: &str,
    ) -> Result<Option<CitizenBindChallenge>, String> {
        let challenge_id = challenge_id.trim().to_string();
        self.with_client(move |conn| {
            let mut tx = conn
                .transaction()
                .map_err(|e| format!("begin challenge transaction failed: {e}"))?;
            let row = tx
                .query_opt(
                    "SELECT payload
                     FROM citizen_bind_challenges
                     WHERE challenge_id = $1 AND consumed = false AND expires_at > now()
                     FOR UPDATE",
                    &[&challenge_id],
                )
                .map_err(|e| format!("query citizen challenge failed: {e}"))?;
            let Some(row) = row else {
                tx.commit()
                    .map_err(|e| format!("commit empty challenge transaction failed: {e}"))?;
                return Ok(None);
            };
            tx.execute(
                "UPDATE citizen_bind_challenges SET consumed = true WHERE challenge_id = $1",
                &[&challenge_id],
            )
            .map_err(|e| format!("consume citizen challenge failed: {e}"))?;
            tx.commit()
                .map_err(|e| format!("commit challenge transaction failed: {e}"))?;
            let value: serde_json::Value = row.get(0);
            serde_json::from_value(value)
                .map(Some)
                .map_err(|e| format!("deserialize citizen challenge failed: {e}"))
        })
    }

    pub(crate) fn find_bound_citizen_by_archive(
        &self,
        archive_no: &str,
    ) -> Result<Option<CitizenRecord>, String> {
        let archive_no = archive_no.trim().to_string();
        self.find_bound_citizen_by_clause("archive_no = $1", archive_no)
    }

    pub(crate) fn find_bound_citizen_by_wallet(
        &self,
        wallet_pubkey: &str,
    ) -> Result<Option<CitizenRecord>, String> {
        let wallet_pubkey = wallet_pubkey.trim().to_string();
        self.find_bound_citizen_by_clause("lower(wallet_pubkey) = lower($1)", wallet_pubkey)
    }

    pub(crate) fn find_bound_citizen_by_id(
        &self,
        citizen_id: u64,
    ) -> Result<Option<CitizenRecord>, String> {
        let citizen_id =
            i64::try_from(citizen_id).map_err(|_| "citizen id too large".to_string())?;
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT COALESCE(id, 0), wallet_pubkey, wallet_address, archive_no,
                            sfid_number, citizen_status, voting_eligible, valid_from,
                            valid_until, status_updated_at, p_code, c_code, bound_at,
                            bound_by, created_at
                     FROM citizens
                     WHERE id = $1 AND bind_status = 'BOUND'
                     LIMIT 1",
                    &[&citizen_id],
                )
                .map_err(|e| format!("query citizen by id failed: {e}"))?;
            Ok(row.as_ref().map(citizen_record_from_row))
        })
    }

    fn find_bound_citizen_by_clause(
        &self,
        clause: &'static str,
        value: String,
    ) -> Result<Option<CitizenRecord>, String> {
        self.with_client(move |conn| {
            let sql = format!(
                "SELECT COALESCE(id, 0), wallet_pubkey, wallet_address, archive_no,
                        sfid_number, citizen_status, voting_eligible, valid_from,
                        valid_until, status_updated_at, p_code, c_code, bound_at,
                        bound_by, created_at
                 FROM citizens
                 WHERE {clause} AND bind_status = 'BOUND'
                 ORDER BY created_at DESC
                 LIMIT 1"
            );
            let row = conn
                .query_opt(sql.as_str(), &[&value])
                .map_err(|e| format!("query citizen failed: {e}"))?;
            Ok(row.as_ref().map(citizen_record_from_row))
        })
    }

    pub(crate) fn next_citizen_id(&self) -> Result<u64, String> {
        self.with_client(|conn| {
            let row = conn
                .query_one("SELECT COALESCE(MAX(id), 0) + 1 FROM citizens", &[])
                .map_err(|e| format!("allocate citizen id failed: {e}"))?;
            let id: i64 = row.get(0);
            Ok(u64::try_from(id).unwrap_or(1))
        })
    }

    pub(crate) fn sfid_exists(&self, sfid_number: &str) -> Result<bool, String> {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_one(
                    "SELECT EXISTS(SELECT 1 FROM ids WHERE sfid_number = $1)",
                    &[&sfid_number],
                )
                .map_err(|e| format!("query sfid exists failed: {e}"))?;
            Ok(row.get(0))
        })
    }
}

/// 从 SS58 地址解出 hex 格式公钥。
pub(crate) fn ss58_to_pubkey_hex(address: &str) -> Option<String> {
    let decoded = bs58::decode(address.trim()).into_vec().ok()?;
    let prefix_len = if decoded.first().copied().unwrap_or(0) < 64 {
        1
    } else {
        2
    };
    if decoded.len() < prefix_len + 32 + 2 {
        return None;
    }
    let pubkey = &decoded[prefix_len..prefix_len + 32];
    Some(format!("0x{}", hex::encode(pubkey)))
}

/// 0x hex 公钥转 SS58 地址（prefix=2027）。
pub(crate) fn pubkey_hex_to_ss58(pubkey_hex: &str) -> Option<String> {
    let pubkey_bytes = hex::decode(pubkey_hex.trim_start_matches("0x")).ok()?;
    if pubkey_bytes.len() != 32 {
        return None;
    }
    use blake2::{digest::VariableOutput, Blake2bVar};
    let prefix: u16 = 2027;
    let first = ((prefix & 0b0000_0000_1111_1100) as u8) >> 2 | 0b01000000;
    let second = (prefix >> 8) as u8 | ((prefix & 0b0000_0000_0000_0011) as u8) << 6;
    let mut payload = vec![first, second];
    payload.extend_from_slice(&pubkey_bytes);
    let mut hasher = Blake2bVar::new(64).ok()?;
    use blake2::digest::Update;
    hasher.update(b"SS58PRE");
    hasher.update(&payload);
    let mut hash = vec![0u8; 64];
    hasher.finalize_variable(&mut hash).ok()?;
    payload.extend_from_slice(&hash[..2]);
    Some(bs58::encode(payload).into_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn citizen_bind_sign_request_includes_locked_wallet() {
        let issued_at = Utc.timestamp_opt(1_000, 0).single().unwrap();
        let expires_at = Utc.timestamp_opt(1_300, 0).single().unwrap();
        let raw = build_citizen_bind_sign_request(
            "challenge-1",
            issued_at,
            expires_at,
            "sfid-citizen-bind-v1|challenge-1|create|ARCHIVE-1|NORMAL|true|2026-05-24|2036-05-23|1000|0xabc|1000",
            "addr2027",
            "0xabc",
            "ARCHIVE-1",
            &CitizenStatus::Normal,
            true,
            "create",
        );
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();

        assert_eq!(value["kind"], "sign_request");
        assert_eq!(value["id"], "challenge-1");
        assert_eq!(value["body"]["address"], "addr2027");
        assert_eq!(value["body"]["pubkey"], "0xabc");
        assert_eq!(value["body"]["display"]["fields"][4]["value"], "addr2027");
    }
}
