/// 严格解析机构交易明确传入的账户，不接受 CID、前缀身份串或主账户回退。
pub fn institution_account_from_hex(account_hex: &str) -> Result<[u8; 32], String> {
    let clean = account_hex.strip_prefix("0x").unwrap_or(account_hex);
    let account = hex::decode(clean).map_err(|e| format!("institution_account 解码失败: {e}"))?;
    if account.len() != 32 {
        return Err("institution_account 必须是 32 字节 AccountId".to_string());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&account);
    Ok(out)
}
