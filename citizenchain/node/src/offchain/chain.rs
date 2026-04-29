// 链上 ClearingBankNodes / NodePeerToInstitution storage 查询。
//
// pallet 名 = "OffchainTransaction"(runtime 注册的实例名,见 runtime/src/lib.rs:366)。
// storage 名 = "ClearingBankNodes" / "NodePeerToInstitution"。
// key 哈希器 = Blake2_128Concat(blake2_128 + raw_key)。
// key 数据 = SCALE 编码的 BoundedVec<u8, ConstU32<64>>:[compact_u32_len][bytes]。

use codec::{Decode, Encode};
use serde_json::Value;
use sp_core::ConstU32;
use sp_runtime::{AccountId32, BoundedVec};
use std::time::Duration;

use crate::ui::governance::signing::pubkey_to_ss58;
use crate::ui::governance::storage_keys;
use crate::ui::shared::{constants::RPC_RESPONSE_LIMIT_SMALL, rpc};

use super::types::ClearingBankNodeOnChainInfo;

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// 链上 `ClearingBankNodeInfo<AccountId, BlockNumber>` 在 node 端的 SCALE 镜像。
///
/// runtime 端定义见 [citizenchain/runtime/transaction/offchain-transaction/src/lib.rs:65]。
/// 字段顺序 / 边界长度必须与 runtime 严格一致,SCALE 解码才能成功。
#[derive(Decode, Encode)]
struct OnChainNodeInfo {
    peer_id: BoundedVec<u8, ConstU32<64>>,
    rpc_domain: BoundedVec<u8, ConstU32<128>>,
    rpc_port: u16,
    /// runtime 端 `BlockNumber = u32`(citizenchain Runtime 配置)。
    registered_at: u32,
    registered_by: AccountId32,
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    )
}

/// SCALE 编码 sfid_id 的 `BoundedVec<u8, ConstU32<64>>` 形式(用作 storage key data)。
///
/// 字段编码:`Compact<u32>(len)` + `bytes`。
fn encode_sfid_key_data(sfid_id: &str) -> Result<Vec<u8>, String> {
    let raw = sfid_id.as_bytes();
    if raw.is_empty() || raw.len() > 64 {
        return Err(format!("sfid_id 长度需在 1..=64 字节,实际:{}", raw.len()));
    }
    let bv: BoundedVec<u8, ConstU32<64>> = raw
        .to_vec()
        .try_into()
        .map_err(|_| "sfid_id 超出链上 BoundedVec<u8, 64>".to_string())?;
    Ok(bv.encode())
}

/// 构造 `OffchainTransaction::ClearingBankNodes(sfid_id)` 的 storage key(hex 含 0x 前缀)。
pub fn clearing_bank_nodes_key(sfid_id: &str) -> Result<String, String> {
    let key_data = encode_sfid_key_data(sfid_id)?;
    Ok(storage_keys::map_key(
        "OffchainTransaction",
        "ClearingBankNodes",
        &key_data,
    ))
}

/// 链上查询单条清算行节点声明信息。返回 None 表示该 sfid_id 尚未注册节点。
pub fn fetch_clearing_bank_node(
    sfid_id: &str,
) -> Result<Option<ClearingBankNodeOnChainInfo>, String> {
    let key = clearing_bank_nodes_key(sfid_id)?;
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;

    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
            let clean = hex_data.strip_prefix("0x").unwrap_or(&hex_data);
            let bytes = hex::decode(clean).map_err(|e| format!("storage hex 解码失败:{e}"))?;
            let info = OnChainNodeInfo::decode(&mut &bytes[..])
                .map_err(|e| format!("ClearingBankNodeInfo SCALE 解码失败:{e}"))?;

            let peer_id = String::from_utf8(info.peer_id.into_inner())
                .map_err(|_| "PeerId 编码非 UTF-8".to_string())?;
            let rpc_domain = String::from_utf8(info.rpc_domain.into_inner())
                .map_err(|_| "RPC 域名编码非 UTF-8".to_string())?;

            let registered_by_bytes: [u8; 32] = info.registered_by.into();
            let pubkey_hex = format!("0x{}", hex::encode(registered_by_bytes));
            let ss58 = pubkey_to_ss58(&registered_by_bytes)?;

            Ok(Some(ClearingBankNodeOnChainInfo {
                sfid_id: sfid_id.to_string(),
                peer_id,
                rpc_domain,
                rpc_port: info.rpc_port,
                registered_at: info.registered_at as u64,
                registered_by_pubkey_hex: pubkey_hex,
                registered_by_ss58: ss58,
            }))
        }
        _ => Err("state_getStorage 返回格式无效".to_string()),
    }
}

/// 用 `state_getKeysPaged` 估算当前已声明节点的总数,供网络面板"清算节点"指标。
///
/// 实现策略:用 `OffchainTransaction::ClearingBankNodes` 的 storage 前缀
/// (twox_128(pallet) || twox_128(storage))分页拉 key,每次最多 1000 条;
/// 一直拉到返回长度 < 1000 即代表全部取完。
pub fn count_clearing_bank_nodes() -> Result<u64, String> {
    let prefix = format!(
        "0x{}{}",
        hex::encode(storage_keys::twox_128(b"OffchainTransaction")),
        hex::encode(storage_keys::twox_128(b"ClearingBankNodes")),
    );

    const PAGE: u32 = 1000;
    let mut total: u64 = 0;
    let mut start_key: Option<String> = None;
    loop {
        let mut params = vec![
            Value::String(prefix.clone()),
            Value::Number(serde_json::Number::from(PAGE)),
        ];
        if let Some(s) = start_key.as_ref() {
            params.push(Value::String(s.clone()));
        }
        let result = rpc_post("state_getKeysPaged", Value::Array(params))?;
        let keys = result
            .as_array()
            .ok_or_else(|| "state_getKeysPaged 返回非数组".to_string())?;
        let n = keys.len();
        total = total.saturating_add(n as u64);
        if n < PAGE as usize {
            break;
        }
        start_key = keys.last().and_then(|v| v.as_str().map(|s| s.to_string()));
        if start_key.is_none() {
            break;
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_sfid_key_data_round_trip() {
        let raw = "GFR-LN001-CB0C-617776487-20260222";
        let encoded = encode_sfid_key_data(raw).unwrap();
        // Compact<u32> 长度前缀(单字节模式 raw.len() < 64)+ raw 字节
        assert_eq!(encoded[0], (raw.len() as u8) << 2);
        assert_eq!(&encoded[1..], raw.as_bytes());
    }

    #[test]
    fn empty_sfid_rejected() {
        let err = encode_sfid_key_data("").unwrap_err();
        assert!(err.contains("长度"));
    }

    #[test]
    fn over_long_sfid_rejected() {
        let s = "a".repeat(65);
        let err = encode_sfid_key_data(&s).unwrap_err();
        assert!(err.contains("长度"));
    }

    #[test]
    fn clearing_bank_nodes_key_starts_with_pallet_prefix() {
        let key = clearing_bank_nodes_key("SFR-12345-AAAA-678901234-20260101").unwrap();
        let pallet_hex = hex::encode(storage_keys::twox_128(b"OffchainTransaction"));
        let storage_hex = hex::encode(storage_keys::twox_128(b"ClearingBankNodes"));
        let prefix = format!("0x{pallet_hex}{storage_hex}");
        assert!(key.starts_with(&prefix), "实际 key:{key}");
    }
}
