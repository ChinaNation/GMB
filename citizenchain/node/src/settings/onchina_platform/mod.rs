//! 设置页“链上中国平台”手动启动入口。
//!
//! 中文注释:节点软件默认只启动区块链节点;链上中国平台服务占用数据库、HTTPS 和
//! 浏览器管理后台资源,因此必须由用户在设置页二次确认后显式启动。

use serde::Serialize;
use tauri::AppHandle;

use crate::shared::security;

/// 节点安装后管理员在局域网浏览器中访问的唯一固定入口。
const ONCHINA_PLATFORM_URL: &str = "https://onchina.local:8964";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnChinaPlatformState {
    running: bool,
    url: &'static str,
}

fn current_state() -> OnChinaPlatformState {
    OnChinaPlatformState {
        running: crate::onchina_proc::is_onchina_running(),
        url: ONCHINA_PLATFORM_URL,
    }
}

#[tauri::command]
pub fn get_onchina_platform() -> Result<OnChinaPlatformState, String> {
    Ok(current_state())
}

#[tauri::command]
pub fn start_onchina_platform(app: AppHandle) -> Result<OnChinaPlatformState, String> {
    if let Err(err) = security::append_audit_log(&app, "start_onchina_platform", "attempt") {
        eprintln!("[审计] start_onchina_platform attempt 日志写入失败:{err}");
    }
    crate::onchina_proc::start_onchina(&app)?;
    if let Err(err) = security::append_audit_log(&app, "start_onchina_platform", "success") {
        eprintln!("[审计] start_onchina_platform success 日志写入失败:{err}");
    }
    Ok(current_state())
}
