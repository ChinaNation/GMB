// 签名管理员（验证者）管理。
//
// 原冷钱包导入/列表/删除功能已迁移到治理模块的管理员激活机制
// （governance/activation.rs），本模块仅保留签名管理员设置。
// 签名管理员用于省储行验证者出块签名，需要存储私钥到加密 keystore。

use crate::ui::{
    settings::{
        address_utils::decode_hex_32_with_optional_0x,
        device_password,
    },
    shared::security,
};
use crate::offchain_keystore::OffchainKeystore;
use primitives::china::china_ch::CHINA_CH;
use serde::{Deserialize, Serialize};
use sp_core::Pair;
use std::{fs, io::ErrorKind};
use tauri::AppHandle;

/// 签名管理员元数据文件（不含私钥，仅公钥和身份信息）。
const SIGNING_ADMIN_META_FILE: &str = "offchain/signing_admin_meta.json";

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

/// 设置签名管理员（验证者）。
///
/// 省储行管理员激活后，可通过此命令将私钥写入加密 keystore，
/// 用于出块签名。需要提供私钥种子（hex）和设备密码。
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

    // 2. 验证公钥格式
    let pubkey_lower = pubkey_hex.trim().to_ascii_lowercase();
    if pubkey_lower.len() != 64 || !pubkey_lower.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
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
