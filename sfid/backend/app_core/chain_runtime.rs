use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};
use std::sync::{Arc, OnceLock, RwLock};
use subxt::{OnlineClient, PolkadotConfig};

use crate::*;

// 中文注释（适用于本文件所有 DUOQIAN_DOMAIN + OP_SIGN_* 常量）：
// 必须使用 [u8; N] 数组类型 + u8，与链端 verifier 里
// `primitives::core_const::DUOQIAN_DOMAIN` (= `*b"DUOQIAN_V1"` 即 [u8; 10])
// + `OP_SIGN_*` u8 完全对齐。SCALE 编码下：
//   [u8; N] / &[u8; N]  →  N 字节，无长度前缀
//   u8                   →  1 字节
//   &[u8] / Vec<u8>     →  Compact(N) ++ N 字节，多 1~4 字节长度前缀
// 任何一个 domain 写成 &[u8] 都会导致 message 与链端不一致 → blake2_256 不同
// → sr25519 verify 失败 → 链端返回 InvalidSfidXxxSignature。
// 历史教训：INSTITUTION_DOMAIN 曾被错误声明为 &[u8]，导致 register_sfid_institution
// 长期 InvalidSfidInstitutionSignature。修复见 ADR
// `04-decisions/sfid/2026-04-07-subxt-0.43-pow-chain-quirks.md`。
// 中文注释：2026-04-20 统一 DUOQIAN_V1 单域方案：domain 字节 10B
// (b"DUOQIAN_V1") + 1B op_tag 区分子命名空间。按 op_tag 分别标识 BIND / VOTE /
// POP / INST。与 citizenchain `primitives::core_const::{DUOQIAN_DOMAIN, OP_SIGN_*}`
// 严格对齐。
const DUOQIAN_DOMAIN: [u8; 10] = *b"DUOQIAN_V1";
const OP_SIGN_BIND: u8 = 0x10;
const OP_SIGN_VOTE: u8 = 0x11;
const OP_SIGN_POP: u8 = 0x12;
const OP_SIGN_INST: u8 = 0x13;
#[allow(dead_code)]
pub(crate) const POPULATION_DOMAIN_STR: &str = "DUOQIAN_V1";
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
pub(crate) struct RuntimeBindCredential {
    pub(crate) genesis_hash: String,
    pub(crate) who: String,
    pub(crate) binding_id: String,
    pub(crate) bind_nonce: String,
    pub(crate) signature: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimeVoteCredential {
    pub(crate) genesis_hash: String,
    pub(crate) who: String,
    pub(crate) binding_id: String,
    pub(crate) proposal_id: u64,
    pub(crate) vote_nonce: String,
    pub(crate) signature: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimePopulationSnapshotCredential {
    pub(crate) who: String,
    pub(crate) eligible_total: u64,
    pub(crate) snapshot_nonce: String,
    pub(crate) signature: String,
    pub(crate) genesis_hash: String,
    pub(crate) payload_digest: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimeInstitutionRegistrationInfoCredential {
    pub(crate) genesis_hash: String,
    pub(crate) register_nonce: String,
    pub(crate) province: String,
    pub(crate) signer_admin_pubkey: String,
    pub(crate) signature: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[allow(dead_code)]
pub(crate) fn build_bind_credential(
    state: &AppState,
    account_pubkey: &str,
    binding_seed: &str,
    bind_nonce: String,
) -> Result<RuntimeBindCredential, String> {
    if bind_nonce.trim().is_empty() {
        return Err("bind nonce is required".to_string());
    }
    if binding_seed.trim().is_empty() {
        return Err("binding seed is required".to_string());
    }
    let (normalized_who, who) = normalize_and_parse_account_id32(account_pubkey)?;
    let genesis_hash = resolve_chain_genesis_hash()?;
    let binding_id = blake2_256(binding_seed.as_bytes());
    let payload = (
        DUOQIAN_DOMAIN,
        OP_SIGN_BIND,
        genesis_hash,
        who,
        binding_id,
        bind_nonce.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimeBindCredential {
        genesis_hash: hex::encode(genesis_hash),
        who: normalized_who,
        binding_id: hex::encode(binding_id),
        bind_nonce,
        signature,
        meta: runtime_signature_meta(state),
    })
}

/// 用省级签名密钥构建 bind_sfid 链上凭证（推链绑定用）。
///
/// 与 `build_bind_credential` 的区别：签名用省级 Pair 而非 SFID MAIN。
pub(crate) fn build_bind_credential_with_province(
    state: &AppState,
    account_pubkey: &str,
    binding_seed: &str,
    bind_nonce: String,
    province_pair: &sp_core::sr25519::Pair,
) -> Result<RuntimeBindCredential, String> {
    if bind_nonce.trim().is_empty() {
        return Err("bind nonce is required".to_string());
    }
    if binding_seed.trim().is_empty() {
        return Err("binding seed is required".to_string());
    }
    let (normalized_who, who) = normalize_and_parse_account_id32(account_pubkey)?;
    let genesis_hash = resolve_chain_genesis_hash()?;
    let binding_id = blake2_256(binding_seed.as_bytes());
    let payload = (
        DUOQIAN_DOMAIN,
        OP_SIGN_BIND,
        genesis_hash,
        who,
        binding_id,
        bind_nonce.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = province_pair.sign(&payload_digest).0;
    Ok(RuntimeBindCredential {
        genesis_hash: hex::encode(genesis_hash),
        who: normalized_who,
        binding_id: hex::encode(binding_id),
        bind_nonce,
        signature: hex::encode(signature),
        meta: runtime_signature_meta(state),
    })
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
    let payload = (
        DUOQIAN_DOMAIN,
        OP_SIGN_VOTE,
        genesis_hash,
        who,
        binding_id,
        proposal_id,
        vote_nonce.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimeVoteCredential {
        genesis_hash: hex::encode(genesis_hash),
        who: normalized_who,
        binding_id: hex::encode(binding_id),
        proposal_id,
        vote_nonce,
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
    let payload = (
        DUOQIAN_DOMAIN,
        OP_SIGN_POP,
        genesis_hash,
        who,
        eligible_total,
        snapshot_nonce.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimePopulationSnapshotCredential {
        who: normalized_who,
        eligible_total,
        snapshot_nonce,
        signature,
        genesis_hash: hex::encode(genesis_hash),
        payload_digest: hex::encode(payload_digest),
        meta: runtime_signature_meta(state),
    })
}

/// 用**省级签名密钥**签发机构注册信息凭证。
///
/// 中文注释:本函数只签“链端注册真正需要的机构信息”:
/// `sfid_id / institution_name / account_names`。`province`、`signer_admin_pubkey`、
/// `genesis_hash`、`register_nonce` 是验签与防重放安全字段,不是业务注册字段。
///
/// payload 字节布局必须与链端 `sfid-system` verifier 保持一致:
/// `blake2_256(scale_encode((
///   DUOQIAN_DOMAIN, OP_SIGN_INST, genesis_hash,
///   sfid_id, institution_name, account_names,
///   register_nonce, province, signer_admin_pubkey
/// )))`
///
/// 任何字段顺序、编码类型或名称列表排序变更,都必须同步改链端 verifier,
/// 否则 sr25519 verify 必败。
pub(crate) fn build_institution_registration_info_credential(
    state: &AppState,
    sfid_id: &str,
    institution_name: &str,
    account_names: &[String],
    register_nonce: String,
    province: &str,
    signer_admin_pubkey: [u8; 32],
    province_pair: &sp_core::sr25519::Pair,
) -> Result<RuntimeInstitutionRegistrationInfoCredential, String> {
    if sfid_id.trim().is_empty() {
        return Err("sfid_id is required".to_string());
    }
    if institution_name.trim().is_empty() {
        return Err("institution_name is required".to_string());
    }
    if account_names.is_empty() {
        return Err("account_names is required".to_string());
    }
    if register_nonce.trim().is_empty() {
        return Err("register_nonce is required".to_string());
    }
    if province.trim().is_empty() {
        return Err("province is required".to_string());
    }
    let genesis_hash = resolve_chain_genesis_hash()?;
    let account_name_bytes: Vec<Vec<u8>> = account_names
        .iter()
        .map(|name| name.as_bytes().to_vec())
        .collect();
    let payload = (
        DUOQIAN_DOMAIN,
        OP_SIGN_INST,
        genesis_hash,
        sfid_id.as_bytes(),
        institution_name.as_bytes(),
        account_name_bytes,
        register_nonce.as_bytes(),
        province.as_bytes(),
        signer_admin_pubkey,
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = province_pair.sign(&payload_digest).0;
    Ok(RuntimeInstitutionRegistrationInfoCredential {
        genesis_hash: hex::encode(genesis_hash),
        register_nonce,
        province: province.to_string(),
        signer_admin_pubkey: format!("0x{}", hex::encode(signer_admin_pubkey)),
        signature: hex::encode(signature),
        meta: runtime_signature_meta(state),
    })
}

#[allow(dead_code)]
pub(crate) fn current_chain_genesis_hash_hex() -> Result<String, String> {
    resolve_chain_genesis_hash().map(hex::encode)
}

fn runtime_signature_meta(_state: &AppState) -> RuntimeSignatureMeta {
    // ADR-008 Phase 23e:AppState 不再持有 key_id / key_version / key_alg。
    // 凭证 metadata 退化成固定标识(消费方只用 alg 校验签名算法,前两个字段
    // 已无业务消费方,保留固定串避免 wire format 变更。)
    RuntimeSignatureMeta {
        key_id: "sfid-sheng-v1".to_string(),
        key_version: "v1".to_string(),
        alg: "sr25519".to_string(),
    }
}

fn is_production_mode() -> bool {
    std::env::var("SFID_ENV")
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
    if let Ok(raw) = std::env::var("SFID_CHAIN_GENESIS_HASH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let parsed = parse_hex_hash32(trimmed)
                .map_err(|_| "SFID_CHAIN_GENESIS_HASH must be 32-byte hex".to_string())?;
            let _ = CHAIN_GENESIS_HASH.set(parsed);
            return Ok(CHAIN_GENESIS_HASH.get().copied().unwrap_or(parsed));
        }
    }
    Err("genesis hash not available: configure SFID_CHAIN_GENESIS_HASH or call init_genesis_hash_from_chain() at startup".to_string())
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
/// 调用一次后，后续 resolve_chain_genesis_hash() 直接返回缓存值。
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
    if let Ok(raw) = std::env::var("SFID_CHAIN_GENESIS_HASH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let parsed = parse_hex_hash32(trimmed)
                .map_err(|_| "SFID_CHAIN_GENESIS_HASH must be 32-byte hex".to_string())?;
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
    // ADR-008 Phase 23e:SFID main signer 直接从环境变量 SFID_SIGNING_SEED_HEX
    // 派生(由 `crate::crypto::sr25519` helper 加载),AppState 不再持有 seed。
    // 本函数仅由 `build_*_credential`(非 *_with_province 变体)调用,后者已是
    // dead code(全部 caller 改走 *_with_province 用省级签名密钥);保留只为
    // 让 fallback path(全国级凭证)继续可用。
    let seed_hex = std::env::var("SFID_SIGNING_SEED_HEX")
        .map_err(|_| "SFID_SIGNING_SEED_HEX not set".to_string())?;
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
    use super::{is_production_mode, parse_hex_hash32, trusted_production_chain_by_hash};

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
        let previous = std::env::var("SFID_ENV").ok();
        std::env::set_var("SFID_ENV", "prod");
        assert!(is_production_mode());
        if let Some(value) = previous {
            std::env::set_var("SFID_ENV", value);
        } else {
            std::env::remove_var("SFID_ENV");
        }
    }
}
