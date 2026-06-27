use super::account_id;
use super::codec;
use super::types::{institution_code_label, kind_label, status_label, AdminAccountState};
use crate::governance::registry;
use crate::governance::types::OrgType;
use crate::governance::{chain_query, storage_keys};
use primitives::cid::code::{
    is_fixed_governance_code, is_personal_code, is_private_legal_code, is_public_legal_code,
    is_unincorporated_code, InstitutionCode, NRC, PRB, PRC,
};

const FRG: InstitutionCode = *b"FRG\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdminPalletSpec {
    pub pallet_name: &'static str,
    pub pallet_index: u8,
}

const GENESIS_ADMINS: AdminPalletSpec = AdminPalletSpec {
    pallet_name: "GenesisAdmins",
    pallet_index: 12,
};
const PERSONAL_ADMINS: AdminPalletSpec = AdminPalletSpec {
    pallet_name: "PersonalAdmins",
    pallet_index: 7,
};
const PUBLIC_ADMINS: AdminPalletSpec = AdminPalletSpec {
    pallet_name: "PublicAdmins",
    pallet_index: 29,
};
const PRIVATE_ADMINS: AdminPalletSpec = AdminPalletSpec {
    pallet_name: "PrivateAdmins",
    pallet_index: 30,
};

/// 按机构码选择新 runtime 管理员 pallet。
pub fn admin_pallet_for_code(code: &InstitutionCode) -> Result<AdminPalletSpec, String> {
    if is_fixed_governance_code(code) || *code == FRG {
        return Ok(GENESIS_ADMINS);
    }
    if is_personal_code(code) {
        return Ok(PERSONAL_ADMINS);
    }
    if is_public_legal_code(code) {
        return Ok(PUBLIC_ADMINS);
    }
    if is_private_legal_code(code) || is_unincorporated_code(code) {
        return Ok(PRIVATE_ADMINS);
    }
    Err(format!(
        "无法按机构码 {} 路由管理员模块",
        institution_code_label(code)
    ))
}

fn admin_pallet_candidates() -> [AdminPalletSpec; 4] {
    [
        GENESIS_ADMINS,
        PERSONAL_ADMINS,
        PUBLIC_ADMINS,
        PRIVATE_ADMINS,
    ]
}

fn builtin_governance_code(cid_number: &str) -> Result<InstitutionCode, String> {
    let entry = registry::find_institution(cid_number)
        .ok_or_else(|| format!("未知的内置治理机构 cidNumber: {cid_number}"))?;
    Ok(match entry.org_type() {
        OrgType::Nrc => NRC,
        OrgType::Prc => PRC,
        OrgType::Prb => PRB,
    })
}

/// 构造新 runtime 管理员 pallet 的 `AdminAccounts` StorageMap key。
pub fn admin_accounts_key_for_pallet(pallet_name: &str, account_id: &[u8; 32]) -> String {
    let pallet_hash = storage_keys::twox_128(pallet_name.as_bytes());
    let storage_hash = storage_keys::twox_128(b"AdminAccounts");
    let blake2_hash = storage_keys::blake2b_128(account_id);

    let mut key = Vec::with_capacity(16 + 16 + 16 + 32);
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash);
    key.extend_from_slice(account_id);
    format!("0x{}", hex::encode(key))
}

/// 构造当前机构码对应管理员 pallet 的 `AdminAccounts` StorageMap key。
pub fn admin_accounts_key(
    institution_code: &InstitutionCode,
    account_id: &[u8; 32],
) -> Result<String, String> {
    let spec = admin_pallet_for_code(institution_code)?;
    Ok(admin_accounts_key_for_pallet(spec.pallet_name, account_id))
}

pub fn fetch_admin_account_by_cid_number(
    cid_number: &str,
) -> Result<Option<AdminAccountState>, String> {
    let account_id = account_id::account_id_from_builtin_cid(cid_number)?;
    let institution_code = builtin_governance_code(cid_number)?;
    fetch_admin_account_for_code(&account_id, institution_code, Some(cid_number.to_string()))
}

pub fn fetch_admin_account(
    account_id: &[u8; 32],
    cid_number: Option<String>,
) -> Result<Option<AdminAccountState>, String> {
    let mut found = None;
    for spec in admin_pallet_candidates() {
        let storage_key = admin_accounts_key_for_pallet(spec.pallet_name, account_id);
        let Some(hex_data) = chain_query::fetch_finalized_storage(&storage_key)? else {
            continue;
        };
        if found.is_some() {
            return Err("同一账户在多个管理员模块中存在，链上状态不一致".to_string());
        }
        found = Some(decode_admin_account_state(
            account_id,
            cid_number.clone(),
            &hex_data,
        )?);
    }
    Ok(found)
}

pub fn fetch_admin_account_for_code(
    account_id: &[u8; 32],
    institution_code: InstitutionCode,
    cid_number: Option<String>,
) -> Result<Option<AdminAccountState>, String> {
    let storage_key = admin_accounts_key(&institution_code, account_id)?;
    // 中文注释(ADR-017):管理员账户状态按 finalized 口径读取,禁止 best。
    let Some(hex_data) = chain_query::fetch_finalized_storage(&storage_key)? else {
        return Ok(None);
    };
    let state = decode_admin_account_state(account_id, cid_number, &hex_data)?;
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
    account_id: &[u8; 32],
    cid_number: Option<String>,
    hex_data: &str,
) -> Result<AdminAccountState, String> {
    let data = decode_hex_storage(&hex_data)?;
    let decoded = codec::decode_admin_account(&data)?;
    Ok(AdminAccountState {
        account_hex: hex::encode(account_id),
        cid_number,
        institution_code: decoded.institution_code,
        institution_code_label: institution_code_label(&decoded.institution_code),
        kind: decoded.kind,
        kind_label: kind_label(decoded.kind).to_string(),
        admins: decoded.admins,
        creator_hex: decoded.creator_hex,
        created_at: decoded.created_at,
        updated_at: decoded.updated_at,
        status: decoded.status,
        status_label: status_label(decoded.status).to_string(),
    })
}

pub fn fetch_admins_by_cid_number(cid_number: &str) -> Result<Vec<String>, String> {
    Ok(fetch_admin_account_by_cid_number(cid_number)?
        .map(|state| state.admins)
        .unwrap_or_default())
}

fn decode_hex_storage(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).map_err(|e| format!("hex 解码失败: {e}"))
}
