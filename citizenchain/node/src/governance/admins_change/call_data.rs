use super::{codec, subject_id};

pub const ADMINS_CHANGE_PALLET_INDEX: u8 = 12;
pub const PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX: u8 = 0;

/// 构造 `AdminsChange::propose_admin_set_change` 的完整 call data。
pub fn build_admin_set_change_call_data(
    org: u8,
    subject_id: &[u8; 48],
    new_admins: &[String],
) -> Result<Vec<u8>, String> {
    let encoded_admins = codec::encode_admins(new_admins)?;
    let mut call_data = Vec::with_capacity(2 + 1 + 48 + encoded_admins.len());
    call_data.push(ADMINS_CHANGE_PALLET_INDEX);
    call_data.push(PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
    call_data.push(org);
    call_data.extend_from_slice(subject_id);
    call_data.extend_from_slice(&encoded_admins);
    Ok(call_data)
}

pub fn normalize_admins(admins: &[String]) -> Result<Vec<String>, String> {
    admins
        .iter()
        .map(|item| subject_id::normalize_pubkey_hex(item))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_admin_set_change_call_prefix() {
        let subject_id = [0x11u8; 48];
        let admins = vec!["22".repeat(32)];
        let call = build_admin_set_change_call_data(0, &subject_id, &admins).unwrap();
        assert_eq!(call[0], 12);
        assert_eq!(call[1], 0);
        assert_eq!(call[2], 0);
        assert_eq!(&call[3..51], &[0x11u8; 48]);
        assert_eq!(call[51], 0x04);
    }

    #[test]
    fn builds_dynamic_org_admin_set_change_call_prefix() {
        let subject_id = [0x55u8; 48];
        let admins = vec!["66".repeat(32), "77".repeat(32)];

        for org in [4u8, 5u8] {
            let call = build_admin_set_change_call_data(org, &subject_id, &admins).unwrap();
            assert_eq!(call[0], ADMINS_CHANGE_PALLET_INDEX);
            assert_eq!(call[1], PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
            assert_eq!(call[2], org);
            assert_eq!(&call[3..51], &[0x55u8; 48]);
        }
    }
}
