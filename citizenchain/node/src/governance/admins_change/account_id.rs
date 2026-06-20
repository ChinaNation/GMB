use crate::governance::registry;

/// 将 `0x` 可选的 64 hex 账户地址解码为链上 AccountId32。
pub fn account_id_from_hex(account_hex: &str) -> Result<[u8; 32], String> {
    let clean = normalize_hex(account_hex);
    let bytes = hex::decode(&clean).map_err(|e| format!("accountId hex 解码失败: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!(
            "accountId 必须为 32 字节，实际 {} 字节",
            bytes.len()
        ));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// 内置治理机构用 runtime 常量中的机构多签主账户作为管理员治理账户。
pub fn account_id_from_builtin_sfid(sfid_number: &str) -> Result<[u8; 32], String> {
    let entry = registry::find_institution(sfid_number)
        .ok_or_else(|| format!("未知的内置治理机构 sfidNumber: {sfid_number}"))?;
    account_id_from_hex(&entry.main_account_hex())
}

pub fn normalize_pubkey_hex(pubkey_hex: &str) -> Result<String, String> {
    let clean = normalize_hex(pubkey_hex);
    if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("管理员公钥必须为 64 位十六进制".to_string());
    }
    Ok(clean)
}

pub fn normalize_hex(hex: &str) -> String {
    let trimmed = hex.trim();
    trimmed
        .strip_prefix("0x")
        .unwrap_or(trimmed)
        .to_ascii_lowercase()
}
