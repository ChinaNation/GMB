use crate::{settings::security, validation::normalize_wallet_address};
use blake2::{Blake2b512, Digest};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    hash::Hasher,
    path::{Path, PathBuf},
    str::FromStr,
};
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};
use subxt_signer::{sr25519::Keypair as Sr25519Keypair, SecretUri};
use tauri::AppHandle;
use twox_hash::XxHash64;

const SS58_PREFIX: u16 = 2027;
const POWR_KEY_TYPE_HEX_PREFIX: &str = "706f7772";
const DEFAULT_CHAIN_WS_URL: &str = "ws://127.0.0.1:9944";
const LEGACY_MINER_SURI_FILENAME: &str = "miner-suri.txt";
const KEYCHAIN_ACCOUNT_MINER_SURI: &str = "powr-miner-suri";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardWallet {
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredWallet {
    address: String,
}

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = security::app_data_dir(app)?.join("node-data");
    fs::create_dir_all(&path).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(path)
}

fn reward_wallet_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("reward-wallet.json"))
}

fn legacy_miner_suri_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join(LEGACY_MINER_SURI_FILENAME))
}

pub(crate) fn load_reward_wallet(app: &AppHandle) -> Result<Option<String>, String> {
    let path = reward_wallet_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read reward wallet failed: {e}"))?;
    let stored: StoredWallet =
        serde_json::from_str(&raw).map_err(|e| format!("parse reward wallet failed: {e}"))?;
    let address = stored.address.trim().to_string();
    if address.is_empty() {
        return Ok(None);
    }
    Ok(Some(address))
}

fn save_reward_wallet(app: &AppHandle, address: &str) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredWallet {
        address: address.to_string(),
    })
    .map_err(|e| format!("encode reward wallet failed: {e}"))?;
    security::write_text_atomic(&reward_wallet_path(app)?, &format!("{raw}\n"))
        .map_err(|e| format!("write reward wallet failed: {e}"))
}

fn load_legacy_miner_suri_file(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw =
        fs::read_to_string(path).map_err(|e| format!("read legacy miner suri failed: {e}"))?;
    let suri = raw.trim().to_string();
    if suri.is_empty() {
        return Ok(None);
    }
    Ok(Some(suri))
}

fn parse_keystore_secret(raw: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<String>(raw) {
        return parsed.trim().to_string();
    }
    raw.trim().trim_matches('"').to_string()
}

