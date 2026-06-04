//! 公权机构自动目录与公安局对账服务。
//!
//! 中文注释:自动生成的公权机构只归 gov 模块维护;subjects 只保留公共主体能力。

// 中文注释:这里只读取 citizenchain 常量中的机构名称和 sfid_number;
// 地址、管理员等链端字段由链端继续维护,在 SFID 自动目录中不展开使用。
#![allow(dead_code)]

use chrono::Utc;
use std::collections::HashSet;

use crate::china::provinces;
use crate::models::Store;
use crate::sfid_number::{generate_sfid_code, GenerateSfidInput, InstitutionCategory};
use crate::subjects::model::{InstitutionLevel, InstitutionSource, MultisigInstitution};
use crate::subjects::service::insert_default_accounts_into_global_store;
use crate::subjects::InstitutionChainStatus;

// 中文注释:宪法内置机构的 sfid_number 由 citizenchain 常量维护。
// SFID 这里只读取常量,避免在机构模块手写第二套国家/省级机构清单。
#[path = "../../../citizenchain/runtime/primitives/china/china_cb.rs"]
mod china_cb_constants;
#[path = "../../../citizenchain/runtime/primitives/china/china_ch.rs"]
mod china_ch_constants;
#[path = "../../../citizenchain/runtime/primitives/china/china_jc.rs"]
mod china_jc_constants;
#[path = "../../../citizenchain/runtime/primitives/china/china_jy.rs"]
mod china_jy_constants;
#[path = "../../../citizenchain/runtime/primitives/china/china_lf.rs"]
mod china_lf_constants;
#[path = "../../../citizenchain/runtime/primitives/china/china_sf.rs"]
mod china_sf_constants;
#[path = "../../../citizenchain/runtime/primitives/china/china_zf.rs"]
mod china_zf_constants;

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

/// 自动公权/宪法机构目录对账统计。
///
/// 中文注释:`touched_sfids` 用于启动流程把本次目标目录写入行表和分片缓存;
/// `removed_sfids` 用于清理已不属于目标目录的普通公权记录。
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct OfficialReconcileReport {
    pub inserted: usize,
    pub updated: usize,
    pub removed: usize,
    pub total_after: usize,
    pub touched_sfids: Vec<String>,
    pub removed_sfids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OfficialTargetOrigin {
    ConstitutionConstant,
    CityTemplate,
}

#[derive(Debug, Clone)]
struct OfficialInstitutionTarget {
    sfid_number: String,
    institution_name: String,
    category: InstitutionCategory,
    a3: String,
    p1: String,
    province: String,
    city: String,
    province_code: String,
    city_code: String,
    institution_code: String,
    institution_level: InstitutionLevel,
    origin: OfficialTargetOrigin,
    account_seed: String,
}

/// 按宪法常量 + SFID 行政区划生成自动机构目标目录。
///
/// 中文注释:
/// - 国家/省级机构直接读取 citizenchain 的 china_* 常量,复用其中不可变 `sfid_number`。
/// - 市级自治政府/立法会/司法院/监察院/教育委员会按 SFID 行政区划派生;
///   行政区划是唯一真源,市名变化时 reconcile 通过省市代码匹配现有记录并保持 sfid_number 不变。
/// - 手动注册的学校机构不进入这里;它们只是 `JY` 类型机构,不是学校内部组织。
fn official_institution_targets() -> Vec<OfficialInstitutionTarget> {
    let mut targets = Vec::new();

    for (idx, item) in china_zf_constants::CHINA_ZF.iter().enumerate() {
        let level = if idx < 11 {
            InstitutionLevel::National
        } else {
            InstitutionLevel::Province
        };
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number, level);
    }
    for (idx, item) in china_lf_constants::CHINA_LF.iter().enumerate() {
        let level = if idx == 0 {
            InstitutionLevel::National
        } else {
            InstitutionLevel::Province
        };
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number, level);
    }
    for (idx, item) in china_sf_constants::CHINA_SF.iter().enumerate() {
        let level = if idx == 0 {
            InstitutionLevel::National
        } else {
            InstitutionLevel::Province
        };
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number, level);
    }
    for (idx, item) in china_jc_constants::CHINA_JC.iter().enumerate() {
        let level = if idx < 4 {
            InstitutionLevel::National
        } else {
            InstitutionLevel::Province
        };
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number, level);
    }
    for item in china_jy_constants::CHINA_JY.iter() {
        // 中文注释:当前链端常量名仍是“公民教育委员会”,SFID 展示按目标制度名称显示为国家教育委员会。
        push_constant_target(
            &mut targets,
            "国家教育委员会",
            item.sfid_number,
            InstitutionLevel::National,
        );
    }
    for (idx, item) in china_cb_constants::CHINA_CB.iter().enumerate() {
        let level = if idx == 0 {
            InstitutionLevel::National
        } else {
            InstitutionLevel::Province
        };
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number, level);
    }
    for item in china_ch_constants::CHINA_CH.iter() {
        push_constant_target(
            &mut targets,
            item.sfid_name,
            item.sfid_number,
            InstitutionLevel::Province,
        );
    }

    for province in provinces().iter() {
        for city in province.cities.iter().filter(|city| city.code != "000") {
            push_city_template_target(
                &mut targets,
                province.name,
                province.code,
                city.name,
                city.code,
                "ZF",
                "自治政府",
            );
            push_city_template_target(
                &mut targets,
                province.name,
                province.code,
                city.name,
                city.code,
                "LF",
                "立法会",
            );
            push_city_template_target(
                &mut targets,
                province.name,
                province.code,
                city.name,
                city.code,
                "SF",
                "司法院",
            );
            push_city_template_target(
                &mut targets,
                province.name,
                province.code,
                city.name,
                city.code,
                "JC",
                "监察院",
            );
            push_city_template_target(
                &mut targets,
                province.name,
                province.code,
                city.name,
                city.code,
                "JY",
                "教育委员会",
            );
        }
    }

    targets
}

