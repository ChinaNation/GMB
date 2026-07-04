use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use codec::{Decode, Encode};
use primitives::core_const::{GMB, OP_SIGN_DEREGISTER, OP_SIGN_INST};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};
use std::{
    collections::BTreeMap,
    hash::Hasher,
    sync::{Arc, OnceLock, RwLock},
};
use subxt::{dynamic, OnlineClient, PolkadotConfig};
use twox_hash::XxHash64;

use crate::auth::login::parse_sr25519_pubkey_bytes;
use crate::*;

// 本文件所有 GMB + OP_SIGN_* 均直接来自
// `primitives::core_const`。SCALE 编码下：
//   [u8; N] / &[u8; N]  →  N 字节，无长度前缀
//   u8                   →  1 字节
//   &[u8] / Vec<u8>     →  Compact(N) ++ N 字节，多 1~4 字节长度前缀
// 任何一个 domain 写成 &[u8] 都会导致 message 与链端不一致 → blake2_256 不同
// → sr25519 verify 失败 → 链端返回对应的签名错误。
// 历史教训：INSTITUTION_DOMAIN 曾被错误声明为 &[u8]，导致机构注册签名长期无法通过。
static CHAIN_GENESIS_HASH: OnceLock<[u8; 32]> = OnceLock::new();
static SIGNING_KEY_CACHE: OnceLock<RwLock<Option<CachedSigningKey>>> = OnceLock::new();
const TRUSTED_PRODUCTION_CHAINS: &[TrustedProductionChain] = &[
    // 正式链创世哈希在这里做源码级白名单绑定；新增正式链时只允许在此处追加。
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

pub(crate) fn build_institution_registration_credential(
    state: &AppState,
    cid_number: &str,
    cid_full_name: &str,
    cid_short_name: &str,
    account_names: &[String],
    register_nonce: String,
    scope_province_name: &str,
    scope_city_name: &str,
    town_code: &str,
) -> Result<RuntimeInstitutionRegistrationCredential, String> {
    if cid_number.trim().is_empty() {
        return Err("cid_number is required".to_string());
    }
    if cid_full_name.trim().is_empty() {
        return Err("cid_full_name is required".to_string());
    }
    if cid_short_name.trim().is_empty() {
        return Err("cid_short_name is required".to_string());
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
    // 字段顺序必须与 RuntimeCidInstitutionVerifier 完全一致:
    // genesis_hash + cid_number + cid_full_name + cid_short_name + account_names[]
    // + nonce + 签发机构 + 作用域 + town_code。
    let payload = (
        GMB,
        OP_SIGN_INST,
        genesis_hash,
        cid_number.trim().as_bytes(),
        cid_full_name.trim().as_bytes(),
        cid_short_name.trim().as_bytes(),
        &account_name_payload,
        register_nonce.trim().as_bytes(),
        signing_ctx.issuer_cid_number.as_bytes(),
        &signing_ctx.issuer_main_account,
        &signing_ctx.signer_pubkey,
        signing_ctx.scope_province_name.as_bytes(),
        signing_ctx.scope_city_name.as_bytes(),
        town_code.trim().as_bytes(),
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

/// 注销凭证签名 payload 的 blake2_256 摘要(纯函数,便于 golden 测试锁字节)。
///
/// **铁律**:元素顺序与 SCALE 类型必须与链端 `verify_institution_deregistration`
/// (runtime/src/configs/mod.rs,被 public-manage/private-manage close 消费)逐字节一致——
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

/// 签发机构/账户注销凭证(对称 `build_institution_registration_credential`)。
/// scope=`SCOPE_INSTITUTION`(0,关主账户=注销整机构)/ `SCOPE_ACCOUNT`(1,只关该非主账户)。
/// 由注册局管理员动作(冷签特殊档)校验通过后调用;机构管理员持此凭证冷签 propose_close。
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
    // metadata 只用于排查签发来源;链上只信任 payload 中的
    // issuer_cid_number / issuer_main_account / signer_pubkey。
    RuntimeSignatureMeta {
        key_id: "onchina-admins-v1".to_string(),
        key_version: "v1".to_string(),
        alg: "sr25519".to_string(),
    }
}

fn runtime_signing_context(
    scope_province_override: Option<&str>,
    scope_city_override: Option<&str>,
) -> Result<RuntimeSigningContext, String> {
    let issuer_cid_number = std::env::var("ONCHAIN_CREDENTIAL_ISSUER_CID_NUMBER")
        .map_err(|_| "ONCHAIN_CREDENTIAL_ISSUER_CID_NUMBER not set".to_string())?
        .trim()
        .to_string();
    if issuer_cid_number.is_empty() {
        return Err("ONCHAIN_CREDENTIAL_ISSUER_CID_NUMBER is empty".to_string());
    }
    let issuer_main_account_raw = std::env::var("ONCHAIN_CREDENTIAL_ISSUER_MAIN_ACCOUNT")
        .map_err(|_| "ONCHAIN_CREDENTIAL_ISSUER_MAIN_ACCOUNT not set".to_string())?;
    let issuer_main_account = parse_sr25519_pubkey_bytes(issuer_main_account_raw.as_str())
        .ok_or_else(|| {
            "ONCHAIN_CREDENTIAL_ISSUER_MAIN_ACCOUNT must be a 32-byte account hex".to_string()
        })?;
    let signer_pubkey_raw = std::env::var("ONCHAIN_CREDENTIAL_SIGNER_PUBKEY")
        .map_err(|_| "ONCHAIN_CREDENTIAL_SIGNER_PUBKEY not set".to_string())?;
    let signer_pubkey =
        parse_sr25519_pubkey_bytes(signer_pubkey_raw.as_str()).ok_or_else(|| {
            "ONCHAIN_CREDENTIAL_SIGNER_PUBKEY must be a 32-byte sr25519 pubkey hex".to_string()
        })?;
    let default_scope_province = std::env::var("ONCHAIN_CREDENTIAL_SCOPE_PROVINCE_NAME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let scope_province_name = scope_province_override
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or(default_scope_province)
        .ok_or_else(|| "ONCHAIN_CREDENTIAL_SCOPE_PROVINCE_NAME not set".to_string())?;
    let default_scope_city =
        std::env::var("ONCHAIN_CREDENTIAL_SCOPE_CITY_NAME").unwrap_or_default();
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
    std::env::var("ONCHINA_ENV")
        .ok()
        .map(|v| v.eq_ignore_ascii_case("prod") || v.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
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
    message.contains("ONCHAIN_CREDENTIAL_")
        || message.contains("ONCHAIN_GENESIS_HASH")
        || message.contains("ONCHINA_SIGNING_SEED_HEX")
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
    if let Ok(raw) = std::env::var("ONCHAIN_GENESIS_HASH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let parsed = parse_hex_hash32(trimmed)
                .map_err(|_| "ONCHAIN_GENESIS_HASH must be 32-byte hex".to_string())?;
            let _ = CHAIN_GENESIS_HASH.set(parsed);
            return Ok(CHAIN_GENESIS_HASH.get().copied().unwrap_or(parsed));
        }
    }
    Err("genesis hash not available: configure ONCHAIN_GENESIS_HASH or call init_genesis_hash_from_chain() at startup".to_string())
}

/// 返回已经通过启动校验缓存的链创世哈希。
pub(crate) fn cached_chain_genesis_hash_hex() -> Option<String> {
    CHAIN_GENESIS_HASH
        .get()
        .map(|hash| format!("0x{}", hex::encode(hash)))
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

#[derive(Deserialize)]
struct ChainRpcValueResponse {
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub(crate) struct ChainFinalizedAnchor {
    pub(crate) block_hash: String,
    pub(crate) block_number: i64,
}

#[derive(Deserialize)]
struct ChainHeaderResponse {
    result: Option<ChainHeader>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChainHeader {
    number: String,
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

async fn fetch_finalized_head_via_http(
    client: &reqwest::Client,
    http_url: &str,
) -> Result<String, String> {
    let response = client
        .post(http_url)
        .json(&serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "chain_getFinalizedHead",
            "params": []
        }))
        .send()
        .await
        .map_err(|e| format!("connect chain http rpc for finalized head failed: {e}"))?;
    let status = response.status();
    let payload: ChainGetBlockHashResponse = response
        .json()
        .await
        .map_err(|e| format!("decode chain http rpc finalized head response failed: {e}"))?;
    if !status.is_success() {
        return Err(format!("chain http rpc returned status {status}"));
    }
    if let Some(error) = payload.error {
        return Err(format!("chain http rpc returned error: {error}"));
    }
    payload
        .result
        .ok_or_else(|| "chain http rpc missing finalized head result".to_string())
}

async fn fetch_header_via_http(
    client: &reqwest::Client,
    http_url: &str,
    block_hash: &str,
) -> Result<ChainHeader, String> {
    let response = client
        .post(http_url)
        .json(&serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "chain_getHeader",
            "params": [block_hash]
        }))
        .send()
        .await
        .map_err(|e| format!("connect chain http rpc for header failed: {e}"))?;
    let status = response.status();
    let payload: ChainHeaderResponse = response
        .json()
        .await
        .map_err(|e| format!("decode chain http rpc header response failed: {e}"))?;
    if !status.is_success() {
        return Err(format!("chain http rpc returned status {status}"));
    }
    if let Some(error) = payload.error {
        return Err(format!("chain http rpc returned error: {error}"));
    }
    payload
        .result
        .ok_or_else(|| "chain http rpc missing header result".to_string())
}

fn parse_header_number(raw: &str) -> Result<i64, String> {
    let hex = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .ok_or_else(|| "chain header number must be hex".to_string())?;
    i64::from_str_radix(hex, 16).map_err(|e| format!("parse chain header number failed: {e}"))
}

/// 读取当前 finalized head 作为链投影版本锚点。
pub(crate) async fn fetch_finalized_anchor() -> Result<ChainFinalizedAnchor, String> {
    let http_url = super::chain_url::chain_http_url()?;
    let client = reqwest::Client::new();
    let block_hash = fetch_finalized_head_via_http(&client, http_url.as_str()).await?;
    let header = fetch_header_via_http(&client, http_url.as_str(), block_hash.as_str()).await?;
    let block_number = parse_header_number(header.number.as_str())?;
    Ok(ChainFinalizedAnchor {
        block_hash,
        block_number,
    })
}

fn clean_account_hex_key(account_hex: &str) -> String {
    let trimmed = account_hex.trim();
    trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed)
        .to_ascii_lowercase()
}

fn twox_128(input: &[u8]) -> [u8; 16] {
    let mut h0 = XxHash64::with_seed(0);
    h0.write(input);
    let r0 = h0.finish();

    let mut h1 = XxHash64::with_seed(1);
    h1.write(input);
    let r1 = h1.finish();

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&r0.to_le_bytes());
    out[8..].copy_from_slice(&r1.to_le_bytes());
    out
}

fn blake2b_128(input: &[u8]) -> [u8; 16] {
    let mut output = [0_u8; 16];
    let mut hasher = Blake2bVar::new(16).expect("invalid blake2 output length");
    hasher.update(input);
    hasher
        .finalize_variable(&mut output)
        .expect("finalize blake2b_128 failed");
    output
}

fn system_account_storage_key(account_id: &[u8; 32]) -> String {
    let pallet_hash = twox_128(b"System");
    let storage_hash = twox_128(b"Account");
    let account_hash = blake2b_128(account_id);
    let mut key = Vec::with_capacity(16 + 16 + 16 + 32);
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&account_hash);
    key.extend_from_slice(account_id);
    format!("0x{}", hex::encode(key))
}

fn decode_account_free_balance_fen(storage_hex: &str) -> Result<Option<String>, String> {
    let clean = storage_hex
        .strip_prefix("0x")
        .or_else(|| storage_hex.strip_prefix("0X"))
        .unwrap_or(storage_hex);
    let data = hex::decode(clean).map_err(|e| format!("decode System.Account hex failed: {e}"))?;
    if data.len() < 32 {
        return Ok(None);
    }
    // System.Account AccountInfo 前 16 字节为 nonce/consumers/providers/sufficients,
    // AccountData.free 是随后 16 字节 little-endian u128,单位为分。
    let mut free = [0_u8; 16];
    free.copy_from_slice(&data[16..32]);
    Ok(Some(u128::from_le_bytes(free).to_string()))
}

/// 批量读取账户 finalized free 余额,返回 key 为不带 0x 的小写 hex。
///
/// 管理员卡片只展示链上真实余额;查询失败或账户不存在时返回 None,
/// 由 UI 保留“余额”标签但不渲染余额值。0 余额是有效值,必须返回 Some("0")。
pub(crate) async fn fetch_account_balances_onchain(
    account_hexes: &[String],
) -> Result<BTreeMap<String, Option<String>>, String> {
    let mut result = BTreeMap::new();
    let mut unique_accounts: BTreeMap<String, [u8; 32]> = BTreeMap::new();
    for raw in account_hexes {
        let key = clean_account_hex_key(raw);
        result.entry(key.clone()).or_insert(None);
        if let Some(account) = parse_sr25519_pubkey_bytes(raw) {
            unique_accounts.insert(key, account);
        }
    }
    if unique_accounts.is_empty() {
        return Ok(result);
    }

    let http_url = super::chain_url::chain_http_url()?;
    let client = reqwest::Client::new();
    let finalized_hash = fetch_finalized_head_via_http(&client, http_url.as_str()).await?;
    let mut id_to_account = BTreeMap::new();
    let requests = unique_accounts
        .iter()
        .enumerate()
        .map(|(index, (account_hex, account_id))| {
            let id = (index + 1) as u64;
            id_to_account.insert(id, account_hex.clone());
            serde_json::json!({
                "id": id,
                "jsonrpc": "2.0",
                "method": "state_getStorage",
                "params": [system_account_storage_key(account_id), finalized_hash.clone()]
            })
        })
        .collect::<Vec<_>>();
    let response = client
        .post(http_url.as_str())
        .json(&requests)
        .send()
        .await
        .map_err(|e| format!("connect chain http rpc for account balances failed: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("chain http rpc returned status {status}"));
    }
    let payload = response
        .json::<Vec<ChainRpcValueResponse>>()
        .await
        .map_err(|e| format!("decode chain http rpc account balance response failed: {e}"))?;

    for item in payload {
        let Some(account_hex) = id_to_account.get(&item.id) else {
            continue;
        };
        if item.error.is_some() {
            result.insert(account_hex.clone(), None);
            continue;
        }
        let balance = match item.result {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(storage_hex)) => {
                decode_account_free_balance_fen(storage_hex.as_str()).unwrap_or(None)
            }
            Some(_) => None,
        };
        result.insert(account_hex.clone(), balance);
    }
    Ok(result)
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
    if let Ok(raw) = std::env::var("ONCHAIN_GENESIS_HASH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let parsed = parse_hex_hash32(trimmed)
                .map_err(|_| "ONCHAIN_GENESIS_HASH must be 32-byte hex".to_string())?;
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

fn parse_hex_2(raw: &str) -> Result<[u8; 2], String> {
    let trimmed = raw.trim();
    let no_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if no_prefix.len() != 4 || !no_prefix.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid 2-byte hex".to_string());
    }
    let bytes = hex::decode(no_prefix).map_err(|_| "invalid 2-byte hex".to_string())?;
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| "invalid 2-byte length".to_string())
}

fn sign_runtime_digest(_state: &AppState, digest: &[u8; 32]) -> Result<String, String> {
    // OnChina 系统签名密钥直接从环境变量 ONCHINA_SIGNING_SEED_HEX 派生,
    // AppState 不持有 seed,避免长期密钥进入运行态共享状态。
    let seed_hex = std::env::var("ONCHINA_SIGNING_SEED_HEX")
        .map_err(|_| "ONCHINA_SIGNING_SEED_HEX not set".to_string())?;
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
// 链上管理员集合读取(去中心化鉴权)
//
// 真源:机构 Active 管理员集合落链端两个机构 pallet 的 `AdminAccounts` storage——
// `PublicAdmins`(公权法人,含固定治理档 NRC/PRC/PRB/NJD 与 FRG 省级组)、
// `PrivateAdmins`(私权法人:股权/股份/有限合伙/公益/协会/私立学校等)。
// 节点按自身机构码路由到对应 pallet,登录验签后比对该集合放行,
// 本地 admins 表仅作元数据/省映射缓存。个人多签 PMUL 不在控制台范围。
/// 联邦注册局机构码,镜像 `admin_primitives::FRG`(`*b"FRG\0"`)。
/// onchina 不依赖 admin-primitives(避免引入 frame-support 重依赖),
/// 此处单字面镜像;FRG 为稳定常量,与链端保持一致即可。
const FRG_CODE: [u8; 4] = *b"FRG\0";
/// 国家司法院机构码。NJD 虽属固定治理档,但按产品边界进入 OnChina 控制台。
const NJD_CODE: [u8; 4] = *b"NJD\0";
pub(crate) const DESKTOP_GOVERNANCE_LOGIN_UNSUPPORTED: &str =
    "desktop governance institution is not supported by OnChina";
pub(crate) const PERSONAL_MULTISIG_LOGIN_UNSUPPORTED: &str =
    "personal multisig is not supported by OnChina";

/// `AdminAccountStatus::Active` 的 SCALE 判别式(Pending=0 / Active=1 / Closed=2)。
const ADMIN_STATUS_ACTIVE: u8 = 1;

/// 链上机构管理员资料解码镜像(对齐 runtime `admin-primitives::AdminProfile`)。
///
/// **铁律**:字段顺序与类型必须与 runtime `AdminProfile` 逐字节一致——
/// `account=[u8;32]` 无前缀;`admin_cid_number`/`name`/`admin_role` 为 `BoundedVec<u8>`,
/// 与 `Vec<u8>` SCALE 编码相同(Compact 长度 + 字节);`term_start`/`term_end=u32`;
/// `AdminSource` 枚举 1 字节判别。membership 只取 `account`,资料字段供展示。
#[derive(Debug, Decode)]
struct OnChainAdminProfile {
    account: [u8; 32],
    admin_cid_number: Vec<u8>,
    // name/admin_role 供管理员列表与大屏只读看板席位展示读取。
    name: Vec<u8>,
    admin_role: Vec<u8>,
    term_start: u32,
    term_end: u32,
    source: u8,
}

/// 链上 `AdminAccount` 解码镜像。
///
/// **铁律**:字段顺序与类型必须与 runtime `admin-primitives::AdminAccount` 逐字节一致——
/// `InstitutionCode=[u8;4]` 无前缀;`AdminAccountKind`/`AdminAccountStatus` 枚举各 1 字节判别;
/// 机构(public/private)管理员集合为 `BoundedVec<AdminProfile>`(A2);
/// `BlockNumberFor=u32`。任一字段宽度/顺序漂移都会解码错位 → membership 误判。
#[derive(Debug, Decode)]
struct OnChainAdminAccount {
    #[allow(dead_code)]
    institution_code: [u8; 4],
    #[allow(dead_code)]
    kind: u8,
    admins: Vec<OnChainAdminProfile>,
    #[allow(dead_code)]
    creator: [u8; 32],
    #[allow(dead_code)]
    created_at: u32,
    #[allow(dead_code)]
    updated_at: u32,
    status: u8,
}

/// 机构 Active 管理员集合所属链上 pallet。
///
/// 机构码决定容器:`PublicAdmins` 收公权法人和固定治理档,`PrivateAdmins` 收私权法人。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdminPallet {
    PublicAdmins,
    PrivateAdmins,
}

impl AdminPallet {
    /// construct_runtime 中的 pallet 名(subxt dynamic storage 寻址用)。
    fn pallet_name(self) -> &'static str {
        match self {
            AdminPallet::PublicAdmins => "PublicAdmins",
            AdminPallet::PrivateAdmins => "PrivateAdmins",
        }
    }
}

