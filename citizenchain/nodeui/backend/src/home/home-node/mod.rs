use crate::{
    rpc,
    settings::{bootnodes_address, fee_address, grandpa_address, security},
    validation::normalize_node_name,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const EXPECTED_SS58_PREFIX: u64 = 2027;
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_RPC_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const RPC_RETRY_COUNT: usize = 3;
const NODE_BIN_BASENAME: &str = "citizenchain-node";
const RUNTIME_SECRETS_DIR_NAME: &str = "runtime-secrets";
const NODE_KEY_TEMP_PREFIX: &str = "node-key-";
const NODE_KEY_TEMP_SUFFIX: &str = ".tmp";
const NODE_BIN_STAGE_PREFIX: &str = "node-bin-";

pub struct RuntimeState {
    pub local_node: Option<Child>,
    pub node_key_file: Option<PathBuf>,
    pub node_bin_file: Option<PathBuf>,
}

pub struct AppState(pub Mutex<RuntimeState>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub running: bool,
    pub state: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainStatus {
    pub block_height: Option<u64>,
    pub finalized_height: Option<u64>,
    pub syncing: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeIdentity {
    pub node_name: Option<String>,
    pub peer_id: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredNodeName {
    node_name: String,
}

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data = security::app_data_dir(app)?.join("node-data");
    fs::create_dir_all(&data).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(data)
}

fn runtime_secrets_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let secrets = security::app_data_dir(app)?.join(RUNTIME_SECRETS_DIR_NAME);
    fs::create_dir_all(&secrets).map_err(|e| format!("create runtime secrets dir failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&secrets, fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("set runtime secrets dir permission failed: {e}"))?;
    }
    Ok(secrets)
}

fn remove_node_key_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "remove node-key temp file failed ({}): {e}",
            path.display()
        )),
    }
}

fn clear_runtime_node_key_file(state: &mut RuntimeState) {
    if let Some(path) = state.node_key_file.take() {
        if let Err(err) = remove_node_key_file(&path) {
            eprintln!("{err}");
        }
    }
}

fn cleanup_stale_node_key_temp_files(app: &AppHandle) -> Result<(), String> {
    let secrets_dir = runtime_secrets_dir(app)?;
    let entries = fs::read_dir(&secrets_dir).map_err(|e| {
        format!(
            "read runtime secrets dir failed ({}): {e}",
            secrets_dir.display()
        )
    })?;
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name.starts_with(NODE_KEY_TEMP_PREFIX) && name.ends_with(NODE_KEY_TEMP_SUFFIX) {
            if let Err(err) = remove_node_key_file(&path) {
                eprintln!("{err}");
            }
        }
    }
    Ok(())
}

fn write_node_key_temp_file(app: &AppHandle, node_key: &str) -> Result<PathBuf, String> {
    let secrets_dir = runtime_secrets_dir(app)?;
    let pid = std::process::id();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_nanos())
        .unwrap_or(0);
    for seq in 0..8u8 {
        let path = secrets_dir.join(format!(
            "{NODE_KEY_TEMP_PREFIX}{pid}-{stamp}-{seq}{NODE_KEY_TEMP_SUFFIX}"
        ));
        if path.exists() {
            continue;
        }
        security::write_secret_text_atomic(&path, node_key)?;
        return Ok(path);
    }
    Err("create node-key temp file failed: exhausted retries".to_string())
}

fn remove_staged_node_bin_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "remove staged node binary failed ({}): {e}",
            path.display()
        )),
    }
}

fn clear_runtime_node_bin_file(state: &mut RuntimeState) {
    if let Some(path) = state.node_bin_file.take() {
        if let Err(err) = remove_staged_node_bin_file(&path) {
            eprintln!("{err}");
        }
    }
}

