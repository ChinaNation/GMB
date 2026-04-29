//! 机构/账户业务校验 + 分类 + 唯一性
//!
//! 中文注释:凡是要在 handler 里做的业务级校验(不是简单的格式校验),都放这里。
//! handler 只负责调用 service + 转 HTTP 响应。

#![allow(dead_code)]

use chrono::Utc;

use crate::institutions::model::{MultisigAccount, MultisigInstitution};
use crate::institutions::store;
use crate::models::{InstitutionChainStatus, MultisigChainStatus, Store};
use crate::sfid::{
    classify, generate_sfid_code, province::PROVINCES, GenerateSfidInput, InstitutionCategory,
    InstitutionCode, A3, PUBLIC_SECURITY_INSTITUTION_NAME,
};

/// 所有机构创建时自动生成的 2 个默认账户 `account_name`。
///
/// (2026-04-21 统一两步模式) sfid 后端侧只认 `account_name` 字符串,
/// 链端按 `account_name` 值翻译到对应 `InstitutionAccountRole`:
/// - `"主账户"`  → `Role::Main`,preimage 不含 name
/// - `"费用账户"` → `Role::Fee`, preimage 不含 name
/// - 其他任意非空字符串 → `Role::Named(name)`,preimage 含 name
///
/// 派生细节完全是链端内部关注点(见 `citizenchain/runtime/transaction/
/// duoqian-manage-pow/src/lib.rs` 的 `InstitutionAccountRole` /
/// `derive_institution_address`)。sfid 后端只负责把字符串原样传过去。
///
/// **两个保留名**:`delete_account` 对这两个 `account_name` 直接 409 拒绝,
/// 保证每家机构至少始终挂着这两个默认账户记录。
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
/// 可选 `exclude_sfid_id` 用于更新自身时排除自己。
pub fn institution_name_exists(store: &Store, name: &str) -> bool {
    institution_name_exists_excluding(store, name, None)
}

