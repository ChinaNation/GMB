use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::chain::runtime_align::build_bind_credential;
use crate::*;

// ── 公民身份绑定新接口 ────────────────────────────────────────────────

const BIND_CHALLENGE_TTL_SECONDS: i64 = 300; // 5 分钟

/// 生成绑定/解绑 challenge。
///
/// 返回 challenge_text 和 WUMIN_SIGN_V1.0.0 格式的签名请求 JSON，
/// 前端直接将 sign_request 展示为二维码供用户扫码签名。
pub(crate) async fn citizen_bind_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let challenge_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let challenge_text = format!(
        "sfid-citizen-bind-v1|{}|{}",
        challenge_id,
        now.timestamp()
    );
    let expire_at = now + chrono::Duration::seconds(BIND_CHALLENGE_TTL_SECONDS);

    // 构造 WUMIN_SIGN_V1.0.0 签名请求
    let sign_request = serde_json::json!({
        "proto": "WUMIN_SIGN_V1.0.0",
        "type": "sign_request",
        "request_id": format!("bind-{}", challenge_id),
        "account": "",
        "pubkey": "",
        "sig_alg": "sr25519",
        "payload_hex": format!("0x{}", hex::encode(challenge_text.as_bytes())),
        "issued_at": now.timestamp(),
        "expires_at": expire_at.timestamp(),
        "display": {
            "action": "citizen_bind",
            "action_label": "公民身份绑定",
            "summary": "确认将您的公钥绑定到公民身份记录",
            "fields": []
        }
    });
    let sign_request_str = serde_json::to_string(&sign_request).unwrap_or_default();

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    store.citizen_bind_challenges.retain(|_, c| c.expire_at > Utc::now());
    store.citizen_bind_challenges.insert(
        challenge_id.clone(),
        CitizenBindChallenge {
            challenge_id: challenge_id.clone(),
            challenge_text: challenge_text.clone(),
            account_pubkey: String::new(), // 签名时才确定
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
            sign_request: sign_request_str,
            expire_at: expire_at.timestamp(),
        },
    })
    .into_response()
}

