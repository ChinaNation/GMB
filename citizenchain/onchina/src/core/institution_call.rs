//! `propose_create_institution` SCALE call-data 编码器。
//!
//! OnChina 只构造裸 call data，不提交 extrinsic。机构初始管理员只通过
//! 机构岗位 `roles` 与管理员任职 `assignments` 编码，
//! `admins` 由 entity 从有效任职去重派生。

use codec::{Compact, Encode};

/// PublicManage pallet 索引。
pub const PUBLIC_MANAGE_PALLET_INDEX: u8 = 30;
/// PrivateManage pallet 索引。
pub const PRIVATE_MANAGE_PALLET_INDEX: u8 = 31;
/// 公私权机构创建 call index。
pub const PROPOSE_CREATE_INSTITUTION_CALL_INDEX: u8 = 5;

/// 按机构码派生机构创建目标 pallet。
pub fn create_institution_pallet_index(institution_code: &[u8; 4]) -> u8 {
    if primitives::cid::code::is_private_legal_code(institution_code) {
        PRIVATE_MANAGE_PALLET_INDEX
    } else {
        PUBLIC_MANAGE_PALLET_INDEX
    }
}

/// 机构岗位状态 SCALE 判别值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InstitutionRoleStatusTag {
    Active = 0,
    Inactive = 1,
}

/// 机构任职来源 SCALE 判别值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InstitutionAssignmentSourceTag {
    Genesis = 0,
    Registry = 1,
    PopularElection = 2,
    MutualElection = 3,
    NominationAppointment = 4,
}

/// 机构任职状态 SCALE 判别值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InstitutionAssignmentStatusTag {
    Active = 0,
    Ended = 1,
}

/// 单个机构初始账户。
#[derive(Debug, Clone)]
pub struct InitialAccountArg {
    pub account_name: String,
    pub amount: u128,
}

/// 单个机构岗位定义，对齐 `entity_primitives::InstitutionRole` 字段顺序。
#[derive(Debug, Clone)]
pub struct InstitutionRoleArg {
    pub cid_number: Vec<u8>,
    pub role_code: Vec<u8>,
    pub role_name: Vec<u8>,
    pub term_required: bool,
    pub role_status: InstitutionRoleStatusTag,
}

/// 单条管理员任职，对齐 `entity_primitives::InstitutionAdminAssignment` 字段顺序。
#[derive(Debug, Clone)]
pub struct InstitutionAssignmentArg {
    pub cid_number: Vec<u8>,
    pub admin_account: [u8; 32],
    pub role_code: Vec<u8>,
    pub term_start: u32,
    pub term_end: u32,
    pub assignment_source: InstitutionAssignmentSourceTag,
    pub assignment_source_ref: Vec<u8>,
    pub assignment_status: InstitutionAssignmentStatusTag,
}

/// `propose_create_{public,private}_institution` 完整参数。
#[derive(Debug, Clone)]
pub struct ProposeCreateInstitutionArgs {
    pub cid_number: Vec<u8>,
    pub cid_full_name: Vec<u8>,
    pub cid_short_name: Vec<u8>,
    pub town_code: Vec<u8>,
    pub legal_representative_name: Vec<u8>,
    pub legal_representative_cid_number: Vec<u8>,
    pub legal_representative_account: [u8; 32],
    pub accounts: Vec<InitialAccountArg>,
    pub institution_code: [u8; 4],
    pub roles: Vec<InstitutionRoleArg>,
    pub assignments: Vec<InstitutionAssignmentArg>,
    pub threshold: u32,
    pub register_nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub issuer_cid_number: Vec<u8>,
    pub issuer_main_account: [u8; 32],
    pub signer_pubkey: [u8; 32],
    pub scope_province_name: Vec<u8>,
    pub scope_city_name: Vec<u8>,
}

fn encode_bytes(out: &mut Vec<u8>, value: &[u8]) {
    out.extend(Compact(value.len() as u32).encode());
    out.extend_from_slice(value);
}

fn encode_role(out: &mut Vec<u8>, role: &InstitutionRoleArg) {
    encode_bytes(out, &role.cid_number);
    encode_bytes(out, &role.role_code);
    encode_bytes(out, &role.role_name);
    out.push(u8::from(role.term_required));
    out.push(role.role_status as u8);
}

