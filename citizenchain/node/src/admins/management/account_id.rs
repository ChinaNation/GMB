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
