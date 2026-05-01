//! 公民投票凭证(chain pull)。
//!
//! - HTTP 端点:`POST /api/v1/app/vote/credential`
//! - 调用方:wuminapp 公民投票流程,pull 凭证后作为 extrinsic 入参提交链上 vote 调用
//! - 返回字段:`genesis_hash` / `who` / `binding_id` / `proposal_id` / `vote_nonce` / `signature`
//! - 签名:`blake2_256(scale_encode(DUOQIAN_DOMAIN ++ OP_SIGN_VOTE ++ payload))` 用 SFID main 私钥
//!
//! 链上 verifier:`citizenchain/runtime/otherpallet/sfid-system/src/lib.rs::verify_and_consume_vote_credential`。
//! 任何字段或签名顺序变更都必须与 runtime 同步,否则验签失败。

pub(crate) mod handler;

pub(crate) use handler::app_vote_credential;
