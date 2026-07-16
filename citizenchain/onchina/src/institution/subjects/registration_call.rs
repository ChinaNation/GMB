//! 组装 `propose_create_institution` 的链上参数并编码为裸 SCALE call data。
//!
//! 本模块把链下机构/账户/管理员数据 + 注册局签发凭证,组装成与链端逐字节
//! 对齐的 `ProposeCreateInstitutionArgs`,再交 `core::institution_call` 编码。
//! onchina 只产 call data,不拼签名扩展尾、不提交 extrinsic。
//!
//! 管理员与岗位组装规则:
//! - `admins` 由任职记录中的钱包账户去重后在 runtime 派生，不在本调用重复编码；
//! - `roles` 保存机构岗位定义；
//! - `assignments` 保存管理员钱包与岗位的绑定，来源固定为注册局；
//! - 管理员公民身份和姓名不进入机构管理员链上结构。
//! 机构 `cid_short_name` 只取 subjects.cid_short_name,与 `cid_full_name` 同源上链。

use postgres::Client;
use uuid::Uuid;

use crate::auth::login::parse_sr25519_pubkey_bytes;
use crate::core::institution_call::{
    encode_assignments_payload, encode_propose_create_institution, encode_roles_payload, ChainCall,
    InitialAccountArg, InstitutionAssignmentArg, InstitutionAssignmentSourceTag,
    InstitutionAssignmentStatusTag, InstitutionRoleArg, InstitutionRoleStatusTag,
    ProposeCreateInstitutionArgs,
};
use crate::institution::subjects::model::CreateInstitutionAdminInput;
use crate::AppState;

/// 机构逻辑账户允许零初始余额；非零金额才受 ED 约束。
const DEFAULT_INITIAL_ACCOUNT_AMOUNT_FEN: u128 = 0;

