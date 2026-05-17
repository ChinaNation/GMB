//! 本机同步守护：检测底层 P2P 已连接但 block sync peer 表为空的脱钩状态。
//!
//! 守护器只访问本机 `127.0.0.1` RPC，不定时请求公网参考节点，也不把区块高度
//! 是否增长作为故障条件，避免无交易出块暂停时误重启。

use crate::shared::{constants::RPC_RESPONSE_LIMIT_LARGE, rpc, security};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::VecDeque,
    sync::{mpsc, Mutex, OnceLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

use super::{identity::current_status, process};

const CHECK_INTERVAL: Duration = Duration::from_secs(30);
const STARTUP_GRACE: Duration = Duration::from_secs(120);
const REQUIRED_SUSPECT_SAMPLES: u32 = 6;
const RESTART_WINDOW: Duration = Duration::from_secs(10 * 60);
const MAX_RESTARTS_IN_WINDOW: usize = 2;
const MAX_PENDING_EXTRINSICS: usize = 128;
const MAX_PENDING_EXTRINSICS_BYTES: usize = 4 * 1024 * 1024;

static GUARD_RUNTIME: OnceLock<Mutex<Option<SyncGuardRuntime>>> = OnceLock::new();
static GUARD_STATUS: OnceLock<Mutex<SyncGuardStatus>> = OnceLock::new();

struct SyncGuardRuntime {
    stop_tx: mpsc::Sender<()>,
    thread: JoinHandle<()>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// 前端/诊断读取的同步守护状态。
pub struct SyncGuardStatus {
    pub running: bool,
    pub state: String,
    pub consecutive_suspects: u32,
    pub restart_count_in_window: usize,
    pub degraded: bool,
    pub last_reason: Option<String>,
    pub last_updated_unix_secs: Option<u64>,
}

impl Default for SyncGuardStatus {
    fn default() -> Self {
        Self {
            running: false,
            state: "stopped".to_string(),
            consecutive_suspects: 0,
            restart_count_in_window: 0,
            degraded: false,
            last_reason: None,
            last_updated_unix_secs: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct LocalSyncSample {
    should_have_peers: bool,
    health_peers: usize,
    is_syncing: bool,
    system_peers: usize,
    raw_connected_peers: usize,
    raw_identified_peers: usize,
}

fn guard_runtime() -> &'static Mutex<Option<SyncGuardRuntime>> {
    GUARD_RUNTIME.get_or_init(|| Mutex::new(None))
}

fn guard_status() -> &'static Mutex<SyncGuardStatus> {
    GUARD_STATUS.get_or_init(|| Mutex::new(SyncGuardStatus::default()))
}

fn unix_secs_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn update_status<F>(f: F)
where
    F: FnOnce(&mut SyncGuardStatus),
{
    let mut status = guard_status().lock().unwrap_or_else(|err| err.into_inner());
    f(&mut status);
    status.last_updated_unix_secs = Some(unix_secs_now());
}

fn rpc_post_local(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        rpc::RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_LARGE,
    )
}

fn parse_bool(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(v)) => *v,
        Some(Value::String(v)) => v.trim().eq_ignore_ascii_case("true"),
        _ => false,
    }
}

fn sample_local_sync() -> Result<LocalSyncSample, String> {
    let health = rpc_post_local("system_health", Value::Array(vec![]))?;
    let peers = rpc_post_local("system_peers", Value::Array(vec![]))?;
    let network_state = rpc_post_local("system_unstable_networkState", Value::Array(vec![]))?;

    let health_peers = health.get("peers").and_then(Value::as_u64).unwrap_or(0) as usize;
    let should_have_peers = parse_bool(health.get("shouldHavePeers"));
    let is_syncing = parse_bool(health.get("isSyncing"));
    let system_peers = peers.as_array().map(Vec::len).unwrap_or(0);

    let connected = network_state
        .get("connectedPeers")
        .and_then(Value::as_object);
    let raw_connected_peers = connected.map(|items| items.len()).unwrap_or(0);
    let raw_identified_peers = connected
        .map(|items| {
            items
                .values()
                .filter(|peer| {
                    let has_version = peer
                        .get("versionString")
                        .and_then(Value::as_str)
                        .map(|v| !v.trim().is_empty())
                        .unwrap_or(false);
                    let has_ping = peer
                        .get("latestPingTime")
                        .map(|v| !v.is_null())
                        .unwrap_or(false);
                    has_version && has_ping
                })
                .count()
        })
        .unwrap_or(0);

    Ok(LocalSyncSample {
        should_have_peers,
        health_peers,
        is_syncing,
        system_peers,
        raw_connected_peers,
        raw_identified_peers,
    })
}

