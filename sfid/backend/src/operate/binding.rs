use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::*;

// ── 公民身份绑定新接口 ────────────────────────────────────────────────

const BIND_CHALLENGE_TTL_SECONDS: i64 = 300; // 5 分钟

/// 生成绑定/解绑 challenge。
///
/// 返回 challenge_text 和 WUMIN_QR_V1 格式的签名请求 JSON，
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
    let challenge_text = format!("sfid-citizen-bind-v1|{}|{}", challenge_id, now.timestamp());
    let expire_at = now + chrono::Duration::seconds(BIND_CHALLENGE_TTL_SECONDS);

    // 构造 WUMIN_QR_V1 kind=sign_request envelope
    let sign_request = serde_json::json!({
        "proto": crate::qr::WUMIN_QR_V1,
        "kind": "sign_request",
        "id": format!("bind-{}", challenge_id),
        "issued_at": now.timestamp(),
        "expires_at": expire_at.timestamp(),
        "body": {
            "address": "",
            "pubkey": "",
            "sig_alg": "sr25519",
            "payload_hex": format!("0x{}", hex::encode(challenge_text.as_bytes())),
            "spec_version": 0,
            "display": {
                "action": "citizen_bind",
                "summary": "确认将您的公钥绑定到公民身份记录",
                "fields": []
            }
        }
    });
    let sign_request_str = serde_json::to_string(&sign_request).unwrap_or_default();

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
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "user_address, challenge_id, signature are required",
        );
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
        let Some(challenge) = store
            .citizen_bind_challenges
            .remove(input.challenge_id.trim())
        else {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "challenge not found or expired",
            );
        };
        if Utc::now() > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "challenge expired");
        }
        challenge
    };

    // 验证公钥签名（WUMIN_QR_V1 签名结果）
    let pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&account_pubkey_hex) {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "invalid account_pubkey derived from address",
            )
        }
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
                _ => {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "qr4_payload is required for bind_archive",
                    )
                }
            };

            // 解析 QR4
            let qr4: CpmsArchiveQrPayload = match serde_json::from_str(qr4_str) {
                Ok(v) => v,
                Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid QR4 payload"),
            };
            if qr4.r#type != "ARCHIVE" {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "qr_type must be CPMS_ARCHIVE",
                );
            }

            // 验证 anon_cert + archive_sig（复用 archive_import 的逻辑）
            let sfid_sig_bytes = match hex::decode(qr4.cert.sig.trim().trim_start_matches("0x")) {
                Ok(v) => v,
                Err(_) => {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "anon_cert.sfid_sig hex decode failed",
                    )
                }
            };
            let msg_randomizer = qr4
                .cert
                .mr
                .as_deref()
                .and_then(|r| hex::decode(r.trim().trim_start_matches("0x")).ok());
            let cert_valid = match crate::institutions::anon_cert::rsa_blind::verify_anon_cert(
                &qr4.cert.prov,
                &qr4.cert.pk,
                &sfid_sig_bytes,
                msg_randomizer.as_deref(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    return api_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        1004,
                        &format!("verify error: {e}"),
                    )
                }
            };
            if !cert_valid {
                return api_error(StatusCode::UNAUTHORIZED, 2004, "anon_cert.sfid_sig invalid");
            }
            if qr4.cert.prov != qr4.prov {
                return api_error(StatusCode::BAD_REQUEST, 1001, "province_code mismatch");
            }
            // 验 archive_sig
            let archive_sign_source = format!(
                "sfid-cpms-v1|archive|{}|{}|{}|{}",
                qr4.prov, qr4.ano, qr4.cs, qr4.ve
            );
            let anon_pk = match crate::login::parse_sr25519_pubkey_bytes(&qr4.cert.pk) {
                Some(v) => v,
                None => {
                    return api_error(StatusCode::BAD_REQUEST, 1001, "anon_pubkey format invalid")
                }
            };
            let archive_sig = match hex::decode(qr4.sig.trim().trim_start_matches("0x")) {
                Ok(v) => v,
                Err(_) => {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "archive_sig hex decode failed",
                    )
                }
            };
            if !crate::sheng_admins::institutions::verify_sr25519_signature(
                &anon_pk,
                &archive_sign_source,
                &archive_sig,
            ) {
                return api_error(StatusCode::UNAUTHORIZED, 2004, "archive_sig invalid");
            }

            let province_code = qr4.cert.prov.clone();
            let archive_no = qr4.ano.clone();

            // 生成 SFID 码
            let sfid_result =
                match crate::sfid::generate_sfid_code(crate::sfid::GenerateSfidInput {
                    account_pubkey: account_pubkey_hex.as_str(),
                    a3: "GMR",
                    p1: "1",
                    province: &crate::sfid::province::province_name_by_code(&province_code)
                        .unwrap_or(""),
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
            let existing_cid = store
                .citizen_id_by_pubkey
                .get(account_pubkey_hex.as_str())
                .copied();
            if let Some(cid) = existing_cid {
                // 已有记录，检查是否已绑定档案
                let already_bound = store
                    .citizen_records
                    .get(&cid)
                    .and_then(|r| r.archive_no.as_ref())
                    .is_some();
                if already_bound {
                    return api_error(
                        StatusCode::CONFLICT,
                        1005,
                        "pubkey already bound to archive",
                    );
                }
                // 更新记录
                if let Some(record) = store.citizen_records.get_mut(&cid) {
                    record.archive_no = Some(archive_no.clone());
                    record.sfid_code = Some(sfid_result.clone());
                    record.province_code = Some(province_code.clone());
                    record.chain_confirmed = false;
                    record.bound_at = Some(Utc::now());
                    record.bound_by = Some(ctx.admin_pubkey.clone());
                }
                store.citizen_id_by_archive_no.insert(archive_no, cid);
                store
                    .citizen_id_by_sfid_code
                    .insert(sfid_result.clone(), cid);
                let record = &store.citizen_records[&cid];
                let output = CitizenBindOutput {
                    id: cid,
                    account_pubkey: record.account_pubkey.clone(),
                    account_address: record.account_address.clone(),
                    archive_no: record.archive_no.clone(),
                    sfid_code: record.sfid_code.clone(),
                    province_code: record.province_code.clone(),
                    status: record.status(),
                };
                drop(store);
                return Json(ApiResponse {
                    code: 0,
                    message: "ok".to_string(),
                    data: output,
                })
                .into_response();
            }
            if store.citizen_id_by_archive_no.contains_key(&archive_no) {
                return api_error(StatusCode::CONFLICT, 1005, "archive_no already bound");
            }
            let cid = store.next_citizen_id;
            store.next_citizen_id += 1;
            let account_address = pubkey_hex_to_ss58(&account_pubkey_hex);
            let record = CitizenRecord {
                id: cid,
                account_pubkey: Some(account_pubkey_hex.as_str().to_string()),
                account_address: account_address.clone(),
                archive_no: Some(archive_no.clone()),
                sfid_code: Some(sfid_result.clone()),
                sfid_signature: None,
                province_code: Some(province_code.clone()),
                chain_confirmed: false,
                bound_at: Some(Utc::now()),
                bound_by: Some(ctx.admin_pubkey.clone()),
                created_at: Utc::now(),
            };
            let output = CitizenBindOutput {
                id: cid,
                account_pubkey: record.account_pubkey.clone(),
                account_address: record.account_address.clone(),
                archive_no: record.archive_no.clone(),
                sfid_code: record.sfid_code.clone(),
                province_code: record.province_code.clone(),
                status: record.status(),
            };
            store.citizen_records.insert(cid, record);
            store
                .citizen_id_by_pubkey
                .insert(account_pubkey_hex.as_str().to_string(), cid);
            store
                .citizen_id_by_archive_no
                .insert(archive_no.clone(), cid);
            store.citizen_id_by_sfid_code.insert(sfid_result, cid);
            drop(store);
            Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: output,
            })
            .into_response()
        }
        "bind_pubkey" => {
            // 旧档案绑新账户：
            // 1. 用 user_address 在系统中查找 PENDING 状态的记录(用户已通过 wuminapp 注册)
            // 2. 将 UNLINKED 记录的档案号+SFID码与该账户关联
            // 3. 删除原 PENDING 记录
            let citizen_id = match input.citizen_id {
                Some(v) => v,
                None => {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "citizen_id is required for bind_pubkey",
                    )
                }
            };
            let mut store = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            // 比对：user_address 对应的 pubkey 必须已在系统中注册(PENDING 状态)
            let source_citizen_id =
                match store.citizen_id_by_pubkey.get(account_pubkey_hex.as_str()) {
                    Some(id) => *id,
                    None => {
                        return api_error(
                            StatusCode::NOT_FOUND,
                            1004,
                            "该账户未在系统中注册，请先在公民钱包中设置投票账户",
                        )
                    }
                };
            // 校验 source 记录是 PENDING 状态(只有地址无档案)
            {
                let source = match store.citizen_records.get(&source_citizen_id) {
                    Some(v) => v,
                    None => {
                        return api_error(
                            StatusCode::NOT_FOUND,
                            1004,
                            "source citizen record not found",
                        )
                    }
                };
                if source.archive_no.is_some() {
                    return api_error(StatusCode::CONFLICT, 1005, "该账户已绑定档案，不能重复绑定");
                }
            }
            // 校验 target 记录是 UNLINKED 状态(有档案无账户)
            let target = match store.citizen_records.get(&citizen_id) {
                Some(v) => v,
                None => return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found"),
            };
            if target.account_pubkey.is_some() {
                return api_error(StatusCode::CONFLICT, 1005, "该记录已有账户，请先解绑");
            }
            if target.archive_no.is_none() {
                return api_error(StatusCode::BAD_REQUEST, 1001, "该记录没有档案号，无法绑定");
            }
            // 将 target 记录关联账户
            let target = store.citizen_records.get_mut(&citizen_id).unwrap();
            target.account_pubkey = Some(account_pubkey_hex.clone());
            target.account_address = pubkey_hex_to_ss58(&account_pubkey_hex);
            target.chain_confirmed = false;
            target.bound_at = Some(Utc::now());
            target.bound_by = Some(ctx.admin_pubkey.clone());
            let output = CitizenBindOutput {
                id: citizen_id,
                account_pubkey: target.account_pubkey.clone(),
                account_address: target.account_address.clone(),
                archive_no: target.archive_no.clone(),
                sfid_code: target.sfid_code.clone(),
                province_code: target.province_code.clone(),
                status: target.status(),
            };
            // 删除原 PENDING 记录,更新索引指向 target
            store.citizen_records.remove(&source_citizen_id);
            store
                .citizen_id_by_pubkey
                .insert(account_pubkey_hex, citizen_id);
            drop(store);
            Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: output,
            })
            .into_response()
        }
        _ => api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "mode must be bind_archive or bind_pubkey",
        ),
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
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id and signature are required",
        );
    }

    // 验证 challenge
    let challenge = {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let Some(challenge) = store
            .citizen_bind_challenges
            .remove(input.challenge_id.trim())
        else {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "challenge not found or expired",
            );
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
    let old_pubkey = store
        .citizen_records
        .get(&input.citizen_id)
        .and_then(|r| r.account_pubkey.clone());
    let old_pubkey = match old_pubkey {
        Some(pk) if !pk.is_empty() => pk,
        _ => return api_error(StatusCode::CONFLICT, 1005, "record has no pubkey to unbind"),
    };

    // 验证公钥签名
    let pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&old_pubkey) {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "stored pubkey format invalid",
            )
        }
    };
    let sig_bytes = match hex::decode(input.signature.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid signature hex"),
    };
    if !verify_citizen_bind_signature(&pubkey_bytes, &challenge.challenge_text, &sig_bytes) {
        return api_error(
            StatusCode::UNAUTHORIZED,
            2004,
            "unbind signature verify failed",
        );
    }

    // 清除公钥和链上确认状态（保留 archive_no + sfid_code）
    store.citizen_id_by_pubkey.remove(&old_pubkey);
    if let Some(record) = store.citizen_records.get_mut(&input.citizen_id) {
        record.account_pubkey = None;
        record.account_address = None;
        record.chain_confirmed = false;
    }
    let record = &store.citizen_records[&input.citizen_id];
    let output = CitizenBindOutput {
        id: input.citizen_id,
        account_pubkey: record.account_pubkey.clone(),
        account_address: record.account_address.clone(),
        archive_no: record.archive_no.clone(),
        sfid_code: record.sfid_code.clone(),
        province_code: record.province_code.clone(),
        status: record.status(),
    };
    drop(store);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