fn encode_assignment(out: &mut Vec<u8>, assignment: &InstitutionAssignmentArg) {
    encode_bytes(out, &assignment.cid_number);
    out.extend_from_slice(&assignment.admin_account);
    encode_bytes(out, &assignment.role_code);
    out.extend(assignment.term_start.to_le_bytes());
    out.extend(assignment.term_end.to_le_bytes());
    out.push(assignment.assignment_source as u8);
    encode_bytes(out, &assignment.assignment_source_ref);
    out.push(assignment.assignment_status as u8);
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
    encode_bytes(&mut out, &args.legal_representative_name);
    encode_bytes(&mut out, &args.legal_representative_cid_number);
    out.extend_from_slice(&args.legal_representative_account);

    out.extend(Compact(args.accounts.len() as u32).encode());
    for account in &args.accounts {
        encode_bytes(&mut out, account.account_name.as_bytes());
        out.extend(account.amount.to_le_bytes());
    }

    out.extend_from_slice(&args.institution_code);
    out.extend(Compact(args.roles.len() as u32).encode());
    for role in &args.roles {
        encode_role(&mut out, role);
    }
    out.extend(Compact(args.assignments.len() as u32).encode());
    for assignment in &args.assignments {
        encode_assignment(&mut out, assignment);
    }

    out.extend(args.threshold.to_le_bytes());
    encode_bytes(&mut out, &args.register_nonce);
    encode_bytes(&mut out, &args.signature);
    encode_bytes(&mut out, &args.issuer_cid_number);
    out.extend_from_slice(&args.issuer_main_account);
    out.extend_from_slice(&args.signer_pubkey);
    encode_bytes(&mut out, &args.scope_province_name);
    encode_bytes(&mut out, &args.scope_city_name);

    ChainCall {
        action: chain_action_code(pallet_index, PROPOSE_CREATE_INSTITUTION_CALL_INDEX),
        call_data: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity_primitives::{
        InstitutionAdminAssignment, InstitutionAssignmentSource, InstitutionAssignmentStatus,
        InstitutionRole, InstitutionRoleStatus,
    };

    fn sample_role() -> InstitutionRoleArg {
        InstitutionRoleArg {
            cid_number: b"GD001-COMPANY-0001".to_vec(),
            role_code: b"DIRECTOR".to_vec(),
            role_name: "董事".as_bytes().to_vec(),
            term_required: true,
            role_status: InstitutionRoleStatusTag::Active,
        }
    }

    fn sample_assignment(seed: u8) -> InstitutionAssignmentArg {
        InstitutionAssignmentArg {
            cid_number: b"GD001-COMPANY-0001".to_vec(),
            admin_account: [seed; 32],
            role_code: b"DIRECTOR".to_vec(),
            term_start: 20_000,
            term_end: 21_825,
            assignment_source: InstitutionAssignmentSourceTag::Registry,
            assignment_source_ref: b"registry-record-1".to_vec(),
            assignment_status: InstitutionAssignmentStatusTag::Active,
        }
    }

    #[test]
    fn role_and_assignment_tags_match_entity_enums() {
        assert_eq!(
            InstitutionRoleStatusTag::Active as u8,
            InstitutionRoleStatus::Active.encode()[0]
        );
        assert_eq!(
            InstitutionRoleStatusTag::Inactive as u8,
            InstitutionRoleStatus::Inactive.encode()[0]
        );
        let sources = [
            (
                InstitutionAssignmentSourceTag::Genesis,
                InstitutionAssignmentSource::Genesis,
            ),
            (
                InstitutionAssignmentSourceTag::Registry,
                InstitutionAssignmentSource::Registry,
            ),
            (
                InstitutionAssignmentSourceTag::PopularElection,
                InstitutionAssignmentSource::PopularElection,
            ),
            (
                InstitutionAssignmentSourceTag::MutualElection,
                InstitutionAssignmentSource::MutualElection,
            ),
            (
                InstitutionAssignmentSourceTag::NominationAppointment,
                InstitutionAssignmentSource::NominationAppointment,
            ),
        ];
        for (tag, real) in sources {
            assert_eq!(tag as u8, real.encode()[0]);
        }
        assert_eq!(
            InstitutionAssignmentStatusTag::Active as u8,
            InstitutionAssignmentStatus::Active.encode()[0]
        );
        assert_eq!(
            InstitutionAssignmentStatusTag::Ended as u8,
            InstitutionAssignmentStatus::Ended.encode()[0]
        );
    }

    #[test]
    fn role_and_assignment_encoding_match_entity_types() {
        let role = sample_role();
        let mut encoded_role = Vec::new();
        encode_role(&mut encoded_role, &role);
        let real_role = InstitutionRole {
            cid_number: role.cid_number.clone(),
            role_code: role.role_code.clone(),
            role_name: role.role_name.clone(),
            term_required: role.term_required,
            role_status: InstitutionRoleStatus::Active,
        };
        assert_eq!(encoded_role, real_role.encode());

        let assignment = sample_assignment(7);
        let mut encoded_assignment = Vec::new();
        encode_assignment(&mut encoded_assignment, &assignment);
        let real_assignment = InstitutionAdminAssignment {
            cid_number: assignment.cid_number.clone(),
            admin_account: assignment.admin_account,
            role_code: assignment.role_code.clone(),
            term_start: assignment.term_start,
            term_end: assignment.term_end,
            assignment_source: InstitutionAssignmentSource::Registry,
            assignment_source_ref: assignment.assignment_source_ref.clone(),
            assignment_status: InstitutionAssignmentStatus::Active,
        };
        assert_eq!(encoded_assignment, real_assignment.encode());
    }

    #[test]
    fn full_create_payload_uses_roles_and_assignments() {
        let args = ProposeCreateInstitutionArgs {
            cid_number: b"GD001-COMPANY-0001".to_vec(),
            cid_full_name: "测试机构".as_bytes().to_vec(),
            cid_short_name: "测试".as_bytes().to_vec(),
            town_code: Vec::new(),
            legal_representative_name: "法人".as_bytes().to_vec(),
            legal_representative_cid_number: b"GD001-CTZN-1".to_vec(),
            legal_representative_account: [9; 32],
            accounts: vec![InitialAccountArg {
                account_name: "主账户".to_string(),
                amount: 111,
            }],
            institution_code: *b"SFLP",
            roles: vec![sample_role()],
            assignments: vec![sample_assignment(1), sample_assignment(2)],
            threshold: 2,
            register_nonce: b"nonce".to_vec(),
            signature: vec![3; 64],
            issuer_cid_number: b"issuer".to_vec(),
            issuer_main_account: [4; 32],
            signer_pubkey: [5; 32],
            scope_province_name: "广东省".as_bytes().to_vec(),
            scope_city_name: "广州市".as_bytes().to_vec(),
        };
        let encoded = encode_propose_create_institution(&args);
        assert_eq!(&encoded.call_data[..2], &[31, 5]);
        assert_eq!(encoded.action, 0x1f05);

        let real_roles = vec![InstitutionRole {
            cid_number: args.roles[0].cid_number.clone(),
            role_code: args.roles[0].role_code.clone(),
            role_name: args.roles[0].role_name.clone(),
            term_required: true,
            role_status: InstitutionRoleStatus::Active,
        }];
        let real_assignments = args
            .assignments
            .iter()
            .map(|assignment| InstitutionAdminAssignment {
                cid_number: assignment.cid_number.clone(),
                admin_account: assignment.admin_account,
                role_code: assignment.role_code.clone(),
                term_start: assignment.term_start,
                term_end: assignment.term_end,
                assignment_source: InstitutionAssignmentSource::Registry,
                assignment_source_ref: assignment.assignment_source_ref.clone(),
                assignment_status: InstitutionAssignmentStatus::Active,
            })
            .collect::<Vec<_>>();

        let real_accounts = vec![("主账户".as_bytes().to_vec(), 111_u128)];
        let mut expected = vec![31, 5];
        expected.extend(args.cid_number.encode());
        expected.extend(args.cid_full_name.encode());
        expected.extend(args.cid_short_name.encode());
        expected.extend(args.town_code.encode());
        expected.extend(args.legal_representative_name.encode());
        expected.extend(args.legal_representative_cid_number.encode());
        expected.extend(args.legal_representative_account.encode());
        expected.extend(real_accounts.encode());
        expected.extend(args.institution_code.encode());
        expected.extend(real_roles.encode());
        expected.extend(real_assignments.encode());
        expected.extend(args.threshold.encode());
        expected.extend(args.register_nonce.encode());
        expected.extend(args.signature.encode());
        expected.extend(args.issuer_cid_number.encode());
        expected.extend(args.issuer_main_account.encode());
        expected.extend(args.signer_pubkey.encode());
        expected.extend(args.scope_province_name.encode());
        expected.extend(args.scope_city_name.encode());
        assert_eq!(encoded.call_data, expected);
    }
}
