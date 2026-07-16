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
    admin_account: [u8; 32],
    role_code: Vec<u8>,
    term_start: u32,
    term_end: u32,
    assignment_source: u8,
    assignment_source_ref: Vec<u8>,
    assignment_status: u8,
}

/// 管理员钱包在机构岗位上的一条有效任职投影。
#[derive(Debug, Clone)]
pub(crate) struct InstitutionAssignmentView {
    pub(crate) account_hex: String,
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
        _ => "",
    }
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
    active_admins: &HashSet<[u8; 32]>,
) -> Result<Vec<InstitutionAssignmentView>, String> {
    let active_roles: HashMap<Vec<u8>, RawInstitutionRole> = roles
        .into_iter()
        .filter(|role| role.role_status == ROLE_STATUS_ACTIVE)
        .map(|role| (role.role_code.clone(), role))
        .collect();
    let mut views = Vec::new();
    for assignment in assignments {
        if assignment.assignment_status != ASSIGNMENT_STATUS_ACTIVE
            || !active_admins.contains(&assignment.admin_account)
        {
            continue;
        }
        let Some(role) = active_roles.get(&assignment.role_code) else {
            return Err(
                "active institution assignment references a missing active role".to_string(),
            );
        };
        views.push(InstitutionAssignmentView {
            account_hex: format!("0x{}", hex::encode(assignment.admin_account)),
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
            .then(left.account_hex.cmp(&right.account_hex))
    });
    Ok(views)
}

/// 合并本机构 Active 管理员集合与 Active 任职；缺少任职的管理员视为链上不一致。
pub(crate) async fn fetch_active_assignments_onchain(
    identity: &NodeInstitutionIdentity,
) -> Result<Option<Vec<InstitutionAssignmentView>>, String> {
    let cid_number = identity.cid_number.as_str();
    let Some(admins) = fetch_active_admins_onchain(identity).await? else {
        return Ok(None);
    };
    let active_admins: HashSet<[u8; 32]> = admins
        .iter()
        .map(|account| {
            crate::auth::login::parse_sr25519_pubkey_bytes(account)
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
        let covered: HashSet<&str> = views.iter().map(|view| view.account_hex.as_str()).collect();
        if identity.frg_province_code.is_none() && covered.len() != active_admins.len() {
            return Err("active institution admin has no active role assignment".to_string());
        }
        return Ok(Some(views));
    }
    Err("institution roles not found in matching entity pallet".to_string())
}

/// 查找某个联邦注册局管理员钱包当前担任专员的省码。
pub(crate) async fn fetch_frg_province_codes_for_admin(
    cid_number: &[u8],
    admin_account: [u8; 32],
) -> Result<Vec<[u8; 2]>, String> {
    let (roles, assignments) =
        read_roles_and_assignments(cid_number, AdminPallet::PublicAdmins).await?;
    let active_role_codes: HashSet<Vec<u8>> = roles
        .into_iter()
        .filter(|role| role.role_status == ROLE_STATUS_ACTIVE)
        .map(|role| role.role_code)
        .collect();
    let assigned_codes: HashSet<Vec<u8>> = assignments
        .into_iter()
        .filter(|assignment| {
            assignment.admin_account == admin_account
                && assignment.assignment_status == ASSIGNMENT_STATUS_ACTIVE
                && active_role_codes.contains(&assignment.role_code)
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

/// 从 FRG entity 任职真源取指定省专员岗位的有效管理员钱包。
pub(crate) async fn fetch_frg_admins_for_province(
    cid_number: &[u8],
    province_code: [u8; 2],
) -> Result<HashSet<[u8; 32]>, String> {
    let (roles, assignments) =
        read_roles_and_assignments(cid_number, AdminPallet::PublicAdmins).await?;
    let expected = primitives::governance_skeleton::province_commissioner_role_code(province_code);
    let role_active = roles
        .into_iter()
        .any(|role| role.role_status == ROLE_STATUS_ACTIVE && role.role_code == expected);
    if !role_active {
        return Err("federal registry province commissioner role is not active".to_string());
    }
    Ok(assignments
        .into_iter()
        .filter(|assignment| {
            assignment.role_code == expected
                && assignment.assignment_status == ASSIGNMENT_STATUS_ACTIVE
        })
        .map(|assignment| assignment.admin_account)
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