fn collect_powr_keystore_files(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let chains_root = node_data_dir(app)?.join("chains");
    if !chains_root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let chain_dirs =
        fs::read_dir(chains_root).map_err(|e| format!("read chains dir failed: {e}"))?;
    for chain_dir in chain_dirs {
        let chain_dir = chain_dir.map_err(|e| format!("read chain dir entry failed: {e}"))?;
        let keystore_dir = chain_dir.path().join("keystore");
        if !keystore_dir.is_dir() {
            continue;
        }
        let entries = fs::read_dir(&keystore_dir)
            .map_err(|e| format!("read keystore dir failed ({}): {e}", keystore_dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read keystore file entry failed: {e}"))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if name.starts_with(POWR_KEY_TYPE_HEX_PREFIX) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn load_powr_suri_from_keystore(app: &AppHandle) -> Result<Option<String>, String> {
    let files = collect_powr_keystore_files(app)?;
    for path in files {
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("read powr keystore file failed ({}): {e}", path.display()))?;
        let suri = parse_keystore_secret(&raw);
        if !suri.is_empty() {
            return Ok(Some(suri));
        }
    }
    Ok(None)
}

fn load_miner_suri_secure(unlock_password: &str) -> Result<Option<String>, String> {
    let Some(enveloped) = security::secure_store_get(KEYCHAIN_ACCOUNT_MINER_SURI)? else {
        return Ok(None);
    };
    let suri = security::decrypt_secret_value(&enveloped, unlock_password)?;
    let normalized = suri.trim().to_string();
    if normalized.is_empty() {
        return Ok(None);
    }
    Ok(Some(normalized))
}

fn save_miner_suri_secure(suri: &str, unlock_password: &str) -> Result<(), String> {
    let normalized = suri.trim();
    if normalized.is_empty() {
        return Err("矿工签名密钥不能为空".to_string());
    }
    let encrypted = security::encrypt_secret_value(normalized, unlock_password)?;
    security::secure_store_set(KEYCHAIN_ACCOUNT_MINER_SURI, &encrypted)
}

fn migrate_legacy_miner_suri_file(app: &AppHandle, unlock_password: &str) -> Result<(), String> {
    let legacy_path = legacy_miner_suri_path(app)?;
    if !legacy_path.exists() {
        return Ok(());
    }

    let legacy = load_legacy_miner_suri_file(&legacy_path)?;
    let has_secure = security::secure_store_get(KEYCHAIN_ACCOUNT_MINER_SURI)?.is_some();
    if !has_secure {
        if let Some(suri) = legacy {
            save_miner_suri_secure(&suri, unlock_password)?;
        }
    }

    fs::remove_file(&legacy_path)
        .map_err(|e| format!("remove legacy miner suri file failed: {e}"))?;
    Ok(())
}

fn decode_hex_32(input: &str) -> Result<[u8; 32], String> {
    let mut value = input.trim().to_string();
    if let Some(stripped) = value.strip_prefix("0x") {
        value = stripped.to_string();
    }
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("hex 地址格式无效，应为 0x + 64 位十六进制".to_string());
    }
    let mut out = [0u8; 32];
    for (i, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let part = std::str::from_utf8(chunk).map_err(|_| "hex 地址格式无效".to_string())?;
        out[i] = u8::from_str_radix(part, 16).map_err(|_| "hex 地址格式无效".to_string())?;
    }
    Ok(out)
}

fn decode_ss58_prefix(data: &[u8]) -> Result<(u16, usize), String> {
    if data.is_empty() {
        return Err("SS58 地址为空".to_string());
    }
    let first = data[0];
    match first {
        0..=63 => Ok((first as u16, 1)),
        64..=127 => {
            if data.len() < 2 {
                return Err("SS58 地址格式无效".to_string());
            }
            let second = data[1];
            let prefix = (((first & 0x3f) as u16) << 2)
                | ((second as u16) >> 6)
                | (((second & 0x3f) as u16) << 8);
            Ok((prefix, 2))
        }
        _ => Err("SS58 地址格式无效".to_string()),
    }
}

fn decode_ss58_account_id(address: &str) -> Result<[u8; 32], String> {
    let data = bs58::decode(address)
        .into_vec()
        .map_err(|_| "SS58 地址解码失败".to_string())?;
    let (prefix, prefix_len) = decode_ss58_prefix(&data)?;
    if prefix != SS58_PREFIX {
        return Err("SS58 地址前缀无效，必须为 2027".to_string());
    }
    if data.len() < prefix_len + 32 + 2 {
        return Err("SS58 地址长度无效".to_string());
    }
    let payload_len = data.len() - prefix_len - 2;
    if payload_len != 32 {
        return Err("SS58 地址账户长度无效，必须是 32 字节".to_string());
    }

    let (without_checksum, checksum) = data.split_at(data.len() - 2);
    let mut hasher = Blake2b512::new();
    hasher.update(b"SS58PRE");
    hasher.update(without_checksum);
    let hash = hasher.finalize();
    if checksum != &hash[..2] {
        return Err("SS58 地址校验和无效".to_string());
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&data[prefix_len..prefix_len + 32]);
    Ok(out)
}

fn account_id_from_address(address: &str) -> Result<[u8; 32], String> {
    if address.starts_with("0x") {
        return decode_hex_32(address);
    }
    decode_ss58_account_id(address)
}

fn twox_128(input: &[u8]) -> [u8; 16] {
    let mut h1 = XxHash64::with_seed(0);
    h1.write(input);
    let mut h2 = XxHash64::with_seed(1);
    h2.write(input);

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h1.finish().to_le_bytes());
    out[8..].copy_from_slice(&h2.finish().to_le_bytes());
    out
}

fn blake2_128(input: &[u8]) -> [u8; 16] {
    let mut hasher = Blake2b512::new();
    hasher.update(input);
    let hash = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash[..16]);
    out
}

fn reward_wallet_storage_key(miner_account: &[u8; 32]) -> Vec<u8> {
    let mut key = Vec::with_capacity(16 + 16 + 16 + 32);
    key.extend_from_slice(&twox_128(b"FullnodePowReward"));
    key.extend_from_slice(&twox_128(b"RewardWalletByMiner"));
    key.extend_from_slice(&blake2_128(miner_account));
    key.extend_from_slice(miner_account);
    key
}

