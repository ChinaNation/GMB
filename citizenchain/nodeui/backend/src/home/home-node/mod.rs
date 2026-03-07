use crate::{
    settings::{bootnodes_address, fee_address, grandpa_address, security},
    validation::normalize_node_name,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
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
use tauri::{AppHandle, Manager};

const RPC_ADDR: &str = "127.0.0.1:9944";
const EXPECTED_SS58_PREFIX: u64 = 2027;

pub struct RuntimeState {
    pub local_node: Option<Child>,
    pub node_key_file: Option<PathBuf>,
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

fn node_name_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("node-name.json"))
}

fn node_key_runtime_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = security::app_data_dir(app)?.join("runtime-secrets");
    fs::create_dir_all(&path).map_err(|e| format!("create runtime secrets dir failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("set runtime secrets dir permission failed: {e}"))?;
    }
    Ok(path)
}

fn cleanup_stale_runtime_secret_files(app: &AppHandle) -> Result<(), String> {
    let dir = node_key_runtime_dir(app)?;
    let entries =
        fs::read_dir(&dir).map_err(|e| format!("read runtime secrets dir failed: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read runtime secrets entry failed: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name.starts_with("node-key-") && name.ends_with(".tmp") {
            fs::remove_file(&path).map_err(|e| {
                format!(
                    "remove stale node-key file failed ({}): {e}",
                    path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn write_node_key_runtime_file(app: &AppHandle, node_key: &str) -> Result<PathBuf, String> {
    let dir = node_key_runtime_dir(app)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("get system time failed: {e}"))?
        .as_nanos();
    let pid = std::process::id();

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
                cleanup_node_key_runtime_file_in_state(state);
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

fn node_bin_path() -> PathBuf {
    node_bin_dir().join("citizenchain-node")
}

fn node_bin_hash_path(node_bin: &Path) -> Result<PathBuf, String> {
    let file_name = node_bin
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| {
            format!(
                "resolve node binary filename failed ({})",
                node_bin.display()
            )
        })?;
    Ok(node_bin.with_file_name(format!("{file_name}.sha256")))
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

fn verify_node_bin_integrity(node_bin: &Path) -> Result<(), String> {
    let hash_path = node_bin_hash_path(node_bin)?;
    if !hash_path.is_file() {
        return Err(format!(
            "node binary hash file missing: {}",
            hash_path.display()
        ));
    }

    let expected_raw = fs::read_to_string(&hash_path).map_err(|e| {
        format!(
            "read node binary hash failed ({}): {e}",
            hash_path.display()
        )
    })?;
    let expected = parse_sha256_hex(&expected_raw)?;
    let actual = file_sha256_hex(node_bin)?;
    if actual != expected {
        return Err(format!(
            "node binary sha256 mismatch (bin={}, hash_file={})",
            node_bin.display(),
            hash_path.display()
        ));
    }
    Ok(())
}

fn find_node_bin() -> Result<PathBuf, String> {
    let node_bin = node_bin_path();
    if !node_bin.is_file() {
        return Err(format!("node binary not found: {}", node_bin.display()));
    }

    let canonical_bin = node_bin
        .canonicalize()
        .map_err(|e| format!("canonicalize node binary failed: {e}"))?;
    let canonical_dir = node_bin_dir()
        .canonicalize()
        .map_err(|e| format!("canonicalize node binary dir failed: {e}"))?;
    if !canonical_bin.starts_with(&canonical_dir) {
        return Err(format!(
            "node binary is outside trusted dir: {}",
            canonical_bin.display()
        ));
    }

    verify_node_bin_integrity(&canonical_bin)?;
    Ok(canonical_bin)
}

fn verify_start_unlock_password(unlock_password: &str) -> Result<(), String> {
    let unlock = security::ensure_unlock_password(unlock_password)?;
    security::verify_device_login_password(unlock)?;
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
    let rc = unsafe { libc::kill(pid as i32, 0) };
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

fn terminate_pid(pid: u32) {
    #[cfg(unix)]
    unsafe {
        let _ = libc::kill(pid as i32, libc::SIGTERM);
    }

    for _ in 0..20 {
        if !pid_alive(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }

    #[cfg(unix)]
    unsafe {
        let _ = libc::kill(pid as i32, libc::SIGKILL);
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
    let mut node_key_runtime_file: Option<PathBuf> = None;

    let mut cmd = Command::new(node_bin);
    cmd.arg("--base-path")
        .arg(base_path)
        .arg("--rpc-port")
        .arg("9944")
        .arg("--no-prometheus")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd.env(
        "POWR_MINER_SURI",
        fee_address::ensure_miner_suri(app, unlock_password)?,
    );

    if let Some(node_key) = bootnode_key {
        let key_file = write_node_key_runtime_file(app, &node_key)?;
        cmd.arg("--node-key-file").arg(&key_file);
        node_key_runtime_file = Some(key_file);
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
        Ok(child) => Ok((child, node_key_runtime_file)),
        Err(e) => {
            cleanup_node_key_runtime_file(node_key_runtime_file);
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

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    })
    .to_string();

    let req = format!(
        "POST / HTTP/1.1\r\nHost: {RPC_ADDR}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let addr = RPC_ADDR
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

pub(crate) fn cleanup_on_exit(app: &AppHandle) {
    if let Ok(mut state) = app.state::<AppState>().0.lock() {
        if let Some(mut child) = state.local_node.take() {
            terminate_child(&mut child);
        }
        cleanup_node_key_runtime_file_in_state(&mut state);
    }
    let _ = terminate_trusted_listener_nodes(app);
}

pub(crate) fn current_status(app: &AppHandle) -> Result<NodeStatus, String> {
    let app_state = app.state::<AppState>();
    let mut state = app_state
        .0
        .lock()
        .map_err(|_| "acquire process state failed".to_string())?;

    let (managed_running, managed_pid) = refresh_managed_process(&mut state);
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

#[tauri::command]
pub fn get_node_status(app: AppHandle) -> Result<NodeStatus, String> {
    current_status(&app)
}

#[tauri::command]
pub fn start_node(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    let unlock_password = security::ensure_unlock_password(&unlock_password)?.to_string();
    verify_start_unlock_password(&unlock_password)?;
    cleanup_stale_runtime_secret_files(&app)?;
    let node_bin = find_node_bin()?;

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

    terminate_trusted_listener_nodes(&app)?;
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
    if let Err(err) = fee_address::sync_saved_reward_wallet_binding(&app, &unlock_password) {
        eprintln!("sync reward wallet binding skipped: {err}");
    }
    grandpa_address::verify_grandpa_after_start(&app, &unlock_password)?;
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

    terminate_trusted_listener_nodes(&app)?;
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
