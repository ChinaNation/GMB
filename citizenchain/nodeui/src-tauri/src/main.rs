use serde::Serialize;
use std::{
    fs,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::Duration,
};
use tauri::{AppHandle, Manager};

struct RuntimeState {
    local_node: Option<Child>,
}

struct AppState(Mutex<RuntimeState>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeStatus {
    running: bool,
    state: String,
    pid: Option<u32>,
}

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))?;
    let data = app_data.join("node-data");
    fs::create_dir_all(&data).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(data)
}

fn is_rpc_9944_reachable() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:9944".parse().expect("hardcoded socket address must parse"),
        Duration::from_millis(250),
    )
    .is_ok()
}

fn refresh_managed_process(state: &mut RuntimeState) -> (bool, Option<u32>) {
    if let Some(child) = state.local_node.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => {
                state.local_node = None;
                (false, None)
            }
            Ok(None) => (true, Some(child.id())),
        }
    } else {
        (false, None)
    }
}

fn current_status(app: &AppHandle) -> Result<NodeStatus, String> {
    let app_state = app.state::<AppState>();
    let mut state = app_state
        .0
        .lock()
        .map_err(|_| "acquire process state failed".to_string())?;

    let (managed_running, pid) = refresh_managed_process(&mut state);
    let running = managed_running || is_rpc_9944_reachable();

    Ok(NodeStatus {
        running,
        state: if running { "running" } else { "stopped" }.to_string(),
        pid,
    })
}

fn find_node_bin() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Runtime CWD candidates (tauri dev / packaged run may differ).
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("../target/debug/node"));
        candidates.push(cwd.join("../target/release/node"));
        candidates.push(cwd.join("../../target/debug/node"));
        candidates.push(cwd.join("../../target/release/node"));
        candidates.push(cwd.join("sidecar/citizenchain-node"));
        candidates.push(cwd.join("desktop/sidecar/citizenchain-node"));
    }

    // Compile-time anchor from src-tauri directory: /.../nodeui/src-tauri
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join("../../target/debug/node"));
    candidates.push(manifest_dir.join("../../target/release/node"));
    candidates.push(manifest_dir.join("../target/debug/node"));
    candidates.push(manifest_dir.join("../target/release/node"));
    candidates.push(manifest_dir.join("binaries/citizenchain-node"));

    for path in candidates {
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

fn spawn_node(app: &AppHandle, node_bin: &Path) -> Result<Child, String> {
    let base_path = node_data_dir(app)?;

    let mut cmd = Command::new(node_bin);
    cmd.arg("--base-path")
        .arg(base_path)
        .arg("--rpc-port")
        .arg("9944")
        .arg("--no-prometheus")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

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

    cmd.spawn()
        .map_err(|e| format!("spawn node failed from {}: {e}", node_bin.display()))
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
        let _ = Command::new("pkill")
            .args(["-f", "citizenchain-node"])
            .status();
        let _ = Command::new("pkill")
            .args(["-f", "/target/debug/node"])
            .status();
        let _ = Command::new("pkill")
            .args(["-f", "/target/release/node"])
            .status();
    }
}

#[tauri::command]
fn get_node_status(app: AppHandle) -> Result<NodeStatus, String> {
    current_status(&app)
}

#[tauri::command]
fn start_node(app: AppHandle) -> Result<NodeStatus, String> {
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
    }

    kill_external_nodes();
    thread::sleep(Duration::from_millis(250));

    let child = spawn_node(&app, &node_bin)?;
    {
        let app_state = app.state::<AppState>();
        let mut state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;
        state.local_node = Some(child);
    }

    thread::sleep(Duration::from_millis(800));
    current_status(&app)
}

#[tauri::command]
fn stop_node(app: AppHandle) -> Result<NodeStatus, String> {
    {
        let app_state = app.state::<AppState>();
        let mut state = app_state
            .0
            .lock()
            .map_err(|_| "acquire process state failed".to_string())?;

        if let Some(mut child) = state.local_node.take() {
            terminate_child(&mut child);
        }
    }

    kill_external_nodes();
    thread::sleep(Duration::from_millis(250));

    current_status(&app)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(RuntimeState { local_node: None })))
        .invoke_handler(tauri::generate_handler![get_node_status, start_node, stop_node])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Ok(mut state) = app.state::<AppState>().0.lock() {
                    if let Some(mut child) = state.local_node.take() {
                        terminate_child(&mut child);
                    }
                }
                kill_external_nodes();
            }
        });
}
