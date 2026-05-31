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
use crate::admins::operation_auth::AdminActionType;
use crate::cpms::CpmsArchiveCodePayload;
use crate::login::AdminAuthContext;
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
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if mode == "create" {
            if existing_bound_archive_owner(&store, &verified.archive_no).is_some() {
                return api_error(StatusCode::CONFLICT, 1005, "archive_no already bound");
            }
        } else {
            let citizen_id = input.citizen_id.unwrap();
            let Some(record) = store.citizen_records.get(&citizen_id) else {
                return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found");
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
            if let Some(owner) = store.citizen_id_by_archive_no.get(&verified.archive_no) {
                if *owner != citizen_id && is_bound_citizen_record(&store, *owner) {
                    return api_error(StatusCode::CONFLICT, 1005, "archive_no already bound");
                }
            }
        }
        if let Some(owner) = store
            .citizen_id_by_wallet_pubkey
            .get(wallet_pubkey.as_str())
        {
            if (mode == "create" || Some(*owner) != input.citizen_id)
                && is_bound_citizen_record(&store, *owner)
            {
                return api_error(StatusCode::CONFLICT, 1005, "wallet_pubkey already bound");
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

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    store
        .citizen_bind_challenges
        .retain(|_, c| c.expire_at > Utc::now());
    store.citizen_bind_challenges.insert(
        challenge_id.clone(),
        CitizenBindChallenge {
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
        },
    );

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

    let challenge = {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let Some(challenge) = store.citizen_bind_challenges.remove(challenge_id.as_str()) else {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "challenge not found or expired",
            );
        };
        if Utc::now() > challenge.expire_at {
            return api_error(StatusCode::GONE, 1007, "challenge expired");
        }
        challenge
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

    let pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&wallet_pubkey) {
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

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let cid = if challenge.mode == "create" {
        match create_citizen_record(&mut store, &ctx, &challenge) {
            Ok(v) => v,
            Err(resp) => return resp,
        }
    } else {
        match replace_citizen_record(&mut store, &ctx, &challenge) {
            Ok(v) => v,
            Err(resp) => return resp,
        }
    };

    let record = store.citizen_records.get(&cid).cloned().unwrap();
    append_audit_log_with_meta(
        &mut store,
        "CITIZEN_BIND",
        &ctx.admin_pubkey,
        record.wallet_pubkey.clone(),
        record.archive_no.clone(),
        request_id_from_headers(&headers),
        actor_ip_from_headers(&headers),
        "SUCCESS",
        format!(
            "mode={} sfid_code={}",
            challenge.mode,
            record.sfid_code.clone().unwrap_or_default()
        ),
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
    site_sfid: &str,
) -> Result<(), axum::response::Response> {
    let store = store_read_or_500(state)?;
    let Some(site) = store.cpms_site_keys.get(site_sfid) else {
        return Err(api_error(
            StatusCode::NOT_FOUND,
            1004,
            "cpms install authorization not found",
        ));
    };
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
    store: &mut Store,
    ctx: &AdminAuthContext,
    challenge: &CitizenBindChallenge,
) -> Result<u64, axum::response::Response> {
    if let Some(owner) = store
        .citizen_id_by_archive_no
        .get(&challenge.archive_no)
        .copied()
    {
        if is_bound_citizen_record(store, owner) {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "archive_no already bound",
            ));
        }
        cleanup_stale_citizen_record(store, owner);
        store.citizen_id_by_archive_no.remove(&challenge.archive_no);
    }
    if existing_bound_archive_owner(store, &challenge.archive_no).is_some() {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "archive_no already bound",
        ));
    }
    if let Some(owner) = store
        .citizen_id_by_wallet_pubkey
        .get(challenge.wallet_pubkey.as_str())
        .copied()
    {
        if is_bound_citizen_record(store, owner) {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "wallet_pubkey already bound",
            ));
        }
        cleanup_stale_citizen_record(store, owner);
        store
            .citizen_id_by_wallet_pubkey
            .remove(challenge.wallet_pubkey.as_str());
    }
    if existing_bound_wallet_owner(store, &challenge.wallet_pubkey).is_some() {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "wallet_pubkey already bound",
        ));
    }

    let province_name =
        crate::sfid::province::province_name_by_code(&challenge.province_code).unwrap_or("");
    let sfid_code = generate_unique_citizen_sfid(store, province_name, &challenge.wallet_pubkey)?;
    let cid = store.next_citizen_id;
    store.next_citizen_id += 1;
    let record = CitizenRecord {
        id: cid,
        wallet_pubkey: Some(challenge.wallet_pubkey.clone()),
        wallet_address: Some(challenge.wallet_address.clone()),
        archive_no: Some(challenge.archive_no.clone()),
        sfid_code: Some(sfid_code.clone()),
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
    };
    store.citizen_records.insert(cid, record);
    store
        .citizen_id_by_archive_no
        .insert(challenge.archive_no.clone(), cid);
    store
        .citizen_id_by_wallet_pubkey
        .insert(challenge.wallet_pubkey.clone(), cid);
    store.citizen_id_by_sfid_code.insert(sfid_code, cid);
    Ok(cid)
}

