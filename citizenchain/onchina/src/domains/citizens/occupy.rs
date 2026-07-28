//! 公民 CID 占号两阶段流程(ADR-031 D6/D7)。
//!
//! prepare = 校验建档输入 → 发号(种子 + nonce 碰撞重试,本地/链上双预查,
//!           链上同承诺幂等续用)→ 构造 `occupy_cid` 冷签载荷 → 会话落库 → 返回 QR;
//! submit  = 管理员扫码回签 → 组装/dry-run/提交/等进块 → 档案落库(占号先行:
//!           链上成功才建档)。
//! 吊销(purpose=CITIZEN_REVOKE)与链上身份推送(purpose=CITIZEN_IDENTITY_PUSH)
//! 复用同一 submit 入口，按会话 purpose 分派落库动作。

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use codec::{Compact, Encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::actions::require_admin_security_grant;
use crate::auth::operation_auth::AdminActionType;
use crate::core::chain_submit;
use crate::auth::login::parse_account_id_bytes;
use crate::crypto::pubkey::{normalize_account_id, same_account_id};
use sp_core::{sr25519, Pair};
use crate::domains::citizens::admin_entry::{
    cid_seed, create_output_from_record, generate_citizen_cid_candidate,
    persist_citizen_record, validate_citizen_input, AdminCreateCitizenInput,
    AdminCreateCitizenOutput, ValidatedCitizenInput,
};
use crate::domains::citizens::chain_identity::{
    active_registry_cid_number, ensure_registry_admin, validate_actor_role_code,
};
use crate::*;

const CITIZEN_IDENTITY_PALLET_INDEX: u8 = 10;
const OCCUPY_CID_CALL_INDEX: u8 = 6;
const REVOKE_CID_CALL_INDEX: u8 = 8;
/// 发号碰撞重试上限(对齐 n9 桶 1000 次重试死规则)。
const CID_GENERATE_MAX_RETRY: u32 = 1000;
/// 冷签会话有效期(秒)。
pub(crate) const SESSION_TTL_SECS: i64 = 600;

pub(crate) const PURPOSE_CITIZEN_OCCUPY: &str = "CITIZEN_OCCUPY";
/// 占号 pending:发号后、用户占号签名收集前的占位会话(call_data 尚未构建)。
pub(crate) const PURPOSE_CITIZEN_OCCUPY_PENDING: &str = "CITIZEN_OCCUPY_PENDING";
pub(crate) const PURPOSE_CITIZEN_REVOKE: &str = "CITIZEN_REVOKE";
pub(crate) const PURPOSE_CITIZEN_IDENTITY_PUSH: &str = "CITIZEN_IDENTITY_PUSH";

/// 链冷签会话:prepare 只保存短期签名 payload。
///
/// 这不是公民或机构的业务草稿状态。submit 成功或失败后都必须删除;
/// 公民/机构正式数据只能在链上确认成功后写入正式投影表。
pub(crate) struct ChainSignSession {
    pub(crate) request_id: String,
    pub(crate) purpose: String,
    /// 发起管理员账户对应的当前签名公钥（签名者必须与之一致）。
    pub(crate) account_id: String,
    pub(crate) call_data: Vec<u8>,
    pub(crate) nonce: u32,
    /// sha256(签名输入) hex,submit 阶段重建校验防 runtime 漂移。
    pub(crate) signing_hash: String,
    pub(crate) context: serde_json::Value,
    pub(crate) expires_at: DateTime<Utc>,
    pub(crate) consumed_at: Option<DateTime<Utc>>,
}

impl Db {
    pub(crate) fn insert_chain_sign_session(&self, s: &ChainSignSession) -> Result<(), String> {
        let s = ChainSignSession {
            request_id: s.request_id.clone(),
            purpose: s.purpose.clone(),
            account_id: s.account_id.clone(),
            call_data: s.call_data.clone(),
            nonce: s.nonce,
            signing_hash: s.signing_hash.clone(),
            context: s.context.clone(),
            expires_at: s.expires_at,
            consumed_at: s.consumed_at,
        };
        self.with_client(move |conn| {
            conn.execute(
                "INSERT INTO chain_sign_sessions
                    (request_id, purpose, account_id, call_data, nonce, signing_hash,
                     context, expires_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &s.request_id,
                    &s.purpose,
                    &s.account_id,
                    &hex::encode(&s.call_data),
                    &(s.nonce as i64),
                    &s.signing_hash,
                    &s.context,
                    &s.expires_at,
                ],
            )
            .map_err(|e| format!("insert chain sign session failed: {e}"))?;
            Ok(())
        })
    }

    pub(crate) fn find_chain_sign_session(
        &self,
        request_id: &str,
    ) -> Result<Option<ChainSignSession>, String> {
        let request_id = request_id.trim().to_string();
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT request_id, purpose, account_id, call_data, nonce, signing_hash,
                            context, expires_at, consumed_at
                     FROM chain_sign_sessions WHERE request_id = $1",
                    &[&request_id],
                )
                .map_err(|e| format!("query chain sign session failed: {e}"))?;
            Ok(row.map(|r| ChainSignSession {
                request_id: r.get(0),
                purpose: r.get(1),
                account_id: r.get(2),
                call_data: hex::decode(r.get::<_, String>(3)).unwrap_or_default(),
                nonce: r.get::<_, i64>(4) as u32,
                signing_hash: r.get(5),
                context: r.get(6),
                expires_at: r.get(7),
                consumed_at: r.get(8),
            }))
        })
    }

    pub(crate) fn delete_chain_sign_session(&self, request_id: &str) -> Result<(), String> {
        let request_id = request_id.trim().to_string();
        self.with_client(move |conn| {
            conn.execute(
                "DELETE FROM chain_sign_sessions WHERE request_id = $1",
                &[&request_id],
            )
            .map_err(|e| format!("delete chain sign session failed: {e}"))?;
            Ok(())
        })
    }

    /// 把占号 pending 会话(用户签名收集前的占位)升级为可提交的冷签会话:
    /// 用户签名回来后回填 call_data/nonce/signing_hash + 转正 purpose + 追加 account_id。
    pub(crate) fn promote_chain_sign_session(
        &self,
        request_id: &str,
        purpose: &str,
        call_data: &[u8],
        nonce: u32,
        signing_hash: &str,
        context: &serde_json::Value,
    ) -> Result<u64, String> {
        let request_id = request_id.trim().to_string();
        let purpose = purpose.to_string();
        let call_data = hex::encode(call_data);
        let signing_hash = signing_hash.to_string();
        let context = context.clone();
        self.with_client(move |conn| {
            conn.execute(
                "UPDATE chain_sign_sessions
                 SET purpose = $2, call_data = $3, nonce = $4, signing_hash = $5, context = $6
                 WHERE request_id = $1 AND consumed_at IS NULL",
                &[
                    &request_id,
                    &purpose,
                    &call_data,
                    &(nonce as i64),
                    &signing_hash,
                    &context,
                ],
            )
            .map_err(|e| format!("promote chain sign session failed: {e}"))
        })
    }

    /// 吊销落库:本地档案状态置 REVOKED(墓碑语义,档案保留)。
    pub(crate) fn mark_citizen_revoked(
        &self,
        cid_number: &str,
        account_id: &str,
        onchain_tx_hash: &str,
    ) -> Result<u64, String> {
        let cid_number = cid_number.to_string();
        let account_id = account_id.to_string();
        let onchain_tx_hash = onchain_tx_hash.to_string();
        self.with_client(move |conn| {
            conn.execute(
                "UPDATE citizens
                 SET citizen_status = 'REVOKED', status_updated_at = extract(epoch from now())::bigint,
                     onchain_tx_hash = $2, onchain_at = now(), updater_account_id = $3, updated_at = now()
                 WHERE cid_number = $1",
                &[&cid_number, &onchain_tx_hash, &account_id],
            )
            .map_err(|e| format!("mark citizen revoked failed: {e}"))
        })
    }

    /// 链上身份推送成功回写(D8:提交路径同步回写,精确到交易哈希与块高)。
    ///
    /// 出生日期 `citizen_birth_date` 是新增公民时必填、写入后不可修改的字段,
    /// 任何编辑/回写路径都不得进入其 SET 子句(与链端 `BirthDateImmutable` 对齐)。
    pub(crate) fn confirm_citizen_identity_onchain(
        &self,
        cid_number: &str,
        citizen_account_id: &str,
        registrar_account_id: &str,
        onchain_tx_hash: &str,
        onchain_block_number: Option<u64>,
    ) -> Result<u64, String> {
        let cid_number = cid_number.to_string();
        let citizen_account_id = citizen_account_id.to_string();
        let registrar_account_id = registrar_account_id.to_string();
        let onchain_tx_hash = onchain_tx_hash.to_string();
        let block = onchain_block_number.map(|n| n as i64);
        self.with_client(move |conn| {
            conn.execute(
                "UPDATE citizens
                 SET account_id = $2, account_verified_at = now(), onchain_tx_hash = $3,
                     onchain_block_number = $4, onchain_at = now(),
                     updater_account_id = $5, updated_at = now()
                 WHERE cid_number = $1",
                &[
                    &cid_number,
                    &citizen_account_id,
                    &onchain_tx_hash,
                    &block,
                    &registrar_account_id,
                ],
            )
            .map_err(|e| format!("confirm citizen identity onchain failed: {e}"))
        })
    }
}

