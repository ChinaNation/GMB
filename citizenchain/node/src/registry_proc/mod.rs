//! 节点桌面端拉起 / 停止注册局(registry)子进程。
//!
//! 形态:注册局是独立二进制(`registry` crate)。节点桌面端启动后把它拉起为子进程,
//! App 退出时统一杀掉,做到"桌面=节点运维台、浏览器=注册局管理员"双面并存。
//!
//! Card 05:注册局二进制 + PostgreSQL 官方二进制 + 前端产物 + china.sqlite 均随安装包
//! (Tauri resources)。本模块**不碰 PG**——只把资源/数据路径用环境变量告诉 registry,
//! registry 自管内嵌 PG 与内网 TLS(见 `registry/src/core/{embedded_pg,tls}.rs`)。

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

/// 进程内唯一的 registry 子进程句柄。
static REGISTRY_CHILD: Mutex<Option<Child>> = Mutex::new(None);

/// 解析 registry 二进制路径:优先随包 `resources/registry-bin/`(打包形态),
/// 兜底节点可执行文件同目录(开发期 `cargo build` 后的 `target/<profile>/registry`)。
fn registry_binary_path(app: &AppHandle) -> Option<PathBuf> {
    let name = if cfg!(target_os = "windows") {
        "registry.exe"
    } else {
        "registry"
    };
    if let Ok(res) = app.path().resource_dir() {
        let packaged = res.join("registry-bin").join(name);
        if packaged.exists() {
            ensure_executable(&packaged);
            return Some(packaged);
        }
    }
    let exe = std::env::current_exe().ok()?;
    let dev = exe.parent()?.join(name);
    if dev.exists() {
        Some(dev)
    } else {
        None
    }
}

/// Unix:随包资源可能丢失可执行位,拉起前补上(best-effort)。
#[cfg(unix)]
fn ensure_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o755);
        let _ = std::fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn ensure_executable(_path: &Path) {}

/// 当前平台在 `resources/postgres/<os>/` 下的子目录名。
fn pg_os_subdir() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

/// 把内嵌 PG / 内网 TLS / 前端产物 / china.sqlite / 链 RPC 的路径用环境变量传给 registry。
///
/// 中文注释:仅当随包 PostgreSQL 二进制确实存在(打包构建)才开内嵌 PG + HTTPS;
/// 开发期(cargo build,无随包 PG)保持外部 `DATABASE_URL` + HTTP,不强行内嵌。
fn apply_registry_env(app: &AppHandle, cmd: &mut Command) {
    // 链 RPC = 本机节点(打包/开发都设)。
    let ws_url = format!("ws://127.0.0.1:{}", crate::shared::rpc::current_rpc_port());
    cmd.env("CID_CHAIN_WS_URL", ws_url);

    // 中文注释:只有**打包形态**(随包 PostgreSQL 官方二进制存在)才用随包资源路径覆盖
    // china.sqlite / 前端产物 / 内嵌 PG / TLS;开发期(cargo tauri dev,无随包 PG)这些全
    // 由启动脚本 run.sh/clean-run.sh 的环境变量提供,registry_proc 不覆盖,避免拿占位资源
    // 路径把 dev 配置盖掉。
    let Some(res) = app.path().resource_dir().ok() else {
        return;
    };
    let pg_bin_dir = res.join("postgres").join(pg_os_subdir()).join("bin");
    let initdb = pg_bin_dir.join(if cfg!(windows) {
        "initdb.exe"
    } else {
        "initdb"
    });
    if !initdb.exists() {
        return;
    }
    cmd.env("CID_CHINA_DB", res.join("china.sqlite"));
    cmd.env(
        "REGISTRY_FRONTEND_DIST",
        res.join("registry-frontend").join("dist"),
    );
    cmd.env("CID_EMBEDDED_PG", "1");
    cmd.env("CID_ENABLE_TLS", "1");
    cmd.env("CID_PG_BIN_DIR", &pg_bin_dir);
    if let Ok(base) = crate::shared::security::app_data_dir(app) {
        apply_data_dir_env(&base, cmd);
    }
}

/// 数据目录派生的环境变量:PG 数据目录 / TLS 证书目录 / WAL 归档目录(与节点数据同根)。
fn apply_data_dir_env(base: &Path, cmd: &mut Command) {
    cmd.env("CID_PG_DATA_DIR", base.join("pgdata"));
    cmd.env("CID_TLS_DIR", base.join("registry-tls"));
    // 默认本地 WAL 归档;大市部署由运维把 CID_PG_WAL_ARCHIVE_DIR 指向 NAS(见 citizenchain/scripts/registry-{backup,restore}.sh)。
    cmd.env("CID_PG_WAL_ARCHIVE_DIR", base.join("pg-wal-archive"));
}

/// 启动 registry 子进程;已在运行则忽略。
///
/// 找不到二进制时只打印提示并返回,不影响节点桌面端启动
/// (开发期先 `cargo build -p registry` 生成二进制)。
pub fn start_registry(app: &AppHandle) {
    let mut guard = match REGISTRY_CHILD.lock() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("[registry] 获取子进程锁失败: {err}");
            return;
        }
    };
    if guard.is_some() {
        return;
    }
    let Some(binary) = registry_binary_path(app) else {
        eprintln!("[registry] 未找到 registry 二进制,跳过启动(开发期先 cargo build -p registry)");
        return;
    };
    let mut cmd = Command::new(&binary);
    apply_registry_env(app, &mut cmd);
    match cmd.spawn() {
        Ok(child) => {
            eprintln!("[registry] 已启动注册局子进程 pid={}", child.id());
            *guard = Some(child);
        }
        Err(err) => eprintln!("[registry] 启动注册局子进程失败: {err}"),
    }
}

/// 停止 registry 子进程(App 退出时调用)。
pub fn stop_registry() {
    let mut guard = match REGISTRY_CHILD.lock() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("[registry] 获取子进程锁失败: {err}");
            return;
        }
    };
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
        eprintln!("[registry] 已停止注册局子进程");
    }
}
