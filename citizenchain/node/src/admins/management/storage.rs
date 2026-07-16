use std::collections::{BTreeMap, HashMap};

use ::codec::{Decode, Encode};
use entity_primitives::{
    InstitutionAdminAssignment, InstitutionAssignmentSource, InstitutionAssignmentStatus,
    InstitutionRole, InstitutionRoleStatus,
};
use primitives::cid::code::{
    is_fixed_governance_code, is_private_legal_code, is_public_legal_code, is_unincorporated_code,
    InstitutionCode, NRC, PRB, PRC,
};

use super::codec;
use super::types::{
    institution_code_label, kind_label, AdminAccountState, InstitutionAdminInfo,
    InstitutionRoleAssignmentInfo,
};
use crate::governance::registry;
use crate::governance::types::InstitutionType;
use crate::governance::{chain_query, storage_keys};

type RawRole = InstitutionRole<Vec<u8>, Vec<u8>, Vec<u8>>;
type RawAssignment = InstitutionAdminAssignment<Vec<u8>, [u8; 32], Vec<u8>, Vec<u8>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdminPalletSpec {
    pub pallet_name: &'static str,
    pub entity_pallet_name: &'static str,
    pub kind: u8,
}

const PUBLIC_ADMINS: AdminPalletSpec = AdminPalletSpec {
    pallet_name: "PublicAdmins",
    entity_pallet_name: "PublicManage",
    kind: 0,
};
const PRIVATE_ADMINS: AdminPalletSpec = AdminPalletSpec {
    pallet_name: "PrivateAdmins",
    entity_pallet_name: "PrivateManage",
    kind: 1,
};

/// 机构码能明确分类时选择管理员 pallet；非法人必须通过双探测取得实际归属。
pub fn admin_pallet_for_code(code: &InstitutionCode) -> Result<AdminPalletSpec, String> {
    if is_fixed_governance_code(code) || is_public_legal_code(code) {
        return Ok(PUBLIC_ADMINS);
    }
    if is_private_legal_code(code) {
        return Ok(PRIVATE_ADMINS);
    }
    if is_unincorporated_code(code) {
        return Err("非法人机构不能仅按机构码推断公权或私权管理员模块".to_string());
    }
    Err(format!(
        "无法按机构码 {} 路由机构管理员模块",
        institution_code_label(code)
    ))
}

fn builtin_governance_code(cid_number: &str) -> Result<InstitutionCode, String> {
    let entry = registry::find_institution(cid_number)
        .ok_or_else(|| format!("未知的内置治理机构 cidNumber: {cid_number}"))?;
    Ok(match entry.org_type() {
        InstitutionType::Nrc => NRC,
        InstitutionType::Prc => PRC,
        InstitutionType::Prb => PRB,
    })
}

/// 构造机构管理员 pallet 的 `AdminAccounts[cid_number]` StorageMap key。
pub fn admin_accounts_key_for_pallet(pallet_name: &str, cid_number: &[u8]) -> String {
    let pallet_hash = storage_keys::twox_128(pallet_name.as_bytes());
    let storage_hash = storage_keys::twox_128(b"AdminAccounts");
    let encoded_cid = cid_number.to_vec().encode();
    let blake2_hash = storage_keys::blake2_128(&encoded_cid);
    let mut key = Vec::with_capacity(48 + encoded_cid.len());
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash);
    key.extend_from_slice(&encoded_cid);
    format!("0x{}", hex::encode(key))
}

pub fn fetch_admin_account_by_cid_number(
    cid_number: &str,
) -> Result<Option<AdminAccountState>, String> {
    if let Ok(institution_code) = builtin_governance_code(cid_number) {
        return fetch_admin_account_for_code(cid_number, institution_code);
    }
    fetch_admin_account(cid_number)
}

