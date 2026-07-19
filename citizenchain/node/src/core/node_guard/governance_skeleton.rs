//! 固定治理机构的管理员、岗位与任职节点策略（档 A）。
//!
//! `PublicAdmins::AdminAccounts` 保存管理员账户与姓、名，`PublicManage` 保存岗位与任职。
//! 本策略按共享编译期清单校验 89 个公权固定治理机构和中国公民链技术有限公司：
//! 固定岗位目录和席位不允许漂移，具体管理员允许依法原子轮换。任职来源、引用和任期
//! 属于 runtime 业务合法性，不在原生组织结构守卫中重复解释。
//! 既有公权机构的法定代表人不是创世必填项；技术公司明确具有法定代表人，守卫只额外
//! 校验其机构信息账户与固定 `LR` 任职一致，不复制公民资料真源。

use std::collections::{BTreeMap, BTreeSet};

use admin_primitives::{Admin, InstitutionAdmins};
use codec::Decode;
#[cfg(test)]
use codec::Encode;
#[cfg(test)]
use entity_primitives::InstitutionAssignmentSource;
use entity_primitives::{
    InstitutionAdminAssignment as SharedInstitutionAdminAssignment, InstitutionAssignmentStatus,
    InstitutionRole as SharedInstitutionRole, InstitutionRoleStatus,
};

use primitives::{
    cid::china::citizenchain::{CITIZENCHAIN_FIXED_ROLES, CITIZENCHAIN_TECHNOLOGY},
    cid::code::{FRG, PROVINCE_CODE_INFOS},
    count_const::FRG_PROVINCE_GROUP_ADMIN_COUNT,
    governance_skeleton::{
        fixed_institutions, fixed_role_specs, province_commissioner_role_code,
        province_commissioner_role_name, FixedInstitution,
    },
};

const PUBLIC_ADMINS_PALLET: &[u8] = b"PublicAdmins";
const PUBLIC_MANAGE_PALLET: &[u8] = b"PublicManage";
const PRIVATE_ADMINS_PALLET: &[u8] = b"PrivateAdmins";
const PRIVATE_MANAGE_PALLET: &[u8] = b"PrivateManage";

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedRole {
    role_code: Vec<u8>,
    role_name: Vec<u8>,
    seats: u32,
}

fn protected_institutions() -> Vec<FixedInstitution> {
    let mut institutions = fixed_institutions();
    institutions.push(FixedInstitution {
        code: *b"SFGQ",
        main_account: CITIZENCHAIN_TECHNOLOGY.main_account,
        cid_number: CITIZENCHAIN_TECHNOLOGY.cid_number,
        expected_len: 3,
    });
    institutions
}

fn is_private_protected_cid(cid_number: &[u8]) -> bool {
    cid_number == CITIZENCHAIN_TECHNOLOGY.cid_number.as_bytes()
}

