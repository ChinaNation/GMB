// 机构查询：管理员列表、多签余额，通过 RPC 读取链上存储。

use crate::ui::shared::rpc;
use serde_json::Value;
use std::time::Duration;

use super::storage_keys;

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
use crate::ui::shared::constants::RPC_RESPONSE_LIMIT_SMALL;

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
pub fn fetch_admins(shenfen_id: &str) -> Result<Vec<String>, String> {
    let storage_key = storage_keys::admin_institutions_key(shenfen_id);
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(storage_key)]),
    )?;

    match result {
        Value::Null => Ok(Vec::new()),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            decode_admin_institution_admins(&data)
        }
        _ => Err("state_getStorage 返回格式无效".to_string()),
    }
}

/// 查询账户余额（返回 free 余额，单位为最小精度）。
pub fn fetch_balance(account_hex: &str) -> Result<Option<u128>, String> {
    fetch_balance_at(account_hex, None)
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

/// 解码 `AdminsChange::Institutions` 中的管理员列表。
///
/// AdminInstitution 布局前缀为 org:u8 + kind:enum(u8) + admins:BoundedVec<AccountId32>。
fn decode_admin_institution_admins(data: &[u8]) -> Result<Vec<String>, String> {
    if data.len() < 2 {
        return Ok(Vec::new());
    }
    let (count, bytes_read) = read_compact_u32(data, 2)?;
    let mut offset = 2 + bytes_read;
    let mut admins = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if offset + 32 > data.len() {
            break;
        }
        admins.push(hex::encode(&data[offset..offset + 32]));
        offset += 32;
    }
    Ok(admins)
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

/// 读取 SCALE Compact<u32>，返回 (值, 消耗字节数)。
fn read_compact_u32(data: &[u8], offset: usize) -> Result<(u32, usize), String> {
    if offset >= data.len() {
        return Err("Compact<u32> 数据不足".to_string());
    }
    let first = data[offset];
    let mode = first & 0x03;
    match mode {
        0 => Ok(((first >> 2) as u32, 1)),
        1 => {
            if offset + 2 > data.len() {
                return Err("Compact<u32> two-byte 数据不足".to_string());
            }
            let value = (((data[offset + 1] as u32) << 8) | first as u32) >> 2;
            Ok((value, 2))
        }
        2 => {
            if offset + 4 > data.len() {
                return Err("Compact<u32> four-byte 数据不足".to_string());
            }
            let value = ((data[offset + 3] as u32) << 24)
                | ((data[offset + 2] as u32) << 16)
                | ((data[offset + 1] as u32) << 8)
                | (data[offset] as u32);
            Ok((value >> 2, 4))
        }
        _ => Err("Compact<u32> big-integer 模式暂不支持".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_admin_institution_admins_empty() {
        assert!(decode_admin_institution_admins(&[]).unwrap().is_empty());
    }

    #[test]
    fn decode_admin_institution_admins_single() {
        // org=0, kind=0, Compact<u32> 1 = 0x04, 后跟 32 字节管理员公钥。
        let mut data = vec![0x00, 0x00, 0x04];
        data.extend_from_slice(&[0xAA; 32]);
        let admins = decode_admin_institution_admins(&data).unwrap();
        assert_eq!(admins.len(), 1);
        assert_eq!(admins[0], "aa".repeat(32));
    }

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

    #[test]
    fn read_compact_u32_single_byte() {
        // 值 13: (13 << 2) | 0 = 52 = 0x34
        let data = [0x34];
        let (val, len) = read_compact_u32(&data, 0).unwrap();
        assert_eq!(val, 13);
        assert_eq!(len, 1);
    }
}