fn decode_storage_account_id(raw: &[u8]) -> Result<[u8; 32], String> {
    if raw.len() < 32 {
        return Err("链上 RewardWalletByMiner 数据长度无效".to_string());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&raw[..32]);
    Ok(out)
}

fn generate_random_hex_suri() -> String {
    let mut seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);
    format!("0x{}", hex::encode(seed))
}

pub(crate) fn ensure_miner_suri(app: &AppHandle, unlock_password: &str) -> Result<String, String> {
    let unlock = security::ensure_unlock_password(unlock_password)?;
    migrate_legacy_miner_suri_file(app, unlock)?;

    if let Some(suri) = load_miner_suri_secure(unlock)? {
        return Ok(suri);
    }
    if let Some(suri) = load_powr_suri_from_keystore(app)? {
        save_miner_suri_secure(&suri, unlock)?;
        return Ok(suri);
    }
    let suri = generate_random_hex_suri();
    save_miner_suri_secure(&suri, unlock)?;
    Ok(suri)
}

async fn sync_saved_reward_wallet_inner(
    app: &AppHandle,
    unlock_password: &str,
) -> Result<(), String> {
    let Some(saved_address) = load_reward_wallet(app)? else {
        return Ok(());
    };
    let normalized = normalize_wallet_address(&saved_address)?;
    let target_wallet = account_id_from_address(&normalized)?;
    let miner_suri = ensure_miner_suri(app, unlock_password)?;

    let secret_uri =
        SecretUri::from_str(&miner_suri).map_err(|e| format!("矿工签名密钥解析失败: {e}"))?;
    let signer =
        Sr25519Keypair::from_uri(&secret_uri).map_err(|e| format!("矿工签名密钥加载失败: {e}"))?;
    let signer_account = AccountId32(signer.public_key().0);

    let client = OnlineClient::<PolkadotConfig>::from_url(DEFAULT_CHAIN_WS_URL)
        .await
        .map_err(|e| format!("连接本地链节点失败: {e}"))?;

    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("读取链上状态失败: {e}"))?;
    let current_raw = storage
        .fetch_raw(reward_wallet_storage_key(&signer_account.0))
        .await
        .map_err(|e| format!("读取 RewardWalletByMiner 失败: {e}"))?;
    let current_wallet = match current_raw {
        Some(bytes) => Some(decode_storage_account_id(&bytes)?),
        None => None,
    };

    if current_wallet == Some(target_wallet) {
        return Ok(());
    }

    let call_name = if current_wallet.is_some() {
        "rebind_reward_wallet"
    } else {
        "bind_reward_wallet"
    };

    let payload = tx(
        "FullnodePowReward",
        call_name,
        vec![Value::from_bytes(target_wallet.to_vec())],
    );
    let mut partial = client
        .tx()
        .create_partial(&payload, &signer_account, Default::default())
        .await
        .map_err(|e| format!("构造 {} 交易失败: {e}", call_name))?;
    let signer_payload = partial.signer_payload();
    let signature = signer.sign(&signer_payload).0;
    let tx = partial
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));

    let progress = tx
        .submit_and_watch()
        .await
        .map_err(|e| format!("提交 {} 交易失败: {e}", call_name))?;
    progress
        .wait_for_finalized_success()
        .await
        .map_err(|e| format!("{} 交易执行失败: {e}", call_name))?;
    Ok(())
}

pub(crate) fn sync_saved_reward_wallet_binding(
    app: &AppHandle,
    unlock_password: &str,
) -> Result<(), String> {
    tauri::async_runtime::block_on(sync_saved_reward_wallet_inner(app, unlock_password))
}

#[tauri::command]
pub fn get_reward_wallet(app: AppHandle) -> Result<RewardWallet, String> {
    Ok(RewardWallet {
        address: load_reward_wallet(&app)?,
    })
}

#[tauri::command]
pub fn set_reward_wallet(
    app: AppHandle,
    address: String,
    unlock_password: String,
) -> Result<RewardWallet, String> {
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    security::verify_device_login_password(unlock)?;
    let normalized = normalize_wallet_address(&address)?;

    save_reward_wallet(&app, &normalized)?;
    let _ = ensure_miner_suri(&app, unlock)?;

    if let Err(err) = sync_saved_reward_wallet_binding(&app, unlock) {
        return Err(format!("地址已保存，但链上绑定失败：{err}"));
    }

    Ok(RewardWallet {
        address: Some(normalized),
    })
}
