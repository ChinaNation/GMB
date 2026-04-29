// 清算行 register/update/unregister 三个 extrinsic 的 QR 签名请求构建 +
// call_data 重建。复用 `governance::signing::build_sign_request_from_call_data`
// 与 `verify_and_submit`,只在本模块构造 call_data 与 display 字段。
//
// pallet_index = 21(OffchainTransactionPos,见 runtime/src/lib.rs:366)
// call_index:
//   50 = register_clearing_bank(sfid_id, peer_id, rpc_domain, rpc_port)
//   51 = update_clearing_bank_endpoint(sfid_id, new_domain, new_port)
//   52 = unregister_clearing_bank(sfid_id)
//
// 入参 SCALE 编码:
//   BoundedVec<u8, ConstU32<N>> 等价 Vec<u8> = Compact<u32>(len) || bytes
//   u16 = 2 字节 little-endian

use crate::ui::governance::signing::{
    build_sign_request_from_call_data, encode_compact_u32_pub, VoteSignRequestResult,
};

/// pallet_index = 21(OffchainTransactionPos)。
const PALLET_INDEX: u8 = 21;

const CALL_REGISTER: u8 = 50;
const CALL_UPDATE_ENDPOINT: u8 = 51;
const CALL_UNREGISTER: u8 = 52;

/// 校验 0x 前缀小写 hex 公钥(64 hex 字符)。返回 (clean_hex, raw_bytes)。
fn parse_pubkey(pubkey_hex: &str) -> Result<(String, Vec<u8>), String> {
    let clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效,应为 64 位十六进制".to_string());
    }
    let bytes = hex::decode(&clean).map_err(|e| format!("公钥解码失败:{e}"))?;
    Ok((clean, bytes))
}

/// 把 sfid_id / peer_id / domain 等可变长字段按 SCALE Vec<u8> 编码:
/// `Compact<u32>(len) || raw_bytes`。
fn encode_bytes_with_len(raw: &[u8]) -> Vec<u8> {
    let mut out = encode_compact_u32_pub(raw.len() as u32);
    out.extend_from_slice(raw);
    out
}

/// 构造 register_clearing_bank 的 call_data。
pub fn build_register_call_data(
    sfid_id: &str,
    peer_id: &str,
    rpc_domain: &str,
    rpc_port: u16,
) -> Result<Vec<u8>, String> {
    if sfid_id.is_empty() || sfid_id.len() > 64 {
        return Err("sfid_id 长度需在 1..=64".to_string());
    }
    if peer_id.is_empty() || peer_id.len() > 64 {
        return Err("peer_id 长度需在 1..=64".to_string());
    }
    if rpc_domain.is_empty() || rpc_domain.len() > 128 {
        return Err("rpc_domain 长度需在 1..=128".to_string());
    }
    if rpc_port < 1024 {
        return Err("rpc_port 必须 >= 1024".to_string());
    }

    let mut call =
        Vec::with_capacity(2 + 1 + sfid_id.len() + 1 + peer_id.len() + 1 + rpc_domain.len() + 2);
    call.push(PALLET_INDEX);
    call.push(CALL_REGISTER);
    call.extend_from_slice(&encode_bytes_with_len(sfid_id.as_bytes()));
    call.extend_from_slice(&encode_bytes_with_len(peer_id.as_bytes()));
    call.extend_from_slice(&encode_bytes_with_len(rpc_domain.as_bytes()));
    call.extend_from_slice(&rpc_port.to_le_bytes());
    Ok(call)
}

/// 构造 update_clearing_bank_endpoint 的 call_data。
pub fn build_update_endpoint_call_data(
    sfid_id: &str,
    new_domain: &str,
    new_port: u16,
) -> Result<Vec<u8>, String> {
    if sfid_id.is_empty() || sfid_id.len() > 64 {
        return Err("sfid_id 长度需在 1..=64".to_string());
    }
    if new_domain.is_empty() || new_domain.len() > 128 {
        return Err("rpc_domain 长度需在 1..=128".to_string());
    }
    if new_port < 1024 {
        return Err("rpc_port 必须 >= 1024".to_string());
    }
    let mut call = Vec::with_capacity(2 + 1 + sfid_id.len() + 1 + new_domain.len() + 2);
    call.push(PALLET_INDEX);
    call.push(CALL_UPDATE_ENDPOINT);
    call.extend_from_slice(&encode_bytes_with_len(sfid_id.as_bytes()));
    call.extend_from_slice(&encode_bytes_with_len(new_domain.as_bytes()));
    call.extend_from_slice(&new_port.to_le_bytes());
    Ok(call)
}

/// 构造 unregister_clearing_bank 的 call_data。
pub fn build_unregister_call_data(sfid_id: &str) -> Result<Vec<u8>, String> {
    if sfid_id.is_empty() || sfid_id.len() > 64 {
        return Err("sfid_id 长度需在 1..=64".to_string());
    }
    let mut call = Vec::with_capacity(2 + 1 + sfid_id.len());
    call.push(PALLET_INDEX);
    call.push(CALL_UNREGISTER);
    call.extend_from_slice(&encode_bytes_with_len(sfid_id.as_bytes()));
    Ok(call)
}

