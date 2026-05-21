// 机构查询：管理员列表、多签余额，通过 RPC 读取链上存储。

use crate::shared::rpc;
use serde_json::Value;
use std::time::Duration;

use super::{admins_change, storage_keys};

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
use crate::shared::constants::RPC_RESPONSE_LIMIT_SMALL;

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    )
}

/// 查询指定机构的管理员公钥列表。
/// 返回不含 0x 前缀的小写 hex 公钥列表。
pub fn fetch_admins(sfid_number: &str) -> Result<Vec<String>, String> {
    admins_change::storage::fetch_admins_by_sfid_number(sfid_number)
}

/// 查询 finalized 块上的账户余额（返回 free 余额，单位为最小精度）。
pub fn fetch_balance(account_hex: &str) -> Result<Option<u128>, String> {
    let hash = fetch_finalized_head()?;
    fetch_balance_at(account_hex, Some(&hash))
}

/// 查询指定 finalized 块上的账户余额（返回 free 余额，单位为最小精度）。
pub fn fetch_balance_at(
    account_hex: &str,
    block_hash: Option<&str>,
) -> Result<Option<u128>, String> {
    let storage_key = storage_keys::system_account_key(account_hex)?;
    let mut params = vec![Value::String(storage_key)];
    if let Some(hash) = block_hash {
        params.push(Value::String(hash.to_string()));
    }
    let result = rpc_post("state_getStorage", Value::Array(params))?;

    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            decode_account_balance(&data)
        }
        _ => Err("state_getStorage 返回格式无效".to_string()),
    }
}

/// 查询最新 finalized 区块哈希。
pub fn fetch_finalized_head() -> Result<String, String> {
    let result = rpc_post("chain_getFinalizedHead", Value::Array(vec![]))?;
    match result {
        Value::String(hash) => Ok(hash),
        _ => Err("chain_getFinalizedHead 返回格式无效".to_string()),
    }
}

/// 解码 0x 前缀的 hex 存储数据为字节。
fn decode_hex_storage(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).map_err(|e| format!("hex 解码失败: {e}"))
}

/// 解码 AccountInfo 中的 free 余额。
/// AccountInfo 布局（SCALE）：
///   nonce: u32 (4 bytes)
///   consumers: u32 (4 bytes)
///   providers: u32 (4 bytes)
///   sufficients: u32 (4 bytes)
///   data.free: u128 (16 bytes)  ← 目标
///   data.reserved: u128 (16 bytes)
///   data.frozen: u128 (16 bytes)
fn decode_account_balance(data: &[u8]) -> Result<Option<u128>, String> {
    // nonce(4) + consumers(4) + providers(4) + sufficients(4) = 16 字节偏移
    let free_offset = 16;
    if data.len() < free_offset + 16 {
        return Ok(None);
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&data[free_offset..free_offset + 16]);
    Ok(Some(u128::from_le_bytes(buf)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_account_balance_basic() {
        // 16 字节头 + 16 字节 free 余额
        let mut data = vec![0u8; 16]; // nonce+consumers+providers+sufficients
        let balance: u128 = 1_000_000;
        data.extend_from_slice(&balance.to_le_bytes());
        data.extend_from_slice(&[0u8; 32]); // reserved + frozen
        let result = decode_account_balance(&data).unwrap();
        assert_eq!(result, Some(1_000_000));
    }
}
