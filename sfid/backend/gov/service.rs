//! 公权机构自动目录与公安局对账服务。
//!
//! 中文注释:自动生成的公权机构只归 gov 模块维护。编译不写库,serve 不全量写库;
//! 部署入口用 `ensure-gov` 做幂等守门,只有目录缺失或不完整时才初始化。

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::china::{china_sqlite_hash, provinces};
use crate::number::{generate_sfid_number, GenerateSfidInput, InstitutionCategory};
use crate::subjects::service::build_default_accounts;
use crate::Db;

#[allow(dead_code)]
#[path = "../../../citizenchain/runtime/primitives/china/china_cb.rs"]
mod china_cb_constants;
#[allow(dead_code)]
#[path = "../../../citizenchain/runtime/primitives/china/china_ch.rs"]
mod china_ch_constants;
#[allow(dead_code)]
#[path = "../../../citizenchain/runtime/primitives/china/china_jc.rs"]
mod china_jc_constants;
#[allow(dead_code)]
#[path = "../../../citizenchain/runtime/primitives/china/china_jy.rs"]
mod china_jy_constants;
#[allow(dead_code)]
#[path = "../../../citizenchain/runtime/primitives/china/china_lf.rs"]
mod china_lf_constants;
#[allow(dead_code)]
#[path = "../../../citizenchain/runtime/primitives/china/china_sf.rs"]
mod china_sf_constants;
#[allow(dead_code)]
#[path = "../../../citizenchain/runtime/primitives/china/china_zf.rs"]
mod china_zf_constants;

pub const GOV_TEMPLATE_VERSION: &str = "gov-deterministic-v3";
pub const DEFAULT_ACCOUNT_COUNT: i64 = 2;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ReconcileReport {
    pub province: String,
    pub inserted: usize,
    pub updated: usize,
    pub removed: usize,
    pub total_after: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OfficialReconcileReport {
    pub inserted: usize,
    pub updated: usize,
    pub account_inserted: usize,
    pub removed: usize,
    pub total_after: usize,
    pub target_sfids: Vec<String>,
    pub touched_sfids: Vec<String>,
    pub removed_sfids: Vec<String>,
    pub scope_key: String,
    pub china_hash: String,
    pub catalog_hash: String,
    pub template_version: &'static str,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GovDirectoryCheckReport {
    pub ok: bool,
    pub scope_key: String,
    pub china_hash: String,
    pub catalog_hash: String,
    pub manifest_catalog_hash: Option<String>,
    pub template_version: &'static str,
    pub target_count: usize,
    pub active_count: usize,
    pub missing_sfids: Vec<String>,
    pub mismatched_sfids: Vec<String>,
    pub missing_account_sfids: Vec<String>,
    pub obsolete_sfids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfficialReconcileScope {
    All,
    Province {
        province_code: String,
    },
    City {
        province_code: String,
        city_code: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovTargetKind {
    All,
    Official,
    PublicSecurity,
}

#[derive(Debug, Clone)]
struct OfficialInstitutionTarget {
    sfid_number: String,
    institution_name: String,
    full_name: String,
    short_name: String,
    category: InstitutionCategory,
    subject_property: String,
    p1: String,
    province: String,
    city: String,
    town: String,
    province_code: String,
    city_code: String,
    town_code: String,
    institution_code: String,
    org_code: String,
}

#[derive(Debug, Clone, Copy)]
struct OfficialOrgTemplate {
    institution_code: &'static str,
    org_code: &'static str,
    suffix: &'static str,
    full_suffix: &'static str,
}

const PROVINCE_DEPARTMENT_TEMPLATES: &[OfficialOrgTemplate] = &[
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_DEFENSE",
        suffix: "国防厅",
        full_suffix: "国家防务厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_SECURITY",
        suffix: "国安厅",
        full_suffix: "国土安全厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_CIVIL_LIFE",
        suffix: "民生厅",
        full_suffix: "公民生活保障厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_HOUSING",
        suffix: "住建厅",
        full_suffix: "住房与城镇建设厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_AGRICULTURE",
        suffix: "农业厅",
        full_suffix: "农业与农村发展厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_COMMERCE",
        suffix: "商贸厅",
        full_suffix: "商务与市场贸易厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_FINANCE_TAX",
        suffix: "财税厅",
        full_suffix: "财政与税务厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_ENERGY",
        suffix: "能源厅",
        full_suffix: "能源与环保发展厅",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "PROVINCE_TRANSPORT",
        suffix: "交通厅",
        full_suffix: "交通运输厅",
    },
    OfficialOrgTemplate {
        institution_code: "LF",
        org_code: "PROVINCE_SENATE_COUNCIL",
        suffix: "参议员议政会",
        full_suffix: "参议员议政会",
    },
    OfficialOrgTemplate {
        institution_code: "LF",
        org_code: "PROVINCE_REPRESENTATIVE_COUNCIL",
        suffix: "众议员议政会",
        full_suffix: "众议员议政会",
    },
];

const CITY_TEMPLATES: &[OfficialOrgTemplate] = &[
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_GOV",
        suffix: "自治政府",
        full_suffix: "自治政府",
    },
    OfficialOrgTemplate {
        institution_code: "LF",
        org_code: "CITY_LEGISLATURE",
        suffix: "立法会",
        full_suffix: "公民立法委员会",
    },
    OfficialOrgTemplate {
        institution_code: "JC",
        org_code: "CITY_SUPERVISION",
        suffix: "监察院",
        full_suffix: "监察院",
    },
    OfficialOrgTemplate {
        institution_code: "SF",
        org_code: "CITY_COURT",
        suffix: "司法院",
        full_suffix: "司法院",
    },
    OfficialOrgTemplate {
        institution_code: "JY",
        org_code: "CITY_EDU",
        suffix: "公民教育委员会",
        full_suffix: "公民教育委员会",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_CITIZEN_SELF_GOV",
        suffix: "自治会",
        full_suffix: "公民自治委员会",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_DEFENSE",
        suffix: "国防局",
        full_suffix: "国家防务局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_SECURITY",
        suffix: "国安局",
        full_suffix: "国土安全局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_CIVIL_LIFE",
        suffix: "民生局",
        full_suffix: "公民生活保障局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_HOUSING",
        suffix: "住建局",
        full_suffix: "住房与城镇建设局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_AGRICULTURE",
        suffix: "农业局",
        full_suffix: "农业与农村发展局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_COMMERCE",
        suffix: "商贸局",
        full_suffix: "商务与市场贸易局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_FINANCE_TAX",
        suffix: "财税局",
        full_suffix: "财政与税务局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_ENERGY",
        suffix: "能源局",
        full_suffix: "能源与环保发展局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_TRANSPORT",
        suffix: "交通局",
        full_suffix: "交通运输局",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "CITY_REGISTRY",
        suffix: "注册局",
        full_suffix: "身份注册局",
    },
];

const TOWN_TEMPLATES: &[OfficialOrgTemplate] = &[
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "TOWN_GOV",
        suffix: "自治政府",
        full_suffix: "自治政府",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "TOWN_CIVIL_LIFE",
        suffix: "民生科",
        full_suffix: "公民生活保障科",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "TOWN_HOUSING",
        suffix: "住建科",
        full_suffix: "住房与城镇建设科",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "TOWN_AGRICULTURE",
        suffix: "农业科",
        full_suffix: "农业与农村发展科",
    },
    OfficialOrgTemplate {
        institution_code: "ZF",
        org_code: "TOWN_FINANCE_TAX",
        suffix: "财税科",
        full_suffix: "财政与税务科",
    },
];

impl OfficialReconcileScope {
    fn scope_key(&self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Province { province_code } => {
                format!("province:{}", province_code.to_ascii_uppercase())
            }
            Self::City {
                province_code,
                city_code,
            } => format!(
                "city:{}:{}",
                province_code.to_ascii_uppercase(),
                city_code.to_ascii_uppercase()
            ),
        }
    }
}