// ──── SCALE 调用编码(citizen-identity pallet)────

fn append_bounded(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend(Compact(bytes.len() as u32).encode());
    out.extend_from_slice(bytes);
}

/// occupy_cid(actor_cid_number, actor_role_code, cid_number, account_id, citizen_signature)
///
/// 占即绑:居住地/承诺哈希不再是 call 参数(commitment 链上算 blake2_256(account_id),
/// 居住地去地域)。`account_id` = AccountId32,32 裸字节(无长度前缀);
/// `occupy_signature` = 用户对 (cid_number, account_id) 的签名,BoundedVec(Compact(len)+bytes)。
fn encode_occupy_cid_call(
    actor_cid_number: &str,
    actor_role_code: &str,
    cid_number: &str,
    account_id: &[u8; 32],
    occupy_signature: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(CITIZEN_IDENTITY_PALLET_INDEX);
    out.push(OCCUPY_CID_CALL_INDEX);
    append_bounded(&mut out, actor_cid_number.as_bytes());
    append_bounded(&mut out, actor_role_code.as_bytes());
    append_bounded(&mut out, cid_number.as_bytes());
    out.extend_from_slice(account_id);
    append_bounded(&mut out, occupy_signature);
    out
}

/// 验用户占号签名:sr25519 over `signing_message(OP_SIGN_CID_OCCUPY, (cid_number, account_id))`。
/// payload 与链端 `(cid_number, account_id).encode()` 字节一致(Compact(len)+cid ++ AccountId32)。
fn verify_occupy_signature(account_id: &str, cid_number: &str, signature_hex: &str) -> bool {
    let Some(account_id_bytes) = parse_account_id_bytes(account_id) else {
        return false;
    };
    let Some(signature) = parse_signature_bytes(signature_hex) else {
        return false;
    };
    let mut payload = Vec::new();
    append_bounded(&mut payload, cid_number.as_bytes());
    payload.extend_from_slice(&account_id_bytes);
    let message =
        primitives::sign::signing_message(primitives::sign::OP_SIGN_CID_OCCUPY, &payload);
    let public = sr25519::Public::from_raw(account_id_bytes);
    let signature = sr25519::Signature::from_raw(signature);
    sr25519::Pair::verify(&signature, message, &public)
}

