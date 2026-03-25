// 进程管理子模块：节点进程的启动、停止、生命周期管理及二进制文件校验。

use crate::{
    settings::{device_password, grandpa_address},
    shared::{keystore, rpc, security},
};
use sha2::{Digest, Sha256};
#[cfg(target_os = "linux")]
use std::collections::HashSet;
use std::{
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::{AppHandle, Manager};

use super::identity::{current_status, load_node_name, NodeStatus};
use super::rpc::is_expected_rpc_node;

const NODE_BIN_BASENAME: &str = "citizenchain-node";
const RUNTIME_SECRETS_DIR_NAME: &str = "runtime-secrets";
const NODE_KEY_TEMP_PREFIX: &str = "node-key-";
const NODE_KEY_TEMP_SUFFIX: &str = ".tmp";
const NODE_BIN_STAGE_PREFIX: &str = "node-bin-";

// 串行化节点启停与退出清理，避免并发命令互删临时文件或遗留孤儿进程。
static NODE_LIFECYCLE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// 进程托管状态，只记录当前桌面应用会话直接拉起的节点进程及其临时文件。
pub struct RuntimeState {
    pub local_node: Option<Child>,
    pub node_key_file: Option<PathBuf>,
    pub node_bin_file: Option<PathBuf>,
}

/// Tauri 全局状态，供首页节点相关命令共享。
pub struct AppState(pub Mutex<RuntimeState>);

struct ProcessSnapshot {
    pid: u32,
    cmdline: String,
    exe_path: Option<PathBuf>,
}

pub(super) fn lock_node_lifecycle() -> std::sync::MutexGuard<'static, ()> {
    NODE_LIFECYCLE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

pub(super) fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    keystore::node_data_dir(app)
}

