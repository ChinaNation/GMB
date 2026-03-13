// 共享 RPC 请求封装，供 home/settings/mining/network 复用同一套连接池与安全限制。
use serde_json::Value;
use std::{
    io::Read,
    sync::{Mutex, OnceLock},
    time::Duration,
};

const RPC_HTTP_URL: &str = "http://127.0.0.1:9944/";
const RPC_CONNECT_TIMEOUT_MS: u64 = 2500;
const GENESIS_HASH_TIMEOUT: Duration = Duration::from_secs(3);
const GENESIS_HASH_MAX_BYTES: u64 = 1024;

static RPC_HTTP_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
static RPC_HTTP_CLIENT_INIT_LOCK: Mutex<()> = Mutex::new(());
static CACHED_GENESIS_HASH: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn rpc_http_client() -> Result<&'static reqwest::blocking::Client, String> {
    if let Some(client) = RPC_HTTP_CLIENT.get() {
        return Ok(client);
    }
    let _guard = RPC_HTTP_CLIENT_INIT_LOCK.lock().unwrap_or_else(|err| {
        eprintln!("RPC_HTTP_CLIENT_INIT_LOCK poisoned; continuing with recovered lock");
        err.into_inner()
    });
    if let Some(client) = RPC_HTTP_CLIENT.get() {
        return Ok(client);
    }
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_millis(RPC_CONNECT_TIMEOUT_MS))
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("create RPC HTTP client failed: {e}"))?;
    let _ = RPC_HTTP_CLIENT.set(client);
    RPC_HTTP_CLIENT
        .get()
        .ok_or_else(|| "create RPC HTTP client failed: unset client".to_string())
}

pub(crate) fn rpc_post(
    method: &str,
    params: Value,
    request_timeout: Duration,
    max_response_bytes: u64,
) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let client = rpc_http_client()?;
    let response = client
        .post(RPC_HTTP_URL)
        .timeout(request_timeout)
        .json(&payload)
        .send()
        .map_err(|e| format!("RPC 请求失败: {e}"))?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("RPC HTTP 状态异常: {}", response.status()));
    }
    if let Some(content_length) = response.content_length() {
        if content_length > max_response_bytes {
            return Err(format!(
                "RPC 响应体过大: {} bytes (limit {})",
                content_length, max_response_bytes
            ));
        }
    }

    let mut limited_reader = response.take(max_response_bytes.saturating_add(1));
    let mut body: Vec<u8> = Vec::new();
    limited_reader
        .read_to_end(&mut body)
        .map_err(|e| format!("RPC 读取失败: {e}"))?;
    if body.len() as u64 > max_response_bytes {
        return Err(format!("RPC 响应体超过限制: {} bytes", max_response_bytes));
    }

    let json: Value =
        serde_json::from_slice(&body).map_err(|e| format!("RPC JSON 解析失败: {e}"))?;
    if let Some(err) = json.get("error") {
        return Err(format!("RPC 返回错误: {err}"));
    }
    Ok(json.get("result").cloned().unwrap_or(Value::Null))
}

/// 获取本地 RPC 节点的 genesis hash 并缓存。
/// 首次连接时从 `chain_getBlockHash(0)` 获取并存储，后续直接返回缓存。
pub(crate) fn cached_genesis_hash() -> Result<String, String> {
    let mutex = CACHED_GENESIS_HASH.get_or_init(|| Mutex::new(None));
    let mut guard = mutex
        .lock()
        .map_err(|_| "genesis hash cache lock poisoned".to_string())?;
    if let Some(ref hash) = *guard {
        return Ok(hash.clone());
    }
    let hash = rpc_post(
        "chain_getBlockHash",
        Value::Array(vec![Value::Number(0.into())]),
        GENESIS_HASH_TIMEOUT,
        GENESIS_HASH_MAX_BYTES,
    )?;
    let hash_str = hash
        .as_str()
        .ok_or_else(|| "chain_getBlockHash(0) 返回值不是字符串".to_string())?
        .to_string();
    if hash_str.is_empty() {
        return Err("chain_getBlockHash(0) 返回空哈希".to_string());
    }
    *guard = Some(hash_str.clone());
    Ok(hash_str)
}

/// 校验当前连接的 RPC 节点 genesis hash 是否与首次缓存的一致。
/// 如果尚无缓存，首次调用会自动缓存。
pub(crate) fn verify_genesis_hash() -> Result<(), String> {
    let expected = cached_genesis_hash()?;
    let current = rpc_post(
        "chain_getBlockHash",
        Value::Array(vec![Value::Number(0.into())]),
        GENESIS_HASH_TIMEOUT,
        GENESIS_HASH_MAX_BYTES,
    )?;
    let current_str = current
        .as_str()
        .ok_or_else(|| "chain_getBlockHash(0) 返回值不是字符串".to_string())?;
    if current_str != expected {
        return Err(format!(
            "RPC 节点 genesis hash 不匹配（期望 {expected}，实际 {current_str}），可能连接到了错误的链"
        ));
    }
    Ok(())
}

/// 节点停止后清除 genesis hash 缓存，以便下次启动时重新校验。
pub(crate) fn clear_genesis_hash_cache() {
    if let Some(mutex) = CACHED_GENESIS_HASH.get() {
        if let Ok(mut guard) = mutex.lock() {
            *guard = None;
        }
    }
}