fn parse_signature_bytes(signature_hex: &str) -> Option<[u8; 64]> {
    let raw = hex::decode(signature_hex.trim_start_matches("0x")).ok()?;
    raw.try_into().ok()
}

/// revoke_cid(actor_cid_number, actor_role_code, cid_number)
fn encode_revoke_cid_call(
    actor_cid_number: &str,
    actor_role_code: &str,
    cid_number: &str,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(CITIZEN_IDENTITY_PALLET_INDEX);
    out.push(REVOKE_CID_CALL_INDEX);
    append_bounded(&mut out, actor_cid_number.as_bytes());
    append_bounded(&mut out, actor_role_code.as_bytes());
    append_bounded(&mut out, cid_number.as_bytes());
    out
}

// ──── DTO ────

#[derive(Serialize)]
pub(crate) struct PrepareCitizenOccupyOutput {
    pub(crate) request_id: String,
    pub(crate) cid_number: String,
    pub(crate) expires_at: i64,
}

/// 提交用户占号签名(第二段):管理员回扫用户已签名的响应二维码后回传。
#[derive(Deserialize)]
pub(crate) struct SubmitCitizenOccupyInput {
    pub(crate) request_id: String,
    /// 用户钱包账户(0x 小写 hex),占即绑主键;由用户签名响应二维码带回。
    pub(crate) account_id: String,
    /// 用户对 (cid_number, account_id) 的占号授权签名(域 OP_SIGN_CID_OCCUPY)。
    pub(crate) occupy_signature: String,
}

