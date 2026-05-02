// 清算行 register/update/unregister 三个 extrinsic 的 QR 签名请求构建 +
// call_data 重建。复用 `governance::signing::build_sign_request_from_call_data`
// 与 `verify_and_submit`,只在本模块构造 call_data 与 display 字段。
//
// pallet_index = 21(OffchainTransaction,见 runtime/src/lib.rs:366)
// call_index:
//   50 = register_clearing_bank(sfid_id, peer_id, rpc_domain, rpc_port)
//   51 = update_clearing_bank_endpoint(sfid_id, new_domain, new_port)
//   52 = unregister_clearing_bank(sfid_id)
//
// 入参 SCALE 编码:
//   BoundedVec<u8, ConstU32<N>> 等价 Vec<u8> = Compact<u32>(len) || bytes
//   u16 = 2 字节 little-endian

use crate::governance::signing::{
    build_sign_request_from_call_data, encode_compact_u32_pub, VoteSignRequestResult,
};

/// pallet_index = 21(OffchainTransaction)。
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

// ─── 创建机构多签:propose_create_institution(pallet=17, call_index=5) ──────

const DUOQIAN_PALLET_INDEX: u8 = 17;
const CALL_PROPOSE_CREATE_INSTITUTION: u8 = 5;

/// 创建机构时每个账户的初始资金条目(已派生 hex 地址 / 字符串金额"分")。
#[derive(Debug, Clone)]
pub struct InitialAccountInput {
    pub account_name: String,
    /// 初始资金,单位"分"(u128 字符串,避免 JS BigInt 跨语言传输精度问题)。
    pub amount_fen: u128,
}

/// 把 32 字节公钥 hex 转 Vec<u8>(供管理员列表 / SignatureBoundedVec 编码用)。
fn parse_account32(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str
        .strip_prefix("0x")
        .unwrap_or(hex_str)
        .to_ascii_lowercase();
    if clean.len() != 64 {
        return Err("account 公钥必须 32 字节(64 hex 字符)".to_string());
    }
    hex::decode(&clean).map_err(|e| format!("公钥解码失败:{e}"))
}

/// SCALE 编码 `Option<BoundedVec<u8>>`:`0x00`(None)或 `0x01 + Compact(len) + bytes`(Some)。
fn encode_optional_bytes(value: Option<&[u8]>) -> Vec<u8> {
    match value {
        None => vec![0u8],
        Some(b) => {
            let mut out = Vec::with_capacity(1 + 1 + b.len());
            out.push(1u8);
            out.extend_from_slice(&encode_bytes_with_len(b));
            out
        }
    }
}

/// SCALE 编码 u128 little-endian(16 字节)。
fn encode_u128_le(v: u128) -> [u8; 16] {
    v.to_le_bytes()
}

