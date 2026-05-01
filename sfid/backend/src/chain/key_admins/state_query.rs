//! 通用 chain JSON-RPC 查询 helper。
//!
//! 给 `chain::balance` / `chain::key_admins` 内部其他模块共用。
//! 主请求路径走 HTTP JSON-RPC(同 `chain::url::chain_http_url`)。

use reqwest::Client as HttpClient;
use serde_json::json;

use crate::chain::url::chain_http_url;

/// 暴露给 chain 内其他模块按需调用 `state_getStorage`。
pub(crate) async fn call_chain_state_get_storage(
    storage_key_hex: &str,
) -> Result<Option<String>, String> {
    let result = chain_rpc_call("state_getStorage", json!([storage_key_hex])).await?;
    Ok(result.as_str().map(str::to_string))
}

/// 任意 substrate JSON-RPC 调用,返回 `result` 字段(JSON Value)。
pub(crate) async fn chain_rpc_call(
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = HttpClient::new();
    let url = chain_http_url()?;
    let response = client
        .post(url)
        .json(&json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .map_err(|e| format!("chain rpc request failed: {e}"))?;
    let status = response.status();
    let payload = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("decode chain rpc response failed: {e}"))?;
    if !status.is_success() {
        return Err(format!("chain rpc returned status {status}"));
    }
    if let Some(err) = payload.get("error") {
        return Err(format!("chain rpc returned error: {err}"));
    }
    Ok(payload["result"].clone())
}