pub(super) fn runtime_secrets_dir(app: &AppHandle) -> Result<PathBuf, String> {
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
            security::sanitize_path(&secrets_dir)
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

// 引导节点密钥在 set_bootnode_key 时已直接写入 secret_ed25519，
// 节点启动无需额外注入。
// cleanup_stale_node_key_temp_files 保留用于清理旧版本遗留的临时文件。

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
            security::sanitize_path(&secrets_dir)
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

pub(super) fn refresh_managed_process(state: &mut RuntimeState) -> (bool, Option<u32>) {
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

fn current_executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
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
    if let Some(exe_dir) = current_executable_dir() {
        dirs.push(exe_dir);
    }
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
                security::sanitize_path(node_bin)
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
    if let Some(exe_dir) = current_executable_dir() {
        if let Ok(canonical_exe_dir) = exe_dir.canonicalize() {
            if !dirs.contains(&canonical_exe_dir) {
                dirs.push(canonical_exe_dir);
            }
        }
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        if let Ok(canonical_resource_dir) = resource_dir.canonicalize() {
            dirs.push(canonical_resource_dir);
        }
    }
    // staged binary 在 runtime-secrets 目录，也要纳入信任范围，
    // 否则 cleanup 找不到从 staged binary 启动的残留进程。
    if let Ok(secrets_dir) = runtime_secrets_dir(app) {
        if let Ok(canonical_secrets_dir) = secrets_dir.canonicalize() {
            if !dirs.contains(&canonical_secrets_dir) {
                dirs.push(canonical_secrets_dir);
            }
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
                .map(|v| security::sanitize_path(v))
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

// 节点二进制先复制到运行时目录并在副本上再次验 hash，
// 这样可以把"校验通过的文件"和"真正执行的文件"绑定到同一个副本。
fn stage_verified_node_bin(app: &AppHandle) -> Result<PathBuf, String> {
    let source_bin = find_node_bin_source(app)?;
    let hash_path = resolve_node_bin_hash_path(&source_bin)?;
    let expected_raw = fs::read_to_string(&hash_path).map_err(|e| {
        format!(
            "read node binary hash failed ({}): {e}",
            security::sanitize_path(&hash_path)
        )
    })?;
    let expected = parse_sha256_hex(&expected_raw)?;
    let staged_bin = staged_node_bin_path(app, &source_bin)?;
    if let Err(e) = fs::copy(&source_bin, &staged_bin) {
        let _ = remove_staged_node_bin_file(&staged_bin);
        return Err(format!(
            "copy staged node binary failed ({} -> {}): {e}",
            security::sanitize_path(&source_bin),
            security::sanitize_path(&staged_bin)
        ));
    }
    if let Err(e) = fs::set_permissions(
        &staged_bin,
        fs::metadata(&source_bin)
            .map_err(|err| {
                format!(
                    "read source node binary metadata failed ({}): {err}",
                    security::sanitize_path(&source_bin)
                )
            })?
            .permissions(),
    ) {
        let _ = remove_staged_node_bin_file(&staged_bin);
        return Err(format!(
            "set staged node binary permissions failed ({}): {e}",
            security::sanitize_path(&staged_bin)
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
            security::sanitize_path(&source_bin),
            security::sanitize_path(&hash_path),
            security::sanitize_path(&staged_bin)
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
                    .map(|v| security::sanitize_path(v))
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
            security::sanitize_path(&canonical_bin)
        ));
    }
    Ok(canonical_bin)
}

/// 只信任 sysinfo 提供的结构化可执行路径，避免从拼接后的命令行字符串反推路径。
fn is_trusted_executable_path(exe_path: &Path, trusted_dirs: &[PathBuf]) -> bool {
    let canonical = match exe_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };
    trusted_dirs.iter().any(|dir| canonical.starts_with(dir))
}

fn is_trusted_node_process(proc: &ProcessSnapshot, trusted_dirs: &[PathBuf]) -> bool {
    proc.exe_path
        .as_deref()
        .map(|path| is_trusted_executable_path(path, trusted_dirs))
        .unwrap_or(false)
}

fn parse_rpc_port_from_cmdline(cmd: &str) -> Option<u16> {
    let mut tokens = cmd.split_whitespace();
    while let Some(token) = tokens.next() {
        if let Some(raw) = token.strip_prefix("--rpc-port=") {
            return raw.parse::<u16>().ok().filter(|port| *port > 0);
        }
        if token == "--rpc-port" {
            return tokens
                .next()
                .and_then(|raw| raw.parse::<u16>().ok())
                .filter(|port| *port > 0);
        }
    }
    None
}

fn effective_rpc_port_for_cmdline(cmd: &str) -> u16 {
    parse_rpc_port_from_cmdline(cmd).unwrap_or(rpc::DEFAULT_LOCAL_RPC_PORT)
}

fn process_snapshot(pid: u32) -> Option<ProcessSnapshot> {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(
            ProcessRefreshKind::nothing()
                .with_cmd(sysinfo::UpdateKind::Always)
                .with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
        ),
    );
    let sysinfo_pid = sysinfo::Pid::from_u32(pid);
    let proc = sys.process(sysinfo_pid)?;
    let cmdline = proc
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    if cmdline.is_empty() && proc.exe().is_none() {
        return None;
    }
    Some(ProcessSnapshot {
        pid,
        cmdline,
        exe_path: proc.exe().map(Path::to_path_buf),
    })
}

/// 使用 Rust/Linux procfs 或 lsof 获取监听目标 RPC 端口的进程 PID 列表。
#[cfg(target_os = "linux")]
fn listener_pids_on_rpc_port_best_effort(port: u16) -> Vec<u32> {
    let from_proc = listener_pids_on_rpc_port_from_procfs(port);
    if !from_proc.is_empty() {
        return from_proc;
    }
    listener_pids_on_rpc_port_from_lsof(port)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn listener_pids_on_rpc_port_best_effort(port: u16) -> Vec<u32> {
    listener_pids_on_rpc_port_from_lsof(port)
}

#[cfg(not(unix))]
fn listener_pids_on_rpc_port_best_effort(_port: u16) -> Vec<u32> {
    Vec::new()
}

#[cfg(unix)]
fn listener_pids_on_rpc_port_from_lsof(port: u16) -> Vec<u32> {
    let Ok(out) = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-t"])
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

#[cfg(target_os = "linux")]
fn listener_pids_on_rpc_port_from_procfs(port: u16) -> Vec<u32> {
    let mut inodes = HashSet::new();
    collect_listener_socket_inodes("/proc/net/tcp", port, &mut inodes);
    collect_listener_socket_inodes("/proc/net/tcp6", port, &mut inodes);
    if inodes.is_empty() {
        return Vec::new();
    }

    let Ok(proc_entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };

    let mut pids = Vec::new();
    for entry in proc_entries.flatten() {
        let file_name = entry.file_name();
        let Some(pid_str) = file_name.to_str() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        let fd_dir = entry.path().join("fd");
        let Ok(fd_entries) = fs::read_dir(fd_dir) else {
            continue;
        };
        let mut matched = false;
        for fd_entry in fd_entries.flatten() {
            let Ok(target) = fs::read_link(fd_entry.path()) else {
                continue;
            };
            let Some(target_str) = target.to_str() else {
                continue;
            };
            let Some(raw_inode) = target_str
                .strip_prefix("socket:[")
                .and_then(|value| value.strip_suffix(']'))
            else {
                continue;
            };
            if inodes.contains(raw_inode) {
                pids.push(pid);
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }
    }
    pids.sort_unstable();
    pids.dedup();
    pids
}

#[cfg(target_os = "linux")]
fn collect_listener_socket_inodes(path: &str, port: u16, out: &mut HashSet<String>) {
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    for line in raw.lines().skip(1) {
        let cols = line.split_whitespace().collect::<Vec<_>>();
        if cols.len() <= 9 {
            continue;
        }
        let Some((_, local_port_hex)) = cols[1].rsplit_once(':') else {
            continue;
        };
        let Ok(local_port) = u16::from_str_radix(local_port_hex, 16) else {
            continue;
        };
        if local_port != port {
            continue;
        }
        if cols[3] != "0A" {
            continue;
        }
        out.insert(cols[9].to_string());
    }
}

/// 使用 sysinfo 获取所有进程的结构化快照。
fn node_process_snapshots() -> Result<Vec<ProcessSnapshot>, String> {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(
            ProcessRefreshKind::nothing()
                .with_cmd(sysinfo::UpdateKind::Always)
                .with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
        ),
    );
    let mut pairs = Vec::new();
    for (pid, proc) in sys.processes() {
        let full_cmd = proc
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let exe_path = proc.exe().map(Path::to_path_buf);
        if full_cmd.is_empty() && exe_path.is_none() {
            continue;
        }
        pairs.push(ProcessSnapshot {
            pid: pid.as_u32(),
            cmdline: full_cmd,
            exe_path,
        });
    }
    Ok(pairs)
}

// 历史会话可能遗留旧节点进程，这里按"命令行特征 -> lsof -> RPC 指纹"逐层收敛，
// 只把高度疑似本应用节点的 PID 作为可信目标返回。
pub(super) fn trusted_node_process_pids_on_rpc_port(app: &AppHandle) -> Result<Vec<u32>, String> {
    let expected_rpc_port = rpc::current_rpc_port();
    let trusted_dirs = trusted_node_bin_dirs(app).unwrap_or_default();
    let data_dir_raw = node_data_dir(app)?;
    let mut base_tokens = vec![data_dir_raw.to_string_lossy().to_string()];
    if let Ok(canonical) = data_dir_raw.canonicalize() {
        base_tokens.push(canonical.to_string_lossy().to_string());
    }

    let all = node_process_snapshots().unwrap_or_default();
    let mut candidate: Vec<ProcessSnapshot> = all
        .into_iter()
        .filter(|proc| {
            let has_bin = is_trusted_node_process(proc, &trusted_dirs);
            let rpc_port = effective_rpc_port_for_cmdline(&proc.cmdline);
            let has_rpc = rpc_port == expected_rpc_port;
            let has_base = base_tokens.iter().any(|token| proc.cmdline.contains(token));
            has_bin && (has_rpc || has_base)
        })
        .collect();

    let mut resolved_pids: Vec<u32> = Vec::new();

    let filtered: Vec<u32> = candidate
        .iter_mut()
        .filter_map(|proc| {
            if base_tokens.iter().any(|token| proc.cmdline.contains(token)) {
                Some(proc.pid)
            } else {
                None
            }
        })
        .collect();

    if !filtered.is_empty() {
        if let Some(proc) = candidate.iter().find(|proc| filtered.contains(&proc.pid)) {
            rpc::remember_rpc_port(effective_rpc_port_for_cmdline(&proc.cmdline));
        }
        resolved_pids.extend(filtered);
    } else if candidate.len() == 1 {
        rpc::remember_rpc_port(effective_rpc_port_for_cmdline(&candidate[0].cmdline));
        resolved_pids.push(candidate[0].pid);
    } else {
        let fallback: Vec<u32> = candidate
            .iter()
            .filter_map(|proc| {
                let refreshed = process_snapshot(proc.pid)?;
                if is_trusted_node_process(&refreshed, &trusted_dirs)
                    && effective_rpc_port_for_cmdline(&refreshed.cmdline) == expected_rpc_port
                {
                    Some(proc.pid)
                } else {
                    None
                }
            })
            .collect();
        resolved_pids.extend(fallback);
    }

    if resolved_pids.is_empty() {
        let from_lsof: Vec<u32> = listener_pids_on_rpc_port_best_effort(expected_rpc_port)
            .into_iter()
            .filter(|pid| {
                process_snapshot(*pid)
                    .map(|proc| is_trusted_node_process(&proc, &trusted_dirs))
                    .unwrap_or(false)
            })
            .collect();
        resolved_pids.extend(from_lsof);
    }

    if resolved_pids.is_empty() {
        let listeners = listener_pids_on_rpc_port_best_effort(expected_rpc_port);
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
    if rc != 0
        && !matches!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EPERM)
        )
    {
        return false;
    }
    // 验证进程可执行文件名以节点二进制名称开头，防止 PID 复用导致误判。
    // 使用 starts_with 而非 contains，避免 "fake-citizenchain-node" 等伪造名称通过校验。
    let sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_processes(ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::Always)),
    );
    let sysinfo_pid = sysinfo::Pid::from_u32(pid);
    sys.process(sysinfo_pid)
        .and_then(|p| p.exe())
        .and_then(|exe| exe.file_name())
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with(NODE_BIN_BASENAME))
        .unwrap_or(false)
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
    let my_pid = std::process::id();
    for pid in trusted_node_process_pids_on_rpc_port(app)? {
        if pid == my_pid {
            continue;
        }
        terminate_pid(pid);
    }
    Ok(())
}

