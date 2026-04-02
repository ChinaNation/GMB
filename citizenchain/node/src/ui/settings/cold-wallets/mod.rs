// 冷钱包管理：导入、列表、删除。
// 冷钱包只存公钥和地址，不存私钥。签名通过外部设备二维码扫码完成。

use crate::ui::{
    settings::{
        address_utils::{decode_hex_32_with_optional_0x, decode_ss58_prefix},
        device_password,
    },
    shared::{
        constants::SS58_PREFIX,
        security,
        validation::normalize_wallet_address,
    },
};
use crate::offchain_keystore::OffchainKeystore;
use primitives::china::china_ch::CHINA_CH;
use serde::{Deserialize, Serialize};
use sp_core::Pair;
use std::{fs, io::ErrorKind, time::{SystemTime, UNIX_EPOCH}};
use tauri::AppHandle;

const COLD_WALLETS_FILE: &str = "cold-wallets.json";
/// 签名管理员元数据文件（不含私钥，仅公钥和身份信息）。
const SIGNING_ADMIN_META_FILE: &str = "offchain/signing_admin_meta.json";
/// 单个节点最多导入 50 个冷钱包。
const MAX_COLD_WALLETS: usize = 50;

/// 前端展示的冷钱包信息。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdWallet {
    /// SS58 地址或 0x hex。
    pub address: String,
    /// 32 字节公钥 hex（不含 0x，小写）。
    pub pubkey_hex: String,
    /// 用户自定义名称。
    pub name: String,
    /// 导入时间（毫秒时间戳）。
    pub created_at_ms: u64,
}

/// 冷钱包列表结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdWalletList {
    pub wallets: Vec<ColdWallet>,
}

/// 签名管理员信息（返回给前端）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigningAdminInfo {
    /// 管理员公钥 hex（不含 0x，小写）。
    pub pubkey_hex: String,
    /// 省储行身份 ID。
    pub shenfen_id: String,
    /// 省储行名称。
    pub shenfen_name: String,
}

// 存储结构
#[derive(Debug, Serialize, Deserialize)]
struct StoredColdWallets {
    #[serde(default)]
    wallets: Vec<StoredColdWallet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredColdWallet {
    address: String,
    pubkey_hex: String,
    name: String,
    created_at_ms: u64,
}

fn cold_wallets_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(security::app_data_dir(app)?.join(COLD_WALLETS_FILE))
}

fn load_wallets(app: &AppHandle) -> Result<Vec<StoredColdWallet>, String> {
    let path = cold_wallets_path(app)?;
    let raw = match fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("读取冷钱包文件失败: {e}")),
    };
    let stored: StoredColdWallets =
        serde_json::from_str(&raw).map_err(|e| format!("解析冷钱包文件失败: {e}"))?;
    Ok(stored.wallets)
}

fn save_wallets(app: &AppHandle, wallets: &[StoredColdWallet]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredColdWallets {
        wallets: wallets.to_vec(),
    })
    .map_err(|e| format!("序列化冷钱包失败: {e}"))?;
    let path = cold_wallets_path(app)?;
    security::write_text_atomic_restricted(&path, &format!("{raw}\n"))
        .map_err(|e| format!("写入冷钱包文件失败 ({}): {e}", security::sanitize_path(&path)))
}

