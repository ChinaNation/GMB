// RPC 子模块：节点 RPC 调用、链同步状态查询。

use crate::shared::{constants::EXPECTED_SS58_PREFIX, rpc};
use serde::Serialize;
use serde_json::Value;
use std::{thread, time::Duration};
use tauri::AppHandle;

use super::identity::current_status;

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_RPC_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const RPC_RETRY_COUNT: usize = 3;

pub(super) fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    let mut last_err = String::new();
    for attempt in 0..RPC_RETRY_COUNT {
        match rpc::rpc_post(
            method,
            params.clone(),
            RPC_REQUEST_TIMEOUT,
            MAX_RPC_RESPONSE_BYTES,
        ) {
            Ok(v) => return Ok(v),
            Err(err) => {
                last_err = err;
                if attempt + 1 < RPC_RETRY_COUNT {
                    thread::sleep(Duration::from_millis(250));
                }
            }
        }
    }
    Err(last_err)
}

pub(super) fn is_expected_rpc_node() -> bool {
    let Ok(properties) = rpc_post("system_properties", Value::Array(vec![])) else {
        return false;
    };
    let ss58 = properties
        .get("ss58Format")
        .and_then(|v| {
            if let Some(raw) = v.as_u64() {
                Some(raw)
            } else {
                v.as_str().and_then(|s| s.parse::<u64>().ok())
            }
        })
        .unwrap_or(0);
    if ss58 != EXPECTED_SS58_PREFIX {
        return false;
    }

    let has_name = rpc_post("system_name", Value::Array(vec![]))
        .ok()
        .and_then(|v| v.as_str().map(|s| !s.trim().is_empty()))
        .unwrap_or(false);
    if !has_name {
        return false;
    }

    // 补充 genesis hash 校验：首次连接缓存，后续比对。
    if rpc::verify_genesis_hash().is_err() {
        return false;
    }

    true
}

fn hex_to_u64(hex: &str) -> Option<u64> {
    let trimmed = hex.strip_prefix("0x")?;
    u64::from_str_radix(trimmed, 16).ok()
}

fn header_block_height(header: &Value) -> Option<u64> {
    header
        .get("number")
        .and_then(Value::as_str)
        .and_then(hex_to_u64)
}

fn finalized_block_height() -> Option<u64> {
    let hash = rpc_post("chain_getFinalizedHead", Value::Array(vec![]))
        .ok()?
        .as_str()?
        .to_string();
    let header = rpc_post("chain_getHeader", Value::Array(vec![Value::String(hash)])).ok()?;
    header_block_height(&header)
}

fn syncing_flag() -> Option<bool> {
    let health = rpc_post("system_health", Value::Array(vec![])).ok()?;
    if let Some(v) = health.get("isSyncing") {
        if let Some(b) = v.as_bool() {
            return Some(b);
        }
        if let Some(s) = v.as_str() {
            let lowered = s.trim().to_ascii_lowercase();
            if lowered == "true" {
                return Some(true);
            }
            if lowered == "false" {
                return Some(false);
            }
        }
    }
    None
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// 首页展示的链同步状态。
pub struct ChainStatus {
    pub block_height: Option<u64>,
    pub finalized_height: Option<u64>,
    pub syncing: Option<bool>,
}

fn get_chain_status_sync(app: AppHandle) -> Result<ChainStatus, String> {
    if !current_status(&app)?.running {
        return Ok(ChainStatus {
            block_height: None,
            finalized_height: None,
            syncing: None,
        });
    }

    let block_height = rpc_post("chain_getHeader", Value::Array(vec![]))
        .ok()
        .as_ref()
        .and_then(header_block_height);
    let finalized_height = finalized_block_height();
    let syncing = syncing_flag();

    Ok(ChainStatus {
        block_height,
        finalized_height,
        syncing,
    })
}

#[tauri::command]
pub async fn get_chain_status(app: AppHandle) -> Result<ChainStatus, String> {
    super::join_blocking_task(
        "get_chain_status",
        tauri::async_runtime::spawn_blocking(move || get_chain_status_sync(app)),
    )
    .await
}
