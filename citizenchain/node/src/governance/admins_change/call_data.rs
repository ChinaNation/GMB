use primitives::code::{
    fixed_governance_pass_threshold, is_registered_multisig_code, InstitutionCode,
};

use super::{account_id, codec};

pub const ADMINS_CHANGE_PALLET_INDEX: u8 = 12;
pub const PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX: u8 = 0;

/// 构造 `AdminsChange::propose_admin_set_change` 的完整 call data。
pub fn build_admin_set_change_call_data(
    institution_code: &InstitutionCode,
    account_id: &[u8; 32],
    admins: &[String],
) -> Result<Vec<u8>, String> {
    let encoded_admins = codec::encode_admins(admins)?;
    let new_threshold = admin_change_threshold(institution_code, admins.len())?;
    let mut call_data = Vec::with_capacity(2 + 4 + 32 + encoded_admins.len() + 4);
    call_data.push(ADMINS_CHANGE_PALLET_INDEX);
    call_data.push(PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
    // institution_code: InstitutionCode([u8; 4]) = 4 个裸字节，无长度前缀。
    call_data.extend_from_slice(institution_code);
    call_data.extend_from_slice(account_id);
    call_data.extend_from_slice(&encoded_admins);
    call_data.extend_from_slice(&new_threshold.to_le_bytes());
    Ok(call_data)
}

fn admin_change_threshold(
    institution_code: &InstitutionCode,
    admins_len: usize,
) -> Result<u32, String> {
    if let Some(threshold) = fixed_governance_pass_threshold(institution_code) {
        Ok(threshold)
    } else if is_registered_multisig_code(institution_code) {
        if admins_len == 0 {
            return Err("管理员数量不能为空".to_string());
        }
        Ok((admins_len as u32 / 2) + 1)
    } else {
        Err("非法机构码".to_string())
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
    use primitives::count_const::NRC_INTERNAL_THRESHOLD;
    use primitives::code::{code_bytes, NRC};

    #[test]
    fn builds_admin_set_change_call_prefix() {
        let account_id = [0x11u8; 32];
        let admins = vec!["22".repeat(32)];
        let call = build_admin_set_change_call_data(&NRC, &account_id, &admins).unwrap();
        assert_eq!(call[0], 12);
        assert_eq!(call[1], 0);
        // institution_code 4 字节 = b"NRC\0"。
        assert_eq!(&call[2..6], &NRC);
        assert_eq!(&call[6..38], &[0x11u8; 32]);
        // admins Compact<u32> 长度前缀(1 项 → 0x04)。
        assert_eq!(call[38], 0x04);
        assert_eq!(&call[71..75], &NRC_INTERNAL_THRESHOLD.to_le_bytes());
    }

    #[test]
    fn builds_dynamic_code_admin_set_change_call_prefix() {
        let account_id = [0x55u8; 32];
        let admins = vec!["66".repeat(32), "77".repeat(32)];

        for code in [code_bytes("CGOV"), code_bytes("SFLP")] {
            let call = build_admin_set_change_call_data(&code, &account_id, &admins).unwrap();
            assert_eq!(call[0], ADMINS_CHANGE_PALLET_INDEX);
            assert_eq!(call[1], PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
            assert_eq!(&call[2..6], &code);
            assert_eq!(&call[6..38], &[0x55u8; 32]);
            // 2 个管理员:threshold = 2/2 + 1 = 2。
            // 偏移 = 2(prefix) + 4(code) + 32(account) + 1(compact len) + 2×32(admins) = 103。
            assert_eq!(&call[103..107], &2u32.to_le_bytes());
        }
    }
}
