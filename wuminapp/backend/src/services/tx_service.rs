use std::{
    collections::HashMap,
    env,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use blake2::digest::Digest;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};

use crate::{
    errors::ApiError,
    models::{TxPrepareData, TxPrepareRequest, TxStatusData, TxSubmitData, TxSubmitRequest},
};

const STATUS_PENDING: &str = "pending";
const STATUS_CONFIRMED: &str = "confirmed";
const STATUS_FAILED: &str = "failed";
const FAIL_REASON_NOT_FOUND: &str = "tx not found";
const FAIL_REASON_PREPARED_NOT_FOUND: &str = "prepared tx not found or expired";
const FAIL_REASON_BROADCAST: &str = "broadcast failed";
const FAIL_REASON_EXECUTION: &str = "onchain execution failed";
const DEFAULT_CHAIN_WS_URL: &str = "ws://127.0.0.1:9944";
const PREPARED_TTL_SECS: i64 = 180;
const TX_STATE_TTL_SECS: i64 = 24 * 60 * 60;
const TX_STATE_MAX: usize = 20_000;
const AMOUNT_DECIMALS: f64 = 100.0;

#[derive(Clone)]
struct TxRuntimeState {
    updated_at: i64,
    status: String,
    failure_reason: Option<String>,
}

struct PreparedTxState {
    created_at: i64,
    signer_pubkey: [u8; 32],
    partial_tx: subxt::tx::PartialTransaction<PolkadotConfig, OnlineClient<PolkadotConfig>>,
}

static TX_STATE: OnceLock<Mutex<HashMap<String, TxRuntimeState>>> = OnceLock::new();
static PREPARED_STATE: OnceLock<Mutex<HashMap<String, PreparedTxState>>> = OnceLock::new();

fn tx_state() -> &'static Mutex<HashMap<String, TxRuntimeState>> {
    TX_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn prepared_state() -> &'static Mutex<HashMap<String, PreparedTxState>> {
    PREPARED_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn prepare_tx(req: TxPrepareRequest) -> Result<TxPrepareData, ApiError> {
    validate_prepare_request(&req)?;

    let signer_pubkey = parse_pubkey_hex(&req.pubkey_hex)?;
    let from_pubkey = parse_account_to_pubkey(&req.from_address)?;
    if signer_pubkey != from_pubkey {
        return Err(ApiError::new(1001, "from_address and pubkey_hex mismatch"));
    }
    let to_pubkey = parse_account_to_pubkey(&req.to_address)?;
    let amount_fen = parse_amount_to_fen(req.amount)?;

    let payload = tx(
        "Balances",
        "transfer_allow_death",
        vec![
            Value::unnamed_variant("Id", [Value::from_bytes(to_pubkey)]),
            Value::u128(amount_fen),
        ],
    );

    let client = chain_client().await?;
    let signer = AccountId32(signer_pubkey);
    let partial_tx = client
        .tx()
        .create_partial(&payload, &signer, Default::default())
        .await
        .map_err(|_| ApiError::new(5002, "prepare tx failed"))?;
    let signer_payload = partial_tx.signer_payload();

    let prepared_id = generate_prepared_id(&signer_pubkey, &signer_payload);
    let now = now_secs();
    let expires_at = now + PREPARED_TTL_SECS;

    if let Ok(mut map) = prepared_state().lock() {
        map.retain(|_, v| now.saturating_sub(v.created_at) < PREPARED_TTL_SECS);
        map.insert(
            prepared_id.clone(),
            PreparedTxState {
                created_at: now,
                signer_pubkey,
                partial_tx,
            },
        );
    }

    Ok(TxPrepareData {
        prepared_id,
        signer_payload_hex: format!("0x{}", hex_encode(&signer_payload)),
        expires_at,
    })
}

