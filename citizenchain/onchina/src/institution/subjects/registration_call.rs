//! 组装 `propose_create_institution` 的链上参数并编码为裸 SCALE call data。
//!
//! 中文注释:本模块把链下机构/账户/管理员数据 + 注册局签发凭证,组装成与链端逐字节
//! 对齐的 `ProposeCreateInstitutionArgs`,再交 `core::institution_call` 编码。
//! onchina 只产 call data,不拼签名扩展尾、不提交 extrinsic。
//!
//! AdminProfile 组装规则(ADR-030/A2):
//! - `account`:管理员进链账户(institution_admins.admin_account);
//! - `admin_cid_number` / `name`:来自注册局公民记录(citizens 关联 subjects.cid_full_name);
//! - `title` / `term_start` / `term_end`:来自创建表单;`source` 固定 `Registry`。
//! 公权机构 `cid_short_name` 取官方简称;私权机构留空(链端按 A1 存空)。

use postgres::Client;
use uuid::Uuid;

use crate::auth::login::parse_sr25519_pubkey_bytes;
use crate::core::institution_call::{
    encode_propose_create_institution, AdminProfileArg, AdminSourceTag, InitialAccountArg,
    ProposeCreateInstitutionArgs,
};
use crate::AppState;

/// 创建表单里每个管理员的职务/任期补充(account 与链下 institution_admins 对齐)。
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct AdminProfileFormInput {
    /// 进链账户(hex 或 SS58),与 institution_admins.admin_account 同源。
    pub account: String,
    #[serde(default)]
    pub title: Option<String>,
    /// 任期开始(天数自纪元;无任期填 0)。
    #[serde(default)]
    pub term_start: Option<u32>,
    /// 任期结束(天数自纪元;无任期填 0)。
    #[serde(default)]
    pub term_end: Option<u32>,
}

/// 一个管理员的实名锚信息(admin_cid_number + name),由 DB 联表派生。
struct AdminIdentity {
    admin_cid_number: Vec<u8>,
    name: Vec<u8>,
}

/// 在已有连接上,按管理员进链账户联表派生其实名锚(cid_number + 姓名)。
/// 链:institution_admins.admin_account → citizens.wallet_pubkey → citizens.cid_number
/// → subjects(kind='CITIZEN').cid_full_name。查不到则留空(链端接受空 BoundedVec)。
fn resolve_admin_identity_conn(conn: &mut Client, admin_account: &str) -> AdminIdentity {
    let sql = "SELECT c.cid_number, s.cid_full_name
               FROM citizens c
               LEFT JOIN subjects s
                 ON s.cid_number = c.cid_number AND s.kind = 'CITIZEN'
               WHERE c.wallet_pubkey = $1
               LIMIT 1";
    match conn.query_opt(sql, &[&admin_account]) {
        Ok(Some(row)) => {
            let cid: Option<String> = row.get(0);
            let name: Option<String> = row.get(1);
            AdminIdentity {
                admin_cid_number: cid.unwrap_or_default().into_bytes(),
                name: name.unwrap_or_default().into_bytes(),
            }
        }
        _ => AdminIdentity {
            admin_cid_number: Vec::new(),
            name: Vec::new(),
        },
    }
}

