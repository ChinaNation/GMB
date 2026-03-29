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
use std::{fs, hash::Hasher, io::ErrorKind, path::PathBuf};
use tauri::{AppHandle, Emitter};
use twox_hash::XxHash64;

const POWR_KEY_TYPE_HEX_PREFIX: &str = "706f7772";
const REWARD_BIND_TIMEOUT_SECS: u64 = 45;
use crate::shared::constants::RPC_RESPONSE_LIMIT_LARGE;

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

/// 仅扫描默认链（citizenchain）的 keystore 目录中的 powr 文件。
/// 不遍历其他链目录，避免旧链残留 keystore 导致矿工身份错位。
fn collect_powr_keystore_files(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let keystore_dir = keystore::default_chain_keystore_dir(app)?;
    if !keystore_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
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
    let hash = blake2b_simd::Params::new().hash_length(16).hash(input);
    let mut out = [0u8; 16];
    out.copy_from_slice(hash.as_bytes());
    out
}

fn ensure_expected_reward_wallet_rpc_node() -> Result<(), String> {
    let properties = rpc::rpc_post(
        "system_properties",
        serde_json::Value::Array(vec![]),
        rpc::RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_LARGE,
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
        RPC_RESPONSE_LIMIT_LARGE,
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

/// 通过 node 端自定义 RPC 绑定或重绑奖励钱包。
/// node 端使用 keystore 中的矿工密钥直接签名并提交交易。
///
/// 本函数在以下两种场景被调用：
/// 1. 用户主动设置奖励钱包（`set_reward_wallet`）；
/// 2. 节点启动后自动同步（`start_node` 成功后），确保清链/重装后
///    本地已保存的钱包地址能重新绑定到链上。
pub(crate) async fn sync_saved_reward_wallet_inner(app: &AppHandle) -> Result<(), String> {
    let app_clone = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let Some(saved_address) = load_reward_wallet(&app_clone)? else {
            return Ok(None);
        };
        ensure_expected_reward_wallet_rpc_node()?;
        let normalized = normalize_wallet_address(&saved_address)?;
        let target_wallet = account_id_from_address(&normalized)?;

        // 从 keystore 文件名读取矿工公钥（不读取私钥）
        let miner_hex =
            local_powr_miner_account_hex(&app_clone)?.ok_or("未找到矿工公钥，请先启动节点")?;
        let miner_bytes = decode_hex_32_with_optional_0x(&miner_hex)?;

        if target_wallet == miner_bytes {
            return Err("奖励钱包不能与矿工账户相同，请使用独立收款钱包".to_string());
        }

        // 查询链上当前绑定状态
        let storage_key = reward_wallet_storage_key(&miner_bytes);
        let hex_key = format!("0x{}", hex::encode(&storage_key));
        let raw = rpc::rpc_post(
            "state_getStorage",
            serde_json::json!([hex_key]),
            rpc::RPC_REQUEST_TIMEOUT,
            RPC_RESPONSE_LIMIT_LARGE,
        )?;
        let current_wallet = if let Some(hex_val) = raw.as_str() {
            let hex_val = hex_val.trim_start_matches("0x");
            if hex_val.is_empty() {
                None
            } else {
                let bytes =
                    hex::decode(hex_val).map_err(|e| format!("解码链上绑定数据失败: {e}"))?;
                Some(decode_storage_account_id(&bytes)?)
            }
        } else {
            None
        };

        // 已是目标地址，无需操作
        if current_wallet == Some(target_wallet) {
            return Ok(None);
        }

        // 通过 node 端自定义 RPC 提交绑定/重绑交易
        let rpc_method = if current_wallet.is_some() {
            "reward_rebindWallet"
        } else {
            "reward_bindWallet"
        };
        let bind_timeout = std::time::Duration::from_secs(REWARD_BIND_TIMEOUT_SECS);
        rpc::rpc_post(
            rpc_method,
            serde_json::json!([normalized]),
            bind_timeout,
            RPC_RESPONSE_LIMIT_LARGE,
        )?;

        Ok(Some(()))
    })
    .await
    .map_err(|e| format!("sync task failed: {e}"))??;

    let _ = result;
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

    // 地址格式校验
    let target_wallet = account_id_from_address(&normalized)?;

    // 同步路径提前拒绝：奖励钱包不能与矿工账户相同。
    // 避免先存后验导致本地保存了一个链上必然被拒绝的无效地址。
    if let Some(miner_hex) = local_powr_miner_account_hex(&app)? {
        let miner_bytes = decode_hex_32_with_optional_0x(&miner_hex)?;
        if target_wallet == miner_bytes {
            return Err("奖励钱包不能与矿工账户相同，请使用独立收款钱包".to_string());
        }
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
        if let Err(e) =
            security::append_audit_log(&app2, "set_reward_wallet", &format!("chain_bind_{status}"))
        {
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