/// 绑定公民身份。
///
/// 两种模式：
/// - `bind_archive`：有公钥的记录，扫 QR4 获取档案号，生成 SFID 码
/// - `bind_pubkey`：有档案号的记录（解绑后），绑定新公钥
pub(crate) async fn citizen_bind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CitizenBindInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.user_address.trim().is_empty()
        || input.challenge_id.trim().is_empty()
        || input.signature.trim().is_empty()
    {
        return api_error(StatusCode::BAD_REQUEST, 1001, "user_address, challenge_id, signature are required");
    }

    // 从 SS58 地址解出公钥
    let account_pubkey_hex = match ss58_to_pubkey_hex(input.user_address.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid SS58 user_address"),
    };

    // 验证 challenge
    let challenge = {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let Some(challenge) = store.citizen_bind_challenges.remove(input.challenge_id.trim()) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found or expired");
        };
        if Utc::now() > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "challenge expired");
        }
        challenge
    };

    // 验证公钥签名（WUMIN_SIGN_V1.0.0 签名结果）
    let pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&account_pubkey_hex) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid account_pubkey derived from address"),
    };
    let sig_bytes = match hex::decode(input.signature.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid signature hex"),
    };
    if !verify_citizen_bind_signature(&pubkey_bytes, &challenge.challenge_text, &sig_bytes) {
        return api_error(StatusCode::UNAUTHORIZED, 2004, "signature verify failed");
    }

    match input.mode.as_str() {
        "bind_archive" => {
            // 有公钥，绑定档案号 + 生成 SFID 码
            let qr4_str = match input.qr4_payload.as_deref() {
                Some(v) if !v.trim().is_empty() => v.trim(),
                _ => return api_error(StatusCode::BAD_REQUEST, 1001, "qr4_payload is required for bind_archive"),
            };

            // 解析 QR4
            let qr4: CpmsArchiveQrPayload = match serde_json::from_str(qr4_str) {
                Ok(v) => v,
                Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid QR4 payload"),
            };
            if qr4.qr_type != "CPMS_ARCHIVE_QR" {
                return api_error(StatusCode::BAD_REQUEST, 1001, "qr_type must be CPMS_ARCHIVE_QR");
            }

            // 验证 anon_cert + archive_sig（复用 archive_import 的逻辑）
            let sfid_sig_bytes = match hex::decode(qr4.anon_cert.sfid_sig.trim().trim_start_matches("0x")) {
                Ok(v) => v,
                Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "anon_cert.sfid_sig hex decode failed"),
            };
            let msg_randomizer = qr4.anon_cert.msg_randomizer.as_deref().and_then(|r| {
                hex::decode(r.trim().trim_start_matches("0x")).ok()
            });
            let cert_valid = match key_admins::rsa_blind::verify_anon_cert(
                &qr4.anon_cert.province_code,
                &qr4.anon_cert.anon_pubkey,
                &sfid_sig_bytes,
                msg_randomizer.as_deref(),
            ) {
                Ok(v) => v,
                Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("verify error: {e}")),
            };
            if !cert_valid {
                return api_error(StatusCode::UNAUTHORIZED, 2004, "anon_cert.sfid_sig invalid");
            }
            if qr4.anon_cert.province_code != qr4.province_code {
                return api_error(StatusCode::BAD_REQUEST, 1001, "province_code mismatch");
            }
            // 验 archive_sig
            let archive_sign_source = format!(
                "cpms-archive-qr-v1|{}|{}|{}|{}",
                qr4.province_code, qr4.archive_no, qr4.citizen_status, qr4.voting_eligible
            );
            let anon_pk = match crate::login::parse_sr25519_pubkey_bytes(&qr4.anon_cert.anon_pubkey) {
                Some(v) => v,
                None => return api_error(StatusCode::BAD_REQUEST, 1001, "anon_pubkey format invalid"),
            };
            let archive_sig = match hex::decode(qr4.archive_sig.trim().trim_start_matches("0x")) {
                Ok(v) => v,
                Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "archive_sig hex decode failed"),
            };
            if !crate::super_admins::institutions::verify_sr25519_signature(&anon_pk, &archive_sign_source, &archive_sig) {
                return api_error(StatusCode::UNAUTHORIZED, 2004, "archive_sig invalid");
            }

            let province_code = qr4.anon_cert.province_code.clone();
            let archive_no = qr4.archive_no.clone();

            // 生成 SFID 码
            let sfid_result = match crate::sfid::generate_sfid_code(crate::sfid::GenerateSfidInput {
                account_pubkey: account_pubkey_hex.as_str(),
                a3: "GMR",
                p1: "1",
                province: &crate::sfid::province::province_name_by_code(&province_code).unwrap_or(""),
                city: "省辖市",
                institution: "ZG",
            }) {
                Ok(v) => v,
                Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
            };

            // 写入
            let mut store = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            // 检查唯一性
            let existing_cid = store.citizen_id_by_pubkey.get(account_pubkey_hex.as_str()).copied();
            if let Some(cid) = existing_cid {
                // 已有记录，检查是否已绑定档案
                let already_bound = store.citizen_records.get(&cid).and_then(|r| r.archive_no.as_ref()).is_some();
                if already_bound {
                    return api_error(StatusCode::CONFLICT, 1005, "pubkey already bound to archive");
                }
                // 更新记录
                if let Some(record) = store.citizen_records.get_mut(&cid) {
                    record.archive_no = Some(archive_no.clone());
                    record.sfid_code = Some(sfid_result.clone());
                    record.province_code = Some(province_code.clone());
                    record.bound_at = Some(Utc::now());
                    record.bound_by = Some(ctx.admin_pubkey.clone());
                }
                store.citizen_id_by_archive_no.insert(archive_no, cid);
                let record = &store.citizen_records[&cid];
                let output = CitizenBindOutput {
                    id: cid,
                    account_pubkey: record.account_pubkey.clone(),
                    archive_no: record.archive_no.clone(),
                    sfid_code: record.sfid_code.clone(),
                    province_code: record.province_code.clone(),
                    status: record.status(),
                };
                drop(store);
                persist_runtime_state(&state);
                return Json(ApiResponse { code: 0, message: "ok".to_string(), data: output }).into_response();
            }
            if store.citizen_id_by_archive_no.contains_key(&archive_no) {
                return api_error(StatusCode::CONFLICT, 1005, "archive_no already bound");
            }
            let cid = store.next_citizen_id;
            store.next_citizen_id += 1;
            let record = CitizenRecord {
                id: cid,
                account_pubkey: Some(account_pubkey_hex.as_str().to_string()),
                archive_no: Some(archive_no.clone()),
                sfid_code: Some(sfid_result.clone()),
                sfid_signature: None,
                province_code: Some(province_code.clone()),
                bound_at: Some(Utc::now()),
                bound_by: Some(ctx.admin_pubkey.clone()),
                created_at: Utc::now(),
            };
            let output = CitizenBindOutput {
                id: cid,
                account_pubkey: record.account_pubkey.clone(),
                archive_no: record.archive_no.clone(),
                sfid_code: record.sfid_code.clone(),
                province_code: record.province_code.clone(),
                status: record.status(),
            };
            store.citizen_records.insert(cid, record);
            store.citizen_id_by_pubkey.insert(account_pubkey_hex.as_str().to_string(), cid);
            store.citizen_id_by_archive_no.insert(archive_no, cid);
            drop(store);
            persist_runtime_state(&state);
            Json(ApiResponse { code: 0, message: "ok".to_string(), data: output }).into_response()
        }
        "bind_pubkey" => {
            // 有档案号的记录，绑定新公钥
            let citizen_id = match input.citizen_id {
                Some(v) => v,
                None => return api_error(StatusCode::BAD_REQUEST, 1001, "citizen_id is required for bind_pubkey"),
            };
            let mut store = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            // 检查公钥是否已被占用
            if store.citizen_id_by_pubkey.contains_key(account_pubkey_hex.as_str()) {
                return api_error(StatusCode::CONFLICT, 1005, "pubkey already bound to another record");
            }
            let record = match store.citizen_records.get_mut(&citizen_id) {
                Some(v) => v,
                None => return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found"),
            };
            if record.account_pubkey.is_some() {
                return api_error(StatusCode::CONFLICT, 1005, "record already has a pubkey, unbind first");
            }
            record.account_pubkey = Some(account_pubkey_hex.as_str().to_string());
            record.bound_at = Some(Utc::now());
            record.bound_by = Some(ctx.admin_pubkey.clone());
            let output = CitizenBindOutput {
                id: citizen_id,
                account_pubkey: record.account_pubkey.clone(),
                archive_no: record.archive_no.clone(),
                sfid_code: record.sfid_code.clone(),
                province_code: record.province_code.clone(),
                status: record.status(),
            };
            store.citizen_id_by_pubkey.insert(account_pubkey_hex.as_str().to_string(), citizen_id);
            drop(store);
            persist_runtime_state(&state);
            Json(ApiResponse { code: 0, message: "ok".to_string(), data: output }).into_response()
        }
        _ => api_error(StatusCode::BAD_REQUEST, 1001, "mode must be bind_archive or bind_pubkey"),
    }
}

