// 共享输入校验与标准化逻辑。
use crate::ui::settings::address_utils::decode_ss58_prefix;
use crate::ui::shared::constants::SS58_PREFIX;
const SS58_PRE: &[u8] = b"SS58PRE";

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("SS58 地址解码失败")]
    Ss58DecodeFailed,
    #[error("SS58 地址前缀无效，必须为 2027")]
    Ss58InvalidPrefix,
    #[error("SS58 地址长度无效")]
    Ss58InvalidLength,
    #[error("SS58 地址账户长度无效，必须是 32 字节账户地址")]
    Ss58InvalidAccountLength,
    #[error("SS58 地址校验和无效")]
    Ss58InvalidChecksum,
    #[error("{0}")]
    Ss58PrefixDecode(String),
    #[error("钱包地址不能为空")]
    WalletEmpty,
    #[error("十六进制钱包地址格式无效，应为 0x + 64 位十六进制")]
    WalletInvalidHex,
    #[error("node-key 不能为空")]
    NodeKeyEmpty,
    #[error("node-key 必须是 64 位十六进制字符串")]
    NodeKeyInvalidHex,
    #[error("GRANDPA 私钥不能为空")]
    GrandpaKeyEmpty,
    #[error("GRANDPA 私钥必须是 64 位十六进制字符串")]
    GrandpaKeyInvalidHex,
}

impl From<ValidationError> for String {
    fn from(e: ValidationError) -> Self {
        e.to_string()
    }
}

fn validate_ss58_address(input: &str) -> Result<(), ValidationError> {
    let data = bs58::decode(input)
        .into_vec()
        .map_err(|_| ValidationError::Ss58DecodeFailed)?;

    let (prefix, prefix_len) =
        decode_ss58_prefix(&data).map_err(ValidationError::Ss58PrefixDecode)?;
    if prefix != SS58_PREFIX {
        return Err(ValidationError::Ss58InvalidPrefix);
    }

    if data.len() < prefix_len + 32 + 2 {
        return Err(ValidationError::Ss58InvalidLength);
    }
    let payload_len = data.len() - prefix_len - 2;
    if payload_len != 32 {
        return Err(ValidationError::Ss58InvalidAccountLength);
    }

    let (without_checksum, checksum) = data.split_at(data.len() - 2);
    let hash = blake2b_simd::Params::new()
        .hash_length(64)
        .to_state()
        .update(SS58_PRE)
        .update(without_checksum)
        .finalize();
    if checksum != &hash.as_bytes()[..2] {
        return Err(ValidationError::Ss58InvalidChecksum);
    }

    Ok(())
}

pub fn normalize_wallet_address(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err(ValidationError::WalletEmpty.into());
    }

    // 允许 0x + 64 的十六进制地址。
    if let Some(raw) = value.strip_prefix("0x") {
        if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::WalletInvalidHex.into());
        }
        return Ok(format!("0x{}", raw.to_ascii_lowercase()));
    }

    // 允许 SS58 地址，且强制链前缀 2027。
    validate_ss58_address(value)?;

    Ok(value.to_string())
}

pub fn normalize_node_key(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err(ValidationError::NodeKeyEmpty.into());
    }

    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::NodeKeyInvalidHex.into());
    }

    Ok(raw.to_ascii_lowercase())
}

pub fn normalize_grandpa_key(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err(ValidationError::GrandpaKeyEmpty.into());
    }

    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::GrandpaKeyInvalidHex.into());
    }

    Ok(raw.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- normalize_wallet_address ---

    #[test]
    fn wallet_empty_rejected() {
        assert!(normalize_wallet_address("").is_err());
        assert!(normalize_wallet_address("   ").is_err());
    }

    #[test]
    fn wallet_valid_hex() {
        let hex = format!("0x{}", "a1b2c3d4".repeat(8));
        let result = normalize_wallet_address(&hex).unwrap();
        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 66);
        assert_eq!(result, result.to_ascii_lowercase());
    }

    #[test]
    fn wallet_hex_wrong_length() {
        assert!(normalize_wallet_address("0xabcd").is_err());
    }

    #[test]
    fn wallet_hex_non_hex_chars() {
        let bad = format!("0x{}zz", "a1".repeat(31));
        assert!(normalize_wallet_address(&bad).is_err());
    }

    #[test]
    fn wallet_hex_uppercase_normalized() {
        let hex = format!("0x{}", "A1B2C3D4".repeat(8));
        let result = normalize_wallet_address(&hex).unwrap();
        assert_eq!(result, format!("0x{}", "a1b2c3d4".repeat(8)));
    }

    // --- normalize_node_key ---

    #[test]
    fn node_key_empty_rejected() {
        assert!(normalize_node_key("").is_err());
    }

    #[test]
    fn node_key_valid_no_prefix() {
        let key = "a1b2c3d4".repeat(8);
        let result = normalize_node_key(&key).unwrap();
        assert_eq!(result.len(), 64);
        assert!(!result.starts_with("0x"));
    }

    #[test]
    fn node_key_valid_with_0x_prefix() {
        let key = format!("0x{}", "a1b2c3d4".repeat(8));
        let result = normalize_node_key(&key).unwrap();
        assert_eq!(result.len(), 64);
        assert!(!result.starts_with("0x"));
    }

    #[test]
    fn node_key_wrong_length() {
        assert!(normalize_node_key("abcdef").is_err());
    }

    // --- normalize_grandpa_key ---

    #[test]
    fn grandpa_key_empty_rejected() {
        assert!(normalize_grandpa_key("").is_err());
    }

    #[test]
    fn grandpa_key_valid() {
        let key = "a1b2c3d4".repeat(8);
        let result = normalize_grandpa_key(&key).unwrap();
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn grandpa_key_strips_0x() {
        let key = format!("0x{}", "A1B2C3D4".repeat(8));
        let result = normalize_grandpa_key(&key).unwrap();
        assert_eq!(result, "a1b2c3d4".repeat(8));
    }

    #[test]
    fn grandpa_key_wrong_length() {
        assert!(normalize_grandpa_key("abcd").is_err());
    }
}
