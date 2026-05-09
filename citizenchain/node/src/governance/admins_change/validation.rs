use std::collections::BTreeSet;

use super::call_data::normalize_admins;
use super::types::AdminSubjectState;

/// 桌面端前置校验。链端仍是最终裁判，这里只提前给用户明确错误。
pub fn validate_admin_set_change(
    state: &AdminSubjectState,
    proposer_pubkey_hex: &str,
    new_admins: &[String],
) -> Result<Vec<String>, String> {
    if state.status != 1 {
        return Err("管理员主体不是已激活状态，不能发起更换".to_string());
    }
    let proposer = super::subject_id::normalize_pubkey_hex(proposer_pubkey_hex)?;
    if !state.admins.iter().any(|admin| admin == &proposer) {
        return Err("当前签名账户不是该主体管理员，不能发起管理员更换".to_string());
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
        1 | 3 => {
            if !(2..=1989).contains(&count) {
                return Err("机构账户管理员数量必须在 2..=1989 之间".to_string());
            }
        }
        2 => {
            if !(2..=64).contains(&count) {
                return Err("个人多签管理员数量必须在 2..=64 之间".to_string());
            }
        }
        _ => return Err("未知管理员主体类型".to_string()),
    }
    Ok(())
}