/// 提交用户占号签名返回:管理员冷签请求二维码(第三段冷签用)。
#[derive(Serialize)]
pub(crate) struct SubmitCitizenOccupyOutput {
    pub(crate) request_id: String,
    pub(crate) cid_number: String,
    pub(crate) sign_request: String,
    pub(crate) expires_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct ChainSubmitInput {
    pub(crate) request_id: String,
    /// 冷钱包扫码回签(前端已从响应 QR 解析);后端按会话签名字节重新验签。
    pub(crate) account_id: String,
    pub(crate) signature: String,
}

#[derive(Serialize)]
pub(crate) struct ChainSubmitOutput {
    pub(crate) purpose: String,
    pub(crate) cid_number: String,
    pub(crate) tx_hash: String,
    pub(crate) block_number: Option<u64>,
    pub(crate) citizen: Option<AdminCreateCitizenOutput>,
}

#[derive(Serialize)]
pub(crate) struct PrepareCitizenRevokeOutput {
    pub(crate) request_id: String,
    pub(crate) cid_number: String,
    pub(crate) sign_request: String,
    pub(crate) expires_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct PrepareCitizenRevokeInput {
    pub(crate) actor_role_code: String,
}

// ──── handlers ────

/// 建档占号 prepare(第一段):校验 + onchina 服务端 `cid_seed` 发号;不建 call、不落档案。
/// 返回 cid_number,供前端向用户展示占号签名请求二维码(用户对 (cid_number, account_id) 签名)。
pub(crate) async fn prepare_citizen_occupy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminCreateCitizenInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_registry_admin(&ctx) {
        return resp;
    }
    let actor_role_code = match validate_actor_role_code(input.actor_role_code.as_str()) {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    let validated = match validate_citizen_input(&ctx, &input) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let seed = cid_seed(&validated);

    // 发号:本地/链上双预查(占即绑,commitment 链上算 blake2_256(account_id),
    // 此刻账户未知,故只判该号是否空闲;落库失败恢复的同账户续用在 submit 阶段处理)。
    let mut chosen: Option<String> = None;
    for nonce in 0..CID_GENERATE_MAX_RETRY {
        let candidate = match generate_citizen_cid_candidate(&validated, &seed, nonce) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        match state.db.find_citizen_by_cid(candidate.as_str()) {
            Ok(Some(_)) => continue,
            Ok(None) => {}
            Err(err) => {
                tracing::error!(error = %err, "cid local pre-check failed");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "发号本地查重失败");
            }
        }
        match crate::core::chain_runtime::cid_registry_lookup(candidate.as_str()).await {
            Ok(false) => {
                chosen = Some(candidate);
                break;
            }
            Ok(true) => continue,
            Err(err) => {
                tracing::error!(error = %err, "cid chain pre-check failed");
                return api_error(StatusCode::BAD_GATEWAY, 1004, "发号链上查重失败(链不可用)");
            }
        }
    }
    let Some(cid_number) = chosen else {
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "发号重试耗尽");
    };

    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(SESSION_TTL_SECS);
    let request_id = format!("citizen-occupy-{}", Uuid::new_v4());
    // pending 会话:call_data/nonce/signing_hash 占位,待用户签名回来后 promote 回填。
    let session = ChainSignSession {
        request_id: request_id.clone(),
        purpose: PURPOSE_CITIZEN_OCCUPY_PENDING.to_string(),
        account_id: ctx.account_id.clone(),
        call_data: Vec::new(),
        nonce: 0,
        signing_hash: String::new(),
        context: serde_json::json!({
            "validated": validated,
            "cid_number": cid_number,
            "actor_role_code": actor_role_code,
        }),
        expires_at,
        consumed_at: None,
    };
    if let Err(err) = state.db.insert_chain_sign_session(&session) {
        tracing::error!(error = %err, "insert occupy pending session failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "占号会话落库失败");
    }

    crate::core::runtime_ops::append_audit_log(
        &state,
        "CITIZEN_OCCUPY_PREPARE",
        &ctx.account_id,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number,
            "request_id": request_id,
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PrepareCitizenOccupyOutput {
            request_id,
            cid_number,
            expires_at: expires_at.timestamp(),
        },
    })
    .into_response()
}

