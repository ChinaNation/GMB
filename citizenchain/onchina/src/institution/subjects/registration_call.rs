//! 组装 `propose_create_institution` 的链上参数并编码为裸 SCALE call data。
//!
//! 本模块把链下机构最小身份/管理员 + 注册局签发凭证,组装成与链端逐字节
//! 对齐的 `ProposeCreateInstitutionArgs`,再交 `core::institution_call` 编码。
//! onchina 只产 call data,不拼签名扩展尾、不提交 extrinsic。
//!
//! 管理员组装规则：钱包能命中公民资料时使用公民姓名，否则名称固定为“管理员”；
//! 姓名只展示，唯一授权字段仍是钱包账户。首次登记不提交岗位或任职。
//! 机构 `cid_short_name` 只取 subjects.cid_short_name,与 `cid_full_name` 同源上链。

use uuid::Uuid;

use crate::auth::login::parse_sr25519_pubkey_bytes;
use crate::core::institution_call::{
    encode_admins_payload, encode_propose_create_institution, ChainCall, InstitutionAdminArg,
    ProposeCreateInstitutionArgs,
};
use crate::institution::subjects::model::CreateInstitutionAdminInput;
use crate::AppState;

/// 组装并编码 `propose_create_institution` 裸 call data(进 QR `b.d`)。
///
/// 凭证里的 register_nonce/signature/issuer/scope 已嵌入返回的 call data;
/// onchina 不提交 extrinsic,冷钱包解码核对后冷签 origin 并由 CitizenWallet 提交。
pub(crate) fn build_create_institution_call_data(
    state: &AppState,
    actor_cid_number: &str,
    inst: &crate::institution::subjects::model::Institution,
    admin_forms: &[CreateInstitutionAdminInput],
) -> Result<ChainCall, String> {
    let cid_number = inst.cid_number.trim();
    if cid_number.is_empty() {
        return Err("http:bad_request:cid_number is required".to_string());
    }

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
    let mut seen_accounts = std::collections::HashSet::new();
    let mut admins = Vec::with_capacity(admin_forms.len());
    for form in admin_forms {
        let admin_account = parse_sr25519_pubkey_bytes(form.admin_account.as_str())
            .ok_or_else(|| "http:bad_request:admin_account format invalid".to_string())?;
        if !seen_accounts.insert(admin_account) {
            return Err("http:bad_request:duplicate admin_account".to_string());
        }
        let admin_name = state
            .db
            .find_citizen_by_wallet(form.admin_account.as_str())?
            .map(|citizen| {
                format!(
                    "{}{}",
                    citizen.citizen_family_name.trim(),
                    citizen.citizen_given_name.trim()
                )
            })
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "管理员".to_string());
        admins.push(InstitutionAdminArg {
            admin_name: admin_name.into_bytes(),
            admin_account,
        });
    }
    if admins.len() < 2 {
        return Err("http:bad_request:at least two admins are required".to_string());
    }

    // ── 注册局签发凭证(复用唯一原语;不在此处重写签名逻辑)。
    let register_nonce = Uuid::new_v4().to_string();
    let admins_payload = encode_admins_payload(&admins);
    let credential = crate::core::chain_runtime::build_institution_creation_credential(
        state,
        actor_cid_number,
        cid_number,
        cid_full_name.as_str(),
        cid_short_name.as_str(),
        &admins_payload,
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
        admins,
        institution_code: code_bytes,
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