/// register_clearing_bank QR 签名请求。
pub fn build_register_sign_request(
    pubkey_hex: &str,
    sfid_id: &str,
    peer_id: &str,
    rpc_domain: &str,
    rpc_port: u16,
) -> Result<VoteSignRequestResult, String> {
    let (clean, bytes) = parse_pubkey(pubkey_hex)?;
    let call_data = build_register_call_data(sfid_id, peer_id, rpc_domain, rpc_port)?;
    let summary = format!("声明清算行节点 {sfid_id} @ {rpc_domain}:{rpc_port}");
    // display.fields key/value 必须与 wumin PayloadDecoder 输出 1:1 对齐(Step 3 加 decoder)。
    // 当前 wumin 尚未支持本 action,Step 2 dev 链端到端验证用 dry-run + 黄色盲签兜底。
    let fields = serde_json::json!([
        { "key": "sfid_id", "label": "机构身份码", "value": sfid_id },
        { "key": "peer_id", "label": "节点 PeerId", "value": peer_id },
        { "key": "rpc_domain", "label": "RPC 域名", "value": rpc_domain },
        { "key": "rpc_port", "label": "RPC 端口", "value": rpc_port.to_string() },
    ]);
    build_sign_request_from_call_data(
        &clean,
        &bytes,
        &call_data,
        "register_clearing_bank",
        &summary,
        &fields,
    )
}

/// update_clearing_bank_endpoint QR 签名请求。
pub fn build_update_endpoint_sign_request(
    pubkey_hex: &str,
    sfid_id: &str,
    new_domain: &str,
    new_port: u16,
) -> Result<VoteSignRequestResult, String> {
    let (clean, bytes) = parse_pubkey(pubkey_hex)?;
    let call_data = build_update_endpoint_call_data(sfid_id, new_domain, new_port)?;
    let summary = format!("更新清算行 {sfid_id} 端点 → {new_domain}:{new_port}");
    let fields = serde_json::json!([
        { "key": "sfid_id", "label": "机构身份码", "value": sfid_id },
        { "key": "new_domain", "label": "新域名", "value": new_domain },
        { "key": "new_port", "label": "新端口", "value": new_port.to_string() },
    ]);
    build_sign_request_from_call_data(
        &clean,
        &bytes,
        &call_data,
        "update_clearing_bank_endpoint",
        &summary,
        &fields,
    )
}

/// unregister_clearing_bank QR 签名请求。
pub fn build_unregister_sign_request(
    pubkey_hex: &str,
    sfid_id: &str,
) -> Result<VoteSignRequestResult, String> {
    let (clean, bytes) = parse_pubkey(pubkey_hex)?;
    let call_data = build_unregister_call_data(sfid_id)?;
    let summary = format!("注销清算行节点 {sfid_id}");
    let fields = serde_json::json!([
        { "key": "sfid_id", "label": "机构身份码", "value": sfid_id },
    ]);
    build_sign_request_from_call_data(
        &clean,
        &bytes,
        &call_data,
        "unregister_clearing_bank",
        &summary,
        &fields,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_call_data_starts_with_pallet_and_call() {
        let cd = build_register_call_data(
            "SFR-12345-AAAA-678901234-20260101",
            "12D3KooWAbcDef0123456789012345678901234567890123456",
            "rpc.example.com",
            9944,
        )
        .unwrap();
        assert_eq!(cd[0], PALLET_INDEX);
        assert_eq!(cd[1], CALL_REGISTER);
        // tail 应为 little-endian u16 端口
        let n = cd.len();
        assert_eq!(u16::from_le_bytes([cd[n - 2], cd[n - 1]]), 9944);
    }

    #[test]
    fn register_call_rejects_empty_sfid() {
        let err = build_register_call_data(
            "",
            "12D3KooWaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "x.com",
            9944,
        )
        .unwrap_err();
        assert!(err.contains("sfid_id"));
    }

    #[test]
    fn register_call_rejects_low_port() {
        let err = build_register_call_data(
            "S",
            "12D3KooWaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "x.com",
            80,
        )
        .unwrap_err();
        assert!(err.contains("rpc_port"));
    }

    #[test]
    fn unregister_call_only_has_sfid() {
        let cd = build_unregister_call_data("SFR-X").unwrap();
        assert_eq!(cd[0], PALLET_INDEX);
        assert_eq!(cd[1], CALL_UNREGISTER);
        // 单字节模式 compact: len << 2
        assert_eq!(cd[2], (5u8) << 2);
        assert_eq!(&cd[3..], b"SFR-X");
    }

    #[test]
    fn parse_pubkey_normalizes_uppercase_and_strips_prefix() {
        let (clean, bytes) =
            parse_pubkey("0xAABBCCDDEEFF00112233445566778899AABBCCDDEEFF00112233445566778899")
                .unwrap();
        assert_eq!(clean.len(), 64);
        assert!(clean
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn parse_pubkey_rejects_bad_length() {
        assert!(parse_pubkey("0x").is_err());
        assert!(parse_pubkey("xyz").is_err());
    }
}