/// 提交用户占号签名(第二段):验用户签名 → 构造 occupy_cid call(占即绑 account_id)→
/// prepare_signing → 会话转正 → 返回管理员冷签请求二维码。
pub(crate) async fn submit_citizen_occupy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SubmitCitizenOccupyInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_registry_admin(&ctx) {
        return resp;
    }
    let session = match state.db.find_chain_sign_session(input.request_id.as_str()) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "占号会话不存在"),
        Err(err) => {
            tracing::error!(error = %err, "query occupy pending session failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "占号会话查询失败");
        }
    };
    if session.purpose != PURPOSE_CITIZEN_OCCUPY_PENDING {
        return api_error(StatusCode::CONFLICT, 1005, "占号会话状态不正确");
    }
    if session.consumed_at.is_some() {
        delete_session_best_effort(&state, session.request_id.as_str(), "consumed pending");
        return api_error(StatusCode::CONFLICT, 1005, "占号会话已被消费");
    }
    if session.expires_at < Utc::now() {
        delete_session_best_effort(&state, session.request_id.as_str(), "pending expired");
        return api_error(StatusCode::GONE, 1005, "占号会话已过期,请重新发起");
    }
    if !same_account_id(session.account_id.as_str(), ctx.account_id.as_str()) {
        return api_error(StatusCode::FORBIDDEN, 1003, "只有发起管理员可以提交本会话");
    }

    let cid_number = session
        .context
        .get("cid_number")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let actor_role_code = session
        .context
        .get("actor_role_code")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if cid_number.is_empty() || actor_role_code.is_empty() {
        delete_session_best_effort(&state, session.request_id.as_str(), "pending context invalid");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "占号会话数据损坏");
    }

    // 用户钱包账户(0x 小写 hex,占即绑主键)。
    let Some(account_id_hex) = normalize_account_id(input.account_id.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "用户钱包账户格式错误");
    };
    // 验用户占号签名:sr25519 over signing_message(OP_SIGN_CID_OCCUPY, (cid_number, account_id))。
    if !verify_occupy_signature(
        account_id_hex.as_str(),
        cid_number.as_str(),
        input.occupy_signature.as_str(),
    ) {
        return api_error(StatusCode::BAD_REQUEST, 1003, "用户占号签名验证失败");
    }
    let Some(account_id_bytes) = parse_account_id_bytes(account_id_hex.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "用户钱包账户格式错误");
    };
    let Some(occupy_signature_bytes) = parse_signature_bytes(input.occupy_signature.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "用户占号签名格式错误");
    };

    let actor_cid_number = match active_registry_cid_number(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let call = encode_occupy_cid_call(
        &actor_cid_number,
        actor_role_code.as_str(),
        cid_number.as_str(),
        &account_id_bytes,
        &occupy_signature_bytes,
    );
    let prepared = match chain_submit::prepare_signing(&call, ctx.account_id.as_str()).await {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "prepare occupy signing failed");
            return api_error(
                StatusCode::BAD_GATEWAY,
                1004,
                "链签名载荷准备失败(链不可用)",
            );
        }
    };

    let issued_at = Utc::now();
    let action = crate::core::institution_call::chain_action_code(
        CITIZEN_IDENTITY_PALLET_INDEX,
        OCCUPY_CID_CALL_INDEX,
    );
    let sign_request = match crate::core::qr::build_sign_request_bytes(
        session.request_id.as_str(),
        issued_at.timestamp(),
        session.expires_at.timestamp(),
        ctx.account_id.as_str(),
        &prepared.payload,
        action,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let mut context = session.context.clone();
    if let Some(map) = context.as_object_mut() {
        map.insert(
            "citizen_account_id".to_string(),
            serde_json::Value::String(account_id_hex.clone()),
        );
    }
    if let Err(err) = state.db.promote_chain_sign_session(
        session.request_id.as_str(),
        PURPOSE_CITIZEN_OCCUPY,
        &call,
        prepared.nonce,
        prepared.signing_hash_hex.as_str(),
        &context,
    ) {
        tracing::error!(error = %err, "promote occupy session failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "占号会话转正失败");
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: SubmitCitizenOccupyOutput {
            request_id: session.request_id,
            cid_number,
            sign_request,
            expires_at: session.expires_at.timestamp(),
        },
    })
    .into_response()
}