/// 组装并编码 `propose_create_institution` 裸 call data(进 QR `b.d`)。
///
/// 凭证里的 register_nonce/signature/issuer/scope 已嵌入返回的 call data;
/// onchina 不提交 extrinsic,冷钱包解码核对后冷签 origin 并由 CitizenWallet 提交。
pub(crate) fn build_create_institution_call_data(
    state: &AppState,
    conn: &mut Client,
    actor_cid_number: &str,
    cid_number: &str,
    threshold: u32,
    admin_forms: &[CreateInstitutionAdminInput],
) -> Result<ChainCall, String> {
    let cid_number = cid_number.trim();
    if cid_number.is_empty() {
        return Err("http:bad_request:cid_number is required".to_string());
    }

    // ── 机构 + 账户(账户名进链 name;初始余额按链端 MinCreateAmount 最小值构造)。
    let Some((inst, accounts)) = crate::Db::get_institution_with_accounts_conn(conn, cid_number)?
    else {
        return Err("http:not_found:institution not found".to_string());
    };
    let cid_full_name = inst.cid_full_name.clone().unwrap_or_default();
    if cid_full_name.trim().is_empty() {
        return Err(
            "http:conflict:cid_full_name is required before chain registration".to_string(),
        );
    }
    let institution_code = inst.institution_code.clone();
    let code_bytes: [u8; 4] = {
        let mut buf = [0u8; 4];
        let raw = institution_code.as_bytes();
        if raw.len() > 4 {
            return Err("http:bad_request:institution_code must be <=4 bytes".to_string());
        }
        buf[..raw.len()].copy_from_slice(raw);
        buf
    };
    let cid_short_name = inst.cid_short_name.clone().unwrap_or_default();
    if cid_short_name.trim().is_empty() {
        return Err(
            "http:conflict:cid_short_name is required before chain registration".to_string(),
        );
    }
    let legal_representative_name = inst.legal_representative_name.clone().unwrap_or_default();
    if legal_representative_name.trim().is_empty() {
        return Err(
            "http:conflict:legal_representative_name is required before chain registration"
                .to_string(),
        );
    }
    let legal_representative_cid_number = inst
        .legal_representative_cid_number
        .clone()
        .unwrap_or_default();
    if legal_representative_cid_number.trim().is_empty() {
        return Err(
            "http:conflict:legal_representative_cid_number is required before chain registration"
                .to_string(),
        );
    }
    let legal_representative_account_text = inst
        .legal_representative_account
        .clone()
        .unwrap_or_default();
    let legal_representative_account = parse_sr25519_pubkey_bytes(
        legal_representative_account_text.as_str(),
    )
    .ok_or_else(|| {
        "http:conflict:legal_representative_account is required before chain registration"
            .to_string()
    })?;

    let account_args: Vec<InitialAccountArg> = accounts
        .iter()
        .filter(|a| !a.account_name.trim().is_empty())
        .map(|a| InitialAccountArg {
            account_name: a.account_name.trim().to_string(),
            amount: DEFAULT_INITIAL_ACCOUNT_AMOUNT_FEN,
        })
        .collect();
    if account_args.is_empty() {
        return Err("http:conflict:at least one account_name is required".to_string());
    }

    // ── 本地管理员表只核对钱包集合；岗位与任职以本次表单为唯一创建载荷。
    let db_admins =
        crate::institution::admins::repo::list_institution_admins_by_cid_conn(conn, cid_number)?;
    if db_admins.len() < 2 {
        return Err("http:conflict:at least two admins are required".to_string());
    }
    let mut db_accounts = std::collections::HashSet::new();
    for admin in &db_admins {
        let account = parse_sr25519_pubkey_bytes(admin.admin_account.as_str())
            .ok_or_else(|| "http:bad_request:admin_account format invalid".to_string())?;
        db_accounts.insert(account);
    }
    let mut form_accounts = std::collections::HashSet::new();
    for form in admin_forms {
        let account = parse_sr25519_pubkey_bytes(form.admin_account.as_str())
            .ok_or_else(|| "http:bad_request:admin_account format invalid".to_string())?;
        form_accounts.insert(account);
    }
    if db_accounts != form_accounts {
        return Err("http:conflict:admin wallet set changed before call encoding".to_string());
    }
    let admins_len = form_accounts.len() as u32;
    let min_threshold = admins_len / 2 + 1;
    if threshold < min_threshold || threshold > admins_len {
        return Err(format!(
            "http:bad_request:threshold must be in {min_threshold}..={admins_len}"
        ));
    }

    // ── 注册局签发凭证(复用唯一原语;不在此处重写签名逻辑)。
    let account_names: Vec<String> = account_args
        .iter()
        .map(|a| a.account_name.clone())
        .collect();
    let register_nonce = Uuid::new_v4().to_string();

    // 岗位按 role_code 去重，任职逐条保留；注册 nonce 是本次注册局任职来源引用。
    let mut role_codes = std::collections::HashSet::new();
    let mut roles = Vec::new();
    let mut assignments = Vec::with_capacity(admin_forms.len());
    for form in admin_forms {
        if role_codes.insert(form.role_code.clone()) {
            roles.push(InstitutionRoleArg {
                cid_number: cid_number.as_bytes().to_vec(),
                role_code: form.role_code.as_bytes().to_vec(),
                role_name: form.role_name.as_bytes().to_vec(),
                term_required: form.term_required,
                role_status: InstitutionRoleStatusTag::Active,
            });
        }
        assignments.push(InstitutionAssignmentArg {
            cid_number: cid_number.as_bytes().to_vec(),
            admin_account: parse_sr25519_pubkey_bytes(form.admin_account.as_str())
                .ok_or_else(|| "http:bad_request:admin_account format invalid".to_string())?,
            role_code: form.role_code.as_bytes().to_vec(),
            term_start: form.term_start.unwrap_or(0),
            term_end: form.term_end.unwrap_or(0),
            assignment_source: InstitutionAssignmentSourceTag::Registry,
            assignment_source_ref: register_nonce.as_bytes().to_vec(),
            assignment_status: InstitutionAssignmentStatusTag::Active,
        });
    }
    let roles_payload = encode_roles_payload(&roles);
    let assignments_payload = encode_assignments_payload(&assignments);
    // OnChina 当前创建账户初始余额统一为零，因此不存在本金支出账户。
    // 未来允许非零初始余额时，必须由 API 明确选择同一 actor CID 的账户，不能从管理员推导。
    let funding_account: Option<[u8; 32]> = None;
    let credential = crate::core::chain_runtime::build_institution_creation_credential(
        state,
        actor_cid_number,
        cid_number,
        cid_full_name.as_str(),
        cid_short_name.as_str(),
        legal_representative_name.as_str(),
        legal_representative_cid_number.as_str(),
        &legal_representative_account,
        &account_names,
        funding_account.as_ref(),
        &roles_payload,
        &assignments_payload,
        register_nonce.clone(),
        inst.province_name.as_str(),
        inst.city_name.as_str(),
        inst.town_code.as_str(),
    )?;

    let credential_signer_pubkey = hex_to_bytes32(credential.credential_signer_pubkey.as_str())
        .ok_or_else(|| "http:internal:credential_signer_pubkey parse failed".to_string())?;
    let signature = hex_to_vec(credential.signature.as_str())
        .ok_or_else(|| "http:internal:signature parse failed".to_string())?;

    let args = ProposeCreateInstitutionArgs {
        cid_number: cid_number.as_bytes().to_vec(),
        cid_full_name: cid_full_name.trim().as_bytes().to_vec(),
        cid_short_name: cid_short_name.trim().as_bytes().to_vec(),
        town_code: inst.town_code.trim().as_bytes().to_vec(),
        legal_representative_name: legal_representative_name.trim().as_bytes().to_vec(),
        legal_representative_cid_number: legal_representative_cid_number.trim().as_bytes().to_vec(),
        legal_representative_account,
        accounts: account_args,
        funding_account,
        institution_code: code_bytes,
        roles,
        assignments,
        threshold,
        register_nonce: credential.register_nonce.into_bytes(),
        signature,
        actor_cid_number: credential.actor_cid_number.into_bytes(),
        credential_signer_pubkey,
        scope_province_name: credential.scope_province_name.into_bytes(),
        scope_city_name: credential.scope_city_name.into_bytes(),
    };

    Ok(encode_propose_create_institution(&args))
}

/// 0x/裸 hex → 32 字节定长。
fn hex_to_bytes32(value: &str) -> Option<[u8; 32]> {
    let cleaned = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(cleaned).ok()?;
    bytes.as_slice().try_into().ok()
}

/// 0x/裸 hex → 变长字节。
fn hex_to_vec(value: &str) -> Option<Vec<u8>> {
    let cleaned = value.strip_prefix("0x").unwrap_or(value);
    hex::decode(cleaned).ok()
}
