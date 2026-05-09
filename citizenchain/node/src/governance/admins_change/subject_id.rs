pub const SUBJECT_KIND_BUILTIN: u8 = 0x01;
pub const SUBJECT_KIND_SFID_INSTITUTION: u8 = 0x02;
pub const SUBJECT_KIND_PERSONAL_DUOQIAN: u8 = 0x03;
pub const SUBJECT_KIND_INSTITUTION_ACCOUNT: u8 = 0x05;

/// 将内置治理机构 sfidNumber 派生为 48 字节 SubjectId。
pub fn subject_id_from_builtin_sfid(sfid_number: &str) -> Result<[u8; 48], String> {
    build_padded_subject_id(SUBJECT_KIND_BUILTIN, sfid_number.as_bytes())
}

/// 将 `0x` 可选的 64 hex 账户公钥派生为账户型 SubjectId。
pub fn subject_id_from_account_hex(kind: u8, account_hex: &str) -> Result<[u8; 48], String> {
    let clean = normalize_hex(account_hex);
    let bytes = hex::decode(&clean).map_err(|e| format!("账户公钥 hex 解码失败: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("账户公钥必须为 32 字节，实际 {} 字节", bytes.len()));
    }
    build_padded_subject_id(kind, &bytes)
}

pub fn subject_id_from_hex(subject_id_hex: &str) -> Result<[u8; 48], String> {
    let clean = normalize_hex(subject_id_hex);
    let bytes = hex::decode(&clean).map_err(|e| format!("subjectId hex 解码失败: {e}"))?;
    if bytes.len() != 48 {
        return Err(format!(
            "subjectId 必须为 48 字节，实际 {} 字节",
            bytes.len()
        ));
    }
    let mut out = [0u8; 48];
    out.copy_from_slice(&bytes);
    Ok(out)
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

fn build_padded_subject_id(kind: u8, payload: &[u8]) -> Result<[u8; 48], String> {
    if payload.is_empty() || payload.len() > 47 {
        return Err(format!(
            "SubjectId payload 长度必须在 1..=47 字节，实际 {} 字节",
            payload.len()
        ));
    }
    let mut out = [0u8; 48];
    out[0] = kind;
    out[1..1 + payload.len()].copy_from_slice(payload);
    Ok(out)
}
