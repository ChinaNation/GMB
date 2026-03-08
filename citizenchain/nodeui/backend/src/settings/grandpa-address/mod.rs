use crate::{home::home_node, settings::security, validation::normalize_grandpa_key};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    thread,
    time::Duration,
};
use tauri::AppHandle;
use zeroize::Zeroizing;

const KEYCHAIN_ACCOUNT_GRANDPA: &str = "grandpa-key";
const GRANDPA_KEY_TYPE_HEX_PREFIX: &str = "6772616e";
const DEFAULT_CHAIN_ID: &str = "citizenchain";
const LOCAL_RPC_ADDR: &str = "127.0.0.1:9944";
const INSTITUTION_CATALOG_SRC: &str = include_str!("../institution-catalog.json");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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

pub(crate) fn load_institution_catalog() -> Result<Vec<InstitutionCatalogEntry>, String> {
    let entries: Vec<InstitutionCatalogEntry> = serde_json::from_str(INSTITUTION_CATALOG_SRC)
        .map_err(|e| format!("parse institution-catalog.json failed: {e}"))?;
    if entries.is_empty() {
        return Err("institution-catalog.json 为空".to_string());
    }

    let mut seen_names = HashSet::new();
    let mut seen_peer_ids = HashSet::new();
    let mut seen_grandpa = HashSet::new();
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
    }

    Ok(entries)
}

fn grandpa_meta_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("grandpa-meta.json"))
}

fn load_grandpa_meta(app: &AppHandle) -> Result<Option<StoredGrandpaMeta>, String> {
    let path = grandpa_meta_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read grandpa meta failed: {e}"))?;
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

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = security::app_data_dir(app)?.join("node-data");
    fs::create_dir_all(&path).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(path)
}

fn keystore_dirs_for_grandpa(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let chains_root = node_data_dir(app)?.join("chains");
    fs::create_dir_all(&chains_root).map_err(|e| format!("create chains dir failed: {e}"))?;

    let mut dirs: Vec<PathBuf> = Vec::new();
    if chains_root.exists() {
        let entries =
            fs::read_dir(&chains_root).map_err(|e| format!("read chains dir failed: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read chain dir entry failed: {e}"))?;
            if !entry.path().is_dir() {
                continue;
            }
            dirs.push(entry.path().join("keystore"));
        }
    }

    dirs.push(chains_root.join(DEFAULT_CHAIN_ID).join("keystore"));

    dirs.sort();
    dirs.dedup();
    for dir in &dirs {
        fs::create_dir_all(dir).map_err(|e| {
            format!(
                "create grandpa keystore dir failed ({}): {e}",
                dir.display()
            )
        })?;
    }

    Ok(dirs)
}

fn grandpa_keystore_filename(pubkey_hex: &str) -> String {
    format!("{GRANDPA_KEY_TYPE_HEX_PREFIX}{pubkey_hex}")
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
    let filename = grandpa_keystore_filename(pubkey_hex);

    for dir in keystore_dirs_for_grandpa(app)? {
        let path = dir.join(&filename);
        security::write_secret_text_atomic(&path, &content).map_err(|e| {
            format!(
                "write grandpa keystore file failed ({}): {e}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn has_grandpa_key_in_keystore(app: &AppHandle, pubkey_hex: &str) -> Result<bool, String> {
    let filename = grandpa_keystore_filename(pubkey_hex);
    for dir in keystore_dirs_for_grandpa(app)? {
        if dir.join(&filename).is_file() {
            return Ok(true);
        }
    }
    Ok(false)
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

fn decode_hex_32(input: &str) -> Result<[u8; 32], String> {
    if input.len() != 64 {
        return Err("node-key 长度无效，必须是 64 位十六进制字符串".to_string());
    }
    let mut out = [0u8; 32];
    for (i, chunk) in input.as_bytes().chunks_exact(2).enumerate() {
        let part = std::str::from_utf8(chunk).map_err(|_| "node-key 格式无效".to_string())?;
        out[i] = u8::from_str_radix(part, 16).map_err(|_| "node-key 格式无效".to_string())?;
    }
    Ok(out)
}

fn grandpa_pubkey_from_private_hex(key_hex: &str) -> Result<String, String> {
    let secret = decode_hex_32(key_hex)?;
    let signing = ed25519_dalek::SigningKey::from_bytes(&secret);
    let verify = signing.verifying_key();
    Ok(hex::encode(verify.to_bytes()))
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    })
    .to_string();

    let req = format!(
        "POST / HTTP/1.1\r\nHost: {LOCAL_RPC_ADDR}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let addr = LOCAL_RPC_ADDR
        .parse()
        .map_err(|e| format!("parse RPC socket address failed: {e}"))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(600))
        .map_err(|e| format!("RPC 连接失败: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set RPC read timeout failed: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set RPC write timeout failed: {e}"))?;
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("RPC 写入失败: {e}"))?;

    let mut response = String::new();
    stream
        .take(4 * 1024 * 1024)
        .read_to_string(&mut response)
        .map_err(|e| format!("RPC 读取失败: {e}"))?;

    let Some((header, body)) = response.split_once("\r\n\r\n") else {
        return Err("RPC 响应格式错误：缺少 header/body 分隔符".to_string());
    };
    let status_line = header
        .lines()
        .next()
        .ok_or_else(|| "RPC 响应格式错误：缺少状态行".to_string())?;
    if !status_line.contains(" 200 ") {
        return Err(format!("RPC HTTP 状态异常: {status_line}"));
    }

    let json: Value = serde_json::from_str(body).map_err(|e| format!("RPC JSON 解析失败: {e}"))?;
    if let Some(err) = json.get("error") {
        return Err(format!("RPC 返回错误: {err}"));
    }

    Ok(json.get("result").cloned().unwrap_or(Value::Null))
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
    let key = Zeroizing::new(security::decrypt_secret_value(&enveloped, unlock_password)?);
    Ok(Some(key.to_string()))
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
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    security::verify_device_login_password(unlock)?;
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
    if home_node::current_status(&app)?.running {
        let _ = home_node::stop_node(app.clone())?;
        let _ = home_node::start_node(app.clone(), unlock.to_string())?;
        verify_grandpa_after_start(&app, unlock)?;
    }

    Ok(GrandpaKey {
        key: None,
        institution_name: Some(institution_name),
    })
}
