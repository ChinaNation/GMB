use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use parity_scale_codec::Encode;
use schnorrkel::{signing_context, Keypair as Sr25519Keypair};
use std::sync::{Arc, OnceLock, RwLock};

use crate::*;

const BIND_DOMAIN: [u8; 16] = *b"GMB_SFID_BIND_V1";
const VOTE_DOMAIN: [u8; 16] = *b"GMB_SFID_VOTE_V1";
pub(crate) const POPULATION_DOMAIN_STR: &str = "GMB_SFID_POPULATION_V1";
const POPULATION_DOMAIN: [u8; 22] = *b"GMB_SFID_POPULATION_V1";
static CHAIN_GENESIS_HASH: OnceLock<[u8; 32]> = OnceLock::new();
static SIGNING_KEY_CACHE: OnceLock<RwLock<Option<CachedSigningKey>>> = OnceLock::new();

struct CachedSigningKey {
    seed_hex: SensitiveSeed,
    keypair: Arc<Sr25519Keypair>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSignatureMeta {
    pub(crate) key_id: String,
    pub(crate) key_version: String,
    pub(crate) alg: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBindCredential {
    pub(crate) sfid_code_hash: String,
    pub(crate) nonce: String,
    pub(crate) signature: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeVoteCredential {
    pub(crate) sfid_hash: String,
    pub(crate) proposal_id: u64,
    pub(crate) vote_nonce: String,
    pub(crate) signature: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimePopulationSnapshotSignature {
    pub(crate) who: String,
    pub(crate) eligible_total: u64,
    pub(crate) snapshot_nonce: String,
    pub(crate) signature: String,
    pub(crate) genesis_hash: String,
    pub(crate) payload_digest: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

pub(crate) fn build_bind_credential(
    state: &AppState,
    account_pubkey: &str,
    sfid_code: &str,
    nonce: String,
) -> Result<RuntimeBindCredential, String> {
    if nonce.trim().is_empty() {
        return Err("bind nonce is required".to_string());
    }
    let (_, who) = normalize_and_parse_account_id32(account_pubkey)?;
    let genesis_hash = resolve_chain_genesis_hash()?;
    let sfid_code_hash = blake2_256(sfid_code.as_bytes());
    let payload = (
        BIND_DOMAIN,
        genesis_hash,
        who,
        sfid_code_hash,
        nonce.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimeBindCredential {
        sfid_code_hash: hex::encode(sfid_code_hash),
        nonce,
        signature,
        meta: runtime_signature_meta(state),
    })
}

pub(crate) fn build_vote_credential(
    state: &AppState,
    account_pubkey: &str,
    sfid_code: &str,
    proposal_id: u64,
    vote_nonce: String,
) -> Result<RuntimeVoteCredential, String> {
    if vote_nonce.trim().is_empty() {
        return Err("vote_nonce is required".to_string());
    }
    let (_, who) = normalize_and_parse_account_id32(account_pubkey)?;
    let genesis_hash = resolve_chain_genesis_hash()?;
    let sfid_hash = blake2_256(sfid_code.as_bytes());
    let payload = (
        VOTE_DOMAIN,
        genesis_hash,
        who,
        sfid_hash,
        proposal_id,
        vote_nonce.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimeVoteCredential {
        sfid_hash: hex::encode(sfid_hash),
        proposal_id,
        vote_nonce,
        signature,
        meta: runtime_signature_meta(state),
    })
}

pub(crate) fn build_population_snapshot_signature(
    state: &AppState,
    account_pubkey: &str,
    eligible_total: u64,
    snapshot_nonce: String,
) -> Result<RuntimePopulationSnapshotSignature, String> {
    if snapshot_nonce.trim().is_empty() {
        return Err("snapshot_nonce is required".to_string());
    }
    let (normalized_who, who) = normalize_and_parse_account_id32(account_pubkey)?;
    let genesis_hash = resolve_chain_genesis_hash()?;
    let payload = (
        POPULATION_DOMAIN,
        genesis_hash,
        who,
        eligible_total,
        snapshot_nonce.as_bytes(),
    );
    let payload_digest = blake2_256(&payload.encode());
    let signature = sign_runtime_digest(state, &payload_digest)?;
    Ok(RuntimePopulationSnapshotSignature {
        who: normalized_who,
        eligible_total,
        snapshot_nonce,
        signature,
        genesis_hash: hex::encode(genesis_hash),
        payload_digest: hex::encode(payload_digest),
        meta: runtime_signature_meta(state),
    })
}

fn runtime_signature_meta(state: &AppState) -> RuntimeSignatureMeta {
    RuntimeSignatureMeta {
        key_id: state.key_id.clone(),
        key_version: state.key_version.clone(),
        alg: state.key_alg.clone(),
    }
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
    let raw = std::env::var("SFID_CHAIN_GENESIS_HASH")
        .map_err(|_| "SFID_CHAIN_GENESIS_HASH must be configured".to_string())?;
    let parsed = parse_hex_hash32(raw.as_str())
        .map_err(|_| "SFID_CHAIN_GENESIS_HASH must be 32-byte hex".to_string())
    ?;
    let _ = CHAIN_GENESIS_HASH.set(parsed);
    Ok(CHAIN_GENESIS_HASH.get().copied().unwrap_or(parsed))
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

fn sign_runtime_digest(state: &AppState, digest: &[u8; 32]) -> Result<String, String> {
    let seed = state
        .signing_seed_hex
        .read()
        .map_err(|_| "signing seed read lock poisoned".to_string())?
        .clone();
    let signing_key = resolve_signing_keypair(seed.expose_secret())?;
    let signature = signing_key.sign(signing_context(b"substrate").bytes(digest));
    Ok(hex::encode(signature.to_bytes()))
}

fn resolve_signing_keypair(seed_text: &str) -> Result<Arc<Sr25519Keypair>, String> {
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

    let loaded = Arc::new(key_admins::chain_keyring::try_load_signing_key_from_seed(
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
