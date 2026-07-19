//! 机构治理 SCALE call-data 编码器。
//!
//! OnChina 只构造裸 call data，不提交 extrinsic。旧机构直接创建 call 5 已关闭；
//! 本模块只保留现存机构治理和注册局管理员登记调用。

use codec::{Compact, Encode};

/// PublicManage pallet 索引。
pub const PUBLIC_MANAGE_PALLET_INDEX: u8 = 30;
/// PrivateManage pallet 索引。
pub const PRIVATE_MANAGE_PALLET_INDEX: u8 = 31;
/// 机构内部治理提案 call index。
#[allow(dead_code)]
pub const PROPOSE_INSTITUTION_GOVERNANCE_CALL_INDEX: u8 = 8;
/// 注册局登记机构管理员集合 call index。
#[allow(dead_code)]
pub const REGISTER_INSTITUTION_ADMINS_CALL_INDEX: u8 = 9;

/// 按机构码派生机构管理目标 pallet。
pub fn institution_manage_pallet_index(institution_code: &[u8; 4]) -> u8 {
    if primitives::cid::code::is_private_legal_code(institution_code) {
        PRIVATE_MANAGE_PALLET_INDEX
    } else {
        PUBLIC_MANAGE_PALLET_INDEX
    }
}

/// 机构治理和登记使用的管理员人员记录。
#[derive(Debug, Clone, Encode)]
pub struct InstitutionAdminArg {
    pub admin_account: [u8; 32],
    pub family_name: admin_primitives::FamilyName,
    pub given_name: admin_primitives::GivenName,
}

/// `propose_institution_governance` 完整参数。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProposeInstitutionGovernanceArgs {
    pub cid_number: Vec<u8>,
    /// `entity_primitives::InstitutionGovernanceAction` 的 SCALE 字节。
    pub governance_action: Vec<u8>,
    pub institution_code: [u8; 4],
    pub register_nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub actor_cid_number: Vec<u8>,
    pub credential_signer_pubkey: [u8; 32],
    pub scope_province_name: Vec<u8>,
    pub scope_city_name: Vec<u8>,
}

/// `register_institution_admins` 完整参数。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RegisterInstitutionAdminsArgs {
    pub cid_number: Vec<u8>,
    pub admins: Vec<InstitutionAdminArg>,
    pub institution_code: [u8; 4],
    pub register_nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub actor_cid_number: Vec<u8>,
    pub credential_signer_pubkey: [u8; 32],
    pub scope_province_name: Vec<u8>,
    pub scope_city_name: Vec<u8>,
}

fn encode_bytes(out: &mut Vec<u8>, value: &[u8]) {
    out.extend(Compact(value.len() as u32).encode());
    out.extend_from_slice(value);
}

/// 构造与 runtime `BoundedVec<Admin>` 完全一致的签名与 call 载荷。
pub fn encode_admins_payload(admins: &[InstitutionAdminArg]) -> Vec<u8> {
    admins.encode()
}

/// QR 链动作码：`(pallet_index << 8) | call_index`。
pub const fn chain_action_code(pallet_index: u8, call_index: u8) -> u16 {
    ((pallet_index as u16) << 8) | call_index as u16
}

/// 一条待冷签链调用。
pub struct ChainCall {
    pub action: u16,
    pub call_data: Vec<u8>,
}

/// 编码机构内部治理提案调用。字段顺序与 runtime call index 8 完全一致。
#[allow(dead_code)]
pub fn encode_propose_institution_governance(args: &ProposeInstitutionGovernanceArgs) -> ChainCall {
    let pallet_index = institution_manage_pallet_index(&args.institution_code);
    let mut out = vec![pallet_index, PROPOSE_INSTITUTION_GOVERNANCE_CALL_INDEX];

    encode_bytes(&mut out, &args.cid_number);
    out.extend_from_slice(&args.governance_action);
    encode_bytes(&mut out, &args.register_nonce);
    encode_bytes(&mut out, &args.signature);
    encode_bytes(&mut out, &args.actor_cid_number);
    out.extend_from_slice(&args.credential_signer_pubkey);
    encode_bytes(&mut out, &args.scope_province_name);
    encode_bytes(&mut out, &args.scope_city_name);

    ChainCall {
        action: chain_action_code(pallet_index, PROPOSE_INSTITUTION_GOVERNANCE_CALL_INDEX),
        call_data: out,
    }
}