fn expected_roles(institution: &FixedInstitution) -> Vec<ExpectedRole> {
    if is_private_protected_cid(institution.cid_number.as_bytes()) {
        return CITIZENCHAIN_FIXED_ROLES
            .iter()
            .map(|role| ExpectedRole {
                role_code: role.role_code.to_vec(),
                role_name: role.role_name.to_vec(),
                seats: role.seats,
            })
            .collect();
    }
    let code = institution.code;
    if code == FRG {
        let mut roles = PROVINCE_CODE_INFOS
            .iter()
            .map(|province| ExpectedRole {
                role_code: province_commissioner_role_code(province.province_code),
                role_name: province_commissioner_role_name(province.province_name),
                seats: FRG_PROVINCE_GROUP_ADMIN_COUNT,
            })
            .collect::<Vec<_>>();
        roles.push(ExpectedRole {
            role_code: primitives::institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE.to_vec(),
            role_name: primitives::institution_constraints::ROLE_NAME_LEGAL_REPRESENTATIVE.to_vec(),
            seats: 0,
        });
        return roles;
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
    use super::{
        is_private_protected_cid, protected_institutions, FixedInstitution, PRIVATE_ADMINS_PALLET,
        PRIVATE_MANAGE_PALLET, PUBLIC_ADMINS_PALLET, PUBLIC_MANAGE_PALLET,
    };
    use codec::{Decode, Encode};
    use sp_core::hashing::blake2_128;

    fn storage_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::prefix(pallet, storage)
    }

    fn blake2_128_concat(encoded: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_128_concat(encoded)
    }

    pub fn admin_accounts_prefix() -> Vec<u8> {
        storage_prefix(PUBLIC_ADMINS_PALLET, b"AdminAccounts")
    }

    pub fn private_admin_accounts_prefix() -> Vec<u8> {
        storage_prefix(PRIVATE_ADMINS_PALLET, b"AdminAccounts")
    }

    pub fn institution_roles_prefix() -> Vec<u8> {
        storage_prefix(PUBLIC_MANAGE_PALLET, b"InstitutionRoles")
    }

    pub fn institution_role_assignments_prefix() -> Vec<u8> {
        storage_prefix(PUBLIC_MANAGE_PALLET, b"InstitutionRoleAssignments")
    }

    pub fn private_institution_roles_prefix() -> Vec<u8> {
        storage_prefix(PRIVATE_MANAGE_PALLET, b"InstitutionRoles")
    }

    pub fn private_institution_role_assignments_prefix() -> Vec<u8> {
        storage_prefix(PRIVATE_MANAGE_PALLET, b"InstitutionRoleAssignments")
    }

    pub fn private_institutions_prefix() -> Vec<u8> {
        storage_prefix(PRIVATE_MANAGE_PALLET, b"Institutions")
    }

    pub fn private_institution(cid_number: &[u8]) -> Vec<u8> {
        let mut key = private_institutions_prefix();
        key.extend_from_slice(&blake2_128_concat(&cid_number.to_vec().encode()));
        key
    }

    pub fn admin_account(cid_number: &[u8]) -> Vec<u8> {
        let mut key = if is_private_protected_cid(cid_number) {
            private_admin_accounts_prefix()
        } else {
            admin_accounts_prefix()
        };
        key.extend_from_slice(&blake2_128_concat(&cid_number.to_vec().encode()));
        key
    }

    fn admin_cid_from_key(key: &[u8], prefix: &[u8]) -> Option<Vec<u8>> {
        let encoded = key.strip_prefix(prefix)?;
        if encoded.len() < 17 || blake2_128(&encoded[16..]) != encoded[..16] {
            return None;
        }
        let mut input = &encoded[16..];
        let cid_number = Vec::<u8>::decode(&mut input).ok()?;
        if !input.is_empty() {
            return None;
        }
        Some(cid_number)
    }

    fn double_map_key(storage_prefix: Vec<u8>, cid_number: &[u8], role_code: &[u8]) -> Vec<u8> {
        let mut key = storage_prefix;
        key.extend_from_slice(&blake2_128_concat(&cid_number.to_vec().encode()));
        key.extend_from_slice(&blake2_128_concat(&role_code.to_vec().encode()));
        key
    }

    pub fn institution_role(cid_number: &[u8], role_code: &[u8]) -> Vec<u8> {
        let prefix = if is_private_protected_cid(cid_number) {
            private_institution_roles_prefix()
        } else {
            institution_roles_prefix()
        };
        double_map_key(prefix, cid_number, role_code)
    }

    pub fn institution_role_assignments(cid_number: &[u8], role_code: &[u8]) -> Vec<u8> {
        let prefix = if is_private_protected_cid(cid_number) {
            private_institution_role_assignments_prefix()
        } else {
            institution_role_assignments_prefix()
        };
        double_map_key(prefix, cid_number, role_code)
    }

    fn fixed_cid_prefix(storage_prefix: Vec<u8>, cid_number: &[u8]) -> Vec<u8> {
        let mut prefix = storage_prefix;
        prefix.extend_from_slice(&blake2_128_concat(&cid_number.to_vec().encode()));
        prefix
    }

    /// 启动时只枚举固定机构的岗位/任职子树，不扫描全部公权机构。
    pub fn fixed_catalog_prefixes() -> Vec<Vec<u8>> {
        let mut prefixes = Vec::new();
        for institution in protected_institutions() {
            let cid = institution.cid_number.as_bytes();
            let (roles, assignments) = if is_private_protected_cid(cid) {
                (
                    private_institution_roles_prefix(),
                    private_institution_role_assignments_prefix(),
                )
            } else {
                (
                    institution_roles_prefix(),
                    institution_role_assignments_prefix(),
                )
            };
            prefixes.push(fixed_cid_prefix(roles, cid));
            prefixes.push(fixed_cid_prefix(assignments, cid));
            if is_private_protected_cid(cid) {
                prefixes.push(private_institution(cid));
            }
        }
        prefixes
    }

    /// 返回 RAW key 对应的受保护创世机构；普通机构治理 key 明确返回 `None`。
    pub fn protected_institution_for_key(key: &[u8]) -> Option<FixedInstitution> {
        if key == private_institution(super::CITIZENCHAIN_TECHNOLOGY.cid_number.as_bytes()) {
            return protected_institutions()
                .into_iter()
                .find(|institution| is_private_protected_cid(institution.cid_number.as_bytes()));
        }
        for prefix in [admin_accounts_prefix(), private_admin_accounts_prefix()] {
            if let Some(cid_number) = admin_cid_from_key(key, &prefix) {
                return protected_institutions()
                    .into_iter()
                    .find(|institution| institution.cid_number.as_bytes() == cid_number);
            }
        }
        let prefixes = [
            institution_roles_prefix(),
            institution_role_assignments_prefix(),
            private_institution_roles_prefix(),
            private_institution_role_assignments_prefix(),
        ];
        let parsed = prefixes
            .iter()
            .find(|prefix| key.starts_with(prefix.as_slice()))
            .map(|prefix| super::parse_double_map_key(key, prefix))?;
        let (cid_number, _) = parsed.ok()?;
        protected_institutions()
            .into_iter()
            .find(|institution| institution.cid_number.as_bytes() == cid_number)
    }

    /// 普通区块和完整状态分区只关注 90 个受保护机构，不收集一般机构治理状态。
    pub fn is_relevant(key: &[u8]) -> bool {
        protected_institution_for_key(key).is_some()
    }
}

