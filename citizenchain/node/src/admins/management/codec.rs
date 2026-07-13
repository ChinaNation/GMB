use super::account_id::normalize_pubkey_hex;
use super::types::{AdminAccountDecoded, AdminProfileInfo};

/// 解码机构管理员模块的 `InstitutionAdminAccount`。
///
/// SCALE 布局固定为 `cid_number + institution_code + admins(AccountId[]) + status`。
/// 岗位、任期、来源和姓名不在 admins 值中，桌面端需另查 entity 岗位任职表；个人
/// 多签使用独立账户模型，不通过此机构解码器读取。
pub fn decode_admin_account(data: &[u8]) -> Result<AdminAccountDecoded, String> {
    // 头部 cid_number: BoundedVec<u8> = Compact(len) + bytes。个人多签无机构 cid → Compact(0)。
    // 展示层用查询入参里的 cid,这里只需跳过以对齐后续字段偏移。
    let (cid_len, cid_len_size) = read_compact_u32(data, 0)?;
    let mut offset = cid_len_size + cid_len as usize;

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
        admins.push(AdminProfileInfo::account_only(account));
    }

    if offset >= data.len() {
        return Err("InstitutionAdminAccount status 数据不足".to_string());
    }
    let status = data[offset];
    let kind = if primitives::cid::code::is_private_legal_code(&institution_code) {
        1 // PrivateInstitution
    } else if primitives::cid::code::is_public_legal_code(&institution_code) {
        0 // PublicInstitution
    } else {
        2 // 仅作为未知/个人路由的展示兜底；个人多签不走本解码器。
    };

    Ok(AdminAccountDecoded {
        institution_code,
        kind,
        admins,
        creator_hex: String::new(),
        created_at: 0,
        updated_at: 0,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_compact_matches_scale_small_values() {
        assert_eq!(encode_compact_u32(0), vec![0x00]);
        assert_eq!(encode_compact_u32(1), vec![0x04]);
        assert_eq!(encode_compact_u32(64), vec![0x01, 0x01]);
    }

    // 机构管理员链上值已收口为 entity 外部的纯账户集合；旧 AdminProfile 金标向量已删除。
    // 岗位/任职 SCALE 金标由 public-manage/entity 测试维护，避免在节点端复制第二份模型。
    #[test]
    fn compact_encoder_matches_account_list_prefix() {
        assert_eq!(encode_admins(&["aa".repeat(32)]).unwrap()[0], 0x04);
    }

    /* 旧 AdminProfile 金标已随 admins/entity 模型迁移删除。
    // 逐字段对齐。任一字段序漂移(改 admin-primitives 布局)→ encode 字节变 → 本测试红。
    use admin_primitives::{
        AdminAccount, AdminAccountKind, AdminAccountStatus, AdminProfile, AdminSource,
    };
    use codec::Encode;
    use frame_support::traits::ConstU32;
    use frame_support::BoundedVec;
    use sp_runtime::AccountId32;

    #[test]
    fn decode_admin_account_matches_admin_primitives_encode() {
        use primitives::cid::code::NRC;
        let profile = AdminProfile::<AccountId32> {
            admin_account: AccountId32::new([0xaa; 32]),
            admin_cid_number: b"CID-001".to_vec().try_into().unwrap(),
            admin_name: "张三".as_bytes().to_vec().try_into().unwrap(),
            role_code: b"R01".to_vec().try_into().unwrap(),
            role_name: "委员".as_bytes().to_vec().try_into().unwrap(),
            term_start: 7,
            term_end: 9,
            admin_source: AdminSource::Registry,
            admin_source_ref: b"prop-42".to_vec().try_into().unwrap(),
        };
        let profiles: BoundedVec<AdminProfile<AccountId32>, ConstU32<64>> =
            vec![profile].try_into().unwrap();
        let account = AdminAccount::<_, AccountId32, u32> {
            cid_number: b"NRC-GENESIS".to_vec().try_into().unwrap(),
            institution_code: NRC,
            kind: AdminAccountKind::PublicInstitution,
            admins: profiles,
            creator: AccountId32::new([0xbb; 32]),
            created_at: 100,
            updated_at: 200,
            status: AdminAccountStatus::Active,
        };

        let decoded = decode_admin_account(&account.encode()).unwrap();
        assert_eq!(decoded.institution_code, NRC);
        assert_eq!(decoded.kind, 0); // PublicInstitution
        assert_eq!(decoded.admins.len(), 1);
        assert_eq!(decoded.admins[0].account, "aa".repeat(32));
        assert_eq!(decoded.admins[0].admin_cid_number, "CID-001");
        assert_eq!(decoded.admins[0].name, "张三");
        // 展示 admin_role 取链上 role_name(对外岗位名称),不是 role_code。
        assert_eq!(decoded.admins[0].admin_role, "委员");
        assert_eq!(decoded.admins[0].term_start, 7);
        assert_eq!(decoded.admins[0].term_end, 9);
        assert_eq!(decoded.admins[0].source, 1); // Registry
        assert_eq!(decoded.creator_hex, "bb".repeat(32));
        assert_eq!(decoded.created_at, 100);
        assert_eq!(decoded.updated_at, 200);
        assert_eq!(decoded.status, 1); // Active
    }

    #[test]
    fn decode_personal_admin_account_keeps_account_only_layout() {
        use primitives::cid::code::PMUL;
        // 个人多签 admins 是裸 AccountId 列表(无 profile),cid_number 为空。
        let admins: BoundedVec<AccountId32, ConstU32<64>> =
            vec![AccountId32::new([0xaa; 32]), AccountId32::new([0xcc; 32])]
                .try_into()
                .unwrap();
        let account = AdminAccount::<_, AccountId32, u32> {
            cid_number: Vec::new().try_into().unwrap(),
            institution_code: PMUL,
            kind: AdminAccountKind::PersonalMultisig,
            admins,
            creator: AccountId32::new([0xbb; 32]),
            created_at: 1,
            updated_at: 2,
            status: AdminAccountStatus::Active,
        };

        let decoded = decode_admin_account(&account.encode()).unwrap();
        assert_eq!(decoded.kind, 2); // PersonalMultisig
        assert_eq!(decoded.admins.len(), 2);
        assert_eq!(decoded.admins[0].account, "aa".repeat(32));
        assert_eq!(decoded.admins[0].admin_role, ""); // 无 profile
        assert_eq!(decoded.admins[1].account, "cc".repeat(32));
        assert_eq!(decoded.creator_hex, "bb".repeat(32));
        assert_eq!(decoded.status, 1);
    }
    */
}
