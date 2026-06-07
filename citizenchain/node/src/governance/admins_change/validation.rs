use std::collections::BTreeSet;

use super::call_data::normalize_admins;
use super::types::AdminAccountState;

/// 桌面端前置校验。链端仍是最终裁判，这里只提前给用户明确错误。
pub fn validate_admin_set_change(
    state: &AdminAccountState,
    proposer_pubkey_hex: &str,
    new_admins: &[String],
) -> Result<Vec<String>, String> {
    if state.status != 1 {
        return Err("管理员账户不是已激活状态，不能发起更换".to_string());
    }
    let proposer = super::account_id::normalize_pubkey_hex(proposer_pubkey_hex)?;
    if !state.admins.iter().any(|admin| admin == &proposer) {
        return Err("当前签名账户不是该账户管理员，不能发起管理员更换".to_string());
    }

    let normalized = normalize_admins(new_admins)?;
    validate_count(state.kind, state.org, normalized.len())?;

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

fn validate_count(kind: u8, org: u8, count: usize) -> Result<(), String> {
    match kind {
        0 => {
            let expected = match org {
                0 => 19,
                1 | 2 => 9,
                _ => return Err("内置治理机构 org 无效".to_string()),
            };
            if count != expected {
                return Err(format!("内置治理机构管理员数量必须保持 {expected} 人"));
            }
        }
        1 => {
            if org != 3 {
                return Err("个人多签管理员更换必须使用 ORG_REN".to_string());
            }
            if !(2..=64).contains(&count) {
                return Err("个人多签管理员数量必须在 2..=64 之间".to_string());
            }
        }
        2 => {
            if !matches!(org, 4 | 5) {
                return Err("机构账户管理员更换必须使用 ORG_PUP 或 ORG_OTH".to_string());
            }
            if !(2..=1989).contains(&count) {
                return Err("机构账户管理员数量必须在 2..=1989 之间".to_string());
            }
        }
        _ => return Err("未知管理员账户类型".to_string()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn state(kind: u8, org: u8, admins: Vec<String>) -> AdminAccountState {
        AdminAccountState {
            account_hex: "11".repeat(32),
            sfid_number: Some("TEST-SFID".to_string()),
            org,
            org_label: String::new(),
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
    fn personal_duoqian_requires_org_ren() {
        let current = vec![admin(1), admin(2)];
        let next = vec![admin(1), admin(3)];
        let err = validate_admin_set_change(&state(1, 4, current), &admin(1), &next).unwrap_err();
        assert_eq!(err, "个人多签管理员更换必须使用 ORG_REN");
    }

    #[test]
    fn institution_account_requires_org_pup_or_oth() {
        let current = vec![admin(1), admin(2)];
        let next = vec![admin(1), admin(3)];

        for org in [4u8, 5u8] {
            validate_admin_set_change(&state(2, org, current.clone()), &admin(1), &next)
                .expect("PUP/OTH 机构账户应允许管理员更换");
        }

        let err = validate_admin_set_change(&state(2, 3, current), &admin(1), &next).unwrap_err();
        assert_eq!(err, "机构账户管理员更换必须使用 ORG_PUP 或 ORG_OTH");
    }
}
