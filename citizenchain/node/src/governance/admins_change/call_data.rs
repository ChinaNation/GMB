use primitives::count_const::{
    NRC_INTERNAL_THRESHOLD, PRB_INTERNAL_THRESHOLD, PRC_INTERNAL_THRESHOLD,
};

use super::{account_id, codec};

pub const ADMINS_CHANGE_PALLET_INDEX: u8 = 12;
pub const PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX: u8 = 0;

/// 构造 `AdminsChange::propose_admin_set_change` 的完整 call data。
pub fn build_admin_set_change_call_data(
    org: u8,
    account_id: &[u8; 32],
    new_admins: &[String],
) -> Result<Vec<u8>, String> {
    let encoded_admins = codec::encode_admins(new_admins)?;
    let new_threshold = admin_change_threshold(org, new_admins.len())?;
    let mut call_data = Vec::with_capacity(2 + 1 + 32 + encoded_admins.len() + 4);
    call_data.push(ADMINS_CHANGE_PALLET_INDEX);
    call_data.push(PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
    call_data.push(org);
    call_data.extend_from_slice(account_id);
    call_data.extend_from_slice(&encoded_admins);
    call_data.extend_from_slice(&new_threshold.to_le_bytes());
    Ok(call_data)
}

fn admin_change_threshold(org: u8, admin_count: usize) -> Result<u32, String> {
    match org {
        0 => Ok(NRC_INTERNAL_THRESHOLD),
        1 => Ok(PRC_INTERNAL_THRESHOLD),
        2 => Ok(PRB_INTERNAL_THRESHOLD),
        3..=5 => {
            if admin_count == 0 {
                return Err("管理员数量不能为空".to_string());
            }
            Ok((admin_count as u32 / 2) + 1)
        }
        _ => Err("org 必须在 0..=5 范围内".to_string()),
    }
}

pub fn normalize_admins(admins: &[String]) -> Result<Vec<String>, String> {
    admins
        .iter()
        .map(|item| account_id::normalize_pubkey_hex(item))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_admin_set_change_call_prefix() {
        let account_id = [0x11u8; 32];
        let admins = vec!["22".repeat(32)];
        let call = build_admin_set_change_call_data(0, &account_id, &admins).unwrap();
        assert_eq!(call[0], 12);
        assert_eq!(call[1], 0);
        assert_eq!(call[2], 0);
        assert_eq!(&call[3..35], &[0x11u8; 32]);
        assert_eq!(call[35], 0x04);
        assert_eq!(&call[68..72], &NRC_INTERNAL_THRESHOLD.to_le_bytes());
    }

    #[test]
    fn builds_dynamic_org_admin_set_change_call_prefix() {
        let account_id = [0x55u8; 32];
        let admins = vec!["66".repeat(32), "77".repeat(32)];

        for org in [4u8, 5u8] {
            let call = build_admin_set_change_call_data(org, &account_id, &admins).unwrap();
            assert_eq!(call[0], ADMINS_CHANGE_PALLET_INDEX);
            assert_eq!(call[1], PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
            assert_eq!(call[2], org);
            assert_eq!(&call[3..35], &[0x55u8; 32]);
            assert_eq!(&call[100..104], &2u32.to_le_bytes());
        }
    }
}