/// 吊销 prepare:登记表墓碑(最严档 PasskeyColdSign grant,与身份上链同档)。
pub(crate) async fn prepare_citizen_revoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(cid_number): Path<String>,
    Json(input): Json<PrepareCitizenRevokeInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_registry_admin(&ctx) {
        return resp;
    }
    let actor_role_code = match validate_actor_role_code(input.actor_role_code.as_str()) {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    let grant_payload = serde_json::json!({
        "cid_number": cid_number,
        "actor_role_code": actor_role_code,
        "op": "revoke",
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
    match state.db.find_citizen_by_cid(cid_number.as_str()) {
        Ok(Some(_)) => {}
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "公民档案不存在"),
        Err(err) => {
            tracing::error!(error = %err, "query citizen by cid failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "公民档案查询失败");
        }
    }
    let actor_cid_number = match active_registry_cid_number(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let call = encode_revoke_cid_call(
        &actor_cid_number,
        actor_role_code.as_str(),
        cid_number.as_str(),
    );
    let prepared = match chain_submit::prepare_signing(&call, ctx.account_id.as_str()).await {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "prepare revoke signing failed");
            return api_error(
                StatusCode::BAD_GATEWAY,
                1004,
                "链签名载荷准备失败(链不可用)",
            );
        }
    };
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(SESSION_TTL_SECS);
    let request_id = format!("citizen-revoke-{}", Uuid::new_v4());
    let action = crate::core::institution_call::chain_action_code(
        CITIZEN_IDENTITY_PALLET_INDEX,
        REVOKE_CID_CALL_INDEX,
    );
    let sign_request = match crate::core::qr::build_sign_request_bytes(
        request_id.as_str(),
        issued_at.timestamp(),
        expires_at.timestamp(),
        ctx.account_id.as_str(),
        &prepared.payload,
        action,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let session = ChainSignSession {
        request_id: request_id.clone(),
        purpose: PURPOSE_CITIZEN_REVOKE.to_string(),
        account_id: ctx.account_id.clone(),
        call_data: call,
        nonce: prepared.nonce,
        signing_hash: prepared.signing_hash_hex.clone(),
        context: serde_json::json!({
            "cid_number": cid_number,
            "actor_role_code": actor_role_code,
        }),
        expires_at,
        consumed_at: None,
    };
    if let Err(err) = state.db.insert_chain_sign_session(&session) {
        tracing::error!(error = %err, "insert revoke session failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "吊销会话落库失败");
    }
    crate::core::runtime_ops::append_audit_log(
        &state,
        "CITIZEN_REVOKE_PREPARE",
        &ctx.account_id,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number,
            "request_id": request_id,
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PrepareCitizenRevokeOutput {
            request_id,
            cid_number,
            sign_request,
            expires_at: expires_at.timestamp(),
        },
    })
    .into_response()
}

fn delete_session_best_effort(state: &AppState, request_id: &str, reason: &str) {
    if let Err(err) = state.db.delete_chain_sign_session(request_id) {
        tracing::error!(
            error = %err,
            request_id = %request_id,
            reason = %reason,
            "delete chain sign session failed"
        );
    }
}

/// 统一链交易 submit:验签者一致 → 组装/dry-run/提交 → 等进块 → 按 purpose 落正式投影。
pub(crate) async fn submit_chain_sign(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ChainSubmitInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let session = match state.db.find_chain_sign_session(input.request_id.as_str()) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "冷签会话不存在"),
        Err(err) => {
            tracing::error!(error = %err, "query chain sign session failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "冷签会话查询失败");
        }
    };
    if session.consumed_at.is_some() {
        delete_session_best_effort(&state, session.request_id.as_str(), "consumed residue");
        return api_error(StatusCode::CONFLICT, 1005, "冷签会话已被消费");
    }
    if session.expires_at < Utc::now() {
        delete_session_best_effort(&state, session.request_id.as_str(), "expired");
        return api_error(StatusCode::GONE, 1005, "冷签会话已过期,请重新发起");
    }
    if !same_account_id(session.account_id.as_str(), ctx.account_id.as_str()) {
        delete_session_best_effort(&state, session.request_id.as_str(), "actor mismatch");
        return api_error(StatusCode::FORBIDDEN, 1003, "只有发起管理员可以提交本会话");
    }
    if !same_account_id(
        input.account_id.as_str(),
        session.account_id.as_str(),
    ) {
        delete_session_best_effort(&state, session.request_id.as_str(), "signer mismatch");
        return api_error(StatusCode::FORBIDDEN, 1003, "签名钱包与会话管理员不一致");
    }
    if matches!(
        session.purpose.as_str(),
        PURPOSE_CITIZEN_OCCUPY | PURPOSE_CITIZEN_REVOKE | PURPOSE_CITIZEN_IDENTITY_PUSH
    ) {
        if let Err(resp) = ensure_registry_admin(&ctx) {
            delete_session_best_effort(&state, session.request_id.as_str(), "registry auth failed");
            return resp;
        }
    }
    if session.purpose == crate::domains::membership::PURPOSE_PLATFORM_PRICE_PROPOSAL {
        if let Err(resp) =
            crate::domains::membership::handler::recheck_platform_admin(&state, &ctx).await
        {
            delete_session_best_effort(
                &state,
                session.request_id.as_str(),
                "platform admin recheck failed",
            );
            return resp;
        }
    }
    if matches!(
        session.purpose.as_str(),
        crate::domains::legislation::law::action::PURPOSE_LEGISLATION_PROPOSE
            | crate::domains::legislation::law::action::PURPOSE_LEGISLATION_REPRESENTATIVE_VOTE
    ) {
        let session_cid_number = session
            .context
            .get("cid_number")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let authorized = session_cid_number == ctx.institution_cid_number
            && match session.purpose.as_str() {
                crate::domains::legislation::law::action::PURPOSE_LEGISLATION_PROPOSE => {
                    let vote_type = session
                        .context
                        .get("operation")
                        .and_then(|value| value.get("vote_type"))
                        .and_then(|value| value.as_u64())
                        .and_then(|value| u8::try_from(value).ok());
                    vote_type.is_some_and(|vote_type| {
                        crate::domains::legislation::category::proposable_candidates(
                            &ctx.institution_code,
                        )
                        .iter()
                        .any(|candidate| candidate.vote_types.contains(&vote_type))
                    })
                }
                crate::domains::legislation::law::action::PURPOSE_LEGISLATION_REPRESENTATIVE_VOTE => {
                    matches!(
                        crate::domains::legislation::category::legislation_role(
                            &ctx.institution_code
                        ),
                        Some(
                            crate::domains::legislation::category::LegislationRole::ProposerHouse
                                | crate::domains::legislation::category::LegislationRole::ReviewHouse
                        )
                    )
                }
                _ => false,
            };
        if !authorized {
            delete_session_best_effort(
                &state,
                session.request_id.as_str(),
                "legislation authorization recheck failed",
            );
            return api_error(StatusCode::FORBIDDEN, 1003, "当前机构无权提交该立法链交易");
        }
    }

    let tx_hash = match chain_submit::assemble_and_submit(
        &session.call_data,
        session.account_id.as_str(),
        input.signature.as_str(),
        session.nonce,
        session.signing_hash.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "chain submit failed");
            delete_session_best_effort(&state, session.request_id.as_str(), "chain submit failed");
            let detail = format!("链交易提交失败: {err}");
            return api_error(StatusCode::UNPROCESSABLE_ENTITY, 2004, detail.as_str());
        }
    };
    if let Err(err) =
        chain_submit::wait_nonce_consumed(session.account_id.as_str(), session.nonce).await
    {
        tracing::error!(error = %err, tx_hash = %tx_hash, "wait inclusion failed");
        delete_session_best_effort(&state, session.request_id.as_str(), "wait inclusion failed");
        let detail = format!("交易已提交({tx_hash})但未确认进块: {err}");
        return api_error(StatusCode::BAD_GATEWAY, 2004, detail.as_str());
    }
    let block_number = chain_submit::find_extrinsic_block(tx_hash.as_str())
        .await
        .ok()
        .flatten();

    let cid_number = session
        .context
        .get("cid_number")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    // 按 purpose 分派落正式投影。这里已经链上确认;失败路径不得提前写业务数据。
    let mut citizen_output = None;
    match session.purpose.as_str() {
        PURPOSE_CITIZEN_OCCUPY => {
            let validated: ValidatedCitizenInput = match session
                .context
                .get("validated")
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok())
            {
                Some(v) => v,
                None => {
                    delete_session_best_effort(
                        &state,
                        session.request_id.as_str(),
                        "invalid citizen context",
                    );
                    return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "会话档案数据损坏");
                }
            };
            // 占即绑:用户钱包账户在 submit_citizen_occupy 阶段已存入会话 context。
            let citizen_account_id = match session
                .context
                .get("citizen_account_id")
                .and_then(|v| v.as_str())
            {
                Some(v) => v.to_string(),
                None => {
                    delete_session_best_effort(
                        &state,
                        session.request_id.as_str(),
                        "missing citizen account_id",
                    );
                    return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "会话缺少用户钱包账户");
                }
            };
            let record = match persist_citizen_record(
                &state,
                &headers,
                ctx.account_id.as_str(),
                citizen_account_id.as_str(),
                &validated,
                cid_number.as_str(),
                tx_hash.as_str(),
                block_number,
            ) {
                Ok(v) => v,
                Err(resp) => {
                    delete_session_best_effort(
                        &state,
                        session.request_id.as_str(),
                        "persist citizen failed",
                    );
                    return resp;
                }
            };
            citizen_output = Some(create_output_from_record(record));
        }
        PURPOSE_CITIZEN_REVOKE => {
            if let Err(err) = state.db.mark_citizen_revoked(
                cid_number.as_str(),
                ctx.account_id.as_str(),
                tx_hash.as_str(),
            ) {
                tracing::error!(error = %err, "mark citizen revoked failed");
                delete_session_best_effort(
                    &state,
                    session.request_id.as_str(),
                    "mark citizen revoked failed",
                );
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "吊销落库失败");
            }
        }
        PURPOSE_CITIZEN_IDENTITY_PUSH => {
            let citizen_account_id = session
                .context
                .get("citizen_account_id")
                .and_then(|v| v.as_str());
            let Some(citizen_account_id) = citizen_account_id else {
                delete_session_best_effort(
                    &state,
                    session.request_id.as_str(),
                    "identity context invalid",
                );
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "身份上链会话数据损坏",
                );
            };
            // 只有链交易最终确认后，才一次性绑定公民账户并记录上链结果。
            if let Err(err) = state.db.confirm_citizen_identity_onchain(
                cid_number.as_str(),
                citizen_account_id,
                ctx.account_id.as_str(),
                tx_hash.as_str(),
                block_number,
            ) {
                tracing::error!(error = %err, "update citizen onchain failed");
                delete_session_best_effort(
                    &state,
                    session.request_id.as_str(),
                    "update citizen onchain failed",
                );
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "上链状态回写失败");
            }
        }
        crate::institution::admins::PURPOSE_INSTITUTION_GOVERNANCE
        | crate::institution::admins::PURPOSE_INSTITUTION_REGISTER_ADMINS
        | crate::institution::accounts::handler::PURPOSE_INSTITUTION_ADD_ACCOUNT
        | crate::institution::accounts::handler::PURPOSE_INSTITUTION_CLOSE_ACCOUNT => {
            // 机构治理、注册局登记管理员、机构自定义账户增/删提案的最终真源都在链上。
            // 提交成功后仅记录审计；OnChina 读侧继续通过链上 admins / roles / accounts 读取。
        }
        crate::domains::membership::PURPOSE_PLATFORM_PRICE_PROPOSAL => {
            // 平台价格与内部投票提案的唯一真源均在链上；提交成功后不保存本地价格副本。
        }
        crate::domains::legislation::law::action::PURPOSE_LEGISLATION_PROPOSE
        | crate::domains::legislation::law::action::PURPOSE_LEGISLATION_REPRESENTATIVE_VOTE => {
            // 立法提案和代表机构表决的真源均在链上；OnChina 不保存投票副本、不推进状态。
        }
        other => {
            tracing::error!(purpose = %other, "unknown chain sign purpose");
            delete_session_best_effort(&state, session.request_id.as_str(), "unknown purpose");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "未知会话用途");
        }
    }
    delete_session_best_effort(&state, session.request_id.as_str(), "completed");

    crate::core::runtime_ops::append_audit_log(
        &state,
        "CHAIN_SIGN_SUBMIT",
        &ctx.account_id,
        Some(cid_number.clone()),
        serde_json::json!({
            "purpose": session.purpose,
            "cid_number": cid_number,
            "tx_hash": tx_hash,
            "block_number": block_number,
            "request_id": session.request_id,
            "actor_ip": actor_ip_from_headers(&headers),
        }),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: ChainSubmitOutput {
            purpose: session.purpose,
            cid_number,
            tx_hash,
            block_number,
            citizen: citizen_output,
        },
    })
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 占号调用字节 golden:锁死链↔onchina 字节契约(pallet 10 / call 6 占即绑新签名)。
    /// 布局 = [10][6] Compact(len)+actor_cid ‖ Compact(len)+actor_role ‖ Compact(len)+cid
    ///        ‖ account_id(32 裸字节) ‖ Compact(len)+occupy_signature。
    #[test]
    fn encode_occupy_cid_call_byte_golden() {
        let account_id = [0x11u8; 32];
        let occupy_signature = [0x22u8; 4];
        let out = encode_occupy_cid_call("A", "B", "C", &account_id, &occupy_signature);
        // 0a06 | 04 41 | 04 42 | 04 43 | 11*32 | 10 | 22*4
        let expected = concat!(
            "0a06",
            "0441",
            "0442",
            "0443",
            "1111111111111111111111111111111111111111111111111111111111111111",
            "10",
            "22222222",
        );
        assert_eq!(hex::encode(out), expected);
    }
}
