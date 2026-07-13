//! 固定治理机构的管理员、岗位与任职节点策略（档 A）。
//!
//! `PublicAdmins::AdminAccounts` 只保存管理员钱包集合，`PublicManage` 保存岗位与任职。
//! 本策略按 `primitives::governance_skeleton` 的编译期清单校验五类固定机构：固定岗位
//! 目录和席位不允许漂移，具体管理员、任职来源和任期允许依法原子轮换。
//! 法定代表人不属于本策略，也不是创世必填项。

use std::collections::{BTreeMap, BTreeSet};

use codec::{Decode, Encode};

use primitives::{
    cid::code::{InstitutionCode, FRG, PROVINCE_CODE_INFOS},
    count_const::FRG_PROVINCE_GROUP_ADMIN_COUNT,
    governance_skeleton::{
        fixed_institutions, fixed_role_specs, province_commissioner_role_code,
        province_commissioner_role_name, FixedInstitution, STATUS_ACTIVE,
    },
};

const PUBLIC_ADMINS_PALLET: &[u8] = b"PublicAdmins";
const PUBLIC_MANAGE_PALLET: &[u8] = b"PublicManage";

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedRole {
    role_code: Vec<u8>,
    role_name: Vec<u8>,
    seats: u32,
}

fn expected_roles(code: InstitutionCode) -> Vec<ExpectedRole> {
    if code == FRG {
        return PROVINCE_CODE_INFOS
            .iter()
            .map(|province| ExpectedRole {
                role_code: province_commissioner_role_code(province.province_code),
                role_name: province_commissioner_role_name(province.province_name),
                seats: FRG_PROVINCE_GROUP_ADMIN_COUNT,
            })
            .collect();
    }
    fixed_role_specs(code)
        .into_iter()
        .map(|role| ExpectedRole {
            role_code: role.role_code.to_vec(),
            role_name: role.role_name.to_vec(),
            seats: role.seats,
        })
        .collect()
}

/// 三张受保护 storage 的完整 RAW key 和精确前缀。
pub mod storage_key {
    use super::{fixed_institutions, PUBLIC_ADMINS_PALLET, PUBLIC_MANAGE_PALLET};
    use codec::Encode;

    fn storage_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::prefix(pallet, storage)
    }

    fn blake2_128_concat(encoded: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_128_concat(encoded)
    }

    pub fn admin_accounts_prefix() -> Vec<u8> {
        storage_prefix(PUBLIC_ADMINS_PALLET, b"AdminAccounts")
    }

    pub fn institution_roles_prefix() -> Vec<u8> {
        storage_prefix(PUBLIC_MANAGE_PALLET, b"InstitutionRoles")
    }

    pub fn institution_role_assignments_prefix() -> Vec<u8> {
        storage_prefix(PUBLIC_MANAGE_PALLET, b"InstitutionRoleAssignments")
    }

    pub fn admin_account(account: &[u8; 32]) -> Vec<u8> {
        let mut key = admin_accounts_prefix();
        key.extend_from_slice(&blake2_128_concat(account));
        key
    }

    fn double_map_key(storage_prefix: Vec<u8>, cid_number: &[u8], role_code: &[u8]) -> Vec<u8> {
        let mut key = storage_prefix;
        key.extend_from_slice(&blake2_128_concat(&cid_number.to_vec().encode()));
        key.extend_from_slice(&blake2_128_concat(&role_code.to_vec().encode()));
        key
    }

    pub fn institution_role(cid_number: &[u8], role_code: &[u8]) -> Vec<u8> {
        double_map_key(institution_roles_prefix(), cid_number, role_code)
    }

    pub fn institution_role_assignments(cid_number: &[u8], role_code: &[u8]) -> Vec<u8> {
        double_map_key(institution_role_assignments_prefix(), cid_number, role_code)
    }

    fn fixed_cid_prefix(storage_prefix: Vec<u8>, cid_number: &[u8]) -> Vec<u8> {
        let mut prefix = storage_prefix;
        prefix.extend_from_slice(&blake2_128_concat(&cid_number.to_vec().encode()));
        prefix
    }

    /// 启动时只枚举固定机构的岗位/任职子树，不扫描全部公权机构。
    pub fn fixed_catalog_prefixes() -> Vec<Vec<u8>> {
        let mut prefixes = Vec::new();
        for institution in fixed_institutions() {
            let cid = institution.cid_number.as_bytes();
            prefixes.push(fixed_cid_prefix(institution_roles_prefix(), cid));
            prefixes.push(fixed_cid_prefix(institution_role_assignments_prefix(), cid));
        }
        prefixes
    }

    /// 普通区块和完整状态分区只关注三张目标 storage，不扩大到整个 pallet。
    pub fn is_relevant(key: &[u8]) -> bool {
        key.starts_with(&admin_accounts_prefix())
            || key.starts_with(&institution_roles_prefix())
            || key.starts_with(&institution_role_assignments_prefix())
    }
}

