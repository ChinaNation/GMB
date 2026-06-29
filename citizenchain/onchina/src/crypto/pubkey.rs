//! sr25519 pubkey 规范化与等值比较
//!
//! 中文注释:这是跨业务通用的 sr25519 公钥工具,归入 `crypto`。
//! 内部统一 0x 小写 hex,前端展示 SS58(prefix=2027),禁止混用。

use crate::auth::login::{parse_sr25519_pubkey, parse_sr25519_pubkey_bytes};

pub(crate) fn normalize_admin_account(input: &str) -> Option<String> {
    normalize_sr25519_pubkey(input)
}

pub(crate) fn same_admin_account(left: &str, right: &str) -> bool {
    same_sr25519_pubkey(left, right)
}

fn normalize_sr25519_pubkey(input: &str) -> Option<String> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(input) {
        return Some(hex_pubkey);
    }
    if parse_sr25519_pubkey_bytes(input).is_some() {
        return Some(input.trim().to_string());
    }
    None
}

fn same_sr25519_pubkey(left: &str, right: &str) -> bool {
    match (parse_sr25519_pubkey(left), parse_sr25519_pubkey(right)) {
        (Some(l), Some(r)) => l.eq_ignore_ascii_case(r.as_str()),
        _ => left.trim().eq_ignore_ascii_case(right.trim()),
    }
}

/// 从 SS58 地址解出 0x 小写 hex 公钥。
///
/// 中文注释:SS58↔hex 互转是跨业务通用的钱包地址工具,归入 `crypto::pubkey`。
pub(crate) fn ss58_to_pubkey_hex(address: &str) -> Option<String> {
    let decoded = bs58::decode(address.trim()).into_vec().ok()?;
    let prefix_len = if decoded.first().copied().unwrap_or(0) < 64 {
        1
    } else {
        2
    };
    if decoded.len() < prefix_len + 32 + 2 {
        return None;
    }
    let pubkey = &decoded[prefix_len..prefix_len + 32];
    Some(format!("0x{}", hex::encode(pubkey)))
}

/// 0x hex 公钥转 SS58 地址(prefix=2027)。
pub(crate) fn pubkey_hex_to_ss58(pubkey_hex: &str) -> Option<String> {
    let pubkey_bytes = hex::decode(pubkey_hex.trim_start_matches("0x")).ok()?;
    if pubkey_bytes.len() != 32 {
        return None;
    }
    use blake2::{Blake2bVar, digest::VariableOutput};
    let prefix: u16 = 2027;
    let first = ((prefix & 0b0000_0000_1111_1100) as u8) >> 2 | 0b01000000;
    let second = (prefix >> 8) as u8 | ((prefix & 0b0000_0000_0000_0011) as u8) << 6;
    let mut payload = vec![first, second];
    payload.extend_from_slice(&pubkey_bytes);
    let mut hasher = Blake2bVar::new(64).ok()?;
    use blake2::digest::Update;
    hasher.update(b"SS58PRE");
    hasher.update(&payload);
    let mut hash = vec![0u8; 64];
    hasher.finalize_variable(&mut hash).ok()?;
    payload.extend_from_slice(&hash[..2]);
    Some(bs58::encode(payload).into_string())
}