pub async fn submit_tx(req: TxSubmitRequest) -> Result<TxSubmitData, ApiError> {
    let signer_pubkey = parse_pubkey_hex(&req.pubkey_hex)?;
    let signature = parse_signature(&req.signature_hex)?;

    let mut state = {
        let mut map = prepared_state()
            .lock()
            .map_err(|_| ApiError::new(5002, "prepared tx lock failed"))?;
        let now = now_secs();
        map.retain(|_, v| now.saturating_sub(v.created_at) < PREPARED_TTL_SECS);
        map.remove(&req.prepared_id)
    }
    .ok_or(ApiError::new(3004, FAIL_REASON_PREPARED_NOT_FOUND))?;

    if state.signer_pubkey != signer_pubkey {
        return Err(ApiError::new(1001, "signer mismatch"));
    }

    let account_id = AccountId32(signer_pubkey);
    let multi_sig = MultiSignature::Sr25519(signature);
    let tx = state
        .partial_tx
        .sign_with_account_and_signature(&account_id, &multi_sig);
    let tx_hash = format!("0x{}", hex_encode(tx.hash().as_ref()));
    let tx_hash_for_task = tx_hash.clone();

    if let Ok(mut map) = tx_state().lock() {
        prune_tx_state(&mut map, now_secs());
        map.insert(
            tx_hash.clone(),
            TxRuntimeState {
                updated_at: now_secs(),
                status: STATUS_PENDING.to_string(),
                failure_reason: None,
            },
        );
    }

    tokio::spawn(async move {
        let result = tx.submit_and_watch().await;
        match result {
            Ok(progress) => match progress.wait_for_finalized_success().await {
                Ok(_) => update_tx_runtime(&tx_hash_for_task, STATUS_CONFIRMED, None),
                Err(_) => update_tx_runtime(
                    &tx_hash_for_task,
                    STATUS_FAILED,
                    Some(FAIL_REASON_EXECUTION.to_string()),
                ),
            },
            Err(_) => update_tx_runtime(
                &tx_hash_for_task,
                STATUS_FAILED,
                Some(FAIL_REASON_BROADCAST.to_string()),
            ),
        }
    });

    Ok(TxSubmitData {
        tx_hash: Some(tx_hash),
        status: STATUS_PENDING.to_string(),
        failure_reason: None,
    })
}

pub fn get_tx_status(tx_hash: &str) -> Result<TxStatusData, ApiError> {
    let state = match tx_state().lock() {
        Ok(mut map) => {
            prune_tx_state(&mut map, now_secs());
            map.get(tx_hash).cloned()
        }
        Err(_) => None,
    };

    let Some(state) = state else {
        return Err(ApiError::new(3004, FAIL_REASON_NOT_FOUND));
    };

    Ok(TxStatusData {
        tx_hash: tx_hash.to_string(),
        status: state.status,
        failure_reason: state.failure_reason,
        updated_at: state.updated_at,
    })
}

fn update_tx_runtime(tx_hash: &str, status: &str, failure_reason: Option<String>) {
    if let Ok(mut map) = tx_state().lock() {
        prune_tx_state(&mut map, now_secs());
        map.insert(
            tx_hash.to_string(),
            TxRuntimeState {
                updated_at: now_secs(),
                status: status.to_string(),
                failure_reason,
            },
        );
    }
}

fn prune_tx_state(map: &mut HashMap<String, TxRuntimeState>, now: i64) {
    map.retain(|_, item| now.saturating_sub(item.updated_at) < TX_STATE_TTL_SECS);
    if map.len() <= TX_STATE_MAX {
        return;
    }
    let mut entries = map
        .iter()
        .map(|(k, v)| (k.clone(), v.updated_at))
        .collect::<Vec<_>>();
    entries.sort_by_key(|(_, ts)| *ts);
    let overflow = map.len() - TX_STATE_MAX;
    for (key, _) in entries.into_iter().take(overflow) {
        map.remove(&key);
    }
}

fn validate_prepare_request(req: &TxPrepareRequest) -> Result<(), ApiError> {
    if req.from_address.trim().is_empty()
        || req.to_address.trim().is_empty()
        || req.pubkey_hex.trim().is_empty()
    {
        return Err(ApiError::new(1001, "missing required tx fields"));
    }
    if req.amount <= 0.0 || !req.amount.is_finite() {
        return Err(ApiError::new(1001, "invalid amount"));
    }
    if req.symbol.trim().is_empty() {
        return Err(ApiError::new(1001, "missing symbol"));
    }
    Ok(())
}

fn parse_amount_to_fen(amount: f64) -> Result<u128, ApiError> {
    if !amount.is_finite() || amount <= 0.0 {
        return Err(ApiError::new(1001, "invalid amount"));
    }
    let raw = (amount * AMOUNT_DECIMALS).round();
    if raw <= 0.0 {
        return Err(ApiError::new(1001, "invalid amount"));
    }
    Ok(raw as u128)
}

