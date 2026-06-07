use super::account_id::normalize_pubkey_hex;
use super::types::AdminAccountDecoded;

/// 解码 `AdminsChange::AdminAccounts` 的完整核心字段。
///
/// 链上布局:
/// org:u8 + kind:u8 + admins:BoundedVec<AccountId32>
/// + creator:AccountId32 + created_at:u32 + updated_at:u32 + status:u8。
/// 中文注释:created_at/updated_at 是 BlockNumberFor<T>,citizenchain runtime 配置为 u32。
pub fn decode_admin_account(data: &[u8]) -> Result<AdminAccountDecoded, String> {
    if data.len() < 2 {
        return Err("AdminAccount 数据不足".to_string());
    }
    let org = data[0];
    let kind = data[1];

    let (count, len_size) = read_compact_u32(data, 2)?;
    let mut offset = 2 + len_size;
    let mut admins = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if offset + 32 > data.len() {
            return Err("AdminAccount 管理员列表数据不足".to_string());
        }
        admins.push(hex::encode(&data[offset..offset + 32]));
        offset += 32;
    }

    if offset + 32 > data.len() {
        return Err("AdminAccount creator 数据不足".to_string());
    }
    let creator_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    if offset + 4 > data.len() {
        return Err("AdminAccount created_at 数据不足".to_string());
    }
    let created_at = read_u32_le(data, offset);
    offset += 4;

    if offset + 4 > data.len() {
        return Err("AdminAccount updated_at 数据不足".to_string());
    }
    let updated_at = read_u32_le(data, offset);
    offset += 4;

    if offset >= data.len() {
        return Err("AdminAccount status 数据不足".to_string());
    }
    let status = data[offset];

    Ok(AdminAccountDecoded {
        org,
        kind,
        admins,
        creator_hex,
        created_at,
        updated_at,
        status,
    })
}

pub fn encode_admins(admins: &[String]) -> Result<Vec<u8>, String> {
    let mut out = encode_compact_u32(admins.len() as u32);
    for admin in admins {
        let clean = normalize_pubkey_hex(admin)?;
        let bytes = hex::decode(clean).map_err(|e| format!("管理员公钥解码失败: {e}"))?;
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

pub fn encode_compact_u32(value: u32) -> Vec<u8> {
    if value < 1 << 6 {
        vec![(value as u8) << 2]
    } else if value < 1 << 14 {
        let encoded = ((value << 2) | 0x01) as u16;
        encoded.to_le_bytes().to_vec()
    } else {
        let encoded = (value << 2) | 0x02;
        encoded.to_le_bytes().to_vec()
    }
}

pub fn read_compact_u32(data: &[u8], offset: usize) -> Result<(u32, usize), String> {
    if offset >= data.len() {
        return Err("Compact<u32> 数据不足".to_string());
    }
    let first = data[offset];
    match first & 0x03 {
        0 => Ok(((first >> 2) as u32, 1)),
        1 => {
            if offset + 2 > data.len() {
                return Err("Compact<u32> two-byte 数据不足".to_string());
            }
            let raw = u16::from_le_bytes([data[offset], data[offset + 1]]);
            Ok(((raw >> 2) as u32, 2))
        }
        2 => {
            if offset + 4 > data.len() {
                return Err("Compact<u32> four-byte 数据不足".to_string());
            }
            let raw = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            Ok((raw >> 2, 4))
        }
        _ => Err("Compact<u32> big-integer 模式暂不支持".to_string()),
    }
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_compact_matches_scale_small_values() {
        assert_eq!(encode_compact_u32(0), vec![0x00]);
        assert_eq!(encode_compact_u32(1), vec![0x04]);
        assert_eq!(encode_compact_u32(64), vec![0x01, 0x01]);
    }

    #[test]
    fn decode_admin_account_full_layout() {
        let mut data = vec![0, 0, 0x04];
        data.extend_from_slice(&[0xaa; 32]);
        data.extend_from_slice(&[0xbb; 32]);
        data.extend_from_slice(&7u32.to_le_bytes());
        data.extend_from_slice(&9u32.to_le_bytes());
        data.push(1);
        let decoded = decode_admin_account(&data).unwrap();
        assert_eq!(decoded.admins, vec!["aa".repeat(32)]);
        assert_eq!(decoded.status, 1);
    }
}