/// 本节点已绑定的链上机构身份(由首次 active admin 登录确认后落库)。
pub(crate) struct NodeInstitutionIdentity {
    /// 本机构 Active 管理员集合的候选 pallet;非法人为 [Public, Private] 按序探测。
    pub(crate) admin_pallets: Vec<AdminPallet>,
    /// 本机构 CID 号。提案归属/订阅统一按 CID,机构码只用于分类。
    pub(crate) cid_number: Option<String>,
    /// 机构主账户(AdminAccounts 键)。
    pub(crate) main_account: [u8; 32],
    /// 联邦注册局专用:本节点所辖省的链上省码([u8;2]);其它机构为 `None`。
    ///
    /// 链上 FRG 215 人按 43 省切组,Active 集合落 `PublicAdmins::FederalRegistryProvinceGroups`
    /// (键=`ProvinceCode`),**不在** `AdminAccounts`。本字段来自绑定候选的链上省组键。
    pub(crate) frg_province_code: Option<[u8; 2]>,
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveAdminMembership {
    pub(crate) institution_code: [u8; 4],
    pub(crate) main_account: Option<[u8; 32]>,
    pub(crate) frg_province_code: Option<[u8; 2]>,
}

impl ActiveAdminMembership {
    pub(crate) fn candidate_id(&self) -> String {
        let code = institution_code_label(&self.institution_code);
        if let Some(province_code) = self.frg_province_code {
            return format!("FRG:{}:{}", code, hex::encode(province_code));
        }
        let main_account = self
            .main_account
            .map(hex::encode)
            .unwrap_or_else(|| "missing".to_string());
        format!("ADM:{}:{}", code, main_account)
    }

