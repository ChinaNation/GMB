//! 立法冷签动作:组织 `ChainCall` → 构造扫码上链 `sign_request`(复用机构创建同款 `build_sign_request_bytes`)。
//!
//! 立法提案/代表机构表决是链上 extrinsic（管理员 origin 冷签提交），走链交易 QR 路径。
//! (`b.a = chain_action_code(pallet, call)`、`b.d = SCALE call_data`),**不走** `onchina_admin_governance`
//! 文本 QR 路径,故不经 `auth/actions.rs` 的 prepare/commit 治理流。范式与
//! `institution::subjects::registration::build_institution_create_sign_request` 完全一致。
//!
//! 机构 CID 解析闭包(`resolve_cid_number`)由 handler 注入(subjects 表按机构码和行政区查
//! cid_number),本文件保持与 DB 解耦、可单测。越权前置由
//! `service::precheck_legislation_scope` 在 handler 先行拦截。
//!

use super::model::ProposeLawInput;
use super::service::{build_propose_law_call, build_representative_vote_call};
use crate::api_error;
use crate::core::qr::build_sign_request_bytes;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{Duration, Utc};
use uuid::Uuid;

/// 冷签动作有效期(秒),与机构创建冷签一致。
const LEGISLATION_SIGN_TTL_SECONDS: i64 = 120;

/// 立法提案冷签 `sign_request`(actor_pubkey = 发起机构管理员;闭包只解析机构 CID)。
///
/// houses/actor/executive/legislature 由宪法路由解析,前端不传(防越权)。
pub(crate) fn build_propose_law_sign_request(
    input: &ProposeLawInput,
    proposer_code: [u8; 4],
    actor_pubkey: &str,
    resolve_cid_number: impl Fn(&[u8; 4]) -> Option<String>,
) -> Result<String, Response> {
    let chain = build_propose_law_call(input, proposer_code, resolve_cid_number)
        .map_err(|e| api_error(StatusCode::UNPROCESSABLE_ENTITY, 2001, e.code()))?;
    build_chain_sign_request("leg-propose", actor_pubkey, &chain.call_data, chain.action)
}

/// 代表机构表决冷签 `sign_request`（actor_pubkey = 投票管理员）。
pub(crate) fn build_representative_vote_sign_request(
    proposal_id: u64,
    approve: bool,
    actor_pubkey: &str,
) -> Result<String, Response> {
    let chain = build_representative_vote_call(proposal_id, approve);
    build_chain_sign_request(
        "leg-representative-vote",
        actor_pubkey,
        &chain.call_data,
        chain.action,
    )
}

/// 统一构造链交易 `sign_request`(action_id + TTL + `build_sign_request_bytes`)。
fn build_chain_sign_request(
    prefix: &str,
    actor_pubkey: &str,
    call_data: &[u8],
    action: u16,
) -> Result<String, Response> {
    let action_id = format!("{prefix}-{}", Uuid::new_v4());
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(LEGISLATION_SIGN_TTL_SECONDS);
    build_sign_request_bytes(
        action_id.as_str(),
        issued_at.timestamp(),
        expires_at.timestamp(),
        actor_pubkey,
        call_data,
        action,
    )
}

#[cfg(test)]
mod tests {
    use super::super::model::{LawActionInput, LawChapter, LawSection};
    use super::*;

    fn fixture_resolver(code: &[u8; 4]) -> Option<String> {
        Some(format!(
            "LN001-{}0G-000000001-2026",
            super::super::model::institution_code_text(code)
        ))
    }

    fn enact_input() -> ProposeLawInput {
        ProposeLawInput {
            law_action: LawActionInput::Enact,
            tier: 1,
            scope_code: 0,
            vote_type: 2,
            title: "道路交通安全法".to_string(),
            title_en: Some("Road Traffic Safety Law".to_string()),
            chapters: vec![LawChapter {
                number: 1,
                title: "总则".to_string(),
                title_en: None,
                sections: vec![LawSection {
                    number: 1,
                    title: "定义".to_string(),
                    title_en: None,
                    articles: vec![],
                }],
            }],
            effective_at: 1000,
            law_id: None,
        }
    }

    /// 32 字节 hex(64 字符)actor pubkey 夹具。
    fn actor_hex(byte: &str) -> String {
        byte.repeat(32)
    }

    /// 提案 sign_request 承载 enact 动作码(0x1900)与非空 b.d(base64 call_data)。
    #[test]
    fn propose_law_sign_request_carries_enact_action_and_calldata() {
        let sign_request = build_propose_law_sign_request(
            &enact_input(),
            *b"NRP\0",
            actor_hex("11").as_str(),
            fixture_resolver,
        )
        .expect("build propose sign_request");

        let json: serde_json::Value = serde_json::from_str(&sign_request).expect("parse json");
        assert_eq!(json["b"]["a"].as_u64().unwrap(), 0x1900); // (25<<8)|0 = propose_enact_law
        assert!(!json["b"]["d"].as_str().unwrap().is_empty()); // call_data(base64)非空
    }

    /// 代表机构表决 sign_request 承载 cast_representative_vote 动作码（0x1A01）。
    #[test]
    fn representative_vote_sign_request_targets_legislation_vote() {
        let sign_request =
            build_representative_vote_sign_request(42, true, actor_hex("22").as_str())
                .expect("build vote sign_request");
        let json: serde_json::Value = serde_json::from_str(&sign_request).expect("parse json");
        assert_eq!(json["b"]["a"].as_u64().unwrap(), 0x1A01); // (26<<8)|1 = cast_representative_vote
    }

    /// 越权/非法输入在组织阶段即拒(省教育案无路由 → 提案组织错误映射为 422)。
    #[test]
    fn invalid_routing_is_rejected_before_sign_request() {
        let mut input = enact_input();
        input.tier = 2; // 省
        input.vote_type = 1; // 教育案(省无教委会 → 无路由)
        let result = build_propose_law_sign_request(
            &input,
            *b"PRP\0",
            actor_hex("33").as_str(),
            fixture_resolver,
        );
        assert!(result.is_err());
    }
}
