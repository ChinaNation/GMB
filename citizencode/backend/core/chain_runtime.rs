use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use parity_scale_codec::Encode;
use primitives::core_const::{
    GMB, OP_SIGN_DEREGISTER, OP_SIGN_INST, OP_SIGN_POP, OP_SIGN_VOTE,
};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};
use std::sync::{Arc, OnceLock, RwLock};
use subxt::{OnlineClient, PolkadotConfig};

use crate::admins::login::parse_sr25519_pubkey_bytes;
use crate::*;

// 中文注释：本文件所有 GMB + OP_SIGN_* 均直接来自
// `primitives::core_const`。SCALE 编码下：
//   [u8; N] / &[u8; N]  →  N 字节，无长度前缀
//   u8                   →  1 字节
//   &[u8] / Vec<u8>     →  Compact(N) ++ N 字节，多 1~4 字节长度前缀
// 任何一个 domain 写成 &[u8] 都会导致 message 与链端不一致 → blake2_256 不同
// → sr25519 verify 失败 → 链端返回 InvalidCidXxxSignature。
// 历史教训：INSTITUTION_DOMAIN 曾被错误声明为 &[u8]，导致 register_cid_institution
// 长期 InvalidCidInstitutionSignature。修复见 ADR
// `04-decisions/citizencode/2026-04-07-subxt-0.43-pow-chain-quirks.md`。
static CHAIN_GENESIS_HASH: OnceLock<[u8; 32]> = OnceLock::new();
static SIGNING_KEY_CACHE: OnceLock<RwLock<Option<CachedSigningKey>>> = OnceLock::new();
const TRUSTED_PRODUCTION_CHAINS: &[TrustedProductionChain] = &[
    // 中文注释：正式链创世哈希在这里做源码级白名单绑定；新增正式链时只允许在此处追加。
    // TrustedProductionChain { name: "mainnet", genesis_hash_hex: "0x<正式链创世哈希>" },
];

struct CachedSigningKey {
    seed_hex: SensitiveSeed,
    keypair: Arc<Sr25519Pair>,
}

