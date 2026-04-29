//! 节点桌面端访问 SFID 服务的统一配置。
//!
//! 约定：
//! - 显式传入 `SFID_BASE_URL` 时永远优先使用；
//! - 本地开发 debug 构建默认连接本机 SFID，方便和本地后端联调；
//! - 正式 release 构建默认连接 147 服务器上的 SFID 正式服务。

const SFID_BASE_URL_ENV: &str = "SFID_BASE_URL";
const DEV_SFID_BASE_URL: &str = "http://127.0.0.1:8899";
const PROD_SFID_BASE_URL: &str = "http://147.224.14.117:8899";

fn default_sfid_base_url() -> &'static str {
    if cfg!(debug_assertions) {
        DEV_SFID_BASE_URL
    } else {
        PROD_SFID_BASE_URL
    }
}

/// 返回节点桌面端调用 SFID HTTP API 使用的基地址。
///
/// 末尾斜杠会被清理，调用方可以稳定拼接 `/api/...` 路径。
pub(crate) fn sfid_base_url() -> String {
    std::env::var(SFID_BASE_URL_ENV)
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_sfid_base_url().to_string())
}