    pub(crate) fn main_account_hex(&self) -> Option<String> {
        self.main_account
            .map(|account| format!("0x{}", hex::encode(account)))
    }

    pub(crate) fn frg_province_code_hex(&self) -> Option<String> {
        self.frg_province_code
            .map(|code| format!("0x{}", hex::encode(code)))
    }
}

/// 机构码 → 控制台准入的候选 admin pallet。
///
/// 镜像链端 `admin-primitives` 路由语义(用 `primitives::cid::code` 分类,不引入 admin-primitives
/// 重依赖):FRG→公权省级组;NJD/其它公权法人→公权;私权法人→私权;
/// 非法人按所属法人落公权或私权——账户键全局唯一,登录时按 [Public, Private] 顺序探测命中。
/// 国家储委会/省储委会/省储行走节点桌面端,个人主体/个人多签都不在控制台范围,返回错误拒绝。
fn console_admin_pallets(code: &[u8; 4]) -> Result<Vec<AdminPallet>, String> {
    use primitives::cid::code::{
        is_fixed_governance_code, is_private_legal_code, is_public_legal_code,
        is_unincorporated_code,
    };
    if *code == FRG_CODE {
        return Ok(vec![AdminPallet::PublicAdmins]);
    }
    if *code == NJD_CODE {
        return Ok(vec![AdminPallet::PublicAdmins]);
    }
    if let Some(reason) = console_login_block_reason(code) {
        return Err(reason.to_string());
    }
    if is_fixed_governance_code(code) {
        return Err("fixed-governance institution is not managed by this console".to_string());
    }
    if is_public_legal_code(code) {
        return Ok(vec![AdminPallet::PublicAdmins]);
    }
    if is_private_legal_code(code) {
        return Ok(vec![AdminPallet::PrivateAdmins]);
    }
    if is_unincorporated_code(code) {
        return Ok(vec![AdminPallet::PublicAdmins, AdminPallet::PrivateAdmins]);
    }
    Err("node institution code is not a console-managed institution".to_string())
}

fn console_login_block_reason(code: &[u8; 4]) -> Option<&'static str> {
    use primitives::cid::code::{is_personal_code, NRC, PRB, PRC};
    if matches!(*code, NRC | PRC | PRB) {
        return Some(DESKTOP_GOVERNANCE_LOGIN_UNSUPPORTED);
    }
    if is_personal_code(code) {
        return Some(PERSONAL_MULTISIG_LOGIN_UNSUPPORTED);
    }
    None
}

