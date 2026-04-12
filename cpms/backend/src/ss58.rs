/// SS58 地址解码为 0x hex 公钥。
///
/// 复制自 sfid/backend/src/operate/binding.rs:ss58_to_pubkey_hex。
/// prefix < 64 → 1 字节前缀；prefix >= 64 → 2 字节前缀。
pub fn ss58_to_pubkey_hex(address: &str) -> Option<String> {
    let decoded = bs58::decode(address.trim()).into_vec().ok()?;
    let prefix_len = if decoded.first().copied().unwrap_or(0) < 64 { 1 } else { 2 };
    if decoded.len() < prefix_len + 32 + 2 {
        return None;
    }
    let pubkey = &decoded[prefix_len..prefix_len + 32];
    Some(format!("0x{}", hex::encode(pubkey)))
}