fn verify_start_unlock_password(app: &AppHandle, unlock_password: &str) -> Result<(), String> {
    let unlock = security::ensure_unlock_password(unlock_password)?;
    device_password::verify_device_login_password(app, unlock)?;
    Ok(())
}

// 启动命令只拼接固定参数，敏感密钥通过本地文件或 keystore 注入，
// 避免把明文秘密直接暴露在命令行参数或环境变量里。
//
// 引导节点密钥：存放在 `<base-path>/node-key/secret_ed25519`，
// 通过 `--node-key-file` 显式加载，使 dev 链和正式链共用同一个 Peer ID。
fn spawn_node(
    app: &AppHandle,
    node_bin: &Path,
    unlock_password: &str,
) -> Result<(Child, Option<PathBuf>), String> {
    let rpc_port = rpc::current_rpc_port();
    let base_path = node_data_dir(app)?;
    let enable_grandpa_validator =
        grandpa_address::prepare_grandpa_for_start(app, unlock_password)?;
    let node_name = load_node_name(app)?;

    let mut cmd = Command::new(node_bin);

    cmd.arg("--base-path")
        .arg(&base_path);

    // 如果存在统一的节点身份密钥，通过 --node-key-file 显式加载
    let node_key_file = base_path.join("node-key").join("secret_ed25519");
    if node_key_file.is_file() {
        cmd.arg("--node-key-file").arg(&node_key_file);
    }

    cmd.arg("--rpc-port")
        .arg(rpc_port.to_string())
        .arg("--unsafe-rpc-external")
        .arg("--rpc-methods")
        .arg("Unsafe")
        .arg("--rpc-cors")
        .arg("all")
        .arg("--no-prometheus")
        .arg("--bootnodes")
        .arg("/ip4/147.224.14.117/tcp/30333/ws/p2p/12D3KooWCbzGSMnRUmNkfE4kNPxQRzzVH7Ac9drR5fF7vJwUdTBa")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

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
        Ok(child) => Ok((child, None)),
        Err(e) => Err(format!(
            "spawn node failed from {}: {e}",
            security::sanitize_path(node_bin)
        )),
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

// 启动后的关键校验失败时，需要把刚拉起的节点和临时文件一并回滚，
// 避免前端看到"启动失败"，而本机实际上还有节点继续运行。
fn rollback_started_node(app: &AppHandle) {
    let app_state = app.state::<AppState>();
    let mut state = match app_state.0.lock() {
        Ok(state) => state,
        Err(err) => err.into_inner(),
    };
    if let Some(mut child) = state.local_node.take() {
        terminate_child(&mut child);
    }
    clear_runtime_node_key_file(&mut state);
    clear_runtime_node_bin_file(&mut state);
}

/// 启动时清理上次异常退出可能残留的节点进程和临时文件。
pub(crate) fn cleanup_on_startup(app: &AppHandle) {
    if let Err(err) = terminate_trusted_listener_nodes(app) {
        eprintln!("startup cleanup: terminate residual nodes failed: {err}");
    }
    if let Err(err) = cleanup_stale_staged_node_bins(app, None) {
        eprintln!("startup cleanup: stale staged bins failed: {err}");
    }
    if let Err(err) = cleanup_stale_node_key_temp_files(app) {
        eprintln!("startup cleanup: stale node-key temps failed: {err}");
    }
    rpc::clear_genesis_hash_cache();
}

pub(crate) fn cleanup_on_exit(app: &AppHandle) {
    let _lifecycle_guard = lock_node_lifecycle();
    if let Ok(mut state) = app.state::<AppState>().0.lock() {
        if let Some(mut child) = state.local_node.take() {
            terminate_child(&mut child);
        }
        clear_runtime_node_key_file(&mut state);
        clear_runtime_node_bin_file(&mut state);
    }
    if let Err(err) = terminate_trusted_listener_nodes(app) {
        eprintln!("terminate trusted listener nodes on exit failed: {err}");
    }
    if let Err(err) = cleanup_stale_staged_node_bins(app, None) {
        eprintln!("cleanup stale staged node bins on exit failed: {err}");
    }
    if let Err(err) = cleanup_stale_node_key_temp_files(app) {
        eprintln!("cleanup stale node-key temp files on exit failed: {err}");
    }
}

fn start_node_sync(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    let _lifecycle_guard = lock_node_lifecycle();
    if let Err(e) = security::append_audit_log(&app, "start_node", "attempt") {
        eprintln!("[审计] start_node attempt 日志写入失败: {e}");
    }
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

        // 检查节点是否在启动后立即退出
        {
            let app_state = app.state::<AppState>();
            let mut state = app_state
                .0
                .lock()
                .map_err(|_| "acquire process state failed".to_string())?;
            if let Some(child) = state.local_node.as_mut() {
                match child.try_wait() {
                    Ok(Some(_exit_status)) => {
                        state.local_node = None;
                        clear_runtime_node_key_file(&mut state);
                        clear_runtime_node_bin_file(&mut state);
                        return Err("节点启动失败，请稍后重试".to_string());
                    }
                    Ok(None) => { /* 节点正在运行，继续 */ }
                    Err(_) => {
                        state.local_node = None;
                        clear_runtime_node_key_file(&mut state);
                        clear_runtime_node_bin_file(&mut state);
                        return Err("节点启动失败，请稍后重试".to_string());
                    }
                }
            }
        }

        if let Err(err) = grandpa_address::verify_grandpa_after_start(&app, &unlock_password) {
            rollback_started_node(&app);
            let _ = cleanup_stale_staged_node_bins(&app, None);
            let _ = cleanup_stale_node_key_temp_files(&app);
            return Err(format!("verify grandpa after start failed: {err}"));
        }
        current_status(&app)
    })();
    if let Err(e) = security::append_audit_log(
        &app,
        "start_node",
        if result.is_ok() { "success" } else { "failed" },
    ) {
        eprintln!("[审计] start_node 结果日志写入失败: {e}");
    }
    result
}

