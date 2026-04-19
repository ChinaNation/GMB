//! 机构/账户业务校验 + 分类 + 唯一性
//!
//! 中文注释:凡是要在 handler 里做的业务级校验(不是简单的格式校验),都放这里。
//! handler 只负责调用 service + 转 HTTP 响应。

#![allow(dead_code)]

use chrono::Utc;

use crate::institutions::model::{MultisigAccount, MultisigInstitution};
use crate::institutions::store;
use crate::models::Store;
use crate::sfid::{
    classify, generate_sfid_code, province::PROVINCES, A3, GenerateSfidInput,
    InstitutionCategory, InstitutionCode, PUBLIC_SECURITY_INSTITUTION_NAME,
};

/// 清算行设置启用时自动创建的 2 个默认账户 name(链上 extrinsic 的 name 参数)。
/// 链上地址 = blake2_256("DUOQIAN_SFID_V1" + SS58_PREFIX_LE + sfid_id + name)。
pub const CLEARING_BANK_NAMES: &[&str] = &["主账户", "费用账户"];

pub const MAX_ACCOUNT_NAME_CHARS: usize = 30;
pub const MAX_ACCOUNT_NAME_BYTES: usize = 128;
pub const MAX_INSTITUTION_NAME_CHARS: usize = 30;
pub const MAX_INSTITUTION_NAME_BYTES: usize = 128;

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
/// 可选 `exclude_sfid_id` 用于更新自身时排除自己。
pub fn institution_name_exists(store: &Store, name: &str) -> bool {
    institution_name_exists_excluding(store, name, None)
}

pub fn institution_name_exists_excluding(
    store: &Store,
    name: &str,
    exclude_sfid_id: Option<&str>,
) -> bool {
    store
        .multisig_institutions
        .values()
        .any(|i| {
            i.institution_name.as_deref() == Some(name)
                && exclude_sfid_id.map_or(true, |ex| i.sfid_id != ex)
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
            let st = trimmed.ok_or(ServiceError::BadInput(
                "私法人(SFR)必须选择企业类型",
            ))?;
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
                return Err(ServiceError::BadInput(
                    "仅私法人(SFR)才允许设置企业类型",
                ));
            }
            Ok(None)
        }
    }
}

/// 校验机构主键 sfid_id 未被占用。
pub fn ensure_institution_not_exists(store: &Store, sfid_id: &str) -> Result<(), ServiceError> {
    if store::contains_institution(store, sfid_id) {
        return Err(ServiceError::Conflict("institution sfid_id already exists"));
    }
    Ok(())
}

/// 校验机构存在。
pub fn ensure_institution_exists(store: &Store, sfid_id: &str) -> Result<(), ServiceError> {
    if !store::contains_institution(store, sfid_id) {
        return Err(ServiceError::NotFound("institution not found"));
    }
    Ok(())
}

/// 校验同 sfid_id 下账户名未被占用。
/// 这是**进链前**的硬校验,避免白交链上手续费。
pub fn ensure_account_name_unique(
    store: &Store,
    sfid_id: &str,
    account_name: &str,
) -> Result<(), ServiceError> {
    if store::contains_account(store, sfid_id, account_name) {
        return Err(ServiceError::Conflict(
            "account_name already exists under this institution",
        ));
    }
    Ok(())
}

// ─── 任务卡 6:公安局 ↔ sfid 工具市清单对账 ─────────────────────

/// 对账结果统计(用于日志和 HTTP 响应)。
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ReconcileReport {
    pub province: String,
    pub inserted: usize,
    pub updated: usize,
    pub removed: usize,
    pub total_after: usize,
}