/// 编码注册局登记机构管理员集合调用。字段顺序与 runtime call index 9 完全一致。
#[allow(dead_code)]
pub fn encode_register_institution_admins(args: &RegisterInstitutionAdminsArgs) -> ChainCall {
    let pallet_index = institution_manage_pallet_index(&args.institution_code);
    let mut out = vec![pallet_index, REGISTER_INSTITUTION_ADMINS_CALL_INDEX];

    encode_bytes(&mut out, &args.cid_number);
    out.extend(encode_admins_payload(&args.admins));
    encode_bytes(&mut out, &args.register_nonce);
    encode_bytes(&mut out, &args.signature);
    encode_bytes(&mut out, &args.actor_cid_number);
    out.extend_from_slice(&args.credential_signer_pubkey);
    encode_bytes(&mut out, &args.scope_province_name);
    encode_bytes(&mut out, &args.scope_city_name);

    ChainCall {
        action: chain_action_code(pallet_index, REGISTER_INSTITUTION_ADMINS_CALL_INDEX),
        call_data: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin(admin_account: [u8; 32], family_name: &str, given_name: &str) -> InstitutionAdminArg {
        InstitutionAdminArg {
            admin_account,
            family_name: family_name
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("family name fits"),
            given_name: given_name
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("given name fits"),
        }
    }

    #[test]
    fn admin_payload_encodes_account_family_name_and_given_name() {
        let admins = vec![admin([1; 32], "张", "三"), admin([2; 32], "管理", "员")];
        let expected = admins
            .iter()
            .map(|admin| {
                (
                    admin.admin_account,
                    admin.family_name.clone(),
                    admin.given_name.clone(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(encode_admins_payload(&admins), expected.encode());
    }

    #[test]
    fn governance_payload_matches_runtime_call_field_order() {
        let args = ProposeInstitutionGovernanceArgs {
            cid_number: b"LN001-SFAS-0001".to_vec(),
            governance_action: vec![0, 8, b'A'],
            institution_code: *b"SFAS",
            register_nonce: b"nonce".to_vec(),
            signature: vec![9; 64],
            actor_cid_number: b"LN001-SFAS-0001".to_vec(),
            credential_signer_pubkey: [5; 32],
            scope_province_name: "广东省".as_bytes().to_vec(),
            scope_city_name: "广州市".as_bytes().to_vec(),
        };
        let encoded = encode_propose_institution_governance(&args);
        assert_eq!(&encoded.call_data[..2], &[31, 8]);
        assert_eq!(encoded.action, 0x1f08);
    }

    #[test]
    fn register_admins_payload_matches_runtime_call_field_order() {
        let args = RegisterInstitutionAdminsArgs {
            cid_number: b"LN001-SFAS-0001".to_vec(),
            admins: vec![admin([1; 32], "张", "三"), admin([2; 32], "李", "四")],
            institution_code: *b"SFAS",
            register_nonce: b"nonce".to_vec(),
            signature: vec![9; 64],
            actor_cid_number: b"LN001-FRG0-0001".to_vec(),
            credential_signer_pubkey: [5; 32],
            scope_province_name: "广东省".as_bytes().to_vec(),
            scope_city_name: "广州市".as_bytes().to_vec(),
        };
        let encoded = encode_register_institution_admins(&args);
        assert_eq!(&encoded.call_data[..2], &[31, 9]);
        assert_eq!(encoded.action, 0x1f09);
    }
}
