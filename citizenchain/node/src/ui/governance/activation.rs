// 管理员激活模块：在机构详情页内直接激活管理员身份。
//
// 激活流程：
// 1. 用户点击管理员行的"激活"按钮
// 2. 后端生成 activate_admin 签名请求 QR JSON
// 3. 用户用 wumin 冷钱包扫码 → 确认签名
// 4. 后端验证签名 → 写入本地加密存储
// 5. 管理员状态变为已激活，提案按钮解锁
//
// 激活 payload 格式（非链上交易）：
//   "GMB_ACTIVATE"(12B) + shenfen_id(48B) + timestamp(8B) + nonce(16B) = 84 bytes

use crate::ui::home;
use crate::ui::settings::device_password;
use crate::ui::shared::security;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

use super::institution;
use super::signing::{self, pubkey_to_ss58};

/// 激活管理员存储文件名。
const ACTIVATED_ADMINS_FILE: &str = "activated-admins.json";

/// "GMB_ACTIVATE" 前缀（12 字节）。
const ACTIVATE_PREFIX: &[u8; 12] = b"GMB_ACTIVATE";

// ──── 数据结构 ────

/// 已激活的管理员信息（返回给前端）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivatedAdmin {
    /// 管理员公钥 hex（不含 0x，小写）。
    pub pubkey_hex: String,
    /// 所属机构身份码。
    pub shenfen_id: String,
    /// 激活时间（毫秒时间戳）。
    pub activated_at_ms: u64,
}

/// 本地存储的激活凭证。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredActivation {
    /// 管理员公钥 hex（不含 0x，小写）。
    pubkey_hex: String,
    /// 所属机构身份码。
    shenfen_id: String,
    /// 激活时间（毫秒时间戳）。
    activated_at_ms: u64,
    /// 激活时的签名（用于凭证校验）。
    signature_hex: String,
    /// 激活时签名的 payload hash。
    payload_hash_hex: String,
}

/// 存储文件根结构。
#[derive(Debug, Serialize, Deserialize)]
struct StoredActivations {
    #[serde(default)]
    activations: Vec<StoredActivation>,
}

/// 构建激活请求的返回结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateRequestResult {
    /// 完整的 QR 签名请求 JSON 字符串。
    pub request_json: String,
    /// 请求 ID（用于后续验证匹配）。
    pub request_id: String,
    /// 签名 payload 的 SHA-256 哈希（用于验证响应）。
    pub expected_payload_hash: String,
    /// 激活 payload hex（用于本地验证）。
    pub payload_hex: String,
}

// ──── 存储操作 ────

fn activations_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(security::app_data_dir(app)?.join(ACTIVATED_ADMINS_FILE))
}

fn load_activations(app: &AppHandle) -> Result<Vec<StoredActivation>, String> {
    let path = activations_path(app)?;
    let raw = match fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("读取激活记录文件失败: {e}")),
    };
    let stored: StoredActivations =
        serde_json::from_str(&raw).map_err(|e| format!("解析激活记录文件失败: {e}"))?;
    Ok(stored.activations)
}

fn save_activations(app: &AppHandle, activations: &[StoredActivation]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredActivations {
        activations: activations.to_vec(),
    })
    .map_err(|e| format!("序列化激活记录失败: {e}"))?;
    let path = activations_path(app)?;
    security::write_text_atomic_restricted(&path, &format!("{raw}\n")).map_err(|e| {
        format!(
            "写入激活记录文件失败 ({}): {e}",
            security::sanitize_path(&path)
        )
    })
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 构建 84 字节激活 payload：
///   "GMB_ACTIVATE"(12B) + shenfen_id(48B, 右补零) + timestamp(8B, u64 LE) + nonce(16B)
fn build_activate_payload(shenfen_id: &str, timestamp: u64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(84);
    // 前缀
    payload.extend_from_slice(ACTIVATE_PREFIX);
    // shenfen_id 固定 48 字节，右补零
    let id_bytes = shenfen_id.as_bytes();
    let mut id_buf = [0u8; 48];
    let copy_len = id_bytes.len().min(48);
    id_buf[..copy_len].copy_from_slice(&id_bytes[..copy_len]);
    payload.extend_from_slice(&id_buf);
    // 时间戳 u64 LE
    payload.extend_from_slice(&timestamp.to_le_bytes());
    // 随机 nonce 16 字节
    let nonce: [u8; 16] = rand::random();
    payload.extend_from_slice(&nonce);
    payload
}

