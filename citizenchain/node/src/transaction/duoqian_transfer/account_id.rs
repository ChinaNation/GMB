use crate::governance;

/// 将转账流程传入的机构身份解析为机构多签 AccountId32。
pub fn account_id_from_transfer_identity(identity: &str) -> Result<[u8; 32], String> {
    if let Some(hex) = identity.strip_prefix("duoqian:") {
        return account_id_from_hex(hex, "注册机构多签账户");
    }
    if identity.starts_with("personal:") {
        return Err("node 端不支持个人多签转账".to_string());
    }

    let entry = governance::registry::find_institution(identity)
        .ok_or_else(|| format!("未知的治理机构 cidNumber: {identity}"))?;
    account_id_from_hex(&entry.main_account_hex(), "内置机构多签账户")
}

fn account_id_from_hex(account_hex: &str, label: &str) -> Result<[u8; 32], String> {
    let clean = account_hex.strip_prefix("0x").unwrap_or(account_hex);
    let account = hex::decode(clean).map_err(|e| format!("{label}地址解码失败: {e}"))?;
    if account.len() != 32 {
        return Err(format!("{label}地址必须是 32 字节 AccountId"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&account);
    Ok(out)
}
