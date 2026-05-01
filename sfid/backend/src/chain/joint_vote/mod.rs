//! 联合投票 · 公民人数快照凭证(chain pull)。
//!
//! - HTTP 端点:`GET /api/v1/app/voters/count?account_pubkey=<hex>`
//! - 调用方:`citizenchain/node` 在创建联合投票提案时 pull 此凭证,作为 extrinsic 入参随提案上链
//! - 返回字段:`genesis_hash` / `who` / `eligible_total` / `snapshot_nonce` / `signature`
//! - 签名:`blake2_256(scale_encode(DUOQIAN_DOMAIN ++ OP_SIGN_POP ++ payload))` 用 SFID main 私钥
//!
//! 链上 verifier:`citizenchain/runtime/src/configs/mod.rs::verify_population_snapshot`。
//! 任何字段或签名顺序变更都必须与 runtime 同步,否则验签失败。

pub(crate) mod handler;

pub(crate) use handler::app_voters_count;
