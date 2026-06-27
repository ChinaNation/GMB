//! 机构/账户业务校验 + 分类 + 唯一性
//!
//! 中文注释:凡是要在 handler 里做的业务级校验(不是简单的格式校验),都放这里。
//! handler 只负责调用 service + 转 HTTP 响应。

#![allow(dead_code)]

use crate::cid::code;
use crate::cid::{classify, validate_cid_number_format, AdminLevel, InstitutionCategory};
use crate::subjects::model::{Institution, InstitutionAccount};
use crate::subjects::MultisigChainStatus;
use primitives::account_derive::is_forbidden_account_name;

// 保留名字面单源 = primitives::account_derive::RESERVED_NAME_*_STR(链端唯一字面)。
// 本处仅以业务别名 re-export,禁止再写 "主账户" 等字面。
pub const ACCOUNT_NAME_MAIN: &str = primitives::account_derive::RESERVED_NAME_MAIN_STR;
pub const ACCOUNT_NAME_FEE: &str = primitives::account_derive::RESERVED_NAME_FEE_STR;
pub const ACCOUNT_NAME_STAKE: &str = primitives::account_derive::RESERVED_NAME_STAKE_STR;
pub const ACCOUNT_NAME_SAFETYFUND: &str = primitives::account_derive::RESERVED_NAME_SAFETYFUND_STR;
pub const ACCOUNT_NAME_HE: &str = primitives::account_derive::RESERVED_NAME_HE_STR;

pub const COMMON_DEFAULT_ACCOUNT_NAMES: &[&str] = &[ACCOUNT_NAME_MAIN, ACCOUNT_NAME_FEE];
pub const PROVINCE_RESERVE_BANK_DEFAULT_ACCOUNT_NAMES: &[&str] =
    &[ACCOUNT_NAME_MAIN, ACCOUNT_NAME_FEE, ACCOUNT_NAME_STAKE];
pub const NATIONAL_RESERVE_DEFAULT_ACCOUNT_NAMES: &[&str] = &[
    ACCOUNT_NAME_MAIN,
    ACCOUNT_NAME_FEE,
    ACCOUNT_NAME_SAFETYFUND,
    ACCOUNT_NAME_HE,
];

pub const DEFAULT_ACCOUNT_NAMES: &[&str] = &[
    ACCOUNT_NAME_MAIN,
    ACCOUNT_NAME_FEE,
    ACCOUNT_NAME_STAKE,
    ACCOUNT_NAME_SAFETYFUND,
    ACCOUNT_NAME_HE,
];

pub const MAX_ACCOUNT_NAME_CHARS: usize = 30;
pub const MAX_ACCOUNT_NAME_BYTES: usize = 128;
pub const MAX_INSTITUTION_NAME_CHARS: usize = 30;
pub const MAX_INSTITUTION_NAME_BYTES: usize = 128;
pub const MAX_LEGAL_REP_NAME_CHARS: usize = 30;
pub const MAX_LEGAL_REP_NAME_BYTES: usize = 128;
pub const MAX_LEGAL_REP_PHOTO_BYTES: u64 = 5 * 1024 * 1024;

pub struct LegalRepresentativeFields {
    pub legal_rep_name: String,
    pub cid_number: String,
    pub photo_path: String,
    pub photo_name: String,
    pub photo_mime: String,
    pub photo_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegalRepresentativeCitizenScope {
    Nationwide,
    Province {
        province_code: String,
    },
    City {
        province_code: String,
        city_code: String,
    },
}

impl LegalRepresentativeCitizenScope {
    pub fn province_code(&self) -> Option<&str> {
        match self {
            Self::Nationwide => None,
            Self::Province { province_code } | Self::City { province_code, .. } => {
                Some(province_code.as_str())
            }
        }
    }

    pub fn city_code(&self) -> Option<&str> {
        match self {
            Self::City { city_code, .. } => Some(city_code.as_str()),
            Self::Nationwide | Self::Province { .. } => None,
        }
    }

    pub fn legal_rep_error_message(&self) -> &'static str {
        match self {
            Self::Nationwide => "法定代表人身份ID必须选择正常状态公民",
            Self::Province { .. } => "该机构法定代表人必须是本省正常状态公民",
            Self::City { .. } => "该机构法定代表人必须是本市正常状态公民",
        }
    }
}

fn local_public_scope(province_code: &str, city_code: &str) -> LegalRepresentativeCitizenScope {
    let province_code = province_code.trim().to_string();
    let city_code = city_code.trim().to_string();
    if city_code.is_empty() || city_code == "000" {
        LegalRepresentativeCitizenScope::Province { province_code }
    } else {
        LegalRepresentativeCitizenScope::City {
            province_code,
            city_code,
        }
    }
}

