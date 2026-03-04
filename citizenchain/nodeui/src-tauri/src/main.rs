use serde::{Deserialize, Serialize};
use rand::RngCore;
use std::{
    collections::VecDeque,
    env, fs,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const MAX_LOG_LINES: usize = 500;

struct BackgroundProcessState {
    local_node: Option<Child>,
    node_started_at: Option<u64>,
    logs: VecDeque<String>,
    install_progress: InstallProgress,
}

struct AppProcessState(Mutex<BackgroundProcessState>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallProgress {
    stage: String,
    message: String,
    updated_at: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallerStatus {
    installed: bool,
    running: bool,
    state: String,
    node_bin: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeHealth {
    running: bool,
    pid: Option<u32>,
    uptime_sec: Option<u64>,
    rpc_reachable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreflightCheck {
    ready: bool,
    node_bin_found: bool,
    port_9944_available: bool,
    data_dir_writable: bool,
    node_bin: Option<String>,
    data_dir: String,
    issues: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallRecord {
    installed_at: u64,
    node_bin: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RewardWalletRecord {
    address: String,
    updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BootnodeNodeKeyRecord {
    node_key: String,
    updated_at: u64,
}

fn now_unix() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("clock error: {e}"))
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))?;
    fs::create_dir_all(&app_data).map_err(|e| format!("create app data dir failed: {e}"))?;
    Ok(app_data)
}

fn install_record_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("install-state.json"))
}

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let node_data = app_data_dir(app)?.join("node-data");
    fs::create_dir_all(&node_data).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(node_data)
}

fn reward_wallet_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("reward-wallet.json"))
}

fn miner_suri_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("miner-suri.txt"))
}

fn bootnode_node_key_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("bootnode-node-key.json"))
}

fn load_or_create_miner_suri(app: &AppHandle) -> Result<String, String> {
    let path = miner_suri_path(app)?;
    if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| format!("read miner suri failed: {e}"))?;
        let suri = raw.trim().to_string();
        if !suri.is_empty() {
            return Ok(suri);
        }
    }

    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    let suri = format!("0x{}", hex::encode(seed));
    fs::write(&path, format!("{suri}\n")).map_err(|e| format!("write miner suri failed: {e}"))?;
    Ok(suri)
}

fn normalize_wallet_address(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err("收款钱包地址不能为空".to_string());
    }

    if value.starts_with("0x") {
        if value.len() != 66 {
            return Err("十六进制地址长度必须为 66（0x + 64）".to_string());
        }
        if !value[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("十六进制地址格式无效".to_string());
        }
        return Ok(value.to_ascii_lowercase());
    }

    if value.contains(char::is_whitespace) {
        return Err("地址中不能包含空白字符".to_string());
    }
    if value.len() < 32 || value.len() > 80 {
        return Err("地址长度无效".to_string());
    }
    Ok(value.to_string())
}

fn normalize_node_key(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err("node-key 不能为空".to_string());
    }

    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.len() != 64 {
        return Err("node-key 长度必须为 64 位十六进制".to_string());
    }
    if !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("node-key 格式无效，必须是十六进制".to_string());
    }
    Ok(raw.to_ascii_lowercase())
}

fn load_bootnode_node_key(app: &AppHandle) -> Result<Option<String>, String> {
    let path = bootnode_node_key_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read bootnode node-key failed: {e}"))?;
    let record: BootnodeNodeKeyRecord =
        serde_json::from_str(&raw).map_err(|e| format!("parse bootnode node-key failed: {e}"))?;
    let normalized = normalize_node_key(&record.node_key)?;
    Ok(Some(normalized))
}

fn read_install_record(app: &AppHandle) -> Option<InstallRecord> {
    let path = install_record_path(app).ok()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<InstallRecord>(&raw).ok()
}

fn write_install_record(app: &AppHandle, node_bin: &Path) -> Result<(), String> {
    let record = InstallRecord {
        installed_at: now_unix()?,
        node_bin: node_bin.display().to_string(),
    };
    let raw =
        serde_json::to_string_pretty(&record).map_err(|e| format!("encode install state failed: {e}"))?;
    let path = install_record_path(app)?;
    fs::write(path, format!("{raw}\n")).map_err(|e| format!("write install state failed: {e}"))
}

