//! 多签转账签名请求构造。
//!
//! 本文件只负责 `DuoqianTransfer::propose_transfer` 的 call_data 和离线签名展示字段；
//! 通用 QR 协议、nonce、payload 和提交校验复用治理签名基础设施。

use crate::governance;

/// 构建 propose_transfer 签名请求（创建转账提案：pallet=19, call=0）。
pub fn build_propose_transfer_sign_request(
    pubkey_hex: &str,
    sfid_number: &str,
    org_type: u8,
    beneficiary_address: &str,
    amount_yuan: f64,
    remark: &str,
) -> Result<governance::signing::VoteSignRequestResult, String> {
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;

    if amount_yuan < 1.11 {
        return Err("转账金额不能低于 1.11 元".to_string());
    }
    let amount_fen = (amount_yuan * 100.0).round() as u128;

    let remark_bytes = remark.as_bytes();
    if remark_bytes.len() > 256 {
        return Err(format!(
            "备注长度不能超过 256 字节，当前 {} 字节",
            remark_bytes.len()
        ));
    }

    let beneficiary_bytes = governance::signing::decode_ss58_to_pubkey(beneficiary_address)?;

    // node 端支持内置治理机构和注册机构多签账户，明确拒绝个人多签。
    // 内置治理机构可校验“收款地址不能等于主账户”；注册机构账户没有 registry 条目，
    // 只按 `duoqian:<account_hex>` 生成 0x05 SubjectId。
    let entry = governance::registry::find_institution(sfid_number);
    if let Some(entry) = entry {
        let institution_duoqian = hex::decode(entry.main_address_hex())
            .map_err(|e| format!("主账户地址解码失败: {e}"))?;
        if beneficiary_bytes[..] == institution_duoqian[..] {
            return Err("收款地址不能为本机构多签地址".to_string());
        }
    }

    let institution_id = super::subject_id::subject_id_from_transfer_identity(sfid_number)?;

    let remark_compact = governance::signing::encode_compact_u32_pub(remark_bytes.len() as u32);
    let mut call_data =
        Vec::with_capacity(2 + 1 + 48 + 32 + 16 + remark_compact.len() + remark_bytes.len());
    call_data.push(19u8);
    call_data.push(0u8);
    call_data.push(org_type);
    call_data.extend_from_slice(&institution_id);
    call_data.extend_from_slice(&beneficiary_bytes);
    call_data.extend_from_slice(&amount_fen.to_le_bytes());
    call_data.extend_from_slice(&remark_compact);
    call_data.extend_from_slice(remark_bytes);

    let institution_label = if let Some(hex) = sfid_number.strip_prefix("duoqian:") {
        let short = if hex.len() > 14 {
            format!("{}...{}", &hex[..8], &hex[hex.len() - 6..])
        } else {
            hex.to_string()
        };
        format!("机构账户 {short}")
    } else {
        match org_type {
            0 => "国储会".to_string(),
            1 => "省储会".to_string(),
            2 => "省储行".to_string(),
            _ => "未知".to_string(),
        }
    };

    // display.fields 必须与 wumin PayloadDecoder 解码结果的 key/value 完全一致。
    // wumin 解码 propose_transfer 返回: institution / beneficiary / amount_yuan / remark。
    let fields = serde_json::json!([
        { "key": "institution", "label": "转出账户", "value": institution_label },
        { "key": "beneficiary", "label": "收款地址", "value": beneficiary_address },
        {
            "key": "amount_yuan",
            "label": "金额",
            "value": format!("{} GMB", governance::signing::format_amount(amount_yuan))
        },
        { "key": "remark", "label": "备注", "value": remark }
    ]);
    let summary = format!(
        "{institution_label} 提案转账 {} GMB 给 {beneficiary_address}",
        governance::signing::format_amount(amount_yuan)
    );

    governance::signing::build_sign_request_from_call_data(
        &pubkey_clean,
        &pubkey_bytes,
        &call_data,
        "propose_transfer",
        &summary,
        &fields,
    )
}

