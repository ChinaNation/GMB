//! 中文注释:sr25519 种子 → keypair / 公钥 hex 派生工具。
//!
//! 本文件仅保留通用 sr25519 工具,放在 `crypto/` 顶级目录,与具体业务模块解耦。
//!
//! 本模块只承载"种子文本 → sr25519::Pair / 公钥 hex"的纯函数派生,不持有
//! 任何状态、不读取环境变量。调用方负责安全处理 seed 字节(zeroize 等)。

use hex::FromHex;
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};

/// 把 64 hex 字符的 seed 文本解析成 sr25519 keypair。失败返回错误描述。
pub(crate) fn try_load_signing_key_from_seed(seed_text: &str) -> Result<Sr25519Pair, String> {
    let seed = decode_seed_to_32(seed_text)?;
    Sr25519Pair::from_seed_slice(&seed)
        .map_err(|_| "invalid sr25519 seed for substrate pair derivation".to_string())
}

/// 启动期 / 测试期把 64 hex 字符的 seed 文本解码成 keypair。失败 panic。
#[cfg(test)]
pub(crate) fn load_signing_key_from_seed(seed_text: &str) -> Sr25519Pair {
    try_load_signing_key_from_seed(seed_text)
        .unwrap_or_else(|err| panic!("invalid signing seed hex: {err}"))
}

/// 由 seed 文本派生 32 字节公钥 hex(`0x` + 64 字符,小写)。
#[allow(dead_code)]
pub(crate) fn try_derive_pubkey_hex_from_seed(seed_text: &str) -> Result<String, String> {
    let keypair = try_load_signing_key_from_seed(seed_text)?;
    Ok(format!("0x{}", hex::encode(keypair.public().0)))
}

/// 同上,失败 panic(测试 / 启动期使用)。
#[allow(dead_code)]
pub(crate) fn derive_pubkey_hex_from_seed(seed_text: &str) -> String {
    try_derive_pubkey_hex_from_seed(seed_text)
        .unwrap_or_else(|err| panic!("invalid signing seed hex: {err}"))
}

fn normalize_hex(value: &str) -> &str {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
}

fn decode_seed_to_32(raw: &str) -> Result<[u8; 32], String> {
    let trimmed = normalize_hex(raw);
    if trimmed.len() != 64 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("seed must be exactly 64 hex characters".to_string());
    }
    let bytes = Vec::from_hex(trimmed).map_err(|_| "seed contains invalid hex".to_string())?;
    if bytes.len() != 32 {
        return Err("seed must decode to 32 bytes".to_string());
    }
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_non_hex_seed_is_rejected() {
        assert!(try_load_signing_key_from_seed("password123").is_err());
        assert!(try_derive_pubkey_hex_from_seed("test-seed").is_err());
    }

    #[test]
    fn substrate_seed_derivation_matches_dev_chain_main_pubkey() {
        let pubkey = derive_pubkey_hex_from_seed(
            "0xb642a34db79f5adbc800415b27bd271a5459e5e53f80d63c4e4c920fc247f4da",
        );
        assert_eq!(
            pubkey,
            "0x14e4f684453a0ccf9ebb3113d05ae1da934b7f7b2dbd3b9dcdf4138357ab1607"
        );
    }
}
