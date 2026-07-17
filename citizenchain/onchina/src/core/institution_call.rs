//! `propose_create_institution` SCALE call-data 编码器。
//!
//! OnChina 只构造裸 call data，不提交 extrinsic。首次登记只编码机构最小身份和
//! `admins(admin_name + admin_account)`；协议账户、法定代表人岗位和阈值由 runtime 派生。

use codec::{Compact, Encode};

/// PublicManage pallet 索引。
pub const PUBLIC_MANAGE_PALLET_INDEX: u8 = 30;
/// PrivateManage pallet 索引。
pub const PRIVATE_MANAGE_PALLET_INDEX: u8 = 31;
/// 公私权机构创建 call index。
pub const PROPOSE_CREATE_INSTITUTION_CALL_INDEX: u8 = 5;
/// 机构内部治理提案 call index。
#[allow(dead_code)]
pub const PROPOSE_INSTITUTION_GOVERNANCE_CALL_INDEX: u8 = 8;
/// 注册局登记机构管理员集合 call index。
#[allow(dead_code)]
pub const REGISTER_INSTITUTION_ADMINS_CALL_INDEX: u8 = 9;

/// 按机构码派生机构创建目标 pallet。
pub fn create_institution_pallet_index(institution_code: &[u8; 4]) -> u8 {
    if primitives::cid::code::is_private_legal_code(institution_code) {
        PRIVATE_MANAGE_PALLET_INDEX
    } else {
        PUBLIC_MANAGE_PALLET_INDEX
    }
}

/// 首次登记管理员人员记录。
#[derive(Debug, Clone)]
pub struct InstitutionAdminArg {
    pub admin_name: Vec<u8>,
    pub admin_account: [u8; 32],
}

/// `propose_create_{public,private}_institution` 完整参数。
#[derive(Debug, Clone)]
pub struct ProposeCreateInstitutionArgs {
    pub cid_number: Vec<u8>,
    pub cid_full_name: Vec<u8>,
    pub cid_short_name: Vec<u8>,
    pub town_code: Vec<u8>,
    pub admins: Vec<InstitutionAdminArg>,
    /// 只用于选择 public/private pallet，不编码进 runtime call。
    pub institution_code: [u8; 4],
    pub register_nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub actor_cid_number: Vec<u8>,
    pub credential_signer_pubkey: [u8; 32],
    pub scope_province_name: Vec<u8>,
    pub scope_city_name: Vec<u8>,
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

/// 构造与 runtime `BoundedVec<InstitutionAdmin>` 完全一致的签名与 call 载荷。
pub fn encode_admins_payload(admins: &[InstitutionAdminArg]) -> Vec<u8> {
    let mut out = Compact(admins.len() as u32).encode();
    for admin in admins {
        encode_bytes(&mut out, &admin.admin_name);
        out.extend_from_slice(&admin.admin_account);
    }
    out
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

/// 编码机构创建调用。字段顺序与 runtime call index 5 完全一致。
pub fn encode_propose_create_institution(args: &ProposeCreateInstitutionArgs) -> ChainCall {
    let pallet_index = create_institution_pallet_index(&args.institution_code);
    let mut out = vec![pallet_index, PROPOSE_CREATE_INSTITUTION_CALL_INDEX];

    encode_bytes(&mut out, &args.cid_number);
    encode_bytes(&mut out, &args.cid_full_name);
    encode_bytes(&mut out, &args.cid_short_name);
    encode_bytes(&mut out, &args.town_code);
    out.extend(encode_admins_payload(&args.admins));
    encode_bytes(&mut out, &args.register_nonce);
    encode_bytes(&mut out, &args.signature);
    encode_bytes(&mut out, &args.actor_cid_number);
    out.extend_from_slice(&args.credential_signer_pubkey);
    encode_bytes(&mut out, &args.scope_province_name);
    encode_bytes(&mut out, &args.scope_city_name);

    ChainCall {
        action: chain_action_code(pallet_index, PROPOSE_CREATE_INSTITUTION_CALL_INDEX),
        call_data: out,
    }
}

/// 编码机构内部治理提案调用。字段顺序与 runtime call index 8 完全一致。
#[allow(dead_code)]
pub fn encode_propose_institution_governance(args: &ProposeInstitutionGovernanceArgs) -> ChainCall {
    let pallet_index = create_institution_pallet_index(&args.institution_code);
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
    let pallet_index = create_institution_pallet_index(&args.institution_code);
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
    #[test]
    fn admin_payload_encodes_name_then_account() {
        let admins = vec![
            InstitutionAdminArg {
                admin_name: "张三".as_bytes().to_vec(),
                admin_account: [1; 32],
            },
            InstitutionAdminArg {
                admin_name: "管理员".as_bytes().to_vec(),
                admin_account: [2; 32],
            },
        ];
        let expected = admins
            .iter()
            .map(|admin| (admin.admin_name.clone(), admin.admin_account))
            .collect::<Vec<_>>();
        assert_eq!(encode_admins_payload(&admins), expected.encode());
    }

    #[test]
    fn minimal_create_payload_matches_runtime_call_field_order() {
        let args = ProposeCreateInstitutionArgs {
            cid_number: b"GD001-COMPANY-0001".to_vec(),
            cid_full_name: "测试机构".as_bytes().to_vec(),
            cid_short_name: "测试".as_bytes().to_vec(),
            town_code: Vec::new(),
            admins: vec![
                InstitutionAdminArg {
                    admin_name: "张三".as_bytes().to_vec(),
                    admin_account: [1; 32],
                },
                InstitutionAdminArg {
                    admin_name: "管理员".as_bytes().to_vec(),
                    admin_account: [2; 32],
                },
            ],
            institution_code: *b"SFLP",
            register_nonce: b"nonce".to_vec(),
            signature: vec![3; 64],
            actor_cid_number: b"issuer".to_vec(),
            credential_signer_pubkey: [5; 32],
            scope_province_name: "广东省".as_bytes().to_vec(),
            scope_city_name: "广州市".as_bytes().to_vec(),
        };
        let encoded = encode_propose_create_institution(&args);
        assert_eq!(&encoded.call_data[..2], &[31, 5]);
        assert_eq!(encoded.action, 0x1f05);

        let mut expected = vec![31, 5];
        expected.extend(args.cid_number.encode());
        expected.extend(args.cid_full_name.encode());
        expected.extend(args.cid_short_name.encode());
        expected.extend(args.town_code.encode());
        expected.extend(encode_admins_payload(&args.admins));
        expected.extend(args.register_nonce.encode());
        expected.extend(args.signature.encode());
        expected.extend(args.actor_cid_number.encode());
        expected.extend(args.credential_signer_pubkey.encode());
        expected.extend(args.scope_province_name.encode());
        expected.extend(args.scope_city_name.encode());
        assert_eq!(encoded.call_data, expected);
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
            admins: vec![
                InstitutionAdminArg {
                    admin_name: "张三".as_bytes().to_vec(),
                    admin_account: [1; 32],
                },
                InstitutionAdminArg {
                    admin_name: "李四".as_bytes().to_vec(),
                    admin_account: [2; 32],
                },
            ],
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
