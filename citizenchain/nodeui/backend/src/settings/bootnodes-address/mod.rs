use crate::{
    home::home_node,
    settings::{grandpa_address, security},
    validation::normalize_node_key,
};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, thread, time::Duration};
use tauri::AppHandle;
use zeroize::Zeroizing;

const KEYCHAIN_ACCOUNT_BOOTNODE: &str = "bootnode-node-key";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootnodeKey {
    pub node_key: Option<String>,
    pub peer_id: Option<String>,
    pub institution_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisBootnodeOption {
    pub name: String,
    pub peer_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredBootnodeMeta {
    peer_id: String,
    #[serde(default)]
    institution_name: Option<String>,
}

fn bootnode_meta_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("bootnode-meta.json"))
}

fn load_bootnode_meta(app: &AppHandle) -> Result<Option<StoredBootnodeMeta>, String> {
    let path = bootnode_meta_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read bootnode meta failed: {e}"))?;
    let record: StoredBootnodeMeta =
        serde_json::from_str(&raw).map_err(|e| format!("parse bootnode meta failed: {e}"))?;
    Ok(Some(record))
}

fn save_bootnode_meta(
    app: &AppHandle,
    peer_id: &str,
    institution_name: Option<String>,
) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredBootnodeMeta {
        peer_id: peer_id.to_string(),
        institution_name,
    })
    .map_err(|e| format!("encode bootnode meta failed: {e}"))?;
    security::write_text_atomic(&bootnode_meta_path(app)?, &format!("{raw}\n"))
        .map_err(|e| format!("write bootnode meta failed: {e}"))
}

pub(crate) fn genesis_bootnode_options() -> Result<Vec<GenesisBootnodeOption>, String> {
    let options = grandpa_address::load_institution_catalog()?
        .into_iter()
        .map(|entry| GenesisBootnodeOption {
            name: entry.name,
            peer_id: entry.peer_id,
        })
        .collect::<Vec<GenesisBootnodeOption>>();
    if options.is_empty() {
        return Err("未配置创世引导节点".to_string());
    }
    Ok(options)
}

pub(crate) fn find_genesis_bootnode_name_by_peer_id(
    peer_id: &str,
) -> Result<Option<String>, String> {
    Ok(genesis_bootnode_options()?
        .into_iter()
        .find(|n| n.peer_id == peer_id)
        .map(|n| n.name))
}

fn is_genesis_bootnode_peer_id(peer_id: &str) -> Result<bool, String> {
    Ok(genesis_bootnode_options()?
        .iter()
        .any(|node| node.peer_id == peer_id))
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

fn peer_id_from_node_key_hex(node_key_hex: &str) -> Result<String, String> {
    let secret_bytes = decode_hex_32(node_key_hex)?;
    let secret = libp2p_identity::secp256k1::SecretKey::try_from_bytes(secret_bytes)
        .map_err(|_| "无效 node-key，无法生成 secp256k1 私钥".to_string())?;
    let keypair = libp2p_identity::secp256k1::Keypair::from(secret);
    let public = libp2p_identity::PublicKey::from(keypair.public().clone());
    let peer_id: PeerId = public.to_peer_id();
    Ok(peer_id.to_string())
}

pub(crate) fn load_bootnode_node_key(
    _app: &AppHandle,
    unlock_password: &str,
) -> Result<Option<String>, String> {
    let Some(enveloped) = security::secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)? else {
        return Ok(None);
    };
    let key = Zeroizing::new(security::decrypt_secret_value(&enveloped, unlock_password)?);
    Ok(Some(key.to_string()))
}

pub(crate) fn verify_bootnode_secret_unlock(unlock_password: &str) -> Result<(), String> {
    if let Some(enveloped) = security::secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)? {
        let _key = Zeroizing::new(security::decrypt_secret_value(&enveloped, unlock_password)?);
    }
    Ok(())
}

fn wait_peer_id_applied(app: &AppHandle, expected_peer_id: &str) -> Result<(), String> {
    for _ in 0..20 {
        if let Ok(identity) = home_node::get_node_identity(app.clone()) {
            if identity.peer_id.as_deref() == Some(expected_peer_id) {
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err(format!(
        "节点重启后 PeerId 未切换到目标引导节点（expected={expected_peer_id}）"
    ))
}

#[tauri::command]
pub fn get_bootnode_key(app: AppHandle) -> Result<BootnodeKey, String> {
    if security::secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)?.is_none() {
        return Ok(BootnodeKey {
            node_key: None,
            peer_id: None,
            institution_name: None,
        });
    }

    match load_bootnode_meta(&app)? {
        Some(meta) => Ok(BootnodeKey {
            // 私钥不回传给前端，避免二次暴露。
            node_key: None,
            peer_id: Some(meta.peer_id),
            institution_name: meta.institution_name,
        }),
        None => Ok(BootnodeKey {
            node_key: None,
            peer_id: None,
            institution_name: None,
        }),
    }
}

#[tauri::command]
pub fn set_bootnode_key(
    app: AppHandle,
    node_key: String,
    unlock_password: String,
) -> Result<BootnodeKey, String> {
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    security::verify_device_login_password(unlock)?;
    let normalized = normalize_node_key(&node_key)?;
    let derived_peer_id = peer_id_from_node_key_hex(&normalized)?;
    if !is_genesis_bootnode_peer_id(&derived_peer_id)? {
        return Err(format!(
            "该私钥不对应任何创世引导节点（推导 Peer ID: {derived_peer_id}）"
        ));
    }
    let institution_name = find_genesis_bootnode_name_by_peer_id(&derived_peer_id)?;

    let normalized = Zeroizing::new(normalized);
    let encrypted = security::encrypt_secret_value(&normalized, unlock)?;
    security::secure_store_set(KEYCHAIN_ACCOUNT_BOOTNODE, &encrypted)?;
    save_bootnode_meta(&app, &derived_peer_id, institution_name.clone())?;

    // 若节点当前在运行，保存新私钥后立即重启以应用新的 p2p 身份。
    if home_node::current_status(&app)?.running {
        let _ = home_node::stop_node(app.clone())?;
        let _ = home_node::start_node(app.clone(), unlock.to_string())?;
        wait_peer_id_applied(&app, &derived_peer_id)?;
    }

    Ok(BootnodeKey {
        node_key: None,
        peer_id: Some(derived_peer_id),
        institution_name,
    })
}

#[tauri::command]
pub fn get_genesis_bootnode_options() -> Result<Vec<GenesisBootnodeOption>, String> {
    genesis_bootnode_options()
}