fn public_org_scope(
    institution_code: &str,
    province_code: &str,
    city_code: &str,
) -> LegalRepresentativeCitizenScope {
    match code::institution_code_from_str(institution_code).and_then(|c| code::admin_level(&c)) {
        Some(AdminLevel::National) => LegalRepresentativeCitizenScope::Nationwide,
        Some(AdminLevel::Province) => LegalRepresentativeCitizenScope::Province {
            province_code: province_code.trim().to_string(),
        },
        // 市级、镇级、无层级公权机构都按落位省市收口;若没有市码则按省级处理。
        Some(AdminLevel::City) | Some(AdminLevel::Town) | None => {
            local_public_scope(province_code, city_code)
        }
    }
}

pub fn resolve_legal_representative_scope_for_codes(
    institution_code: &str,
    _education_type: Option<&str>,
    province_code: &str,
    city_code: &str,
    parent: Option<&Institution>,
) -> LegalRepresentativeCitizenScope {
    let parsed_institution_code = code::institution_code_from_str(institution_code);
    let is_public_legal = parsed_institution_code.map_or(false, |c| code::is_public_legal_code(&c));
    let is_unincorporated =
        parsed_institution_code.map_or(false, |c| code::is_unincorporated_code(&c));
    if is_public_legal {
        return public_org_scope(institution_code, province_code, city_code);
    }

    let parent_is_public_legal_person = parent
        .map(|parent| {
            code::institution_code_from_str(parent.institution_code.as_str())
                .map_or(false, |c| code::is_public_legal_code(&c))
        })
        .unwrap_or(false);
    if is_unincorporated && parent_is_public_legal_person {
        return local_public_scope(province_code, city_code);
    }

    // 私法人、私法人学校、挂靠私法人的非法人学校/机构都允许全国正常公民担任法定代表人。
    LegalRepresentativeCitizenScope::Nationwide
}

pub fn resolve_legal_representative_scope_for_institution(
    inst: &Institution,
    parent: Option<&Institution>,
) -> LegalRepresentativeCitizenScope {
    resolve_legal_representative_scope_for_codes(
        inst.institution_code.as_str(),
        inst.education_type.as_deref(),
        inst.province_code.as_str(),
        inst.city_code.as_str(),
        parent,
    )
}

pub fn is_default_account_name(account_name: &str) -> bool {
    DEFAULT_ACCOUNT_NAMES
        .iter()
        .any(|name| *name == account_name)
}

pub fn can_delete_account(account: &InstitutionAccount) -> bool {
    !is_default_account_name(&account.account_name)
        && matches!(
            account.chain_status,
            MultisigChainStatus::NotOnChain | MultisigChainStatus::RevokedOnChain
        )
}

pub fn default_account_names_for_codes(institution_code: &str) -> &'static [&'static str] {
    match institution_code {
        "NRC" => NATIONAL_RESERVE_DEFAULT_ACCOUNT_NAMES,
        "PRB" => PROVINCE_RESERVE_BANK_DEFAULT_ACCOUNT_NAMES,
        _ => COMMON_DEFAULT_ACCOUNT_NAMES,
    }
}

pub fn default_account_names_for_institution(inst: &Institution) -> &'static [&'static str] {
    default_account_names_for_codes(inst.institution_code.as_str())
}

/// 机构 / 账户 service 层错误。
#[derive(Debug, Clone)]
pub enum ServiceError {
    BadInput(&'static str),
    NotFound(&'static str),
    Conflict(&'static str),
}

impl ServiceError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::BadInput(m) | Self::NotFound(m) | Self::Conflict(m) => m,
        }
    }
}

/// 校验机构全称格式。
pub fn validate_cid_full_name(cid_full_name: &str) -> Result<String, ServiceError> {
    let trimmed = cid_full_name.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::BadInput("cid_full_name is required"));
    }
    if trimmed.chars().count() > MAX_INSTITUTION_NAME_CHARS {
        return Err(ServiceError::BadInput(
            "cid_full_name too long (max 30 chars)",
        ));
    }
    if trimmed.len() > MAX_INSTITUTION_NAME_BYTES {
        return Err(ServiceError::BadInput(
            "cid_full_name too long (max 128 bytes)",
        ));
    }
    Ok(trimmed.to_string())
}