/// `admin-primitives::InstitutionAdminAccount` 的 SCALE 镜像。
#[derive(Decode, Encode)]
struct MInstitutionAdminAccount {
    cid_number: Vec<u8>,
    institution_code: [u8; 4],
    admins: Vec<[u8; 32]>,
    status: u8,
}

/// `entity-primitives::InstitutionRoleStatus` 的 SCALE 镜像。
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
enum MInstitutionRoleStatus {
    Active,
    Inactive,
}

/// `entity-primitives::InstitutionAssignmentSource` 的 SCALE 镜像。
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
enum MInstitutionAssignmentSource {
    Genesis,
    Registry,
    PopularElection,
    MutualElection,
    NominationAppointment,
}

/// `entity-primitives::InstitutionAssignmentStatus` 的 SCALE 镜像。
#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
enum MInstitutionAssignmentStatus {
    Active,
    Ended,
}

/// `entity-primitives::InstitutionRole` 的 SCALE 镜像。
#[derive(Decode, Encode)]
struct MInstitutionRole {
    cid_number: Vec<u8>,
    role_code: Vec<u8>,
    role_name: Vec<u8>,
    term_required: bool,
    role_status: MInstitutionRoleStatus,
}

/// `entity-primitives::InstitutionAdminAssignment` 的 SCALE 镜像。
#[derive(Clone, Decode, Encode)]
struct MInstitutionAdminAssignment {
    cid_number: Vec<u8>,
    admin_account: [u8; 32],
    role_code: Vec<u8>,
    term_start: u32,
    term_end: u32,
    assignment_source: MInstitutionAssignmentSource,
    assignment_source_ref: Vec<u8>,
    assignment_status: MInstitutionAssignmentStatus,
}

