//! 多签转账 SubjectId 编码。

/// 将多签转账支出主体编码为 D/ADR-015 `SubjectId(48)`。
///
/// 中文注释：node 端只支持内置治理机构和注册机构多签账户；个人多签
/// 属于 wuminapp 个人钱包能力，桌面 node 明确拒绝。
pub fn subject_id_from_transfer_identity(identity: &str) -> Result<[u8; 48], String> {
    if let Some(hex) = identity.strip_prefix("duoqian:") {
        return account_subject_id(0x05, hex, "注册机构多签账户");
    }
    if identity.starts_with("personal:") {
        return Err("node 端不支持个人多签转账".to_string());
    }

    let raw = identity.as_bytes();
    if raw.is_empty() || raw.len() > 47 {
        return Err(format!(
            "sfidNumber 长度必须在 1..47 字节，实际: {}",
            raw.len()
        ));
    }
    let mut out = [0u8; 48];
    out[0] = 0x01;
    out[1..1 + raw.len()].copy_from_slice(raw);
    Ok(out)
}

fn account_subject_id(kind: u8, account_hex: &str, label: &str) -> Result<[u8; 48], String> {
    let clean = account_hex.strip_prefix("0x").unwrap_or(account_hex);
    let account = hex::decode(clean).map_err(|e| format!("{label}地址解码失败: {e}"))?;
    if account.len() != 32 {
        return Err(format!("{label}地址必须是 32 字节 AccountId"));
    }
    let mut out = [0u8; 48];
    out[0] = kind;
    out[1..33].copy_from_slice(&account);
    Ok(out)
}