pub(crate) fn identity_from_binding_parts(
    institution_code: &str,
    institution_cid_number: Option<&str>,
    institution_main_account: Option<&str>,
    frg_province_code: Option<&str>,
) -> Result<NodeInstitutionIdentity, String> {
    let code = primitives::cid::code::institution_code_from_str(institution_code)
        .ok_or_else(|| "binding institution_code is invalid".to_string())?;
    let admin_pallets = console_admin_pallets(&code)?;
    let frg_code = frg_province_code
        .map(parse_hex_2)
        .transpose()
        .map_err(|_| "binding frg_province_code must be 2-byte hex".to_string())?;
    let main_account = if frg_code.is_some() {
        [0_u8; 32]
    } else {
        let raw = institution_main_account
            .ok_or_else(|| "binding institution_main_account is required".to_string())?;
        parse_sr25519_pubkey_bytes(raw)
            .ok_or_else(|| "binding institution_main_account must be 32-byte hex".to_string())?
    };
    let cid_number = institution_cid_number
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    if frg_code.is_none() && cid_number.is_none() {
        return Err("binding institution_cid_number is required".to_string());
    }
    Ok(NodeInstitutionIdentity {
        admin_pallets,
        cid_number,
        main_account,
        frg_province_code: frg_code,
    })
}

/// 机构码字节转 3/4 字符文本(供会话/DTO 存储)。
pub(crate) fn institution_code_label(code: &[u8; 4]) -> String {
    primitives::cid::code::institution_code_text(code)
        .unwrap_or("")
        .to_string()
}

/// 机构码行政层级标签(NATIONAL/PROVINCE/CITY/TOWN);私权法人/非法人无层级返回 None。
pub(crate) fn admin_level_label(code: &[u8; 4]) -> Option<String> {
    use primitives::cid::code::AdminLevel;
    primitives::cid::code::admin_level(code).map(|level| {
        match level {
            AdminLevel::National => "NATIONAL",
            AdminLevel::Province => "PROVINCE",
            AdminLevel::City => "CITY",
            AdminLevel::Town => "TOWN",
        }
        .to_string()
    })
}

