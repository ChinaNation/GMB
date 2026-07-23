//! 立法链交易签名准备。
//!
//! 本模块只负责把已经完成权限与辖区前置校验的 `ChainCall` 转为统一链签名会话：
//! OnChina 展示请求二维码，CitizenWallet 只签名一次并显示响应二维码，OnChina 回扫后
//! 统一通过 `/api/v1/admin/chain/submit` 验签、dry-run、提交并等待进块。

use super::model::ProposeLawInput;
use super::service::{build_propose_law_call, build_representative_vote_call};
use crate::core::institution_call::ChainCall;
use crate::domains::citizens::occupy::{ChainSignSession, SESSION_TTL_SECS};
use crate::{api_error, AppState};
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{Duration, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) const PURPOSE_LEGISLATION_PROPOSE: &str = "LEGISLATION_PROPOSE";
pub(crate) const PURPOSE_LEGISLATION_REPRESENTATIVE_VOTE: &str = "LEGISLATION_REPRESENTATIVE_VOTE";

/// 所有立法链交易 prepare 接口统一返回请求编号和请求二维码载荷。
#[derive(Debug, Serialize)]
pub(crate) struct LegislationSignOutput {
    request_id: String,
    sign_request: String,
}

/// 准备法律案提案签名会话；路由机构 CID 仍由 handler 注入的数据库解析器提供。
pub(crate) async fn prepare_propose_law_sign(
    state: &AppState,
    input: &ProposeLawInput,
    proposer_code: [u8; 4],
    actor_public_key: &str,
    institution_cid_number: &str,
    resolve_cid_number: impl Fn(&[u8; 4]) -> Option<String>,
) -> Result<LegislationSignOutput, Response> {
    let chain = build_propose_law_call(input, proposer_code, resolve_cid_number)
        .map_err(|error| api_error(StatusCode::UNPROCESSABLE_ENTITY, 2001, error.code()))?;
    prepare_legislation_sign(
        state,
        "leg-propose",
        PURPOSE_LEGISLATION_PROPOSE,
        actor_public_key,
        institution_cid_number,
        chain,
        json!({
            "tier": input.tier,
            "scope_code": input.scope_code,
            "vote_type": input.vote_type,
        }),
    )
    .await
}

/// 准备代表机构表决签名会话。
pub(crate) async fn prepare_representative_vote_sign(
    state: &AppState,
    proposal_id: u64,
    voter_role_code: &str,
    approve: bool,
    actor_public_key: &str,
    institution_cid_number: &str,
) -> Result<LegislationSignOutput, Response> {
    prepare_legislation_sign(
        state,
        "leg-representative-vote",
        PURPOSE_LEGISLATION_REPRESENTATIVE_VOTE,
        actor_public_key,
        institution_cid_number,
        build_representative_vote_call(proposal_id, voter_role_code, approve),
        json!({
            "proposal_id": proposal_id,
            "voter_role_code": voter_role_code,
            "approve": approve,
        }),
    )
    .await
}

/// 统一读取实时 nonce/runtime/创世哈希，保存短期会话并生成完整审阅载荷。
async fn prepare_legislation_sign(
    state: &AppState,
    request_prefix: &str,
    purpose: &str,
    actor_public_key: &str,
    institution_cid_number: &str,
    chain: ChainCall,
    operation_context: Value,
) -> Result<LegislationSignOutput, Response> {
    let prepared = crate::core::chain_submit::prepare_signing(&chain.call_data, actor_public_key)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, purpose, "prepare legislation signing failed");
            api_error(
                StatusCode::BAD_GATEWAY,
                5002,
                "链签名载荷准备失败(链不可用)",
            )
        })?;
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(SESSION_TTL_SECS);
    let request_id = format!("{request_prefix}-{}", Uuid::new_v4());
    let sign_request = crate::core::qr::build_sign_request_bytes(
        request_id.as_str(),
        issued_at.timestamp(),
        expires_at.timestamp(),
        actor_public_key,
        &prepared.payload,
        chain.action,
    )?;
    let session = ChainSignSession {
        request_id: request_id.clone(),
        purpose: purpose.to_string(),
        actor_public_key: actor_public_key.to_string(),
        call_data: chain.call_data,
        nonce: prepared.nonce,
        signing_hash: prepared.signing_hash_hex,
        context: json!({
            "cid_number": institution_cid_number,
            "operation": operation_context,
        }),
        expires_at,
        consumed_at: None,
    };
    state
        .db
        .insert_chain_sign_session(&session)
        .map_err(|error| {
            tracing::error!(error = %error, purpose, "insert legislation chain sign session failed");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "立法链签名会话保存失败",
            )
        })?;
    Ok(LegislationSignOutput {
        request_id,
        sign_request,
    })
}
