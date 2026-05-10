// 管理员激活模块：为 admins-change 主体生成 subject 级本地激活凭证。
//
// 激活流程：
// 1. 用户点击管理员行的"激活"按钮；
// 2. 后端读取链上 AdminsChange::Subjects，确认目标 pubkey 是当前 Active 管理员；
// 3. 后端生成 activate_admin_subject 签名请求 QR JSON；
// 4. 用户用 wumin 冷钱包扫码确认并签名；
// 5. 后端验证签名、payload、链上主体仍一致后，写入本地激活记录；
// 6. 管理员状态变为已激活，提案按钮解锁。
//
// 激活 payload 格式（非链上交易）：
//   "GMB_ACTIVATE_SUBJECT_V1"(23B)
//   + subject_id(48B) + org(1B) + kind(1B) + pubkey(32B)
//   + timestamp(8B) + nonce(16B) = 129 bytes

use crate::home;
use crate::settings::device_password;
use crate::shared::security;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

use crate::governance::admins_change::storage;
use crate::governance::signing::{self, pubkey_to_ss58};

use super::subject_id;
use super::types::{qr_org_display_value, AdminSubjectState};

/// subject 级激活管理员存储文件名。旧 `activated-admins.json` 不读取、不迁移。
const ACTIVATED_ADMINS_FILE: &str = "activated-admin-subjects.json";

/// "GMB_ACTIVATE_SUBJECT_V1" 前缀（23 字节）。
const ACTIVATE_SUBJECT_PREFIX: &[u8; 23] = b"GMB_ACTIVATE_SUBJECT_V1";
const ACTIVATE_SUBJECT_PAYLOAD_LEN: usize = 23 + 48 + 1 + 1 + 32 + 8 + 16;

// ──── 数据结构 ────

/// 已激活的管理员信息（返回给前端）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivatedAdmin {
    /// 管理员公钥 hex（不含 0x，小写）。
    pub pubkey_hex: String,
    /// admins-change 链上主体 SubjectId hex（不含 0x，小写）。
    pub subject_id_hex: String,
    /// 链上 org 编码。
    pub org: u8,
    /// 链上 AdminSubjectKind 编码。
    pub kind: u8,
    /// 激活时间（毫秒时间戳）。
    pub activated_at_ms: u64,
}

/// 本地存储的激活凭证。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredActivation {
    /// 管理员公钥 hex（不含 0x，小写）。
    pubkey_hex: String,
    /// admins-change 链上主体 SubjectId hex（不含 0x，小写）。
    subject_id_hex: String,
    /// 链上 org 编码。
    org: u8,
    /// 链上 AdminSubjectKind 编码。
    kind: u8,
    /// 激活时间（毫秒时间戳）。
    activated_at_ms: u64,
    /// 激活时的签名（用于凭证校验）。
    signature_hex: String,
    /// 激活时签名的 payload hash。
    payload_hash_hex: String,
}

