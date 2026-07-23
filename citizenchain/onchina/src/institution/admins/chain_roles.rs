//! 机构岗位与管理员任职的链上只读适配层。
//!
//! `admins` pallet 只回答“哪些钱包是本机构管理员”；岗位定义和任职关系只从
//! `PublicManage` / `PrivateManage` 读取。本模块负责合并两类真源，不参与投票或任免业务。

use std::collections::{HashMap, HashSet};

use codec::Decode;
use subxt::{dynamic, OnlineClient, PolkadotConfig};

use crate::core::chain_runtime::{
    fetch_active_admins_onchain, AdminPallet, NodeInstitutionIdentity,
};

const ROLE_STATUS_ACTIVE: u8 = 0;
const ASSIGNMENT_STATUS_ACTIVE: u8 = 0;

#[derive(Debug, Decode)]
struct RawInstitutionRole {
    cid_number: Vec<u8>,
    role_code: Vec<u8>,
    role_name: Vec<u8>,
    term_required: bool,
    role_status: u8,
}

#[derive(Debug, Decode)]
struct RawInstitutionAssignment {
    cid_number: Vec<u8>,
    account_id: [u8; 32],
    role_code: Vec<u8>,
    term_start: u32,
    term_end: u32,
    assignment_source: u8,
    assignment_source_ref: Vec<u8>,
    assignment_status: u8,
}

/// 管理员人员记录和一条可选有效任职的联合投影。
#[derive(Debug, Clone)]
pub(crate) struct InstitutionAssignmentView {
    pub(crate) account_id: String,
    pub(crate) family_name: String,
    pub(crate) given_name: String,
    pub(crate) role_code: String,
    pub(crate) role_name: String,
    pub(crate) term_required: bool,
    pub(crate) term_start: u32,
    pub(crate) term_end: u32,
    pub(crate) assignment_source: u8,
    pub(crate) assignment_source_label: String,
    pub(crate) assignment_source_ref: String,
}

/// 联邦注册局单省专员岗位的任职投影。
#[derive(Debug, Clone)]
pub(crate) struct FederalRegistryProvinceAssignments {
    pub(crate) province_code: [u8; 2],
    pub(crate) province_name: String,
    pub(crate) assignments: Vec<InstitutionAssignmentView>,
}

fn assignment_source_label(source: u8) -> &'static str {
    match source {
        0 => "创世",
        1 => "注册局",
        2 => "普选",
        3 => "互选",
        4 => "提名任免",
        5 => "机构内部治理",
        _ => "",
    }
}

fn assignment_is_effective(
    role: &RawInstitutionRole,
    assignment: &RawInstitutionAssignment,
    current_day: u32,
) -> bool {
    if assignment.assignment_status != ASSIGNMENT_STATUS_ACTIVE {
        return false;
    }
    if !role.term_required {
        return assignment.term_start == 0 && assignment.term_end == 0;
    }
    assignment.term_start > 0
        && assignment.term_start <= current_day
        && current_day <= assignment.term_end
}

fn current_utc_day() -> Result<u32, String> {
    let current_day = chrono::Utc::now().timestamp().div_euclid(86_400);
    u32::try_from(current_day).map_err(|_| "current UTC day is outside u32 range".to_string())
}

fn entity_pallet_name(admin_pallet: AdminPallet) -> &'static str {
    match admin_pallet {
        AdminPallet::PublicAdmins => "PublicManage",
        AdminPallet::PrivateAdmins => "PrivateManage",
    }
}