fn push_log(app: &AppHandle, line: String) {
    if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
        if state.logs.len() >= MAX_LOG_LINES {
            state.logs.pop_front();
        }
        state.logs.push_back(line);
    }
}

fn set_install_progress(app: &AppHandle, stage: &str, message: &str) {
    if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
        state.install_progress = InstallProgress {
            stage: stage.to_string(),
            message: message.to_string(),
            updated_at: now_unix().unwrap_or(0),
        };
    }
}

fn stream_reader_thread(
    app: AppHandle,
    reader: impl BufRead + Send + 'static,
    stream_tag: &'static str,
) {
    thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(content) => push_log(&app, format!("[{stream_tag}] {content}")),
                Err(err) => {
                    push_log(&app, format!("[{stream_tag}] read error: {err}"));
                    break;
                }
            }
        }
    });
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

fn find_node_bin(app: &AppHandle) -> Option<PathBuf> {
    candidate_node_bins(app)
        .into_iter()
        .find(|candidate| candidate.exists())
}

fn refresh_process_state(state: &mut BackgroundProcessState) -> bool {
    if let Some(child) = state.local_node.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) => {
                state.local_node = None;
                state.node_started_at = None;
                false
            }
            Ok(None) => true,
            Err(_) => {
                state.local_node = None;
                state.node_started_at = None;
                false
            }
        }
    } else {
        false
    }
}

fn current_status(app: &AppHandle) -> InstallerStatus {
    let installed = read_install_record(app).is_some();
    let node_bin = find_node_bin(app).map(|p| p.display().to_string());

    let running = if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
        refresh_process_state(&mut state)
    } else {
        false
    };

    let state_text = if running { "running" } else { "stopped" };

    InstallerStatus {
        installed,
        running,
        state: state_text.to_string(),
        node_bin,
    }
}

fn current_health(app: &AppHandle) -> NodeHealth {
    let mut running = false;
    let mut pid = None;
    let mut uptime_sec = None;

    if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
        running = refresh_process_state(&mut state);
        if running {
            pid = state.local_node.as_ref().map(|child| child.id());
            if let (Some(started_at), Ok(now)) = (state.node_started_at, now_unix()) {
                uptime_sec = Some(now.saturating_sub(started_at));
            }
        }
    }

    let rpc_reachable = TcpStream::connect_timeout(
        &"127.0.0.1:9944".parse().expect("hardcoded socket address must parse"),
        Duration::from_millis(300),
    )
    .is_ok();

    NodeHealth {
        running,
        pid,
        uptime_sec,
        rpc_reachable,
    }
}

