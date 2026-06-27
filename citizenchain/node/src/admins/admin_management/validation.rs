use std::collections::BTreeSet;

use primitives::cid::code::{
    is_personal_code, is_private_legal_code, is_public_legal_code, is_unincorporated_code,
    InstitutionCode, NRC, PRB, PRC,
};

use super::call_data::normalize_admins;
use super::types::AdminAccountState;

const FRG: InstitutionCode = *b"FRG\0";

/// 桌面端前置校验。链端仍是最终裁判，这里只提前给用户明确错误。
pub fn validate_admin_set_change(
    state: &AdminAccountState,
    proposer_pubkey_hex: &str,
    admins: &[String],
) -> Result<Vec<String>, String> {
    if state.status != 1 {
        return Err("管理员账户不是已激活状态，不能发起更换".to_string());
    }
    let proposer = super::account_id::normalize_pubkey_hex(proposer_pubkey_hex)?;
    if !state.admins.iter().any(|admin| admin == &proposer) {
        return Err("当前签名账户不是该账户管理员，不能发起管理员更换".to_string());
    }

    let normalized = normalize_admins(admins)?;
    validate_count(state.kind, &state.institution_code, normalized.len())?;

    let mut seen = BTreeSet::new();
    for admin in &normalized {
        if !seen.insert(admin.clone()) {
            return Err("新管理员列表存在重复公钥".to_string());
        }
    }

    let current: BTreeSet<_> = state.admins.iter().cloned().collect();
    let next: BTreeSet<_> = normalized.iter().cloned().collect();
    if current == next {
        return Err("新管理员集合与当前管理员集合没有变化".to_string());
    }
    Ok(normalized)
}

fn validate_count(
    kind: u8,
    institution_code: &InstitutionCode,
    count: usize,
) -> Result<(), String> {
    match kind {
        0 => {
            // 管理员人数单一真源 = primitives::count_const,桌面端不再硬编码。
            use primitives::count_const::{NRC_ADMIN_COUNT, PRB_ADMIN_COUNT, PRC_ADMIN_COUNT};
            let expected = match *institution_code {
                NRC => Some(NRC_ADMIN_COUNT as usize),
                PRC => Some(PRC_ADMIN_COUNT as usize),
                PRB => Some(PRB_ADMIN_COUNT as usize),
                FRG => None,
                _ => return Err("创世管理员机构码无效".to_string()),
            };
            if let Some(expected) = expected {
                if count != expected {
                    return Err(format!("创世治理机构管理员数量必须保持 {expected} 人"));
                }
            } else if !(1..=1989).contains(&count) {
                return Err("联邦注册局管理员数量必须在 1..=1989 之间".to_string());
            }
        }
        1 => {
            if !is_public_legal_code(institution_code)
                || matches!(*institution_code, NRC | PRC | PRB | FRG)
            {
                return Err("公权机构管理员更换必须使用非创世公权机构码".to_string());
            }
            if !(2..=1989).contains(&count) {
                return Err("公权机构管理员数量必须在 2..=1989 之间".to_string());
            }
        }
        2 => {
            if !(is_private_legal_code(institution_code)
                || is_unincorporated_code(institution_code))
            {
                return Err("私权机构管理员更换必须使用私权或非法人机构码".to_string());
            }
            if !(2..=1989).contains(&count) {
                return Err("私权机构管理员数量必须在 2..=1989 之间".to_string());
            }
        }
        3 => {
            if !is_personal_code(institution_code) {
                return Err("个人多签管理员更换必须使用个人多签机构码".to_string());
            }
            if !(2..=64).contains(&count) {
                return Err("个人多签管理员数量必须在 2..=64 之间".to_string());
            }
        }
        _ => return Err("未知管理员账户类型".to_string()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use primitives::cid::code::{code_bytes, PMUL};

    fn admin(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn state(
        kind: u8,
        institution_code: InstitutionCode,
        admins: Vec<String>,
    ) -> AdminAccountState {
        AdminAccountState {
            account_hex: "11".repeat(32),
            cid_number: Some("TEST-CID".to_string()),
            institution_code,
            institution_code_label: String::new(),
            kind,
            kind_label: String::new(),
            admins,
            creator_hex: admin(9),
            created_at: 1,
            updated_at: 1,
            status: 1,
            status_label: "已激活".to_string(),
        }
    }

    #[test]
    fn personal_account_requires_personal_code() {
        let current = vec![admin(1), admin(2)];
        let next = vec![admin(1), admin(3)];
        // kind=3(个人多签)但机构码是机构账户码 → 拒绝。
        let err =
            validate_admin_set_change(&state(3, code_bytes("CGOV"), current), &admin(1), &next)
                .unwrap_err();
        assert_eq!(err, "个人多签管理员更换必须使用个人多签机构码");
    }

    #[test]
    fn institution_account_requires_institution_code() {
        let current = vec![admin(1), admin(2)];
        let next = vec![admin(1), admin(3)];

        validate_admin_set_change(
            &state(1, code_bytes("CGOV"), current.clone()),
            &admin(1),
            &next,
        )
        .expect("公权机构账户应允许管理员更换");
        validate_admin_set_change(
            &state(2, code_bytes("SFLP"), current.clone()),
            &admin(1),
            &next,
        )
        .expect("私权机构账户应允许管理员更换");

        // kind=1(公权机构)但机构码是个人多签码 → 拒绝。
        let err =
            validate_admin_set_change(&state(1, PMUL, current), &admin(1), &next).unwrap_err();
        assert_eq!(err, "公权机构管理员更换必须使用非创世公权机构码");
    }
}