fn is_sync_network_detached(sample: &LocalSyncSample) -> bool {
    sample.should_have_peers
        && !sample.is_syncing
        && sample.health_peers == 0
        && sample.system_peers == 0
        && sample.raw_connected_peers > 0
        && sample.raw_identified_peers > 0
}

fn sample_reason(sample: &LocalSyncSample) -> String {
    format!(
        "healthPeers={}, systemPeers={}, rawConnected={}, rawIdentified={}, isSyncing={}, shouldHavePeers={}",
        sample.health_peers,
        sample.system_peers,
        sample.raw_connected_peers,
        sample.raw_identified_peers,
        sample.is_syncing,
        sample.should_have_peers,
    )
}

fn capture_pending_extrinsics() -> Vec<String> {
    let Ok(value) = rpc_post_local("author_pendingExtrinsics", Value::Array(vec![])) else {
        return Vec::new();
    };
    filter_pending_extrinsics(&value)
}

fn filter_pending_extrinsics(value: &Value) -> Vec<String> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };

    let mut total_bytes = 0usize;
    let mut extrinsics = Vec::new();
    for item in items.iter().take(MAX_PENDING_EXTRINSICS) {
        let Some(raw) = item.as_str() else {
            continue;
        };
        if !raw.starts_with("0x") {
            continue;
        }
        let bytes = raw.len();
        if total_bytes.saturating_add(bytes) > MAX_PENDING_EXTRINSICS_BYTES {
            break;
        }
        total_bytes = total_bytes.saturating_add(bytes);
        extrinsics.push(raw.to_string());
    }
    extrinsics
}

fn resubmit_pending_extrinsics(extrinsics: &[String]) -> (usize, usize) {
    let mut submitted = 0usize;
    let mut failed = 0usize;
    for extrinsic in extrinsics {
        match rpc_post_local(
            "author_submitExtrinsic",
            Value::Array(vec![Value::String(extrinsic.clone())]),
        ) {
            Ok(_) => submitted = submitted.saturating_add(1),
            Err(err) => {
                failed = failed.saturating_add(1);
                eprintln!("[sync_guard] 重提交 pending extrinsic 失败: {err}");
            }
        }
    }
    (submitted, failed)
}

fn prune_restart_window(restarts: &mut VecDeque<Instant>, now: Instant) {
    while let Some(front) = restarts.front() {
        if now.duration_since(*front) <= RESTART_WINDOW {
            break;
        }
        restarts.pop_front();
    }
}

fn append_guard_audit(app: &AppHandle, status: &str) {
    if let Err(err) = security::append_audit_log(app, "sync_guard", status) {
        eprintln!("[sync_guard] 审计日志写入失败: {err}");
    }
}