impl GovTargetKind {
    fn scope_key_suffix(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Official => "official",
            Self::PublicSecurity => "public-security",
        }
    }

    fn includes(self, category: InstitutionCategory) -> bool {
        match self {
            Self::All => true,
            Self::Official => category == InstitutionCategory::GovInstitution,
            Self::PublicSecurity => category == InstitutionCategory::PublicSecurity,
        }
    }
}

fn scoped_manifest_key(scope: &OfficialReconcileScope, kind: GovTargetKind) -> String {
    format!("{}:{}", kind.scope_key_suffix(), scope.scope_key())
}

pub fn gov_manifest_key(scope: &OfficialReconcileScope, kind: GovTargetKind) -> String {
    scoped_manifest_key(scope, kind)
}

fn official_institution_targets() -> Vec<OfficialInstitutionTarget> {
    let mut targets = Vec::new();
    for item in china_zf_constants::CHINA_ZF.iter() {
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number);
    }
    for item in china_lf_constants::CHINA_LF.iter() {
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number);
    }
    for item in china_sf_constants::CHINA_SF.iter() {
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number);
    }
    for item in china_jc_constants::CHINA_JC.iter() {
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number);
    }
    for item in china_jy_constants::CHINA_JY.iter() {
        push_constant_target(&mut targets, "国家教育委员会", item.sfid_number);
    }
    for item in china_cb_constants::CHINA_CB.iter() {
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number);
    }
    for item in china_ch_constants::CHINA_CH.iter() {
        push_constant_target(&mut targets, item.sfid_name, item.sfid_number);
    }
    push_extra_national_targets(&mut targets);
    for province in provinces().iter() {
        let province_home_city = province
            .cities
            .iter()
            .find(|city| city.code == "001")
            .or_else(|| province.cities.first());
        if let Some(home_city) = province_home_city {
            for template in PROVINCE_DEPARTMENT_TEMPLATES {
                push_area_template_target(
                    &mut targets,
                    province.name,
                    province.code,
                    home_city.name,
                    home_city.code,
                    "",
                    "",
                    province.name,
                    template,
                    "PROVINCE",
                );
            }
        }
        for city in province.cities.iter().filter(|city| city.code != "000") {
            for template in CITY_TEMPLATES {
                push_area_template_target(
                    &mut targets,
                    province.name,
                    province.code,
                    city.name,
                    city.code,
                    "",
                    "",
                    city.name,
                    template,
                    "CITY",
                );
            }
            for town in city.towns {
                for template in TOWN_TEMPLATES {
                    push_area_template_target(
                        &mut targets,
                        province.name,
                        province.code,
                        city.name,
                        city.code,
                        town.name,
                        town.code,
                        town.name,
                        template,
                        "TOWN",
                    );
                }
            }
        }
    }
    targets
}

fn public_security_targets() -> Vec<OfficialInstitutionTarget> {
    let mut targets = Vec::new();
    for province in provinces().iter() {
        for city in province.cities.iter().filter(|city| city.code != "000") {
            let seed = format!("PS-{}-{}", province.code, city.code);
            let Some(sfid_number) =
                generate_official_template_sfid(&seed, province.name, city.name, "ZF")
            else {
                continue;
            };
            targets.push(OfficialInstitutionTarget {
                sfid_number,
                institution_name: format!("{}公安局", city.name),
                full_name: format!("{}公民安全局", city.name),
                short_name: format!("{}公安局", city.name),
                category: InstitutionCategory::PublicSecurity,
                subject_property: "G".to_string(),
                p1: "0".to_string(),
                province: province.name.to_string(),
                city: city.name.to_string(),
                town: String::new(),
                province_code: province.code.to_string(),
                city_code: city.code.to_string(),
                town_code: String::new(),
                institution_code: "ZF".to_string(),
                org_code: "CITY_POLICE".to_string(),
            });
        }
    }
    targets
}

fn build_raw_targets(kind: GovTargetKind) -> Vec<OfficialInstitutionTarget> {
    let mut targets = Vec::new();
    if matches!(kind, GovTargetKind::All | GovTargetKind::Official) {
        targets.extend(official_institution_targets());
    }
    if matches!(kind, GovTargetKind::All | GovTargetKind::PublicSecurity) {
        targets.extend(public_security_targets());
    }
    targets
}

