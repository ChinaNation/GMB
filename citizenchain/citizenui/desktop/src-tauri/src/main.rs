use base64::Engine;
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey as Ed25519PublicKey};
use hex::FromHex;
use image::Luma;
use qrcode::QrCode;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use serde::Serialize;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    env, fs,
    hash::{Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use subxt::{config::HashFor, dynamic::Value, OnlineClient, PolkadotConfig};
use tauri::{AppHandle, Manager, RunEvent};

struct BackgroundProcessState {
    local_node: Option<Child>,
    registry_sync_stop: Option<Arc<AtomicBool>>,
    registry_sync_thread: Option<JoinHandle<()>>,
}

struct AppProcessState(Mutex<BackgroundProcessState>);

#[derive(Clone)]
struct InstitutionMeta {
    role: &'static str,
    organization_name: String,
    province: Option<String>,
}

#[derive(Clone, Debug, Serialize, Hash)]
struct OrgRegistryRecord {
    role: String,
    #[serde(rename = "organizationName")]
    organization_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    province: Option<String>,
    #[serde(rename = "adminAddress")]
    admin_address: String,
}

#[derive(Debug, Serialize)]
struct OrgRegistrySnapshot {
    version: u64,
    generated_at: u64,
    records: Vec<OrgRegistryRecord>,
}

#[tauri::command]
fn generate_qr_data_url(payload: String) -> Result<String, String> {
    let code = QrCode::new(payload.as_bytes()).map_err(|e| e.to_string())?;
    let image = code.render::<Luma<u8>>().min_dimensions(220, 220).build();

    let mut buffer: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(buffer);
    Ok(format!("data:image/png;base64,{}", b64))
}

fn decode_hex_32(label: &str, value: &str) -> Result<[u8; 32], String> {
    let text = value.trim().trim_start_matches("0x");
    let bytes = <Vec<u8>>::from_hex(text).map_err(|_| format!("invalid {} hex", label))?;
    if bytes.len() != 32 {
        return Err(format!("{} must be 32 bytes", label));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_signature(value: &str) -> Result<Vec<u8>, String> {
    let text = value.trim();
    if text.is_empty() {
        return Err("signature is empty".into());
    }

    if text.starts_with("0x") {
        let raw = <Vec<u8>>::from_hex(text.trim_start_matches("0x"))
            .map_err(|_| "invalid signature hex".to_string())?;
        return Ok(raw);
    }

    if text.chars().all(|c| c.is_ascii_hexdigit()) && text.len() % 2 == 0 {
        let raw = <Vec<u8>>::from_hex(text).map_err(|_| "invalid signature hex".to_string())?;
        return Ok(raw);
    }

    base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|_| "invalid signature encoding".into())
}

fn admin_registry_snapshot_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))?;
    fs::create_dir_all(&app_data).map_err(|e| format!("create app data dir failed: {e}"))?;
    Ok(app_data.join("org-registry.snapshot.json"))
}

#[tauri::command]
fn read_org_registry_snapshot_json(app: AppHandle) -> Result<Option<String>, String> {
    let snapshot = admin_registry_snapshot_path(&app)?;
    if !snapshot.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&snapshot).map_err(|e| format!("read snapshot failed: {e}"))?;
    Ok(Some(raw))
}

fn normalize_org_name(raw: &str) -> String {
    raw.replace("公民", "")
        .replace("权威节点", "")
        .replace("权益节点", "")
        .replace("  ", " ")
        .trim()
        .to_string()
}

fn derive_province(name: &str) -> Option<String> {
    name.find('省')
        .map(|idx| name[..idx].trim().to_string())
        .filter(|s| !s.is_empty())
}

fn shenfen_id_to_fixed48(shenfen_id: &str) -> Option<[u8; 48]> {
    let raw = shenfen_id.as_bytes();
    if raw.is_empty() || raw.len() > 48 {
        return None;
    }
    let mut out = [0u8; 48];
    out[..raw.len()].copy_from_slice(raw);
    Some(out)
}

fn parse_quoted_field(line: &str, field: &str) -> Option<String> {
    let marker = format!("{field}:");
    let idx = line.find(&marker)?;
    let rest = &line[idx + marker.len()..];
    let start = rest.find('"')?;
    let rem = &rest[start + 1..];
    let end = rem.find('"')?;
    Some(rem[..end].to_string())
}