async fn read_roles_and_assignments(
    cid_number: &[u8],
    admin_pallet: AdminPallet,
) -> Result<(Vec<RawInstitutionRole>, Vec<RawInstitutionAssignment>), String> {
    let ws_url = crate::core::chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for institution roles failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest institution role storage failed: {e}"))?;
    let pallet = entity_pallet_name(admin_pallet);

    // 值本身携带 CID；全表遍历后按 CID 过滤，避免依赖动态 storage key 的哈希后缀。
    let mut roles = Vec::new();
    let mut role_iter = storage
        .iter(dynamic::storage(
            pallet,
            "InstitutionRoles",
            Vec::<dynamic::Value>::new(),
        ))
        .await
        .map_err(|e| format!("iterate {pallet} InstitutionRoles failed: {e}"))?;
    while let Some(item) = role_iter.next().await {
        let item = item.map_err(|e| format!("read {pallet} InstitutionRoles failed: {e}"))?;
        let mut encoded = item.value.encoded();
        let role = RawInstitutionRole::decode(&mut encoded)
            .map_err(|e| format!("decode {pallet} InstitutionRoles failed: {e}"))?;
        if role.cid_number == cid_number {
            roles.push(role);
        }
    }

    let mut assignments = Vec::new();
    let mut assignment_iter = storage
        .iter(dynamic::storage(
            pallet,
            "InstitutionRoleAssignments",
            Vec::<dynamic::Value>::new(),
        ))
        .await
        .map_err(|e| format!("iterate {pallet} InstitutionRoleAssignments failed: {e}"))?;
    while let Some(item) = assignment_iter.next().await {
        let item =
            item.map_err(|e| format!("read {pallet} InstitutionRoleAssignments failed: {e}"))?;
        let mut encoded = item.value.encoded();
        let role_assignments = Vec::<RawInstitutionAssignment>::decode(&mut encoded)
            .map_err(|e| format!("decode {pallet} InstitutionRoleAssignments failed: {e}"))?;
        assignments.extend(
            role_assignments
                .into_iter()
                .filter(|assignment| assignment.cid_number == cid_number),
        );
    }
    Ok((roles, assignments))
}

fn merge_active_assignments(
    roles: Vec<RawInstitutionRole>,
    assignments: Vec<RawInstitutionAssignment>,
    active_admins: &HashMap<[u8; 32], (String, String)>,
) -> Result<Vec<InstitutionAssignmentView>, String> {
    let active_roles: HashMap<Vec<u8>, RawInstitutionRole> = roles
        .into_iter()
        .filter(|role| role.role_status == ROLE_STATUS_ACTIVE)
        .map(|role| (role.role_code.clone(), role))
        .collect();
    let current_day = current_utc_day()?;
    let mut views = Vec::new();
    for assignment in assignments {
        if !active_admins.contains_key(&assignment.account_id) {
            continue;
        }
        let Some(role) = active_roles.get(&assignment.role_code) else {
            return Err(
                "active institution assignment references a missing active role".to_string(),
            );
        };
        if !assignment_is_effective(role, &assignment, current_day) {
            continue;
        }
        views.push(InstitutionAssignmentView {
            account_id: format!("0x{}", hex::encode(assignment.account_id)),
            family_name: active_admins[&assignment.account_id].0.clone(),
            given_name: active_admins[&assignment.account_id].1.clone(),
            role_code: String::from_utf8_lossy(&assignment.role_code).to_string(),
            role_name: String::from_utf8_lossy(&role.role_name).to_string(),
            term_required: role.term_required,
            term_start: assignment.term_start,
            term_end: assignment.term_end,
            assignment_source: assignment.assignment_source,
            assignment_source_label: assignment_source_label(assignment.assignment_source)
                .to_string(),
            assignment_source_ref: String::from_utf8_lossy(&assignment.assignment_source_ref)
                .to_string(),
        });
    }
    views.sort_by(|left, right| {
        left.role_code
            .cmp(&right.role_code)
            .then(left.account_id.cmp(&right.account_id))
    });
    Ok(views)
}