/// 由机构码文本派生行政层级标签(供 DTO 构造)。无法解析或无层级返回 None。
pub(crate) fn admin_level_label_for(institution_code: &str) -> Option<String> {
    let bytes = primitives::cid::code::institution_code_from_str(institution_code)?;
    admin_level_label(&bytes)
}

/// Tier1 创世注册局机构码(本期 = 联邦注册局)。控制台注册局码单源,谓词与 SQL bind 共用,
/// 取代散落各处的 `"FRG"` 字面(谓词单点除外)。
pub(crate) const TIER1_REGISTRY_CODE: &str = "FRG";

/// Tier2 下级注册局机构码(本期 = 市注册局),由 Tier1 供给。控制台注册局码单源,
/// 取代散落各处的 `"CREG"` 字面。
pub(crate) const TIER2_REGISTRY_CODE: &str = "CREG";

/// 控制台注册局分层单点谓词:Tier1 = 创世注册局(本期 = 联邦注册局 FRG)。
///
/// 取代散落各处的 `institution_code == "FRG"` 字面。FRG 的 `admin_level` 虽为
/// `National`(链端铁律不可改),但其管理员按省分区(每节点单省),故控制台据此谓词
/// 单点矫正为省级分层 / 治理边界,而非全国。
pub(crate) fn is_tier1_registry(institution_code: &str) -> bool {
    institution_code == TIER1_REGISTRY_CODE
}

/// 控制台注册局分层单点谓词:Tier2 = 下级注册局(本期 = 市注册局 CREG),由 Tier1 供给。
///
/// 取代散落各处的 `institution_code == "CREG"` 字面。
pub(crate) fn is_subordinate_registry(institution_code: &str) -> bool {
    institution_code == TIER2_REGISTRY_CODE
}

/// 省名 → 链上省码([u8;2]),单源 `primitives::cid::code::PROVINCE_CODE_INFOS`。
///
/// 此为链上 `ProvinceCode`(FRG 省级组 storage 键),与 china.sqlite 行政区编码
/// (`crate::cid::china::province_code_by_name`)是两套不同口径,勿混用。
pub(crate) fn chain_province_code_by_name(province_name: &str) -> Option<[u8; 2]> {
    let trimmed = province_name.trim();
    primitives::cid::code::PROVINCE_CODE_INFOS
        .iter()
        .find(|info| info.province_name == trimmed)
        .map(|info| info.province_code)
}

pub(crate) fn chain_province_name_by_code(province_code: [u8; 2]) -> Option<String> {
    primitives::cid::code::PROVINCE_CODE_INFOS
        .iter()
        .find(|info| info.province_code == province_code)
        .map(|info| info.province_name.to_string())
}

pub(crate) fn storage_key_suffix<const N: usize>(key_bytes: &[u8]) -> Result<[u8; N], String> {
    if key_bytes.len() < N {
        return Err("storage key shorter than expected".to_string());
    }
    key_bytes[key_bytes.len() - N..]
        .try_into()
        .map_err(|_| "storage key suffix decode failed".to_string())
}

fn contains_admin(decoded: &OnChainAdminAccount, target: &[u8; 32]) -> bool {
    decoded
        .admins
        .iter()
        .any(|profile| &profile.account == target)
}

/// 用冷钱包签名账户反查其所属的链上 active admin 机构集合。
///
/// 这是链上中国通用平台的登录真源。平台启动时不再预设机构;
/// 已验签账户在链上哪些机构的 Active 管理员集合内,就得到哪些可绑定候选。
/// 当前不改 runtime,先扫描现有 storage;后续若要性能优化,再单独给链端加反向索引。
/// 链上公权机构登记查询结果(创世目录抽样/全量对账用,字段最小化)。
pub(crate) struct OnChainInstitution {
    pub(crate) cid_full_name: Vec<u8>,
    pub(crate) cid_short_name: Vec<u8>,
    pub(crate) town_code: Vec<u8>,
    pub(crate) institution_code: [u8; 4],
    pub(crate) created_at: u32,
    /// InstitutionLifecycleStatus 判别值:0=Pending 1=Active 2=Closed。
    pub(crate) status: u8,
}

/// 链上公权机构账户投影。真源为 `PublicManage::InstitutionAccounts`。
pub(crate) struct OnChainInstitutionAccount {
    pub(crate) cid_number: Vec<u8>,
    pub(crate) account_name: Vec<u8>,
    pub(crate) account: [u8; 32],
    /// InstitutionLifecycleStatus 判别值:0=Pending 1=Active 2=Closed。
    pub(crate) status: u8,
    pub(crate) created_at: u32,
}

/// 与 public-manage `InstitutionInfo` 字段序一致的最小解码结构。
#[derive(codec::Decode)]
struct RawInstitutionInfo {
    cid_full_name: Vec<u8>,
    cid_short_name: Vec<u8>,
    town_code: Vec<u8>,
    institution_code: [u8; 4],
    created_at: u32,
    status: u8,
}

/// 与 public-manage `InstitutionAccountInfo<AccountId, Balance, BlockNumber>` 字段序一致。
#[derive(codec::Decode)]
struct RawInstitutionAccountInfo {
    address: [u8; 32],
    _initial_balance: u128,
    status: u8,
    _is_default: bool,
    created_at: u32,
}

/// 读链上 `PublicManage::Institutions[cid]`;None = 未登记。
pub(crate) async fn institution_lookup(
    cid_number: &str,
) -> Result<Option<OnChainInstitution>, String> {
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for institutions failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    let query = dynamic::storage(
        "PublicManage",
        "Institutions",
        vec![dynamic::Value::from_bytes(cid_number.as_bytes())],
    );
    let Some(value) = storage
        .fetch(&query)
        .await
        .map_err(|e| format!("fetch institution failed: {e}"))?
    else {
        return Ok(None);
    };
    let mut raw = value.encoded();
    let info = RawInstitutionInfo::decode(&mut raw)
        .map_err(|e| format!("decode institution info failed: {e}"))?;
    Ok(Some(OnChainInstitution {
        cid_full_name: info.cid_full_name,
        cid_short_name: info.cid_short_name,
        town_code: info.town_code,
        institution_code: info.institution_code,
        created_at: info.created_at,
        status: info.status,
    }))
}

