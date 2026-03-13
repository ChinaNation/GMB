use crate::{
    settings::{
        address_utils::{decode_hex_32_with_optional_0x, decode_ss58_prefix},
        device_password,
    },
    shared::{constants::{SS58_PREFIX, EXPECTED_SS58_PREFIX}, keystore, rpc, security, validation::normalize_wallet_address},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    hash::Hasher,
    io::ErrorKind,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};
use subxt_signer::{sr25519::Keypair as Sr25519Keypair, SecretUri};
use tauri::AppHandle;
use twox_hash::XxHash64;
use zeroize::Zeroizing;

const POWR_KEY_TYPE_HEX_PREFIX: &str = "706f7772";
const DEFAULT_CHAIN_WS_URL: &str = "ws://127.0.0.1:9944";
const DEFAULT_CHAIN_ID: &str = "citizenchain";
const LEGACY_MINER_SURI_FILENAME: &str = "miner-suri.txt";
const KEYCHAIN_ACCOUNT_MINER_SURI: &str = "powr-miner-suri";
const REWARD_BIND_TIMEOUT_SECS: u64 = 45;
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_RPC_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 前端展示的手续费收款地址配置。
pub struct RewardWallet {
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredWallet {
    address: String,
}

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    keystore::node_data_dir(app)
}

fn reward_wallet_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("reward-wallet.json"))
}

fn legacy_miner_suri_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join(LEGACY_MINER_SURI_FILENAME))
}

pub(crate) fn load_reward_wallet(app: &AppHandle) -> Result<Option<String>, String> {
    let path = reward_wallet_path(app)?;
    let raw = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("read reward wallet failed: {e}")),
    };
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

// 迁移旧明文文件时只做一次性读取，然后立即删除，避免继续保留历史明文秘密。
fn load_legacy_miner_suri_file(path: &Path) -> Result<Option<String>, String> {
    let raw = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("read legacy miner suri failed: {e}")),
    };
    let suri = raw.trim().to_string();
    if suri.is_empty() {
        return Ok(None);
    }
    Ok(Some(suri))
}

fn parse_keystore_secret(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if trimmed.starts_with('"')
        || trimmed.starts_with('{')
        || trimmed.starts_with('[')
        || trimmed.starts_with('\'')
    {
        let parsed = serde_json::from_str::<String>(trimmed)
            .map_err(|e| format!("powr keystore secret 格式无效: {e}"))?;
        return Ok(parsed.trim().to_string());
    }
    Ok(trimmed.to_string())
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
        let file_type = chain_dir
            .file_type()
            .map_err(|e| format!("read chain dir file type failed: {e}"))?;
        if file_type.is_symlink() || !file_type.is_dir() {
            continue;
        }
        let keystore_dir = chain_dir.path().join("keystore");
        if let Ok(meta) = fs::symlink_metadata(&keystore_dir) {
            if meta.file_type().is_symlink() {
                continue;
            }
        }
        if !keystore_dir.is_dir() {
            continue;
        }
        let entries = fs::read_dir(&keystore_dir)
            .map_err(|e| format!("read keystore dir failed ({}): {e}", security::sanitize_path(&keystore_dir)))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read keystore file entry failed: {e}"))?;
            let file_type = entry
                .file_type()
                .map_err(|e| format!("read keystore file type failed: {e}"))?;
            if file_type.is_symlink() {
                continue;
            }
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

fn miner_account_hex_from_keystore_filename(name: &str) -> Option<String> {
    let hex = name.strip_prefix(POWR_KEY_TYPE_HEX_PREFIX)?;
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("0x{}", hex.to_ascii_lowercase()))
}

pub(crate) fn local_powr_miner_account_hex(app: &AppHandle) -> Result<Option<String>, String> {
    for path in collect_powr_keystore_files(app)? {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if let Some(account_hex) = miner_account_hex_from_keystore_filename(name) {
            return Ok(Some(account_hex));
        }
    }
    Ok(None)
}

fn keystore_dirs_for_powr(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let chains_root = node_data_dir(app)?.join("chains");
    fs::create_dir_all(&chains_root).map_err(|e| format!("create chains dir failed: {e}"))?;

    let mut dirs: Vec<PathBuf> = Vec::new();
    if chains_root.exists() {
        let entries =
            fs::read_dir(&chains_root).map_err(|e| format!("read chains dir failed: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read chain dir entry failed: {e}"))?;
            let file_type = entry
                .file_type()
                .map_err(|e| format!("read chain dir file type failed: {e}"))?;
            if file_type.is_symlink() || !file_type.is_dir() {
                continue;
            }
            let candidate = entry.path().join("keystore");
            if let Ok(meta) = fs::symlink_metadata(&candidate) {
                if meta.file_type().is_symlink() {
                    continue;
                }
            }
            dirs.push(candidate);
        }
    }

    dirs.push(chains_root.join(DEFAULT_CHAIN_ID).join("keystore"));
    dirs.sort();
    dirs.dedup();
    for dir in &dirs {
        fs::create_dir_all(dir)
            .map_err(|e| format!("create powr keystore dir failed ({}): {e}", security::sanitize_path(dir)))?;
    }
    Ok(dirs)
}