/// 固定治理骨架校验失败原因。
#[derive(Debug, PartialEq)]
pub enum GuardError {
    FixedInstitutionMissing([u8; 4]),
    AdminAccountDecodeFailed([u8; 4]),
    AdminCidChanged([u8; 4]),
    InstitutionCodeChanged([u8; 4]),
    NotActive([u8; 4]),
    AdminsLenChanged {
        code: [u8; 4],
        expected: u32,
        found: u32,
    },
    DuplicateAdminWallet([u8; 4]),
    RoleMissing {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    RoleDecodeFailed {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    RoleCidChanged {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    RoleCodeChanged {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    RoleNameChanged {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    RoleNotActive {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    AssignmentsMissing {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    AssignmentsDecodeFailed {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    SeatsChanged {
        code: [u8; 4],
        role_code: Vec<u8>,
        expected: u32,
        found: u32,
    },
    AssignmentCidChanged {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    AssignmentRoleChanged {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    AssignmentNotActive {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    InvalidAssignmentTerm {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
    DuplicateAssignmentWallet([u8; 4]),
    AdminAssignmentSetMismatch([u8; 4]),
    MalformedRoleStorageKey,
    UnknownFixedRole {
        code: [u8; 4],
        role_code: Vec<u8>,
    },
}

fn decode_exact<T: Decode>(raw: &[u8]) -> Result<T, ()> {
    let mut input = raw;
    let value = T::decode(&mut input).map_err(|_| ())?;
    if !input.is_empty() {
        return Err(());
    }
    Ok(value)
}

fn decode_concat_vec(input: &[u8]) -> Result<(Vec<u8>, usize), ()> {
    if input.len() < 16 {
        return Err(());
    }
    let hash = &input[..16];
    let encoded = &input[16..];
    let mut remaining = encoded;
    let value = Vec::<u8>::decode(&mut remaining).map_err(|_| ())?;
    let consumed = encoded.len().saturating_sub(remaining.len());
    if sp_core::hashing::blake2_128(&encoded[..consumed]) != hash {
        return Err(());
    }
    Ok((value, 16 + consumed))
}

fn parse_double_map_key(key: &[u8], prefix: &[u8]) -> Result<(Vec<u8>, Vec<u8>), ()> {
    let mut remaining = key.strip_prefix(prefix).ok_or(())?;
    let (cid_number, cid_consumed) = decode_concat_vec(remaining)?;
    remaining = &remaining[cid_consumed..];
    let (role_code, role_consumed) = decode_concat_vec(remaining)?;
    remaining = &remaining[role_consumed..];
    if !remaining.is_empty() {
        return Err(());
    }
    Ok((cid_number, role_code))
}

fn fixed_institution_by_cid<'a>(
    fixed: &'a [FixedInstitution],
    cid_number: &[u8],
) -> Option<&'a FixedInstitution> {
    fixed
        .iter()
        .find(|institution| institution.cid_number.as_bytes() == cid_number)
}

/// 校验岗位/任职 RAW key 形态，并禁止固定机构出现协议清单外岗位。
pub fn check_catalog_keys<I, K>(keys: I) -> Result<(), GuardError>
where
    I: IntoIterator<Item = K>,
    K: AsRef<[u8]>,
{
    let role_prefix = storage_key::institution_roles_prefix();
    let assignment_prefix = storage_key::institution_role_assignments_prefix();
    let fixed = fixed_institutions();

    for raw_key in keys {
        let key = raw_key.as_ref();
        let parsed = if key.starts_with(&role_prefix) {
            Some(parse_double_map_key(key, &role_prefix))
        } else if key.starts_with(&assignment_prefix) {
            Some(parse_double_map_key(key, &assignment_prefix))
        } else {
            None
        };
        let Some(parsed) = parsed else {
            continue;
        };
        let (cid_number, role_code) = parsed.map_err(|_| GuardError::MalformedRoleStorageKey)?;
        let Some(institution) = fixed_institution_by_cid(&fixed, &cid_number) else {
            continue;
        };
        if !expected_roles(institution.code)
            .iter()
            .any(|expected| expected.role_code == role_code)
        {
            return Err(GuardError::UnknownFixedRole {
                code: institution.code,
                role_code,
            });
        }
    }
    Ok(())
}

/// 校验五类固定机构的管理员集合、固定岗位和任职席位。
///
/// 固定岗位代码、名称、所属机构和席位数不可改变；管理员钱包、任职来源、来源引用和
/// 任期可依法更新。岗位要求任期时必须满足 `0 < start < end`，否则起止日必须同时为 0。
pub fn check_skeleton_invariants<F>(read_raw: F) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    for institution in fixed_institutions() {
        let expected_cid = institution.cid_number.as_bytes();
        let raw = read_raw(&storage_key::admin_account(&institution.main_account))
            .ok_or(GuardError::FixedInstitutionMissing(institution.code))?;
        let account: MInstitutionAdminAccount = decode_exact(&raw)
            .map_err(|_| GuardError::AdminAccountDecodeFailed(institution.code))?;
        if account.cid_number != expected_cid {
            return Err(GuardError::AdminCidChanged(institution.code));
        }
        if account.institution_code != institution.code {
            return Err(GuardError::InstitutionCodeChanged(institution.code));
        }
        if account.status != STATUS_ACTIVE {
            return Err(GuardError::NotActive(institution.code));
        }
        let found = account.admins.len() as u32;
        if found != institution.expected_len {
            return Err(GuardError::AdminsLenChanged {
                code: institution.code,
                expected: institution.expected_len,
                found,
            });
        }
        let admin_set = account.admins.iter().copied().collect::<BTreeSet<_>>();
        if admin_set.len() != account.admins.len() {
            return Err(GuardError::DuplicateAdminWallet(institution.code));
        }

        let mut assigned_wallets = BTreeSet::new();
        for expected_role in expected_roles(institution.code) {
            let role_key = storage_key::institution_role(expected_cid, &expected_role.role_code);
            let role_raw = read_raw(&role_key).ok_or_else(|| GuardError::RoleMissing {
                code: institution.code,
                role_code: expected_role.role_code.clone(),
            })?;
            let role: MInstitutionRole =
                decode_exact(&role_raw).map_err(|_| GuardError::RoleDecodeFailed {
                    code: institution.code,
                    role_code: expected_role.role_code.clone(),
                })?;
            if role.cid_number != expected_cid {
                return Err(GuardError::RoleCidChanged {
                    code: institution.code,
                    role_code: expected_role.role_code,
                });
            }
            if role.role_code != expected_role.role_code {
                return Err(GuardError::RoleCodeChanged {
                    code: institution.code,
                    role_code: expected_role.role_code,
                });
            }
            if role.role_name != expected_role.role_name {
                return Err(GuardError::RoleNameChanged {
                    code: institution.code,
                    role_code: expected_role.role_code,
                });
            }
            if role.role_status != MInstitutionRoleStatus::Active {
                return Err(GuardError::RoleNotActive {
                    code: institution.code,
                    role_code: expected_role.role_code,
                });
            }

            let assignments_key =
                storage_key::institution_role_assignments(expected_cid, &expected_role.role_code);
            let assignments_raw =
                read_raw(&assignments_key).ok_or_else(|| GuardError::AssignmentsMissing {
                    code: institution.code,
                    role_code: expected_role.role_code.clone(),
                })?;
            let assignments: Vec<MInstitutionAdminAssignment> = decode_exact(&assignments_raw)
                .map_err(|_| GuardError::AssignmentsDecodeFailed {
                    code: institution.code,
                    role_code: expected_role.role_code.clone(),
                })?;
            let found = assignments.len() as u32;
            if found != expected_role.seats {
                return Err(GuardError::SeatsChanged {
                    code: institution.code,
                    role_code: expected_role.role_code,
                    expected: expected_role.seats,
                    found,
                });
            }
            for assignment in assignments {
                if assignment.cid_number != expected_cid {
                    return Err(GuardError::AssignmentCidChanged {
                        code: institution.code,
                        role_code: expected_role.role_code,
                    });
                }
                if assignment.role_code != expected_role.role_code {
                    return Err(GuardError::AssignmentRoleChanged {
                        code: institution.code,
                        role_code: expected_role.role_code,
                    });
                }
                if assignment.assignment_status != MInstitutionAssignmentStatus::Active {
                    return Err(GuardError::AssignmentNotActive {
                        code: institution.code,
                        role_code: expected_role.role_code,
                    });
                }
                let valid_term = if role.term_required {
                    assignment.term_start > 0 && assignment.term_end > assignment.term_start
                } else {
                    assignment.term_start == 0 && assignment.term_end == 0
                };
                if !valid_term {
                    return Err(GuardError::InvalidAssignmentTerm {
                        code: institution.code,
                        role_code: expected_role.role_code,
                    });
                }
                if !assigned_wallets.insert(assignment.admin_account) {
                    return Err(GuardError::DuplicateAssignmentWallet(institution.code));
                }
            }
        }
        if assigned_wallets != admin_set {
            return Err(GuardError::AdminAssignmentSetMismatch(institution.code));
        }
    }
    Ok(())
}

/// 触及三张目标 storage 或 runtime code 时必须跑完整治理骨架校验。
pub(super) fn needs_full_check(delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> bool {
    delta.keys().any(|key| storage_key::is_relevant(key))
        || delta.contains_key(sp_storage::well_known_keys::CODE)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use sp_core::hashing::{blake2_128, twox_128};

    const STATUS_PENDING: u8 = 0;

    fn accounts_for(institution: &FixedInstitution) -> Vec<[u8; 32]> {
        (0..institution.expected_len)
            .map(|index| [(index + 1) as u8; 32])
            .collect()
    }

    fn account_bytes(institution: &FixedInstitution, status: u8, admins: Vec<[u8; 32]>) -> Vec<u8> {
        MInstitutionAdminAccount {
            cid_number: institution.cid_number.as_bytes().to_vec(),
            institution_code: institution.code,
            admins,
            status,
        }
        .encode()
    }

    fn role_bytes(institution: &FixedInstitution, role: &ExpectedRole) -> Vec<u8> {
        MInstitutionRole {
            cid_number: institution.cid_number.as_bytes().to_vec(),
            role_code: role.role_code.clone(),
            role_name: role.role_name.clone(),
            term_required: false,
            role_status: MInstitutionRoleStatus::Active,
        }
        .encode()
    }

    fn assignment(
        institution: &FixedInstitution,
        role: &ExpectedRole,
        admin_account: [u8; 32],
    ) -> MInstitutionAdminAssignment {
        MInstitutionAdminAssignment {
            cid_number: institution.cid_number.as_bytes().to_vec(),
            admin_account,
            role_code: role.role_code.clone(),
            term_start: 0,
            term_end: 0,
            assignment_source: MInstitutionAssignmentSource::Genesis,
            assignment_source_ref: Vec::new(),
            assignment_status: MInstitutionAssignmentStatus::Active,
        }
    }

    fn valid_state() -> BTreeMap<Vec<u8>, Vec<u8>> {
        let mut state = BTreeMap::new();
        for institution in fixed_institutions() {
            let admins = accounts_for(&institution);
            state.insert(
                storage_key::admin_account(&institution.main_account),
                account_bytes(&institution, STATUS_ACTIVE, admins.clone()),
            );
            let mut offset = 0usize;
            for role in expected_roles(institution.code) {
                state.insert(
                    storage_key::institution_role(
                        institution.cid_number.as_bytes(),
                        &role.role_code,
                    ),
                    role_bytes(&institution, &role),
                );
                let end = offset + role.seats as usize;
                let assignments = admins[offset..end]
                    .iter()
                    .copied()
                    .map(|admin| assignment(&institution, &role, admin))
                    .collect::<Vec<_>>();
                state.insert(
                    storage_key::institution_role_assignments(
                        institution.cid_number.as_bytes(),
                        &role.role_code,
                    ),
                    assignments.encode(),
                );
                offset = end;
            }
            assert_eq!(offset, admins.len());
        }
        state
    }

    fn check_state(state: &BTreeMap<Vec<u8>, Vec<u8>>) -> Result<(), GuardError> {
        check_catalog_keys(state.keys())?;
        check_skeleton_invariants(|key| state.get(key).cloned())
    }

    #[test]
    fn valid_fixed_admin_role_and_assignment_state_passes() {
        assert_eq!(check_state(&valid_state()), Ok(()));
    }

    #[test]
    fn institution_admin_account_layout_is_exact() {
        let institution = fixed_institutions()[0];
        let raw = account_bytes(&institution, STATUS_ACTIVE, accounts_for(&institution));
        let decoded: MInstitutionAdminAccount = decode_exact(&raw).expect("layout decodes");
        assert_eq!(decoded.cid_number, institution.cid_number.as_bytes());
        assert_eq!(decoded.institution_code, institution.code);
        assert_eq!(decoded.admins.len() as u32, institution.expected_len);
        assert_eq!(decoded.status, STATUS_ACTIVE);
    }

    #[test]
    fn missing_or_inactive_fixed_account_is_rejected() {
        let institution = fixed_institutions()[0];
        let mut state = valid_state();
        state.remove(&storage_key::admin_account(&institution.main_account));
        assert_eq!(
            check_state(&state),
            Err(GuardError::FixedInstitutionMissing(institution.code))
        );

        let mut state = valid_state();
        state.insert(
            storage_key::admin_account(&institution.main_account),
            account_bytes(&institution, STATUS_PENDING, accounts_for(&institution)),
        );
        assert_eq!(
            check_state(&state),
            Err(GuardError::NotActive(institution.code))
        );
    }

    #[test]
    fn missing_renamed_or_extra_fixed_role_is_rejected() {
        let institution = fixed_institutions()[0];
        let role = expected_roles(institution.code)[0].clone();
        let role_key =
            storage_key::institution_role(institution.cid_number.as_bytes(), &role.role_code);

        let mut state = valid_state();
        state.remove(&role_key);
        assert_eq!(
            check_state(&state),
            Err(GuardError::RoleMissing {
                code: institution.code,
                role_code: role.role_code.clone(),
            })
        );

        let mut state = valid_state();
        let mut renamed: MInstitutionRole =
            decode_exact(state.get(&role_key).expect("role exists")).expect("role decodes");
        renamed.role_name = "攻击者岗位".as_bytes().to_vec();
        state.insert(role_key, renamed.encode());
        assert_eq!(
            check_state(&state),
            Err(GuardError::RoleNameChanged {
                code: institution.code,
                role_code: role.role_code.clone(),
            })
        );

        let mut state = valid_state();
        state.insert(
            storage_key::institution_role(institution.cid_number.as_bytes(), b"EXTRA_ROLE"),
            Vec::new(),
        );
        assert_eq!(
            check_state(&state),
            Err(GuardError::UnknownFixedRole {
                code: institution.code,
                role_code: b"EXTRA_ROLE".to_vec(),
            })
        );
    }

    #[test]
    fn changed_seat_count_or_admin_union_is_rejected() {
        let institution = fixed_institutions()[0];
        let role = expected_roles(institution.code)[0].clone();
        let assignments_key = storage_key::institution_role_assignments(
            institution.cid_number.as_bytes(),
            &role.role_code,
        );

        let mut state = valid_state();
        let mut assignments: Vec<MInstitutionAdminAssignment> =
            decode_exact(state.get(&assignments_key).expect("assignments exist"))
                .expect("assignments decode");
        assignments.pop();
        state.insert(assignments_key.clone(), assignments.encode());
        assert_eq!(
            check_state(&state),
            Err(GuardError::SeatsChanged {
                code: institution.code,
                role_code: role.role_code.clone(),
                expected: role.seats,
                found: role.seats - 1,
            })
        );

        let mut state = valid_state();
        let mut assignments: Vec<MInstitutionAdminAssignment> =
            decode_exact(state.get(&assignments_key).expect("assignments exist"))
                .expect("assignments decode");
        assignments[0].admin_account = [250u8; 32];
        state.insert(assignments_key, assignments.encode());
        assert_eq!(
            check_state(&state),
            Err(GuardError::AdminAssignmentSetMismatch(institution.code))
        );
    }

    #[test]
    fn lawful_member_source_and_term_rotation_passes() {
        let institution = fixed_institutions()[0];
        let role = expected_roles(institution.code)[0].clone();
        let role_key =
            storage_key::institution_role(institution.cid_number.as_bytes(), &role.role_code);
        let assignments_key = storage_key::institution_role_assignments(
            institution.cid_number.as_bytes(),
            &role.role_code,
        );
        let admin_key = storage_key::admin_account(&institution.main_account);
        let mut state = valid_state();

        let mut role_value: MInstitutionRole =
            decode_exact(state.get(&role_key).expect("role exists")).expect("role decodes");
        role_value.term_required = true;
        state.insert(role_key, role_value.encode());

        let mut assignments: Vec<MInstitutionAdminAssignment> =
            decode_exact(state.get(&assignments_key).expect("assignments exist"))
                .expect("assignments decode");
        for assignment in &mut assignments {
            assignment.term_start = 10;
            assignment.term_end = 20;
            assignment.assignment_source = MInstitutionAssignmentSource::PopularElection;
            assignment.assignment_source_ref = b"VOTE-1".to_vec();
        }
        assignments[0].admin_account = [250u8; 32];
        state.insert(assignments_key, assignments.encode());

        let mut account: MInstitutionAdminAccount =
            decode_exact(state.get(&admin_key).expect("admin account exists"))
                .expect("admin account decodes");
        account.admins[0] = [250u8; 32];
        state.insert(admin_key, account.encode());

        assert_eq!(check_state(&state), Ok(()));
    }

    #[test]
    fn raw_key_derivation_and_trigger_prefixes_are_stable() {
        let account = [7u8; 32];
        let mut expected_admin = twox_128(b"PublicAdmins").to_vec();
        expected_admin.extend_from_slice(&twox_128(b"AdminAccounts"));
        expected_admin.extend_from_slice(&blake2_128(&account));
        expected_admin.extend_from_slice(&account);
        assert_eq!(storage_key::admin_account(&account), expected_admin);

        let cid = b"CID-1".to_vec().encode();
        let role = b"ROLE-1".to_vec().encode();
        let mut expected_role = twox_128(b"PublicManage").to_vec();
        expected_role.extend_from_slice(&twox_128(b"InstitutionRoles"));
        expected_role.extend_from_slice(&blake2_128(&cid));
        expected_role.extend_from_slice(&cid);
        expected_role.extend_from_slice(&blake2_128(&role));
        expected_role.extend_from_slice(&role);
        assert_eq!(
            storage_key::institution_role(b"CID-1", b"ROLE-1"),
            expected_role
        );

        let mut delta = BTreeMap::new();
        delta.insert(expected_role, Some(Vec::new()));
        assert!(needs_full_check(&delta));
        assert!(storage_key::fixed_catalog_prefixes().len() >= 2);
    }

    #[test]
    fn malformed_role_key_and_trailing_value_are_rejected() {
        let mut malformed = storage_key::institution_roles_prefix();
        malformed.extend_from_slice(b"bad");
        assert_eq!(
            check_catalog_keys([malformed]),
            Err(GuardError::MalformedRoleStorageKey)
        );

        let institution = fixed_institutions()[0];
        let role = expected_roles(institution.code)[0].clone();
        let role_key =
            storage_key::institution_role(institution.cid_number.as_bytes(), &role.role_code);
        let mut state = valid_state();
        state.get_mut(&role_key).expect("role exists").push(0);
        assert_eq!(
            check_state(&state),
            Err(GuardError::RoleDecodeFailed {
                code: institution.code,
                role_code: role.role_code,
            })
        );
    }
}