/// 全量遍历链上 `PublicManage::Institutions`(部署验收对账用),
/// 每条回调 `(cid_number 字节, 机构信息)`,返回遍历总数。
pub(crate) async fn for_each_chain_institution(
    mut f: impl FnMut(Vec<u8>, OnChainInstitution),
) -> Result<usize, String> {
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for institutions failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    let query = dynamic::storage("PublicManage", "Institutions", Vec::<dynamic::Value>::new());
    let mut iter = storage
        .iter(query)
        .await
        .map_err(|e| format!("iterate institutions failed: {e}"))?;
    let mut count = 0usize;
    while let Some(item) = iter.next().await {
        let kv = item.map_err(|e| format!("read institution entry failed: {e}"))?;
        // 键 = 32 前缀 + 16 blake2_128 + SCALE(BoundedVec<u8>);取尾段解出号字节。
        let suffix = &kv.key_bytes[48..];
        let mut cursor = suffix;
        let cid: Vec<u8> = codec::Decode::decode(&mut cursor)
            .map_err(|e| format!("decode institution key failed: {e}"))?;
        let mut raw = kv.value.encoded();
        let info = RawInstitutionInfo::decode(&mut raw)
            .map_err(|e| format!("decode institution info failed: {e}"))?;
        f(
            cid,
            OnChainInstitution {
                cid_full_name: info.cid_full_name,
                cid_short_name: info.cid_short_name,
                town_code: info.town_code,
                institution_code: info.institution_code,
                created_at: info.created_at,
                status: info.status,
            },
        );
        count += 1;
    }
    Ok(count)
}

/// 全量遍历链上 `PublicManage::InstitutionAccounts`。
///
/// 只读取链上 storage,不按本地行政区或模板派生账户;本地 PostgreSQL 仅作为投影缓存。
pub(crate) async fn for_each_chain_institution_account(
    mut f: impl FnMut(OnChainInstitutionAccount),
) -> Result<usize, String> {
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for institution accounts failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    let query = dynamic::storage(
        "PublicManage",
        "InstitutionAccounts",
        Vec::<dynamic::Value>::new(),
    );
    let mut iter = storage
        .iter(query)
        .await
        .map_err(|e| format!("iterate institution accounts failed: {e}"))?;
    let mut count = 0usize;
    while let Some(item) = iter.next().await {
        let kv = item.map_err(|e| format!("read institution account entry failed: {e}"))?;
        // 键 = 32 前缀 + 16 blake2_128 + SCALE(cid) + 16 blake2_128 + SCALE(account_name)。
        let mut suffix = &kv.key_bytes[48..];
        let cid_number: Vec<u8> = codec::Decode::decode(&mut suffix)
            .map_err(|e| format!("decode institution account cid key failed: {e}"))?;
        if suffix.len() < 16 {
            return Err("institution account key missing account_name hash suffix".to_string());
        }
        suffix = &suffix[16..];
        let account_name: Vec<u8> = codec::Decode::decode(&mut suffix)
            .map_err(|e| format!("decode institution account name key failed: {e}"))?;
        let mut raw = kv.value.encoded();
        let info = RawInstitutionAccountInfo::decode(&mut raw)
            .map_err(|e| format!("decode institution account info failed: {e}"))?;
        f(OnChainInstitutionAccount {
            cid_number,
            account_name,
            account: info.address,
            status: info.status,
            created_at: info.created_at,
        });
        count += 1;
    }
    Ok(count)
}

/// 链上 CID 占号登记查询结果(发号预查与幂等续用识别用,字段最小化)。
pub(crate) struct OnChainCidRecord {
    pub(crate) commitment: [u8; 32],
    pub(crate) status_active: bool,
}

/// 读链上 `CitizenIdentity::CidRegistry`;None = 号未被占。
pub(crate) async fn cid_registry_lookup(
    cid_number: &str,
) -> Result<Option<OnChainCidRecord>, String> {
    /// 与 pallet `CidRecord` 字段序一致的最小解码结构。
    #[derive(codec::Decode)]
    struct RawRecord {
        _registrar: [u8; 32],
        commitment: [u8; 32],
        _province: alloc_vec_u8::Bytes,
        _city: alloc_vec_u8::Bytes,
        status: u8,
        _registered_at: u32,
        _revoked_at: Option<u32>,
    }
    mod alloc_vec_u8 {
        pub(super) type Bytes = Vec<u8>;
    }
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for cid registry failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    let query = dynamic::storage(
        "CitizenIdentity",
        "CidRegistry",
        vec![dynamic::Value::from_bytes(cid_number.as_bytes())],
    );
    let Some(value) = storage
        .fetch(&query)
        .await
        .map_err(|e| format!("fetch cid registry failed: {e}"))?
    else {
        return Ok(None);
    };
    let mut raw = value.encoded();
    let record = RawRecord::decode(&mut raw)
        .map_err(|e| format!("decode cid registry record failed: {e}"))?;
    Ok(Some(OnChainCidRecord {
        commitment: record.commitment,
        status_active: record.status == 0,
    }))
}

pub(crate) async fn find_active_admin_memberships(
    verified_pubkey: &str,
) -> Result<Vec<ActiveAdminMembership>, String> {
    let target = parse_sr25519_pubkey_bytes(verified_pubkey)
        .ok_or_else(|| "verified_pubkey must be a 32-byte account hex".to_string())?;
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for admin membership scan failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;

    let mut memberships = Vec::new();
    let mut blocked_login_reason: Option<&'static str> = None;
    let frg_query = dynamic::storage(
        AdminPallet::PublicAdmins.pallet_name(),
        "FederalRegistryProvinceGroups",
        Vec::<dynamic::Value>::new(),
    );
    let mut frg_iter = storage
        .iter(frg_query)
        .await
        .map_err(|e| format!("iterate federal registry province groups failed: {e}"))?;
    while let Some(item) = frg_iter.next().await {
        let kv = item.map_err(|e| format!("read federal registry province group failed: {e}"))?;
        let mut raw = kv.value.encoded();
        let decoded = OnChainAdminAccount::decode(&mut raw)
            .map_err(|e| format!("decode federal registry province group failed: {e}"))?;
        if decoded.status == ADMIN_STATUS_ACTIVE && contains_admin(&decoded, &target) {
            let province_code = storage_key_suffix::<2>(&kv.key_bytes)?;
            memberships.push(ActiveAdminMembership {
                institution_code: FRG_CODE,
                main_account: None,
                frg_province_code: Some(province_code),
            });
        }
    }

    for pallet in [AdminPallet::PublicAdmins, AdminPallet::PrivateAdmins] {
        let query = dynamic::storage(
            pallet.pallet_name(),
            "AdminAccounts",
            Vec::<dynamic::Value>::new(),
        );
        let mut iter = storage
            .iter(query)
            .await
            .map_err(|e| format!("iterate {} AdminAccounts failed: {e}", pallet.pallet_name()))?;
        while let Some(item) = iter.next().await {
            let kv = item
                .map_err(|e| format!("read {} AdminAccounts failed: {e}", pallet.pallet_name()))?;
            let mut raw = kv.value.encoded();
            let decoded = OnChainAdminAccount::decode(&mut raw).map_err(|e| {
                format!("decode {} AdminAccounts failed: {e}", pallet.pallet_name())
            })?;
            if decoded.status != ADMIN_STATUS_ACTIVE || !contains_admin(&decoded, &target) {
                continue;
            }
            if let Some(reason) = console_login_block_reason(&decoded.institution_code) {
                blocked_login_reason.get_or_insert(reason);
                continue;
            }
            let allowed = console_admin_pallets(&decoded.institution_code)?;
            if !allowed.contains(&pallet) {
                continue;
            }
            let main_account = storage_key_suffix::<32>(&kv.key_bytes)?;
            memberships.push(ActiveAdminMembership {
                institution_code: decoded.institution_code,
                main_account: Some(main_account),
                frg_province_code: None,
            });
        }
    }

    memberships.sort_by_key(|m| m.candidate_id());
    memberships.dedup_by_key(|m| m.candidate_id());
    if memberships.is_empty() {
        if let Some(reason) = blocked_login_reason {
            return Err(reason.to_string());
        }
    }
    Ok(memberships)
}