/// 合并本机构管理员人员集合与有效任职；没有岗位的管理员仍返回一条空任职投影。
pub(crate) async fn fetch_active_assignments_onchain(
    identity: &NodeInstitutionIdentity,
) -> Result<Option<Vec<InstitutionAssignmentView>>, String> {
    let cid_number = identity.cid_number.as_str();
    let Some(admins) = fetch_active_admins_onchain(identity).await? else {
        return Ok(None);
    };
    let active_admins: HashMap<[u8; 32], (String, String)> = admins
        .iter()
        .map(|admin| {
            crate::auth::login::parse_account_id_bytes(&admin.account_id)
                .map(|account| {
                    (
                        account,
                        (admin.family_name.clone(), admin.given_name.clone()),
                    )
                })
                .ok_or_else(|| "active admin account decode failed".to_string())
        })
        .collect::<Result<_, _>>()?;

    for pallet in &identity.admin_pallets {
        let (roles, assignments) =
            read_roles_and_assignments(cid_number.as_bytes(), *pallet).await?;
        if roles.is_empty() {
            continue;
        }
        let mut views = merge_active_assignments(roles, assignments, &active_admins)?;
        if let Some(province_code) = identity.frg_province_code {
            let expected =
                primitives::governance_skeleton::province_commissioner_role_code(province_code);
            views.retain(|view| view.role_code.as_bytes() == expected.as_slice());
        }
        let covered = views
            .iter()
            .filter_map(|view| crate::auth::login::parse_account_id_bytes(&view.account_id))
            .collect::<HashSet<_>>();
        for (account_id, (family_name, given_name)) in &active_admins {
            if covered.contains(account_id) {
                continue;
            }
            views.push(InstitutionAssignmentView {
                account_id: format!("0x{}", hex::encode(account_id)),
                family_name: family_name.clone(),
                given_name: given_name.clone(),
                role_code: String::new(),
                role_name: String::new(),
                term_required: false,
                term_start: 0,
                term_end: 0,
                assignment_source: 0,
                assignment_source_label: String::new(),
                assignment_source_ref: String::new(),
            });
        }
        views.sort_by(|left, right| {
            left.account_id
                .cmp(&right.account_id)
                .then(left.role_code.cmp(&right.role_code))
        });
        return Ok(Some(views));
    }
    Err("institution roles not found in matching entity pallet".to_string())
}

/// 查找某个联邦注册局管理员账户当前担任专员的省码。
pub(crate) async fn fetch_frg_province_codes_for_admin(
    cid_number: &[u8],
    account_id: [u8; 32],
) -> Result<Vec<[u8; 2]>, String> {
    let (roles, assignments) =
        read_roles_and_assignments(cid_number, AdminPallet::PublicAdmins).await?;
    let active_roles: HashMap<Vec<u8>, RawInstitutionRole> = roles
        .into_iter()
        .filter(|role| role.role_status == ROLE_STATUS_ACTIVE)
        .map(|role| (role.role_code.clone(), role))
        .collect();
    let current_day = current_utc_day()?;
    let assigned_codes: HashSet<Vec<u8>> = assignments
        .into_iter()
        .filter(|assignment| {
            assignment.account_id == account_id
                && active_roles
                    .get(&assignment.role_code)
                    .is_some_and(|role| assignment_is_effective(role, assignment, current_day))
        })
        .map(|assignment| assignment.role_code)
        .collect();
    Ok(primitives::cid::code::PROVINCE_CODE_INFOS
        .iter()
        .filter_map(|info| {
            assigned_codes
                .contains(
                    &primitives::governance_skeleton::province_commissioner_role_code(
                        info.province_code,
                    ),
                )
                .then_some(info.province_code)
        })
        .collect())
}

/// 从 FRG entity 任职真源取指定省专员岗位的有效管理员账户 ID。
pub(crate) async fn fetch_frg_admins_for_province(
    cid_number: &[u8],
    province_code: [u8; 2],
) -> Result<HashSet<[u8; 32]>, String> {
    let (roles, assignments) =
        read_roles_and_assignments(cid_number, AdminPallet::PublicAdmins).await?;
    let expected = primitives::governance_skeleton::province_commissioner_role_code(province_code);
    let role = roles
        .into_iter()
        .find(|role| role.role_status == ROLE_STATUS_ACTIVE && role.role_code == expected);
    let Some(role) = role else {
        return Err("federal registry province commissioner role is not active".to_string());
    };
    let current_day = current_utc_day()?;
    Ok(assignments
        .into_iter()
        .filter(|assignment| {
            assignment.role_code == expected
                && assignment_is_effective(&role, assignment, current_day)
        })
        .map(|assignment| assignment.account_id)
        .collect())
}

/// 按 43 个省专员岗位输出联邦注册局全部有效任职。
pub(crate) async fn fetch_all_federal_registry_assignments(
    identity: &NodeInstitutionIdentity,
) -> Result<Vec<FederalRegistryProvinceAssignments>, String> {
    let assignments = fetch_active_assignments_onchain(identity)
        .await?
        .ok_or_else(|| "federal registry admin account not found".to_string())?;
    Ok(primitives::cid::code::PROVINCE_CODE_INFOS
        .iter()
        .map(|info| {
            let role_code = primitives::governance_skeleton::province_commissioner_role_code(
                info.province_code,
            );
            FederalRegistryProvinceAssignments {
                province_code: info.province_code,
                province_name: info.province_name.to_string(),
                assignments: assignments
                    .iter()
                    .filter(|assignment| assignment.role_code.as_bytes() == role_code.as_slice())
                    .cloned()
                    .collect(),
            }
        })
        .collect())
}