fn replace_citizen_record(
    store: &mut Store,
    ctx: &AdminAuthContext,
    challenge: &CitizenBindChallenge,
) -> Result<u64, axum::response::Response> {
    let citizen_id = challenge.citizen_id.unwrap_or_default();
    let Some(existing) = store.citizen_records.get(&citizen_id).cloned() else {
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
    if let Some(owner) = store.citizen_id_by_archive_no.get(&challenge.archive_no) {
        if *owner != citizen_id && is_bound_citizen_record(store, *owner) {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "archive_no already bound",
            ));
        }
        if *owner != citizen_id {
            cleanup_stale_citizen_record(store, *owner);
        }
    }
    if let Some(owner) = store
        .citizen_id_by_wallet_pubkey
        .get(&challenge.wallet_pubkey)
        .copied()
    {
        if owner != citizen_id && is_bound_citizen_record(store, owner) {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "wallet_pubkey already bound",
            ));
        }
        if owner != citizen_id {
            cleanup_stale_citizen_record(store, owner);
        }
    }
    if let Some(old_pubkey) = existing.wallet_pubkey {
        store.citizen_id_by_wallet_pubkey.remove(&old_pubkey);
    }
    let record = store.citizen_records.get_mut(&citizen_id).unwrap();
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
    store
        .citizen_id_by_archive_no
        .insert(existing_archive_no, citizen_id);
    store
        .citizen_id_by_wallet_pubkey
        .insert(challenge.wallet_pubkey.clone(), citizen_id);
    Ok(citizen_id)
}

fn citizen_bind_output(record: &CitizenRecord) -> CitizenBindOutput {
    CitizenBindOutput {
        id: record.id,
        wallet_pubkey: record.wallet_pubkey.clone(),
        wallet_address: record.wallet_address.clone(),
        archive_no: record.archive_no.clone(),
        sfid_code: record.sfid_code.clone(),
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

fn existing_bound_archive_owner<'a>(store: &'a Store, archive_no: &str) -> Option<&'a u64> {
    store
        .citizen_id_by_archive_no
        .get(archive_no)
        .filter(|cid| is_bound_citizen_record(store, **cid))
}

fn existing_bound_wallet_owner<'a>(store: &'a Store, wallet_pubkey: &str) -> Option<&'a u64> {
    store
        .citizen_id_by_wallet_pubkey
        .get(wallet_pubkey)
        .filter(|cid| is_bound_citizen_record(store, **cid))
}

fn is_bound_citizen_record(store: &Store, citizen_id: u64) -> bool {
    store
        .citizen_records
        .get(&citizen_id)
        .map(|record| record.bind_status() == CitizenBindStatus::Bound)
        .unwrap_or(false)
}