pub fn institution_name_exists_excluding(
    store: &Store,
    name: &str,
    exclude_sfid_id: Option<&str>,
) -> bool {
    store.multisig_institutions.values().any(|i| {
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

// ─── 清算行资格白名单(2026-04-24, ADR-007) ─────────────────────
//
// 仅"私法人股份公司"和"从属于私法人股份公司的非法人"有资格成为清算行。
// 详见 memory/04-decisions/ADR-007-clearing-bank-three-phase.md
// 与 memory/05-modules/sfid/clearing-bank-eligibility.md。
//
// 规则:
//   SFR + sub_type=JOINT_STOCK            → ✅
//   FFR + parent.SFR + parent.JOINT_STOCK → ✅
//   其他                                   → ❌

/// 清算行资格白名单判定:仅允许"私法人股份公司"及其下属非法人。
///
/// - `inst.a3 == "SFR"`:必须 `sub_type == "JOINT_STOCK"`
/// - `inst.a3 == "FFR"`:`parent` 必须存在,`parent.a3 == "SFR"` 且 `parent.sub_type == "JOINT_STOCK"`
/// - 其他 a3(GFR / SF 等):一律不允许
///
/// `parent` 由调用方按需提供(FFR 才需要;SFR / 其他可传 `None`)。
/// 跨省 parent 查询由 caller 通过 sharded_store.read_province 完成,
/// 本函数只做纯逻辑判定,便于单测。
pub fn is_clearing_bank_eligible(
    inst: &MultisigInstitution,
    parent: Option<&MultisigInstitution>,
) -> bool {
    match inst.a3.as_str() {
        "SFR" => inst.sub_type.as_deref() == Some("JOINT_STOCK"),
        "FFR" => match parent {
            Some(p) => p.a3 == "SFR" && p.sub_type.as_deref() == Some("JOINT_STOCK"),
            None => false,
        },
        _ => false,
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
///      (不触链,账户记录由后续清理任务统一处理)
/// - 改:city_code 相同但 city 字符串不同 → 更新 city + institution_name
///      sfid_id 不变
///
/// 特殊处理:
/// - 跳过 city_code == "000" 的保留占位
/// - 跳过全局 store 已有但 city_code 为空的记录(会被 backfill 函数修复)
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
                sfid_finalized: false,
                chain_status: InstitutionChainStatus::NotRegistered,
                chain_tx_hash: None,
                chain_block_number: None,
                chain_synced_at: None,
                created_by: actor.to_string(),
                created_at: Utc::now(),
            };
            store.multisig_institutions.insert(sfid_id.clone(), inst);
            // 公安局 reconcile 同步插入 2 条默认未上链账户。
            insert_default_accounts_into_global_store(store, &sfid_id, actor);
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
            matches!(i.category, InstitutionCategory::PublicSecurity) && i.province == province_name
        })
        .count();

    report
}

/// 给指定机构写入 2 条默认未上链账户(全局 store)。
///
/// 幂等:已存在账户不覆盖;仅在该 `(sfid_id, account_name)` 缺失时补齐。
/// reconcile 本身持全局 store 写锁;sharded_store 的同步由启动后的分片同步流程补齐。
pub fn insert_default_accounts_into_global_store(store: &mut Store, sfid_id: &str, actor: &str) {
    use crate::institutions::derive::derive_duoqian_address;
    use crate::institutions::model::{account_key_to_string, MultisigAccount};
    let now = Utc::now();
    for name in DEFAULT_ACCOUNT_NAMES {
        let key = account_key_to_string(sfid_id, name);
        // DUOQIAN_V1 本地派生(主账户→0x00 / 费用账户→0x01);公安局 SFID 固定,
        // 账户地址在 reconcile 时即可完全确定,无需等激活上链。
        let addr = derive_duoqian_address(sfid_id, name);
        store
            .multisig_accounts
            .entry(key)
            .or_insert_with(|| MultisigAccount {
                sfid_id: sfid_id.to_string(),
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

    // ─── 清算行资格白名单(ADR-007)─────────────────────────────

    /// 测试 fixture:按所需字段构造一个最小机构样本。
    /// `a3`/`sub_type`/`parent_sfid_id` 是判定关键字段,其他用合理默认值。
    fn fixture_institution(
        a3: &str,
        sub_type: Option<&str>,
        parent_sfid_id: Option<&str>,
    ) -> MultisigInstitution {
        MultisigInstitution {
            sfid_id: format!("{a3}-GD-CB01-000000000-20260101"),
            institution_name: Some("测试机构".to_string()),
            category: InstitutionCategory::PrivateInstitution,
            a3: a3.to_string(),
            p1: if sub_type == Some("NON_PROFIT") {
                "0".to_string()
            } else {
                "1".to_string()
            },
            province: "广东省".to_string(),
            city: "广州市".to_string(),
            province_code: "GD".to_string(),
            city_code: "001".to_string(),
            institution_code: "CB".to_string(),
            sub_type: sub_type.map(|s| s.to_string()),
            parent_sfid_id: parent_sfid_id.map(|s| s.to_string()),
            sfid_finalized: true,
            chain_status: InstitutionChainStatus::NotRegistered,
            chain_tx_hash: None,
            chain_block_number: None,
            chain_synced_at: None,
            created_by: "test".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn clearing_bank_eligible_sfr_joint_stock() {
        // case 1: SFR + JOINT_STOCK → ✅
        let inst = fixture_institution("SFR", Some("JOINT_STOCK"), None);
        assert!(is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_sfr_limited_liability_rejected() {
        // case 2: SFR + LIMITED_LIABILITY → ❌
        let inst = fixture_institution("SFR", Some("LIMITED_LIABILITY"), None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_sfr_non_profit_rejected() {
        // case 3: SFR + NON_PROFIT → ❌
        let inst = fixture_institution("SFR", Some("NON_PROFIT"), None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_ffr_with_jointstock_parent() {
        // case 4: FFR + parent(SFR + JOINT_STOCK) → ✅
        let parent = fixture_institution("SFR", Some("JOINT_STOCK"), None);
        let inst = fixture_institution("FFR", None, Some(&parent.sfid_id));
        assert!(is_clearing_bank_eligible(&inst, Some(&parent)));
    }

    #[test]
    fn clearing_bank_eligible_ffr_with_non_jointstock_parent_rejected() {
        // case 5: FFR + parent(SFR + LIMITED_LIABILITY) → ❌
        let parent = fixture_institution("SFR", Some("LIMITED_LIABILITY"), None);
        let inst = fixture_institution("FFR", None, Some(&parent.sfid_id));
        assert!(!is_clearing_bank_eligible(&inst, Some(&parent)));
    }

    #[test]
    fn clearing_bank_eligible_ffr_without_parent_rejected() {
        // case 6: FFR + 缺 parent(查不到 / 未设置 parent_sfid_id) → ❌
        let inst = fixture_institution("FFR", None, None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_other_a3_rejected() {
        // GFR / SF 等其他 a3 一律 ❌
        let gfr = fixture_institution("GFR", None, None);
        assert!(!is_clearing_bank_eligible(&gfr, None));
        let sf = fixture_institution("SF", None, None);
        assert!(!is_clearing_bank_eligible(&sf, None));
    }

    #[test]
    fn clearing_bank_eligible_ffr_with_gfr_parent_rejected() {
        // FFR 即使 parent 是 GFR 也不允许(必须 SFR + JOINT_STOCK)
        let parent = fixture_institution("GFR", None, None);
        let inst = fixture_institution("FFR", None, Some(&parent.sfid_id));
        assert!(!is_clearing_bank_eligible(&inst, Some(&parent)));
    }
}