#[derive(Debug, Clone, Copy)]
struct TrustedProductionChain {
    name: &'static str,
    genesis_hash_hex: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct RuntimeSignatureMeta {
    pub(crate) key_id: String,
    pub(crate) key_version: String,
    pub(crate) alg: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimeVoteCredential {
    pub(crate) genesis_hash: String,
    pub(crate) who: String,
    pub(crate) binding_id: String,
    pub(crate) proposal_id: u64,
    pub(crate) vote_nonce: String,
    pub(crate) issuer_cid_number: String,
    pub(crate) issuer_main_account: String,
    pub(crate) signer_pubkey: String,
    pub(crate) scope_province_name: String,
    pub(crate) scope_city_name: String,
    pub(crate) signature: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimePopulationSnapshotCredential {
    pub(crate) who: String,
    pub(crate) eligible_total: u64,
    pub(crate) snapshot_nonce: String,
    pub(crate) issuer_cid_number: String,
    pub(crate) issuer_main_account: String,
    pub(crate) signer_pubkey: String,
    pub(crate) scope_province_name: String,
    pub(crate) scope_city_name: String,
    pub(crate) signature: String,
    pub(crate) genesis_hash: String,
    pub(crate) payload_digest: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimeInstitutionRegistrationCredential {
    pub(crate) genesis_hash: String,
    pub(crate) register_nonce: String,
    pub(crate) issuer_cid_number: String,
    pub(crate) issuer_main_account: String,
    pub(crate) signer_pubkey: String,
    pub(crate) scope_province_name: String,
    pub(crate) scope_city_name: String,
    pub(crate) signature: String,
    pub(crate) payload_digest: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimeInstitutionDeregistrationCredential {
    pub(crate) genesis_hash: String,
    pub(crate) scope: u8,
    pub(crate) cid_number: String,
    pub(crate) account_name: String,
    pub(crate) target_account: String,
    pub(crate) deregister_nonce: String,
    pub(crate) issuer_cid_number: String,
    pub(crate) issuer_main_account: String,
    pub(crate) signer_pubkey: String,
    pub(crate) signature: String,
    pub(crate) payload_digest: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

struct RuntimeSigningContext {
    issuer_cid_number: String,
    issuer_main_account: [u8; 32],
    issuer_main_account_hex: String,
    signer_pubkey: [u8; 32],
    signer_pubkey_hex: String,
    scope_province_name: String,
    scope_city_name: String,
}

pub(crate) fn build_vote_credential(
    state: &AppState,
    account_pubkey: &str,
    binding_seed: &str,
    proposal_id: u64,
    vote_nonce: String,
) -> Result<RuntimeVoteCredential, String> {
    if vote_nonce.trim().is_empty() {
        return Err("vote_nonce is required".to_string());
    }
    if binding_seed.trim().is_empty() {
        return Err("binding seed is required".to_string());
    }
    let (normalized_who, who) = normalize_and_parse_account_id32(account_pubkey)?;
    let genesis_hash = resolve_chain_genesis_hash()?;
    let binding_id = blake2_256(binding_seed.as_bytes());
    let signing_ctx = runtime_signing_context(None, None)?;
    let payload = (
        GMB,
        OP_SIGN_VOTE,
        genesis_hash,
        who,
        binding_id,
        proposal_id,
        vote_nonce.as_bytes(),
        signing_ctx.issuer_cid_number.as_bytes(),
        &signing_ctx.issuer_main_account,
        &signing_ctx.signer_pubkey,
        signing_ctx.scope_province_name.as_bytes(),
        signing_ctx.scope_city_name.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimeVoteCredential {
        genesis_hash: hex::encode(genesis_hash),
        who: normalized_who,
        binding_id: hex::encode(binding_id),
        proposal_id,
        vote_nonce,
        issuer_cid_number: signing_ctx.issuer_cid_number,
        issuer_main_account: signing_ctx.issuer_main_account_hex,
        signer_pubkey: signing_ctx.signer_pubkey_hex,
        scope_province_name: signing_ctx.scope_province_name,
        scope_city_name: signing_ctx.scope_city_name,
        signature,
        meta: runtime_signature_meta(state),
    })
}

pub(crate) fn build_population_snapshot_credential(
    state: &AppState,
    account_pubkey: &str,
    eligible_total: u64,
    snapshot_nonce: String,
) -> Result<RuntimePopulationSnapshotCredential, String> {
    if snapshot_nonce.trim().is_empty() {
        return Err("snapshot_nonce is required".to_string());
    }
    let (normalized_who, who) = normalize_and_parse_account_id32(account_pubkey)?;
    let genesis_hash = resolve_chain_genesis_hash()?;
    let signing_ctx = runtime_signing_context(None, None)?;
    let payload = (
        GMB,
        OP_SIGN_POP,
        genesis_hash,
        who,
        eligible_total,
        snapshot_nonce.as_bytes(),
        signing_ctx.issuer_cid_number.as_bytes(),
        &signing_ctx.issuer_main_account,
        &signing_ctx.signer_pubkey,
        signing_ctx.scope_province_name.as_bytes(),
        signing_ctx.scope_city_name.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimePopulationSnapshotCredential {
        who: normalized_who,
        eligible_total,
        snapshot_nonce,
        issuer_cid_number: signing_ctx.issuer_cid_number,
        issuer_main_account: signing_ctx.issuer_main_account_hex,
        signer_pubkey: signing_ctx.signer_pubkey_hex,
        scope_province_name: signing_ctx.scope_province_name,
        scope_city_name: signing_ctx.scope_city_name,
        signature,
        genesis_hash: hex::encode(genesis_hash),
        payload_digest: hex::encode(payload_digest),
        meta: runtime_signature_meta(state),
    })
}

pub(crate) fn build_institution_registration_credential(
    state: &AppState,
    cid_number: &str,
    cid_full_name: &str,
    account_names: &[String],
    register_nonce: String,
    scope_province_name: &str,
    scope_city_name: &str,
) -> Result<RuntimeInstitutionRegistrationCredential, String> {
    if cid_number.trim().is_empty() {
        return Err("cid_number is required".to_string());
    }
    if cid_full_name.trim().is_empty() {
        return Err("cid_full_name is required".to_string());
    }
    if account_names.is_empty() || account_names.iter().any(|name| name.trim().is_empty()) {
        return Err("account_names are required".to_string());
    }
    if register_nonce.trim().is_empty() {
        return Err("register_nonce is required".to_string());
    }
    let genesis_hash = resolve_chain_genesis_hash()?;
    let signing_ctx = runtime_signing_context(Some(scope_province_name), Some(scope_city_name))?;
    let account_name_payload = account_names
        .iter()
        .map(|name| name.trim().as_bytes().to_vec())
        .collect::<Vec<_>>();
    let payload = (
        GMB,
        OP_SIGN_INST,
        genesis_hash,
        cid_number.trim().as_bytes(),
        cid_full_name.trim().as_bytes(),
        &account_name_payload,
        register_nonce.trim().as_bytes(),
        signing_ctx.issuer_cid_number.as_bytes(),
        &signing_ctx.issuer_main_account,
        &signing_ctx.signer_pubkey,
        signing_ctx.scope_province_name.as_bytes(),
        signing_ctx.scope_city_name.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimeInstitutionRegistrationCredential {
        genesis_hash: hex::encode(genesis_hash),
        register_nonce,
        issuer_cid_number: signing_ctx.issuer_cid_number,
        issuer_main_account: signing_ctx.issuer_main_account_hex,
        signer_pubkey: signing_ctx.signer_pubkey_hex,
        scope_province_name: signing_ctx.scope_province_name,
        scope_city_name: signing_ctx.scope_city_name,
        signature,
        payload_digest: hex::encode(payload_digest),
        meta: runtime_signature_meta(state),
    })
}

/// 中文注释:注销凭证签名 payload 的 blake2_256 摘要(纯函数,便于 golden 测试锁字节)。
///
/// **铁律**:元素顺序与 SCALE 类型必须与链端 `organization-manage` 的
/// `verify_institution_deregistration`(runtime/src/configs/mod.rs)逐字节一致——
/// `[u8;32]`/`&[u8;32]` 无长度前缀、`u8` 1 字节、`&[u8]` 带 Compact 长度前缀。
/// target_account 与 scope 入签名,杜绝换账户/换范围/换机构重放。
fn deregistration_payload_digest(
    genesis_hash: &[u8; 32],
    scope: u8,
    cid_number: &[u8],
    account_name: &[u8],
    target_account: &[u8; 32],
    deregister_nonce: &[u8],
    issuer_cid_number: &[u8],
    issuer_main_account: &[u8; 32],
    signer_pubkey: &[u8; 32],
) -> [u8; 32] {
    let payload = (
        GMB,
        OP_SIGN_DEREGISTER,
        genesis_hash,
        scope,
        cid_number,
        account_name,
        target_account,
        deregister_nonce,
        issuer_cid_number,
        issuer_main_account,
        signer_pubkey,
    );
    blake2_256(&payload.encode())
}

/// 中文注释:签发机构/账户注销凭证(对称 `build_institution_registration_credential`)。
/// scope=`SCOPE_INSTITUTION`(0,关主账户=注销整机构)/ `SCOPE_ACCOUNT`(1,只关该非主账户)。
/// 由注册局管理员动作(PasskeyChallenge 最严档)校验通过后调用;机构管理员持此凭证冷签 propose_close。
pub(crate) fn build_institution_deregistration_credential(
    state: &AppState,
    scope: u8,
    cid_number: &str,
    account_name: &str,
    target_account: &[u8; 32],
    deregister_nonce: String,
) -> Result<RuntimeInstitutionDeregistrationCredential, String> {
    if cid_number.trim().is_empty() {
        return Err("cid_number is required".to_string());
    }
    if account_name.trim().is_empty() {
        return Err("account_name is required".to_string());
    }
    if deregister_nonce.trim().is_empty() {
        return Err("deregister_nonce is required".to_string());
    }
    let genesis_hash = resolve_chain_genesis_hash()?;
    let signing_ctx = runtime_signing_context(None, None)?;
    let payload_digest = deregistration_payload_digest(
        &genesis_hash,
        scope,
        cid_number.trim().as_bytes(),
        account_name.trim().as_bytes(),
        target_account,
        deregister_nonce.trim().as_bytes(),
        signing_ctx.issuer_cid_number.as_bytes(),
        &signing_ctx.issuer_main_account,
        &signing_ctx.signer_pubkey,
    );
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimeInstitutionDeregistrationCredential {
        genesis_hash: hex::encode(genesis_hash),
        scope,
        cid_number: cid_number.trim().to_string(),
        account_name: account_name.trim().to_string(),
        target_account: format!("0x{}", hex::encode(target_account)),
        deregister_nonce,
        issuer_cid_number: signing_ctx.issuer_cid_number,
        issuer_main_account: signing_ctx.issuer_main_account_hex,
        signer_pubkey: signing_ctx.signer_pubkey_hex,
        signature,
        payload_digest: hex::encode(payload_digest),
        meta: runtime_signature_meta(state),
    })
}

fn runtime_signature_meta(_state: &AppState) -> RuntimeSignatureMeta {
    // 中文注释:metadata 只用于排查签发来源;链上只信任 payload 中的
    // issuer_cid_number / issuer_main_account / signer_pubkey。
    RuntimeSignatureMeta {
        key_id: "cid-admins-v1".to_string(),
        key_version: "v1".to_string(),
        alg: "sr25519".to_string(),
    }
}

fn runtime_signing_context(
    scope_province_override: Option<&str>,
    scope_city_override: Option<&str>,
) -> Result<RuntimeSigningContext, String> {
    let issuer_cid_number = std::env::var("CID_RUNTIME_ISSUER_CID_NUMBER")
        .map_err(|_| "CID_RUNTIME_ISSUER_CID_NUMBER not set".to_string())?
        .trim()
        .to_string();
    if issuer_cid_number.is_empty() {
        return Err("CID_RUNTIME_ISSUER_CID_NUMBER is empty".to_string());
    }
    let issuer_main_account_raw = std::env::var("CID_RUNTIME_ISSUER_MAIN_ACCOUNT")
        .map_err(|_| "CID_RUNTIME_ISSUER_MAIN_ACCOUNT not set".to_string())?;
    let issuer_main_account = parse_sr25519_pubkey_bytes(issuer_main_account_raw.as_str())
        .ok_or_else(|| {
            "CID_RUNTIME_ISSUER_MAIN_ACCOUNT must be a 32-byte account hex".to_string()
        })?;
    let signer_pubkey_raw = std::env::var("CID_RUNTIME_SIGNER_PUBKEY")
        .map_err(|_| "CID_RUNTIME_SIGNER_PUBKEY not set".to_string())?;
    let signer_pubkey =
        parse_sr25519_pubkey_bytes(signer_pubkey_raw.as_str()).ok_or_else(|| {
            "CID_RUNTIME_SIGNER_PUBKEY must be a 32-byte sr25519 pubkey hex".to_string()
        })?;
    let default_scope_province =
        std::env::var("CID_RUNTIME_SCOPE_PROVINCE_NAME").unwrap_or_else(|_| "全国".to_string());
    let scope_province_name = scope_province_override
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| default_scope_province.trim().to_string());
    if scope_province_name.is_empty() {
        return Err("CID_RUNTIME_SCOPE_PROVINCE_NAME is empty".to_string());
    }
    let default_scope_city = std::env::var("CID_RUNTIME_SCOPE_CITY_NAME").unwrap_or_default();
    let scope_city_name = scope_city_override
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| default_scope_city.trim().to_string());
    Ok(RuntimeSigningContext {
        issuer_cid_number,
        issuer_main_account,
        issuer_main_account_hex: format!("0x{}", hex::encode(issuer_main_account)),
        signer_pubkey,
        signer_pubkey_hex: format!("0x{}", hex::encode(signer_pubkey)),
        scope_province_name,
        scope_city_name,
    })
}

fn is_production_mode() -> bool {
    std::env::var("CID_ENV")
        .ok()
        .map(|v| v.eq_ignore_ascii_case("prod") || v.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

fn normalize_and_parse_account_id32(account_pubkey: &str) -> Result<(String, [u8; 32]), String> {
    let normalized = normalize_account_pubkey(account_pubkey)
        .ok_or_else(|| "account_pubkey is invalid".to_string())?;
    let who = parse_sr25519_pubkey_bytes(normalized.as_str())
        .ok_or_else(|| "account_pubkey is invalid".to_string())?;
    Ok((normalized, who))
}

pub(crate) fn normalize_account_pubkey(account_pubkey: &str) -> Option<String> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(account_pubkey) {
        return Some(hex_pubkey);
    }
    let bytes = parse_sr25519_pubkey_bytes(account_pubkey)?;
    Some(format!("0x{}", hex::encode(bytes)))
}

pub(crate) fn is_chain_runtime_config_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    message.contains("CID_RUNTIME_")
        || message.contains("CID_CHAIN_GENESIS_HASH")
        || message.contains("CID_SIGNING_SEED_HEX")
        || lower.contains("genesis hash")
        || lower.contains("trusted chain")
}

fn resolve_chain_genesis_hash() -> Result<[u8; 32], String> {
    if let Some(cached) = CHAIN_GENESIS_HASH.get() {
        return Ok(*cached);
    }
    // 开发环境允许通过环境变量覆盖，生产环境必须依赖启动时完成的白名单校验结果。
    if is_production_mode() {
        return Err(
            "production genesis hash not initialized: call init_genesis_hash_from_chain() at startup"
                .to_string(),
        );
    }
    if let Ok(raw) = std::env::var("CID_CHAIN_GENESIS_HASH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let parsed = parse_hex_hash32(trimmed)
                .map_err(|_| "CID_CHAIN_GENESIS_HASH must be 32-byte hex".to_string())?;
            let _ = CHAIN_GENESIS_HASH.set(parsed);
            return Ok(CHAIN_GENESIS_HASH.get().copied().unwrap_or(parsed));
        }
    }
    Err("genesis hash not available: configure CID_CHAIN_GENESIS_HASH or call init_genesis_hash_from_chain() at startup".to_string())
}

fn trusted_production_chain_by_hash(
    hash: &[u8; 32],
) -> Result<Option<TrustedProductionChain>, String> {
    for chain in TRUSTED_PRODUCTION_CHAINS {
        let parsed = parse_hex_hash32(chain.genesis_hash_hex).map_err(|_| {
            format!(
                "trusted production chain `{}` has invalid genesis hash literal",
                chain.name
            )
        })?;
        if &parsed == hash {
            return Ok(Some(*chain));
        }
    }
    Ok(None)
}

async fn fetch_chain_genesis_hash_from_rpc() -> Result<[u8; 32], String> {
    if let Ok(http_url) = super::chain_url::chain_http_url() {
        return fetch_chain_genesis_hash_via_http(http_url.as_str()).await;
    }
    let ws_url = super::chain_url::chain_ws_url()?;
    fetch_chain_genesis_hash_via_ws(ws_url.as_str()).await
}

#[derive(Deserialize)]
struct ChainGetBlockHashResponse {
    result: Option<String>,
    error: Option<serde_json::Value>,
}

async fn fetch_chain_genesis_hash_via_http(http_url: &str) -> Result<[u8; 32], String> {
    let client = reqwest::Client::new();
    let response = client
        .post(http_url)
        .json(&serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "chain_getBlockHash",
            "params": [0]
        }))
        .send()
        .await
        .map_err(|e| format!("connect chain http rpc for genesis hash failed: {e}"))?;
    let status = response.status();
    let payload: ChainGetBlockHashResponse = response
        .json()
        .await
        .map_err(|e| format!("decode chain http rpc genesis hash response failed: {e}"))?;
    if !status.is_success() {
        return Err(format!("chain http rpc returned status {status}"));
    }
    if let Some(error) = payload.error {
        return Err(format!("chain http rpc returned error: {error}"));
    }
    let Some(hash_hex) = payload.result else {
        return Err("chain http rpc missing result for genesis hash".to_string());
    };
    parse_hex_hash32(hash_hex.as_str())
}