/// 从地址提取 32 字节公钥。
/// 支持 SS58 (prefix 2027) 和 0x hex。
fn extract_pubkey(address: &str) -> Result<[u8; 32], String> {
    if address.starts_with("0x") {
        return decode_hex_32_with_optional_0x(address);
    }
    // SS58 解码
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
    // 校验和
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

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ──── Tauri 命令 ────

/// 获取冷钱包列表。
#[tauri::command]
pub fn get_cold_wallets(app: AppHandle) -> Result<ColdWalletList, String> {
    let wallets = load_wallets(&app)?;
    Ok(ColdWalletList {
        wallets: wallets
            .into_iter()
            .map(|w| ColdWallet {
                address: ensure_ss58(&w.address, &w.pubkey_hex),
                pubkey_hex: w.pubkey_hex,
                name: w.name,
                created_at_ms: w.created_at_ms,
            })
            .collect(),
    })
}

/// 确保地址为 SS58 格式。如果已有地址是 hex 格式，从 pubkey_hex 转换。
fn ensure_ss58(address: &str, pubkey_hex: &str) -> String {
    if !address.starts_with("0x") && !address.starts_with("0X") {
        return address.to_string();
    }
    if let Ok(bytes) = hex::decode(pubkey_hex) {
        if let Ok(ss58) = crate::ui::governance::signing::pubkey_to_ss58(&bytes) {
            return ss58;
        }
    }
    address.to_string()
}

/// 导入冷钱包。
/// address 支持 SS58 (prefix 2027) 或 0x + 64 hex。
/// name 为用户自定义名称。
#[tauri::command]
pub fn add_cold_wallet(
    app: AppHandle,
    address: String,
    name: String,
    unlock_password: String,
) -> Result<ColdWalletList, String> {
    if let Err(e) = security::append_audit_log(&app, "add_cold_wallet", "attempt") {
        eprintln!("[审计] add_cold_wallet attempt 日志写入失败: {e}");
    }

    // 验证设备密码
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;

    // 验证地址格式
    let normalized = normalize_wallet_address(&address)?;
    let pubkey_bytes = extract_pubkey(&normalized)?;
    let pubkey_hex = hex::encode(pubkey_bytes);

    // 验证名称
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("钱包名称不能为空".to_string());
    }
    if name.len() > 50 {
        return Err("钱包名称不能超过 50 字符".to_string());
    }

    // 加载现有列表，检查去重和上限
    let mut wallets = load_wallets(&app)?;
    if wallets.iter().any(|w| w.pubkey_hex == pubkey_hex) {
        return Err("该公钥已导入，不能重复添加".to_string());
    }
    if wallets.len() >= MAX_COLD_WALLETS {
        return Err(format!("冷钱包数量已达上限 {MAX_COLD_WALLETS} 个").to_string());
    }

    // 添加：地址统一存为 SS58 格式
    let ss58_address = crate::ui::governance::signing::pubkey_to_ss58(&pubkey_bytes)
        .unwrap_or(normalized);
    wallets.push(StoredColdWallet {
        address: ss58_address,
        pubkey_hex: pubkey_hex.clone(),
        name,
        created_at_ms: now_ms(),
    });

    save_wallets(&app, &wallets)?;

    if let Err(e) = security::append_audit_log(
        &app,
        "add_cold_wallet",
        &format!("success pubkey={}", &pubkey_hex[..8]),
    ) {
        eprintln!("[审计] add_cold_wallet success 日志写入失败: {e}");
    }

    Ok(ColdWalletList {
        wallets: wallets
            .into_iter()
            .map(|w| ColdWallet {
                address: w.address,
                pubkey_hex: w.pubkey_hex,
                name: w.name,
                created_at_ms: w.created_at_ms,
            })
            .collect(),
    })
}

/// 删除冷钱包。
#[tauri::command]
pub fn remove_cold_wallet(
    app: AppHandle,
    pubkey_hex: String,
    unlock_password: String,
) -> Result<ColdWalletList, String> {
    if let Err(e) = security::append_audit_log(&app, "remove_cold_wallet", "attempt") {
        eprintln!("[审计] remove_cold_wallet attempt 日志写入失败: {e}");
    }

    // 验证设备密码
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;

    let pubkey = pubkey_hex.trim().to_ascii_lowercase();
    let mut wallets = load_wallets(&app)?;
    let before_len = wallets.len();
    wallets.retain(|w| w.pubkey_hex != pubkey);

    if wallets.len() == before_len {
        return Err("未找到该公钥对应的冷钱包".to_string());
    }

    save_wallets(&app, &wallets)?;

    if let Err(e) = security::append_audit_log(
        &app,
        "remove_cold_wallet",
        &format!("success pubkey={}", &pubkey[..pubkey.len().min(8)]),
    ) {
        eprintln!("[审计] remove_cold_wallet success 日志写入失败: {e}");
    }

    Ok(ColdWalletList {
        wallets: wallets
            .into_iter()
            .map(|w| ColdWallet {
                address: w.address,
                pubkey_hex: w.pubkey_hex,
                name: w.name,
                created_at_ms: w.created_at_ms,
            })
            .collect(),
    })
}

// ──── 签名管理员元数据持久化 ────

/// 签名管理员元数据文件路径。
fn signing_admin_meta_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(security::app_data_dir(app)?.join(SIGNING_ADMIN_META_FILE))
}

/// 保存签名管理员元数据（不含私钥）。
fn save_signing_admin_meta(app: &AppHandle, info: &SigningAdminInfo) -> Result<(), String> {
    let path = signing_admin_meta_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建 offchain 目录失败: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(info)
        .map_err(|e| format!("序列化签名管理员元数据失败: {e}"))?;
    security::write_text_atomic_restricted(&path, &format!("{raw}\n"))
        .map_err(|e| format!("写入签名管理员元数据失败: {e}"))
}

