use admin_primitives::{Admin, InstitutionAdmins};
use codec::Decode;

use super::types::{AdminAccountDecoded, AdminDecoded};

/// 解码机构管理员模块的 `InstitutionAdmins`。
///
/// SCALE 布局固定为 `institution_code + admins(admin_account + family_name + given_name)`。
/// CID 只存在于 storage key，不在 value 中重复保存；岗位、任期和来源另查 entity。
pub fn decode_admin_account(data: &[u8]) -> Result<AdminAccountDecoded, String> {
    type RawInstitutionAdmins = InstitutionAdmins<Vec<Admin<[u8; 32]>>>;
    let mut input = data;
    let decoded = RawInstitutionAdmins::decode(&mut input)
        .map_err(|e| format!("InstitutionAdmins SCALE 解码失败: {e}"))?;
    if !input.is_empty() {
        return Err("InstitutionAdmins 存在尾随字节".to_string());
    }
    let admins = decoded
        .admins
        .into_iter()
        .map(|admin| {
            let family_name = String::from_utf8(admin.family_name.into_inner())
                .map_err(|_| "管理员 family_name 不是 UTF-8".to_string())?;
            let given_name = String::from_utf8(admin.given_name.into_inner())
                .map_err(|_| "管理员 given_name 不是 UTF-8".to_string())?;
            if family_name.is_empty() || given_name.is_empty() {
                return Err("管理员 family_name/given_name 不得为空".to_string());
            }
            Ok(AdminDecoded {
                admin_account: hex::encode(admin.admin_account),
                family_name,
                given_name,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(AdminAccountDecoded {
        institution_code: decoded.institution_code,
        admins,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn institution_admins_decodes_unified_admin_records() {
        use codec::Encode;
        let bytes = InstitutionAdmins {
            institution_code: *b"NRC\0",
            admins: vec![
                Admin {
                    admin_account: [0xaau8; 32],
                    family_name: admin_primitives::FamilyName::truncate_from(
                        "张".as_bytes().to_vec(),
                    ),
                    given_name: admin_primitives::GivenName::truncate_from(
                        "三".as_bytes().to_vec(),
                    ),
                },
                Admin {
                    admin_account: [0xbbu8; 32],
                    family_name: admin_primitives::FamilyName::truncate_from(
                        "管理".as_bytes().to_vec(),
                    ),
                    given_name: admin_primitives::GivenName::truncate_from(
                        "员".as_bytes().to_vec(),
                    ),
                },
            ],
        }
        .encode();
        let decoded = decode_admin_account(&bytes).unwrap();
        assert_eq!(decoded.institution_code, *b"NRC\0");
        assert_eq!(decoded.admins[0].admin_account, "aa".repeat(32));
        assert_eq!(decoded.admins[0].family_name, "张");
        assert_eq!(decoded.admins[0].given_name, "三");
        assert_eq!(decoded.admins[1].family_name, "管理");
        assert_eq!(decoded.admins[1].given_name, "员");
    }

    #[test]
    fn account_only_layout_and_empty_person_name_are_rejected() {
        use codec::Encode;

        let old_layout = (*b"NRC\0", vec![[0xaau8; 32]]).encode();
        assert!(decode_admin_account(&old_layout).is_err());

        let empty_name = InstitutionAdmins {
            institution_code: *b"NRC\0",
            admins: vec![Admin {
                admin_account: [0xaau8; 32],
                family_name: admin_primitives::FamilyName::truncate_from(Vec::new()),
                given_name: admin_primitives::GivenName::truncate_from("员".as_bytes().to_vec()),
            }],
        }
        .encode();
        assert!(decode_admin_account(&empty_name).is_err());
    }
}