fn stop_node_sync(app: AppHandle) -> Result<NodeStatus, String> {
    let _lifecycle_guard = lock_node_lifecycle();
    if let Err(e) = security::append_audit_log(&app, "stop_node", "attempt") {
        eprintln!("[审计] stop_node attempt 日志写入失败: {e}");
    }
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
        rpc::clear_genesis_hash_cache();
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
    if let Err(e) = security::append_audit_log(
        &app,
        "stop_node",
        if result.is_ok() { "success" } else { "failed" },
    ) {
        eprintln!("[审计] stop_node 结果日志写入失败: {e}");
    }
    result
}

#[tauri::command]
pub async fn start_node(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    let app2 = app.clone();
    let status = super::join_blocking_task(
        "start_node",
        tauri::async_runtime::spawn_blocking(move || start_node_sync(app, unlock_password)),
    )
    .await?;

    // 节点启动成功后，异步同步本地已保存的奖励钱包绑定到链上。
    // 场景：用户清链后重启节点，reward-wallet.json 仍存在但链上绑定已丢失，
    // 需要自动重新提交 bind_reward_wallet 交易，确保奖励发到绑定钱包。
    tauri::async_runtime::spawn(async move {
        // 等待 RPC 就绪（节点刚启动可能还未开始监听）
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if let Err(err) = crate::settings::fee_address::sync_saved_reward_wallet_inner(&app2).await
        {
            eprintln!("[reward-wallet] 启动后自动同步链上绑定失败: {err}");
        }
    });

    Ok(status)
}

