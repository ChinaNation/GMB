use std::{
    collections::{HashMap, HashSet},
    env,
    hash::Hasher,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use twox_hash::XxHash64;

use crate::{
    errors::ApiError,
    models::{AdminCatalogData, AdminCatalogEntryData},
};

const DEFAULT_CHAIN_RPC_URL: &str = "http://127.0.0.1:9944";
const ADMIN_KEYS_PAGE_SIZE: u32 = 256;
const MAX_PAGE_ROUNDS: usize = 256;

#[derive(Deserialize)]
struct InstitutionSeedRow {
    shenfen_id: String,
    institution_name: String,
    org: String,
}

#[derive(Clone)]
struct InstitutionSeedMeta {
    institution_name: String,
    org: String,
}

static INSTITUTION_SEED: OnceLock<HashMap<[u8; 48], InstitutionSeedMeta>> = OnceLock::new();

pub async fn fetch_admin_catalog() -> Result<AdminCatalogData, ApiError> {
    let rpc_url = env::var("CHAIN_RPC_URL").unwrap_or_else(|_| DEFAULT_CHAIN_RPC_URL.to_string());
    let prefix = format!("0x{}", hex_encode(&current_admins_storage_prefix()));
    let prefix_bytes = current_admins_storage_prefix();
    let keys = fetch_all_admin_keys(&rpc_url, &prefix).await?;
    let seed = institution_seed_map();

    let mut institutions = HashSet::<String>::new();
    let mut entries = Vec::<AdminCatalogEntryData>::new();
    for key_hex in keys {
        let key_bytes = match hex_decode_strip_0x(&key_hex) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if key_bytes.len() < 96 || !key_bytes.starts_with(&prefix_bytes) {
            continue;
        }

        let mut institution_id = [0u8; 48];
        institution_id.copy_from_slice(&key_bytes[(key_bytes.len() - 48)..]);
        let institution_id_hex = format!("0x{}", hex_encode(&institution_id));
        institutions.insert(institution_id_hex.clone());

        let storage = state_get_storage(&rpc_url, &key_hex).await?;
        let Some(storage_hex) = storage else {
            continue;
        };
        let admins = decode_admin_pubkeys_from_storage(&storage_hex)?;
        if admins.is_empty() {
            continue;
        }

        let (institution_name, org) = if let Some(meta) = seed.get(&institution_id) {
            (meta.institution_name.clone(), meta.org.clone())
        } else {
            (
                format!("未知机构({})", &institution_id_hex[2..14]),
                "unknown".to_string(),
            )
        };
        let role_name = format!("{}管理员", institution_name);
        for pubkey_hex in admins {
            entries.push(AdminCatalogEntryData {
                pubkey_hex,
                role_name: role_name.clone(),
                institution_name: institution_name.clone(),
                institution_id_hex: institution_id_hex.clone(),
                org: org.clone(),
            });
        }
    }

    entries.sort_by(|a, b| {
        a.role_name
            .cmp(&b.role_name)
            .then_with(|| a.pubkey_hex.cmp(&b.pubkey_hex))
    });

    Ok(AdminCatalogData {
        source: "chain",
        updated_at: now_secs(),
        institution_count: institutions.len() as u32,
        admin_count: entries.len() as u32,
        entries,
    })
}

async fn fetch_all_admin_keys(rpc_url: &str, prefix_hex: &str) -> Result<Vec<String>, ApiError> {
    let mut out = Vec::<String>::new();
    let mut start_key: Option<String> = None;
    for _ in 0..MAX_PAGE_ROUNDS {
        let params = json!([prefix_hex, ADMIN_KEYS_PAGE_SIZE, start_key, Value::Null]);
        let payload = rpc_call(rpc_url, "state_getKeysPaged", params).await?;
        let page = payload
            .as_array()
            .ok_or(ApiError::new(5302, "chain rpc keys response invalid"))?;
        if page.is_empty() {
            break;
        }

        for it in page {
            if let Some(key) = it.as_str() {
                out.push(key.to_string());
            }
        }

        start_key = page.last().and_then(Value::as_str).map(|v| v.to_string());
        if page.len() < ADMIN_KEYS_PAGE_SIZE as usize {
            break;
        }
    }
    Ok(out)
}

async fn state_get_storage(rpc_url: &str, key_hex: &str) -> Result<Option<String>, ApiError> {
    let payload = rpc_call(rpc_url, "state_getStorage", json!([key_hex])).await?;
    if payload.is_null() {
        return Ok(None);
    }
    let Some(s) = payload.as_str() else {
        return Err(ApiError::new(5303, "chain rpc storage response invalid"));
    };
    if s == "0x" {
        return Ok(None);
    }
    Ok(Some(s.to_string()))
}

async fn rpc_call(rpc_url: &str, method: &str, params: Value) -> Result<Value, ApiError> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let client = Client::new();
    let resp = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(|_| ApiError::new(5301, "chain rpc request failed"))?;
    if !resp.status().is_success() {
        return Err(ApiError::new(5301, "chain rpc http status failed"));
    }

    let payload = resp
        .json::<Value>()
        .await
        .map_err(|_| ApiError::new(5301, "chain rpc parse failed"))?;
    if payload.get("error").is_some() {
        return Err(ApiError::new(5301, "chain rpc returned error"));
    }
    payload
        .get("result")
        .cloned()
        .ok_or(ApiError::new(5301, "chain rpc result missing"))
}