fn powr_keystore_filename(pubkey_hex: &str) -> String {
    format!("{POWR_KEY_TYPE_HEX_PREFIX}{pubkey_hex}")
}

fn powr_pubkey_hex_from_suri(suri: &str) -> Result<String, String> {
    let secret_uri = SecretUri::from_str(suri).map_err(|e| format!("矿工签名密钥解析失败: {e}"))?;
    let signer =
        Sr25519Keypair::from_uri(&secret_uri).map_err(|e| format!("矿工签名密钥加载失败: {e}"))?;
    Ok(hex::encode(signer.public_key().0))
}

fn write_powr_key_to_keystore(app: &AppHandle, suri: &str, pubkey_hex: &str) -> Result<(), String> {
    let normalized = suri.trim();
    let encoded = serde_json::to_string(normalized)
        .map_err(|e| format!("encode powr keystore secret failed: {e}"))?;
    let content = format!("{encoded}\n");
    let filename = powr_keystore_filename(pubkey_hex);
    for dir in keystore_dirs_for_powr(app)? {
        let path = dir.join(&filename);
        security::write_secret_text_atomic(&path, &content)
            .map_err(|e| format!("write powr keystore file failed ({}): {e}", security::sanitize_path(&path)))?;
    }
    Ok(())
}

fn remove_other_powr_keys(app: &AppHandle, keep_filename: &str) -> Result<(), String> {
    for path in collect_powr_keystore_files(app)? {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == keep_filename {
            continue;
        }
        fs::remove_file(&path).map_err(|e| {
            format!(
                "remove stale powr keystore file failed ({}): {e}",
                security::sanitize_path(&path)
            )
        })?;
    }
    Ok(())
}

fn load_powr_suri_from_keystore(app: &AppHandle) -> Result<Option<String>, String> {
    let files = collect_powr_keystore_files(app)?;
    for path in files {
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("read powr keystore file failed ({}): {e}", security::sanitize_path(&path)))?;
        let suri = parse_keystore_secret(&raw)?;
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
    let legacy = load_legacy_miner_suri_file(&legacy_path)?;
    if legacy.is_none() {
        return Ok(());
    }
    let has_secure = security::secure_store_get(KEYCHAIN_ACCOUNT_MINER_SURI)?.is_some();
    if !has_secure {
        if let Some(suri) = legacy {
            save_miner_suri_secure(&suri, unlock_password)?;
        }
    }

    if let Err(e) = fs::remove_file(&legacy_path) {
        if e.kind() != ErrorKind::NotFound {
            return Err(format!("remove legacy miner suri file failed: {e}"));
        }
    }
    Ok(())
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
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"SS58PRE");
    hasher.update(without_checksum);
    let hash = hasher.finalize();
    if checksum != &hash.as_bytes()[..2] {
        return Err("SS58 地址校验和无效".to_string());
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&data[prefix_len..prefix_len + 32]);
    Ok(out)
}

fn account_id_from_address(address: &str) -> Result<[u8; 32], String> {
    if address.starts_with("0x") {
        return decode_hex_32_with_optional_0x(address);
    }
    decode_ss58_account_id(address)
}

fn miner_account_id(app: &AppHandle, unlock_password: &str) -> Result<[u8; 32], String> {
    let miner_suri = Zeroizing::new(ensure_miner_suri(app, unlock_password)?);
    let secret_uri =
        SecretUri::from_str(&miner_suri).map_err(|e| format!("矿工签名密钥解析失败: {e}"))?;
    let signer =
        Sr25519Keypair::from_uri(&secret_uri).map_err(|e| format!("矿工签名密钥加载失败: {e}"))?;
    Ok(signer.public_key().0)
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

fn blake3_128(input: &[u8]) -> [u8; 16] {
    let hash = blake3::hash(input);
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash.as_bytes()[..16]);
    out
}

fn ensure_expected_reward_wallet_rpc_node() -> Result<(), String> {
    let properties = rpc::rpc_post(
        "system_properties",
        serde_json::Value::Array(vec![]),
        RPC_REQUEST_TIMEOUT,
        MAX_RPC_RESPONSE_BYTES,
    )?;
    let ss58 = properties
        .get("ss58Format")
        .and_then(|v| {
            if let Some(raw) = v.as_u64() {
                Some(raw)
            } else {
                v.as_str().and_then(|s| s.parse::<u64>().ok())
            }
        })
        .ok_or_else(|| "RPC 节点缺少 ss58Format".to_string())?;
    if ss58 != EXPECTED_SS58_PREFIX {
        return Err(format!("RPC 链前缀不匹配：expected=2027, got={ss58}"));
    }

    let name = rpc::rpc_post(
        "system_name",
        serde_json::Value::Array(vec![]),
        RPC_REQUEST_TIMEOUT,
        MAX_RPC_RESPONSE_BYTES,
    )?
    .as_str()
    .map(str::trim)
    .unwrap_or("")
    .to_string();
    if name.is_empty() {
        return Err("RPC 节点名称为空".to_string());
    }
    Ok(())
}