/// 节点直接使用共享协议类型解码，避免维护第二份字段顺序和枚举判别值。
type DecodedInstitutionAdmins = InstitutionAdmins<Vec<Admin<[u8; 32]>>>;
type DecodedInstitutionRole = SharedInstitutionRole<Vec<u8>, Vec<u8>, Vec<u8>>;
type DecodedInstitutionAdminAssignment =
    SharedInstitutionAdminAssignment<Vec<u8>, [u8; 32], Vec<u8>, Vec<u8>>;
type DecodedInstitutionInfo = entity_primitives::InstitutionInfo<u32, Vec<u8>, Vec<u8>, [u8; 32]>;

/// 固定治理骨架校验失败原因。
#[derive(Debug, PartialEq)]
pub enum GuardError {
    FixedInstitutionMissing([u8; 4]),
    AdminAccountDecodeFailed([u8; 4]),
    InstitutionCodeChanged([u8; 4]),
    AdminsLenChanged {
        code: [u8; 4],
        expected: u32,
        found: u32,
    },
    InvalidAdminPersonName([u8; 4]),
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
    DuplicateAssignmentWallet([u8; 4]),
    AdminAssignmentSetMismatch([u8; 4]),
    LegalRepresentativeInfoMissing([u8; 4]),
    LegalRepresentativeAssignmentMismatch([u8; 4]),
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

fn fixed_institution_in<'a>(
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
    let role_prefixes = [
        storage_key::institution_roles_prefix(),
        storage_key::private_institution_roles_prefix(),
    ];
    let assignment_prefixes = [
        storage_key::institution_role_assignments_prefix(),
        storage_key::private_institution_role_assignments_prefix(),
    ];
    let fixed = protected_institutions();