/// 按 city_code 对账指定省的公安局机构。
///
/// 规则:
/// - 增:sfid 工具里有但 multisig_institutions 没有 → 生成新机构
/// - 删:multisig_institutions 有但 sfid 工具没有的 city_code → 删除机构
///      (不触链,multisig_accounts 孤儿记录保留)
/// - 改:city_code 相同但 city 字符串不同 → 更新 city + institution_name
///      sfid_id 不变
///
/// 特殊处理:
/// - 跳过 city_code == "000" 的保留占位
/// - 跳过 legacy 已有但 city_code 为空的记录(会被 backfill 函数修复)
pub fn reconcile_public_security_for_province(
    store: &mut Store,
    province_name: &str,
    actor: &str,
) -> ReconcileReport {
    let mut report = ReconcileReport {
        province: province_name.to_string(),
        ..Default::default()
    };

    // 1. 拿 sfid 工具权威市清单(带 code)
    let province_entry = match PROVINCES.iter().find(|p| p.name == province_name) {
        Some(p) => p,
        None => return report, // 未知省份,跳过
    };
    let authoritative_cities: Vec<(String, String)> = province_entry
        .cities
        .iter()
        .filter(|c| c.code != "000")
        .map(|c| (c.name.to_string(), c.code.to_string()))
        .collect();

    // 2. 现有该省的公安局机构索引:city_code → sfid_id
    let existing_by_code: std::collections::HashMap<String, String> = store
        .multisig_institutions
        .values()
        .filter(|i| {
            matches!(i.category, InstitutionCategory::PublicSecurity)
                && i.province == province_name
                && !i.city_code.is_empty()
        })
        .map(|i| (i.city_code.clone(), i.sfid_id.clone()))
        .collect();

    // 3. 增 + 改
    for (city_name, city_code) in &authoritative_cities {
        if let Some(sfid_id) = existing_by_code.get(city_code) {
            // 存在 → 检查市名是否需要更新
            if let Some(inst) = store.multisig_institutions.get_mut(sfid_id) {
                let new_name = format!("{}公安局", city_name);
                if inst.city != *city_name
                    || inst.institution_name.as_deref() != Some(new_name.as_str())
                {
                    inst.city = city_name.clone();
                    inst.institution_name = Some(new_name);
                    report.updated += 1;
                }
            }
        } else {
            // 缺失 → 生成新机构
            let account_placeholder = format!("PS-{}-{}", province_entry.code, city_code);
            let sfid_id = match generate_sfid_code(GenerateSfidInput {
                account_pubkey: account_placeholder.as_str(),
                a3: "GFR",
                p1: "0",
                province: province_name,
                city: city_name.as_str(),
                institution: "ZF",
            }) {
                Ok(v) => v,
                Err(err) => {
                    tracing::warn!(
                        province = %province_name,
                        city = %city_name,
                        error = %err,
                        "reconcile: failed to generate sfid for public security"
                    );
                    continue;
                }
            };
            if store.multisig_institutions.contains_key(&sfid_id) {
                continue; // 极小概率碰撞
            }
            let inst = MultisigInstitution {
                sfid_id: sfid_id.clone(),
                institution_name: Some(format!("{}公安局", city_name)),
                category: InstitutionCategory::PublicSecurity,
                a3: "GFR".to_string(),
                p1: "0".to_string(),
                province: province_name.to_string(),
                city: city_name.clone(),
                province_code: province_entry.code.to_string(),
                city_code: city_code.clone(),
                institution_code: "ZF".to_string(),
                sub_type: None,
                parent_sfid_id: None,
                is_clearing_bank: false,
                sfid_finalized: false,
                created_by: actor.to_string(),
                created_at: Utc::now(),
            };
            store.multisig_institutions.insert(sfid_id, inst);
            report.inserted += 1;
        }
    }

    // 4. 删:当前 city_code 不在权威清单里的
    let authoritative_codes: std::collections::HashSet<String> = authoritative_cities
        .iter()
        .map(|(_, c)| c.clone())
        .collect();
    let to_remove: Vec<String> = store
        .multisig_institutions
        .values()
        .filter(|i| {
            matches!(i.category, InstitutionCategory::PublicSecurity)
                && i.province == province_name
                && !i.city_code.is_empty()
                && !authoritative_codes.contains(&i.city_code)
        })
        .map(|i| i.sfid_id.clone())
        .collect();
    for sfid_id in to_remove {
        store.multisig_institutions.remove(&sfid_id);
        report.removed += 1;
    }

    // 5. 统计该省对账后公安局总数
    report.total_after = store
        .multisig_institutions
        .values()
        .filter(|i| {
            matches!(i.category, InstitutionCategory::PublicSecurity)
                && i.province == province_name
        })
        .count();

    report
}

/// 老记录 backfill:扫 multisig_institutions,给 city_code 为空的公安局机构
/// 用 (province, city) 反查权威清单补上。必须在 reconcile 之前跑一次,
/// 否则老记录会被 reconcile 误删。
pub fn backfill_public_security_city_codes(store: &mut Store) -> usize {
    let mut fixed = 0usize;
    let targets: Vec<(String, String, String)> = store
        .multisig_institutions
        .values()
        .filter(|i| {
            matches!(i.category, InstitutionCategory::PublicSecurity) && i.city_code.is_empty()
        })
        .map(|i| (i.sfid_id.clone(), i.province.clone(), i.city.clone()))
        .collect();
    for (sfid_id, province, city) in targets {
        let Some(entry) = PROVINCES.iter().find(|p| p.name == province) else {
            continue;
        };
        let Some(cc) = entry.cities.iter().find(|c| c.name == city).map(|c| c.code) else {
            continue;
        };
        if let Some(inst) = store.multisig_institutions.get_mut(&sfid_id) {
            inst.city_code = cc.to_string();
            fixed += 1;
        }
    }
    fixed
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