async fn fetch_chain_genesis_hash_via_ws(ws_url: &str) -> Result<[u8; 32], String> {
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url)
        .await
        .map_err(|e| format!("connect chain websocket for genesis hash failed: {e}"))?;
    Ok(client.genesis_hash().0)
}

/// 启动时从区块链 RPC 获取创世哈希并缓存。
/// 调用一次后，之后的 resolve_chain_genesis_hash() 直接返回缓存值。
pub(crate) async fn init_genesis_hash_from_chain() -> Result<(), String> {
    if CHAIN_GENESIS_HASH.get().is_some() {
        return Ok(());
    }
    if is_production_mode() {
        if TRUSTED_PRODUCTION_CHAINS.is_empty() {
            return Err(
                "production trusted chain whitelist is empty: add chain genesis hashes to TRUSTED_PRODUCTION_CHAINS"
                    .to_string(),
            );
        }
        let hash_bytes = fetch_chain_genesis_hash_from_rpc().await?;
        let Some(chain) = trusted_production_chain_by_hash(&hash_bytes)? else {
            return Err(format!(
                "connected chain genesis hash 0x{} is not in TRUSTED_PRODUCTION_CHAINS",
                hex::encode(hash_bytes)
            ));
        };
        let _ = CHAIN_GENESIS_HASH.set(hash_bytes);
        tracing::info!(
            trusted_chain = chain.name,
            genesis_hash = %format!("0x{}", hex::encode(hash_bytes)),
            "validated production chain genesis hash from RPC"
        );
        return Ok(());
    }

    // 开发环境允许本地显式覆盖，否则启动时自动从链上获取。
    if let Ok(raw) = std::env::var("CID_CHAIN_GENESIS_HASH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let parsed = parse_hex_hash32(trimmed)
                .map_err(|_| "CID_CHAIN_GENESIS_HASH must be 32-byte hex".to_string())?;
            let _ = CHAIN_GENESIS_HASH.set(parsed);
            tracing::info!(
                genesis_hash = %format!("0x{}", hex::encode(parsed)),
                "loaded development genesis hash from environment"
            );
            return Ok(());
        }
    }
    let hash_bytes = fetch_chain_genesis_hash_from_rpc().await?;
    let _ = CHAIN_GENESIS_HASH.set(hash_bytes);
    tracing::info!(
        genesis_hash = %format!("0x{}", hex::encode(hash_bytes)),
        "fetched development chain genesis hash from RPC"
    );
    Ok(())
}