/// 验证公民绑定签名（sr25519，substrate context）。
fn verify_citizen_bind_signature(pubkey_bytes: &[u8; 32], message: &str, signature: &[u8]) -> bool {
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

/// 从 SS58 地址解出 hex 格式公钥。
fn ss58_to_pubkey_hex(address: &str) -> Option<String> {
    let decoded = bs58::decode(address.trim()).into_vec().ok()?;
    // SS58 prefix < 64 → 1 字节前缀；prefix >= 64 → 2 字节前缀
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

/// 0x hex 公钥 → SS58 地址（prefix=2027）。
fn pubkey_hex_to_ss58(pubkey_hex: &str) -> Option<String> {
    let pubkey_bytes = hex::decode(pubkey_hex.trim_start_matches("0x")).ok()?;
    if pubkey_bytes.len() != 32 {
        return None;
    }
    // SS58 prefix 2027: 编码为 2 bytes (0x40 | (2027>>2), (2027&3)<<6 | checksum_prefix)
    // 参考 ss58-registry：prefix >= 64 使用 2 字节编码
    // Simple Account format: [prefix_bytes...] ++ pubkey ++ blake2(prefix ++ pubkey)[..2]
    use blake2::{digest::VariableOutput, Blake2bVar};
    let prefix: u16 = 2027;
    let first = ((prefix & 0b0000_0000_1111_1100) as u8) >> 2 | 0b01000000;
    let second = (prefix >> 8) as u8 | ((prefix & 0b0000_0000_0000_0011) as u8) << 6;
    let mut payload = vec![first, second];
    payload.extend_from_slice(&pubkey_bytes);
    // checksum
    let mut hasher = Blake2bVar::new(64).ok()?;
    use blake2::digest::Update;
    hasher.update(b"SS58PRE");
    hasher.update(&payload);
    let mut hash = vec![0u8; 64];
    hasher.finalize_variable(&mut hash).ok()?;
    payload.extend_from_slice(&hash[..2]);
    Some(bs58::encode(payload).into_string())
}

// ── wuminapp 投票账户接口 ──────────────────────────────────────

/// wuminapp 推送投票账户（公共接口，无 admin 认证）。
///
/// 用户在 wuminapp 选择钱包后，签名证明私钥所有权，推送 pubkey 到 SFID。
pub(crate) async fn app_vote_account_register(
    State(state): State<AppState>,
    Json(input): Json<VoteAccountRegisterInput>,
) -> impl IntoResponse {
    if input.address.trim().is_empty()
        || input.pubkey.trim().is_empty()
        || input.signature.trim().is_empty()
        || input.sign_message.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "address, pubkey, signature, sign_message are required",
        );
    }

    // 验证 SS58 地址与 pubkey 一致
    let derived_pubkey = match ss58_to_pubkey_hex(input.address.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid SS58 address"),
    };
    let input_pubkey = input.pubkey.trim().to_lowercase();
    if derived_pubkey.to_lowercase() != input_pubkey {
        return api_error(StatusCode::BAD_REQUEST, 1001, "address and pubkey mismatch");
    }

    // 验证 sign_message 格式：CITIZEN_VOTE_REGISTER|{address}|{timestamp}
    let parts: Vec<&str> = input.sign_message.trim().split('|').collect();
    if parts.len() != 3 || parts[0] != "CITIZEN_VOTE_REGISTER" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "invalid sign_message format");
    }
    if parts[1] != input.address.trim() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "sign_message address mismatch",
        );
    }
    let timestamp: i64 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "invalid timestamp in sign_message",
            )
        }
    };
    let now = Utc::now().timestamp();
    if (now - timestamp).abs() > 300 {
        return api_error(
            StatusCode::UNAUTHORIZED,
            1007,
            "sign_message expired (>5 min)",
        );
    }

    // sr25519 验签
    let pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&input_pubkey) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid pubkey format"),
    };
    let sig_bytes = match hex::decode(input.signature.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid signature hex"),
    };
    if !verify_citizen_bind_signature(&pubkey_bytes, input.sign_message.trim(), &sig_bytes) {
        return api_error(StatusCode::UNAUTHORIZED, 2004, "signature verify failed");
    }

    // 检查是否已存在
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store.citizen_id_by_pubkey.contains_key(&input_pubkey) {
        // 幂等：已存在直接返回成功
        drop(store);
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: serde_json::json!({}),
        })
        .into_response();
    }

    // 创建新记录（只有 pubkey，状态 Pending）
    let cid = store.next_citizen_id;
    store.next_citizen_id += 1;
    let account_address = pubkey_hex_to_ss58(&input_pubkey);
    let record = CitizenRecord {
        id: cid,
        account_pubkey: Some(input_pubkey.clone()),
        account_address,
        archive_no: None,
        sfid_code: None,
        sfid_signature: None,
        province_code: None,
        chain_confirmed: false,
        bound_at: None,
        bound_by: None,
        created_at: Utc::now(),
    };
    store.citizen_records.insert(cid, record);
    store.citizen_id_by_pubkey.insert(input_pubkey, cid);
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: serde_json::json!({}),
    })
    .into_response()
}

