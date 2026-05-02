//! 清算行注册机构多签创建的 QR 签名请求构造。
//!
//! 中文注释:
//! - 本文件只构造 `duoqian-manage::propose_create_institution` 的 call_data。
//! - 清算行节点声明(register/update/unregister)属于扫码支付网络准入,
//!   放在 `offchain_transaction::signing`。

use crate::governance::signing::{
    build_sign_request_from_call_data, encode_compact_u32_pub, VoteSignRequestResult,
};

const DUOQIAN_PALLET_INDEX: u8 = 17;
const CALL_PROPOSE_CREATE_INSTITUTION: u8 = 5;

/// 创建机构时每个账户的初始资金条目(已派生 hex 地址 / 字符串金额"分")。
#[derive(Debug, Clone)]
pub struct InitialAccountInput {
    pub account_name: String,
    /// 初始资金,单位"分"(u128 字符串,避免 JS BigInt 跨语言传输精度问题)。
    pub amount_fen: u128,
}

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

/// 把可变长字段按 SCALE Vec<u8> 编码:`Compact<u32>(len) || raw_bytes`。
fn encode_bytes_with_len(raw: &[u8]) -> Vec<u8> {
    let mut out = encode_compact_u32_pub(raw.len() as u32);
    out.extend_from_slice(raw);
    out
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

/// SCALE 编码 u128 little-endian(16 字节)。
fn encode_u128_le(v: u128) -> [u8; 16] {
    v.to_le_bytes()
}

/// 构造 `propose_create_institution`(pallet=17, call=5)的 call_data。
///
/// 入参顺序与 [`citizenchain/runtime/transaction/duoqian-manage/src/lib.rs::propose_create_institution`]
/// 严格一致(10 个字段)。任一字段顺序变更必须同步改本函数。
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
    signer_admin_pubkey: &str,
) -> Result<Vec<u8>, String> {
    if sfid_id.is_empty() || sfid_id.len() > 96 {
        return Err("sfid_id 长度需在 1..=96".to_string());
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
    let signer_admin_pubkey_bytes = parse_account32(signer_admin_pubkey)?;

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
    // 9. province: Vec<u8>(本流程必填,链端要查 ShengSigningPubkey[province])
    call.extend_from_slice(&encode_bytes_with_len(signing_province.as_bytes()));
    // 10. signer_admin_pubkey: [u8; 32](固定 32 字节,无长度前缀)
    call.extend_from_slice(&signer_admin_pubkey_bytes);

    Ok(call)
}

/// `propose_create_institution` 的冷钱包 QR 签名请求。
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
    signer_admin_pubkey: &str,
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
        signer_admin_pubkey,
    )?;
    let total_amount_fen: u128 = accounts.iter().map(|a| a.amount_fen).sum();
    let summary = format!("创建清算行机构多签 {institution_name}({sfid_id})");
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