fn decode_admin_pubkeys_from_storage(storage_hex: &str) -> Result<Vec<String>, ApiError> {
    let raw = hex_decode_strip_0x(storage_hex)
        .map_err(|_| ApiError::new(5304, "decode currentAdmins failed"))?;
    if raw.is_empty() {
        return Ok(Vec::new());
    }

    let (len, offset) =
        decode_scale_compact_len(&raw).ok_or(ApiError::new(5304, "decode currentAdmins failed"))?;
    let expected = offset.saturating_add(len.saturating_mul(32));
    if raw.len() < expected {
        return Err(ApiError::new(5304, "decode currentAdmins failed"));
    }

    let mut out = Vec::<String>::with_capacity(len);
    for i in 0..len {
        let start = offset + i * 32;
        let end = start + 32;
        out.push(hex_encode(&raw[start..end]));
    }
    Ok(out)
}

fn decode_scale_compact_len(bytes: &[u8]) -> Option<(usize, usize)> {
    let first = *bytes.first()?;
    let mode = first & 0b11;
    match mode {
        0b00 => Some(((first >> 2) as usize, 1)),
        0b01 => {
            if bytes.len() < 2 {
                return None;
            }
            let v = u16::from_le_bytes([bytes[0], bytes[1]]);
            Some(((v >> 2) as usize, 2))
        }
        0b10 => {
            if bytes.len() < 4 {
                return None;
            }
            let v = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            Some(((v >> 2) as usize, 4))
        }
        0b11 => {
            let bytes_len = ((first >> 2) as usize) + 4;
            if bytes.len() < (1 + bytes_len) || bytes_len > 8 {
                return None;
            }
            let mut v: usize = 0;
            for i in 0..bytes_len {
                v |= (bytes[1 + i] as usize) << (8 * i);
            }
            Some((v, 1 + bytes_len))
        }
        _ => None,
    }
}

fn institution_seed_map() -> &'static HashMap<[u8; 48], InstitutionSeedMeta> {
    INSTITUTION_SEED.get_or_init(|| {
        let rows: Vec<InstitutionSeedRow> =
            serde_json::from_str(include_str!("admin_catalog_seed.json"))
                .expect("admin catalog seed json must be valid");
        let mut map = HashMap::<[u8; 48], InstitutionSeedMeta>::new();
        for row in rows {
            if let Some(id) = shenfen_id_to_fixed48(&row.shenfen_id) {
                map.insert(
                    id,
                    InstitutionSeedMeta {
                        institution_name: row.institution_name,
                        org: row.org,
                    },
                );
            }
        }
        map
    })
}

fn shenfen_id_to_fixed48(s: &str) -> Option<[u8; 48]> {
    let raw = s.as_bytes();
    if raw.len() > 48 {
        return None;
    }
    let mut out = [0u8; 48];
    out[..raw.len()].copy_from_slice(raw);
    Some(out)
}

fn current_admins_storage_prefix() -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..16].copy_from_slice(&twox_128(b"AdminsOriginGov"));
    out[16..].copy_from_slice(&twox_128(b"CurrentAdmins"));
    out
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