fn guard_loop(app: AppHandle, stop_rx: mpsc::Receiver<()>) {
    let mut first_running_seen: Option<Instant> = None;
    let mut consecutive_suspects = 0u32;
    let mut restart_times: VecDeque<Instant> = VecDeque::new();
    let mut degraded = false;

    loop {
        if stop_rx.recv_timeout(CHECK_INTERVAL).is_ok() {
            update_status(|status| {
                status.running = false;
                status.state = "stopped".to_string();
                status.last_reason = Some("sync guard stopped".to_string());
            });
            return;
        }

        let running = current_status(&app)
            .map(|status| status.running)
            .unwrap_or(false);
        if !running {
            first_running_seen = None;
            consecutive_suspects = 0;
            degraded = false;
            update_status(|status| {
                status.running = true;
                status.state = "waiting_node".to_string();
                status.consecutive_suspects = 0;
                status.degraded = false;
                status.restart_count_in_window = restart_times.len();
                status.last_reason = Some("node is not running".to_string());
            });
            continue;
        }

        let started_at = *first_running_seen.get_or_insert_with(Instant::now);
        if started_at.elapsed() < STARTUP_GRACE {
            update_status(|status| {
                status.running = true;
                status.state = "warming_up".to_string();
                status.consecutive_suspects = 0;
                status.degraded = degraded;
                status.restart_count_in_window = restart_times.len();
                status.last_reason = Some("node startup grace window".to_string());
            });
            continue;
        }

        let now = Instant::now();
        prune_restart_window(&mut restart_times, now);
        if degraded && restart_times.len() < MAX_RESTARTS_IN_WINDOW {
            degraded = false;
        }

        let sample = match sample_local_sync() {
            Ok(sample) => sample,
            Err(err) => {
                consecutive_suspects = 0;
                update_status(|status| {
                    status.running = true;
                    status.state = "sample_failed".to_string();
                    status.consecutive_suspects = 0;
                    status.degraded = degraded;
                    status.restart_count_in_window = restart_times.len();
                    status.last_reason = Some(err);
                });
                continue;
            }
        };

        let reason = sample_reason(&sample);
        if !is_sync_network_detached(&sample) {
            consecutive_suspects = 0;
            update_status(|status| {
                status.running = true;
                status.state = "healthy".to_string();
                status.consecutive_suspects = 0;
                status.degraded = degraded;
                status.restart_count_in_window = restart_times.len();
                status.last_reason = Some(reason.clone());
            });
            continue;
        }

        consecutive_suspects = consecutive_suspects.saturating_add(1);
        update_status(|status| {
            status.running = true;
            status.state = "suspect".to_string();
            status.consecutive_suspects = consecutive_suspects;
            status.degraded = degraded;
            status.restart_count_in_window = restart_times.len();
            status.last_reason = Some(reason.clone());
        });

        if consecutive_suspects < REQUIRED_SUSPECT_SAMPLES {
            continue;
        }

        if restart_times.len() >= MAX_RESTARTS_IN_WINDOW {
            if !degraded {
                append_guard_audit(&app, "degraded");
            }
            degraded = true;
            update_status(|status| {
                status.running = true;
                status.state = "degraded".to_string();
                status.consecutive_suspects = consecutive_suspects;
                status.restart_count_in_window = restart_times.len();
                status.degraded = true;
                status.last_reason = Some("restart limit reached".to_string());
            });
            continue;
        }

        append_guard_audit(&app, "restart_attempt");
        update_status(|status| {
            status.running = true;
            status.state = "restarting".to_string();
            status.consecutive_suspects = consecutive_suspects;
            status.restart_count_in_window = restart_times.len();
            status.degraded = false;
            status.last_reason = Some(reason.clone());
        });

        // 中文注释：重启会清掉节点内存交易池，先抓取本机 pending extrinsics；
        // 重启后再按限额重提，已入块或已过期的交易失败只记日志，不阻断节点恢复。
        let pending = capture_pending_extrinsics();
        match process::restart_node_for_sync_guard(app.clone()) {
            Ok(_) => {
                let (submitted, failed) = resubmit_pending_extrinsics(&pending);
                restart_times.push_back(now);
                consecutive_suspects = 0;
                first_running_seen = Some(Instant::now());
                append_guard_audit(&app, "restart_success");
                update_status(|status| {
                    status.running = true;
                    status.state = "restarted".to_string();
                    status.consecutive_suspects = 0;
                    status.restart_count_in_window = restart_times.len();
                    status.degraded = false;
                    status.last_reason = Some(format!(
                        "pending={}, resubmitted={}, failed={}",
                        pending.len(),
                        submitted,
                        failed
                    ));
                });
            }
            Err(err) => {
                restart_times.push_back(now);
                append_guard_audit(&app, "restart_failed");
                update_status(|status| {
                    status.running = true;
                    status.state = "restart_failed".to_string();
                    status.consecutive_suspects = consecutive_suspects;
                    status.restart_count_in_window = restart_times.len();
                    status.degraded = false;
                    status.last_reason = Some(err);
                });
            }
        }
    }
}