fn target_in_scope(target: &OfficialInstitutionTarget, scope: &OfficialReconcileScope) -> bool {
    match scope {
        OfficialReconcileScope::All => true,
        OfficialReconcileScope::Province { province_code } => {
            target.province_code.eq_ignore_ascii_case(province_code)
        }
        OfficialReconcileScope::City {
            province_code,
            city_code,
        } => {
            target.province_code.eq_ignore_ascii_case(province_code)
                && target.city_code.eq_ignore_ascii_case(city_code)
        }
    }
}

fn push_constant_target(
    targets: &mut Vec<OfficialInstitutionTarget>,
    name: &'static str,
    sfid_number: &'static str,
) {
    let Some((subject_property, province_code, city_code, institution_code, p1)) =
        parse_sfid_institution_parts(sfid_number)
    else {
        return;
    };
    let Some((province, city)) = province_city_by_codes(&province_code, &city_code) else {
        return;
    };
    let (full_name, short_name) = official_name_pair(name);
    targets.push(OfficialInstitutionTarget {
        sfid_number: sfid_number.to_string(),
        institution_name: short_name.clone(),
        full_name,
        short_name,
        category: InstitutionCategory::GovInstitution,
        subject_property,
        p1,
        province: province.to_string(),
        city: city.to_string(),
        town: String::new(),
        province_code,
        city_code,
        town_code: String::new(),
        institution_code,
        org_code: org_code_for_constant_name(name).to_string(),
    });
}

fn official_name_pair(name: &str) -> (String, String) {
    const COUNTRY: &str = "中华民族联邦共和国";
    let full = match name {
        "总统府" => format!("{COUNTRY}总统府"),
        "外交部" => format!("{COUNTRY}外事交流部"),
        "国防部" => format!("{COUNTRY}国家防务部"),
        "国安部" => format!("{COUNTRY}国土安全部"),
        "民生部" => format!("{COUNTRY}公民生活保障部"),
        "住建部" => format!("{COUNTRY}住房与城镇建设部"),
        "农业部" => format!("{COUNTRY}农业与农村发展部"),
        "商贸部" => format!("{COUNTRY}商务与市场贸易部"),
        "财税部" => format!("{COUNTRY}财政与税务部"),
        "能源部" => format!("{COUNTRY}能源与环保发展部"),
        "交通部" => format!("{COUNTRY}交通运输部"),
        "国家立法院" | "国家司法院" | "国家监察院" | "国家教育委员会" | "国家储备委员会" =>
        {
            format!("{COUNTRY}{name}")
        }
        "联邦廉政署" | "联邦审计署" | "联邦调查署" => {
            format!("{COUNTRY}国家监察院{name}")
        }
        _ => name.to_string(),
    };
    (full, name.to_string())
}

fn org_code_for_constant_name(name: &str) -> &'static str {
    match name {
        "总统府" => "NATIONAL_PRESIDENT_OFFICE",
        "外交部" => "MINISTRY_FOREIGN",
        "国防部" => "MINISTRY_DEFENSE",
        "国安部" => "MINISTRY_SECURITY",
        "民生部" => "MINISTRY_CIVIL_LIFE",
        "住建部" => "MINISTRY_HOUSING",
        "农业部" => "MINISTRY_AGRICULTURE",
        "商贸部" => "MINISTRY_COMMERCE",
        "财税部" => "MINISTRY_FINANCE_TAX",
        "能源部" => "MINISTRY_ENERGY",
        "交通部" => "MINISTRY_TRANSPORT",
        "国家立法院" => "NATIONAL_LEGISLATURE",
        "国家司法院" => "NATIONAL_COURT",
        "国家监察院" => "NATIONAL_SUPERVISION",
        "联邦廉政署" => "FEDERAL_INTEGRITY",
        "联邦审计署" => "FEDERAL_AUDIT",
        "联邦调查署" => "FEDERAL_INVESTIGATION",
        "国家教育委员会" | "公民教育委员会" => "NATIONAL_EDU",
        "国家储备委员会" => "NATIONAL_RESERVE",
        _ if name.ends_with("省政府") => "PROVINCE_GOV",
        _ if name.ends_with("省立法院") => "PROVINCE_LEGISLATURE",
        _ if name.ends_with("省司法院") => "PROVINCE_COURT",
        _ if name.ends_with("省监察院") => "PROVINCE_SUPERVISION",
        _ if name.ends_with("省储备委员会") => "PROVINCE_RESERVE",
        _ if name.ends_with("省公民储备银行") => "PROVINCE_RESERVE_BANK",
        _ => "PUBLIC_ORG",
    }
}

fn push_extra_national_targets(targets: &mut Vec<OfficialInstitutionTarget>) {
    let Some(province) = provinces()
        .iter()
        .find(|province| province.name == "中枢省")
    else {
        return;
    };
    let Some(city) = province
        .cities
        .iter()
        .find(|city| city.code == "001")
        .or_else(|| province.cities.first())
    else {
        return;
    };
    for (short_name, full_name, org_code) in [
        (
            "联邦特勤局",
            "中华民族联邦共和国总统府联邦特勤局",
            "FEDERAL_SPECIAL_SERVICE",
        ),
        (
            "联邦安全局",
            "中华民族联邦共和国总统府联邦安全局",
            "FEDERAL_SECURITY",
        ),
        (
            "联邦情报局",
            "中华民族联邦共和国总统府联邦情报局",
            "FEDERAL_INTELLIGENCE",
        ),
        (
            "联邦人事局",
            "中华民族联邦共和国总统府联邦人事局",
            "FEDERAL_PERSONNEL",
        ),
        (
            "联邦注册局",
            "中华民族联邦共和国总统府联邦注册局",
            "FEDERAL_REGISTRY",
        ),
        (
            "国家参议会",
            "中华民族联邦共和国国家立法院参议员议政会",
            "NATIONAL_SENATE_COUNCIL",
        ),
        (
            "国家众议会",
            "中华民族联邦共和国国家立法院众议员议政会",
            "NATIONAL_REPRESENTATIVE_COUNCIL",
        ),
    ] {
        let template = OfficialOrgTemplate {
            institution_code: if short_name.ends_with("议会") {
                "LF"
            } else {
                "ZF"
            },
            org_code,
            suffix: short_name,
            full_suffix: full_name,
        };
        push_area_template_target(
            targets,
            province.name,
            province.code,
            city.name,
            city.code,
            "",
            "",
            "",
            &template,
            "NATIONAL",
        );
    }
}

