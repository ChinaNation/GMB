//! 节点桌面端访问 OnChina 服务的统一配置。
//!
//! 约定：
//! - 显式传入 `ONCHINA_BASE_URL` 时永远优先使用；
//! - 未显式配置时固定连接局域网统一入口 `https://onchina.local:8964`。

const ONCHINA_BASE_URL_ENV: &str = "ONCHINA_BASE_URL";
const DEFAULT_ONCHINA_BASE_URL: &str = "https://onchina.local:8964";

/// 返回节点桌面端调用 OnChina HTTP API 使用的基地址。
///
/// 末尾斜杠会被清理，调用方可以稳定拼接 `/api/...` 路径。
pub(crate) fn onchina_base_url() -> String {
    std::env::var(ONCHINA_BASE_URL_ENV)
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_ONCHINA_BASE_URL.to_string())
}

/// 本机 OnChina 默认使用自签证书;节点桌面端只在固定本地入口上放宽证书校验。
pub(crate) fn accepts_local_self_signed_tls(base_url: &str) -> bool {
    base_url.starts_with("https://onchina.local:")
        || base_url.starts_with("https://127.0.0.1:")
        || base_url.starts_with("https://localhost:")
}