async fn chain_client() -> Result<OnlineClient<PolkadotConfig>, ApiError> {
    let url = env::var("CHAIN_WS_URL")
        .ok()
        .or_else(|| env::var("CHAIN_RPC_URL").ok())
        .map(|v| normalize_ws_url(v.trim()))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_CHAIN_WS_URL.to_string());
    OnlineClient::<PolkadotConfig>::from_url(url)
        .await
        .map_err(|_| ApiError::new(5002, "chain websocket connect failed"))
}

fn normalize_ws_url(input: &str) -> String {
    if let Some(rest) = input.strip_prefix("http://") {
        return format!("ws://{rest}");
    }
    if let Some(rest) = input.strip_prefix("https://") {
        return format!("wss://{rest}");
    }
    input.to_string()
}

fn generate_prepared_id(pubkey: &[u8; 32], signer_payload: &[u8]) -> String {
    let mut preimage = Vec::with_capacity(pubkey.len() + signer_payload.len() + 8);
    preimage.extend_from_slice(pubkey);
    preimage.extend_from_slice(signer_payload);
    preimage.extend_from_slice(&now_secs().to_le_bytes());
    let digest = blake2::Blake2b512::digest(preimage);
    hex_encode(&digest[..16])
}

fn parse_account_to_pubkey(input: &str) -> Result<[u8; 32], ApiError> {
    if let Ok(pubkey) = parse_pubkey_hex(input) {
        return Ok(pubkey);
    }
    parse_ss58_account(input)
}

fn parse_pubkey_hex(input: &str) -> Result<[u8; 32], ApiError> {
    let mut v = input.trim().to_lowercase();
    if v.starts_with("0x") {
        v = v[2..].to_string();
    }
    if v.len() != 64 {
        return Err(ApiError::new(1001, "invalid pubkey_hex length"));
    }
    let bytes = hex_decode_strip_0x(&v).map_err(|_| ApiError::new(1001, "invalid pubkey_hex"))?;
    if bytes.len() != 32 {
        return Err(ApiError::new(1001, "invalid pubkey_hex bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn parse_signature(input: &str) -> Result<[u8; 64], ApiError> {
    let bytes =
        hex_decode_strip_0x(input).map_err(|_| ApiError::new(1001, "invalid signature_hex"))?;
    if bytes.len() != 64 {
        return Err(ApiError::new(1001, "invalid signature_hex length"));
    }
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn parse_ss58_account(input: &str) -> Result<[u8; 32], ApiError> {
    const CHECKSUM_LEN: usize = 2;
    const PUBKEY_LEN: usize = 32;
    const PREFIX: &[u8] = b"SS58PRE";

    let raw = bs58::decode(input.trim())
        .into_vec()
        .map_err(|_| ApiError::new(1001, "invalid account: bad ss58"))?;

    let prefix_len = match raw.first().copied() {
        Some(first) if first <= 63 => 1usize,
        Some(first) if first & 0b0100_0000 != 0 => 2usize,
        _ => return Err(ApiError::new(1001, "invalid account: unsupported ss58")),
    };

    if raw.len() != prefix_len + PUBKEY_LEN + CHECKSUM_LEN {
        return Err(ApiError::new(1001, "invalid account: ss58 length"));
    }

    let payload_len = raw.len() - CHECKSUM_LEN;
    let payload = &raw[..payload_len];
    let checksum = &raw[payload_len..];

    let mut preimage = Vec::with_capacity(PREFIX.len() + payload.len());
    preimage.extend_from_slice(PREFIX);
    preimage.extend_from_slice(payload);
    let digest = blake2::Blake2b512::digest(&preimage);
    if digest[..CHECKSUM_LEN] != checksum[..] {
        return Err(ApiError::new(1001, "invalid account: ss58 checksum"));
    }

    let pubkey_start = prefix_len;
    let pubkey_end = pubkey_start + PUBKEY_LEN;
    let mut out = [0u8; PUBKEY_LEN];
    out.copy_from_slice(&raw[pubkey_start..pubkey_end]);
    Ok(out)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn hex_decode_strip_0x(input: &str) -> Result<Vec<u8>, ()> {
    let s = input.strip_prefix("0x").unwrap_or(input);
    if s.len() % 2 != 0 {
        return Err(());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = from_hex_nibble(bytes[i]).ok_or(())?;
        let lo = from_hex_nibble(bytes[i + 1]).ok_or(())?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn from_hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}
