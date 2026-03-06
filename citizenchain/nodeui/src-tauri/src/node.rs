use crate::validation::{
    normalize_grandpa_key, normalize_node_key, normalize_node_name, normalize_wallet_address,
};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use keyring::Entry;
use libp2p_identity::PeerId;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::{
    collections::HashMap,
    collections::HashSet,
    fs,
    fs::OpenOptions,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use std::hash::Hasher;
use tauri::{AppHandle, Manager};

pub struct RuntimeState {
    pub local_node: Option<Child>,
    pub node_key_file: Option<PathBuf>,
}

pub struct AppState(pub Mutex<RuntimeState>);

const KEYCHAIN_SERVICE: &str = "org.chinanation.citizenchain.desktop";
const KEYCHAIN_ACCOUNT_BOOTNODE: &str = "bootnode-node-key";
const KEYCHAIN_ACCOUNT_GRANDPA: &str = "grandpa-key";
const SECRET_FORMAT_VERSION: u8 = 1;
const PBKDF2_ROUNDS: u32 = 210_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub running: bool,
    pub state: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardWallet {
    pub address: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootnodeKey {
    pub node_key: Option<String>,
    pub peer_id: Option<String>,
    pub institution_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrandpaKey {
    pub key: Option<String>,
    pub institution_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainStatus {
    pub block_height: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeIdentity {
    pub node_name: Option<String>,
    pub peer_id: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningIncome {
    pub total_income: String,
    pub total_fee_income: String,
    pub total_reward_income: String,
    pub today_income: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningBlockRecord {
    pub block_height: u64,
    pub timestamp_ms: Option<u64>,
    pub fee: String,
    pub block_reward: String,
    pub author: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUsage {
    pub cpu_percent: Option<f64>,
    pub memory_mb: Option<u64>,
    pub disk_usage_percent: Option<f64>,
    pub node_data_size_mb: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningDashboard {
    pub income: MiningIncome,
    pub records: Vec<MiningBlockRecord>,
    pub resources: ResourceUsage,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkOverview {
    pub total_nodes: u64,
    pub online_nodes: u64,
    pub guochuhui_nodes: u64,
    pub shengchuhui_nodes: u64,
    pub shengchuhang_nodes: u64,
    pub full_nodes: u64,
    pub light_nodes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisBootnodeOption {
    pub name: String,
    pub peer_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredWallet {
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredNodeName {
    node_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredBootnodeMeta {
    peer_id: String,
    #[serde(default)]
    institution_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredGrandpaMeta {
    #[serde(default)]
    institution_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptedSecretEnvelope {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    cipher_b64: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct StoredKnownPeers {
    #[serde(default)]
    peer_ids: Vec<String>,
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))?;
    fs::create_dir_all(&app_data).map_err(|e| format!("create app data dir failed: {e}"))?;
    Ok(app_data)
}

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data = app_data_dir(app)?.join("node-data");
    fs::create_dir_all(&data).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(data)
}

fn reward_wallet_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("reward-wallet.json"))
}

fn bootnode_meta_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("bootnode-meta.json"))
}

fn node_name_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("node-name.json"))
}

fn grandpa_meta_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("grandpa-meta.json"))
}

fn known_peers_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("known-peers.json"))
}

fn secure_store_entry(account: &str) -> Result<Entry, String> {
    Entry::new(KEYCHAIN_SERVICE, account).map_err(|e| format!("初始化系统安全存储失败: {e}"))
}

fn secure_store_set(account: &str, value: &str) -> Result<(), String> {
    let entry = secure_store_entry(account)?;
    entry
        .set_password(value)
        .map_err(|e| format!("写入系统安全存储失败: {e}"))
}

fn secure_store_get(account: &str) -> Result<Option<String>, String> {
    let entry = secure_store_entry(account)?;
    match entry.get_password() {
        Ok(v) => {
            let value = v.trim().to_string();
            if value.is_empty() {
                return Ok(None);
            }
            Ok(Some(value))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("读取系统安全存储失败: {e}")),
    }
}

fn ensure_unlock_password(password: &str) -> Result<&str, String> {
    let trimmed = password.trim();
    if trimmed.is_empty() {
        return Err("设备开机密码不能为空".to_string());
    }
    Ok(trimmed)
}

#[cfg(target_os = "macos")]
fn verify_device_login_password(password: &str) -> Result<(), String> {
    let user = std::env::var("USER").map_err(|e| format!("读取系统用户失败: {e}"))?;
    let output = Command::new("dscl")
        .args(["/Search", "-authonly", &user, password])
        .output()
        .map_err(|e| format!("校验设备密码失败: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err("设备开机密码错误".to_string())
}

#[cfg(not(target_os = "macos"))]
fn verify_device_login_password(_password: &str) -> Result<(), String> {
    // 非 macOS 先保留通过，后续接入各平台原生设备认证。
    Ok(())
}

fn derive_key_from_password(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ROUNDS, &mut key);
    key
}

fn encrypt_secret_value(secret: &str, password: &str) -> Result<String, String> {
    let unlock = ensure_unlock_password(password)?;
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce);
    let key = derive_key_from_password(unlock, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建加密器失败: {e}"))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), secret.as_bytes())
        .map_err(|_| "私钥加密失败".to_string())?;
    let envelope = EncryptedSecretEnvelope {
        version: SECRET_FORMAT_VERSION,
        salt_b64: BASE64.encode(salt),
        nonce_b64: BASE64.encode(nonce),
        cipher_b64: BASE64.encode(ciphertext),
    };
    serde_json::to_string(&envelope).map_err(|e| format!("编码加密数据失败: {e}"))
}

fn decrypt_secret_value(enveloped: &str, password: &str) -> Result<String, String> {
    let unlock = ensure_unlock_password(password)?;
    let envelope: EncryptedSecretEnvelope =
        serde_json::from_str(enveloped).map_err(|e| format!("解析密文数据失败: {e}"))?;
    if envelope.version != SECRET_FORMAT_VERSION {
        return Err("密文版本不支持".to_string());
    }

    let salt = BASE64
        .decode(envelope.salt_b64)
        .map_err(|_| "密文盐值损坏".to_string())?;
    let nonce = BASE64
        .decode(envelope.nonce_b64)
        .map_err(|_| "密文随机数损坏".to_string())?;
    let cipher_bytes = BASE64
        .decode(envelope.cipher_b64)
        .map_err(|_| "密文载荷损坏".to_string())?;
    if salt.len() != 16 || nonce.len() != 12 {
        return Err("密文参数长度无效".to_string());
    }

    let mut salt_arr = [0u8; 16];
    salt_arr.copy_from_slice(&salt);
    let key = derive_key_from_password(unlock, &salt_arr);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建解密器失败: {e}"))?;
    let plain = cipher
        .decrypt(Nonce::from_slice(&nonce), cipher_bytes.as_ref())
        .map_err(|_| "解锁密码错误或密文已损坏".to_string())?;
    String::from_utf8(plain).map_err(|_| "私钥内容格式无效".to_string())
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
    fs::write(bootnode_meta_path(app)?, format!("{raw}\n"))
        .map_err(|e| format!("write bootnode meta failed: {e}"))
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
    fs::write(grandpa_meta_path(app)?, format!("{raw}\n"))
        .map_err(|e| format!("write grandpa meta failed: {e}"))
}

fn node_key_runtime_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app_data_dir(app)?.join("runtime-secrets");
    fs::create_dir_all(&path).map_err(|e| format!("create runtime secrets dir failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700));
    }
    Ok(path)
}

fn write_node_key_runtime_file(app: &AppHandle, node_key: &str) -> Result<PathBuf, String> {
    let dir = node_key_runtime_dir(app)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("get system time failed: {e}"))?
        .as_nanos();
    let pid = std::process::id();

    // create_new 避免覆盖已有文件；循环保证文件名冲突时可重试。
    for seq in 0u32..32 {
        let path = dir.join(format!("node-key-{pid}-{ts}-{seq}.tmp"));
        #[cfg(unix)]
        let file = {
            use std::os::unix::fs::OpenOptionsExt;
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
        };
        #[cfg(not(unix))]
        let file = OpenOptions::new().write(true).create_new(true).open(&path);

        match file {
            Ok(mut f) => {
                f.write_all(node_key.as_bytes())
                    .and_then(|_| f.write_all(b"\n"))
                    .map_err(|e| format!("write node-key runtime file failed: {e}"))?;
                return Ok(path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("create node-key runtime file failed: {e}")),
        }
    }

    Err("create node-key runtime file failed: exhausted retries".to_string())
}

fn cleanup_node_key_runtime_file(path: Option<PathBuf>) {
    if let Some(path) = path {
        let _ = fs::remove_file(path);
    }
}

fn cleanup_node_key_runtime_file_in_state(state: &mut RuntimeState) {
    let path = state.node_key_file.take();
    cleanup_node_key_runtime_file(path);
}

fn quoted_value_after(source: &str, key: &str) -> Option<String> {
    let key_idx = source.find(key)?;
    let after_key = &source[key_idx + key.len()..];
    let first_quote = after_key.find('"')?;
    let quoted = &after_key[first_quote + 1..];
    let end_quote = quoted.find('"')?;
    Some(quoted[..end_quote].to_string())
}

fn peer_id_from_addr(addr: &str) -> Option<String> {
    let idx = addr.find("/p2p/")?;
    let peer_id = &addr[idx + "/p2p/".len()..];
    if peer_id.is_empty() {
        None
    } else {
        Some(peer_id.to_string())
    }
}

fn genesis_bootnode_options() -> Vec<GenesisBootnodeOption> {
    // 与链创世配置保持同源：直接从 node 的 chain_spec.rs 提取 name 与 /p2p/<PeerId>。
    const CHAIN_SPEC_SRC: &str = include_str!("../../../node/src/chain_spec.rs");
    CHAIN_SPEC_SRC
        .split("ChainSpecBootnode")
        .filter_map(|chunk| {
            let name = quoted_value_after(chunk, "name:")?;
            let addr = quoted_value_after(chunk, "addr:")?;
            let peer_id = peer_id_from_addr(&addr)?;
            Some(GenesisBootnodeOption { name, peer_id })
        })
        .collect()
}

fn grandpa_authority_keys_from_runtime_source() -> Vec<String> {
    const RUNTIME_SRC: &str = include_str!("../../../runtime/src/genesis_config_presets.rs");
    let Some(start_idx) = RUNTIME_SRC.find("const GRANDPA_AUTHORITY_KEYS_HEX:") else {
        return Vec::new();
    };
    let sliced = &RUNTIME_SRC[start_idx..];
    let Some(array_start) = sliced.find('[') else {
        return Vec::new();
    };
    let sliced = &sliced[array_start + 1..];
    let Some(array_end) = sliced.find("];") else {
        return Vec::new();
    };
    let body = &sliced[..array_end];

    body.lines()
        .filter_map(|line| {
            let first = line.find('"')?;
            let remain = &line[first + 1..];
            let second = remain.find('"')?;
            let key = &remain[..second];
            if key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit()) {
                Some(key.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect()
}

fn grandpa_institution_options() -> Vec<(String, String)> {
    let names = genesis_bootnode_options()
        .into_iter()
        .map(|b| b.name)
        .collect::<Vec<String>>();
    let keys = grandpa_authority_keys_from_runtime_source();
    let n = names.len().min(keys.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push((names[i].clone(), keys[i].clone()));
    }
    out
}

fn institution_name_by_grandpa_pubkey(pubkey_hex: &str) -> Option<String> {
    grandpa_institution_options()
        .into_iter()
        .find(|(_, key)| key.eq_ignore_ascii_case(pubkey_hex))
        .map(|(name, _)| name)
}

fn find_genesis_bootnode_name_by_peer_id(peer_id: &str) -> Option<String> {
    genesis_bootnode_options()
        .into_iter()
        .find(|n| n.peer_id == peer_id)
        .map(|n| n.name)
}

fn role_from_peer_id(peer_id: Option<&str>) -> String {
    if let Some(pid) = peer_id {
        if let Some(name) = find_genesis_bootnode_name_by_peer_id(pid) {
            return name;
        }
    }
    "全节点".to_string()
}

fn is_genesis_bootnode_peer_id(peer_id: &str) -> bool {
    genesis_bootnode_options()
        .iter()
        .any(|node| node.peer_id == peer_id)
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

fn grandpa_pubkey_from_private_hex(key_hex: &str) -> Result<String, String> {
    let secret = decode_hex_32(key_hex)?;
    let signing = ed25519_dalek::SigningKey::from_bytes(&secret);
    let verify = signing.verifying_key();
    Ok(hex::encode(verify.to_bytes()))
}

fn refresh_managed_process(state: &mut RuntimeState) -> (bool, Option<u32>) {
    if let Some(child) = state.local_node.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => {
                state.local_node = None;
                cleanup_node_key_runtime_file_in_state(state);
                (false, None)
            }
            Ok(None) => (true, Some(child.id())),
        }
    } else {
        (false, None)
    }
}

fn is_rpc_9944_reachable() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:9944".parse().expect("hardcoded socket address must parse"),
        Duration::from_millis(250),
    )
    .is_ok()
}

pub fn current_status(app: &AppHandle) -> Result<NodeStatus, String> {
    let app_state = app.state::<AppState>();
    let mut state = app_state
        .0
        .lock()
        .map_err(|_| "acquire process state failed".to_string())?;

    let (managed_running, pid) = refresh_managed_process(&mut state);
    // 真实运行状态：应用内管理进程在跑，或本机 RPC 端口可达。
    let running = managed_running || is_rpc_9944_reachable();

    Ok(NodeStatus {
        running,
        state: if running { "running" } else { "stopped" }.to_string(),
        pid,
    })
}

fn find_node_bin() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("../target/debug/node"));
        candidates.push(cwd.join("../target/release/node"));
        candidates.push(cwd.join("../../target/debug/node"));
        candidates.push(cwd.join("../../target/release/node"));
        candidates.push(cwd.join("sidecar/citizenchain-node"));
        candidates.push(cwd.join("desktop/sidecar/citizenchain-node"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join("../../target/debug/node"));
    candidates.push(manifest_dir.join("../../target/release/node"));
    candidates.push(manifest_dir.join("../target/debug/node"));
    candidates.push(manifest_dir.join("../target/release/node"));
    candidates.push(manifest_dir.join("binaries/citizenchain-node"));

    candidates.into_iter().find(|p| p.is_file())
}

fn load_bootnode_node_key(_app: &AppHandle, unlock_password: &str) -> Result<Option<String>, String> {
    let Some(enveloped) = secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)? else {
        return Ok(None);
    };
    let key = decrypt_secret_value(&enveloped, unlock_password)?;
    Ok(Some(key))
}

fn verify_start_unlock_password(unlock_password: &str) -> Result<(), String> {
    let unlock = ensure_unlock_password(unlock_password)?;
    verify_device_login_password(unlock)?;

    if let Some(enveloped) = secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)? {
        let _ = decrypt_secret_value(&enveloped, unlock)?;
    }
    if let Some(enveloped) = secure_store_get(KEYCHAIN_ACCOUNT_GRANDPA)? {
        let _ = decrypt_secret_value(&enveloped, unlock)?;
    }
    Ok(())
}

fn load_node_name(app: &AppHandle) -> Result<Option<String>, String> {
    let path = node_name_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read node-name failed: {e}"))?;
    let record: StoredNodeName =
        serde_json::from_str(&raw).map_err(|e| format!("parse node-name failed: {e}"))?;
    Ok(Some(record.node_name))
}

fn load_known_peers(app: &AppHandle) -> Result<HashSet<String>, String> {
    let path = known_peers_path(app)?;
    if !path.exists() {
        return Ok(HashSet::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read known peers failed: {e}"))?;
    let record: StoredKnownPeers =
        serde_json::from_str(&raw).map_err(|e| format!("parse known peers failed: {e}"))?;
    Ok(record.peer_ids.into_iter().collect())
}

fn save_known_peers(app: &AppHandle, peers: &HashSet<String>) -> Result<(), String> {
    let mut peer_ids: Vec<String> = peers.iter().cloned().collect();
    peer_ids.sort();
    let raw = serde_json::to_string_pretty(&StoredKnownPeers { peer_ids })
        .map_err(|e| format!("encode known peers failed: {e}"))?;
    fs::write(known_peers_path(app)?, format!("{raw}\n"))
        .map_err(|e| format!("write known peers failed: {e}"))
}

fn spawn_node(
    app: &AppHandle,
    node_bin: &Path,
    unlock_password: &str,
) -> Result<(Child, Option<PathBuf>), String> {
    let base_path = node_data_dir(app)?;
    let bootnode_key = load_bootnode_node_key(app, unlock_password)?;
    let node_name = load_node_name(app)?;
    let mut node_key_runtime_file: Option<PathBuf> = None;

    let mut cmd = Command::new(node_bin);
    cmd.arg("--base-path")
        .arg(base_path)
        .arg("--rpc-port")
        .arg("9944")
        .arg("--no-prometheus")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // 若用户配置了 node-key，则通过 --node-key-file 传递，避免私钥出现在进程参数中。
    if let Some(node_key) = bootnode_key {
        let key_file = write_node_key_runtime_file(app, &node_key)?;
        cmd.arg("--node-key-file").arg(&key_file);
        node_key_runtime_file = Some(key_file);
    }
    if let Some(name) = node_name {
        cmd.arg("--name").arg(name);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }
    }

    match cmd.spawn() {
        Ok(child) => Ok((child, node_key_runtime_file)),
        Err(e) => {
            cleanup_node_key_runtime_file(node_key_runtime_file);
            Err(format!("spawn node failed from {}: {e}", node_bin.display()))
        }
    }
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        let pid = child.id() as i32;
        if pid > 0 {
            let _ = libc::kill(-pid, libc::SIGTERM);
        }
    }

    let _ = child.kill();
    for _ in 0..20 {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(_) => return,
        }
    }
    let _ = child.kill();
    let _ = child.try_wait();
}

fn kill_external_nodes() {
    #[cfg(unix)]
    {
        let _ = Command::new("pkill").args(["-f", "citizenchain-node"]).status();
        let _ = Command::new("pkill").args(["-f", "/target/debug/node"]).status();
        let _ = Command::new("pkill").args(["-f", "/target/release/node"]).status();
    }
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    // 通过 JSON-RPC 读取真实链数据；端口与启动参数 --rpc-port 保持一致。
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    })
    .to_string();

    let req = format!(
        "POST / HTTP/1.1\r\nHost: 127.0.0.1:9944\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let mut stream = TcpStream::connect("127.0.0.1:9944").map_err(|e| format!("RPC 连接失败: {e}"))?;
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("RPC 写入失败: {e}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("RPC 读取失败: {e}"))?;

    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .ok_or_else(|| "RPC 响应格式错误".to_string())?;

    let json: Value = serde_json::from_str(body).map_err(|e| format!("RPC JSON 解析失败: {e}"))?;
    if let Some(err) = json.get("error") {
        return Err(format!("RPC 返回错误: {err}"));
    }

    Ok(json.get("result").cloned().unwrap_or(Value::Null))
}

fn hex_to_u64(hex: &str) -> Option<u64> {
    let trimmed = hex.strip_prefix("0x")?;
    u64::from_str_radix(trimmed, 16).ok()
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let trimmed = hex.strip_prefix("0x").unwrap_or(hex);
    if trimmed.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(trimmed.len() / 2);
    for i in (0..trimmed.len()).step_by(2) {
        let byte = u8::from_str_radix(&trimmed[i..i + 2], 16).ok()?;
        out.push(byte);
    }
    Some(out)
}

fn scale_u64_from_storage_hex(hex: &str) -> Option<u64> {
    let bytes = hex_to_bytes(hex)?;
    if bytes.len() < 8 {
        return None;
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[..8]);
    Some(u64::from_le_bytes(raw))
}

fn parse_partial_fee(result: &Value) -> Option<u128> {
    let raw = result.get("partialFee")?;
    if let Some(s) = raw.as_str() {
        return s.parse::<u128>().ok();
    }
    raw.as_u64().map(u128::from)
}

fn twox_128(input: &[u8]) -> [u8; 16] {
    let mut h1 = twox_hash::XxHash64::with_seed(0);
    h1.write(input);
    let mut h2 = twox_hash::XxHash64::with_seed(1);
    h2.write(input);

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h1.finish().to_le_bytes());
    out[8..].copy_from_slice(&h2.finish().to_le_bytes());
    out
}

