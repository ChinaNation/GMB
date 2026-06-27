//! 清算行注册机构多签创建的 QR 签名请求构造。
//!
//! 中文注释:
//! - 本文件只构造 `organization-manage::propose_create_institution` 的 call_data。
//! - 清算行节点声明(register/update/unregister)属于扫码支付网络准入,
//!   放在 `offchain_transaction::signing`。

use primitives::cid::code::{is_institution_code, InstitutionCode};

use crate::governance::signing::{
    build_sign_request_from_call_data, encode_compact_u32_pub, VoteSignRequestResult,
};

const ORGANIZATION_MANAGE_PALLET_INDEX: u8 = 17;
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
/// 入参顺序与 [`citizenchain/runtime/governance/organization-manage/src/lib.rs::propose_create_institution`]
/// 严格一致。任一字段顺序变更必须同步改本函数和冷钱包 decoder。
#[allow(clippy::too_many_arguments)]
pub fn build_propose_create_institution_call_data(
    cid_number: &str,
    cid_full_name: &str,
    accounts: &[InitialAccountInput],
    institution_code: &InstitutionCode,
    admins_len: u32,
    admins: &[String],
    threshold: u32,
    register_nonce: &str,
    signature_hex: &str,
    issuer_cid_number: &str,
    issuer_main_account: &str,
    signer_pubkey: &str,
    scope_province_name: &str,
    scope_city_name: &str,
) -> Result<Vec<u8>, String> {
    if cid_number.is_empty() || cid_number.len() > 96 {
        return Err("cid_number 长度需在 1..=96".to_string());
    }
    if cid_full_name.is_empty() || cid_full_name.len() > 128 {
        return Err("cid_full_name 长度需在 1..=128".to_string());
    }
    if accounts.is_empty() {
        return Err("accounts 至少 1 项(主账户)".to_string());
    }
    if !is_institution_code(institution_code) {
        return Err("机构账户管理员机构码必须是公权/私权法人机构码".to_string());
    }
    if admins_len < 2 {
        return Err("admins_len 必须 >= 2".to_string());
    }
    if admins.len() as u32 != admins_len {
        return Err(format!(
            "admins_len={admins_len} 与 admins.len={} 不一致",
            admins.len()
        ));
    }
    let min_threshold = std::cmp::max(2, admins_len.saturating_add(1) / 2);
    if threshold < min_threshold || threshold > admins_len {
        return Err(format!(
            "threshold 范围必须在 {min_threshold}..={admins_len}"
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
    if issuer_cid_number.is_empty() {
        return Err("issuer_cid_number 不可为空".to_string());
    }
    if scope_province_name.is_empty() {
        return Err("scope_province_name 不可为空".to_string());
    }
    let issuer_main_account_bytes = parse_account32(issuer_main_account)?;
    let signer_pubkey_bytes = parse_account32(signer_pubkey)?;

    let mut call: Vec<u8> = Vec::with_capacity(512);
    call.push(ORGANIZATION_MANAGE_PALLET_INDEX);
    call.push(CALL_PROPOSE_CREATE_INSTITUTION);

    // 1. cid_number: BoundedVec<u8>
    call.extend_from_slice(&encode_bytes_with_len(cid_number.as_bytes()));
    // 2. cid_full_name: BoundedVec<u8>
    call.extend_from_slice(&encode_bytes_with_len(cid_full_name.as_bytes()));
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
    // 4. institution_code: InstitutionCode([u8; 4]) = 4 个裸字节，无长度前缀。
    call.extend_from_slice(institution_code);
    // 5. admins_len: u32 LE
    call.extend_from_slice(&admins_len.to_le_bytes());
    // 6. admins: BoundedVec<AccountId32> = Compact(N) + N × 32B
    call.extend_from_slice(&encode_compact_u32_pub(admins.len() as u32));
    for pk in admins {
        let bytes = parse_account32(pk)?;
        call.extend_from_slice(&bytes);
    }
    // 7. threshold: u32 LE
    call.extend_from_slice(&threshold.to_le_bytes());
    // 8. register_nonce: BoundedVec<u8>
    call.extend_from_slice(&encode_bytes_with_len(register_nonce.as_bytes()));
    // 9. signature: BoundedVec<u8>(64 字节)
    call.extend_from_slice(&encode_bytes_with_len(&signature_bytes));
    // 10. issuer_cid_number: Vec<u8>
    call.extend_from_slice(&encode_bytes_with_len(issuer_cid_number.as_bytes()));
    // 11. issuer_main_account: AccountId32
    call.extend_from_slice(&issuer_main_account_bytes);
    // 12. signer_pubkey: [u8; 32](固定 32 字节,无长度前缀)
    call.extend_from_slice(&signer_pubkey_bytes);
    // 13. scope_province_name: Vec<u8>
    call.extend_from_slice(&encode_bytes_with_len(scope_province_name.as_bytes()));
    // 14. scope_city_name: Vec<u8>
    call.extend_from_slice(&encode_bytes_with_len(scope_city_name.as_bytes()));

    Ok(call)
}

/// `propose_create_institution` 的冷钱包 QR 签名请求。
#[allow(clippy::too_many_arguments)]
pub fn build_propose_create_institution_sign_request(
    pubkey_hex: &str,
    cid_number: &str,
    cid_full_name: &str,
    accounts: &[InitialAccountInput],
    institution_code: &InstitutionCode,
    admins_len: u32,
    admins: &[String],
    threshold: u32,
    register_nonce: &str,
    signature_hex: &str,
    issuer_cid_number: &str,
    issuer_main_account: &str,
    signer_pubkey: &str,
    scope_province_name: &str,
    scope_city_name: &str,
) -> Result<VoteSignRequestResult, String> {
    let (clean, bytes) = parse_pubkey(pubkey_hex)?;
    let call_data = build_propose_create_institution_call_data(
        cid_number,
        cid_full_name,
        accounts,
        institution_code,
        admins_len,
        admins,
        threshold,
        register_nonce,
        signature_hex,
        issuer_cid_number,
        issuer_main_account,
        signer_pubkey,
        scope_province_name,
        scope_city_name,
    )?;
    build_sign_request_from_call_data(&clean, &bytes, &call_data)
}