    for raw_key in keys {
        let key = raw_key.as_ref();
        let parsed = role_prefixes
            .iter()
            .chain(assignment_prefixes.iter())
            .find(|prefix| key.starts_with(prefix.as_slice()))
            .map(|prefix| parse_double_map_key(key, prefix));
        let Some(parsed) = parsed else {
            continue;
        };
        // 无法解析的额外 key 不会成为 FRAME 规范岗位；只要受保护机构的规范 key
        // 仍完整存在，它不属于原生治理骨架。通用 storage 合法性由 runtime 负责。
        let Ok((cid_number, role_code)) = parsed else {
            continue;
        };
        let Some(institution) = fixed_institution_in(&fixed, &cid_number) else {
            continue;
        };
        if !expected_roles(institution)
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

/// 校验全部 90 个受保护创世机构的管理员人员集合、固定岗位和任职席位。
///
/// 固定岗位代码、名称、所属机构和席位数不可改变；管理员钱包可以依法更新。任职来源、
/// 来源引用与任期只要求共享 SCALE 能完整解码，具体业务合法性由 runtime 与投票引擎负责。
pub fn check_skeleton_invariants<F>(read_raw: F) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    for institution in protected_institutions() {
        check_institution_invariants(&institution, &read_raw)?;
    }
    Ok(())
}

/// 只校验一个受保护创世机构，供普通区块按受影响身份精确执行。
fn check_institution_invariants<F>(
    institution: &FixedInstitution,
    read_raw: &F,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let expected_cid = institution.cid_number.as_bytes();
    let private_info = if is_private_protected_cid(expected_cid) {
        let raw = read_raw(&storage_key::private_institution(expected_cid))
            .ok_or(GuardError::LegalRepresentativeInfoMissing(institution.code))?;
        let info: DecodedInstitutionInfo = decode_exact(&raw)
            .map_err(|_| GuardError::LegalRepresentativeInfoMissing(institution.code))?;
        if info.institution_code != institution.code
            || info
                .legal_representative_name
                .as_ref()
                .is_none_or(Vec::is_empty)
            || info
                .legal_representative_cid_number
                .as_ref()
                .is_none_or(Vec::is_empty)
            || info.legal_representative_account.is_none()
        {
            return Err(GuardError::LegalRepresentativeInfoMissing(institution.code));
        }
        Some(info)
    } else {
        None
    };
    let raw = read_raw(&storage_key::admin_account(expected_cid))
        .ok_or(GuardError::FixedInstitutionMissing(institution.code))?;
    let account: DecodedInstitutionAdmins =
        decode_exact(&raw).map_err(|_| GuardError::AdminAccountDecodeFailed(institution.code))?;
    if account.institution_code != institution.code {
        return Err(GuardError::InstitutionCodeChanged(institution.code));
    }
    let found = account.admins.len() as u32;
    if found != institution.expected_len {
        return Err(GuardError::AdminsLenChanged {
            code: institution.code,
            expected: institution.expected_len,
            found,
        });
    }
    if account.admins.iter().any(|admin| {
        admin.family_name.is_empty()
            || admin.given_name.is_empty()
            || core::str::from_utf8(admin.family_name.as_slice()).is_err()
            || core::str::from_utf8(admin.given_name.as_slice()).is_err()
    }) {
        return Err(GuardError::InvalidAdminPersonName(institution.code));
    }
    let admin_set = account
        .admins
        .iter()
        .map(|admin| admin.admin_account)
        .collect::<BTreeSet<_>>();
    if admin_set.len() != account.admins.len() {
        return Err(GuardError::DuplicateAdminWallet(institution.code));
    }

    let mut assigned_wallets = BTreeSet::new();
    let mut legal_representative_assignment = None;
    for expected_role in expected_roles(institution) {
        let role_key = storage_key::institution_role(expected_cid, &expected_role.role_code);
        let role_raw = read_raw(&role_key).ok_or_else(|| GuardError::RoleMissing {
            code: institution.code,
            role_code: expected_role.role_code.clone(),
        })?;
        let role: DecodedInstitutionRole =
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
        if role.role_status != InstitutionRoleStatus::Active {
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
        let assignments: Vec<DecodedInstitutionAdminAssignment> = decode_exact(&assignments_raw)
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
            if assignment.assignment_status != InstitutionAssignmentStatus::Active {
                return Err(GuardError::AssignmentNotActive {
                    code: institution.code,
                    role_code: expected_role.role_code,
                });
            }
            if !assigned_wallets.insert(assignment.admin_account) {
                return Err(GuardError::DuplicateAssignmentWallet(institution.code));
            }
            if expected_role.role_code
                == primitives::institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE
            {
                legal_representative_assignment = Some(assignment.admin_account);
            }
        }
    }
    if assigned_wallets != admin_set {
        return Err(GuardError::AdminAssignmentSetMismatch(institution.code));
    }
    if let Some(info) = private_info {
        if info.legal_representative_account != legal_representative_assignment {
            return Err(GuardError::LegalRepresentativeAssignmentMismatch(
                institution.code,
            ));
        }
    }
    Ok(())
}

/// 普通区块仅复核实际被修改的受保护创世机构；runtime 升级仍全量复核。
pub(super) fn check_affected_institutions<F>(
    delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    read_raw: F,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    if delta.contains_key(sp_storage::well_known_keys::CODE) {
        return check_skeleton_invariants(read_raw);
    }
    let mut affected = BTreeMap::new();
    for key in delta.keys() {
        if let Some(institution) = storage_key::protected_institution_for_key(key) {
            // 机构唯一主键始终是 CID；主账户只是该 CID 下的一种协议账户，不能用于机构去重。
            affected.insert(institution.cid_number, institution);
        }
    }
    for institution in affected.values() {
        check_institution_invariants(institution, &read_raw)?;
    }
    Ok(())
}

/// 只有受保护机构治理 key 或 runtime code 变化时才复核治理骨架。
pub(super) fn needs_full_check(delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> bool {
    delta.keys().any(|key| storage_key::is_relevant(key))
        || delta.contains_key(sp_storage::well_known_keys::CODE)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use sp_core::hashing::{blake2_128, twox_128};

    fn accounts_for(institution: &FixedInstitution) -> Vec<[u8; 32]> {
        (0..institution.expected_len)
            .map(|index| [(index + 1) as u8; 32])
            .collect()
    }

    fn account_bytes(institution: &FixedInstitution, admins: Vec<[u8; 32]>) -> Vec<u8> {
        DecodedInstitutionAdmins {
            institution_code: institution.code,
            admins: admins
                .into_iter()
                .map(|admin_account| Admin {
                    admin_account,
                    family_name: "管理".as_bytes().to_vec().try_into().expect("name fits"),
                    given_name: "员".as_bytes().to_vec().try_into().expect("name fits"),
                })
                .collect(),
        }
        .encode()
    }

    fn role_bytes(institution: &FixedInstitution, role: &ExpectedRole) -> Vec<u8> {
        DecodedInstitutionRole {
            cid_number: institution.cid_number.as_bytes().to_vec(),
            role_code: role.role_code.clone(),
            role_name: role.role_name.clone(),
            term_required: false,
            role_status: InstitutionRoleStatus::Active,
        }
        .encode()
    }

    fn private_info_bytes(institution: &FixedInstitution, legal_account: [u8; 32]) -> Vec<u8> {
        DecodedInstitutionInfo {
            cid_full_name: "中国公民链技术有限公司".as_bytes().to_vec(),
            cid_short_name: "公民链技术".as_bytes().to_vec(),
            town_code: Vec::new(),
            legal_representative_name: Some("程伟".as_bytes().to_vec()),
            legal_representative_cid_number: Some(
                primitives::cid::china::citizenchain::LEGAL_REPRESENTATIVE_CITIZEN_CID_NUMBER
                    .as_bytes()
                    .to_vec(),
            ),
            legal_representative_account: Some(legal_account),
            institution_code: institution.code,
            created_at: 0,
        }
        .encode()
    }

    fn assignment(
        institution: &FixedInstitution,
        role: &ExpectedRole,
        admin_account: [u8; 32],
    ) -> DecodedInstitutionAdminAssignment {
        DecodedInstitutionAdminAssignment {
            cid_number: institution.cid_number.as_bytes().to_vec(),
            admin_account,
            role_code: role.role_code.clone(),
            term_start: 0,
            term_end: 0,
            assignment_source: InstitutionAssignmentSource::Genesis,
            assignment_source_ref: Vec::new(),
            assignment_status: InstitutionAssignmentStatus::Active,
        }
    }

    fn valid_state() -> BTreeMap<Vec<u8>, Vec<u8>> {
        let mut state = BTreeMap::new();
        for institution in protected_institutions() {
            let admins = accounts_for(&institution);
            state.insert(
                storage_key::admin_account(institution.cid_number.as_bytes()),
                account_bytes(&institution, admins.clone()),
            );
            if is_private_protected_cid(institution.cid_number.as_bytes()) {
                state.insert(
                    storage_key::private_institution(institution.cid_number.as_bytes()),
                    private_info_bytes(&institution, admins[0]),
                );
            }
            let mut offset = 0usize;
            for role in expected_roles(&institution) {
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
    fn citizenchain_company_uses_private_storage_and_locks_legal_representative_consistency() {
        let company = protected_institutions()
            .into_iter()
            .find(|institution| is_private_protected_cid(institution.cid_number.as_bytes()))
            .expect("protected private genesis company");
        assert!(storage_key::admin_account(company.cid_number.as_bytes())
            .starts_with(&storage_key::private_admin_accounts_prefix()));
        assert!(storage_key::institution_role(
            company.cid_number.as_bytes(),
            primitives::institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE,
        )
        .starts_with(&storage_key::private_institution_roles_prefix()));

        let mut missing = valid_state();
        missing.remove(&storage_key::private_institution(
            company.cid_number.as_bytes(),
        ));
        assert_eq!(
            check_state(&missing),
            Err(GuardError::LegalRepresentativeInfoMissing(company.code))
        );

        let mut mismatched = valid_state();
        mismatched.insert(
            storage_key::private_institution(company.cid_number.as_bytes()),
            private_info_bytes(&company, [250u8; 32]),
        );
        assert_eq!(
            check_state(&mismatched),
            Err(GuardError::LegalRepresentativeAssignmentMismatch(
                company.code
            ))
        );
    }

    #[test]
    fn institution_admins_layout_is_exact() {
        let institution = fixed_institutions()[0];
        let raw = account_bytes(&institution, accounts_for(&institution));
        let decoded: DecodedInstitutionAdmins = decode_exact(&raw).expect("layout decodes");
        assert_eq!(decoded.institution_code, institution.code);
        assert_eq!(decoded.admins.len() as u32, institution.expected_len);
    }

    #[test]
    fn person_name_changes_do_not_change_authority_but_invalid_names_are_rejected() {
        let institution = fixed_institutions()[0];
        let admin_key = storage_key::admin_account(institution.cid_number.as_bytes());
        let mut state = valid_state();
        let mut account: DecodedInstitutionAdmins =
            decode_exact(state.get(&admin_key).expect("admin account exists"))
                .expect("admin account decodes");
        account.admins[0].family_name = "张".as_bytes().to_vec().try_into().expect("name fits");
        account.admins[0].given_name = "三".as_bytes().to_vec().try_into().expect("name fits");
        state.insert(admin_key.clone(), account.encode());
        assert_eq!(check_state(&state), Ok(()));

        let mut account: DecodedInstitutionAdmins =
            decode_exact(state.get(&admin_key).expect("admin account exists"))
                .expect("admin account decodes");
        account.admins[0].family_name = Vec::new().try_into().expect("empty name fits");
        state.insert(admin_key, account.encode());
        assert_eq!(
            check_state(&state),
            Err(GuardError::InvalidAdminPersonName(institution.code))
        );
    }

    #[test]
    fn missing_or_wrong_code_fixed_account_is_rejected() {
        let institution = fixed_institutions()[0];
        let mut state = valid_state();
        state.remove(&storage_key::admin_account(
            institution.cid_number.as_bytes(),
        ));
        assert_eq!(
            check_state(&state),
            Err(GuardError::FixedInstitutionMissing(institution.code))
        );

        let mut state = valid_state();
        state.insert(
            storage_key::admin_account(institution.cid_number.as_bytes()),
            DecodedInstitutionAdmins {
                institution_code: *b"BAD\0",
                admins: accounts_for(&institution)
                    .into_iter()
                    .map(|admin_account| Admin {
                        admin_account,
                        family_name: "管理".as_bytes().to_vec().try_into().expect("name fits"),
                        given_name: "员".as_bytes().to_vec().try_into().expect("name fits"),
                    })
                    .collect(),
            }
            .encode(),
        );
        assert_eq!(
            check_state(&state),
            Err(GuardError::InstitutionCodeChanged(institution.code))
        );
    }

    #[test]
    fn missing_renamed_or_extra_fixed_role_is_rejected() {
        let institution = fixed_institutions()[0];
        let role = expected_roles(&institution)[0].clone();
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
        let mut renamed: DecodedInstitutionRole =
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
        let role = expected_roles(&institution)[0].clone();
        let assignments_key = storage_key::institution_role_assignments(
            institution.cid_number.as_bytes(),
            &role.role_code,
        );

        let mut state = valid_state();
        let mut assignments: Vec<DecodedInstitutionAdminAssignment> =
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
        let mut assignments: Vec<DecodedInstitutionAdminAssignment> =
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
    fn member_source_and_term_fields_are_outside_native_structure_guard() {
        let institution = fixed_institutions()[0];
        let role = expected_roles(&institution)[0].clone();
        let role_key =
            storage_key::institution_role(institution.cid_number.as_bytes(), &role.role_code);
        let assignments_key = storage_key::institution_role_assignments(
            institution.cid_number.as_bytes(),
            &role.role_code,
        );
        let admin_key = storage_key::admin_account(institution.cid_number.as_bytes());
        let mut state = valid_state();

        let mut role_value: DecodedInstitutionRole =
            decode_exact(state.get(&role_key).expect("role exists")).expect("role decodes");
        role_value.term_required = true;
        state.insert(role_key, role_value.encode());

        let mut assignments: Vec<DecodedInstitutionAdminAssignment> =
            decode_exact(state.get(&assignments_key).expect("assignments exist"))
                .expect("assignments decode");
        for assignment in &mut assignments {
            assignment.term_start = 10;
            assignment.term_end = 5;
            assignment.assignment_source = InstitutionAssignmentSource::PopularElection;
            assignment.assignment_source_ref = b"VOTE-1".to_vec();
        }
        assignments[0].admin_account = [250u8; 32];
        state.insert(assignments_key, assignments.encode());

        let mut account: DecodedInstitutionAdmins =
            decode_exact(state.get(&admin_key).expect("admin account exists"))
                .expect("admin account decodes");
        account.admins[0].admin_account = [250u8; 32];
        state.insert(admin_key, account.encode());

        assert_eq!(check_state(&state), Ok(()));
    }

    #[test]
    fn raw_key_derivation_and_trigger_prefixes_are_stable() {
        let account = b"CID-KEY";
        let mut expected_admin = twox_128(b"PublicAdmins").to_vec();
        expected_admin.extend_from_slice(&twox_128(b"AdminAccounts"));
        let encoded_account = account.to_vec().encode();
        expected_admin.extend_from_slice(&blake2_128(&encoded_account));
        expected_admin.extend_from_slice(&encoded_account);
        assert_eq!(storage_key::admin_account(account), expected_admin);

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
        assert!(
            !needs_full_check(&delta),
            "普通机构岗位变化不得触发创世治理骨架"
        );
        let protected = fixed_institutions()[0];
        delta.insert(
            storage_key::institution_role(
                protected.cid_number.as_bytes(),
                &expected_roles(&protected)[0].role_code,
            ),
            Some(Vec::new()),
        );
        assert!(needs_full_check(&delta));
        assert!(storage_key::fixed_catalog_prefixes().len() >= 2);
    }

    #[test]
    fn ordinary_block_checks_only_the_affected_protected_institution() {
        let fixed = fixed_institutions();
        let affected = fixed[0];
        let unrelated = fixed[1];
        let mut state = valid_state();
        state.remove(&storage_key::admin_account(unrelated.cid_number.as_bytes()));

        let delta = BTreeMap::from([(
            storage_key::admin_account(affected.cid_number.as_bytes()),
            state
                .get(&storage_key::admin_account(affected.cid_number.as_bytes()))
                .cloned(),
        )]);
        assert_eq!(
            check_affected_institutions(&delta, |key| state.get(key).cloned()),
            Ok(())
        );
        assert_eq!(
            check_skeleton_invariants(|key| state.get(key).cloned()),
            Err(GuardError::FixedInstitutionMissing(unrelated.code))
        );
    }

    #[test]
    fn malformed_extra_role_key_is_ignored_but_trailing_value_is_rejected() {
        let mut malformed = storage_key::institution_roles_prefix();
        malformed.extend_from_slice(b"bad");
        assert_eq!(check_catalog_keys([malformed]), Ok(()));

        let institution = fixed_institutions()[0];
        let role = expected_roles(&institution)[0].clone();
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