fn timestamp_now_storage_key() -> String {
    let mut key = Vec::with_capacity(32);
    key.extend_from_slice(&twox_128(b"Timestamp"));
    key.extend_from_slice(&twox_128(b"Now"));
    format!("0x{}", hex::encode(key))
}

fn format_2_decimals_fen(amount_fen: u128) -> String {
    let major = amount_fen / 100;
    let minor = amount_fen % 100;
    format!("{major}.{minor:02}")
}

fn block_reward_fen_by_height(height: u64) -> u128 {
    // fullnode-pow-reward 制度常量：奖励区间 [1, 9_999_999]，单块 9999.00（=999_900 分）
    if (1..=9_999_999).contains(&height) {
        999_900
    } else {
        0
    }
}

fn decode_scale_compact_u32_prefix(bytes: &[u8]) -> Option<(usize, usize)> {
    let first = *bytes.first()?;
    match first & 0b11 {
        0b00 => Some((((first >> 2) as usize), 1)),
        0b01 => {
            if bytes.len() < 2 {
                return None;
            }
            let v = u16::from_le_bytes([bytes[0], bytes[1]]) >> 2;
            Some((v as usize, 2))
        }
        0b10 => {
            if bytes.len() < 4 {
                return None;
            }
            let v = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) >> 2;
            Some((v as usize, 4))
        }
        0b11 => {
            let byte_len = ((first >> 2) as usize) + 4;
            if bytes.len() < 1 + byte_len || byte_len > 4 {
                return None;
            }
            let mut v: usize = 0;
            for (i, b) in bytes[1..1 + byte_len].iter().enumerate() {
                v |= (*b as usize) << (8 * i);
            }
            Some((v, 1 + byte_len))
        }
        _ => None,
    }
}