fn push_constant_target(
    targets: &mut Vec<OfficialInstitutionTarget>,
    name: &'static str,
    sfid_number: &'static str,
    level: InstitutionLevel,
) {
    let Some((a3, province_code, city_code, institution_code, p1)) =
        parse_sfid_institution_parts(sfid_number)
    else {
        tracing::warn!(
            sfid = sfid_number,
            "skip invalid constitution institution sfid"
        );
        return;
    };
    let Some((province, city)) = province_city_by_codes(&province_code, &city_code) else {
        tracing::warn!(
            sfid = sfid_number,
            province_code,
            city_code,
            "skip constitution institution with unknown province/city code"
        );
        return;
    };
    targets.push(OfficialInstitutionTarget {
        sfid_number: sfid_number.to_string(),
        institution_name: name.to_string(),
        category: category_for_auto_a3(&a3),
        a3,
        p1,
        province: province.to_string(),
        city: city.to_string(),
        province_code,
        city_code,
        institution_code,
        institution_level: level,
        origin: OfficialTargetOrigin::ConstitutionConstant,
        account_seed: sfid_number.to_string(),
    });
}

fn push_city_template_target(
    targets: &mut Vec<OfficialInstitutionTarget>,
    province_name: &'static str,
    province_code: &'static str,
    city_name: &'static str,
    city_code: &'static str,
    institution_code: &'static str,
    suffix: &'static str,
) {
    let institution_name = format!("{city_name}{suffix}");
    let account_seed = format!("AUTO-CITY-{province_code}-{city_code}-{institution_code}-{suffix}");
    let sfid_number = match generate_official_template_sfid(
        &account_seed,
        province_name,
        city_name,
        institution_code,
    ) {
        Some(v) => v,
        None => return,
    };
    targets.push(OfficialInstitutionTarget {
        sfid_number,
        institution_name,
        category: InstitutionCategory::GovInstitution,
        a3: "GFR".to_string(),
        p1: "0".to_string(),
        province: province_name.to_string(),
        city: city_name.to_string(),
        province_code: province_code.to_string(),
        city_code: city_code.to_string(),
        institution_code: institution_code.to_string(),
        institution_level: InstitutionLevel::City,
        origin: OfficialTargetOrigin::CityTemplate,
        account_seed,
    });
}

fn generate_official_template_sfid(
    account_seed: &str,
    province_name: &str,
    city_name: &str,
    institution_code: &str,
) -> Option<String> {
    match generate_sfid_code(GenerateSfidInput {
        account_pubkey: account_seed,
        a3: "GFR",
        p1: "0",
        province: province_name,
        city: city_name,
        institution: institution_code,
    }) {
        Ok(v) => Some(v),
        Err(err) => {
            tracing::warn!(
                province = %province_name,
                city = %city_name,
                institution = %institution_code,
                error = %err,
                "failed to generate official institution sfid"
            );
            None
        }
    }
}