/// 解绑：清除公钥，保留档案号+SFID码。需要公钥持有者签名确认。
pub(crate) async fn citizen_unbind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CitizenUnbindInput>,
) -> impl IntoResponse {
    let _ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.challenge_id.trim().is_empty() || input.signature.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "challenge_id and signature are required");
    }

    // 验证 challenge
    let challenge = {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let Some(challenge) = store.citizen_bind_challenges.remove(input.challenge_id.trim()) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found or expired");
        };
        if Utc::now() > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "challenge expired");
        }
        challenge
    };

    // 获取已绑定的公钥并验签
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if !store.citizen_records.contains_key(&input.citizen_id) {
        return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found");
    }
    let old_pubkey = store.citizen_records.get(&input.citizen_id)
        .and_then(|r| r.account_pubkey.clone());
    let old_pubkey = match old_pubkey {
        Some(pk) if !pk.is_empty() => pk,
        _ => return api_error(StatusCode::CONFLICT, 1005, "record has no pubkey to unbind"),
    };

    // 验证公钥签名
    let pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&old_pubkey) {
        Some(v) => v,
        None => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "stored pubkey format invalid"),
    };
    let sig_bytes = match hex::decode(input.signature.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid signature hex"),
    };
    if !verify_citizen_bind_signature(&pubkey_bytes, &challenge.challenge_text, &sig_bytes) {
        return api_error(StatusCode::UNAUTHORIZED, 2004, "unbind signature verify failed");
    }

    // 清除公钥
    store.citizen_id_by_pubkey.remove(&old_pubkey);
    if let Some(record) = store.citizen_records.get_mut(&input.citizen_id) {
        record.account_pubkey = None;
    }
    let record = &store.citizen_records[&input.citizen_id];
    let output = CitizenBindOutput {
        id: input.citizen_id,
        account_pubkey: record.account_pubkey.clone(),
        archive_no: record.archive_no.clone(),
        sfid_code: record.sfid_code.clone(),
        province_code: record.province_code.clone(),
        status: record.status(),
    };
    drop(store);
    persist_runtime_state(&state);
    Json(ApiResponse { code: 0, message: "ok".to_string(), data: output }).into_response()
}

/// 验证公民绑定签名（sr25519，substrate context）。
fn verify_citizen_bind_signature(pubkey_bytes: &[u8; 32], message: &str, signature: &[u8]) -> bool {
    use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
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

/// 从 SS58 地址解出 hex 格式公钥。
fn ss58_to_pubkey_hex(address: &str) -> Option<String> {
    let decoded = bs58::decode(address.trim()).into_vec().ok()?;
    // SS58 格式：1 byte prefix + 32 bytes pubkey + 2 bytes checksum (最少 35 字节)
    if decoded.len() < 35 {
        return None;
    }
    let pubkey = &decoded[1..33];
    Some(format!("0x{}", hex::encode(pubkey)))
}