/// 解码后的 subject 级激活 payload。
struct ActivationPayload {
    subject_id: [u8; 48],
    org: u8,
    kind: u8,
    pubkey_hex: String,
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

fn validate_activation_subject(
    state: &AdminSubjectState,
    expected_org: Option<u8>,
) -> Result<(), String> {
    if let Some(org) = expected_org {
        if state.org != org {
            return Err(format!(
                "管理员主体 org 不匹配：请求 org={}，链上 org={}",
                org, state.org
            ));
        }
    }
    if state.status != 1 {
        return Err("管理员主体不是已激活状态，不能激活本地管理员身份".to_string());
    }
    match state.kind {
        0 if matches!(state.org, 0 | 1 | 2) => Ok(()),
        1 => Err("SfidInstitution 只用于机构归属/检索，不能作为管理员激活主体".to_string()),
        2 if state.org == 3 => Ok(()),
        3 if matches!(state.org, 4 | 5) => Ok(()),
        _ => Err("管理员主体类型与 org 不匹配，不能激活".to_string()),
    }
}

fn fetch_chain_subject(
    sfid_number: &str,
    subject_id_hex: Option<String>,
    expected_org: Option<u8>,
) -> Result<AdminSubjectState, String> {
    let state = if let Some(subject_id_hex) = subject_id_hex.filter(|item| !item.trim().is_empty())
    {
        let subject_id = subject_id::subject_id_from_hex(&subject_id_hex)?;
        storage::fetch_admin_subject(&subject_id, Some(sfid_number.to_string()))?
            .ok_or_else(|| "链上不存在该管理员主体".to_string())?
    } else {
        if matches!(expected_org, Some(3 | 4 | 5)) {
            return Err("个人多签或机构账户管理员激活必须提供 subjectIdHex".to_string());
        }
        storage::fetch_admin_subject_by_sfid_number(sfid_number)?
            .ok_or_else(|| "链上不存在该管理员主体".to_string())?
    };
    validate_activation_subject(&state, expected_org)?;
    Ok(state)
}

fn resolve_activation_subject_id_hex(
    sfid_number: &str,
    subject_id_hex: Option<String>,
    expected_org: Option<u8>,
) -> Result<String, String> {
    if let Some(subject_id_hex) = subject_id_hex.filter(|item| !item.trim().is_empty()) {
        let subject_id = subject_id::subject_id_from_hex(&subject_id_hex)?;
        return Ok(hex::encode(subject_id));
    }
    if matches!(expected_org, Some(3 | 4 | 5)) {
        return Err("个人多签或机构账户管理员激活必须提供 subjectIdHex".to_string());
    }
    let subject_id = subject_id::subject_id_from_builtin_sfid(sfid_number)?;
    Ok(hex::encode(subject_id))
}

/// 构建 129 字节 subject 级激活 payload。
fn build_activate_payload(
    subject_id: &[u8; 48],
    org: u8,
    kind: u8,
    pubkey: &[u8; 32],
    timestamp: u64,
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(ACTIVATE_SUBJECT_PAYLOAD_LEN);
    payload.extend_from_slice(ACTIVATE_SUBJECT_PREFIX);
    payload.extend_from_slice(subject_id);
    payload.push(org);
    payload.push(kind);
    payload.extend_from_slice(pubkey);
    payload.extend_from_slice(&timestamp.to_le_bytes());
    let nonce: [u8; 16] = rand::random();
    payload.extend_from_slice(&nonce);
    payload
}

fn decode_activate_payload(payload_bytes: &[u8]) -> Result<ActivationPayload, String> {
    if payload_bytes.len() != ACTIVATE_SUBJECT_PAYLOAD_LEN {
        return Err(format!(
            "激活 payload 长度无效：期望 {} 字节，实际 {} 字节",
            ACTIVATE_SUBJECT_PAYLOAD_LEN,
            payload_bytes.len()
        ));
    }
    if &payload_bytes[..ACTIVATE_SUBJECT_PREFIX.len()] != ACTIVATE_SUBJECT_PREFIX {
        return Err("激活 payload 前缀无效".to_string());
    }
    let mut offset = ACTIVATE_SUBJECT_PREFIX.len();
    let mut subject_id = [0u8; 48];
    subject_id.copy_from_slice(&payload_bytes[offset..offset + 48]);
    offset += 48;
    let org = payload_bytes[offset];
    offset += 1;
    let kind = payload_bytes[offset];
    offset += 1;
    let pubkey_hex = hex::encode(&payload_bytes[offset..offset + 32]);
    Ok(ActivationPayload {
        subject_id,
        org,
        kind,
        pubkey_hex,
    })
}

fn activated_admin_from_stored(item: &StoredActivation) -> ActivatedAdmin {
    ActivatedAdmin {
        pubkey_hex: item.pubkey_hex.clone(),
        subject_id_hex: item.subject_id_hex.clone(),
        org: item.org,
        kind: item.kind,
        activated_at_ms: item.activated_at_ms,
    }
}

// ──── Tauri 命令 ────

/// 构建管理员激活签名请求 QR JSON（需要节点运行）。
///
/// 验证公钥确实在该 admins-change 主体的链上管理员列表中，
/// 然后生成 WUMIN_QR_V1 格式的 subject 级签名请求。
#[tauri::command]
pub async fn build_activate_admin_request(
    app: AppHandle,
    pubkey_hex: String,
    sfid_number: String,
    subject_id_hex: Option<String>,
    expected_org: Option<u8>,
) -> Result<ActivateRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法验证管理员身份".to_string());
    }

    let pubkey_clean = subject_id::normalize_pubkey_hex(&pubkey_hex)?;
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;
    let pubkey_array: [u8; 32] = pubkey_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "公钥长度必须为 32 字节".to_string())?;

    let sid = sfid_number.clone();
    let subject = subject_id_hex.clone();
    let state = tauri::async_runtime::spawn_blocking(move || {
        fetch_chain_subject(&sid, subject, expected_org)
    })
    .await
    .map_err(|e| format!("查询管理员主体失败: {e}"))??;

    if !state.admins.iter().any(|a| *a == pubkey_clean) {
        return Err("该公钥不在此管理员主体的链上管理员列表中".to_string());
    }

    let activations = load_activations(&app)?;
    if activations.iter().any(|a| {
        a.pubkey_hex == pubkey_clean
            && a.subject_id_hex == state.subject_id_hex
            && a.org == state.org
            && a.kind == state.kind
    }) {
        return Err("该管理员已激活，无需重复操作".to_string());
    }

    let subject_id = subject_id::subject_id_from_hex(&state.subject_id_hex)?;
    let timestamp = now_secs();
    let payload =
        build_activate_payload(&subject_id, state.org, state.kind, &pubkey_array, timestamp);
    let payload_hex = format!("0x{}", hex::encode(&payload));

    let payload_hash = signing::sha256_hash_public(&payload);
    let payload_hash_hex = format!("0x{}", hex::encode(payload_hash));
    let request_id = signing::generate_request_id_public("activate-subject");
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    let display = serde_json::json!({
        "action": "activate_admin_subject",
        "summary": format!("激活{}管理员", state.org_label),
        "fields": [
            { "key": "org", "label": "组织类型", "value": qr_org_display_value(state.org) },
            { "key": "subject", "label": "管理员主体", "value": format!("0x{}", state.subject_id_hex) },
            { "key": "pubkey", "label": "管理员公钥", "value": format!("0x{pubkey_clean}") }
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
/// 本地验证 sr25519 签名，不提交链上交易；写入前重新确认链上主体仍 Active。
#[tauri::command]
pub async fn verify_activate_admin(
    app: AppHandle,
    request_id: String,
    pubkey_hex: String,
    expected_payload_hash: String,
    payload_hex: String,
    response_json: String,
) -> Result<ActivatedAdmin, String> {
    if let Err(e) = security::append_audit_log(&app, "activate_admin_subject", "attempt") {
        eprintln!("[审计] activate_admin_subject attempt 日志写入失败: {e}");
    }

    let pubkey_clean = subject_id::normalize_pubkey_hex(&pubkey_hex)?;

    let response: signing::QrSignResponse =
        serde_json::from_str(&response_json).map_err(|e| format!("解析签名响应失败: {e}"))?;

    if response.proto != "WUMIN_QR_V1" {
        return Err(format!(
            "协议版本不匹配：期望 WUMIN_QR_V1，实际 {}",
            response.proto
        ));
    }
    if response.id != request_id {
        return Err("请求 ID 不匹配".to_string());
    }

    let response_pubkey = response
        .body
        .pubkey
        .strip_prefix("0x")
        .unwrap_or(&response.body.pubkey)
        .to_ascii_lowercase();
    if response_pubkey != pubkey_clean {
        return Err("公钥不匹配".to_string());
    }

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
        return Err("payload hash 不匹配，签名数据可能被篡改".to_string());
    }

    let payload_clean = payload_hex.strip_prefix("0x").unwrap_or(&payload_hex);
    let payload_bytes = hex::decode(payload_clean).map_err(|e| format!("payload 解码失败: {e}"))?;
    let decoded = decode_activate_payload(&payload_bytes)?;
    if decoded.pubkey_hex != pubkey_clean {
        return Err("激活 payload 中的管理员公钥与签名公钥不一致".to_string());
    }

    let subject_id_hex = hex::encode(decoded.subject_id);
    let state = tauri::async_runtime::spawn_blocking({
        let subject_id = decoded.subject_id;
        move || {
            storage::fetch_admin_subject(&subject_id, None)?
                .ok_or_else(|| "链上不存在该管理员主体".to_string())
        }
    })
    .await
    .map_err(|e| format!("查询管理员主体失败: {e}"))??;
    if state.org != decoded.org {
        return Err(format!(
            "激活 payload org 与链上主体不一致：payload={}，链上={}",
            decoded.org, state.org
        ));
    }
    if state.kind != decoded.kind {
        return Err(format!(
            "激活 payload kind 与链上主体不一致：payload={}，链上={}",
            decoded.kind, state.kind
        ));
    }
    validate_activation_subject(&state, Some(decoded.org))?;
    if !state.admins.iter().any(|a| *a == pubkey_clean) {
        return Err("该公钥不在此管理员主体的链上管理员列表中".to_string());
    }

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

    let mut activations = load_activations(&app)?;
    activations.retain(|a| !(a.pubkey_hex == pubkey_clean && a.subject_id_hex == subject_id_hex));
    let activated_at = now_ms();
    activations.push(StoredActivation {
        pubkey_hex: pubkey_clean.clone(),
        subject_id_hex: subject_id_hex.clone(),
        org: decoded.org,
        kind: decoded.kind,
        activated_at_ms: activated_at,
        signature_hex: sig_hex.to_string(),
        payload_hash_hex: response_hash,
    });
    save_activations(&app, &activations)?;

    if let Err(e) = security::append_audit_log(
        &app,
        "activate_admin_subject",
        &format!(
            "success pubkey={} subject={}",
            &pubkey_clean[..8],
            &subject_id_hex
        ),
    ) {
        eprintln!("[审计] activate_admin_subject success 日志写入失败: {e}");
    }

    Ok(ActivatedAdmin {
        pubkey_hex: pubkey_clean,
        subject_id_hex,
        org: decoded.org,
        kind: decoded.kind,
        activated_at_ms: activated_at,
    })
}

/// 获取指定管理员主体的已激活管理员列表。
///
/// 每次调用时与链上当前管理员列表交叉校验：
/// - 链上已移除的管理员 → 自动删除本地激活记录；
/// - org/kind 已变化或主体已关闭 → 不再返回本地激活记录；
/// - 返回仍有效的已激活管理员。
#[tauri::command]
pub async fn get_activated_admins(
    app: AppHandle,
    sfid_number: String,
    subject_id_hex: Option<String>,
    expected_org: Option<u8>,
) -> Result<Vec<ActivatedAdmin>, String> {
    let lookup_subject_id_hex =
        resolve_activation_subject_id_hex(&sfid_number, subject_id_hex.clone(), expected_org)?;
    let mut activations = load_activations(&app)?;

    let subject_activations: Vec<&StoredActivation> = activations
        .iter()
        .filter(|a| a.subject_id_hex == lookup_subject_id_hex)
        .collect();

    if subject_activations.is_empty() {
        return Ok(Vec::new());
    }

    let status = home::current_status(&app)?;
    if status.running {
        let sid = sfid_number.clone();
        let subject = subject_id_hex.clone();
        let state = tauri::async_runtime::spawn_blocking(move || {
            fetch_chain_subject(&sid, subject, expected_org)
        })
        .await
        .map_err(|e| format!("查询管理员主体失败: {e}"));

        match state {
            Ok(Ok(state)) => {
                let before_len = activations.len();
                activations.retain(|a| {
                    if a.subject_id_hex != lookup_subject_id_hex {
                        return true;
                    }
                    a.org == state.org
                        && a.kind == state.kind
                        && state.admins.iter().any(|admin| *admin == a.pubkey_hex)
                });
                if activations.len() != before_len {
                    let _ = save_activations(&app, &activations);
                }
            }
            Ok(Err(e))
                if e.contains("链上不存在")
                    || e.contains("不是已激活")
                    || e.contains("不能作为")
                    || e.contains("类型与 org 不匹配") =>
            {
                activations.retain(|a| a.subject_id_hex != lookup_subject_id_hex);
                let _ = save_activations(&app, &activations);
                return Ok(Vec::new());
            }
            _ => {}
        }
    }

    Ok(activations
        .iter()
        .filter(|a| a.subject_id_hex == lookup_subject_id_hex)
        .map(activated_admin_from_stored)
        .collect())
}

/// 取消管理员激活（需要设备密码）。
#[tauri::command]
pub fn deactivate_admin(
    app: AppHandle,
    pubkey_hex: String,
    sfid_number: String,
    subject_id_hex: Option<String>,
    expected_org: Option<u8>,
    unlock_password: String,
) -> Result<(), String> {
    if let Err(e) = security::append_audit_log(&app, "deactivate_admin", "attempt") {
        eprintln!("[审计] deactivate_admin attempt 日志写入失败: {e}");
    }

    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;

    let pubkey_clean = subject_id::normalize_pubkey_hex(&pubkey_hex)?;
    let lookup_subject_id_hex =
        resolve_activation_subject_id_hex(&sfid_number, subject_id_hex, expected_org)?;

    let mut activations = load_activations(&app)?;
    let before_len = activations.len();
    activations
        .retain(|a| !(a.pubkey_hex == pubkey_clean && a.subject_id_hex == lookup_subject_id_hex));

    if activations.len() == before_len {
        return Err("未找到该管理员的激活记录".to_string());
    }

    save_activations(&app, &activations)?;

    if let Err(e) = security::append_audit_log(
        &app,
        "deactivate_admin",
        &format!(
            "success pubkey={} subject={}",
            &pubkey_clean[..pubkey_clean.len().min(8)],
            &lookup_subject_id_hex
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