fn cleanup_stale_citizen_record(store: &mut Store, citizen_id: u64) {
    let Some(record) = store.citizen_records.get(&citizen_id).cloned() else {
        return;
    };
    if record.bind_status() == CitizenBindStatus::Bound {
        return;
    }
    // 中文注释:开发期旧流程可能留下半绑定记录。新流程只保留完整绑定结果。
    store.citizen_records.remove(&citizen_id);
    if let Some(archive_no) = record.archive_no {
        store.citizen_id_by_archive_no.remove(&archive_no);
    }
    if let Some(wallet_pubkey) = record.wallet_pubkey {
        store.citizen_id_by_wallet_pubkey.remove(&wallet_pubkey);
    }
    if let Some(sfid_code) = record.sfid_code {
        store.citizen_id_by_sfid_code.remove(&sfid_code);
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
        "proto": crate::qr::WUMIN_QR_V1,
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
    store: &mut Store,
    province_name: &str,
    wallet_pubkey: &str,
) -> Result<String, axum::response::Response> {
    for retry in 0..1000u32 {
        let attempt_pubkey = if retry == 0 {
            wallet_pubkey.to_string()
        } else {
            format!("{}#{retry}", wallet_pubkey)
        };
        let candidate = match crate::sfid::generate_sfid_code(crate::sfid::GenerateSfidInput {
            account_pubkey: attempt_pubkey.as_str(),
            a3: "GMR",
            p1: "1",
            province: province_name,
            city: "省辖市",
            institution: "ZG",
        }) {
            Ok(v) => v,
            Err(msg) => return Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg)),
        };
        if !store.citizen_id_by_sfid_code.contains_key(&candidate) {
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

    #[test]
    fn replace_citizen_record_rejects_archive_change() {
        let mut store = Store::default();
        let now = Utc.timestamp_opt(1_000, 0).single().unwrap();
        store.citizen_records.insert(
            7,
            CitizenRecord {
                id: 7,
                wallet_pubkey: Some("0xold".to_string()),
                wallet_address: Some("old-address".to_string()),
                archive_no: Some("ARCHIVE-OLD".to_string()),
                sfid_code: Some("GMR-OLD".to_string()),
                citizen_status: Some(CitizenStatus::Normal),
                voting_eligible: true,
                archive_valid_from: Some("2026-05-24".to_string()),
                archive_valid_until: Some("2036-05-23".to_string()),
                status_updated_at: Some(1_000),
                sfid_signature: None,
                province_code: Some("GD".to_string()),
                city_code: Some("001".to_string()),
                bound_at: Some(now),
                bound_by: Some("admin-old".to_string()),
                created_at: now,
            },
        );
        store
            .citizen_id_by_archive_no
            .insert("ARCHIVE-OLD".to_string(), 7);
        store
            .citizen_id_by_wallet_pubkey
            .insert("0xold".to_string(), 7);
        store
            .citizen_id_by_sfid_code
            .insert("GMR-OLD".to_string(), 7);

        let ctx = AdminAuthContext {
            admin_pubkey: "admin-new".to_string(),
            role: AdminRole::ShengAdmin,
            admin_name: "测试管理员".to_string(),
            admin_province: Some("广东省".to_string()),
            admin_city: None,
            passkey_bound: false,
        };
        let challenge = CitizenBindChallenge {
            challenge_id: "challenge-replace".to_string(),
            challenge_text: "text".to_string(),
            mode: "replace".to_string(),
            citizen_id: Some(7),
            archive_no: "ARCHIVE-NEW".to_string(),
            wallet_address: "new-address".to_string(),
            wallet_pubkey: "0xnew".to_string(),
            wallet_sig_alg: "sr25519".to_string(),
            citizen_status: CitizenStatus::Normal,
            voting_eligible: true,
            archive_valid_from: "2026-05-24".to_string(),
            archive_valid_until: "2036-05-23".to_string(),
            status_updated_at: 1_001,
            province_code: "GD".to_string(),
            city_code: "001".to_string(),
            expire_at: now,
            created_at: now,
        };

        assert!(replace_citizen_record(&mut store, &ctx, &challenge).is_err());
        let record = store.citizen_records.get(&7).unwrap();
        assert_eq!(record.archive_no.as_deref(), Some("ARCHIVE-OLD"));
        assert_eq!(record.sfid_code.as_deref(), Some("GMR-OLD"));
        assert_eq!(record.wallet_pubkey.as_deref(), Some("0xold"));
    }
}