fn parse_china_nodes(content: &str) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    let mut shenfen_id: Option<String> = None;
    let mut shenfen_name: Option<String> = None;

    for line in content.lines() {
        if shenfen_id.is_none() {
            shenfen_id = parse_quoted_field(line, "shenfen_id");
        }
        if shenfen_name.is_none() {
            shenfen_name = parse_quoted_field(line, "shenfen_name");
        }
        if line.trim_start().starts_with("},") {
            if let (Some(id), Some(name)) = (shenfen_id.take(), shenfen_name.take()) {
                rows.push((id, name));
            } else {
                shenfen_id = None;
                shenfen_name = None;
            }
        }
    }

    rows
}

fn load_china_source(path: &Path, anchor: &str) -> Result<String, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    let section = raw
        .split(anchor)
        .nth(1)
        .ok_or_else(|| format!("anchor {anchor} not found in {}", path.display()))?;
    Ok(section.to_string())
}

fn build_institution_index() -> HashMap<[u8; 48], InstitutionMeta> {
    let mut map = HashMap::new();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .join("../../../../")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../../../../"));
    let china_dir = workspace_root.join("primitives/china");
    let reserve_file = china_dir.join("china_cb.rs");
    let bank_file = china_dir.join("china_ch.rs");

    let reserve_section = match load_china_source(&reserve_file, "pub const CHINA_CB") {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return map;
        }
    };
    let bank_section = match load_china_source(&bank_file, "pub const CHINA_CH") {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            return map;
        }
    };

    for (shenfen_id, shenfen_name) in parse_china_nodes(&reserve_section) {
        let Some(key) = shenfen_id_to_fixed48(&shenfen_id) else {
            continue;
        };
        let is_nrc = shenfen_name.contains("国家储备委员会") || shenfen_name.contains("国储");
        map.insert(
            key,
            InstitutionMeta {
                role: if is_nrc { "nrc" } else { "prc" },
                organization_name: normalize_org_name(&shenfen_name),
                province: if is_nrc {
                    None
                } else {
                    derive_province(&shenfen_name)
                },
            },
        );
    }

    for (shenfen_id, shenfen_name) in parse_china_nodes(&bank_section) {
        let Some(key) = shenfen_id_to_fixed48(&shenfen_id) else {
            continue;
        };
        map.insert(
            key,
            InstitutionMeta {
                role: "prb",
                organization_name: normalize_org_name(&shenfen_name),
                province: derive_province(&shenfen_name),
            },
        );
    }

    map
}

fn decode_institution_from_storage_key(key_bytes: &[u8]) -> Option<[u8; 48]> {
    if key_bytes.len() < 48 {
        return None;
    }
    let mut out = [0u8; 48];
    out.copy_from_slice(&key_bytes[key_bytes.len() - 48..]);
    Some(out)
}

fn digest_records(records: &[OrgRegistryRecord]) -> u64 {
    let mut hasher = DefaultHasher::new();
    records.hash(&mut hasher);
    hasher.finish()
}

fn write_registry_snapshot(path: &Path, rows: Vec<OrgRegistryRecord>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create snapshot dir failed: {e}"))?;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("clock error: {e}"))?
        .as_secs();
    let snapshot = OrgRegistrySnapshot {
        version: now,
        generated_at: now,
        records: rows,
    };
    let json =
        serde_json::to_string_pretty(&snapshot).map_err(|e| format!("snapshot encode failed: {e}"))?;
    fs::write(path, format!("{json}\n")).map_err(|e| format!("write snapshot failed: {e}"))?;
    Ok(())
}

async fn fetch_registry_rows(
    api: &OnlineClient<PolkadotConfig>,
    institution_index: &HashMap<[u8; 48], InstitutionMeta>,
    at_block: Option<HashFor<PolkadotConfig>>,
) -> Result<Vec<OrgRegistryRecord>, String> {
    let keys: Vec<Value> = vec![];
    let storage_query = subxt::dynamic::storage("AdminsOriginGov", "CurrentAdmins", keys);
    let storage = if let Some(hash) = at_block {
        api.storage().at(hash)
    } else {
        api.storage()
            .at_latest()
            .await
            .map_err(|e| format!("storage at_latest failed: {e}"))?
    };
    let mut iter = storage
        .iter(storage_query)
        .await
        .map_err(|e| format!("storage iter failed: {e}"))?;

    let mut rows = Vec::new();
    while let Some(item) = iter.next().await {
        let kv = item.map_err(|e| format!("storage item decode failed: {e}"))?;
        let Some(institution) = decode_institution_from_storage_key(&kv.key_bytes) else {
            continue;
        };
        let Some(meta) = institution_index.get(&institution) else {
            continue;
        };
        let admins: Vec<[u8; 32]> = kv
            .value
            .as_type()
            .map_err(|e| format!("decode admin list via metadata failed: {e}"))?;
        for admin in admins {
            rows.push(OrgRegistryRecord {
                role: meta.role.to_string(),
                organization_name: meta.organization_name.clone(),
                province: meta.province.clone(),
                admin_address: format!("0x{}", hex::encode(admin)),
            });
        }
    }

    rows.sort_by(|a, b| {
        let r = a.role.cmp(&b.role);
        if r != std::cmp::Ordering::Equal {
            return r;
        }
        let n = a.organization_name.cmp(&b.organization_name);
        if n != std::cmp::Ordering::Equal {
            return n;
        }
        a.admin_address.cmp(&b.admin_address)
    });
    Ok(rows)
}

