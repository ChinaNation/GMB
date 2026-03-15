use crate::{
    settings::{
        address_utils::{decode_hex_32_with_optional_0x, decode_ss58_prefix},
        device_password,
    },
    shared::{
        constants::{EXPECTED_SS58_PREFIX, SS58_PREFIX},
        keystore, rpc, security,
        validation::normalize_wallet_address,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    hash::Hasher,
    io::ErrorKind,
    path::PathBuf,
    str::FromStr,
};
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};
use subxt_signer::{sr25519::Keypair as Sr25519Keypair, SecretUri};
use tauri::{AppHandle, Emitter};
use twox_hash::XxHash64;
use zeroize::Zeroizing;

const POWR_KEY_TYPE_HEX_PREFIX: &str = "706f7772";
const REWARD_BIND_TIMEOUT_SECS: u64 = 45;
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
        let entries = fs::read_dir(&keystore_dir).map_err(|e| {
            format!(
                "read keystore dir failed ({}): {e}",
                security::sanitize_path(&keystore_dir)
            )
        })?;
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

fn load_powr_suri_from_keystore(app: &AppHandle) -> Result<Option<String>, String> {
    let files = collect_powr_keystore_files(app)?;
    for path in files {
        let raw = fs::read_to_string(&path).map_err(|e| {
            format!(
                "read powr keystore file failed ({}): {e}",
                security::sanitize_path(&path)
            )
        })?;
        let suri = parse_keystore_secret(&raw)?;
        if !suri.is_empty() {
            return Ok(Some(suri));
        }
    }
    Ok(None)
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
    let hash = blake2b_simd::Params::new()
        .hash_length(64)
        .to_state()
        .update(b"SS58PRE")
        .update(without_checksum)
        .finalize();
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

fn miner_account_id(app: &AppHandle) -> Result<[u8; 32], String> {
    let miner_suri = load_miner_suri(app)?;
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

fn blake2b_128(input: &[u8]) -> [u8; 16] {
    let hash = blake2b_simd::Params::new()
        .hash_length(16)
        .hash(input);
    let mut out = [0u8; 16];
    out.copy_from_slice(hash.as_bytes());
    out
}

fn ensure_expected_reward_wallet_rpc_node() -> Result<(), String> {
    let properties = rpc::rpc_post(
        "system_properties",
        serde_json::Value::Array(vec![]),
        rpc::RPC_REQUEST_TIMEOUT,
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
        rpc::RPC_REQUEST_TIMEOUT,
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
    key.extend_from_slice(&blake2b_128(miner_account));
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

/// 从 keystore 读取矿工密钥。密钥由 node 进程（Substrate 框架）生成，nodeui 只读取。
fn load_miner_suri(app: &AppHandle) -> Result<Zeroizing<String>, String> {
    if let Some(suri) = load_powr_suri_from_keystore(app)? {
        return Ok(Zeroizing::new(suri));
    }
    Err("未找到矿工密钥，请先启动节点".to_string())
}

async fn sync_saved_reward_wallet_inner(app: &AppHandle) -> Result<(), String> {
    // 同步阻塞操作（reqwest::blocking、文件读取、密钥计算）必须在 spawn_blocking 中执行，
    // 避免在 async 上下文中 drop reqwest::blocking 内部的 tokio runtime 导致 panic。
    let app_clone = app.clone();
    let prep = tauri::async_runtime::spawn_blocking(move || {
        let Some(saved_address) = load_reward_wallet(&app_clone)? else {
            return Ok(None);
        };
        ensure_expected_reward_wallet_rpc_node()?;
        let normalized = normalize_wallet_address(&saved_address)?;
        let target_wallet = account_id_from_address(&normalized)?;
        let miner_suri = load_miner_suri(&app_clone)?;

        let secret_uri = SecretUri::from_str(&miner_suri)
            .map_err(|e| format!("矿工签名密钥解析失败: {e}"))?;
        let signer = Sr25519Keypair::from_uri(&secret_uri)
            .map_err(|e| format!("矿工签名密钥加载失败: {e}"))?;
        let signer_account = AccountId32(signer.public_key().0);

        // 诊断：比较从 SURI 推导的公钥和 keystore 文件名中的公钥
        let derived_hex = hex::encode(signer_account.0);
        let filename_hex = local_powr_miner_account_hex(&app_clone)?;
        eprintln!(
            "[reward-wallet-bind] derived_pubkey=0x{}, keystore_filename_pubkey={:?}",
            derived_hex,
            filename_hex,
        );
        if let Some(ref fh) = filename_hex {
            let fh_lower = fh.to_ascii_lowercase();
            let derived_lower = format!("0x{}", derived_hex.to_ascii_lowercase());
            if fh_lower != derived_lower {
                return Err(format!(
                    "密钥推导不一致：keystore 文件名公钥={}, SURI 推导公钥=0x{}，可能是 sp_core 与 subxt_signer 对助记词推导不兼容",
                    fh, derived_hex,
                ));
            }
        }

        if target_wallet == signer_account.0 {
            return Err("奖励钱包不能与矿工签名账户相同，请使用独立收款钱包".to_string());
        }
        Ok(Some((target_wallet, signer_account, signer)))
    })
    .await
    .map_err(|e| format!("sync prep task failed: {e}"))??;

    let Some((target_wallet, signer_account, signer)) = prep else {
        return Ok(());
    };

    let client = OnlineClient::<PolkadotConfig>::from_url(rpc::local_rpc_ws_url())
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

    // 诊断：提交前检查签名账户余额
    let balance_key = {
        let mut k = Vec::new();
        k.extend_from_slice(&twox_128(b"System"));
        k.extend_from_slice(&twox_128(b"Account"));
        k.extend_from_slice(&blake2b_128(&signer_account.0));
        k.extend_from_slice(&signer_account.0);
        k
    };
    let account_raw = storage
        .fetch_raw(balance_key)
        .await
        .map_err(|e| format!("查询签名账户信息失败: {e}"))?;
    eprintln!(
        "[reward-wallet-bind] signer=0x{}, account_exists={}, account_data_len={}",
        hex::encode(signer_account.0),
        account_raw.is_some(),
        account_raw.as_ref().map(|v| v.len()).unwrap_or(0),
    );

    tx.submit()
        .await
        .map_err(|e| format!("提交 {} 交易失败: {e}", call_name))?;
    Ok(())
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
    if let Err(e) = security::append_audit_log(&app, "set_reward_wallet", "attempt") {
        eprintln!("[审计] set_reward_wallet attempt 日志写入失败: {e}");
    }
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;
    let normalized = normalize_wallet_address(&address)?;
    let target_wallet = account_id_from_address(&normalized)?;
    let miner_account = miner_account_id(&app)?;
    if target_wallet == miner_account {
        return Err("奖励钱包不能与矿工签名账户相同，请使用独立收款钱包".to_string());
    }

    save_reward_wallet(&app, &normalized)?;

    // 链上绑定在后台异步执行，通过事件通知前端结果
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let sync_result = tokio::time::timeout(
            std::time::Duration::from_secs(REWARD_BIND_TIMEOUT_SECS),
            sync_saved_reward_wallet_inner(&app2),
        )
        .await;
        let (status, detail) = match sync_result {
            Ok(Ok(())) => ("success", String::new()),
            Ok(Err(err)) => ("failed", err),
            Err(_) => ("timeout", "链上绑定超时".to_string()),
        };
        if let Err(e) = security::append_audit_log(
            &app2,
            "set_reward_wallet",
            &format!("chain_bind_{status}"),
        ) {
            eprintln!("[审计] set_reward_wallet chain_bind_{status} 日志写入失败: {e}");
        }
        let _ = app2.emit(
            "reward-wallet-bind-result",
            serde_json::json!({ "status": status, "detail": detail }),
        );
    });

    Ok(RewardWallet {
        address: Some(normalized),
    })
}