fn cleanup_stale_staged_node_bins(app: &AppHandle, keep: Option<&Path>) -> Result<(), String> {
    let secrets_dir = runtime_secrets_dir(app)?;
    let entries = fs::read_dir(&secrets_dir).map_err(|e| {
        format!(
            "read runtime secrets dir failed ({}): {e}",
            secrets_dir.display()
        )
    })?;
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if keep.is_some_and(|p| p == path.as_path()) {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name.starts_with(NODE_BIN_STAGE_PREFIX) {
            if let Err(err) = remove_staged_node_bin_file(&path) {
                eprintln!("{err}");
            }
        }
    }
    Ok(())
}

fn node_name_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("node-name.json"))
}

fn role_from_peer_id(peer_id: Option<&str>) -> String {
    if let Some(pid) = peer_id {
        if let Ok(Some(name)) = bootnodes_address::find_genesis_bootnode_name_by_peer_id(pid) {
            return name;
        }
    }
    "全节点".to_string()
}

fn refresh_managed_process(state: &mut RuntimeState) -> (bool, Option<u32>) {
    if let Some(child) = state.local_node.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => {
                state.local_node = None;
                clear_runtime_node_key_file(state);
                clear_runtime_node_bin_file(state);
                (false, None)
            }
            Ok(None) => (true, Some(child.id())),
        }
    } else {
        (false, None)
    }
}

fn node_bin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries")
}