/// 启动同步守护线程。重复调用会被忽略。
pub(crate) fn start_sync_guard(app: AppHandle) {
    let mut runtime = guard_runtime()
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    if runtime.is_some() {
        return;
    }

    let (stop_tx, stop_rx) = mpsc::channel();
    let thread = thread::Builder::new()
        .name("sync-guard".to_string())
        .spawn(move || guard_loop(app, stop_rx))
        .expect("spawn sync guard thread failed");

    *runtime = Some(SyncGuardRuntime { stop_tx, thread });
    update_status(|status| {
        status.running = true;
        status.state = "started".to_string();
        status.last_reason = Some("sync guard started".to_string());
    });
}

/// 停止同步守护线程，App 退出时必须先停守护器再停节点。
pub(crate) fn stop_sync_guard() {
    let runtime = {
        let mut guard = guard_runtime()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        guard.take()
    };
    if let Some(runtime) = runtime {
        let _ = runtime.stop_tx.send(());
        let _ = runtime.thread.join();
    }
}

fn status_snapshot() -> SyncGuardStatus {
    guard_status()
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone()
}

#[tauri::command]
pub async fn get_sync_guard_status() -> Result<SyncGuardStatus, String> {
    Ok(status_snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suspect_sample() -> LocalSyncSample {
        LocalSyncSample {
            should_have_peers: true,
            health_peers: 0,
            is_syncing: false,
            system_peers: 0,
            raw_connected_peers: 1,
            raw_identified_peers: 1,
        }
    }

    #[test]
    fn detects_sync_network_detached_without_block_height() {
        assert!(is_sync_network_detached(&suspect_sample()));
    }

    #[test]
    fn does_not_restart_when_raw_network_is_disconnected() {
        let sample = LocalSyncSample {
            raw_connected_peers: 0,
            raw_identified_peers: 0,
            ..suspect_sample()
        };
        assert!(!is_sync_network_detached(&sample));
    }

    #[test]
    fn does_not_restart_when_sync_peers_exist() {
        let sample = LocalSyncSample {
            health_peers: 1,
            system_peers: 1,
            ..suspect_sample()
        };
        assert!(!is_sync_network_detached(&sample));
    }

    #[test]
    fn does_not_restart_while_major_syncing() {
        let sample = LocalSyncSample {
            is_syncing: true,
            ..suspect_sample()
        };
        assert!(!is_sync_network_detached(&sample));
    }

    #[test]
    fn does_not_restart_without_identified_raw_peer() {
        let sample = LocalSyncSample {
            raw_identified_peers: 0,
            ..suspect_sample()
        };
        assert!(!is_sync_network_detached(&sample));
    }

    #[test]
    fn pending_extrinsics_filter_keeps_hex_items_with_limits() {
        let value = serde_json::json!(["0x1234", "bad", 123, "0xabcd"]);
        assert_eq!(
            filter_pending_extrinsics(&value),
            vec!["0x1234".to_string(), "0xabcd".to_string()]
        );
    }

    #[test]
    fn restart_window_prunes_old_entries() {
        let now = Instant::now();
        let mut restarts = VecDeque::from([
            now - RESTART_WINDOW - Duration::from_secs(1),
            now - Duration::from_secs(1),
        ]);
        prune_restart_window(&mut restarts, now);
        assert_eq!(restarts.len(), 1);
    }
}
