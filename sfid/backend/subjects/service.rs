//! 机构/账户业务校验 + 分类 + 唯一性
//!
//! 中文注释:凡是要在 handler 里做的业务级校验(不是简单的格式校验),都放这里。
//! handler 只负责调用 service + 转 HTTP 响应。

#![allow(dead_code)]

use chrono::Utc;

use crate::models::Store;
use crate::sfid_number::{classify, InstitutionCategory, InstitutionCode, A3};
use crate::subjects::model::MultisigAccount;
use crate::subjects::store;
use crate::subjects::MultisigChainStatus;

pub const DEFAULT_ACCOUNT_NAMES: &[&str] = &["主账户", "费用账户"];

pub const MAX_ACCOUNT_NAME_CHARS: usize = 30;
pub const MAX_ACCOUNT_NAME_BYTES: usize = 128;
pub const MAX_INSTITUTION_NAME_CHARS: usize = 30;
pub const MAX_INSTITUTION_NAME_BYTES: usize = 128;

pub fn is_default_account_name(account_name: &str) -> bool {
    DEFAULT_ACCOUNT_NAMES
        .iter()
        .any(|name| *name == account_name)
}

pub fn can_delete_account(account: &MultisigAccount) -> bool {
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
    Ok(trimmed.to_string())
}

/// 判定机构分类。a3 + institution_code + institution_name → InstitutionCategory。
///
/// 解析失败(a3 或 institution_code 不识别)或不属于任何机构分类(公民类)返回 None,
/// 调用方应当直接拒绝请求。
pub fn derive_category(
    a3: &str,
    institution_code: &str,
    institution_name: &str,
) -> Option<InstitutionCategory> {
    let a3 = A3::from_str(a3)?;
    let code = InstitutionCode::from_str(institution_code)?;
    classify(a3, code, institution_name)
}

/// 检查机构名称是否已被全国任意机构占用(私权机构使用)。
/// 两步式改造:institution_name 为 `Option<String>`,未命名机构视为不占名。
/// 可选 `exclude_sfid_number` 用于更新自身时排除自己。
pub fn institution_name_exists(store: &Store, name: &str) -> bool {
    institution_name_exists_excluding(store, name, None)
}

pub fn institution_name_exists_excluding(
    store: &Store,
    name: &str,
    exclude_sfid_number: Option<&str>,
) -> bool {
    store.multisig_institutions.values().any(|i| {
        i.institution_name.as_deref() == Some(name)
            && exclude_sfid_number.map_or(true, |ex| i.sfid_number != ex)
    })
}

/// 检查同城是否存在同名机构(公权机构使用:不同市允许重名)。
pub fn institution_name_exists_in_city(store: &Store, name: &str, city: &str) -> bool {
    store
        .multisig_institutions
        .values()
        .any(|i| i.institution_name.as_deref() == Some(name) && i.city == city)
}

// ─── 两步式第二步:sub_type 与 P1 联动校验 ──────────────────────
//
// 联动规则:
//   P1 = "0" (非盈利) → sub_type 必须为 "NON_PROFIT"
//   P1 = "1" (盈利)   → sub_type 必须为 SOLE_PROPRIETORSHIP / PARTNERSHIP /
//                       LIMITED_LIABILITY / JOINT_STOCK 四选一
//
// 仅 SFR(私法人)需要 sub_type;FFR(非法人)一律不得传,传了报错。

/// 允许的 SFR sub_type 全集。
pub const VALID_SUB_TYPES: &[&str] = &[
    "SOLE_PROPRIETORSHIP",
    "PARTNERSHIP",
    "LIMITED_LIABILITY",
    "JOINT_STOCK",
    "NON_PROFIT",
];