// ──── Tauri 命令 ────

/// 构建管理员激活签名请求 QR JSON（需要节点运行）。
///
/// 验证公钥确实在该机构的链上管理员列表中，
/// 然后生成 WUMIN_QR_V1 格式的签名请求。
#[tauri::command]
pub async fn build_activate_admin_request(
    app: AppHandle,
    pubkey_hex: String,
    shenfen_id: String,
) -> Result<ActivateRequestResult, String> {
    // 检查节点状态
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法验证管理员身份".to_string());
    }

    // 标准化公钥
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(&pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;

    // 检查是否已激活
    let activations = load_activations(&app)?;
    if activations
        .iter()
        .any(|a| a.pubkey_hex == pubkey_clean && a.shenfen_id == shenfen_id)
    {
        return Err("该管理员已激活，无需重复操作".to_string());
    }

    let sid = shenfen_id.clone();
    let pk = pubkey_clean.clone();

    // 在链上验证管理员身份
    let admins = tauri::async_runtime::spawn_blocking(move || institution::fetch_admins(&sid))
        .await
        .map_err(|e| format!("查询管理员列表失败: {e}"))??;

    if !admins.iter().any(|a| *a == pk) {
        return Err("该公钥不在此机构的链上管理员列表中".to_string());
    }

    // 查找机构名称（用于 display）
    let institution_name =
        super::find_institution_name(&shenfen_id).unwrap_or_else(|| shenfen_id.clone());

    // 构建激活 payload
    let timestamp = now_secs();
    let payload = build_activate_payload(&shenfen_id, timestamp);
    let payload_hex = format!("0x{}", hex::encode(&payload));

    // 计算 payload hash
    let payload_hash = signing::sha256_hash_public(&payload);
    let payload_hash_hex = format!("0x{}", hex::encode(&payload_hash));

    // 生成请求 ID
    let request_id = signing::generate_request_id_public("activate");

    // SS58 编码账户地址
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    // display 字段:wumin 端据此展示激活确认界面。
    // Registry activate_admin fields 严格只含 `shenfen_id`(对齐 wumin
    // decoder `_decodeActivateAdmin` 输出),机构名属辅助信息,通过 summary
    // 展示即可(2026-04-22 两色识别整改)。
    let display = serde_json::json!({
        "action": "activate_admin",
        "summary": format!("激活管理员 - {institution_name}"),
        "fields": [
            { "key": "shenfen_id", "label": "身份码", "value": shenfen_id }
        ]
    });

    let now = now_secs();
    let request = signing::QrSignRequest {
        proto: "WUMIN_QR_V1".to_string(),
        kind: "sign_request".to_string(),
        id: request_id.clone(),
        issued_at: now,
        expires_at: now + 90,
        body: signing::SignRequestBody {
            address: account_ss58,
            pubkey: format!("0x{pubkey_clean}"),
            sig_alg: "sr25519".to_string(),
            payload_hex: payload_hex.clone(),
            spec_version: 0,
            display,
        },
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(ActivateRequestResult {
        request_json,
        request_id,
        expected_payload_hash: payload_hash_hex,
        payload_hex,
    })
}

/// 验证管理员激活签名并写入本地加密存储。
///
/// 本地验证 sr25519 签名，不提交链上交易。
#[tauri::command]
pub async fn verify_activate_admin(
    app: AppHandle,
    request_id: String,
    pubkey_hex: String,
    expected_payload_hash: String,
    payload_hex: String,
    response_json: String,
) -> Result<ActivatedAdmin, String> {
    if let Err(e) = security::append_audit_log(&app, "activate_admin", "attempt") {
        eprintln!("[审计] activate_admin attempt 日志写入失败: {e}");
    }

    // 标准化公钥
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(&pubkey_hex)
        .to_ascii_lowercase();

    // 解析签名响应
    let response: signing::QrSignResponse =
        serde_json::from_str(&response_json).map_err(|e| format!("解析签名响应失败: {e}"))?;

    // 验证协议版本
    if response.proto != "WUMIN_QR_V1" {
        return Err(format!(
            "协议版本不匹配：期望 WUMIN_QR_V1，实际 {}",
            response.proto
        ));
    }

    // 验证请求 ID
    if response.id != request_id {
        return Err("请求 ID 不匹配".to_string());
    }

    // 验证公钥
    let response_pubkey = response
        .body
        .pubkey
        .strip_prefix("0x")
        .unwrap_or(&response.body.pubkey)
        .to_ascii_lowercase();
    if response_pubkey != pubkey_clean {
        return Err("公钥不匹配".to_string());
    }

    // 验证 payload hash
    let expected_hash = expected_payload_hash
        .strip_prefix("0x")
        .unwrap_or(&expected_payload_hash)
        .to_ascii_lowercase();
    let response_hash = response
        .body
        .payload_hash
        .strip_prefix("0x")
        .unwrap_or(&response.body.payload_hash)
        .to_ascii_lowercase();
    if response_hash != expected_hash {
        return Err("payload hash 不匹配,签名数据可能被篡改".to_string());
    }

    // 验证 sr25519 签名(本地验证,不需要提交链上)
    let sig_hex = response
        .body
        .signature
        .strip_prefix("0x")
        .unwrap_or(&response.body.signature);
    if sig_hex.len() != 128 {
        return Err(format!(
            "签名长度无效：期望 128 hex，实际 {}",
            sig_hex.len()
        ));
    }
    let signature_bytes = hex::decode(sig_hex).map_err(|e| format!("签名解码失败: {e}"))?;
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;

    // 解码 payload
    let payload_clean = payload_hex.strip_prefix("0x").unwrap_or(&payload_hex);
    let payload_bytes = hex::decode(payload_clean).map_err(|e| format!("payload 解码失败: {e}"))?;

    // 验证 payload 前缀是 "GMB_ACTIVATE"
    if payload_bytes.len() < 12 || &payload_bytes[..12] != ACTIVATE_PREFIX {
        return Err("激活 payload 前缀无效".to_string());
    }

    // 使用 sr25519 验证签名
    // sr25519 签名时对 payload 做内部哈希处理，直接传原始 payload
    use sp_core::crypto::Pair;
    use sp_core::sr25519::{Public, Signature};
    let public = Public::from_raw(
        <[u8; 32]>::try_from(pubkey_bytes.as_slice()).map_err(|_| "公钥长度必须为 32 字节")?,
    );
    let signature = Signature::from_raw(
        <[u8; 64]>::try_from(signature_bytes.as_slice()).map_err(|_| "签名长度必须为 64 字节")?,
    );
    if !sp_core::sr25519::Pair::verify(&signature, &payload_bytes, &public) {
        return Err("sr25519 签名验证失败，无法证明管理员身份".to_string());
    }

    // 从 payload 中提取 shenfen_id（偏移 12，长度 48，去尾零）
    let id_bytes = &payload_bytes[12..60];
    let end = id_bytes
        .iter()
        .rposition(|&b| b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    let shenfen_id = String::from_utf8_lossy(&id_bytes[..end]).to_string();

    // 写入本地存储
    let mut activations = load_activations(&app)?;
    // 去重：同一 (pubkey, shenfen_id) 只保留最新
    activations.retain(|a| !(a.pubkey_hex == pubkey_clean && a.shenfen_id == shenfen_id));
    let activated_at = now_ms();
    activations.push(StoredActivation {
        pubkey_hex: pubkey_clean.clone(),
        shenfen_id: shenfen_id.clone(),
        activated_at_ms: activated_at,
        signature_hex: sig_hex.to_string(),
        payload_hash_hex: response_hash,
    });
    save_activations(&app, &activations)?;

    if let Err(e) = security::append_audit_log(
        &app,
        "activate_admin",
        &format!(
            "success pubkey={} shenfen={}",
            &pubkey_clean[..8],
            &shenfen_id
        ),
    ) {
        eprintln!("[审计] activate_admin success 日志写入失败: {e}");
    }

    Ok(ActivatedAdmin {
        pubkey_hex: pubkey_clean,
        shenfen_id,
        activated_at_ms: activated_at,
    })
}

/// 获取指定机构的已激活管理员列表。
///
/// 每次调用时与链上当前管理员列表交叉校验：
/// - 链上已移除的管理员 → 自动删除本地激活记录
/// - 返回仍有效的已激活管理员
#[tauri::command]
pub async fn get_activated_admins(
    app: AppHandle,
    shenfen_id: String,
) -> Result<Vec<ActivatedAdmin>, String> {
    let mut activations = load_activations(&app)?;

    // 筛选该机构的激活记录
    let institution_activations: Vec<&StoredActivation> = activations
        .iter()
        .filter(|a| a.shenfen_id == shenfen_id)
        .collect();

    if institution_activations.is_empty() {
        return Ok(Vec::new());
    }

    // 如果节点运行中，与链上管理员列表交叉校验
    let status = home::current_status(&app)?;
    if status.running {
        let sid = shenfen_id.clone();
        let admins = tauri::async_runtime::spawn_blocking(move || institution::fetch_admins(&sid))
            .await
            .map_err(|e| format!("查询管理员列表失败: {e}"));

        if let Ok(Ok(chain_admins)) = admins {
            // 删除不再是链上管理员的激活记录
            let before_len = activations.len();
            activations.retain(|a| {
                if a.shenfen_id != shenfen_id {
                    return true; // 不是当前机构的，保留
                }
                chain_admins.iter().any(|admin| *admin == a.pubkey_hex)
            });
            if activations.len() != before_len {
                // 有失效记录被清除，保存更新
                let _ = save_activations(&app, &activations);
            }
        }
        // 如果 RPC 查询失败，不清除本地记录（容错）
    }

    // 返回该机构的有效激活记录
    Ok(activations
        .iter()
        .filter(|a| a.shenfen_id == shenfen_id)
        .map(|a| ActivatedAdmin {
            pubkey_hex: a.pubkey_hex.clone(),
            shenfen_id: a.shenfen_id.clone(),
            activated_at_ms: a.activated_at_ms,
        })
        .collect())
}

/// 取消管理员激活（需要设备密码）。
#[tauri::command]
pub fn deactivate_admin(
    app: AppHandle,
    pubkey_hex: String,
    shenfen_id: String,
    unlock_password: String,
) -> Result<(), String> {
    if let Err(e) = security::append_audit_log(&app, "deactivate_admin", "attempt") {
        eprintln!("[审计] deactivate_admin attempt 日志写入失败: {e}");
    }

    // 验证设备密码
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;

    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(&pubkey_hex)
        .to_ascii_lowercase();

    let mut activations = load_activations(&app)?;
    let before_len = activations.len();
    activations.retain(|a| !(a.pubkey_hex == pubkey_clean && a.shenfen_id == shenfen_id));

    if activations.len() == before_len {
        return Err("未找到该管理员的激活记录".to_string());
    }

    save_activations(&app, &activations)?;

    if let Err(e) = security::append_audit_log(
        &app,
        "deactivate_admin",
        &format!(
            "success pubkey={} shenfen={}",
            &pubkey_clean[..pubkey_clean.len().min(8)],
            &shenfen_id
        ),
    ) {
        eprintln!("[审计] deactivate_admin success 日志写入失败: {e}");
    }

    Ok(())
}

/// 检查本地是否有任何已激活的管理员（纯本地文件读取，不需要节点运行）。
///
/// 用于设置页面判断是否显示引导节点和投票节点设置项。
#[tauri::command]
pub fn has_any_activated_admin(app: AppHandle) -> Result<bool, String> {
    let activations = load_activations(&app)?;
    Ok(!activations.is_empty())
}
