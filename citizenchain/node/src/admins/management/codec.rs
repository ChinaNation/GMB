use super::types::AdminAccountDecoded;

/// 解码机构管理员模块的 `InstitutionAdminAccount`。
///
/// SCALE 布局固定为 `cid_number + institution_code + admins(AccountId[]) + status`。
/// 岗位、任期、来源和姓名不在 admins 值中，桌面端需另查 entity 岗位任职表；个人
/// 多签使用独立账户模型，不通过此机构解码器读取。
pub fn decode_admin_account(data: &[u8]) -> Result<AdminAccountDecoded, String> {
    // 头部 cid_number: BoundedVec<u8> = Compact(len) + bytes。
    let (cid_len, cid_len_size) = read_compact_u32(data, 0)?;
    let cid_end = cid_len_size + cid_len as usize;
    if cid_end > data.len() {
        return Err("InstitutionAdminAccount CID 数据不足".to_string());
    }
    let cid_number = String::from_utf8(data[cid_len_size..cid_end].to_vec())
        .map_err(|_| "InstitutionAdminAccount CID 不是 UTF-8".to_string())?;
    let mut offset = cid_end;

    // institution_code: [u8;4] 定长(4 裸字节，无 kind 字段)。
    if offset + 4 > data.len() {
        return Err("InstitutionAdminAccount 机构码数据不足".to_string());
    }
    let institution_code: [u8; 4] = data[offset..offset + 4]
        .try_into()
        .map_err(|_| "AdminAccount 机构码数据不足".to_string())?;
    offset += 4;
    let (count, len_size) = read_compact_u32(data, offset)?;
    offset += len_size;
    let mut admins = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if offset + 32 > data.len() {
            return Err("AdminAccount 管理员列表数据不足".to_string());
        }
        let account = hex::encode(&data[offset..offset + 32]);
        offset += 32;
        admins.push(account);
    }

    if offset >= data.len() {
        return Err("InstitutionAdminAccount status 数据不足".to_string());
    }
    let status = data[offset];
    offset += 1;
    if offset != data.len() {
        return Err("InstitutionAdminAccount 存在尾随字节".to_string());
    }
    if status > 2 {
        return Err("InstitutionAdminAccount status 非法".to_string());
    }

    Ok(AdminAccountDecoded {
        cid_number,
        institution_code,
        admins,
        status,
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
    fn institution_admin_account_decodes_wallets_only() {
        use codec::Encode;
        let bytes = (
            b"CID-1".to_vec(),
            *b"NRC\0",
            vec![[0xaau8; 32], [0xbbu8; 32]],
            1u8,
        )
            .encode();
        let decoded = decode_admin_account(&bytes).unwrap();
        assert_eq!(decoded.cid_number, "CID-1");
        assert_eq!(decoded.institution_code, *b"NRC\0");
        assert_eq!(decoded.admins, vec!["aa".repeat(32), "bb".repeat(32)]);
        assert_eq!(decoded.status, 1);
    }
}