#[tauri::command]
pub async fn stop_node(app: AppHandle, unlock_password: String) -> Result<NodeStatus, String> {
    super::join_blocking_task(
        "stop_node",
        tauri::async_runtime::spawn_blocking(move || {
            let unlock = security::ensure_unlock_password(&unlock_password)?;
            device_password::verify_device_login_password(&app, unlock)?;
            stop_node_sync(app)
        }),
    )
    .await
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

#[cfg(test)]
mod tests {
    use super::{is_trusted_executable_path, parse_rpc_port_from_cmdline};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|v| v.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("nodeui-process-{label}-{stamp}"))
    }

    #[test]
    fn trusted_executable_path_accepts_spaces_in_path() {
        let trusted_dir = unique_temp_dir("trusted dir");
        let nested_dir = trusted_dir.join("bin folder");
        fs::create_dir_all(&nested_dir).unwrap();
        let exe = nested_dir.join("citizenchain-node");
        fs::write(&exe, b"test").unwrap();

        let trusted_dirs = vec![trusted_dir.canonicalize().unwrap()];
        assert!(is_trusted_executable_path(&exe, &trusted_dirs));

        let _ = fs::remove_dir_all(&trusted_dir);
    }

    #[test]
    fn parse_rpc_port_from_cmdline_supports_separate_flag() {
        let cmd = "/tmp/citizenchain-node --rpc-port 12345 --validator";
        assert_eq!(parse_rpc_port_from_cmdline(cmd), Some(12345));
    }

    #[test]
    fn parse_rpc_port_from_cmdline_supports_equals_flag() {
        let cmd = "/tmp/citizenchain-node --rpc-port=22334 --validator";
        assert_eq!(parse_rpc_port_from_cmdline(cmd), Some(22334));
    }
}
