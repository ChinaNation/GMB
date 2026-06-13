//! 机构/账户业务校验 + 分类 + 唯一性
//!
//! 中文注释:凡是要在 handler 里做的业务级校验(不是简单的格式校验),都放这里。
//! handler 只负责调用 service + 转 HTTP 响应。

#![allow(dead_code)]

use crate::number::{
    classify, validate_sfid_number_format, InstitutionCategory, InstitutionCode, SubjectProperty,
};
use crate::subjects::model::InstitutionAccount;
use crate::subjects::MultisigChainStatus;
use primitives::core_const::is_forbidden_account_name;

pub const DEFAULT_ACCOUNT_NAMES: &[&str] = &["主账户", "费用账户"];

pub const MAX_ACCOUNT_NAME_CHARS: usize = 30;
pub const MAX_ACCOUNT_NAME_BYTES: usize = 128;
pub const MAX_INSTITUTION_NAME_CHARS: usize = 30;
pub const MAX_INSTITUTION_NAME_BYTES: usize = 128;
pub const MAX_LEGAL_REP_NAME_CHARS: usize = 30;
pub const MAX_LEGAL_REP_NAME_BYTES: usize = 128;
pub const MAX_LEGAL_REP_PHOTO_BYTES: u64 = 5 * 1024 * 1024;

pub struct LegalRepresentativeFields {
    pub name: String,
    pub sfid_number: String,
    pub photo_path: String,
    pub photo_name: String,
    pub photo_mime: String,
    pub photo_size: u64,
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

/// 校验机构名称格式。
pub fn validate_institution_name(name: &str) -> Result<String, ServiceError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::BadInput("institution_name is required"));
    }
    if trimmed.chars().count() > MAX_INSTITUTION_NAME_CHARS {
        return Err(ServiceError::BadInput(
            "institution_name too long (max 30 chars)",
        ));
    }
    if trimmed.len() > MAX_INSTITUTION_NAME_BYTES {
        return Err(ServiceError::BadInput(
            "institution_name too long (max 128 bytes)",
        ));
    }
    Ok(trimmed.to_string())
}

pub fn validate_legal_representative_required(
    name: Option<&str>,
    sfid_number: Option<&str>,
    photo_path: Option<&str>,
    photo_name: Option<&str>,
    photo_mime: Option<&str>,
    photo_size: Option<u64>,
) -> Result<LegalRepresentativeFields, ServiceError> {
    let name = name
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(ServiceError::BadInput("法定代表人姓名不能为空"))?;
    if name.chars().count() > MAX_LEGAL_REP_NAME_CHARS || name.len() > MAX_LEGAL_REP_NAME_BYTES {
        return Err(ServiceError::BadInput("法定代表人姓名过长"));
    }
    let sfid_number = sfid_number
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(ServiceError::BadInput("法定代表人身份ID不能为空"))?;
    let sfid_number = validate_sfid_number_format(sfid_number)
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
        name: name.to_string(),
        sfid_number,
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
    // 与链端 primitives::core_const::is_forbidden_account_name 单一权威源一致。
    if is_forbidden_account_name(trimmed.as_bytes()) {
        return Err(ServiceError::BadInput(
            "account_name 命中制度专属保留名(永久质押/安全基金/两和基金)",
        ));
    }
    Ok(trimmed.to_string())
}

/// 判定机构分类。subject_property + institution_code + institution_name → InstitutionCategory。
///
/// 解析失败(subject_property 或 institution_code 不识别)或不属于任何机构分类(公民类)返回 None,
/// 调用方应当直接拒绝请求。
pub fn derive_category(
    subject_property: &str,
    institution_code: &str,
    institution_name: &str,
) -> Option<InstitutionCategory> {
    let subject_property = SubjectProperty::from_str(subject_property)?;
    let code = InstitutionCode::from_str(institution_code)?;
    classify(subject_property, code, institution_name)
}

/// 构造指定机构的 2 条默认未上链账户。
///
/// 中文注释:默认账户是机构主体的公共能力,由调用方写入结构化 `accounts` 表。
pub fn build_default_accounts(sfid_number: &str, actor: &str) -> Vec<InstitutionAccount> {
    use crate::accounts::derive::derive_duoqian_address;
    use chrono::Utc;

    let now = Utc::now();
    DEFAULT_ACCOUNT_NAMES
        .iter()
        .map(|name| InstitutionAccount {
            sfid_number: sfid_number.to_string(),
            account_name: (*name).to_string(),
            duoqian_address: derive_duoqian_address(sfid_number, name),
            chain_status: MultisigChainStatus::NotOnChain,
            chain_synced_at: None,
            chain_tx_hash: None,
            chain_block_number: None,
            created_by: actor.to_string(),
            created_at: now,
        })
        .collect()
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
        assert_eq!(
            derive_category("G", "ZF", "广州市公安局"),
            Some(InstitutionCategory::PublicSecurity)
        );
        assert_eq!(
            derive_category("G", "ZF", "别的机构"),
            Some(InstitutionCategory::GovInstitution)
        );
        assert_eq!(
            derive_category("S", "GQ", "某公司"),
            Some(InstitutionCategory::PrivateInstitution)
        );
        assert_eq!(derive_category("M", "ZG", "xxx"), None);
        assert_eq!(derive_category("INVALID", "ZG", "xxx"), None);
    }
}
