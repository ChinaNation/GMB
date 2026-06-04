//! 链节点连接 URL 统一入口。
//!
//! 死规则：只用一个环境变量 `SFID_CHAIN_WS_URL`，格式 `ws://host:port`。
//! - subxt 推链/查询：直接用 WS URL
//! - HTTP JSON-RPC（余额查询等）：自动转 `http://host:port`（同一个端口）
//!
//! Substrate 的 `--rpc-port` 同时支持 HTTP 和 WS，所以同一个端口两种协议都能用。

/// 读取 `SFID_CHAIN_WS_URL` 环境变量，返回 WS URL（如 `ws://127.0.0.1:9944`）。
pub(crate) fn chain_ws_url() -> Result<String, String> {
    std::env::var("SFID_CHAIN_WS_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "SFID_CHAIN_WS_URL not configured".to_string())
}

/// 从 WS URL 转换为 HTTP URL（同端口，Substrate RPC 同时支持两种协议）。
pub(crate) fn chain_http_url() -> Result<String, String> {
    let ws = chain_ws_url()?;
    if let Some(rest) = ws.strip_prefix("ws://") {
        return Ok(format!("http://{rest}"));
    }
    if let Some(rest) = ws.strip_prefix("wss://") {
        return Ok(format!("https://{rest}"));
    }
    Ok(ws)
}