fn push_area_template_target(
    targets: &mut Vec<OfficialInstitutionTarget>,
    province_name: &'static str,
    province_code: &'static str,
    city_name: &'static str,
    city_code: &'static str,
    town_name: &'static str,
    town_code: &'static str,
    display_area_name: &'static str,
    template: &OfficialOrgTemplate,
    seed_scope: &'static str,
) {
    let short_name = format!("{display_area_name}{}", template.suffix);
    let full_name = format!("{display_area_name}{}", template.full_suffix);
    let account_seed = format!(
        "GOV-{seed_scope}-{province_code}-{city_code}-{town_code}-{}-{}",
        template.institution_code, template.org_code
    );
    let Some(sfid_number) = generate_official_template_sfid(
        &account_seed,
        province_name,
        city_name,
        template.institution_code,
    ) else {
        return;
    };
    targets.push(OfficialInstitutionTarget {
        sfid_number,
        institution_name: short_name.clone(),
        full_name,
        short_name,
        category: InstitutionCategory::GovInstitution,
        subject_property: "G".to_string(),
        p1: "0".to_string(),
        province: province_name.to_string(),
        city: city_name.to_string(),
        town: town_name.to_string(),
        province_code: province_code.to_string(),
        city_code: city_code.to_string(),
        town_code: town_code.to_string(),
        institution_code: template.institution_code.to_string(),
        org_code: template.org_code.to_string(),
    });
}

fn generate_official_template_sfid(
    account_seed: &str,
    province_name: &str,
    city_name: &str,
    institution_code: &str,
) -> Option<String> {
    generate_sfid_number(GenerateSfidInput {
        account_pubkey: account_seed,
        subject_property: "G",
        p1: "0",
        province: province_name,
        city: city_name,
        institution: institution_code,
    })
    .ok()
}

