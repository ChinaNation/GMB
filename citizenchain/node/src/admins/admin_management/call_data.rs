use primitives::cid::code::{
    fixed_governance_pass_threshold, is_registered_multisig_code, InstitutionCode, FRG,
};

use super::storage::admin_pallet_for_code;
use super::{account_id, codec};

pub const PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX: u8 = 0;
pub const PROPOSE_PERSONAL_ADMIN_SET_CHANGE_CALL_INDEX: u8 = 3;

/// 构造对应管理员 pallet 的 `propose_admin_set_change` 完整 call data。
pub fn build_admin_set_change_call_data(
    institution_code: &InstitutionCode,
    account_id: &[u8; 32],
    admins: &[String],
) -> Result<Vec<u8>, String> {
    if *institution_code == FRG {
        return Err("联邦注册局管理员更换必须走 OnChina 省级 5 人组流程".to_string());
    }
    let encoded_admins = codec::encode_admins(admins)?;
    let new_threshold = admin_change_threshold(institution_code, admins.len())?;
    let admin_pallet = admin_pallet_for_code(institution_code)?;
    let call_index = if primitives::cid::code::is_personal_code(institution_code) {
        PROPOSE_PERSONAL_ADMIN_SET_CHANGE_CALL_INDEX
    } else {
        PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX
    };
    let mut call_data = Vec::with_capacity(2 + 4 + 32 + encoded_admins.len() + 4);
    call_data.push(admin_pallet.pallet_index);
    call_data.push(call_index);
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
    use primitives::cid::code::{code_bytes, NRC, PMUL};
    use primitives::count_const::NRC_INTERNAL_THRESHOLD;

    const GENESIS_ADMINS_PALLET_INDEX: u8 = 12;

    #[test]
    fn builds_admin_set_change_call_prefix() {
        let account_id = [0x11u8; 32];
        let admins = vec!["22".repeat(32)];
        let call = build_admin_set_change_call_data(&NRC, &account_id, &admins).unwrap();
        assert_eq!(call[0], GENESIS_ADMINS_PALLET_INDEX);
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

        let public_code = code_bytes("CGOV");
        let public_call =
            build_admin_set_change_call_data(&public_code, &account_id, &admins).unwrap();
        assert_eq!(public_call[0], 29);
        assert_eq!(public_call[1], PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
        assert_eq!(&public_call[2..6], &public_code);
        assert_eq!(&public_call[6..38], &[0x55u8; 32]);
        // 2 个管理员:threshold = 2/2 + 1 = 2。
        // 偏移 = 2(prefix) + 4(code) + 32(account) + 1(compact len) + 2×32(admins) = 103。
        assert_eq!(&public_call[103..107], &2u32.to_le_bytes());

        let private_code = code_bytes("SFLP");
        let private_call =
            build_admin_set_change_call_data(&private_code, &account_id, &admins).unwrap();
        assert_eq!(private_call[0], 30);
        assert_eq!(private_call[1], PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX);
        assert_eq!(&private_call[2..6], &private_code);
        assert_eq!(&private_call[6..38], &[0x55u8; 32]);
        assert_eq!(&private_call[103..107], &2u32.to_le_bytes());
    }

    #[test]
    fn builds_personal_admin_set_change_call_prefix() {
        let account_id = [0x55u8; 32];
        let admins = vec!["66".repeat(32), "77".repeat(32)];

        let call = build_admin_set_change_call_data(&PMUL, &account_id, &admins).unwrap();
        assert_eq!(call[0], 7);
        assert_eq!(call[1], PROPOSE_PERSONAL_ADMIN_SET_CHANGE_CALL_INDEX);
        assert_eq!(&call[2..6], &PMUL);
        assert_eq!(&call[6..38], &[0x55u8; 32]);
        assert_eq!(&call[103..107], &2u32.to_le_bytes());
    }

    #[test]
    fn rejects_federal_registry_generic_admin_set_change_call() {
        let account_id = [0x55u8; 32];
        let admins = vec!["66".repeat(32), "77".repeat(32)];

        let err = build_admin_set_change_call_data(&FRG, &account_id, &admins).unwrap_err();
        assert_eq!(err, "联邦注册局管理员更换必须走 OnChina 省级 5 人组流程");
    }
}