/// 构建安全基金转账提案签名请求（pallet=19, call=1）。
pub fn build_propose_safety_fund_sign_request(
    pubkey_hex: &str,
    beneficiary_address: &str,
    amount_yuan: f64,
    remark: &str,
) -> Result<governance::signing::VoteSignRequestResult, String> {
    let pubkey_clean = normalize_pubkey(pubkey_hex)?;
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;
    let call_data = build_safety_fund_call_data(beneficiary_address, amount_yuan, remark)?;

    let fields = serde_json::json!([
        { "key": "beneficiary", "label": "收款地址", "value": beneficiary_address },
        {
            "key": "amount_yuan",
            "label": "金额",
            "value": format!("{} GMB", governance::signing::format_amount(amount_yuan))
        },
        { "key": "remark", "label": "备注", "value": remark }
    ]);
    let summary = format!(
        "安全基金转账 {} GMB 给 {beneficiary_address}",
        governance::signing::format_amount(amount_yuan)
    );

    governance::signing::build_sign_request_from_call_data(
        &pubkey_clean,
        &pubkey_bytes,
        &call_data,
        "propose_safety_fund_transfer",
        &summary,
        &fields,
    )
}

/// 构建手续费划转提案签名请求（pallet=19, call=2）。
pub fn build_propose_sweep_sign_request(
    pubkey_hex: &str,
    sfid_number: &str,
    amount_yuan: f64,
) -> Result<governance::signing::VoteSignRequestResult, String> {
    let pubkey_clean = normalize_pubkey(pubkey_hex)?;
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;
    let call_data = build_sweep_call_data(sfid_number, amount_yuan)?;

    let inst_name = governance::registry::find_institution(sfid_number)
        .map(|entry| entry.name())
        .unwrap_or("未知机构");
    let fields = serde_json::json!([
        { "key": "institution", "label": "机构", "value": inst_name },
        {
            "key": "amount_yuan",
            "label": "金额",
            "value": format!("{} GMB", governance::signing::format_amount(amount_yuan))
        }
    ]);
    let summary = format!(
        "{inst_name} 手续费划转 {} 元",
        governance::signing::format_amount(amount_yuan)
    );

    governance::signing::build_sign_request_from_call_data(
        &pubkey_clean,
        &pubkey_bytes,
        &call_data,
        "propose_sweep_to_main",
        &summary,
        &fields,
    )
}

/// 安全基金转账 call_data，供签名构造和签名回执提交复用。
pub(crate) fn build_safety_fund_call_data(
    beneficiary_address: &str,
    amount_yuan: f64,
    remark: &str,
) -> Result<Vec<u8>, String> {
    if amount_yuan < 1.11 {
        return Err("转账金额不能低于 1.11 元".to_string());
    }
    let amount_fen = (amount_yuan * 100.0).round() as u128;
    let remark_bytes = remark.as_bytes();
    if remark_bytes.len() > 256 {
        return Err(format!(
            "备注长度不能超过 256 字节，当前 {} 字节",
            remark_bytes.len()
        ));
    }
    let beneficiary_bytes = governance::signing::decode_ss58_to_pubkey(beneficiary_address)?;
    let remark_compact = governance::signing::encode_compact_u32_pub(remark_bytes.len() as u32);

    let mut call_data = Vec::with_capacity(2 + 32 + 16 + remark_compact.len() + remark_bytes.len());
    call_data.push(19u8);
    call_data.push(1u8);
    call_data.extend_from_slice(&beneficiary_bytes);
    call_data.extend_from_slice(&amount_fen.to_le_bytes());
    call_data.extend_from_slice(&remark_compact);
    call_data.extend_from_slice(remark_bytes);
    Ok(call_data)
}

/// 手续费划转 call_data，供签名构造和签名回执提交复用。
pub(crate) fn build_sweep_call_data(
    sfid_number: &str,
    amount_yuan: f64,
) -> Result<Vec<u8>, String> {
    if amount_yuan <= 0.0 {
        return Err("划转金额必须大于 0".to_string());
    }
    let amount_fen = (amount_yuan * 100.0).round() as u128;
    let institution_id = super::subject_id::subject_id_from_transfer_identity(sfid_number)?;

    let mut call_data = Vec::with_capacity(2 + 48 + 16);
    call_data.push(19u8);
    call_data.push(2u8);
    call_data.extend_from_slice(&institution_id);
    call_data.extend_from_slice(&amount_fen.to_le_bytes());
    Ok(call_data)
}

fn normalize_pubkey(pubkey_hex: &str) -> Result<String, String> {
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    Ok(pubkey_clean)
}