fn parse_sfid_institution_parts(
    sfid_number: &str,
) -> Option<(String, String, String, String, String)> {
    let mut segments = sfid_number.split('-');
    let r5 = segments.next()?;
    let k3p1c1 = segments.next()?;
    if r5.len() != 5 || k3p1c1.len() != 5 {
        return None;
    }
    Some((
        k3p1c1[0..1].to_string(),
        r5[0..2].to_string(),
        r5[2..5].to_string(),
        k3p1c1[1..3].to_string(),
        k3p1c1[3..4].to_string(),
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

fn category_text(category: InstitutionCategory) -> &'static str {
    match category {
        InstitutionCategory::PublicSecurity => "PUBLIC_SECURITY",
        InstitutionCategory::GovInstitution => "GOV_INSTITUTION",
        InstitutionCategory::PrivateInstitution => "PRIVATE_INSTITUTION",
    }
}

fn resolve_targets(
    _db: &Db,
    scope: &OfficialReconcileScope,
    kind: GovTargetKind,
) -> Result<Vec<OfficialInstitutionTarget>, String> {
    let mut targets = build_raw_targets(kind)
        .into_iter()
        .filter(|target| target_in_scope(target, scope))
        .collect::<Vec<_>>();
    targets.sort_by(|a, b| {
        (
            a.province_code.as_str(),
            a.city_code.as_str(),
            a.town_code.as_str(),
            category_text(a.category),
            a.institution_code.as_str(),
            a.org_code.as_str(),
            a.sfid_number.as_str(),
        )
            .cmp(&(
                b.province_code.as_str(),
                b.city_code.as_str(),
                b.town_code.as_str(),
                category_text(b.category),
                b.institution_code.as_str(),
                b.org_code.as_str(),
                b.sfid_number.as_str(),
            ))
    });
    Ok(targets)
}

fn catalog_hash(china_hash: &str, targets: &[OfficialInstitutionTarget]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(GOV_TEMPLATE_VERSION.as_bytes());
    hasher.update(b"\n");
    hasher.update(china_hash.as_bytes());
    hasher.update(b"\n");
    for target in targets {
        hasher.update(target.sfid_number.as_bytes());
        hasher.update(b"|");
        hasher.update(target.institution_name.as_bytes());
        hasher.update(b"|");
        hasher.update(target.full_name.as_bytes());
        hasher.update(b"|");
        hasher.update(target.short_name.as_bytes());
        hasher.update(b"|");
        hasher.update(category_text(target.category).as_bytes());
        hasher.update(b"|");
        hasher.update(target.province_code.as_bytes());
        hasher.update(b"|");
        hasher.update(target.city_code.as_bytes());
        hasher.update(b"|");
        hasher.update(target.town_code.as_bytes());
        hasher.update(b"|");
        hasher.update(target.institution_code.as_bytes());
        hasher.update(b"|");
        hasher.update(target.org_code.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

pub fn current_gov_manifest_version(db: &Db, scope_key: &str) -> Option<String> {
    let scope_key = scope_key.to_string();
    db.with_client(move |conn| {
        let row = conn
            .query_opt(
                "SELECT catalog_hash FROM gov_manifest WHERE scope_key = $1 AND status = 'OK'",
                &[&scope_key],
            )
            .map_err(|e| format!("query gov manifest failed: {e}"))?;
        Ok(row.map(|r| r.get::<_, String>(0)))
    })
    .ok()
    .flatten()
}

fn upsert_manifest(
    db: &Db,
    scope_key: &str,
    china_hash: &str,
    catalog_hash: &str,
    target_count: usize,
    ok: bool,
) -> Result<(), String> {
    let scope_key = scope_key.to_string();
    let china_hash = china_hash.to_string();
    let catalog_hash = catalog_hash.to_string();
    let target_count =
        i64::try_from(target_count).map_err(|_| "gov target count exceeds i64".to_string())?;
    let status = if ok { "OK" } else { "INCOMPLETE" }.to_string();
    db.with_client(move |conn| {
        conn.execute(
            "INSERT INTO gov_manifest (
                scope_key, china_hash, catalog_hash, template_version, target_count, status, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, now())
             ON CONFLICT (scope_key) DO UPDATE SET
                china_hash = EXCLUDED.china_hash,
                catalog_hash = EXCLUDED.catalog_hash,
                template_version = EXCLUDED.template_version,
                target_count = EXCLUDED.target_count,
                status = EXCLUDED.status,
                updated_at = now()",
            &[
                &scope_key,
                &china_hash,
                &catalog_hash,
                &GOV_TEMPLATE_VERSION,
                &target_count,
                &status,
            ],
        )
        .map_err(|e| format!("upsert gov manifest failed: {e}"))?;
        Ok(())
    })
}

pub fn upsert_gov_manifest_from_check_db(
    db: &Db,
    report: &GovDirectoryCheckReport,
) -> Result<(), String> {
    upsert_manifest(
        db,
        report.scope_key.as_str(),
        report.china_hash.as_str(),
        report.catalog_hash.as_str(),
        report.target_count,
        report.ok,
    )
}

fn target_category_sql(kind: GovTargetKind) -> Option<&'static str> {
    match kind {
        GovTargetKind::All => None,
        GovTargetKind::Official => Some("GOV_INSTITUTION"),
        GovTargetKind::PublicSecurity => Some("PUBLIC_SECURITY"),
    }
}

fn auto_rows_in_scope(
    db: &Db,
    scope: &OfficialReconcileScope,
    kind: GovTargetKind,
) -> Result<
    BTreeMap<
        String,
        (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ),
    >,
    String,
> {
    let p_filter = match scope {
        OfficialReconcileScope::All => None,
        OfficialReconcileScope::Province { province_code }
        | OfficialReconcileScope::City { province_code, .. } => Some(province_code.clone()),
    };
    let c_filter = match scope {
        OfficialReconcileScope::City { city_code, .. } => Some(city_code.clone()),
        _ => None,
    };
    let category_filter = target_category_sql(kind).map(str::to_string);
    db.with_client(move |conn| {
        let rows = conn
            .query(
                "SELECT s.sfid_number, COALESCE(s.name, ''), COALESCE(s.full_name, ''),
                        COALESCE(s.short_name, ''), s.category, s.province, s.city,
                        COALESCE(s.town, ''), s.province_code, s.city_code,
                        COALESCE(s.town_code, ''), s.institution_code, COALESCE(g.org_code, '')
                 FROM subjects s
                 JOIN gov g ON g.p_code = s.p_code AND g.sfid_number = s.sfid_number
                 WHERE s.kind = 'PUBLIC'
                   AND s.status = 'ACTIVE'
                   AND ($1::text IS NULL OR s.p_code = $1)
                   AND ($2::text IS NULL OR s.c_code = $2)
                   AND ($3::text IS NULL OR s.category = $3)",
                &[&p_filter, &c_filter, &category_filter],
            )
            .map_err(|e| format!("query active auto gov rows failed: {e}"))?;
        let mut output = BTreeMap::new();
        for row in rows {
            output.insert(
                row.get::<_, String>(0),
                (
                    row.get(1),
                    row.get(2),
                    row.get(3),
                    row.get(4),
                    row.get(5),
                    row.get(6),
                    row.get(7),
                    row.get(8),
                    row.get(9),
                    row.get(10),
                    row.get(11),
                    row.get(12),
                ),
            );
        }
        Ok(output)
    })
}

fn account_counts(db: &Db, sfids: &[String]) -> Result<BTreeMap<String, i64>, String> {
    if sfids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let sfids = sfids.to_vec();
    db.with_client(move |conn| {
        let mut output = BTreeMap::new();
        // 中文注释:全量镇目录接近 30 万机构,账户校验按块查,避免超大数组压垮单条 SQL。
        for chunk in sfids.chunks(10_000) {
            let chunk = chunk.to_vec();
            let rows = conn
                .query(
                    "SELECT sfid_number, COUNT(*)::BIGINT
                     FROM accounts
                     WHERE sfid_number = ANY($1)
                     GROUP BY sfid_number",
                    &[&chunk],
                )
                .map_err(|e| {
                    format!(
                        "query gov account counts failed: {}",
                        crate::core::db::postgres_error_text(&e)
                    )
                })?;
            for row in rows {
                output.insert(row.get::<_, String>(0), row.get::<_, i64>(1));
            }
        }
        Ok(output)
    })
}

pub fn check_gov_catalog_db(
    db: &Db,
    scope: OfficialReconcileScope,
    kind: GovTargetKind,
) -> Result<GovDirectoryCheckReport, String> {
    let targets = resolve_targets(db, &scope, kind)?;
    let china_hash = china_sqlite_hash()?;
    let catalog_hash = catalog_hash(china_hash.as_str(), &targets);
    let scope_key = scoped_manifest_key(&scope, kind);
    let manifest_catalog_hash = current_gov_manifest_version(db, scope_key.as_str());
    let active_rows = auto_rows_in_scope(db, &scope, kind)?;
    let target_sfids = targets
        .iter()
        .map(|target| target.sfid_number.clone())
        .collect::<BTreeSet<_>>();
    let counts = account_counts(db, &target_sfids.iter().cloned().collect::<Vec<_>>())?;

    let mut missing_sfids = Vec::new();
    let mut mismatched_sfids = Vec::new();
    let mut missing_account_sfids = Vec::new();
    for target in &targets {
        match active_rows.get(&target.sfid_number) {
            Some((
                name,
                full_name,
                short_name,
                category,
                province,
                city,
                town,
                province_code,
                city_code,
                town_code,
                institution_code,
                org_code,
            )) => {
                if name != &target.institution_name
                    || full_name != &target.full_name
                    || short_name != &target.short_name
                    || category != category_text(target.category)
                    || province != &target.province
                    || city != &target.city
                    || town != &target.town
                    || province_code != &target.province_code
                    || city_code != &target.city_code
                    || town_code != &target.town_code
                    || institution_code != &target.institution_code
                    || org_code != &target.org_code
                {
                    mismatched_sfids.push(target.sfid_number.clone());
                }
            }
            None => missing_sfids.push(target.sfid_number.clone()),
        }
        if counts.get(&target.sfid_number).copied().unwrap_or(0) < DEFAULT_ACCOUNT_COUNT {
            missing_account_sfids.push(target.sfid_number.clone());
        }
    }
    let obsolete_sfids = active_rows
        .keys()
        .filter(|sfid| !target_sfids.contains(*sfid))
        .cloned()
        .collect::<Vec<_>>();
    let ok = missing_sfids.is_empty()
        && mismatched_sfids.is_empty()
        && missing_account_sfids.is_empty()
        && obsolete_sfids.is_empty();
    Ok(GovDirectoryCheckReport {
        ok,
        scope_key,
        china_hash,
        catalog_hash,
        manifest_catalog_hash,
        template_version: GOV_TEMPLATE_VERSION,
        target_count: targets.len(),
        active_count: active_rows.len(),
        missing_sfids,
        mismatched_sfids,
        missing_account_sfids,
        obsolete_sfids,
    })
}

pub fn reconcile_gov_catalog_db(
    db: &Db,
    actor: &str,
    scope: OfficialReconcileScope,
    kind: GovTargetKind,
) -> Result<OfficialReconcileReport, String> {
    let targets = resolve_targets(db, &scope, kind)?;
    let china_hash = china_sqlite_hash()?;
    let catalog_hash = catalog_hash(china_hash.as_str(), &targets);
    let scope_key = scoped_manifest_key(&scope, kind);
    let mut report = write_targets(db, actor, targets, scope.clone(), kind)?;
    let check = check_gov_catalog_db(db, scope, kind)?;
    upsert_manifest(
        db,
        scope_key.as_str(),
        china_hash.as_str(),
        catalog_hash.as_str(),
        report.total_after,
        check.ok,
    )?;
    report.scope_key = scope_key;
    report.china_hash = china_hash;
    report.catalog_hash = catalog_hash;
    report.template_version = GOV_TEMPLATE_VERSION;
    Ok(report)
}

pub fn reconcile_changed_gov_catalog_db(
    db: &Db,
    actor: &str,
) -> Result<Vec<OfficialReconcileReport>, String> {
    let mut reports = Vec::new();
    for province in provinces() {
        let scope = OfficialReconcileScope::Province {
            province_code: province.code.to_string(),
        };
        let check = check_gov_catalog_db(db, scope.clone(), GovTargetKind::All)?;
        if check.ok && check.manifest_catalog_hash.is_none() {
            upsert_manifest(
                db,
                check.scope_key.as_str(),
                check.china_hash.as_str(),
                check.catalog_hash.as_str(),
                check.target_count,
                true,
            )?;
            continue;
        }
        if !check.ok || check.manifest_catalog_hash.as_deref() != Some(check.catalog_hash.as_str())
        {
            reports.push(reconcile_gov_catalog_db(
                db,
                actor,
                scope,
                GovTargetKind::All,
            )?);
        }
    }
    Ok(reports)
}

pub fn reconcile_public_security_for_province_db(
    db: &Db,
    province_name: &str,
    actor: &str,
) -> Result<ReconcileReport, String> {
    let Some(province) = provinces().iter().find(|item| item.name == province_name) else {
        return Err(format!("unknown province: {province_name}"));
    };
    let report = reconcile_gov_catalog_db(
        db,
        actor,
        OfficialReconcileScope::Province {
            province_code: province.code.to_string(),
        },
        GovTargetKind::PublicSecurity,
    )?;
    Ok(ReconcileReport {
        province: province_name.to_string(),
        inserted: report.inserted,
        updated: report.updated,
        removed: report.removed,
        total_after: report.total_after,
    })
}

fn write_targets(
    db: &Db,
    actor: &str,
    targets: Vec<OfficialInstitutionTarget>,
    scope: OfficialReconcileScope,
    kind: GovTargetKind,
) -> Result<OfficialReconcileReport, String> {
    let mut report = OfficialReconcileReport::default();
    let target_sfids = targets
        .iter()
        .map(|target| target.sfid_number.clone())
        .collect::<HashSet<_>>();
    let target_sfid_vec = target_sfids.iter().cloned().collect::<Vec<_>>();
    let existing_public_count = count_existing_public_targets(db, &target_sfid_vec)?;
    bulk_write_targets(db, actor, &targets)?;
    report.updated = existing_public_count.min(targets.len());
    report.inserted = targets.len().saturating_sub(report.updated);
    report.account_inserted = targets.len() * usize::try_from(DEFAULT_ACCOUNT_COUNT).unwrap_or(2);
    let removed = revoke_obsolete_targets(db, &target_sfids, &scope, kind)?;
    report.removed = removed.len();
    report.removed_sfids = removed;
    report.total_after = target_sfids.len();
    report.target_sfids = target_sfids.into_iter().collect();
    report.target_sfids.sort();
    report.touched_sfids = report.target_sfids.clone();
    Ok(report)
}

fn count_existing_public_targets(db: &Db, target_sfids: &[String]) -> Result<usize, String> {
    if target_sfids.is_empty() {
        return Ok(0);
    }
    let target_sfids = target_sfids.to_vec();
    db.with_client(move |conn| {
        let mut total: usize = 0;
        // 中文注释:全量公权目录接近 30 万行,统计时也按块传参,避免超大数组触发驱动/数据库错误。
        for chunk in target_sfids.chunks(10_000) {
            let chunk = chunk.to_vec();
            let row = conn
                .query_one(
                    "SELECT COUNT(*)::BIGINT
                     FROM subjects
                     WHERE kind = 'PUBLIC'
                       AND sfid_number = ANY($1)",
                    &[&chunk],
                )
                .map_err(|e| format!("count existing gov targets failed: {e}"))?;
            let count: i64 = row.get(0);
            total = total
                .checked_add(
                    usize::try_from(count)
                        .map_err(|_| "existing gov target count exceeds usize".to_string())?,
                )
                .ok_or_else(|| "existing gov target count overflows usize".to_string())?;
        }
        Ok(total)
    })
}

fn bulk_write_targets(
    db: &Db,
    actor: &str,
    targets: &[OfficialInstitutionTarget],
) -> Result<(), String> {
    if targets.is_empty() {
        return Ok(());
    }
    let actor = actor.to_string();
    let targets = targets.to_vec();
    db.with_client(move |conn| {
        let mut tx = conn
            .transaction()
            .map_err(|e| format!("begin bulk gov target write failed: {e}"))?;
        for chunk in targets.chunks(5_000) {
            bulk_write_target_chunk(&mut tx, actor.as_str(), chunk)?;
        }
        tx.commit()
            .map_err(|e| format!("commit bulk gov target write failed: {e}"))?;
        Ok(())
    })
}

fn bulk_write_target_chunk(
    tx: &mut postgres::Transaction<'_>,
    actor: &str,
    targets: &[OfficialInstitutionTarget],
) -> Result<(), String> {
    let sfids = targets
        .iter()
        .map(|target| target.sfid_number.clone())
        .collect::<Vec<_>>();
    let conflict = tx
        .query_opt(
            "SELECT sfid_number, kind
             FROM ids
             WHERE sfid_number = ANY($1)
               AND kind <> 'PUBLIC'
             LIMIT 1",
            &[&sfids],
        )
        .map_err(|e| format!("query gov target id conflict failed: {e}"))?;
    if let Some(row) = conflict {
        let sfid: String = row.get(0);
        let kind: String = row.get(1);
        return Err(format!(
            "sfid_number {sfid} already belongs to {kind}, cannot write PUBLIC"
        ));
    }

    let p_codes = targets
        .iter()
        .map(|target| target.province_code.clone())
        .collect::<Vec<_>>();
    let c_codes = targets
        .iter()
        .map(target_c_code)
        .collect::<Vec<Option<String>>>();
    let t_codes = targets
        .iter()
        .map(target_t_code)
        .collect::<Vec<Option<String>>>();
    let names = targets
        .iter()
        .map(|target| target.institution_name.clone())
        .collect::<Vec<_>>();
    let full_names = targets
        .iter()
        .map(|target| target.full_name.clone())
        .collect::<Vec<_>>();
    let short_names = targets
        .iter()
        .map(|target| target.short_name.clone())
        .collect::<Vec<_>>();
    let categories = targets
        .iter()
        .map(|target| category_text(target.category).to_string())
        .collect::<Vec<_>>();
    let subject_property_values = targets
        .iter()
        .map(|target| target.subject_property.clone())
        .collect::<Vec<_>>();
    let p1_values = targets
        .iter()
        .map(|target| target.p1.clone())
        .collect::<Vec<_>>();
    let provinces = targets
        .iter()
        .map(|target| target.province.clone())
        .collect::<Vec<_>>();
    let cities = targets
        .iter()
        .map(|target| target.city.clone())
        .collect::<Vec<_>>();
    let towns = targets
        .iter()
        .map(|target| target.town.clone())
        .collect::<Vec<_>>();
    let institution_codes = targets
        .iter()
        .map(|target| target.institution_code.clone())
        .collect::<Vec<_>>();
    let org_codes = targets
        .iter()
        .map(|target| target.org_code.clone())
        .collect::<Vec<_>>();
    let home_p_codes = vec![None::<String>; targets.len()];
    let home_c_codes = vec![None::<String>; targets.len()];

    // 中文注释:同一 sfid 如果曾因行政区划修正落在旧分区,批量清掉旧分区行。
    for table in ["subjects", "gov", "accounts"] {
        let sql = format!(
            "DELETE FROM {table} t
             USING unnest($1::text[], $2::text[]) AS u(sfid_number, p_code)
             WHERE t.sfid_number = u.sfid_number
               AND t.p_code <> u.p_code"
        );
        tx.execute(sql.as_str(), &[&sfids, &p_codes])
            .map_err(|e| format!("bulk delete {table} rows outside scope failed: {e}"))?;
    }
    tx.execute("DELETE FROM private WHERE sfid_number = ANY($1)", &[&sfids])
        .map_err(|e| format!("bulk delete private rows for gov targets failed: {e}"))?;

    tx.execute(
        "INSERT INTO ids (sfid_number, kind, p_code, c_code)
         SELECT sfid_number, 'PUBLIC', p_code, c_code
         FROM unnest($1::text[], $2::text[], $3::text[]) AS u(sfid_number, p_code, c_code)
         ON CONFLICT (sfid_number) DO UPDATE SET
            p_code = EXCLUDED.p_code,
            c_code = EXCLUDED.c_code
         WHERE ids.kind = 'PUBLIC'",
        &[&sfids, &p_codes, &c_codes],
    )
    .map_err(|e| format!("bulk upsert gov ids failed: {e}"))?;

    tx.execute(
        "INSERT INTO subjects (
            sfid_number, kind, name, full_name, short_name, p_code, c_code, t_code,
            status, category, subject_property, p1, province, city, town,
            province_code, city_code, town_code, institution_code, org_code, sub_type,
            parent_sfid_number, created_by, created_at, updated_at
         )
         SELECT
            sfid_number, 'PUBLIC', name, full_name, short_name, p_code, c_code, t_code,
            'ACTIVE', category, subject_property, p1, province, city, town,
            p_code, COALESCE(c_code, ''), COALESCE(t_code, ''), institution_code, org_code,
            NULL::text, NULL::text, $18, now(), now()
         FROM unnest(
            $1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
            $6::text[], $7::text[], $8::text[], $9::text[], $10::text[],
            $11::text[], $12::text[], $13::text[], $14::text[], $15::text[],
            $16::text[], $17::text[]
         ) AS u(
            sfid_number, name, full_name, short_name, p_code,
            c_code, t_code, category, subject_property, p1,
            province, city, town, institution_code, org_code,
            province_code, city_code
         )
         ON CONFLICT (p_code, sfid_number) DO UPDATE SET
            kind = EXCLUDED.kind,
            name = EXCLUDED.name,
            full_name = EXCLUDED.full_name,
            short_name = EXCLUDED.short_name,
            c_code = EXCLUDED.c_code,
            t_code = EXCLUDED.t_code,
            status = EXCLUDED.status,
            category = EXCLUDED.category,
            subject_property = EXCLUDED.subject_property,
            p1 = EXCLUDED.p1,
            province = EXCLUDED.province,
            city = EXCLUDED.city,
            town = EXCLUDED.town,
            province_code = EXCLUDED.province_code,
            city_code = EXCLUDED.city_code,
            town_code = EXCLUDED.town_code,
            institution_code = EXCLUDED.institution_code,
            org_code = EXCLUDED.org_code,
            sub_type = EXCLUDED.sub_type,
            parent_sfid_number = EXCLUDED.parent_sfid_number,
            created_by = EXCLUDED.created_by,
            updated_at = now()",
        &[
            &sfids,
            &names,
            &full_names,
            &short_names,
            &p_codes,
            &c_codes,
            &t_codes,
            &categories,
            &subject_property_values,
            &p1_values,
            &provinces,
            &cities,
            &towns,
            &institution_codes,
            &org_codes,
            &p_codes,
            &c_codes,
            &actor,
        ],
    )
    .map_err(|e| format!("bulk upsert gov subjects failed: {e}"))?;

    tx.execute(
        "INSERT INTO gov (
            sfid_number, p_code, c_code, t_code, institution_code, org_code,
            home_p, home_c
         )
         SELECT sfid_number, p_code, c_code, t_code, institution_code, org_code,
                home_p, home_c
         FROM unnest(
            $1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
            $6::text[], $7::text[], $8::text[]
         ) AS u(
            sfid_number, p_code, c_code, t_code, institution_code,
            org_code, home_p, home_c
         )
         ON CONFLICT (p_code, sfid_number) DO UPDATE SET
            c_code = EXCLUDED.c_code,
            t_code = EXCLUDED.t_code,
            institution_code = EXCLUDED.institution_code,
            org_code = EXCLUDED.org_code,
            home_p = EXCLUDED.home_p,
            home_c = EXCLUDED.home_c",
        &[
            &sfids,
            &p_codes,
            &c_codes,
            &t_codes,
            &institution_codes,
            &org_codes,
            &home_p_codes,
            &home_c_codes,
        ],
    )
    .map_err(|e| format!("bulk upsert gov rows failed: {e}"))?;

    let mut account_sfids = Vec::with_capacity(targets.len() * DEFAULT_ACCOUNT_COUNT as usize);
    let mut account_p_codes = Vec::with_capacity(account_sfids.capacity());
    let mut account_c_codes = Vec::with_capacity(account_sfids.capacity());
    let mut account_names = Vec::with_capacity(account_sfids.capacity());
    let mut account_addresses = Vec::with_capacity(account_sfids.capacity());
    for target in targets {
        for account in build_default_accounts(target.sfid_number.as_str(), actor) {
            account_sfids.push(target.sfid_number.clone());
            account_p_codes.push(target.province_code.clone());
            account_c_codes.push(target_c_code(target));
            account_names.push(account.account_name);
            account_addresses.push(account.duoqian_address);
        }
    }
    tx.execute(
        "INSERT INTO accounts (
            sfid_number, p_code, c_code, account_name, duoqian_address, chain_status, created_at
         )
         SELECT sfid_number, p_code, c_code, account_name, duoqian_address, 'NOT_ON_CHAIN', now()
         FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[])
              AS u(sfid_number, p_code, c_code, account_name, duoqian_address)
         ON CONFLICT (p_code, sfid_number, account_name) DO UPDATE SET
            c_code = EXCLUDED.c_code,
            duoqian_address = EXCLUDED.duoqian_address,
            chain_status = EXCLUDED.chain_status,
            created_at = EXCLUDED.created_at",
        &[
            &account_sfids,
            &account_p_codes,
            &account_c_codes,
            &account_names,
            &account_addresses,
        ],
    )
    .map_err(|e| format!("bulk upsert gov accounts failed: {e}"))?;

    Ok(())
}