/// 按 CID 在公权/私权管理员模块双探测，非法人也以实际命中的 pallet 决定 entity 路由。
pub fn fetch_admin_account(cid_number: &str) -> Result<Option<AdminAccountState>, String> {
    let finalized_hash = chain_query::fetch_finalized_head()?;
    let mut found = None;
    for spec in [PUBLIC_ADMINS, PRIVATE_ADMINS] {
        let storage_key = admin_accounts_key_for_pallet(spec.pallet_name, cid_number.as_bytes());
        let Some(hex_data) = chain_query::fetch_storage_at(&storage_key, &finalized_hash)? else {
            continue;
        };
        if found.is_some() {
            return Err("同一账户在多个机构管理员模块中存在，链上状态不一致".to_string());
        }
        found = Some(decode_admin_account_state(
            cid_number,
            spec,
            &hex_data,
            &finalized_hash,
        )?);
    }
    Ok(found)
}

pub fn fetch_admin_account_for_code(
    cid_number: &str,
    institution_code: InstitutionCode,
) -> Result<Option<AdminAccountState>, String> {
    let finalized_hash = chain_query::fetch_finalized_head()?;
    fetch_admin_account_for_code_at(cid_number, institution_code, &finalized_hash)
}

pub fn fetch_admin_account_for_code_at(
    cid_number: &str,
    institution_code: InstitutionCode,
    finalized_hash: &str,
) -> Result<Option<AdminAccountState>, String> {
    let spec = admin_pallet_for_code(&institution_code)?;
    let storage_key = admin_accounts_key_for_pallet(spec.pallet_name, cid_number.as_bytes());
    let Some(hex_data) = chain_query::fetch_storage_at(&storage_key, finalized_hash)? else {
        return Ok(None);
    };
    let state = decode_admin_account_state(cid_number, spec, &hex_data, finalized_hash)?;
    if state.institution_code != institution_code {
        return Err(format!(
            "管理员账户机构码不匹配：查询 {}，链上 {}",
            institution_code_label(&institution_code),
            institution_code_label(&state.institution_code)
        ));
    }
    Ok(Some(state))
}

fn decode_admin_account_state(
    cid_number: &str,
    spec: AdminPalletSpec,
    hex_data: &str,
    finalized_hash: &str,
) -> Result<AdminAccountState, String> {
    let data = decode_hex_storage(hex_data)?;
    let decoded = codec::decode_admin_account(&data)?;
    let admins = fetch_role_assignments(
        spec,
        cid_number.as_bytes(),
        decoded.admins.as_slice(),
        finalized_hash,
    )?;
    Ok(AdminAccountState {
        cid_number: cid_number.to_string(),
        institution_code: decoded.institution_code,
        institution_code_label: institution_code_label(&decoded.institution_code),
        kind: spec.kind,
        kind_label: kind_label(spec.kind).to_string(),
        admins,
    })
}

fn role_storage_prefix(spec: AdminPalletSpec, storage: &str, cid_number: &[u8]) -> String {
    let mut key =
        crate::shared::storage_keys::prefix(spec.entity_pallet_name.as_bytes(), storage.as_bytes());
    key.extend_from_slice(&crate::shared::storage_keys::blake2_128_concat(
        &cid_number.to_vec().encode(),
    ));
    crate::shared::storage_keys::to_hex(&key)
}

fn fetch_all_keys_at(prefix: &str, finalized_hash: &str) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();
    let mut start_key: Option<String> = None;
    loop {
        let page =
            chain_query::fetch_keys_paged_at(prefix, 256, start_key.as_deref(), finalized_hash)?;
        if page.is_empty() {
            break;
        }
        let page_len = page.len();
        start_key = page.last().cloned();
        keys.extend(page);
        if page_len < 256 {
            break;
        }
    }
    Ok(keys)
}

fn decode_exact<T: Decode>(hex_data: &str, label: &str) -> Result<T, String> {
    let bytes = decode_hex_storage(hex_data)?;
    let mut input = bytes.as_slice();
    let value = T::decode(&mut input).map_err(|e| format!("{label} SCALE 解码失败: {e}"))?;
    if !input.is_empty() {
        return Err(format!("{label} 存在尾随字节"));
    }
    Ok(value)
}

fn source_label(source: InstitutionAssignmentSource) -> &'static str {
    match source {
        InstitutionAssignmentSource::Genesis => "创世",
        InstitutionAssignmentSource::Registry => "注册局",
        InstitutionAssignmentSource::PopularElection => "普选",
        InstitutionAssignmentSource::MutualElection => "互选",
        InstitutionAssignmentSource::NominationAppointment => "提名任免",
    }
}