fn spawn_admin_registry_sync(snapshot_path: PathBuf) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&stop);
    let ws_endpoint = env::var("ADMIN_REGISTRY_WS").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string());
    let handle = thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(err) => {
                eprintln!("build registry runtime failed: {err}");
                return;
            }
        };
        runtime.block_on(async move {
            let institution_index = build_institution_index();
            let mut last_digest: Option<u64> = None;
            while !stop_flag.load(Ordering::Relaxed) {
                match OnlineClient::<PolkadotConfig>::from_url(ws_endpoint.as_str()).await {
                    Ok(api) => {
                        match fetch_registry_rows(&api, &institution_index, None).await {
                            Ok(rows) => {
                                let digest = digest_records(&rows);
                                if last_digest != Some(digest) {
                                    if let Err(err) = write_registry_snapshot(&snapshot_path, rows) {
                                        eprintln!("write admin registry snapshot failed: {err}");
                                    } else {
                                        last_digest = Some(digest);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("fetch admin registry failed: {err}");
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                continue;
                            }
                        }

                        let mut finalized = match api.blocks().subscribe_finalized().await {
                            Ok(stream) => stream,
                            Err(err) => {
                                eprintln!("subscribe finalized blocks failed: {err}");
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                continue;
                            }
                        };

                        while !stop_flag.load(Ordering::Relaxed) {
                            tokio::select! {
                                _ = tokio::time::sleep(Duration::from_millis(250)) => {}
                                maybe_block = finalized.next() => {
                                    let Some(block_result) = maybe_block else {
                                        eprintln!("finalized blocks stream ended");
                                        break;
                                    };
                                    let block = match block_result {
                                        Ok(block) => block,
                                        Err(err) => {
                                            eprintln!("read finalized block failed: {err}");
                                            break;
                                        }
                                    };

                                    match fetch_registry_rows(&api, &institution_index, Some(block.hash())).await {
                                        Ok(rows) => {
                                            let digest = digest_records(&rows);
                                            if last_digest != Some(digest) {
                                                if let Err(err) = write_registry_snapshot(&snapshot_path, rows) {
                                                    eprintln!("write admin registry snapshot failed: {err}");
                                                } else {
                                                    last_digest = Some(digest);
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            eprintln!("fetch admin registry failed: {err}");
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("connect admin registry endpoint failed: {err}");
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    });
    (stop, handle)
}

fn normalize_signature(raw: Vec<u8>) -> Vec<u8> {
    if raw.len() == 65 {
        raw[1..].to_vec()
    } else {
        raw
    }
}

fn verify_sr25519(public_key: [u8; 32], signature: &[u8], message: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }

    let Ok(public) = Sr25519PublicKey::from_bytes(&public_key) else {
        return false;
    };
    let Ok(signature_bytes) = <[u8; 64]>::try_from(signature) else {
        return false;
    };
    let Ok(sig) = Sr25519Signature::from_bytes(&signature_bytes) else {
        return false;
    };
    public
        .verify(signing_context(b"substrate").bytes(message), &sig)
        .is_ok()
}

fn verify_ed25519(public_key: [u8; 32], signature: &[u8], message: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }
    let Ok(public) = Ed25519PublicKey::from_bytes(&public_key) else {
        return false;
    };
    let Ok(sig) = Ed25519Signature::from_slice(signature) else {
        return false;
    };
    public.verify(message, &sig).is_ok()
}

#[tauri::command]
fn verify_login_signature(
    payload: String,
    signature: String,
    public_key: String,
    crypto: Option<String>,
) -> Result<bool, String> {
    let message = payload.as_bytes();
    let public_key = decode_hex_32("public_key", &public_key)?;
    let signature = normalize_signature(decode_signature(&signature)?);

    let verified = match crypto.as_deref() {
        Some("sr25519") => verify_sr25519(public_key, &signature, message),
        Some("ed25519") => verify_ed25519(public_key, &signature, message),
        _ => {
            // Auto mode: try sr25519 first, then ed25519.
            verify_sr25519(public_key, &signature, message)
                || verify_ed25519(public_key, &signature, message)
        }
    };

    Ok(verified)
}

fn candidate_node_bins(app: &AppHandle) -> Vec<PathBuf> {
    let mut bins = Vec::new();
    if let Ok(path) = env::var("CITIZENCHAIN_NODE_BIN") {
        if !path.trim().is_empty() {
            bins.push(PathBuf::from(path));
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bins.push(manifest_dir.join("../../../target/debug/citizenchain-node"));
    bins.push(manifest_dir.join("../../../target/debug/node"));
    bins.push(manifest_dir.join("../../../target/release/citizenchain-node"));
    bins.push(manifest_dir.join("../../../target/release/node"));

    if let Ok(resource_dir) = app.path().resource_dir() {
        bins.push(resource_dir.join("citizenchain-node"));
        bins.push(resource_dir.join("citizenchain-node.exe"));
        bins.push(resource_dir.join("node"));
        bins.push(resource_dir.join("node.exe"));
        bins.push(resource_dir.join("binaries/citizenchain-node"));
        bins.push(resource_dir.join("binaries/citizenchain-node.exe"));
    }

    bins
}

fn spawn_local_node(app: &AppHandle) -> Result<Child, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))?;
    let node_data = app_data.join("node-data");
    fs::create_dir_all(&node_data).map_err(|e| format!("create node data dir failed: {e}"))?;

    for bin in candidate_node_bins(app) {
        if !bin.exists() {
            continue;
        }
        let mut cmd = Command::new(&bin);
        cmd.arg("--base-path")
            .arg(node_data.as_os_str())
            .arg("--rpc-port")
            .arg("9944");
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
        if let Ok(child) = cmd.spawn() {
            return Ok(child);
        }
    }

    Err("cannot start local node; set CITIZENCHAIN_NODE_BIN to node executable path".into())
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
    let _ = child.wait();
}

fn main() {
    let app = tauri::Builder::default()
        .manage(AppProcessState(Mutex::new(BackgroundProcessState {
            local_node: None,
            registry_sync_stop: None,
            registry_sync_thread: None,
        })))
        .setup(|app| {
            match spawn_local_node(app.handle()) {
                Ok(child) => {
                    if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
                        state.local_node = Some(child);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                }
            }
            match admin_registry_snapshot_path(&app.handle()) {
                Ok(snapshot_path) => {
                    let (stop, handle) = spawn_admin_registry_sync(snapshot_path);
                    if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
                        state.registry_sync_stop = Some(stop);
                        state.registry_sync_thread = Some(handle);
                    }
                }
                Err(err) => eprintln!("{err}"),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_qr_data_url,
            verify_login_signature,
            read_org_registry_snapshot_json
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let RunEvent::Exit = event {
            if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
                if let Some(stop) = state.registry_sync_stop.take() {
                    stop.store(true, Ordering::Relaxed);
                }
                if let Some(handle) = state.registry_sync_thread.take() {
                    let _ = handle.join();
                }
                if let Some(mut child) = state.local_node.take() {
                    terminate_child(&mut child);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{decode_hex_32, normalize_signature};

    #[test]
    fn decode_hex_32_accepts_prefixed_hex() {
        let key = decode_hex_32(
            "public_key",
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("hex should decode");
        assert_eq!(key[0], 0x01);
        assert_eq!(key[31], 0xef);
    }

    #[test]
    fn decode_hex_32_rejects_wrong_length() {
        assert!(decode_hex_32("public_key", "0x1234").is_err());
    }

    #[test]
    fn normalize_signature_strips_multisig_prefix() {
        let normalized = normalize_signature(vec![0, 1, 2, 3]);
        assert_eq!(normalized, vec![0, 1, 2, 3]);

        let mut prefixed = vec![1];
        prefixed.extend_from_slice(&[9; 64]);
        let normalized_prefixed = normalize_signature(prefixed);
        assert_eq!(normalized_prefixed.len(), 64);
        assert_eq!(normalized_prefixed[0], 9);
    }
}