/// 校验 sub_type 与 (a3, p1) 组合是否合法。
///
/// - `a3 == "SFR"`:必须提供 sub_type,且与 p1 联动正确
/// - `a3 == "FFR"`:不得提供 sub_type(传了则返回错误)
/// - 其他 a3(含 GFR):不得提供 sub_type
pub fn validate_sub_type_with_p1(
    a3: &str,
    p1: &str,
    sub_type: Option<&str>,
) -> Result<Option<String>, ServiceError> {
    let trimmed = sub_type.map(str::trim).filter(|s| !s.is_empty());
    match a3 {
        "SFR" => {
            let st = trimmed.ok_or(ServiceError::BadInput("私法人(SFR)必须选择企业类型"))?;
            if !VALID_SUB_TYPES.contains(&st) {
                return Err(ServiceError::BadInput(
                    "企业类型非法(仅 SOLE_PROPRIETORSHIP/PARTNERSHIP/LIMITED_LIABILITY/JOINT_STOCK/NON_PROFIT)",
                ));
            }
            match p1 {
                "0" => {
                    if st != "NON_PROFIT" {
                        return Err(ServiceError::BadInput(
                            "P1=非盈利 时企业类型必须为 NON_PROFIT",
                        ));
                    }
                }
                "1" => {
                    if st == "NON_PROFIT" {
                        return Err(ServiceError::BadInput(
                            "P1=盈利 时企业类型不得为 NON_PROFIT",
                        ));
                    }
                }
                _ => return Err(ServiceError::BadInput("P1 非法(仅 0/1)")),
            }
            Ok(Some(st.to_string()))
        }
        _ => {
            if trimmed.is_some() {
                return Err(ServiceError::BadInput("仅私法人(SFR)才允许设置企业类型"));
            }
            Ok(None)
        }
    }
}

/// 校验机构主键 sfid_number 未被占用。
pub fn ensure_institution_not_exists(store: &Store, sfid_number: &str) -> Result<(), ServiceError> {
    if store::contains_institution(store, sfid_number) {
        return Err(ServiceError::Conflict(
            "institution sfid_number already exists",
        ));
    }
    Ok(())
}

/// 校验机构存在。
pub fn ensure_institution_exists(store: &Store, sfid_number: &str) -> Result<(), ServiceError> {
    if !store::contains_institution(store, sfid_number) {
        return Err(ServiceError::NotFound("institution not found"));
    }
    Ok(())
}

/// 校验同 sfid_number 下账户名未被占用。
/// 这是**进链前**的硬校验,避免白交链上手续费。
pub fn ensure_account_name_unique(
    store: &Store,
    sfid_number: &str,
    account_name: &str,
) -> Result<(), ServiceError> {
    if store::contains_account(store, sfid_number, account_name) {
        return Err(ServiceError::Conflict(
            "account_name already exists under this institution",
        ));
    }
    Ok(())
}

/// 给指定机构写入 2 条默认未上链账户(模块 Store 快照)。
///
/// 中文注释:默认账户是机构主体的公共能力,不属于公权或私权单独目录。
/// 幂等:已存在账户不覆盖;仅在该 `(sfid_number, account_name)` 缺失时补齐。
pub fn insert_default_accounts_into_global_store(
    store: &mut Store,
    sfid_number: &str,
    actor: &str,
) {
    use crate::accounts::derive::derive_duoqian_address;
    use crate::subjects::model::{account_key_to_string, MultisigAccount};

    let now = Utc::now();
    for name in DEFAULT_ACCOUNT_NAMES {
        let key = account_key_to_string(sfid_number, name);
        // DUOQIAN_V1 本地派生:主账户和费用账户地址由 sfid_number 稳定决定。
        let addr = derive_duoqian_address(sfid_number, name);
        store
            .multisig_accounts
            .entry(key)
            .or_insert_with(|| MultisigAccount {
                sfid_number: sfid_number.to_string(),
                account_name: (*name).to_string(),
                duoqian_address: addr,
                chain_status: MultisigChainStatus::NotOnChain,
                chain_synced_at: None,
                chain_tx_hash: None,
                chain_block_number: None,
                created_by: actor.to_string(),
                created_at: now,
            });
    }
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
            derive_category("GFR", "ZF", "公民安全局"),
            Some(InstitutionCategory::PublicSecurity)
        );
        assert_eq!(
            derive_category("GFR", "ZF", "别的机构"),
            Some(InstitutionCategory::GovInstitution)
        );
        assert_eq!(
            derive_category("SFR", "ZG", "某公司"),
            Some(InstitutionCategory::PrivateInstitution)
        );
        assert_eq!(derive_category("GMR", "ZG", "xxx"), None);
        assert_eq!(derive_category("INVALID", "ZG", "xxx"), None);
    }
}
