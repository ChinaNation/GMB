use crate::{
    home,
    settings::{address_utils::decode_hex_32_strict, device_password},
    shared::{keystore, rpc, security, validation::normalize_grandpa_key},
};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet, fs, io::ErrorKind, path::PathBuf, str::FromStr, sync::OnceLock, thread,
    time::Duration,
};
use tauri::AppHandle;
use zeroize::Zeroizing;

const KEYCHAIN_ACCOUNT_GRANDPA: &str = "grandpa-key";
const GRANDPA_KEY_TYPE_HEX_PREFIX: &str = "6772616e";
const INSTITUTION_CATALOG_SRC: &str = include_str!("../institution-catalog.json");
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_RPC_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 前端展示的 GRANDPA 私钥绑定状态。
pub struct GrandpaKey {
    pub key: Option<String>,
    pub institution_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredGrandpaMeta {
    #[serde(default)]
    institution_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstitutionCatalogEntry {
    pub name: String,
    pub peer_id: String,
    pub grandpa_pubkey_hex: String,
}

static INSTITUTION_CATALOG: OnceLock<Vec<InstitutionCatalogEntry>> = OnceLock::new();

/// 获取机构清单（OnceLock 惰性初始化，编译期内嵌 JSON 只解析一次）。
pub(crate) fn load_institution_catalog() -> Result<Vec<InstitutionCatalogEntry>, String> {
    if let Some(catalog) = INSTITUTION_CATALOG.get() {
        return Ok(catalog.clone());
    }
    let catalog = parse_institution_catalog()?;
    let _ = INSTITUTION_CATALOG.set(catalog);
    Ok(INSTITUTION_CATALOG.get().unwrap().clone())
}

// 机构清单既被 bootnode 模块用于 PeerId 映射，也被 GRANDPA 模块用于公钥匹配，
// 加载时统一做 trim / 去重 / 格式校验，避免配置里的空格或脏数据影响运行时判断。
fn parse_institution_catalog() -> Result<Vec<InstitutionCatalogEntry>, String> {
    let entries: Vec<InstitutionCatalogEntry> = serde_json::from_str(INSTITUTION_CATALOG_SRC)
        .map_err(|e| format!("parse institution-catalog.json failed: {e}"))?;
    if entries.is_empty() {
        return Err("institution-catalog.json 为空".to_string());
    }

    let mut seen_names = HashSet::new();
    let mut seen_peer_ids = HashSet::new();
    let mut seen_grandpa = HashSet::new();
    let mut normalized_entries = Vec::with_capacity(entries.len());
    for (idx, entry) in entries.iter().enumerate() {
        let line = idx + 1;
        let name = entry.name.trim();
        if name.is_empty() {
            return Err(format!(
                "institution-catalog.json 第 {line} 项 name 不能为空"
            ));
        }
        if !seen_names.insert(name.to_string()) {
            return Err(format!("institution-catalog.json 机构名重复: {name}"));
        }

        let peer_id = entry.peer_id.trim();
        if peer_id.is_empty() {
            return Err(format!(
                "institution-catalog.json 第 {line} 项 peerId 不能为空"
            ));
        }
        PeerId::from_str(peer_id)
            .map_err(|_| format!("institution-catalog.json 第 {line} 项 peerId 无效"))?;
        if !seen_peer_ids.insert(peer_id.to_string()) {
            return Err(format!("institution-catalog.json peerId 重复: {peer_id}"));
        }

        let grandpa = entry.grandpa_pubkey_hex.trim();
        if grandpa.len() != 64 || !grandpa.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "institution-catalog.json 第 {line} 项 grandpaPubkeyHex 无效"
            ));
        }
        let grandpa_lower = grandpa.to_ascii_lowercase();
        if !seen_grandpa.insert(grandpa_lower) {
            return Err(format!(
                "institution-catalog.json GRANDPA 公钥重复: {}",
                entry.grandpa_pubkey_hex
            ));
        }

        normalized_entries.push(InstitutionCatalogEntry {
            name: name.to_string(),
            peer_id: peer_id.to_string(),
            grandpa_pubkey_hex: grandpa.to_ascii_lowercase(),
        });
    }

    Ok(normalized_entries)
}