fn run_preflight(app: &AppHandle) -> PreflightCheck {
    let node_bin = find_node_bin(app);
    let node_bin_found = node_bin.is_some();
    let node_bin_display = node_bin.map(|p| p.display().to_string());

    let port_9944_available = TcpListener::bind("127.0.0.1:9944").is_ok();
    let data_dir = node_data_dir(app)
        .unwrap_or_else(|_| PathBuf::from("./node-data"))
        .display()
        .to_string();

    let data_dir_writable = match node_data_dir(app) {
        Ok(path) => {
            let probe = path.join(".write-probe");
            match fs::write(&probe, b"ok") {
                Ok(_) => {
                    let _ = fs::remove_file(probe);
                    true
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    };

    let mut issues = Vec::new();
    if !node_bin_found {
        issues.push("未找到节点二进制（请先执行 prepare-sidecar 或设置 CITIZENCHAIN_NODE_BIN）".to_string());
    }
    if !port_9944_available {
        issues.push("端口 9944 被占用".to_string());
    }
    if !data_dir_writable {
        issues.push("节点数据目录不可写".to_string());
    }

    PreflightCheck {
        ready: issues.is_empty(),
        node_bin_found,
        port_9944_available,
        data_dir_writable,
        node_bin: node_bin_display,
        data_dir,
        issues,
    }
}

fn spawn_node(app: &AppHandle, node_bin: &Path) -> Result<Child, String> {
    let node_data = node_data_dir(app)?;
    let miner_suri = load_or_create_miner_suri(app)?;
    let bootnode_node_key = load_bootnode_node_key(app)?;

    let mut cmd = Command::new(node_bin);
    cmd.arg("--base-path")
        .arg(node_data.as_os_str())
        .arg("--rpc-port")
        .arg("9944")
        .env("POWR_MINER_SURI", miner_suri)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(node_key) = bootnode_node_key {
        cmd.arg("--node-key").arg(node_key);
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

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn node failed from {}: {e}", node_bin.display()))?;

    if let Some(stdout) = child.stdout.take() {
        stream_reader_thread(
            app.clone(),
            BufReader::new(stdout),
            "stdout",
        );
    }
    if let Some(stderr) = child.stderr.take() {
        stream_reader_thread(
            app.clone(),
            BufReader::new(stderr),
            "stderr",
        );
    }

    Ok(child)
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

#[tauri::command]
fn get_installer_status(app: AppHandle) -> Result<InstallerStatus, String> {
    Ok(current_status(&app))
}

#[tauri::command]
fn get_node_health(app: AppHandle) -> Result<NodeHealth, String> {
    Ok(current_health(&app))
}

#[tauri::command]
fn preflight_check(app: AppHandle) -> Result<PreflightCheck, String> {
    Ok(run_preflight(&app))
}

#[tauri::command]
fn get_install_progress(app: AppHandle) -> Result<InstallProgress, String> {
    let app_state = app.state::<AppProcessState>();
    let state = app_state
        .0
        .lock()
        .map_err(|_| "acquire process state failed".to_string())?;
    Ok(state.install_progress.clone())
}

#[tauri::command]
fn install_and_start(app: AppHandle) -> Result<InstallerStatus, String> {
    set_install_progress(&app, "checking", "正在执行安装前检查");
    let preflight = run_preflight(&app);
    if !preflight.ready {
        set_install_progress(&app, "failed", "安装前检查失败");
        return Err(format!("安装前检查未通过：{}", preflight.issues.join("；")));
    }

    set_install_progress(&app, "preparing", "正在写入安装状态");
    let node_bin = find_node_bin(&app)
        .ok_or_else(|| "未找到节点二进制，请先执行构建（prepare-sidecar）".to_string())?;

    write_install_record(&app, &node_bin)?;
    push_log(
        &app,
        format!("[system] 安装完成，使用节点二进制: {}", node_bin.display()),
    );

    set_install_progress(&app, "starting", "正在启动节点进程");
    match start_node(app.clone()) {
        Ok(status) => {
            set_install_progress(&app, "success", "安装并启动成功");
            Ok(status)
        }
        Err(err) => {
            set_install_progress(&app, "failed", "节点启动失败");
            Err(err)
        }
    }
}

#[tauri::command]
fn start_node(app: AppHandle) -> Result<InstallerStatus, String> {
    set_install_progress(&app, "starting", "正在启动节点进程");
    let app_state = app.state::<AppProcessState>();
    let mut state = app_state
        .0
        .lock()
        .map_err(|_| "acquire process state failed".to_string())?;

    if refresh_process_state(&mut state) {
        drop(state);
        set_install_progress(&app, "success", "节点已在运行");
        return Ok(current_status(&app));
    }

    let node_bin = find_node_bin(&app)
        .ok_or_else(|| "未找到节点二进制，请先执行构建（prepare-sidecar）".to_string())?;

    let child = spawn_node(&app, &node_bin)?;
    state.local_node = Some(child);
    state.node_started_at = Some(now_unix().unwrap_or(0));
    drop(state);

    push_log(&app, "[system] 节点已启动".to_string());
    set_install_progress(&app, "success", "节点运行中");
    Ok(current_status(&app))
}

#[tauri::command]
fn stop_node(app: AppHandle) -> Result<InstallerStatus, String> {
    let app_state = app.state::<AppProcessState>();
    let mut state = app_state
        .0
        .lock()
        .map_err(|_| "acquire process state failed".to_string())?;

    if let Some(mut child) = state.local_node.take() {
        terminate_child(&mut child);
        state.node_started_at = None;
        drop(state);
        push_log(&app, "[system] 节点已停止".to_string());
        set_install_progress(&app, "idle", "节点已停止");
    }

    Ok(current_status(&app))
}

#[tauri::command]
fn get_logs(app: AppHandle, tail: Option<usize>) -> Result<Vec<String>, String> {
    let app_state = app.state::<AppProcessState>();
    let state = app_state
        .0
        .lock()
        .map_err(|_| "acquire process state failed".to_string())?;
    let take = tail.unwrap_or(50).max(1);
    let total = state.logs.len();
    let start = total.saturating_sub(take);
    Ok(state.logs.iter().skip(start).cloned().collect())
}

#[tauri::command]
fn get_reward_wallet_address(app: AppHandle) -> Result<Option<String>, String> {
    let path = reward_wallet_path(&app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read reward wallet failed: {e}"))?;
    let record: RewardWalletRecord =
        serde_json::from_str(&raw).map_err(|e| format!("parse reward wallet failed: {e}"))?;
    Ok(Some(record.address))
}

#[tauri::command]
fn get_miner_suri(app: AppHandle) -> Result<String, String> {
    load_or_create_miner_suri(&app)
}

#[tauri::command]
fn set_reward_wallet_address(app: AppHandle, address: String) -> Result<String, String> {
    let normalized = normalize_wallet_address(&address)?;
    let record = RewardWalletRecord {
        address: normalized.clone(),
        updated_at: now_unix().unwrap_or(0),
    };
    let raw = serde_json::to_string_pretty(&record)
        .map_err(|e| format!("encode reward wallet failed: {e}"))?;
    let path = reward_wallet_path(&app)?;
    fs::write(path, format!("{raw}\n")).map_err(|e| format!("write reward wallet failed: {e}"))?;
    push_log(&app, format!("[system] 已保存收款钱包地址: {normalized}"));
    Ok(normalized)
}

#[tauri::command]
fn get_bootnode_node_key(app: AppHandle) -> Result<Option<String>, String> {
    load_bootnode_node_key(&app)
}

#[tauri::command]
fn set_bootnode_node_key(app: AppHandle, node_key: String) -> Result<String, String> {
    let normalized = normalize_node_key(&node_key)?;
    let record = BootnodeNodeKeyRecord {
        node_key: normalized.clone(),
        updated_at: now_unix().unwrap_or(0),
    };
    let raw = serde_json::to_string_pretty(&record)
        .map_err(|e| format!("encode bootnode node-key failed: {e}"))?;
    let path = bootnode_node_key_path(&app)?;
    fs::write(path, format!("{raw}\n")).map_err(|e| format!("write bootnode node-key failed: {e}"))?;
    push_log(&app, "[system] 已保存引导节点 node-key".to_string());
    Ok(normalized)
}

fn main() {
    tauri::Builder::default()
        .manage(AppProcessState(Mutex::new(BackgroundProcessState {
            local_node: None,
            node_started_at: None,
            logs: VecDeque::new(),
            install_progress: InstallProgress {
                stage: "idle".to_string(),
                message: "等待安装".to_string(),
                updated_at: now_unix().unwrap_or(0),
            },
        })))
        .invoke_handler(tauri::generate_handler![
            get_installer_status,
            get_node_health,
            preflight_check,
            get_install_progress,
            install_and_start,
            start_node,
            stop_node,
            get_logs,
            get_miner_suri,
            get_reward_wallet_address,
            set_reward_wallet_address,
            get_bootnode_node_key,
            set_bootnode_node_key
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Ok(mut state) = app.state::<AppProcessState>().0.lock() {
                    if let Some(mut child) = state.local_node.take() {
                        terminate_child(&mut child);
                    }
                }
            }
        });
}