fn author_from_pow_digest_logs(logs: &[Value]) -> Option<String> {
    for log in logs {
        let Some(s) = log.as_str() else {
            continue;
        };
        let bytes = hex_to_bytes(s)?;
        if bytes.len() < 6 {
            continue;
        }
        // DigestItem::PreRuntime + engine_id == "pow_"
        if bytes[0] != 0x06 || &bytes[1..5] != b"pow_" {
            continue;
        }

        let payload = &bytes[5..];
        let Some((payload_len, prefix_len)) = decode_scale_compact_u32_prefix(payload) else {
            continue;
        };
        if payload.len() < prefix_len + payload_len || payload_len < 32 {
            continue;
        }
        let author = &payload[prefix_len..prefix_len + 32];
        return Some(format!("0x{}", hex::encode(author)));
    }
    None
}

fn block_fee_fen(block_hash: &str, extrinsics: &[Value]) -> u128 {
    let mut total_fee: u128 = 0;
    for xt in extrinsics {
        let Some(xt_hex) = xt.as_str() else {
            continue;
        };
        let params = Value::Array(vec![
            Value::String(xt_hex.to_string()),
            Value::String(block_hash.to_string()),
        ]);
        if let Ok(v) = rpc_post("payment_queryInfo", params) {
            if let Some(fee) = parse_partial_fee(&v) {
                total_fee = total_fee.saturating_add(fee);
            }
        }
    }
    total_fee
}