fn utf8(value: Vec<u8>, label: &str) -> Result<String, String> {
    String::from_utf8(value).map_err(|_| format!("{label} 不是 UTF-8"))
}

/// 来源引用由业务模块定义，既可能是可读登记号，也可能是 SCALE 编码的提案 ID。
fn display_source_ref(value: Vec<u8>) -> String {
    match String::from_utf8(value.clone()) {
        Ok(text) if !text.chars().any(char::is_control) => text,
        _ => format!("0x{}", hex::encode(value)),
    }
}

fn fetch_role_assignments(
    spec: AdminPalletSpec,
    cid_number: &[u8],
    admin_accounts: &[String],
    finalized_hash: &str,
) -> Result<Vec<InstitutionAdminInfo>, String> {
    let role_prefix = role_storage_prefix(spec, "InstitutionRoles", cid_number);
    let assignment_prefix = role_storage_prefix(spec, "InstitutionRoleAssignments", cid_number);
    let mut roles = HashMap::<Vec<u8>, RawRole>::new();
    for key in fetch_all_keys_at(&role_prefix, finalized_hash)? {
        let value = chain_query::fetch_storage_at(&key, finalized_hash)?
            .ok_or_else(|| "岗位 key 在同一 finalized 快照中缺少 value".to_string())?;
        let role: RawRole = decode_exact(&value, "InstitutionRoles")?;
        if role.cid_number == cid_number && role.role_status == InstitutionRoleStatus::Active {
            roles.insert(role.role_code.clone(), role);
        }
    }

    let mut grouped = BTreeMap::<String, Vec<InstitutionRoleAssignmentInfo>>::new();
    for account in admin_accounts {
        grouped.entry(account.to_ascii_lowercase()).or_default();
    }
    for key in fetch_all_keys_at(&assignment_prefix, finalized_hash)? {
        let value = chain_query::fetch_storage_at(&key, finalized_hash)?
            .ok_or_else(|| "任职 key 在同一 finalized 快照中缺少 value".to_string())?;
        let assignments: Vec<RawAssignment> = decode_exact(&value, "InstitutionRoleAssignments")?;
        for assignment in assignments {
            if assignment.cid_number != cid_number
                || assignment.assignment_status != InstitutionAssignmentStatus::Active
            {
                continue;
            }
            let account = hex::encode(assignment.admin_account);
            let Some(account_assignments) = grouped.get_mut(&account) else {
                continue;
            };
            let role = roles
                .get(&assignment.role_code)
                .ok_or_else(|| "有效机构任职引用了不存在或已停用的岗位".to_string())?;
            account_assignments.push(InstitutionRoleAssignmentInfo {
                role_code: utf8(assignment.role_code, "岗位码")?,
                role_name: utf8(role.role_name.clone(), "岗位名称")?,
                term_required: role.term_required,
                term_start: assignment.term_start,
                term_end: assignment.term_end,
                assignment_source: assignment.assignment_source as u8,
                assignment_source_label: source_label(assignment.assignment_source).to_string(),
                assignment_source_ref: display_source_ref(assignment.assignment_source_ref),
            });
        }
    }

    let mut admins = Vec::with_capacity(grouped.len());
    for (account, mut assignments) in grouped {
        if assignments.is_empty() {
            return Err(format!("机构管理员钱包 {account} 缺少有效岗位任职"));
        }
        assignments.sort_by(|left, right| left.role_code.cmp(&right.role_code));
        admins.push(InstitutionAdminInfo {
            account,
            assignments,
        });
    }
    Ok(admins)
}

pub fn fetch_institution_admins_by_cid_number(
    cid_number: &str,
) -> Result<Vec<InstitutionAdminInfo>, String> {
    Ok(fetch_admin_account_by_cid_number(cid_number)?
        .map(|state| state.admins)
        .unwrap_or_default())
}

fn decode_hex_storage(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).map_err(|e| format!("hex 解码失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::display_source_ref;

    #[test]
    fn source_ref_keeps_text_and_hexes_binary_ids() {
        assert_eq!(display_source_ref(b"registry-1".to_vec()), "registry-1");
        assert_eq!(display_source_ref(vec![1, 0, 0, 0]), "0x01000000");
    }
}