fn grandpa_meta_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("grandpa-meta.json"))
}

fn load_grandpa_meta(app: &AppHandle) -> Result<Option<StoredGrandpaMeta>, String> {
    let path = grandpa_meta_path(app)?;
    let raw = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("read grandpa meta failed: {e}")),
    };
    let record: StoredGrandpaMeta =
        serde_json::from_str(&raw).map_err(|e| format!("parse grandpa meta failed: {e}"))?;
    Ok(Some(record))
}

fn save_grandpa_meta(app: &AppHandle, institution_name: Option<String>) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredGrandpaMeta { institution_name })
        .map_err(|e| format!("encode grandpa meta failed: {e}"))?;
    security::write_text_atomic(&grandpa_meta_path(app)?, &format!("{raw}\n"))
        .map_err(|e| format!("write grandpa meta failed: {e}"))
}

fn write_grandpa_key_to_keystore(
    app: &AppHandle,
    private_hex: &str,
    pubkey_hex: &str,
) -> Result<(), String> {
    let secret = format!("0x{private_hex}");
    let encoded = serde_json::to_string(&secret)
        .map_err(|e| format!("encode grandpa keystore secret failed: {e}"))?;
    let content = format!("{encoded}\n");
    let dirs = keystore::keystore_dirs(app)?;
    // 始终只保留当前机构对应的一把 gran 密钥，避免旧密钥残留导致节点加载多把 authority key。
    keystore::write_key_to_keystore(&dirs, GRANDPA_KEY_TYPE_HEX_PREFIX, pubkey_hex, &content)
}

fn has_grandpa_key_in_keystore(app: &AppHandle, pubkey_hex: &str) -> Result<bool, String> {
    let dirs = keystore::keystore_dirs(app)?;
    Ok(keystore::has_key_in_keystore(&dirs, GRANDPA_KEY_TYPE_HEX_PREFIX, pubkey_hex))
}

fn grandpa_institution_options() -> Result<Vec<(String, String)>, String> {
    let out = load_institution_catalog()?
        .into_iter()
        .map(|entry| (entry.name, entry.grandpa_pubkey_hex.to_ascii_lowercase()))
        .collect::<Vec<(String, String)>>();
    if out.is_empty() {
        return Err("未配置 GRANDPA 权威公钥".to_string());
    }
    Ok(out)
}

fn institution_name_by_grandpa_pubkey(pubkey_hex: &str) -> Result<Option<String>, String> {
    Ok(grandpa_institution_options()?
        .into_iter()
        .find(|(_, key)| key.eq_ignore_ascii_case(pubkey_hex))
        .map(|(name, _)| name))
}

fn grandpa_pubkey_from_private_hex(key_hex: &str) -> Result<String, String> {
    let secret = decode_hex_32_strict(key_hex)
        .map_err(|_| "GRANDPA 私钥格式无效，应为 64 位十六进制".to_string())?;
    let signing = ed25519_dalek::SigningKey::from_bytes(&secret);
    let verify = signing.verifying_key();
    Ok(hex::encode(verify.to_bytes()))
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(method, params, RPC_REQUEST_TIMEOUT, MAX_RPC_RESPONSE_BYTES)
}

fn node_roles() -> Result<Vec<String>, String> {
    let value = rpc_post("system_nodeRoles", Value::Array(vec![]))?;
    let Some(list) = value.as_array() else {
        return Ok(Vec::new());
    };
    Ok(list
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect())
}

fn is_authority_role(roles: &[String]) -> bool {
    roles.iter().any(|role| {
        let lower = role.to_ascii_lowercase();
        lower.contains("authority") || lower.contains("validator")
    })
}