/// 组装并编码 `propose_create_institution` 裸 call data(进 QR `b.d`)。
///
/// 凭证里的 register_nonce/signature/issuer/scope 已嵌入返回的 call data;
/// onchina 不提交 extrinsic,冷钱包解码核对后冷签 origin 并由 CitizenWallet 提交。
pub(crate) fn build_create_institution_call_data(
    state: &AppState,
    conn: &mut Client,
    cid_number: &str,
    threshold: u32,
    admin_forms: &[AdminProfileFormInput],
) -> Result<Vec<u8>, String> {
    let cid_number = cid_number.trim();
    if cid_number.is_empty() {
        return Err("http:bad_request:cid_number is required".to_string());
    }

    // ── 机构 + 账户(账户名进链 name;初始余额恒 0,链端无 amount>0 约束)。
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
    // 私权机构留空(链端按 A1 存空);公权/教育机构取官方简称(单源 cid_short_name)。
    // 机构码私权判定复用 primitives 单源,杜绝码表漂移。
    let cid_short_name = if primitives::cid::code::is_private_legal_code(&code_bytes) {
        String::new()
    } else {
        inst.cid_short_name.clone().unwrap_or_default()
    };

    let account_args: Vec<InitialAccountArg> = accounts
        .iter()
        .filter(|a| !a.account_name.trim().is_empty())
        .map(|a| InitialAccountArg {
            account_name: a.account_name.trim().to_string(),
            amount: 0,
        })
        .collect();
    if account_args.is_empty() {
        return Err("http:conflict:at least one account_name is required".to_string());
    }

    // ── 管理员集合(AdminProfile)。account 来自 institution_admins;title/term 来自表单;
    //    admin_cid_number/name 联表派生;source 固定 Registry。
    let db_admins =
        crate::institution::admins::repo::list_institution_admins_by_cid_conn(conn, cid_number)?;
    if db_admins.len() < 2 {
        return Err("http:conflict:at least two admins are required".to_string());
    }
    let admins_len = db_admins.len() as u32;
    let min_threshold = admins_len / 2 + 1;
    if threshold < min_threshold || threshold > admins_len {
        return Err(format!(
            "http:bad_request:threshold must be in {min_threshold}..={admins_len}"
        ));
    }

    let mut admin_args: Vec<AdminProfileArg> = Vec::with_capacity(db_admins.len());
    for admin in &db_admins {
        let account = parse_sr25519_pubkey_bytes(admin.admin_account.as_str())
            .ok_or_else(|| "http:bad_request:admin_account format invalid".to_string())?;
        let form = admin_forms
            .iter()
            .find(|f| accounts_match(f.account.as_str(), admin.admin_account.as_str()));
        let identity = resolve_admin_identity_conn(conn, admin.admin_account.as_str());
        admin_args.push(AdminProfileArg {
            account,
            admin_cid_number: identity.admin_cid_number,
            name: identity.name,
            title: form
                .and_then(|f| f.title.clone())
                .unwrap_or_default()
                .into_bytes(),
            term_start: form.and_then(|f| f.term_start).unwrap_or(0),
            term_end: form.and_then(|f| f.term_end).unwrap_or(0),
            source: AdminSourceTag::Registry,
        });
    }

    // ── 注册局签发凭证(复用唯一原语;不在此处重写签名逻辑)。
    let account_names: Vec<String> = account_args
        .iter()
        .map(|a| a.account_name.clone())
        .collect();
    let register_nonce = Uuid::new_v4().to_string();
    let credential = crate::core::chain_runtime::build_institution_registration_credential(
        state,
        cid_number,
        cid_full_name.as_str(),
        &account_names,
        register_nonce.clone(),
        inst.province_name.as_str(),
        inst.city_name.as_str(),
    )?;

    let issuer_main_account = hex_to_bytes32(credential.issuer_main_account.as_str())
        .ok_or_else(|| "http:internal:issuer_main_account parse failed".to_string())?;
    let signer_pubkey = hex_to_bytes32(credential.signer_pubkey.as_str())
        .ok_or_else(|| "http:internal:signer_pubkey parse failed".to_string())?;
    let signature = hex_to_vec(credential.signature.as_str())
        .ok_or_else(|| "http:internal:signature parse failed".to_string())?;

    let args = ProposeCreateInstitutionArgs {
        cid_number: cid_number.as_bytes().to_vec(),
        cid_full_name: cid_full_name.trim().as_bytes().to_vec(),
        cid_short_name: cid_short_name.trim().as_bytes().to_vec(),
        accounts: account_args,
        institution_code: code_bytes,
        admins_len,
        admins: admin_args,
        threshold,
        register_nonce: credential.register_nonce.into_bytes(),
        signature,
        issuer_cid_number: credential.issuer_cid_number.into_bytes(),
        issuer_main_account,
        signer_pubkey,
        scope_province_name: credential.scope_province_name.into_bytes(),
        scope_city_name: credential.scope_city_name.into_bytes(),
    };

    Ok(encode_propose_create_institution(&args))
}

/// 两个账户标识(hex/SS58 任一形态)解析为同一 32 字节即视为相等。
fn accounts_match(left: &str, right: &str) -> bool {
    match (
        parse_sr25519_pubkey_bytes(left),
        parse_sr25519_pubkey_bytes(right),
    ) {
        (Some(l), Some(r)) => l == r,
        _ => left.trim() == right.trim(),
    }
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
