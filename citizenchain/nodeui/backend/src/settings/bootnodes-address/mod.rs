use crate::{
    home,
    settings::{address_utils::decode_hex_32_strict, device_password, grandpa_address},
    shared::{security, validation::normalize_node_key},
};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind, path::PathBuf, thread, time::Duration};
use tauri::AppHandle;
use zeroize::Zeroizing;

const KEYCHAIN_ACCOUNT_BOOTNODE: &str = "bootnode-node-key";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 前端展示的引导节点私钥绑定状态。
pub struct BootnodeKey {
    pub node_key: Option<String>,
    pub peer_id: Option<String>,
    pub institution_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 创世引导节点选项，供前端/首页做 PeerId 到机构名映射。
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
    let raw = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("read bootnode meta failed: {e}")),
    };
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

fn peer_id_from_node_key_hex(node_key_hex: &str) -> Result<String, String> {
    let secret_bytes =
        decode_hex_32_strict(node_key_hex).map_err(|_| "node-key 格式无效".to_string())?;
    let secret = libp2p_identity::secp256k1::SecretKey::try_from_bytes(secret_bytes)
        .map_err(|_| "无效 node-key，无法生成 secp256k1 私钥".to_string())?;
    let keypair = libp2p_identity::secp256k1::Keypair::from(secret);
    let public = libp2p_identity::PublicKey::from(keypair.public().clone());
    let peer_id: PeerId = public.to_peer_id();
    Ok(peer_id.to_string())
}

// node-key 只在本地安全存储中保存，真正启动节点时再临时解密注入。
pub(crate) fn load_bootnode_node_key(
    _app: &AppHandle,
    unlock_password: &str,
) -> Result<Option<String>, String> {
    let Some(enveloped) = security::secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)? else {
        return Ok(None);
    };
    let key = security::decrypt_secret_value(&enveloped, unlock_password)?;
    Ok(Some(key))
}

pub(crate) fn verify_bootnode_secret_unlock(unlock_password: &str) -> Result<(), String> {
    if let Some(enveloped) = security::secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)? {
        let _key = Zeroizing::new(security::decrypt_secret_value(&enveloped, unlock_password)?);
    }
    Ok(())
}

fn wait_peer_id_applied(app: &AppHandle, expected_peer_id: &str) -> Result<(), String> {
    for _ in 0..20 {
        if let Ok(identity) = home::get_node_identity_blocking(app.clone()) {
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
    let _ = security::append_audit_log(&app, "set_bootnode_key", "attempt");
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;
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

    // 若节点当前在运行，保存新私钥后立即重启以应用新的 p2p 身份，
    // 并轮询确认本机 PeerId 已切换到目标引导节点。
    if home::current_status(&app)?.running {
        if let Err(err) = (|| -> Result<(), String> {
            let _ = home::stop_node_blocking(app.clone())?;
            let _ = home::start_node_blocking(app.clone(), unlock.to_string())?;
            wait_peer_id_applied(&app, &derived_peer_id)?;
            Ok(())
        })() {
            let _ = security::append_audit_log(&app, "set_bootnode_key", "saved_restart_failed");
            return Err(format!(
                "引导节点私钥已保存，但节点重启失败：{err}。新密钥将在下次成功启动节点时自动生效。"
            ));
        }
    }
    let _ = security::append_audit_log(&app, "set_bootnode_key", "success");

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