fn parse_sfid_institution_parts(
    sfid_number: &str,
) -> Option<(String, String, String, String, String)> {
    let mut segments = sfid_number.split('-');
    let a3 = segments.next()?.to_string();
    let r5 = segments.next()?;
    let t2p1c1 = segments.next()?;
    if r5.len() != 5 || t2p1c1.len() < 3 {
        return None;
    }
    Some((
        a3,
        r5[0..2].to_string(),
        r5[2..5].to_string(),
        t2p1c1[0..2].to_string(),
        t2p1c1[2..3].to_string(),
    ))
}

fn province_city_by_codes(
    province_code: &str,
    city_code: &str,
) -> Option<(&'static str, &'static str)> {
    let province = provinces()
        .iter()
        .find(|p| p.code.eq_ignore_ascii_case(province_code))?;
    let city = province
        .cities
        .iter()
        .find(|c| c.code.eq_ignore_ascii_case(city_code))?;
    Some((province.name, city.name))
}

fn category_for_auto_a3(_a3: &str) -> InstitutionCategory {
    // 中文注释:宪法/制度目录按“公权机构”归类;A3 只保留 SFID 派生属性,
    // 例如省储备银行常量是 SFR,仍属于公民储备委员会目录。
    InstitutionCategory::GovInstitution
}

fn same_city_template_slot(inst: &MultisigInstitution, target: &OfficialInstitutionTarget) -> bool {
    target.origin == OfficialTargetOrigin::CityTemplate
        && matches!(inst.source, Some(InstitutionSource::Auto))
        && inst.category == target.category
        && inst.a3 == target.a3
        && inst.p1 == target.p1
        && inst.province_code == target.province_code
        && inst.city_code == target.city_code
        && inst.institution_code == target.institution_code
        && inst.institution_level == Some(target.institution_level.clone())
}

