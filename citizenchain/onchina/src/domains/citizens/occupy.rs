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
use crate::domains::citizens::admin_entry::{
    citizen_cid_seed, create_output_from_record, generate_citizen_cid_candidate,
    persist_citizen_record, validate_citizen_input, AdminCreateCitizenInput,
    AdminCreateCitizenOutput, ValidatedCitizenInput,
};
use crate::domains::citizens::chain_identity::{
    active_registry_cid_number, ensure_registry_admin, same_pubkey_hex,
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
pub(crate) const PURPOSE_CITIZEN_REVOKE: &str = "CITIZEN_REVOKE";
pub(crate) const PURPOSE_CITIZEN_IDENTITY_PUSH: &str = "CITIZEN_IDENTITY_PUSH";

/// 链冷签会话:prepare 只保存短期签名 payload。
///
/// 这不是公民或机构的业务草稿状态。submit 成功或失败后都必须删除;
/// 公民/机构正式数据只能在链上确认成功后写入正式投影表。
pub(crate) struct ChainSignSession {
    pub(crate) request_id: String,
    pub(crate) purpose: String,
    /// 管理员钱包公钥 hex(签名者必须与之一致)。
    pub(crate) actor_pubkey: String,
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
            actor_pubkey: s.actor_pubkey.clone(),
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
                    (request_id, purpose, actor_pubkey, call_data, nonce, signing_hash,
                     context, expires_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &s.request_id,
                    &s.purpose,
                    &s.actor_pubkey,
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
                    "SELECT request_id, purpose, actor_pubkey, call_data, nonce, signing_hash,
                            context, expires_at, consumed_at
                     FROM chain_sign_sessions WHERE request_id = $1",
                    &[&request_id],
                )
                .map_err(|e| format!("query chain sign session failed: {e}"))?;
            Ok(row.map(|r| ChainSignSession {
                request_id: r.get(0),
                purpose: r.get(1),
                actor_pubkey: r.get(2),
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

    /// 吊销落库:本地档案状态置 REVOKED(墓碑语义,档案保留)。
    pub(crate) fn mark_citizen_revoked(
        &self,
        cid_number: &str,
        admin_account: &str,
        onchain_tx_hash: &str,
    ) -> Result<u64, String> {
        let cid_number = cid_number.to_string();
        let admin_account = admin_account.to_string();
        let onchain_tx_hash = onchain_tx_hash.to_string();
        self.with_client(move |conn| {
            conn.execute(
                "UPDATE citizens
                 SET citizen_status = 'REVOKED', status_updated_at = extract(epoch from now())::bigint,
                     onchain_tx_hash = $2, onchain_at = now(), updated_by = $3, updated_at = now()
                 WHERE cid_number = $1",
                &[&cid_number, &onchain_tx_hash, &admin_account],
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
        wallet_pubkey: &str,
        wallet_address: &str,
        admin_account: &str,
        onchain_tx_hash: &str,
        onchain_block_number: Option<u64>,
    ) -> Result<u64, String> {
        let cid_number = cid_number.to_string();
        let wallet_pubkey = wallet_pubkey.to_string();
        let wallet_address = wallet_address.to_string();
        let admin_account = admin_account.to_string();
        let onchain_tx_hash = onchain_tx_hash.to_string();
        let block = onchain_block_number.map(|n| n as i64);
        self.with_client(move |conn| {
            conn.execute(
                "UPDATE citizens
                 SET wallet_pubkey = $2, wallet_address = $3, wallet_sig_alg = 'sr25519',
                     wallet_verified_at = now(), onchain_tx_hash = $4,
                     onchain_block_number = $5, onchain_at = now(),
                     updated_by = $6, updated_at = now()
                 WHERE cid_number = $1",
                &[
                    &cid_number,
                    &wallet_pubkey,
                    &wallet_address,
                    &onchain_tx_hash,
                    &block,
                    &admin_account,
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

/// occupy_cid(actor_cid_number, cid_number, commitment, province_code, city_code)
fn encode_occupy_cid_call(
    actor_cid_number: &str,
    cid_number: &str,
    commitment: &[u8; 32],
    province_code: &str,
    city_code: &str,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(CITIZEN_IDENTITY_PALLET_INDEX);
    out.push(OCCUPY_CID_CALL_INDEX);
    append_bounded(&mut out, actor_cid_number.as_bytes());
    append_bounded(&mut out, cid_number.as_bytes());
    out.extend_from_slice(commitment);
    append_bounded(&mut out, province_code.as_bytes());
    append_bounded(&mut out, city_code.as_bytes());
    out
}

/// revoke_cid(actor_cid_number, cid_number)
fn encode_revoke_cid_call(actor_cid_number: &str, cid_number: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(CITIZEN_IDENTITY_PALLET_INDEX);
    out.push(REVOKE_CID_CALL_INDEX);
    append_bounded(&mut out, actor_cid_number.as_bytes());
    append_bounded(&mut out, cid_number.as_bytes());
    out
}

// ──── DTO ────

#[derive(Serialize)]
pub(crate) struct PrepareCitizenOccupyOutput {
    pub(crate) request_id: String,
    pub(crate) cid_number: String,
    pub(crate) sign_request: String,
    pub(crate) expires_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct ChainSubmitInput {
    pub(crate) request_id: String,
    /// 冷钱包扫码回签(前端已从响应 QR 解析);后端按会话签名字节重新验签。
    pub(crate) signer_pubkey: String,
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

// ──── handlers ────

/// 建档占号 prepare:占号先行,本接口不落任何档案。
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
    let validated = match validate_citizen_input(&ctx, &input) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let seed = citizen_cid_seed(&validated);
    let commitment = sp_core::hashing::blake2_256(seed.as_bytes());

    // 发号:本地/链上双预查;链上同承诺记录 = 落库失败恢复,直接续用。
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
            Ok(None) => {
                chosen = Some(candidate);
                break;
            }
            Ok(Some(rec)) if rec.status_active && rec.commitment == commitment => {
                // 幂等续用:同承诺占号已在链上,本地档案缺失(上次落库失败)。
                chosen = Some(candidate);
                break;
            }
            Ok(Some(_)) => continue,
            Err(err) => {
                tracing::error!(error = %err, "cid chain pre-check failed");
                return api_error(StatusCode::BAD_GATEWAY, 1004, "发号链上查重失败(链不可用)");
            }
        }
    }
    let Some(cid_number) = chosen else {
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "发号重试耗尽");
    };

    let actor_cid_number = match active_registry_cid_number(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let call = encode_occupy_cid_call(
        &actor_cid_number,
        cid_number.as_str(),
        &commitment,
        validated.province_code.as_str(),
        validated.city_code.as_str(),
    );
    let prepared = match chain_submit::prepare_signing(&call, ctx.admin_account.as_str()).await {
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
    let expires_at = issued_at + Duration::seconds(SESSION_TTL_SECS);
    let request_id = format!("citizen-occupy-{}", Uuid::new_v4());
    let action = crate::core::institution_call::chain_action_code(
        CITIZEN_IDENTITY_PALLET_INDEX,
        OCCUPY_CID_CALL_INDEX,
    );
    let sign_request = match crate::core::qr::build_sign_request_bytes(
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
    let session = ChainSignSession {
        request_id: request_id.clone(),
        purpose: PURPOSE_CITIZEN_OCCUPY.to_string(),
        actor_pubkey: ctx.admin_account.clone(),
        call_data: call,
        nonce: prepared.nonce,
        signing_hash: prepared.signing_hash_hex.clone(),
        context: serde_json::json!({
            "validated": validated,
            "cid_number": cid_number,
            "commitment": hex::encode(commitment),
        }),
        expires_at,
        consumed_at: None,
    };
    if let Err(err) = state.db.insert_chain_sign_session(&session) {
        tracing::error!(error = %err, "insert occupy session failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "占号会话落库失败");
    }

    crate::core::runtime_ops::append_audit_log(
        &state,
        "CITIZEN_OCCUPY_PREPARE",
        &ctx.admin_account,
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
            sign_request,
            expires_at: expires_at.timestamp(),
        },
    })
    .into_response()
}

/// 吊销 prepare:登记表墓碑(最严档 PasskeyColdSign grant,与身份上链同档)。
pub(crate) async fn prepare_citizen_revoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(cid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_registry_admin(&ctx) {
        return resp;
    }
    let grant_payload = serde_json::json!({ "cid_number": cid_number, "op": "revoke" });
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
    let call = encode_revoke_cid_call(&actor_cid_number, cid_number.as_str());
    let prepared = match chain_submit::prepare_signing(&call, ctx.admin_account.as_str()).await {
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
        ctx.admin_account.as_str(),
        &prepared.payload,
        action,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let session = ChainSignSession {
        request_id: request_id.clone(),
        purpose: PURPOSE_CITIZEN_REVOKE.to_string(),
        actor_pubkey: ctx.admin_account.clone(),
        call_data: call,
        nonce: prepared.nonce,
        signing_hash: prepared.signing_hash_hex.clone(),
        context: serde_json::json!({ "cid_number": cid_number }),
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
        &ctx.admin_account,
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
    if !same_pubkey_hex(session.actor_pubkey.as_str(), ctx.admin_account.as_str()) {
        delete_session_best_effort(&state, session.request_id.as_str(), "actor mismatch");
        return api_error(StatusCode::FORBIDDEN, 1003, "只有发起管理员可以提交本会话");
    }
    if !same_pubkey_hex(input.signer_pubkey.as_str(), session.actor_pubkey.as_str()) {
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
        session.actor_pubkey.as_str(),
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
        chain_submit::wait_nonce_consumed(session.actor_pubkey.as_str(), session.nonce).await
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
            let record = match persist_citizen_record(
                &state,
                &headers,
                ctx.admin_account.as_str(),
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
                ctx.admin_account.as_str(),
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
            let wallet_pubkey = session
                .context
                .get("wallet_pubkey")
                .and_then(|v| v.as_str());
            let wallet_address = session
                .context
                .get("wallet_address")
                .and_then(|v| v.as_str());
            let (Some(wallet_pubkey), Some(wallet_address)) = (wallet_pubkey, wallet_address)
            else {
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
            // 只有链交易最终确认后，才一次性绑定公民钱包并记录上链结果。
            if let Err(err) = state.db.confirm_citizen_identity_onchain(
                cid_number.as_str(),
                wallet_pubkey,
                wallet_address,
                ctx.admin_account.as_str(),
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
        | crate::institution::admins::PURPOSE_INSTITUTION_REGISTER_ADMINS => {
            // 机构治理与注册局登记管理员的最终真源在链上。提交成功后仅记录审计；
            // OnChina 读侧继续通过链上 admins / roles / assignments 读取。
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
        &ctx.admin_account,
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