/// 构造 `propose_create_institution`(pallet=17, call=5)的 call_data。
///
/// 入参顺序与 [`citizenchain/runtime/transaction/duoqian-manage/src/lib.rs::propose_create_institution`]
/// 严格一致(13 个字段)。任一字段顺序变更必须同步改本函数。
#[allow(clippy::too_many_arguments)]
pub fn build_propose_create_institution_call_data(
    sfid_id: &str,
    institution_name: &str,
    accounts: &[InitialAccountInput],
    admin_count: u32,
    admin_pubkeys: &[String],
    threshold: u32,
    register_nonce: &str,
    signature_hex: &str,
    signing_province: &str,
    a3: &str,
    sub_type: Option<&str>,
    parent_sfid_id: Option<&str>,
) -> Result<Vec<u8>, String> {
    if sfid_id.is_empty() || sfid_id.len() > 64 {
        return Err("sfid_id 长度需在 1..=64".to_string());
    }
    if institution_name.is_empty() || institution_name.len() > 128 {
        return Err("institution_name 长度需在 1..=128".to_string());
    }
    if accounts.is_empty() {
        return Err("accounts 至少 1 项(主账户)".to_string());
    }
    if admin_count < 2 {
        return Err("admin_count 必须 >= 2".to_string());
    }
    if admin_pubkeys.len() as u32 != admin_count {
        return Err(format!(
            "admin_count={admin_count} 与 admin_pubkeys.len={} 不一致",
            admin_pubkeys.len()
        ));
    }
    let min_threshold = std::cmp::max(2, admin_count.saturating_add(1) / 2);
    if threshold < min_threshold || threshold > admin_count {
        return Err(format!(
            "threshold 范围必须在 {min_threshold}..={admin_count}"
        ));
    }
    let signature_bytes = hex::decode(signature_hex.strip_prefix("0x").unwrap_or(signature_hex))
        .map_err(|e| format!("signature hex 解码失败:{e}"))?;
    if signature_bytes.len() != 64 {
        return Err(format!(
            "signature 必须 64 字节,实际 {} 字节",
            signature_bytes.len()
        ));
    }
    if signing_province.is_empty() {
        return Err("signing_province 不可为空".to_string());
    }

    let mut call: Vec<u8> = Vec::with_capacity(512);
    call.push(DUOQIAN_PALLET_INDEX);
    call.push(CALL_PROPOSE_CREATE_INSTITUTION);

    // 1. sfid_id: BoundedVec<u8>
    call.extend_from_slice(&encode_bytes_with_len(sfid_id.as_bytes()));
    // 2. institution_name: BoundedVec<u8>
    call.extend_from_slice(&encode_bytes_with_len(institution_name.as_bytes()));
    // 3. accounts: BoundedVec<InstitutionInitialAccount> = Compact(N) + N × (account_name + amount)
    call.extend_from_slice(&encode_compact_u32_pub(accounts.len() as u32));
    for acc in accounts {
        if acc.account_name.is_empty() || acc.account_name.len() > 128 {
            return Err(format!(
                "account_name 长度需在 1..=128:{}",
                acc.account_name
            ));
        }
        call.extend_from_slice(&encode_bytes_with_len(acc.account_name.as_bytes()));
        call.extend_from_slice(&encode_u128_le(acc.amount_fen));
    }
    // 4. admin_count: u32 LE
    call.extend_from_slice(&admin_count.to_le_bytes());
    // 5. duoqian_admins: BoundedVec<AccountId32> = Compact(N) + N × 32B
    call.extend_from_slice(&encode_compact_u32_pub(admin_pubkeys.len() as u32));
    for pk in admin_pubkeys {
        let bytes = parse_account32(pk)?;
        call.extend_from_slice(&bytes);
    }
    // 6. threshold: u32 LE
    call.extend_from_slice(&threshold.to_le_bytes());
    // 7. register_nonce: BoundedVec<u8>
    call.extend_from_slice(&encode_bytes_with_len(register_nonce.as_bytes()));
    // 8. signature: BoundedVec<u8>(64 字节)
    call.extend_from_slice(&encode_bytes_with_len(&signature_bytes));
    // 9. signing_province: Option<Vec<u8>>(本流程必填,链端要查 ShengSigningPubkey[province])
    call.extend_from_slice(&encode_optional_bytes(Some(signing_province.as_bytes())));
    // 10. a3: BoundedVec<u8>
    call.extend_from_slice(&encode_bytes_with_len(a3.as_bytes()));
    // 11. sub_type: Option<BoundedVec<u8>>
    call.extend_from_slice(&encode_optional_bytes(sub_type.map(|s| s.as_bytes())));
    // 12. parent_sfid_id: Option<BoundedVec<u8>>
    call.extend_from_slice(&encode_optional_bytes(parent_sfid_id.map(|s| s.as_bytes())));

    Ok(call)
}

/// `propose_create_institution` 的冷钱包 QR 签名请求。
///
/// SignDisplay 字段 key/value 与 wumin PayloadDecoder 输出 1:1 对齐(decoder
/// 待 wumin 加分支,本任务作为 follow-up 任务卡;期间扫到本 action 在新两色识别
/// 模型下会 🔴 红色拒签,需要 wumin 端补 decoder 后才能完整跑通。
#[allow(clippy::too_many_arguments)]
pub fn build_propose_create_institution_sign_request(
    pubkey_hex: &str,
    sfid_id: &str,
    institution_name: &str,
    accounts: &[InitialAccountInput],
    admin_count: u32,
    admin_pubkeys: &[String],
    threshold: u32,
    register_nonce: &str,
    signature_hex: &str,
    signing_province: &str,
    a3: &str,
    sub_type: Option<&str>,
    parent_sfid_id: Option<&str>,
) -> Result<VoteSignRequestResult, String> {
    let (clean, bytes) = parse_pubkey(pubkey_hex)?;
    let call_data = build_propose_create_institution_call_data(
        sfid_id,
        institution_name,
        accounts,
        admin_count,
        admin_pubkeys,
        threshold,
        register_nonce,
        signature_hex,
        signing_province,
        a3,
        sub_type,
        parent_sfid_id,
    )?;
    let total_amount_fen: u128 = accounts.iter().map(|a| a.amount_fen).sum();
    let summary = format!("创建机构多签 {institution_name}({sfid_id})");
    let mut fields = vec![
        serde_json::json!({ "key": "sfid_id", "label": "机构身份码", "value": sfid_id }),
        serde_json::json!({ "key": "institution_name", "label": "机构名称", "value": institution_name }),
        serde_json::json!({ "key": "admin_count", "label": "管理员数量", "value": admin_count.to_string() }),
        serde_json::json!({ "key": "threshold", "label": "通过阈值", "value": format!("{threshold}/{admin_count}") }),
        serde_json::json!({
            "key": "total_amount_yuan",
            "label": "初始资金合计",
            "value": format!("{}.{:02} GMB", total_amount_fen / 100, (total_amount_fen % 100) as u8),
        }),
    ];
    for acc in accounts {
        fields.push(serde_json::json!({
            "key": format!("amount_{}", acc.account_name),
            "label": format!("{} 初始资金", acc.account_name),
            "value": format!(
                "{}.{:02} GMB",
                acc.amount_fen / 100,
                (acc.amount_fen % 100) as u8
            ),
        }));
    }
    build_sign_request_from_call_data(
        &clean,
        &bytes,
        &call_data,
        "propose_create_institution",
        &summary,
        &serde_json::Value::Array(fields),
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