fn node_bin_filename_candidates() -> Vec<String> {
    let mut names = vec![NODE_BIN_BASENAME.to_string()];

    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            names.push(format!("{NODE_BIN_BASENAME}-aarch64-apple-darwin"));
        }
        if cfg!(target_arch = "x86_64") {
            names.push(format!("{NODE_BIN_BASENAME}-x86_64-apple-darwin"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if cfg!(target_arch = "x86_64") {
            names.push(format!("{NODE_BIN_BASENAME}-x86_64-unknown-linux-gnu"));
        }
        if cfg!(target_arch = "aarch64") {
            names.push(format!("{NODE_BIN_BASENAME}-aarch64-unknown-linux-gnu"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        names.push(format!("{NODE_BIN_BASENAME}.exe"));
        if cfg!(target_arch = "x86_64") {
            names.push(format!("{NODE_BIN_BASENAME}-x86_64-pc-windows-msvc.exe"));
        }
        if cfg!(target_arch = "aarch64") {
            names.push(format!("{NODE_BIN_BASENAME}-aarch64-pc-windows-msvc.exe"));
        }
    }

    names
}

fn node_bin_candidate_paths(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![node_bin_dir()];
    if let Ok(resource_dir) = app.path().resource_dir() {
        dirs.push(resource_dir);
    }

    let mut paths = Vec::new();
    for dir in dirs {
        for name in node_bin_filename_candidates() {
            let path = dir.join(&name);
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    paths
}

fn node_bin_hash_candidates(node_bin: &Path) -> Result<Vec<PathBuf>, String> {
    let file_name = node_bin
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| {
            format!(
                "resolve node binary filename failed ({})",
                node_bin.display()
            )
        })?;
    let mut paths = vec![node_bin.with_file_name(format!("{file_name}.sha256"))];
    if file_name != NODE_BIN_BASENAME {
        paths.push(node_bin.with_file_name(format!("{NODE_BIN_BASENAME}.sha256")));
    }
    Ok(paths)
}

fn trusted_node_bin_dirs(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    if let Ok(node_dir) = node_bin_dir().canonicalize() {
        dirs.push(node_dir);
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        if let Ok(canonical_resource_dir) = resource_dir.canonicalize() {
            dirs.push(canonical_resource_dir);
        }
    }
    if dirs.is_empty() {
        return Err("resolve trusted node binary dirs failed".to_string());
    }
    Ok(dirs)
}

fn parse_sha256_hex(raw: &str) -> Result<String, String> {
    let value = raw
        .split_whitespace()
        .next()
        .ok_or_else(|| "parse sha256 file failed: empty content".to_string())?
        .to_ascii_lowercase();
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("parse sha256 file failed: invalid hex format".to_string());
    }
    Ok(value)
}

fn file_sha256_hex(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("open file for sha256 failed: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 16 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("read file for sha256 failed: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn resolve_node_bin_hash_path(node_bin: &Path) -> Result<PathBuf, String> {
    let hash_paths = node_bin_hash_candidates(node_bin)?;
    hash_paths
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .ok_or_else(|| {
            let expected_list = hash_paths
                .iter()
                .map(|v| v.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("node binary hash file missing: [{expected_list}]")
        })
}

fn staged_node_bin_path(app: &AppHandle, source_bin: &Path) -> Result<PathBuf, String> {
    let secrets_dir = runtime_secrets_dir(app)?;
    let pid = std::process::id();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_nanos())
        .unwrap_or(0);
    let ext = source_bin
        .extension()
        .and_then(|v| v.to_str())
        .filter(|v| !v.is_empty())
        .map(|v| format!(".{v}"))
        .unwrap_or_default();
    for seq in 0..8u8 {
        let path = secrets_dir.join(format!("{NODE_BIN_STAGE_PREFIX}{pid}-{stamp}-{seq}{ext}"));
        if !path.exists() {
            return Ok(path);
        }
    }
    Err("create staged node binary path failed: exhausted retries".to_string())
}

fn stage_verified_node_bin(app: &AppHandle) -> Result<PathBuf, String> {
    let source_bin = find_node_bin_source(app)?;
    let hash_path = resolve_node_bin_hash_path(&source_bin)?;
    let expected_raw = fs::read_to_string(&hash_path).map_err(|e| {
        format!(
            "read node binary hash failed ({}): {e}",
            hash_path.display()
        )
    })?;
    let expected = parse_sha256_hex(&expected_raw)?;
    let staged_bin = staged_node_bin_path(app, &source_bin)?;
    if let Err(e) = fs::copy(&source_bin, &staged_bin) {
        let _ = remove_staged_node_bin_file(&staged_bin);
        return Err(format!(
            "copy staged node binary failed ({} -> {}): {e}",
            source_bin.display(),
            staged_bin.display()
        ));
    }
    if let Err(e) = fs::set_permissions(
        &staged_bin,
        fs::metadata(&source_bin)
            .map_err(|err| {
                format!(
                    "read source node binary metadata failed ({}): {err}",
                    source_bin.display()
                )
            })?
            .permissions(),
    ) {
        let _ = remove_staged_node_bin_file(&staged_bin);
        return Err(format!(
            "set staged node binary permissions failed ({}): {e}",
            staged_bin.display()
        ));
    }
    let actual = match file_sha256_hex(&staged_bin) {
        Ok(v) => v,
        Err(err) => {
            let _ = remove_staged_node_bin_file(&staged_bin);
            return Err(err);
        }
    };
    if actual != expected {
        let _ = remove_staged_node_bin_file(&staged_bin);
        return Err(format!(
            "staged node binary sha256 mismatch (bin={}, hash_file={}, staged={})",
            source_bin.display(),
            hash_path.display(),
            staged_bin.display()
        ));
    }
    Ok(staged_bin)
}

fn find_node_bin_source(app: &AppHandle) -> Result<PathBuf, String> {
    let candidates = node_bin_candidate_paths(app);
    let trusted_dirs = trusted_node_bin_dirs(app)?;
    let existing = candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            format!(
                "node binary not found. searched: {}",
                node_bin_candidate_paths(app)
                    .iter()
                    .map(|v| v.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    let canonical_bin = existing
        .canonicalize()
        .map_err(|e| format!("canonicalize node binary failed: {e}"))?;
    if !trusted_dirs
        .iter()
        .any(|dir| canonical_bin.starts_with(dir))
    {
        return Err(format!(
            "node binary is outside trusted dirs: {}",
            canonical_bin.display()
        ));
    }
    Ok(canonical_bin)
}

fn verify_start_unlock_password(app: &AppHandle, unlock_password: &str) -> Result<(), String> {
    let unlock = security::ensure_unlock_password(unlock_password)?;
    security::verify_device_login_password(app, unlock)?;
    bootnodes_address::verify_bootnode_secret_unlock(unlock)?;
    grandpa_address::verify_grandpa_secret_unlock(unlock)?;
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

#[cfg(unix)]
fn process_args(pid: u32) -> Option<String> {
    let out = Command::new("ps")
        .args(["-ww", "-p", &pid.to_string(), "-o", "args="])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let args = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if args.is_empty() {
        None
    } else {
        Some(args)
    }
}

#[cfg(not(unix))]
fn process_args(_pid: u32) -> Option<String> {
    None
}

#[cfg(unix)]
fn process_comm(pid: u32) -> Option<String> {
    let out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let comm = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if comm.is_empty() {
        None
    } else {
        Some(comm)
    }
}

#[cfg(not(unix))]
fn process_comm(_pid: u32) -> Option<String> {
    None
}

fn likely_node_command(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    lower.contains("citizenchain-node")
        || lower.contains("/target/debug/node")
        || lower.contains("/target/release/node")
}

#[cfg(unix)]
fn listener_pids_on_rpc_port_best_effort() -> Vec<u32> {
    let Ok(out) = Command::new("lsof")
        .args(["-nP", "-iTCP:9944", "-sTCP:LISTEN", "-t"])
        .output()
    else {
        return Vec::new();
    };
    let mut pids = Vec::new();
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        if let Ok(pid) = line.trim().parse::<u32>() {
            pids.push(pid);
        }
    }
    pids
}

#[cfg(not(unix))]
fn listener_pids_on_rpc_port_best_effort() -> Vec<u32> {
    Vec::new()
}

#[cfg(unix)]
fn node_pid_command_pairs() -> Result<Vec<(u32, String)>, String> {
    let out = Command::new("ps")
        .args(["-ww", "-axo", "pid=,command="])
        .output()
        .map_err(|e| format!("execute ps failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "ps failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let mut pairs = Vec::new();
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let mut it = trimmed.split_whitespace();
        let Some(pid_str) = it.next() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        let cmd = trimmed
            .strip_prefix(pid_str)
            .map(str::trim_start)
            .unwrap_or("")
            .to_string();
        if cmd.is_empty() {
            continue;
        }
        pairs.push((pid, cmd));
    }
    Ok(pairs)
}

#[cfg(not(unix))]
fn node_pid_command_pairs() -> Result<Vec<(u32, String)>, String> {
    Ok(Vec::new())
}

fn trusted_node_process_pids_on_rpc_port(app: &AppHandle) -> Result<Vec<u32>, String> {
    let data_dir_raw = node_data_dir(app)?;
    let mut base_tokens = vec![data_dir_raw.to_string_lossy().to_string()];
    if let Ok(canonical) = data_dir_raw.canonicalize() {
        base_tokens.push(canonical.to_string_lossy().to_string());
    }

    let all = node_pid_command_pairs().unwrap_or_default();
    let mut candidate: Vec<(u32, String)> = all
        .into_iter()
        .filter(|(_, cmd)| {
            let has_bin = likely_node_command(cmd);
            let has_rpc = cmd.contains("--rpc-port 9944") || cmd.contains("--rpc-port=9944");
            let has_base = base_tokens.iter().any(|token| cmd.contains(token));
            has_bin && (has_rpc || has_base)
        })
        .collect();

    let mut resolved_pids: Vec<u32> = Vec::new();

    let filtered: Vec<u32> = candidate
        .iter_mut()
        .filter_map(|(pid, cmd)| {
            if base_tokens.iter().any(|token| cmd.contains(token)) {
                Some(*pid)
            } else {
                None
            }
        })
        .collect();

    if !filtered.is_empty() {
        resolved_pids.extend(filtered);
    } else if candidate.len() == 1 {
        resolved_pids.push(candidate[0].0);
    } else {
        let fallback: Vec<u32> = candidate
            .iter()
            .filter_map(|(pid, _)| {
                let args = process_args(*pid)?;
                if likely_node_command(&args)
                    && (args.contains("--rpc-port 9944") || args.contains("--rpc-port=9944"))
                {
                    Some(*pid)
                } else {
                    None
                }
            })
            .collect();
        resolved_pids.extend(fallback);
    }

    if resolved_pids.is_empty() {
        let from_lsof: Vec<u32> = listener_pids_on_rpc_port_best_effort()
            .into_iter()
            .filter(|pid| {
                process_args(*pid)
                    .map(|args| likely_node_command(&args))
                    .unwrap_or(false)
                    || process_comm(*pid)
                        .map(|comm| likely_node_command(&comm))
                        .unwrap_or(false)
            })
            .collect();
        resolved_pids.extend(from_lsof);
    }

    if resolved_pids.is_empty() {
        let listeners = listener_pids_on_rpc_port_best_effort();
        if !listeners.is_empty() && is_expected_rpc_node() {
            resolved_pids.extend(listeners);
        }
    }

    resolved_pids.sort_unstable();
    resolved_pids.dedup();
    Ok(resolved_pids)
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    let Some(raw_pid) = u32_to_pid_t(pid) else {
        return false;
    };
    let rc = unsafe { libc::kill(raw_pid, 0) };
    if rc == 0 {
        return true;
    }
    matches!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::EPERM)
    )
}

#[cfg(not(unix))]
fn pid_alive(_pid: u32) -> bool {
    false
}

#[cfg(unix)]
fn u32_to_pid_t(pid: u32) -> Option<libc::pid_t> {
    i32::try_from(pid).ok().map(|v| v as libc::pid_t)
}

fn terminate_pid(pid: u32) {
    #[cfg(unix)]
    unsafe {
        if let Some(raw_pid) = u32_to_pid_t(pid) {
            let _ = libc::kill(raw_pid, libc::SIGTERM);
        }
    }

    for _ in 0..20 {
        if !pid_alive(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }

    #[cfg(unix)]
    unsafe {
        if let Some(raw_pid) = u32_to_pid_t(pid) {
            let _ = libc::kill(raw_pid, libc::SIGKILL);
        }
    }
}

fn terminate_trusted_listener_nodes(app: &AppHandle) -> Result<(), String> {
    for pid in trusted_node_process_pids_on_rpc_port(app)? {
        terminate_pid(pid);
    }
    Ok(())
}

fn spawn_node(
    app: &AppHandle,
    node_bin: &Path,
    unlock_password: &str,
) -> Result<(Child, Option<PathBuf>), String> {
    let base_path = node_data_dir(app)?;
    let bootnode_key = bootnodes_address::load_bootnode_node_key(app, unlock_password)?;
    let enable_grandpa_validator =
        grandpa_address::prepare_grandpa_for_start(app, unlock_password)?;
    let node_name = load_node_name(app)?;

    let mut cmd = Command::new(node_bin);
    fee_address::ensure_powr_keystore_key(app, unlock_password)?;
    cmd.arg("--base-path")
        .arg(base_path)
        .arg("--rpc-port")
        .arg("9944")
        .arg("--no-prometheus")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut node_key_file: Option<PathBuf> = None;
    if let Some(node_key) = bootnode_key {
        let temp_file = write_node_key_temp_file(app, &node_key)?;
        cmd.arg("--node-key-file").arg(&temp_file);
        node_key_file = Some(temp_file);
    }
    if let Some(name) = node_name {
        cmd.arg("--name").arg(name);
    }
    if enable_grandpa_validator {
        cmd.arg("--validator");
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
        Ok(child) => Ok((child, node_key_file)),
        Err(e) => {
            if let Some(path) = node_key_file.as_ref() {
                let _ = remove_node_key_file(path);
            }
            Err(format!(
                "spawn node failed from {}: {e}",
                node_bin.display()
            ))
        }
    }
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        if let Some(pid) = u32_to_pid_t(child.id()) {
            if pid > 0 {
                let _ = libc::kill(-pid, libc::SIGTERM);
            }
        }
    }

    for _ in 0..20 {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(_) => return,
        }
    }

    #[cfg(unix)]
    unsafe {
        if let Some(pid) = u32_to_pid_t(child.id()) {
            if pid > 0 {
                let _ = libc::kill(-pid, libc::SIGKILL);
            }
        }
    }

    let _ = child.kill();
    let _ = child.try_wait();
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    let mut last_err = String::new();
    for attempt in 0..RPC_RETRY_COUNT {
        match rpc::rpc_post(
            method,
            params.clone(),
            RPC_REQUEST_TIMEOUT,
            MAX_RPC_RESPONSE_BYTES,
        ) {
            Ok(v) => return Ok(v),
            Err(err) => {
                last_err = err;
                if attempt + 1 < RPC_RETRY_COUNT {
                    thread::sleep(Duration::from_millis(250));
                }
            }
        }
    }
    Err(last_err)
}

fn is_expected_rpc_node() -> bool {
    let Ok(properties) = rpc_post("system_properties", Value::Array(vec![])) else {
        return false;
    };
    let ss58 = properties
        .get("ss58Format")
        .and_then(|v| {
            if let Some(raw) = v.as_u64() {
                Some(raw)
            } else {
                v.as_str().and_then(|s| s.parse::<u64>().ok())
            }
        })
        .unwrap_or(0);
    if ss58 != EXPECTED_SS58_PREFIX {
        return false;
    }

    rpc_post("system_name", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| !s.trim().is_empty()))
        .unwrap_or(false)
}

fn hex_to_u64(hex: &str) -> Option<u64> {
    let trimmed = hex.strip_prefix("0x")?;
    u64::from_str_radix(trimmed, 16).ok()
}

fn header_block_height(header: &Value) -> Option<u64> {
    header
        .get("number")
        .and_then(Value::as_str)
        .and_then(hex_to_u64)
}

fn finalized_block_height() -> Option<u64> {
    let hash = rpc_post("chain_getFinalizedHead", Value::Array(vec![]))
        .ok()?
        .as_str()?
        .to_string();
    let header = rpc_post("chain_getHeader", Value::Array(vec![Value::String(hash)])).ok()?;
    header_block_height(&header)
}

fn syncing_flag() -> Option<bool> {
    let health = rpc_post("system_health", Value::Array(vec![])).ok()?;
    if let Some(v) = health.get("isSyncing") {
        if let Some(b) = v.as_bool() {
            return Some(b);
        }
        if let Some(s) = v.as_str() {
            let lowered = s.trim().to_ascii_lowercase();
            if lowered == "true" {
                return Some(true);
            }
            if lowered == "false" {
                return Some(false);
            }
        }
    }
    None
}

pub(crate) fn cleanup_on_exit(app: &AppHandle) {
    if let Ok(mut state) = app.state::<AppState>().0.lock() {
        if let Some(mut child) = state.local_node.take() {
            terminate_child(&mut child);
        }
        clear_runtime_node_key_file(&mut state);
        clear_runtime_node_bin_file(&mut state);
    }
    let _ = terminate_trusted_listener_nodes(app);
    if let Err(err) = cleanup_stale_staged_node_bins(app, None) {
        eprintln!("cleanup stale staged node bins on exit failed: {err}");
    }
    if let Err(err) = cleanup_stale_node_key_temp_files(app) {
        eprintln!("cleanup stale node-key temp files on exit failed: {err}");
    }
}

pub(crate) fn current_status(app: &AppHandle) -> Result<NodeStatus, String> {
    let (managed_running, managed_pid) = {
        let app_state = app.state::<AppState>();
        let mut state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;
        refresh_managed_process(&mut state)
    };
    if managed_running {
        return Ok(NodeStatus {
            running: true,
            state: "running".to_string(),
            pid: managed_pid,
        });
    }

    let listener_pids = trusted_node_process_pids_on_rpc_port(app)?;
    if let Some(pid) = listener_pids.into_iter().next() {
        return Ok(NodeStatus {
            running: true,
            state: "running".to_string(),
            pid: Some(pid),
        });
    }

    let fallback_running = is_expected_rpc_node();
    Ok(NodeStatus {
        running: fallback_running,
        state: if fallback_running {
            "running"
        } else {
            "stopped"
        }
        .to_string(),
        pid: None,
    })
}

async fn join_blocking_task<T>(
    task: &'static str,
    result: tauri::async_runtime::JoinHandle<Result<T, String>>,
) -> Result<T, String> {
    result
        .await
        .map_err(|e| format!("{task} join failed: {e}"))?
}

fn get_node_status_sync(app: AppHandle) -> Result<NodeStatus, String> {
    current_status(&app)
}

fn start_node_sync(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    let _ = security::append_audit_log(&app, "start_node", "attempt");
    let result = (|| -> Result<NodeStatus, String> {
        let unlock_password = security::ensure_unlock_password(&unlock_password)?.to_string();
        verify_start_unlock_password(&app, &unlock_password)?;
        let node_bin = stage_verified_node_bin(&app)?;

        {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            if let Some(mut child) = state.local_node.take() {
                terminate_child(&mut child);
            }
            clear_runtime_node_key_file(&mut state);
            clear_runtime_node_bin_file(&mut state);
        }

        terminate_trusted_listener_nodes(&app)?;
        cleanup_stale_staged_node_bins(&app, Some(node_bin.as_path()))?;
        cleanup_stale_node_key_temp_files(&app)?;
        thread::sleep(Duration::from_millis(250));

        let (mut child, node_key_file) = match spawn_node(&app, &node_bin, &unlock_password) {
            Ok(v) => v,
            Err(err) => {
                let _ = remove_staged_node_bin_file(&node_bin);
                return Err(err);
            }
        };
        {
            let app_state = app.state::<AppState>();
            let mut state = match app_state.0.lock() {
                Ok(state) => state,
                Err(_) => {
                    terminate_child(&mut child);
                    if let Some(path) = node_key_file.as_ref() {
                        let _ = remove_node_key_file(path);
                    }
                    let _ = remove_staged_node_bin_file(&node_bin);
                    return Err("acquire process state failed".to_string());
                }
            };
            state.local_node = Some(child);
            state.node_key_file = node_key_file;
            state.node_bin_file = Some(node_bin);
        }

        thread::sleep(Duration::from_millis(800));
        if let Err(err) = fee_address::sync_saved_reward_wallet_binding(&app, &unlock_password) {
            eprintln!("sync reward wallet binding skipped: {err}");
        }
        grandpa_address::verify_grandpa_after_start(&app, &unlock_password)?;
        current_status(&app)
    })();
    let _ = security::append_audit_log(
        &app,
        "start_node",
        if result.is_ok() { "success" } else { "failed" },
    );
    result
}

fn stop_node_sync(app: AppHandle) -> Result<NodeStatus, String> {
    let _ = security::append_audit_log(&app, "stop_node", "attempt");
    let result = (|| -> Result<NodeStatus, String> {
        {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            if let Some(mut child) = state.local_node.take() {
                terminate_child(&mut child);
            }
            clear_runtime_node_key_file(&mut state);
            clear_runtime_node_bin_file(&mut state);
        }

        terminate_trusted_listener_nodes(&app)?;
        cleanup_stale_staged_node_bins(&app, None)?;
        cleanup_stale_node_key_temp_files(&app)?;
        thread::sleep(Duration::from_millis(250));
        let status = current_status(&app)?;
        if status.running {
            let pid_text = status
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            return Err(format!("停止失败：节点仍在运行（pid={pid_text}）"));
        }
        Ok(status)
    })();
    let _ = security::append_audit_log(
        &app,
        "stop_node",
        if result.is_ok() { "success" } else { "failed" },
    );
    result
}

#[tauri::command]
pub async fn get_node_status(app: AppHandle) -> Result<NodeStatus, String> {
    join_blocking_task(
        "get_node_status",
        tauri::async_runtime::spawn_blocking(move || get_node_status_sync(app)),
    )
    .await
}

#[tauri::command]
pub async fn start_node(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    join_blocking_task(
        "start_node",
        tauri::async_runtime::spawn_blocking(move || start_node_sync(app, unlock_password)),
    )
    .await
}

#[tauri::command]
pub async fn stop_node(app: AppHandle) -> Result<NodeStatus, String> {
    join_blocking_task(
        "stop_node",
        tauri::async_runtime::spawn_blocking(move || stop_node_sync(app)),
    )
    .await
}

#[tauri::command]
pub fn set_node_name(
    app: AppHandle,
    node_name: String,
    unlock_password: String,
) -> Result<NodeIdentity, String> {
    let unlock = security::ensure_unlock_password(&unlock_password)?;
    security::verify_device_login_password(&app, unlock)?;
    let normalized = normalize_node_name(&node_name)?;
    let raw = serde_json::to_string_pretty(&StoredNodeName {
        node_name: normalized.clone(),
    })
    .map_err(|e| format!("encode node-name failed: {e}"))?;

    security::write_text_atomic(&node_name_path(&app)?, &format!("{raw}\n"))
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

fn get_chain_status_sync(app: AppHandle) -> Result<ChainStatus, String> {
    if !current_status(&app)?.running {
        return Ok(ChainStatus {
            block_height: None,
            finalized_height: None,
            syncing: None,
        });
    }

    let block_height = rpc_post("chain_getHeader", Value::Array(vec![]))
        .ok()
        .as_ref()
        .and_then(header_block_height);
    let finalized_height = finalized_block_height();
    let syncing = syncing_flag();

    Ok(ChainStatus {
        block_height,
        finalized_height,
        syncing,
    })
}

fn get_node_identity_sync(app: AppHandle) -> Result<NodeIdentity, String> {
    let configured_node_name = load_node_name(&app)?;
    if !current_status(&app)?.running {
        return Ok(NodeIdentity {
            node_name: configured_node_name,
            peer_id: None,
            role: Some("全节点".to_string()),
        });
    }

    let rpc_node_name = rpc_post("system_name", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()));
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

pub(crate) fn get_node_identity_blocking(app: AppHandle) -> Result<NodeIdentity, String> {
    get_node_identity_sync(app)
}

pub(crate) fn start_node_blocking(
    app: AppHandle,
    unlock_password: String,
) -> Result<NodeStatus, String> {
    start_node_sync(app, unlock_password)
}

pub(crate) fn stop_node_blocking(app: AppHandle) -> Result<NodeStatus, String> {
    stop_node_sync(app)
}

#[tauri::command]
pub async fn get_chain_status(app: AppHandle) -> Result<ChainStatus, String> {
    join_blocking_task(
        "get_chain_status",
        tauri::async_runtime::spawn_blocking(move || get_chain_status_sync(app)),
    )
    .await
}

#[tauri::command]
pub async fn get_node_identity(app: AppHandle) -> Result<NodeIdentity, String> {
    join_blocking_task(
        "get_node_identity",
        tauri::async_runtime::spawn_blocking(move || get_node_identity_sync(app)),
    )
    .await
}