fn resolve_official_target_sfid(store: &Store, target: &OfficialInstitutionTarget) -> String {
    if target.origin == OfficialTargetOrigin::CityTemplate {
        if let Some((sfid, _)) = store
            .multisig_institutions
            .iter()
            .find(|(_, inst)| same_city_template_slot(inst, target))
        {
            return sfid.clone();
        }
        if let Some(existing) = store.multisig_institutions.get(&target.sfid_number) {
            if !same_city_template_slot(existing, target) {
                for retry in 1..1000u32 {
                    let seed = format!("{}#{retry}", target.account_seed);
                    if let Some(candidate) = generate_official_template_sfid(
                        &seed,
                        &target.province,
                        &target.city,
                        &target.institution_code,
                    ) {
                        if !store.multisig_institutions.contains_key(&candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }
    }
    target.sfid_number.clone()
}

fn apply_official_target(
    inst: &mut MultisigInstitution,
    target: &OfficialInstitutionTarget,
) -> bool {
    let mut changed = false;
    let source = Some(InstitutionSource::Auto);
    let level = Some(target.institution_level.clone());
    let name = Some(target.institution_name.clone());
    if inst.institution_name != name {
        inst.institution_name = name;
        changed = true;
    }
    if inst.category != target.category {
        inst.category = target.category;
        changed = true;
    }
    if inst.source != source {
        inst.source = source;
        changed = true;
    }
    if inst.institution_level != level {
        inst.institution_level = level;
        changed = true;
    }
    if inst.a3 != target.a3 {
        inst.a3 = target.a3.clone();
        changed = true;
    }
    if inst.p1 != target.p1 {
        inst.p1 = target.p1.clone();
        changed = true;
    }
    if inst.province != target.province {
        inst.province = target.province.clone();
        changed = true;
    }
    if inst.city != target.city {
        inst.city = target.city.clone();
        changed = true;
    }
    if inst.province_code != target.province_code {
        inst.province_code = target.province_code.clone();
        changed = true;
    }
    if inst.city_code != target.city_code {
        inst.city_code = target.city_code.clone();
        changed = true;
    }
    if inst.institution_code != target.institution_code {
        inst.institution_code = target.institution_code.clone();
        changed = true;
    }
    if inst.sub_type.is_some() {
        inst.sub_type = None;
        changed = true;
    }
    if inst.parent_sfid_number.is_some() {
        inst.parent_sfid_number = None;
        changed = true;
    }
    changed
}

fn new_official_institution(
    sfid_number: String,
    target: &OfficialInstitutionTarget,
    actor: &str,
) -> MultisigInstitution {
    MultisigInstitution {
        sfid_number,
        institution_name: Some(target.institution_name.clone()),
        category: target.category,
        source: Some(InstitutionSource::Auto),
        institution_level: Some(target.institution_level.clone()),
        a3: target.a3.clone(),
        p1: target.p1.clone(),
        province: target.province.clone(),
        city: target.city.clone(),
        province_code: target.province_code.clone(),
        city_code: target.city_code.clone(),
        institution_code: target.institution_code.clone(),
        sub_type: None,
        parent_sfid_number: None,
        chain_status: InstitutionChainStatus::NotRegistered,
        chain_tx_hash: None,
        chain_block_number: None,
        chain_synced_at: None,
        created_by: actor.to_string(),
        created_at: Utc::now(),
    }
}

fn is_manual_school_institution(inst: &MultisigInstitution) -> bool {
    inst.category == InstitutionCategory::GovInstitution
        && inst.source.is_none()
        && inst.institution_code == "JY"
        && inst.institution_level.is_none()
}

fn should_remove_from_official_directory(
    inst: &MultisigInstitution,
    target_sfids: &HashSet<String>,
) -> bool {
    if target_sfids.contains(&inst.sfid_number)
        || matches!(inst.category, InstitutionCategory::PublicSecurity)
        || is_manual_school_institution(inst)
    {
        return false;
    }
    matches!(inst.source, Some(InstitutionSource::Auto))
        || matches!(inst.category, InstitutionCategory::GovInstitution)
}

/// 对账普通公权/宪法机构目录。
///
/// 中文注释:机构唯一身份仍然只有 `sfid_number`。城市模板机构为了应对市名变化,
/// reconcile 只在内存计算阶段按 `(source, level, province_code, city_code, institution_code)`
/// 找现有记录,找到后更新名称但不改 sfid_number,不会把这个匹配键保存成第二身份。
pub fn reconcile_official_institutions(store: &mut Store, actor: &str) -> OfficialReconcileReport {
    let targets = official_institution_targets();
    let mut report = OfficialReconcileReport::default();
    let mut touched = HashSet::<String>::new();
    let mut target_sfids = HashSet::<String>::new();

    for target in &targets {
        let sfid_number = resolve_official_target_sfid(store, target);
        target_sfids.insert(sfid_number.clone());
        touched.insert(sfid_number.clone());

        if let Some(existing) = store.multisig_institutions.get_mut(&sfid_number) {
            if apply_official_target(existing, target) {
                report.updated += 1;
            }
        } else {
            let inst = new_official_institution(sfid_number.clone(), target, actor);
            store
                .multisig_institutions
                .insert(sfid_number.clone(), inst);
            report.inserted += 1;
        }
        insert_default_accounts_into_global_store(store, &sfid_number, actor);
    }

    let to_remove: Vec<String> = store
        .multisig_institutions
        .values()
        .filter(|inst| should_remove_from_official_directory(inst, &target_sfids))
        .map(|inst| inst.sfid_number.clone())
        .collect();
    for sfid in to_remove {
        store.multisig_institutions.remove(&sfid);
        store
            .multisig_accounts
            .retain(|_, account| account.sfid_number != sfid);
        report.removed += 1;
        report.removed_sfids.push(sfid);
    }

    report.total_after = store
        .multisig_institutions
        .values()
        .filter(|inst| {
            matches!(inst.source, Some(InstitutionSource::Auto))
                && !matches!(inst.category, InstitutionCategory::PublicSecurity)
        })
        .count();
    report.touched_sfids = touched.into_iter().collect();
    report.touched_sfids.sort();
    report.removed_sfids.sort();
    report
}

/// 按 city_code 对账指定省的公安局机构。
///
/// 规则:
/// - 增:sfid 工具里有但 multisig_institutions 没有 → 生成新机构
/// - 删:multisig_institutions 有但 sfid 工具没有的 city_code → 删除机构
///      (不触链,账户记录由清理任务统一处理)
/// - 改:city_code 相同但 city 字符串不同 → 更新 city + institution_name
///      sfid_number 不变
///
/// 特殊处理:
/// - 跳过 city_code == "000" 的保留占位
/// - 跳过模块 Store 快照已有但 city_code 为空的记录(会被 backfill 函数修复)
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
    let province_entry = match provinces().iter().find(|p| p.name == province_name) {
        Some(p) => p,
        None => return report, // 未知省份,跳过
    };
    let authoritative_cities: Vec<(String, String)> = province_entry
        .cities
        .iter()
        .filter(|c| c.code != "000")
        .map(|c| (c.name.to_string(), c.code.to_string()))
        .collect();

    // 2. 现有该省的公安局机构索引:city_code → sfid_number
    let existing_by_code: std::collections::HashMap<String, String> = store
        .multisig_institutions
        .values()
        .filter(|i| {
            matches!(i.category, InstitutionCategory::PublicSecurity)
                && i.province == province_name
                && !i.city_code.is_empty()
        })
        .map(|i| (i.city_code.clone(), i.sfid_number.clone()))
        .collect();

    // 3. 增 + 改
    for (city_name, city_code) in &authoritative_cities {
        if let Some(sfid_number) = existing_by_code.get(city_code) {
            // 存在 → 检查市名是否需要更新
            if let Some(inst) = store.multisig_institutions.get_mut(sfid_number) {
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
            // 缺失 → 生成新机构(碰撞重试,1000 次保护栏)
            //
            // 公安局桶 = (a3=GFR, 一省一市, 机构=ZF, year),全国仅几百市级公安局,
            // 桶填充率 < 0.001%,几乎不可能撞;1000 次保护栏只是防代码 bug 死循环。
            let account_placeholder = format!("PS-{}-{}", province_entry.code, city_code);
            let mut generated: Option<String> = None;
            for retry in 0..1000u32 {
                let attempt_account = if retry == 0 {
                    account_placeholder.clone()
                } else {
                    format!("{account_placeholder}#{retry}")
                };
                let candidate = match generate_sfid_code(GenerateSfidInput {
                    account_pubkey: attempt_account.as_str(),
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
                        break;
                    }
                };
                if !store.multisig_institutions.contains_key(&candidate) {
                    generated = Some(candidate);
                    break;
                }
            }
            let sfid_number = match generated {
                Some(v) => v,
                None => {
                    tracing::error!(
                        province = %province_name,
                        city = %city_name,
                        "reconcile: sfid generation exhausted 1000 retries (bucket near-saturation)"
                    );
                    continue;
                }
            };
            let inst = MultisigInstitution {
                sfid_number: sfid_number.clone(),
                institution_name: Some(format!("{}公安局", city_name)),
                category: InstitutionCategory::PublicSecurity,
                source: Some(InstitutionSource::Auto),
                institution_level: Some(InstitutionLevel::City),
                a3: "GFR".to_string(),
                p1: "0".to_string(),
                province: province_name.to_string(),
                city: city_name.clone(),
                province_code: province_entry.code.to_string(),
                city_code: city_code.clone(),
                institution_code: "ZF".to_string(),
                sub_type: None,
                parent_sfid_number: None,
                chain_status: InstitutionChainStatus::NotRegistered,
                chain_tx_hash: None,
                chain_block_number: None,
                chain_synced_at: None,
                created_by: actor.to_string(),
                created_at: Utc::now(),
            };
            store
                .multisig_institutions
                .insert(sfid_number.clone(), inst);
            // 公安局 reconcile 同步插入 2 条默认未上链账户。
            insert_default_accounts_into_global_store(store, &sfid_number, actor);
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
        .map(|i| i.sfid_number.clone())
        .collect();
    for sfid_number in to_remove {
        store.multisig_institutions.remove(&sfid_number);
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

/// 既有记录 backfill:扫 multisig_institutions,给 city_code 为空的公安局机构
/// 用 (province, city) 反查权威清单补上。必须在 reconcile 之前跑一次,
/// 否则既有记录会被 reconcile 误删。
pub fn backfill_public_security_city_code_fields(store: &mut Store) -> usize {
    let mut fixed = 0usize;
    let targets: Vec<(String, String, String)> = store
        .multisig_institutions
        .values()
        .filter(|i| {
            matches!(i.category, InstitutionCategory::PublicSecurity) && i.city_code.is_empty()
        })
        .map(|i| (i.sfid_number.clone(), i.province.clone(), i.city.clone()))
        .collect();
    for (sfid_number, province, city) in targets {
        let Some(entry) = provinces().iter().find(|p| p.name == province) else {
            continue;
        };
        let Some(cc) = entry.cities.iter().find(|c| c.name == city).map(|c| c.code) else {
            continue;
        };
        if let Some(inst) = store.multisig_institutions.get_mut(&sfid_number) {
            inst.city_code = cc.to_string();
            fixed += 1;
        }
    }
    fixed
}