fn resource_usage(app: &AppHandle) -> ResourceUsage {
    let mut cpu_percent = None;
    let mut memory_mb = None;
    let mut disk_usage_percent = None;
    let mut node_data_size_mb = None;

    if let Ok(status) = current_status(app) {
        if let Some(pid) = status.pid {
            if let Ok(out) = Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "%cpu=,rss="])
                .output()
            {
                if out.status.success() {
                    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        cpu_percent = parts[0].parse::<f64>().ok();
                        memory_mb = parts[1]
                            .parse::<u64>()
                            .ok()
                            .map(|kb| kb.saturating_add(1023) / 1024);
                    }
                }
            }
        }
    }

    // 若无法获得托管进程 PID（例如节点由外部方式启动），回退到整机资源占用。
    if cpu_percent.is_none() || memory_mb.is_none() {
        #[cfg(target_os = "macos")]
        {
            if let Ok(out) = Command::new("top").args(["-l", "1", "-n", "0"]).output() {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        if cpu_percent.is_none() && line.contains("CPU usage:") {
                            if let Some(idle_idx) = line.find("% idle") {
                                let prefix = &line[..idle_idx];
                                let token = prefix
                                    .split_whitespace()
                                    .last()
                                    .unwrap_or("")
                                    .trim_end_matches('%');
                                if let Ok(idle) = token.parse::<f64>() {
                                    cpu_percent = Some((100.0 - idle).max(0.0));
                                }
                            }
                        }
                        if memory_mb.is_none() && line.contains("PhysMem:") && line.contains(" used") {
                            let used_token = line
                                .split("PhysMem:")
                                .nth(1)
                                .map(str::trim)
                                .and_then(|s| s.split_whitespace().next())
                                .unwrap_or("");
                            if !used_token.is_empty() {
                                let (num_str, unit) = used_token.split_at(used_token.len().saturating_sub(1));
                                if let Ok(n) = num_str.parse::<f64>() {
                                    memory_mb = match unit {
                                        "T" => Some((n * 1024.0 * 1024.0).round() as u64),
                                        "G" => Some((n * 1024.0).round() as u64),
                                        "M" => Some(n.round() as u64),
                                        "K" => Some((n / 1024.0).round() as u64),
                                        _ => None,
                                    };
                                }
                            }
                        }
                        if cpu_percent.is_some() && memory_mb.is_some() {
                            break;
                        }
                    }
                }
            }
        }
    }

    if let Ok(data_dir) = node_data_dir(app) {
        if let Ok(out) = Command::new("du").args(["-sk", &data_dir.display().to_string()]).output() {
            if out.status.success() {
                let line = String::from_utf8_lossy(&out.stdout);
                if let Some(first) = line.split_whitespace().next() {
                    node_data_size_mb = first
                        .parse::<u64>()
                        .ok()
                        .map(|kb| kb.saturating_add(1023) / 1024);
                }
            }
        }

        if let Ok(out) = Command::new("df").args(["-k", &data_dir.display().to_string()]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = text.lines().nth(1) {
                    for part in line.split_whitespace() {
                        if let Some(v) = part.strip_suffix('%') {
                            if let Ok(p) = v.parse::<f64>() {
                                disk_usage_percent = Some(p);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    ResourceUsage {
        cpu_percent,
        memory_mb,
        disk_usage_percent,
        node_data_size_mb,
    }
}

#[tauri::command]
pub fn get_node_status(app: AppHandle) -> Result<NodeStatus, String> {
    current_status(&app)
}

#[tauri::command]
pub fn start_node(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    let unlock_password = ensure_unlock_password(&unlock_password)?.to_string();
    verify_start_unlock_password(&unlock_password)?;
    let node_bin = find_node_bin()
        .ok_or_else(|| "未找到节点二进制（尝试过 ../target/debug/node, ../target/release/node）".to_string())?;

    {
        let app_state = app.state::<AppState>();
        let mut state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;
        if let Some(mut child) = state.local_node.take() {
            terminate_child(&mut child);
        }
        cleanup_node_key_runtime_file_in_state(&mut state);
    }

    kill_external_nodes();
    thread::sleep(Duration::from_millis(250));

    let (child, node_key_runtime_file) = spawn_node(&app, &node_bin, &unlock_password)?;
    {
        let app_state = app.state::<AppState>();
        let mut state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;
        state.local_node = Some(child);
        state.node_key_file = node_key_runtime_file;
    }

    thread::sleep(Duration::from_millis(800));
    current_status(&app)
}

#[tauri::command]
pub fn stop_node(app: AppHandle) -> Result<NodeStatus, String> {
    {
        let app_state = app.state::<AppState>();
        let mut state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;
        if let Some(mut child) = state.local_node.take() {
            terminate_child(&mut child);
        }
        cleanup_node_key_runtime_file_in_state(&mut state);
    }

    kill_external_nodes();
    thread::sleep(Duration::from_millis(250));
    current_status(&app)
}

#[tauri::command]
pub fn get_reward_wallet(app: AppHandle) -> Result<RewardWallet, String> {
    let path = reward_wallet_path(&app)?;
    if !path.exists() {
        return Ok(RewardWallet { address: None });
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("read reward wallet failed: {e}"))?;
    let stored: StoredWallet = serde_json::from_str(&raw).map_err(|e| format!("parse reward wallet failed: {e}"))?;
    Ok(RewardWallet {
        address: Some(stored.address),
    })
}

#[tauri::command]
pub fn set_reward_wallet(
    app: AppHandle,
    address: String,
    unlock_password: String,
) -> Result<RewardWallet, String> {
    let unlock = ensure_unlock_password(&unlock_password)?;
    verify_device_login_password(unlock)?;
    let normalized = normalize_wallet_address(&address)?;
    let raw = serde_json::to_string_pretty(&StoredWallet {
        address: normalized.clone(),
    })
    .map_err(|e| format!("encode reward wallet failed: {e}"))?;

    fs::write(reward_wallet_path(&app)?, format!("{raw}\n"))
        .map_err(|e| format!("write reward wallet failed: {e}"))?;

    Ok(RewardWallet {
        address: Some(normalized),
    })
}

#[tauri::command]
pub fn get_bootnode_key(app: AppHandle) -> Result<BootnodeKey, String> {
    if secure_store_get(KEYCHAIN_ACCOUNT_BOOTNODE)?.is_none() {
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
pub fn get_grandpa_key(app: AppHandle) -> Result<GrandpaKey, String> {
    if secure_store_get(KEYCHAIN_ACCOUNT_GRANDPA)?.is_none() {
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
pub fn set_bootnode_key(
    app: AppHandle,
    node_key: String,
    unlock_password: String,
) -> Result<BootnodeKey, String> {
    let unlock = ensure_unlock_password(&unlock_password)?;
    verify_device_login_password(unlock)?;
    let normalized = normalize_node_key(&node_key)?;
    let derived_peer_id = peer_id_from_node_key_hex(&normalized)?;
    if !is_genesis_bootnode_peer_id(&derived_peer_id) {
        return Err(format!(
            "该私钥不对应任何创世引导节点（推导 Peer ID: {derived_peer_id}）"
        ));
    }
    let institution_name = find_genesis_bootnode_name_by_peer_id(&derived_peer_id);

    let encrypted = encrypt_secret_value(&normalized, unlock)?;
    secure_store_set(KEYCHAIN_ACCOUNT_BOOTNODE, &encrypted)?;
    save_bootnode_meta(&app, &derived_peer_id, institution_name.clone())?;

    // 若节点当前在运行，保存新私钥后立即重启以应用新的 p2p 身份。
    if current_status(&app)?.running {
        let _ = stop_node(app.clone())?;
        let _ = start_node(app.clone(), unlock.to_string())?;
    }

    Ok(BootnodeKey {
        node_key: None,
        peer_id: Some(derived_peer_id),
        institution_name,
    })
}

#[tauri::command]
pub fn set_grandpa_key(app: AppHandle, key: String, unlock_password: String) -> Result<GrandpaKey, String> {
    let unlock = ensure_unlock_password(&unlock_password)?;
    verify_device_login_password(unlock)?;
    let normalized = normalize_grandpa_key(&key)?;
    let pubkey = grandpa_pubkey_from_private_hex(&normalized)?;
    let institution_name = institution_name_by_grandpa_pubkey(&pubkey).ok_or_else(|| {
        format!(
            "私钥与任何机构 GRANDPA 公钥不匹配（推导公钥: 0x{pubkey}）"
        )
    })?;
    let encrypted = encrypt_secret_value(&normalized, unlock)?;
    secure_store_set(KEYCHAIN_ACCOUNT_GRANDPA, &encrypted)?;
    save_grandpa_meta(&app, Some(institution_name.clone()))?;

    Ok(GrandpaKey {
        key: None,
        institution_name: Some(institution_name),
    })
}

#[tauri::command]
pub fn get_genesis_bootnode_options() -> Result<Vec<GenesisBootnodeOption>, String> {
    let options = genesis_bootnode_options();
    if options.is_empty() {
        return Err("未从链配置中解析到创世引导节点".to_string());
    }
    Ok(options)
}

#[tauri::command]
pub fn set_node_name(app: AppHandle, node_name: String) -> Result<NodeIdentity, String> {
    let normalized = normalize_node_name(&node_name)?;
    let raw = serde_json::to_string_pretty(&StoredNodeName {
        node_name: normalized.clone(),
    })
    .map_err(|e| format!("encode node-name failed: {e}"))?;

    fs::write(node_name_path(&app)?, format!("{raw}\n"))
        .map_err(|e| format!("write node-name failed: {e}"))?;

    let peer_id = rpc_post("system_localPeerId", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let role = role_from_peer_id(peer_id.as_deref());

    Ok(NodeIdentity {
        node_name: Some(normalized),
        peer_id,
        role: Some(role),
    })
}

#[tauri::command]
pub fn get_chain_status(_app: AppHandle) -> Result<ChainStatus, String> {
    // 区块高度
    let header = match rpc_post("chain_getHeader", Value::Array(vec![])) {
        Ok(v) => v,
        Err(_) => return Ok(ChainStatus { block_height: None }),
    };
    let block_height = header
        .get("number")
        .and_then(Value::as_str)
        .and_then(hex_to_u64);

    Ok(ChainStatus { block_height })
}

#[tauri::command]
pub fn get_node_identity(app: AppHandle) -> Result<NodeIdentity, String> {
    let rpc_node_name = rpc_post("system_name", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let configured_node_name = load_node_name(&app)?;
    let node_name = configured_node_name.or(rpc_node_name);

    let local_peer_id = rpc_post("system_localPeerId", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let role = role_from_peer_id(local_peer_id.as_deref());

    Ok(NodeIdentity {
        node_name,
        peer_id: local_peer_id,
        role: Some(role),
    })
}

#[tauri::command]
pub fn get_mining_dashboard(app: AppHandle) -> Result<MiningDashboard, String> {
    let header = match rpc_post("chain_getHeader", Value::Array(vec![])) {
        Ok(v) => v,
        Err(_) => {
            return Ok(MiningDashboard {
                income: MiningIncome {
                    total_income: "0.00".to_string(),
                    total_fee_income: "0.00".to_string(),
                    total_reward_income: "0.00".to_string(),
                    today_income: "0.00".to_string(),
                },
                records: vec![],
                resources: resource_usage(&app),
            })
        }
    };

    let best_height = header
        .get("number")
        .and_then(Value::as_str)
        .and_then(hex_to_u64)
        .unwrap_or(0);

    let mut total_fee_fen: u128 = 0;
    let mut today_income_fen: u128 = 0;
    let mut total_reward_fen: u128 = 0;
    let mut time_cache: HashMap<u64, Option<u64>> = HashMap::new();

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("system time error: {e}"))?
        .as_millis() as u64;
    let today_utc = now_ms / 86_400_000;

    let mut records: Vec<MiningBlockRecord> = Vec::new();
    let records_from = best_height.saturating_sub(19);
    let ts_key = timestamp_now_storage_key();

    for n in 1..=best_height {
        let block_hash = match rpc_post(
            "chain_getBlockHash",
            Value::Array(vec![Value::String(format!("0x{n:x}"))]),
        ) {
            Ok(v) => v.as_str().map(|s| s.to_string()),
            Err(_) => None,
        };
        let Some(block_hash) = block_hash else {
            continue;
        };

        let block = match rpc_post(
            "chain_getBlock",
            Value::Array(vec![Value::String(block_hash.clone())]),
        ) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let extrinsics = block
            .get("block")
            .and_then(|b| b.get("extrinsics"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let block_fee = block_fee_fen(&block_hash, &extrinsics);
        total_fee_fen = total_fee_fen.saturating_add(block_fee);

        let ts = if let Some(v) = time_cache.get(&n) {
            *v
        } else {
            let t = rpc_post(
                "state_getStorage",
                Value::Array(vec![
                    Value::String(ts_key.clone()),
                    Value::String(block_hash.clone()),
                ]),
            )
            .ok()
            .and_then(|v| v.as_str().and_then(scale_u64_from_storage_hex));
            time_cache.insert(n, t);
            t
        };

        let block_reward = block_reward_fen_by_height(n);
        total_reward_fen = total_reward_fen.saturating_add(block_reward);

        if let Some(ms) = ts {
            if ms / 86_400_000 == today_utc {
                today_income_fen = today_income_fen
                    .saturating_add(block_fee)
                    .saturating_add(block_reward);
            }
        }

        if n >= records_from {
            let logs = block
                .get("block")
                .and_then(|b| b.get("header"))
                .and_then(|h| h.get("digest"))
                .and_then(|d| d.get("logs"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();

            let author = author_from_pow_digest_logs(&logs).unwrap_or_else(|| "未知".to_string());
            records.push(MiningBlockRecord {
                block_height: n,
                timestamp_ms: ts,
                fee: format_2_decimals_fen(block_fee),
                block_reward: format_2_decimals_fen(block_reward),
                author,
            });
        }
    }

    records.sort_by(|a, b| b.block_height.cmp(&a.block_height));

    let total_income_fen = total_fee_fen.saturating_add(total_reward_fen);

    Ok(MiningDashboard {
        income: MiningIncome {
            total_income: format_2_decimals_fen(total_income_fen),
            total_fee_income: format_2_decimals_fen(total_fee_fen),
            total_reward_income: format_2_decimals_fen(total_reward_fen),
            today_income: format_2_decimals_fen(today_income_fen),
        },
        records,
        resources: resource_usage(&app),
    })
}

#[tauri::command]
pub fn get_network_overview(app: AppHandle) -> Result<NetworkOverview, String> {
    let bootnodes = genesis_bootnode_options();
    let genesis_peer_ids: HashSet<String> = bootnodes.iter().map(|n| n.peer_id.clone()).collect();
    let mut known_peer_ids = load_known_peers(&app)?;
    let mut online_peer_ids: HashSet<String> = HashSet::new();
    let mut light_nodes: u64 = 0;

    if let Ok(peers) = rpc_post("system_peers", Value::Array(vec![])) {
        if let Some(arr) = peers.as_array() {
            for p in arr {
                if let Some(pid) = p.get("peerId").and_then(Value::as_str) {
                    online_peer_ids.insert(pid.to_string());
                    known_peer_ids.insert(pid.to_string());
                }

                // Substrate 的 roles 可能是字符串或字符串数组，这里都兼容。
                let is_light = p
                    .get("roles")
                    .map(|r| {
                        if let Some(s) = r.as_str() {
                            s.to_ascii_lowercase().contains("light")
                        } else if let Some(list) = r.as_array() {
                            list.iter().any(|v| {
                                v.as_str()
                                    .map(|s| s.to_ascii_lowercase().contains("light"))
                                    .unwrap_or(false)
                            })
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);
                if is_light {
                    light_nodes = light_nodes.saturating_add(1);
                }
            }
        }
    }

    let status = current_status(&app)?;
    let mut online_nodes = online_peer_ids.len() as u64;
    if status.running {
        online_nodes = online_nodes.saturating_add(1);
        if let Ok(local_peer_id) = rpc_post("system_localPeerId", Value::Array(vec![])) {
            if let Some(pid) = local_peer_id.as_str() {
                online_peer_ids.insert(pid.to_string());
                known_peer_ids.insert(pid.to_string());
            }
        }
    }
    save_known_peers(&app, &known_peer_ids)?;

    let mut guochuhui_nodes = 0u64;
    let mut shengchuhui_nodes = 0u64;
    let mut shengchuhang_nodes = 0u64;
    for pid in &online_peer_ids {
        if let Some(name) = find_genesis_bootnode_name_by_peer_id(pid) {
            if name.contains("国储会") {
                guochuhui_nodes = guochuhui_nodes.saturating_add(1);
            } else if name.contains("储行") {
                shengchuhang_nodes = shengchuhang_nodes.saturating_add(1);
            } else {
                shengchuhui_nodes = shengchuhui_nodes.saturating_add(1);
            }
        }
    }

    let full_nodes = online_nodes.saturating_sub(light_nodes);
    let known_non_genesis = known_peer_ids
        .iter()
        .filter(|pid| !genesis_peer_ids.contains(*pid))
        .count() as u64;
    let total_nodes = (bootnodes.len() as u64).saturating_add(known_non_genesis);

    Ok(NetworkOverview {
        total_nodes,
        online_nodes,
        guochuhui_nodes,
        shengchuhui_nodes,
        shengchuhang_nodes,
        full_nodes,
        light_nodes,
    })
}