/// wuminapp 查询投票账户绑定状态（公共接口）。
pub(crate) async fn app_vote_account_status(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<VoteAccountStatusQuery>,
) -> impl IntoResponse {
    if params.address.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "address is required");
    }
    let pubkey_hex = match ss58_to_pubkey_hex(params.address.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid SS58 address"),
    };

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let result = match store
        .citizen_id_by_pubkey
        .get(&pubkey_hex)
        .and_then(|cid| store.citizen_records.get(cid))
    {
        Some(record) => {
            let status_str = match record.status() {
                CitizenBindStatus::Bound => "bound",
                CitizenBindStatus::Pending | CitizenBindStatus::Bindable => "pending",
                CitizenBindStatus::Unlinked => "unset",
            };
            VoteAccountStatusOutput {
                status: status_str.to_string(),
                address: record.account_address.clone(),
                sfid_code: record.sfid_code.clone(),
            }
        }
        None => VoteAccountStatusOutput {
            status: "unset".to_string(),
            address: None,
            sfid_code: None,
        },
    };
    drop(store);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: result,
    })
    .into_response()
}

// ── 管理员推链接口 ──────────────────────────────────────

/// 管理员推链绑定：构造 bind_sfid extrinsic 提交区块链。
pub(crate) async fn citizen_push_chain_bind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CitizenPushChainInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // 读取记录，确认状态为 Bindable
    let (account_pubkey, archive_no) = {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let record = match store.citizen_records.get(&input.citizen_id) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found"),
        };
        if record.status() != CitizenBindStatus::Bindable {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "record is not in Bindable state",
            );
        }
        (
            record.account_pubkey.clone().unwrap(),
            record.archive_no.clone().unwrap(),
        )
    };

    // 获取省级签名密钥
    let (province_pair, _province) =
        match crate::key_admins::signer_router::resolve_business_signer(&state, &ctx) {
            Ok(v) => v,
            Err((status, msg)) => return api_error(status, 5001, &msg),
        };

    // 构建链上凭证
    let bind_nonce = Uuid::new_v4().to_string();
    let credential = match crate::chain::runtime_align::build_bind_credential_with_province(
        &state,
        &account_pubkey,
        &archive_no,
        bind_nonce,
        &province_pair,
    ) {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                &format!("build credential failed: {e}"),
            )
        }
    };

    // 提交 bind_sfid extrinsic(PoW 链三件套,实现在 chain/citizen_binding/push.rs)
    let tx_hash = match crate::chain::citizen_binding::submit_bind_sfid_extrinsic(
        &credential,
        &province_pair,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5003,
                &format!("push chain failed: {e}"),
            )
        }
    };

    // 更新本地状态
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Some(record) = store.citizen_records.get_mut(&input.citizen_id) {
        record.chain_confirmed = true;
    }
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CitizenPushChainOutput { tx_hash },
    })
    .into_response()
}