fn parse_hex_hash32(raw: &str) -> Result<[u8; 32], String> {
    let trimmed = raw.trim();
    let no_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if no_prefix.len() != 64 || !no_prefix.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid hash hex".to_string());
    }
    let bytes = hex::decode(no_prefix).map_err(|_| "invalid hash hex".to_string())?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| "invalid hash length".to_string())?;
    Ok(arr)
}

fn sign_runtime_digest(_state: &AppState, digest: &[u8; 32]) -> Result<String, String> {
    // ADR-008 Phase 23e:CID main signer 直接从环境变量 CID_SIGNING_SEED_HEX
    // 派生(由 `crate::crypto::sr25519` helper 加载),AppState 不再持有 seed。
    // 由 build_vote_credential / build_population_snapshot_credential 调用(全国级签名)。
    let seed_hex = std::env::var("CID_SIGNING_SEED_HEX")
        .map_err(|_| "CID_SIGNING_SEED_HEX not set".to_string())?;
    let signing_key = resolve_signing_keypair(seed_hex.as_str())?;
    let signature = signing_key.sign(digest);
    Ok(hex::encode(signature.0))
}

fn resolve_signing_keypair(seed_text: &str) -> Result<Arc<Sr25519Pair>, String> {
    let cache = SIGNING_KEY_CACHE.get_or_init(|| RwLock::new(None));
    {
        let guard = cache
            .read()
            .map_err(|_| "signing key cache read lock poisoned".to_string())?;
        if let Some(cached) = guard.as_ref() {
            if cached.seed_hex.expose_secret() == seed_text {
                return Ok(Arc::clone(&cached.keypair));
            }
        }
    }

    let loaded = Arc::new(crate::crypto::sr25519::try_load_signing_key_from_seed(
        seed_text,
    )?);
    let mut guard = cache
        .write()
        .map_err(|_| "signing key cache write lock poisoned".to_string())?;
    if let Some(cached) = guard.as_ref() {
        if cached.seed_hex.expose_secret() == seed_text {
            return Ok(Arc::clone(&cached.keypair));
        }
    }
    *guard = Some(CachedSigningKey {
        seed_hex: SensitiveSeed::new(seed_text.to_string()),
        keypair: Arc::clone(&loaded),
    });
    Ok(loaded)
}