fn reward_wallet_storage_key(miner_account: &[u8; 32]) -> Vec<u8> {
    let mut key = Vec::with_capacity(16 + 16 + 16 + 32);
    key.extend_from_slice(&twox_128(b"FullnodePowReward"));
    key.extend_from_slice(&twox_128(b"RewardWalletByMiner"));
    key.extend_from_slice(&blake3_128(miner_account));
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

    // 迁移优先级：安全存储 -> 现有 keystore -> 新生成随机 seed。
    // 这样既兼容历史数据，又尽量保持矿工账户稳定不漂移。
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

pub(crate) fn ensure_powr_keystore_key(
    app: &AppHandle,
    unlock_password: &str,
) -> Result<(), String> {
    // `powr` 只保留当前矿工账户对应的一把密钥，避免旧 key 残留导致收益归属漂移。
    let suri = Zeroizing::new(ensure_miner_suri(app, unlock_password)?);
    let pubkey_hex = powr_pubkey_hex_from_suri(&suri)?;
    write_powr_key_to_keystore(app, &suri, &pubkey_hex)?;
    let keep_filename = powr_keystore_filename(&pubkey_hex);
    remove_other_powr_keys(app, &keep_filename)?;
    Ok(())
}

async fn sync_saved_reward_wallet_inner(
    app: &AppHandle,
    unlock_password: &str,
) -> Result<(), String> {
    let Some(saved_address) = load_reward_wallet(app)? else {
        return Ok(());
    };
    // 在提交链上绑定交易前，先确认当前 9944 端口确实属于目标链，
    // 避免把奖励地址绑定误发到错误节点或错误网络。
    ensure_expected_reward_wallet_rpc_node()?;
    let normalized = normalize_wallet_address(&saved_address)?;
    let target_wallet = account_id_from_address(&normalized)?;
    let miner_suri = Zeroizing::new(ensure_miner_suri(app, unlock_password)?);

    let secret_uri =
        SecretUri::from_str(&miner_suri).map_err(|e| format!("矿工签名密钥解析失败: {e}"))?;
    let signer =
        Sr25519Keypair::from_uri(&secret_uri).map_err(|e| format!("矿工签名密钥加载失败: {e}"))?;
    let signer_account = AccountId32(signer.public_key().0);
    if target_wallet == signer_account.0 {
        return Err("奖励钱包不能与矿工签名账户相同，请使用独立收款钱包".to_string());
    }

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
    tauri::async_runtime::block_on(async {
        tokio::time::timeout(
            std::time::Duration::from_secs(REWARD_BIND_TIMEOUT_SECS),
            sync_saved_reward_wallet_inner(app, unlock_password),
        )
        .await
        .map_err(|_| "链上奖励地址同步超时，请稍后重试".to_string())?
    })
}

#[tauri::command]
pub fn get_reward_wallet(app: AppHandle) -> Result<RewardWallet, String> {
    Ok(RewardWallet {
        address: load_reward_wallet(&app)?,
    })
}

#[tauri::command]
pub async fn set_reward_wallet(
    app: AppHandle,
    address: String,
    unlock_password: String,
) -> Result<RewardWallet, String> {
    let _ = security::append_audit_log(&app, "set_reward_wallet", "attempt");
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;
    let normalized = normalize_wallet_address(&address)?;
    let target_wallet = account_id_from_address(&normalized)?;
    let miner_account = miner_account_id(&app, unlock)?;
    if target_wallet == miner_account {
        return Err("奖励钱包不能与矿工签名账户相同，请使用独立收款钱包".to_string());
    }

    save_reward_wallet(&app, &normalized)?;
    ensure_powr_keystore_key(&app, unlock)?;

    let sync_result = tokio::time::timeout(
        std::time::Duration::from_secs(REWARD_BIND_TIMEOUT_SECS),
        sync_saved_reward_wallet_inner(&app, unlock),
    )
    .await;
    let sync_result = match sync_result {
        Ok(v) => v,
        Err(_) => {
            let _ =
                security::append_audit_log(&app, "set_reward_wallet", "saved_chain_bind_timeout");
            return Err("地址已保存，但链上绑定超时，请稍后重试".to_string());
        }
    };
    if let Err(err) = sync_result {
        let _ = security::append_audit_log(&app, "set_reward_wallet", "saved_chain_bind_failed");
        return Err(format!("地址已保存，但链上绑定失败：{err}"));
    }
    let _ = security::append_audit_log(&app, "set_reward_wallet", "success");

    Ok(RewardWallet {
        address: Some(normalized),
    })
}