fn target_c_code(target: &OfficialInstitutionTarget) -> Option<String> {
    (!target.city_code.is_empty() && target.city_code != "000").then(|| target.city_code.clone())
}

fn target_t_code(target: &OfficialInstitutionTarget) -> Option<String> {
    (!target.town_code.is_empty()).then(|| target.town_code.clone())
}

fn revoke_obsolete_targets(
    db: &Db,
    target_sfids: &HashSet<String>,
    scope: &OfficialReconcileScope,
    kind: GovTargetKind,
) -> Result<Vec<String>, String> {
    let target_sfids = target_sfids.clone();
    let scope = scope.clone();
    let category_filter = target_category_sql(kind).map(str::to_string);
    let candidates = db.with_client(move |conn| {
        let rows = conn
            .query(
                "SELECT sfid_number, category, province_code, city_code
                 FROM subjects
                 WHERE kind = 'PUBLIC'
                   AND status = 'ACTIVE'
                   AND ($1::text IS NULL OR category = $1)",
                &[&category_filter],
            )
            .map_err(|e| format!("query obsolete gov candidates failed: {e}"))?;
        let mut output = Vec::new();
        for row in rows {
            let sfid: String = row.get(0);
            if target_sfids.contains(&sfid) {
                continue;
            }
            let category: String = row.get(1);
            let province_code: String = row.get(2);
            let city_code: String = row.get(3);
            if !kind.includes(match category.as_str() {
                "PUBLIC_SECURITY" => InstitutionCategory::PublicSecurity,
                "GOV_INSTITUTION" => InstitutionCategory::GovInstitution,
                _ => continue,
            }) {
                continue;
            }
            let in_scope = match &scope {
                OfficialReconcileScope::All => true,
                OfficialReconcileScope::Province { province_code: p } => {
                    province_code.eq_ignore_ascii_case(p)
                }
                OfficialReconcileScope::City {
                    province_code: p,
                    city_code: c,
                } => province_code.eq_ignore_ascii_case(p) && city_code.eq_ignore_ascii_case(c),
            };
            if in_scope {
                output.push(sfid);
            }
        }
        Ok(output)
    })?;
    db.revoke_institution_rows_by_sfids(&candidates)?;
    Ok(candidates)
}