/// 读取本节点机构的链上 Active 管理员公钥集合(0x 小写 hex 列表)。
///
/// 定位口径按机构分流:
/// - **联邦注册局(FRG)**:Active 集合落 `PublicAdmins::FederalRegistryProvinceGroups`
///   (键=本节点省码 `ProvinceCode`),**不在** `AdminAccounts`——读单一省组。
/// - **其它机构**:按候选 pallet 顺序探测 `<Pallet>::AdminAccounts`(键=机构主账户,全局唯一),
///   命中首个存在且 Active 的集合即返回。
///
/// 返回:`Ok(Some(set))`=命中 Active 集合;`Ok(None)`=不存在或非 Active;`Err`=链不可达或解码失败。
/// 读 latest 块(membership 变更治理级稀有,后台扫描持续复查)。
pub(crate) async fn fetch_active_admins_onchain(
    identity: &NodeInstitutionIdentity,
) -> Result<Option<Vec<String>>, String> {
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for admin set failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;

    // 候选 storage 地址:FRG → 省级组(键=省码);其它 → 各候选 pallet 的 AdminAccounts(键=主账户)。
    let addresses = if let Some(province_code) = identity.frg_province_code {
        vec![dynamic::storage(
            AdminPallet::PublicAdmins.pallet_name(),
            "FederalRegistryProvinceGroups",
            vec![dynamic::Value::from_bytes(province_code)],
        )]
    } else {
        identity
            .admin_pallets
            .iter()
            .map(|pallet| {
                dynamic::storage(
                    pallet.pallet_name(),
                    "AdminAccounts",
                    vec![dynamic::Value::from_bytes(identity.main_account)],
                )
            })
            .collect::<Vec<_>>()
    };

    for address in &addresses {
        let Some(thunk) = storage
            .fetch(address)
            .await
            .map_err(|e| format!("fetch on-chain admin account failed: {e}"))?
        else {
            continue;
        };
        let mut raw = thunk.encoded();
        let decoded = OnChainAdminAccount::decode(&mut raw)
            .map_err(|e| format!("decode on-chain admin account failed: {e}"))?;
        if decoded.status != ADMIN_STATUS_ACTIVE {
            continue;
        }
        // membership 只认账户:从 AdminProfile 取 account,资料字段(姓名/职务/任期)不参与登录闸。
        let admins = decoded
            .admins
            .iter()
            .map(|profile| format!("0x{}", hex::encode(profile.account)))
            .collect();
        return Ok(Some(admins));
    }
    Ok(None)
}

/// 链上管理员资料只读投影(供大屏看板席位展示;姓名/职务/任期为 UTF-8 有损解码)。
#[derive(Debug, Clone)]
pub(crate) struct OnChainAdminProfileView {
    /// 管理员账户(0x 小写 hex)。
    pub(crate) account_hex: String,
    /// 管理员实名锚:注册局签发的 CID 号。
    pub(crate) admin_cid_number: String,
    /// 姓名快照(链上 `AdminProfile::name`)。
    pub(crate) name: String,
    /// 对外法定职务(链上 `AdminProfile::admin_role`)。
    pub(crate) admin_role: String,
    /// 旧看板字段名,由 `admin_role` 同步赋值;管理员列表不得使用此字段。
    pub(crate) title: String,
    /// 任期开始(天数自纪元;无任期为 0)。
    pub(crate) term_start: u32,
    /// 任期结束(天数自纪元;无任期为 0)。
    pub(crate) term_end: u32,
    /// 职务/任期来源判别值。
    pub(crate) source: u8,
    /// 来源中文标签;未知来源留空。
    pub(crate) source_label: String,
}

fn admin_source_label(source: u8) -> &'static str {
    match source {
        0 => "创世",
        1 => "注册局",
        2 => "内部投票",
        3 => "互选",
        4 => "普选",
        _ => "",
    }
}

/// 读取本节点机构的链上 Active 管理员**资料**集合(账户 + 姓名 + 职务 + 任期)。
///
/// 与 `fetch_active_admins_onchain`(只取账户,用于登录闸)同一 storage 定位口径,
/// 但保留 `AdminProfile` 的展示字段供大屏席位板呈现。定位分流:FRG→省级组;其它→候选 pallet
/// 的 `AdminAccounts`(键=机构主账户)。`Ok(Some(set))`=命中 Active 集合;`Ok(None)`=不存在/非 Active。
pub(crate) async fn fetch_active_admin_profiles_onchain(
    identity: &NodeInstitutionIdentity,
) -> Result<Option<Vec<OnChainAdminProfileView>>, String> {
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for admin profiles failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;

    let addresses = if let Some(province_code) = identity.frg_province_code {
        vec![dynamic::storage(
            AdminPallet::PublicAdmins.pallet_name(),
            "FederalRegistryProvinceGroups",
            vec![dynamic::Value::from_bytes(province_code)],
        )]
    } else {
        identity
            .admin_pallets
            .iter()
            .map(|pallet| {
                dynamic::storage(
                    pallet.pallet_name(),
                    "AdminAccounts",
                    vec![dynamic::Value::from_bytes(identity.main_account)],
                )
            })
            .collect::<Vec<_>>()
    };

    for address in &addresses {
        let Some(thunk) = storage
            .fetch(address)
            .await
            .map_err(|e| format!("fetch on-chain admin profiles failed: {e}"))?
        else {
            continue;
        };
        let mut raw = thunk.encoded();
        let decoded = OnChainAdminAccount::decode(&mut raw)
            .map_err(|e| format!("decode on-chain admin profiles failed: {e}"))?;
        if decoded.status != ADMIN_STATUS_ACTIVE {
            continue;
        }
        let profiles = decoded
            .admins
            .iter()
            .map(|profile| OnChainAdminProfileView {
                account_hex: format!("0x{}", hex::encode(profile.account)),
                admin_cid_number: String::from_utf8_lossy(&profile.admin_cid_number).to_string(),
                name: String::from_utf8_lossy(&profile.name).to_string(),
                admin_role: String::from_utf8_lossy(&profile.admin_role).to_string(),
                title: String::from_utf8_lossy(&profile.admin_role).to_string(),
                term_start: profile.term_start,
                term_end: profile.term_end,
                source: profile.source,
                source_label: admin_source_label(profile.source).to_string(),
            })
            .collect();
        return Ok(Some(profiles));
    }
    Ok(None)
}