#[cfg(test)]
mod scale_contract_tests {
    use super::*;
    use entity_primitives::{
        AuthorizationSubject, BusinessActionId, RoleBusinessPermission, RoleSubject,
    };
    use frame_support::pallet_prelude::{BoundedVec, ConstU32};

    type FixtureCid = BoundedVec<u8, ConstU32<32>>;
    type FixtureRoleCode = BoundedVec<u8, ConstU32<64>>;
    type FixtureModuleTag = BoundedVec<u8, ConstU32<32>>;
    type FixtureSubject = AuthorizationSubject<FixtureCid, FixtureRoleCode, [u8; 32]>;
    type FixtureVoters = BoundedVec<FixtureSubject, ConstU32<256>>;
    type FixtureVotePlan = (
        BusinessActionId<FixtureModuleTag>,
        FixtureModuleTag,
        FixtureSubject,
        FixtureVoters,
        u8,
        [u8; 32],
    );

    #[test]
    fn assignment_term_window_is_inclusive_and_non_term_requires_zeroes() {
        let role = RawInstitutionRole {
            cid_number: b"CID".to_vec(),
            role_code: b"ROLE".to_vec(),
            role_name: b"Role".to_vec(),
            term_required: true,
            role_status: ROLE_STATUS_ACTIVE,
        };
        let mut assignment = RawInstitutionAssignment {
            cid_number: b"CID".to_vec(),
            account_id: [1; 32],
            role_code: b"ROLE".to_vec(),
            term_start: 10,
            term_end: 20,
            assignment_source: 5,
            assignment_source_ref: b"proposal".to_vec(),
            assignment_status: ASSIGNMENT_STATUS_ACTIVE,
        };
        assert!(assignment_is_effective(&role, &assignment, 10));
        assert!(assignment_is_effective(&role, &assignment, 20));
        assert!(!assignment_is_effective(&role, &assignment, 21));
        assignment.term_start = 0;
        assignment.term_end = 0;
        assert!(!assignment_is_effective(&role, &assignment, 10));
    }

    fn fixture_case(name: &str) -> Vec<u8> {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../memory/06-quality/fixtures/institution_role_permission_v1.json"
        )))
        .expect("岗位权限 fixture 必须是合法 JSON");
        let encoded = fixture["cases"]
            .as_array()
            .and_then(|cases| {
                cases
                    .iter()
                    .find(|case| case["name"].as_str() == Some(name))
            })
            .and_then(|case| case["encoded_hex"].as_str())
            .expect("岗位权限 fixture 用例必须存在");
        hex::decode(encoded).expect("岗位权限 fixture 必须是合法 hex")
    }

    fn decode_exact<T: Decode>(bytes: &[u8]) -> T {
        let mut input = bytes;
        let decoded = T::decode(&mut input).expect("fixture SCALE 必须可解码");
        assert!(input.is_empty(), "fixture SCALE 不得存在尾随字节");
        decoded
    }

    #[test]
    fn institution_role_permission_fixture_decodes_exactly() {
        let role: RoleSubject<Vec<u8>, Vec<u8>> =
            decode_exact(&fixture_case("role_subject_nrc_committee"));
        assert_eq!(role.cid_number, b"LN001-NRC0G-944805165-2026");
        assert_eq!(role.role_code, b"COMMITTEE_MEMBER");

        let _: RoleBusinessPermission<Vec<u8>, Vec<u8>, Vec<u8>> =
            decode_exact(&fixture_case("permission_resolution_issuance_propose"));
        let _: AuthorizationSubject<Vec<u8>, Vec<u8>, [u8; 32]> =
            decode_exact(&fixture_case("authorization_personal_multisig"));

        let plan: FixtureVotePlan =
            decode_exact(&fixture_case("vote_plan_resolution_issuance_joint"));
        assert_eq!(plan.1.as_slice(), b"res-iss");
        assert_eq!(plan.3.len(), 3);
        assert_eq!(plan.4, 1);
        assert_eq!(plan.5, [0xabu8; 32]);
    }
}