fn wait_for_authority_role() -> Result<(), String> {
    for _ in 0..20 {
        if let Ok(roles) = node_roles() {
            if is_authority_role(&roles) {
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err("节点未进入 AUTHORITY/VALIDATOR 角色，无法成为投票节点".to_string())
}

fn load_saved_grandpa_private_hex(unlock_password: &str) -> Result<Option<String>, String> {
    let Some(enveloped) = security::secure_store_get(KEYCHAIN_ACCOUNT_GRANDPA)? else {
        return Ok(None);
    };
    let key = security::decrypt_secret_value(&enveloped, unlock_password)?;
    Ok(Some(key))
}

pub(crate) fn verify_grandpa_secret_unlock(unlock_password: &str) -> Result<(), String> {
    if let Some(enveloped) = security::secure_store_get(KEYCHAIN_ACCOUNT_GRANDPA)? {
        let _key = Zeroizing::new(security::decrypt_secret_value(&enveloped, unlock_password)?);
    }
    Ok(())
}

pub(crate) fn prepare_grandpa_for_start(
    app: &AppHandle,
    unlock_password: &str,
) -> Result<bool, String> {
    // 启动前再次校验“私钥 -> 公钥 -> 机构清单”关系，防止历史配置和当前清单漂移后误启动 validator。
    let Some(private_hex) = load_saved_grandpa_private_hex(unlock_password)? else {
        return Ok(false);
    };

    let pubkey = grandpa_pubkey_from_private_hex(&private_hex)?;
    if institution_name_by_grandpa_pubkey(&pubkey)?.is_none() {
        return Err(format!(
            "已保存的投票私钥不在当前 GRANDPA 权威列表中（推导公钥: 0x{pubkey}）"
        ));
    }

    write_grandpa_key_to_keystore(app, &private_hex, &pubkey)?;
    Ok(true)
}

pub(crate) fn verify_grandpa_after_start(
    app: &AppHandle,
    unlock_password: &str,
) -> Result<(), String> {
    let Some(private_hex) = load_saved_grandpa_private_hex(unlock_password)? else {
        return Ok(());
    };
    let pubkey = grandpa_pubkey_from_private_hex(&private_hex)?;

    wait_for_authority_role()?;
    if !has_grandpa_key_in_keystore(app, &pubkey)? {
        return Err(format!(
            "未在本地 keystore 检测到 GRANDPA 密钥文件（pubkey=0x{pubkey}）"
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn get_grandpa_key(app: AppHandle) -> Result<GrandpaKey, String> {
    if security::secure_store_get(KEYCHAIN_ACCOUNT_GRANDPA)?.is_none() {
        return Ok(GrandpaKey {
            key: None,
            institution_name: None,
        });
    }

    let institution_name = load_grandpa_meta(&app)?.and_then(|v| v.institution_name);
    Ok(GrandpaKey {
        // 私钥不回传给前端，避免二次暴露。
        key: None,
        institution_name,
    })
}

#[tauri::command]
pub fn set_grandpa_key(
    app: AppHandle,
    key: String,
    unlock_password: String,
) -> Result<GrandpaKey, String> {
    let _ = security::append_audit_log(&app, "set_grandpa_key", "attempt");
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;
    let normalized = normalize_grandpa_key(&key)?;
    let pubkey = grandpa_pubkey_from_private_hex(&normalized)?;
    let institution_name = institution_name_by_grandpa_pubkey(&pubkey)?
        .ok_or_else(|| format!("私钥与任何机构 GRANDPA 公钥不匹配（推导公钥: 0x{pubkey}）"))?;

    let normalized = Zeroizing::new(normalized);
    let encrypted = security::encrypt_secret_value(&normalized, unlock)?;
    security::secure_store_set(KEYCHAIN_ACCOUNT_GRANDPA, &encrypted)?;
    save_grandpa_meta(&app, Some(institution_name.clone()))?;
    write_grandpa_key_to_keystore(&app, &normalized, &pubkey)?;

    // 若节点当前在运行，保存后立即重启以 authority 模式加载并参与投票。
    if home::current_status(&app)?.running {
        if let Err(err) = (|| -> Result<(), String> {
            let _ = home::stop_node_blocking(app.clone())?;
            let _ = home::start_node_blocking(app.clone(), unlock.to_string())?;
            verify_grandpa_after_start(&app, unlock)?;
            Ok(())
        })() {
            let _ = security::append_audit_log(&app, "set_grandpa_key", "saved_restart_failed");
            return Err(format!(
                "GRANDPA 私钥已保存，但节点重启或校验失败：{err}。新密钥将在下次成功启动节点时自动生效。"
            ));
        }
    }
    let _ = security::append_audit_log(&app, "set_grandpa_key", "success");

    Ok(GrandpaKey {
        key: None,
        institution_name: Some(institution_name),
    })
}