fn blake2_256(input: &[u8]) -> [u8; 32] {
    let mut output = [0_u8; 32];
    let mut hasher = Blake2bVar::new(32).expect("invalid blake2 output length");
    hasher.update(input);
    hasher
        .finalize_variable(&mut output)
        .expect("finalize blake2_256 failed");
    output
}

#[cfg(test)]
mod tests {
    use super::{
        deregistration_payload_digest, is_production_mode, parse_hex_hash32,
        trusted_production_chain_by_hash,
    };

    #[test]
    fn deregistration_payload_digest_is_byte_locked() {
        // 中文注释:golden 测试锁死注销凭证 payload 的 SCALE 字节编码。
        // 该摘要口径必须与链端 verify_institution_deregistration(runtime configs)逐字节一致;
        // 任何字段类型/顺序漂移都会改变摘要,此断言立即红。
        let genesis_hash = [0x11u8; 32];
        let target = [0x22u8; 32];
        let issuer_main = [0x33u8; 32];
        let signer = [0x44u8; 32];
        let digest = deregistration_payload_digest(
            &genesis_hash,
            0u8, // SCOPE_INSTITUTION
            b"AH001-ZF001-123456789-2026",
            "主账户".as_bytes(),
            &target,
            b"dereg-nonce-1",
            b"ZS001-GZF0P-249474503-2026",
            &issuer_main,
            &signer,
        );
        // golden 值:GMB/OP_SIGN_DEREGISTER + 上述固定输入的 SCALE 编码 blake2_256。
        // 已逐字段核对链端 verify_institution_deregistration 的 tuple 类型/顺序一致
        // (AccountId32=[u8;32]、H256=[u8;32] 均 32 字节无前缀;cid/account_name/nonce/issuer &[u8] 均 Compact 前缀)。
        assert_eq!(
            hex::encode(digest),
            "137304f0e5207c3ddd6116eef9e1f42660bec15831b3f4c6b30a2c99bee814a1",
            "注销凭证 payload 字节编码漂移(与链端口径不一致)"
        );
    }

    #[test]
    fn parse_hex_hash32_accepts_prefixed_hash() {
        let parsed = parse_hex_hash32(&format!("0x{}", "11".repeat(32))).unwrap();
        assert_eq!(parsed, [0x11; 32]);
    }

    #[test]
    fn trusted_production_chain_lookup_returns_none_for_unknown_hash() {
        let result = trusted_production_chain_by_hash(&[0x22; 32]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn production_mode_detects_prod_env() {
        let previous = std::env::var("CID_ENV").ok();
        std::env::set_var("CID_ENV", "prod");
        assert!(is_production_mode());
        if let Some(value) = previous {
            std::env::set_var("CID_ENV", value);
        } else {
            std::env::remove_var("CID_ENV");
        }
    }
}