/// 读取签名管理员元数据（不需要密码）。
fn load_signing_admin_meta(app: &AppHandle) -> Result<Option<SigningAdminInfo>, String> {
    let path = signing_admin_meta_path(app)?;
    let raw = match fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("读取签名管理员元数据失败: {e}")),
    };
    let info: SigningAdminInfo =
        serde_json::from_str(&raw).map_err(|e| format!("解析签名管理员元数据失败: {e}"))?;
    Ok(Some(info))
}

/// 根据公钥查找所属省储行。
/// 遍历 CHINA_CH 所有省储行的 duoqian_admins，匹配则返回 (shenfen_id, shenfen_name)。
fn find_province_for_admin(pubkey: &[u8; 32]) -> Option<(&'static str, &'static str)> {
    for ch in CHINA_CH {
        if ch.duoqian_admins.iter().any(|admin| admin == pubkey) {
            return Some((ch.shenfen_id, ch.shenfen_name));
        }
    }
    None
}

// ──── 签名管理员 Tauri 命令 ────

/// 设置冷钱包为离线清算签名管理员。
/// 需要提供私钥种子（hex）和设备密码。
#[tauri::command]
pub fn set_signing_admin(
    app: AppHandle,
    pubkey_hex: String,
    private_key_hex: String,
    unlock_password: String,
) -> Result<SigningAdminInfo, String> {
    if let Err(e) = security::append_audit_log(&app, "set_signing_admin", "attempt") {
        eprintln!("[审计] set_signing_admin attempt 日志写入失败: {e}");
    }

    // 1. 验证设备密码
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;

    // 2. 验证公钥存在于冷钱包列表
    let pubkey_lower = pubkey_hex.trim().to_ascii_lowercase();
    let wallets = load_wallets(&app)?;
    if !wallets.iter().any(|w| w.pubkey_hex == pubkey_lower) {
        return Err("该公钥不在冷钱包列表中，请先导入".to_string());
    }

    // 3. 解码私钥种子（64 hex 字符 = 32 字节）
    let private_hex = private_key_hex.trim().trim_start_matches("0x");
    let seed_vec = hex::decode(private_hex)
        .map_err(|_| "私钥格式错误：必须为 64 位十六进制字符".to_string())?;
    if seed_vec.len() != 32 {
        return Err("私钥长度错误：必须为 32 字节（64 位十六进制）".to_string());
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&seed_vec);

    // 4. 从种子创建 sr25519 密钥对
    let pair = sp_core::sr25519::Pair::from_seed(&seed);

    // 5. 验证派生公钥与提供的公钥一致
    let derived_pubkey = pair.public().0;
    let derived_hex = hex::encode(derived_pubkey);
    if derived_hex != pubkey_lower {
        return Err("私钥与公钥不匹配：请确认提供的是正确的私钥种子".to_string());
    }

    // 6. 查找该管理员所属省储行
    let pubkey_bytes = decode_hex_32_with_optional_0x(&pubkey_lower)?;
    let (shenfen_id, shenfen_name) = find_province_for_admin(&pubkey_bytes)
        .ok_or_else(|| "该公钥不是任何省储行的多签管理员".to_string())?;

    // 7. 使用 OffchainKeystore 加密保存私钥
    let base_path = security::app_data_dir(&app)?;
    let keystore = OffchainKeystore::new(&base_path);
    keystore.save_signing_key(&unlock_password, &seed, shenfen_id)?;

    // 8. 保存元数据（不含私钥）
    let info = SigningAdminInfo {
        pubkey_hex: pubkey_lower.clone(),
        shenfen_id: shenfen_id.to_string(),
        shenfen_name: shenfen_name.to_string(),
    };
    save_signing_admin_meta(&app, &info)?;

    // 9. 审计日志
    if let Err(e) = security::append_audit_log(
        &app,
        "set_signing_admin",
        &format!("success pubkey={} shenfen={}", &pubkey_lower[..8], shenfen_id),
    ) {
        eprintln!("[审计] set_signing_admin success 日志写入失败: {e}");
    }

    Ok(info)
}

/// 获取当前签名管理员信息（不需要密码，只读元数据）。
#[tauri::command]
pub fn get_signing_admin(app: AppHandle) -> Result<Option<SigningAdminInfo>, String> {
    // 检查加密密钥文件是否存在
    let base_path = security::app_data_dir(&app)?;
    let keystore = OffchainKeystore::new(&base_path);
    if !keystore.has_signing_key() {
        return Ok(None);
    }
    // 读取元数据（不需要密码）
    load_signing_admin_meta(&app)
}
