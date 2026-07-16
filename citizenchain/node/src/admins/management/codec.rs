use super::types::AdminAccountDecoded;

/// 解码机构管理员模块的 `InstitutionAdmins`。
///
/// SCALE 布局固定为 `institution_code + admins(AccountId[])`。CID 只存在于 storage key，
/// 不在 value 中重复保存；岗位、任期、来源和姓名另查 entity 岗位任职表。
pub fn decode_admin_account(data: &[u8]) -> Result<AdminAccountDecoded, String> {
    // institution_code: [u8;4] 定长(4 裸字节，无 kind 字段)。
    if data.len() < 4 {
        return Err("InstitutionAdmins 机构码数据不足".to_string());
    }
    let institution_code: [u8; 4] = data[..4]
        .try_into()
        .map_err(|_| "InstitutionAdmins 机构码数据不足".to_string())?;
    let mut offset = 4;
    let (count, len_size) = read_compact_u32(data, offset)?;
    offset += len_size;
    let mut admins = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if offset + 32 > data.len() {
            return Err("InstitutionAdmins 管理员列表数据不足".to_string());
        }
        let account = hex::encode(&data[offset..offset + 32]);
        offset += 32;
        admins.push(account);
    }

    if offset != data.len() {
        return Err("InstitutionAdmins 存在尾随字节".to_string());
    }

    Ok(AdminAccountDecoded {
        institution_code,
        admins,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn institution_admins_decodes_wallets_only() {
        use codec::Encode;
        let bytes = (*b"NRC\0", vec![[0xaau8; 32], [0xbbu8; 32]]).encode();
        let decoded = decode_admin_account(&bytes).unwrap();
        assert_eq!(decoded.institution_code, *b"NRC\0");
        assert_eq!(decoded.admins, vec!["aa".repeat(32), "bb".repeat(32)]);
    }
}
