// 地址与 node-key 的最小校验逻辑。
use blake2::{Blake2b512, Digest};

const SS58_PREFIX: u16 = 2027;
const SS58_PRE: &[u8] = b"SS58PRE";

fn decode_ss58_prefix(data: &[u8]) -> Result<(u16, usize), String> {
    if data.is_empty() {
        return Err("SS58 地址为空".to_string());
    }
    let first = data[0];
    match first {
        0..=63 => Ok((first as u16, 1)),
        64..=127 => {
            if data.len() < 2 {
                return Err("SS58 地址格式无效".to_string());
            }
            let second = data[1];
            let prefix = (((first & 0x3f) as u16) << 2)
                | ((second as u16) >> 6)
                | (((second & 0x3f) as u16) << 8);
            Ok((prefix, 2))
        }
        _ => Err("SS58 地址格式无效".to_string()),
    }
}

fn validate_ss58_address(input: &str) -> Result<(), String> {
    let data = bs58::decode(input)
        .into_vec()
        .map_err(|_| "SS58 地址解码失败".to_string())?;

    let (prefix, prefix_len) = decode_ss58_prefix(&data)?;
    if prefix != SS58_PREFIX {
        return Err("SS58 地址前缀无效，必须为 2027".to_string());
    }

    if data.len() < prefix_len + 32 + 2 {
        return Err("SS58 地址长度无效".to_string());
    }
    let payload_len = data.len() - prefix_len - 2;
    if payload_len != 32 {
        return Err("SS58 地址账户长度无效，必须是 32 字节账户地址".to_string());
    }

    let (without_checksum, checksum) = data.split_at(data.len() - 2);
    let mut hasher = Blake2b512::new();
    hasher.update(SS58_PRE);
    hasher.update(without_checksum);
    let hash = hasher.finalize();
    if checksum != &hash[..2] {
        return Err("SS58 地址校验和无效".to_string());
    }

    Ok(())
}

pub fn normalize_wallet_address(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err("钱包地址不能为空".to_string());
    }

    // 允许 0x + 64 的十六进制地址。
    if let Some(raw) = value.strip_prefix("0x") {
        if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("十六进制钱包地址格式无效，应为 0x + 64 位十六进制".to_string());
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
        return Err("node-key 不能为空".to_string());
    }

    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("node-key 必须是 64 位十六进制字符串".to_string());
    }

    Ok(raw.to_ascii_lowercase())
}

pub fn normalize_node_name(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err("节点名称不能为空".to_string());
    }
    if value.chars().count() > 64 {
        return Err("节点名称不能超过 64 个字符".to_string());
    }
    if value.chars().any(|c| c.is_control()) {
        return Err("节点名称不能包含控制字符".to_string());
    }
    Ok(value.to_string())
}

pub fn normalize_grandpa_key(input: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err("GRANDPA 私钥不能为空".to_string());
    }

    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("GRANDPA 私钥必须是 64 位十六进制字符串".to_string());
    }

    Ok(raw.to_ascii_lowercase())
}