/// 管理员推链解绑：构造 unbind_sfid(target) extrinsic 提交区块链。
pub(crate) async fn citizen_push_chain_unbind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CitizenPushChainInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // 读取记录，确认状态为 Bound
    let account_pubkey = {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let record = match store.citizen_records.get(&input.citizen_id) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "citizen record not found"),
        };
        if record.status() != CitizenBindStatus::Bound {
            return api_error(StatusCode::CONFLICT, 1005, "record is not in Bound state");
        }
        record.account_pubkey.clone().unwrap()
    };

    // 获取省级签名密钥
    let (province_pair, _province) =
        match crate::key_admins::signer_router::resolve_business_signer(&state, &ctx) {
            Ok(v) => v,
            Err((status, msg)) => return api_error(status, 5001, &msg),
        };

    // 提交 unbind_sfid(target) extrinsic(实现在 chain/citizen_binding/push.rs)
    let tx_hash = match crate::chain::citizen_binding::submit_unbind_sfid_extrinsic(
        &account_pubkey,
        &province_pair,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5003,
                &format!("push chain unbind failed: {e}"),
            )
        }
    };

    // 更新本地状态：清除公钥，保留 archive_no + sfid_code
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    store.citizen_id_by_pubkey.remove(&account_pubkey);
    if let Some(record) = store.citizen_records.get_mut(&input.citizen_id) {
        record.account_pubkey = None;
        record.account_address = None;
        record.chain_confirmed = false;
    }
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CitizenPushChainOutput { tx_hash },
    })
    .into_response()
}

// 中文注释:历史 submit_bind_sfid_extrinsic / submit_unbind_sfid_extrinsic 已搬到
// chain/citizen_binding/push.rs;调用入口走 `crate::chain::citizen_binding::*`。