pub fn validate_legal_representative_required(
    name: Option<&str>,
    cid_number: Option<&str>,
    photo_path: Option<&str>,
    photo_name: Option<&str>,
    photo_mime: Option<&str>,
    photo_size: Option<u64>,
) -> Result<LegalRepresentativeFields, ServiceError> {
    let legal_rep_name = name
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(ServiceError::BadInput("法定代表人姓名不能为空"))?;
    if legal_rep_name.chars().count() > MAX_LEGAL_REP_NAME_CHARS
        || legal_rep_name.len() > MAX_LEGAL_REP_NAME_BYTES
    {
        return Err(ServiceError::BadInput("法定代表人姓名过长"));
    }
    let cid_number = cid_number
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(ServiceError::BadInput("法定代表人身份ID不能为空"))?;
    let cid_number = validate_cid_number_format(cid_number)
        .map_err(|_| ServiceError::BadInput("法定代表人身份ID格式错误"))?;
    let photo_path = photo_path
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(ServiceError::BadInput("法定代表人证件照不能为空"))?;
    if !photo_path.starts_with("data/legal-rep-photos/") {
        return Err(ServiceError::BadInput("法定代表人证件照路径非法"));
    }
    let photo_name = photo_name
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(ServiceError::BadInput("法定代表人证件照文件名不能为空"))?;
    let photo_mime = photo_mime
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(ServiceError::BadInput("法定代表人证件照类型不能为空"))?;
    if !matches!(photo_mime, "image/jpeg" | "image/png" | "image/webp") {
        return Err(ServiceError::BadInput(
            "法定代表人证件照只支持 JPEG/PNG/WebP",
        ));
    }
    let photo_size = photo_size
        .filter(|v| *v > 0 && *v <= MAX_LEGAL_REP_PHOTO_BYTES)
        .ok_or(ServiceError::BadInput("法定代表人证件照大小非法"))?;
    Ok(LegalRepresentativeFields {
        legal_rep_name: legal_rep_name.to_string(),
        cid_number,
        photo_path: photo_path.to_string(),
        photo_name: photo_name.to_string(),
        photo_mime: photo_mime.to_string(),
        photo_size,
    })
}

/// 校验账户名称格式。
pub fn validate_account_name(name: &str) -> Result<String, ServiceError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::BadInput("account_name is required"));
    }
    if trimmed.chars().count() > MAX_ACCOUNT_NAME_CHARS {
        return Err(ServiceError::BadInput(
            "account_name too long (max 30 chars)",
        ));
    }
    if trimmed.len() > MAX_ACCOUNT_NAME_BYTES {
        return Err(ServiceError::BadInput(
            "account_name too long (max 128 bytes)",
        ));
    }
    // 制度专属保留名(永久质押/安全基金/两和基金)禁止注册为自定义账户名,
    // 与链端 primitives::account_derive::is_forbidden_account_name 单一权威源一致。
    if is_forbidden_account_name(trimmed.as_bytes()) {
        return Err(ServiceError::BadInput(
            "account_name 命中制度专属保留名(永久质押/安全基金/两和基金)",
        ));
    }
    Ok(trimmed.to_string())
}

/// 判定机构分类。机构码 + cid_full_name → InstitutionCategory。
///
/// 主体属性由机构码派生(K1 已从号码删除)。机构码不识别或不属于任何机构分类(个人/个人多签)
/// 返回 None,调用方应当直接拒绝请求。
pub fn derive_category(institution_code: &str, cid_full_name: &str) -> Option<InstitutionCategory> {
    let institution_code = code::institution_code_from_str(institution_code)?;
    classify(institution_code, cid_full_name)
}

/// 按机构类型构造默认未上链账户。
///
/// 中文注释:默认账户是机构主体的公共能力,由调用方写入结构化 `accounts` 表。
pub fn build_default_accounts_for_names(
    cid_number: &str,
    actor: &str,
    names: &[&str],
) -> Vec<InstitutionAccount> {
    use crate::accounts::derive::derive_account;
    use chrono::Utc;

    let now = Utc::now();
    names
        .iter()
        .map(|name| InstitutionAccount {
            cid_number: cid_number.to_string(),
            account_name: (*name).to_string(),
            account: derive_account(cid_number, name),
            chain_status: MultisigChainStatus::NotOnChain,
            chain_synced_at: None,
            chain_tx_hash: None,
            chain_block_number: None,
            created_by: actor.to_string(),
            created_at: now,
        })
        .collect()
}

pub fn build_default_accounts_for_codes(
    cid_number: &str,
    actor: &str,
    institution_code: &str,
) -> Vec<InstitutionAccount> {
    build_default_accounts_for_names(
        cid_number,
        actor,
        default_account_names_for_codes(institution_code),
    )
}

pub fn build_default_accounts_for_institution(
    inst: &Institution,
    actor: &str,
) -> Vec<InstitutionAccount> {
    build_default_accounts_for_names(
        inst.cid_number.as_str(),
        actor,
        default_account_names_for_institution(inst),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_account_name_basic() {
        assert!(validate_account_name("").is_err());
        assert!(validate_account_name("   ").is_err());
        assert!(validate_account_name("办案账户").is_ok());
        let too_long = "x".repeat(31);
        assert!(validate_account_name(&too_long).is_err());
    }

    #[test]
    fn derive_category_rules() {
        // 主体属性由机构码派生;市公安局(CPOL)已折叠为普通公权机构。
        assert_eq!(
            derive_category("CPOL", "广州市公民安全局"),
            Some(InstitutionCategory::GovInstitution)
        );
        assert_eq!(
            derive_category("CGOV", "别的机构"),
            Some(InstitutionCategory::GovInstitution)
        );
        assert_eq!(
            derive_category("SFGQ", "某公司"),
            Some(InstitutionCategory::PrivateInstitution)
        );
        // 个人主体(公民人)不是注册型机构 → None。
        assert_eq!(derive_category("CTZN", "xxx"), None);
        // 无效机构码 → None。
        assert_eq!(derive_category("XYZ", "xxx"), None);
    }
}
