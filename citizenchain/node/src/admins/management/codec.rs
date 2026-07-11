use super::account_id::normalize_pubkey_hex;
use super::types::{source_label, AdminAccountDecoded, AdminProfileInfo};

/// 解码各管理员 pallet `AdminAccounts` 的完整核心字段。
///
/// 链上布局(逐字段对齐 `admin-primitives::AdminAccount`,SCALE 按声明序):
/// cid_number:BoundedVec<u8>(Compact 长度 + 字节)+ institution_code:[u8;4] + kind:u8
/// + admins:BoundedVec<AdminProfile> + creator:AccountId32 + created_at:u32
/// + updated_at:u32 + status:u8。
///
/// 单个 `AdminProfile`(kind≠2):admin_account:[u8;32] + admin_cid_number + admin_name
/// + role_code + role_name(四个 `BoundedVec<u8>` 均 Compact 长度 + 字节)+ term_start:u32
/// + term_end:u32 + admin_source:u8 + admin_source_ref:BoundedVec<u8>。
/// 个人多签(kind=2)的 admins 仍是裸 `BoundedVec<AccountId32>`,无 profile 附加字段。
///
/// 展示层 `admin_role` 取链上 `role_name`(对外岗位名称);`role_code`/`admin_source_ref`
/// 展示层不用,仅解析以对齐后续字段偏移。头部 `cid_number` 同理跳过(展示用查询入参)。
/// created_at/updated_at 是 BlockNumberFor<T>,citizenchain runtime 配置为 u32。
pub fn decode_admin_account(data: &[u8]) -> Result<AdminAccountDecoded, String> {
    // 头部 cid_number: BoundedVec<u8> = Compact(len) + bytes。个人多签无机构 cid → Compact(0)。
    // 展示层用查询入参里的 cid,这里只需跳过以对齐后续字段偏移。
    let (cid_len, cid_len_size) = read_compact_u32(data, 0)?;
    let mut offset = cid_len_size + cid_len as usize;

    // institution_code: [u8;4] 定长(4 裸字节无长度前缀)+ kind: u8。
    if offset + 4 + 1 > data.len() {
        return Err("AdminAccount 机构码/kind 数据不足".to_string());
    }
    let institution_code: [u8; 4] = data[offset..offset + 4]
        .try_into()
        .map_err(|_| "AdminAccount 机构码数据不足".to_string())?;
    offset += 4;
    let kind = data[offset];
    offset += 1;

    let (count, len_size) = read_compact_u32(data, offset)?;
    offset += len_size;
    let mut admins = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if offset + 32 > data.len() {
            return Err("AdminAccount 管理员列表数据不足".to_string());
        }
        let account = hex::encode(&data[offset..offset + 32]);
        offset += 32;
        if kind == 2 {
            admins.push(AdminProfileInfo::account_only(account));
            continue;
        }
        // AdminProfile 四字符串:admin_cid_number / admin_name / role_code / role_name。
        let (admin_cid_number, next) = read_compact_string(data, offset, "admin_cid_number")?;
        offset = next;
        let (name, next) = read_compact_string(data, offset, "admin_name")?;
        offset = next;
        // role_code 仅内部引用,展示层不用;解析以对齐偏移。
        let (_role_code, next) = read_compact_string(data, offset, "role_code")?;
        offset = next;
        // role_name = 对外岗位名称,展示层 admin_role 取此字段。
        let (role_name, next) = read_compact_string(data, offset, "role_name")?;
        offset = next;
        if offset + 4 + 4 + 1 > data.len() {
            return Err("AdminProfile 任期/来源数据不足".to_string());
        }
        let term_start = read_u32_le(data, offset);
        offset += 4;
        let term_end = read_u32_le(data, offset);
        offset += 4;
        let source = data[offset];
        offset += 1;
        // admin_source_ref: 尾部来源追溯串;展示层不用,解析以对齐偏移。
        let (_admin_source_ref, next) = read_compact_string(data, offset, "admin_source_ref")?;
        offset = next;
        admins.push(AdminProfileInfo {
            account,
            admin_cid_number,
            name,
            admin_role: role_name,
            term_start,
            term_end,
            source,
            source_label: source_label(source).to_string(),
        });
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
        institution_code,
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

fn read_compact_string(
    data: &[u8],
    offset: usize,
    field_name: &str,
) -> Result<(String, usize), String> {
    let (len, len_size) = read_compact_u32(data, offset)?;
    let start = offset + len_size;
    let end = start + len as usize;
    if end > data.len() {
        return Err(format!("AdminProfile {field_name} 数据不足"));
    }
    Ok((String::from_utf8_lossy(&data[start..end]).to_string(), end))
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

    // ── 金标向量:直接 encode 真链上类型 `admin-primitives::AdminAccount`,喂给解码器断言 ──
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
}