/// 读取联邦注册局指定省 5 人组的 Active 管理员完整资料。
pub(crate) async fn fetch_federal_registry_province_admin_profiles(
    province_code: [u8; 2],
) -> Result<Vec<OnChainAdminProfileView>, String> {
    let ws_url = super::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for federal registry profiles failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    let address = dynamic::storage(
        AdminPallet::PublicAdmins.pallet_name(),
        "FederalRegistryProvinceGroups",
        vec![dynamic::Value::from_bytes(province_code)],
    );
    let Some(thunk) = storage
        .fetch(&address)
        .await
        .map_err(|e| format!("fetch federal registry province profiles failed: {e}"))?
    else {
        return Ok(vec![]);
    };
    let mut raw = thunk.encoded();
    let decoded = OnChainAdminAccount::decode(&mut raw)
        .map_err(|e| format!("decode federal registry province profiles failed: {e}"))?;
    if decoded.status != ADMIN_STATUS_ACTIVE {
        return Ok(vec![]);
    }
    Ok(decoded
        .admins
        .iter()
        .map(|profile| OnChainAdminProfileView {
            account_hex: format!("0x{}", hex::encode(profile.account)),
            admin_cid_number: String::from_utf8_lossy(&profile.admin_cid_number).to_string(),
            name: String::from_utf8_lossy(&profile.name).to_string(),
            admin_role: String::from_utf8_lossy(&profile.admin_role).to_string(),
            title: String::from_utf8_lossy(&profile.admin_role).to_string(),
            term_start: profile.term_start,
            term_end: profile.term_end,
            source: profile.source,
            source_label: admin_source_label(profile.source).to_string(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{
        deregistration_payload_digest, is_production_mode, parse_hex_hash32,
        trusted_production_chain_by_hash,
    };

    /// 跨真类型对拍:用真 runtime `AdminAccount<Vec<AdminProfile>>` 编码,经 onchina
    /// `OnChainAdminAccount` 镜像解码,断言取出的 account/status 正确——锁死登录闸读
    /// A2 后管理员资料形态的字节对齐,任何字段漂移此断言立即红。
    #[test]
    fn onchain_admin_account_mirror_matches_runtime_adminprofile() {
        use admin_primitives::{
            AdminAccount, AdminAccountKind, AdminAccountStatus, AdminProfile, AdminSource,
        };
        use codec::{Decode, Encode};
        use frame_support::BoundedVec;

        let profile = AdminProfile::<[u8; 32]> {
            account: [0x42; 32],
            admin_cid_number: BoundedVec::try_from(b"LN001-AAAAA-000000001-2026".to_vec()).unwrap(),
            name: BoundedVec::try_from("张三".as_bytes().to_vec()).unwrap(),
            admin_role: BoundedVec::try_from("局长".as_bytes().to_vec()).unwrap(),
            term_start: 20_100,
            term_end: 21_561,
            source: AdminSource::Registry,
        };
        let account = AdminAccount::<Vec<AdminProfile<[u8; 32]>>, [u8; 32], u32> {
            institution_code: *b"CREG",
            kind: AdminAccountKind::PublicInstitution,
            admins: vec![profile],
            creator: [0x01; 32],
            created_at: 7,
            updated_at: 9,
            status: AdminAccountStatus::Active,
        };
        let bytes = account.encode();
        let decoded = super::OnChainAdminAccount::decode(&mut &bytes[..])
            .expect("onchina mirror must decode runtime AdminAccount<AdminProfile>");
        assert_eq!(decoded.institution_code, *b"CREG");
        assert_eq!(decoded.creator, [0x01; 32]);
        assert_eq!(decoded.created_at, 7);
        assert_eq!(decoded.updated_at, 9);
        assert_eq!(decoded.status, super::ADMIN_STATUS_ACTIVE);
        assert_eq!(decoded.admins.len(), 1);
        assert_eq!(decoded.admins[0].account, [0x42; 32]);
    }

    #[test]
    fn console_pallets_allow_njd_and_block_desktop_governance() {
        assert_eq!(
            super::console_admin_pallets(b"NJD\0").unwrap(),
            vec![super::AdminPallet::PublicAdmins]
        );

        for code in [b"NRC\0", b"PRC\0", b"PRB\0"] {
            assert_eq!(
                super::console_admin_pallets(code).unwrap_err(),
                super::DESKTOP_GOVERNANCE_LOGIN_UNSUPPORTED
            );
        }
    }

    #[test]
    fn console_pallets_keep_unincorporated_dual_probe_and_personal_rejected() {
        assert_eq!(
            super::console_admin_pallets(b"UNIN").unwrap(),
            vec![
                super::AdminPallet::PublicAdmins,
                super::AdminPallet::PrivateAdmins
            ]
        );
        assert_eq!(
            super::console_admin_pallets(b"PMUL").unwrap_err(),
            super::PERSONAL_MULTISIG_LOGIN_UNSUPPORTED
        );
    }

    #[test]
    fn deregistration_payload_digest_is_byte_locked() {
        // golden 测试锁死注销凭证 payload 的 SCALE 字节编码。
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
        let previous = std::env::var("ONCHINA_ENV").ok();
        std::env::set_var("ONCHINA_ENV", "prod");
        assert!(is_production_mode());
        if let Some(value) = previous {
            std::env::set_var("ONCHINA_ENV", value);
        } else {
            std::env::remove_var("ONCHINA_ENV");
        }
    }
}
