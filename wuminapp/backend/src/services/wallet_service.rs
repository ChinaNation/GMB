use std::{
    env,
    hash::Hasher,
    time::{SystemTime, UNIX_EPOCH},
};

use blake2::{
    digest::{Digest, Update, VariableOutput},
    Blake2bVar,
};
use reqwest::Client;
use serde_json::{json, Value};
use twox_hash::XxHash64;

use crate::{errors::ApiError, models::WalletBalanceData};

const SYMBOL_CITIZEN_COIN: &str = "CIT";
const DEFAULT_CHAIN_RPC_URL: &str = "http://127.0.0.1:9944";
const AMOUNT_DECIMALS: f64 = 100.0;

pub async fn get_wallet_balance(
    account: &str,
    pubkey_hex: Option<&str>,
) -> Result<WalletBalanceData, ApiError> {
    let normalized_account = account.trim();
    if normalized_account.is_empty() {
        return Err(ApiError::new(1001, "missing account"));
    }

    let account_id = normalize_account_to_pubkey(normalized_account, pubkey_hex)?;

    let rpc_url = env::var("CHAIN_RPC_URL").unwrap_or_else(|_| DEFAULT_CHAIN_RPC_URL.to_string());
    let storage_key = system_account_storage_key(&account_id);
    let storage_hex = format!("0x{}", hex_encode(&storage_key));
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "state_getStorage",
        "params": [storage_hex]
    });

    let client = Client::new();
    let resp = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(|_| ApiError::new(5001, "chain rpc request failed"))?;
    if !resp.status().is_success() {
        return Err(ApiError::new(5001, "chain rpc http status failed"));
    }

    let payload = resp
        .json::<Value>()
        .await
        .map_err(|_| ApiError::new(5001, "chain rpc response parse failed"))?;
    if payload.get("error").is_some() {
        return Err(ApiError::new(5001, "chain rpc returned error"));
    }

    let free = match payload.get("result").and_then(Value::as_str) {
        Some("0x") | None => 0u128,
        Some(hex) => {
            let bytes =
                hex_decode_strip_0x(hex).map_err(|_| ApiError::new(5001, "invalid storage hex"))?;
            decode_free_from_account_info(&bytes)
                .map_err(|_| ApiError::new(5001, "invalid account info data"))?
        }
    };

    let balance = (free as f64) / AMOUNT_DECIMALS;
    Ok(WalletBalanceData {
        account: normalized_account.to_string(),
        balance,
        symbol: SYMBOL_CITIZEN_COIN,
        updated_at: now_secs(),
    })
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

fn normalize_account_to_pubkey(
    account: &str,
    pubkey_hex: Option<&str>,
) -> Result<[u8; 32], ApiError> {
    if let Some(pubkey) = pubkey_hex {
        return parse_pubkey_hex(pubkey);
    }

    if let Ok(parsed) = parse_pubkey_hex(account) {
        return Ok(parsed);
    }

    parse_ss58_account(account)
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

fn system_account_storage_key(account: &[u8; 32]) -> Vec<u8> {
    // System.Account = Twox128("System") ++ Twox128("Account") ++ Blake2_128Concat(account)
    let mut out = Vec::with_capacity(16 + 16 + 16 + 32);
    out.extend_from_slice(&twox_128(b"System"));
    out.extend_from_slice(&twox_128(b"Account"));
    out.extend_from_slice(&blake2_128(account));
    out.extend_from_slice(account);
    out
}

fn decode_free_from_account_info(bytes: &[u8]) -> Result<u128, ()> {
    // AccountInfo layout prefix(16) + AccountData.free(u128 LE)
    const MIN_LEN: usize = 32;
    if bytes.len() < MIN_LEN {
        return Err(());
    }
    let mut free = [0u8; 16];
    free.copy_from_slice(&bytes[16..32]);
    Ok(u128::from_le_bytes(free))
}

fn twox_128(data: &[u8]) -> [u8; 16] {
    let mut h0 = XxHash64::with_seed(0);
    h0.write(data);
    let mut h1 = XxHash64::with_seed(1);
    h1.write(data);

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h0.finish().to_le_bytes());
    out[8..].copy_from_slice(&h1.finish().to_le_bytes());
    out
}

fn blake2_128(data: &[u8]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut hasher =
        Blake2bVar::new(16).expect("creating blake2b-128 hasher with fixed size should succeed");
    hasher.update(data);
    hasher
        .finalize_variable(&mut out)
        .expect("writing blake2b-128 into 16-byte buffer should succeed");
    out
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
