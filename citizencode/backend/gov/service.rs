//! 公权机构自动目录与公安局对账服务。
//!
//! 中文注释:自动生成的公权机构只归 gov 模块维护。编译不写库,serve 只做版本守门;
//! 部署入口必须先运行 `reconcile-gov --changed-only` 与 `check-gov --strict`。

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::china::{china_sqlite_hash, provinces};
use crate::number::{generate_cid_number, GenerateCidInput, InstitutionCategory};
use crate::subjects::{
    service::{build_default_accounts_for_codes, default_account_names_for_codes},
    EDUCATION_TYPE_CITY_CITIZEN_EDU_COMMITTEE, EDUCATION_TYPE_NATIONAL_CITIZEN_EDU_COMMITTEE,
};
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

pub const GOV_TEMPLATE_VERSION: &str = "gov-deterministic-v6";
pub const MIN_DEFAULT_ACCOUNT_COUNT: i64 = 2;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ReconcileReport {
    pub province_name: String,
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
    pub target_cids: Vec<String>,
    pub touched_cids: Vec<String>,
    pub removed_cids: Vec<String>,
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
    pub missing_cids: Vec<String>,
    pub mismatched_cids: Vec<String>,
    pub missing_account_cids: Vec<String>,
    pub obsolete_cids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GovDirectoryManifestCheckReport {
    pub ok: bool,
    pub scope_key: String,
    pub china_hash: String,
    pub catalog_hash: String,
    pub target_count: usize,
    pub manifest_china_hash: Option<String>,
    pub manifest_catalog_hash: Option<String>,
    pub manifest_template_version: Option<String>,
    pub manifest_status: Option<String>,
    pub manifest_target_count: Option<i64>,
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
    cid_number: String,
    cid_full_name: String,
    cid_short_name: String,
    category: InstitutionCategory,
    subject_property: String,
    p1: String,
    province_name: String,
    city_name: String,
    town_name: String,
    province_code: String,
    city_code: String,
    town_code: String,
    institution_code: String,
    org_code: String,
    education_type: Option<String>,
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
        push_constant_target(&mut targets, item.cid_full_name, item.cid_number);
    }
    for item in china_lf_constants::CHINA_LF.iter() {
        push_constant_target(&mut targets, item.cid_full_name, item.cid_number);
    }
    for item in china_sf_constants::CHINA_SF.iter() {
        push_constant_target(&mut targets, item.cid_full_name, item.cid_number);
    }
    for item in china_jc_constants::CHINA_JC.iter() {
        push_constant_target(&mut targets, item.cid_full_name, item.cid_number);
    }
    for item in china_jy_constants::CHINA_JY.iter() {
        push_constant_target(&mut targets, "国家教育委员会", item.cid_number);
    }
    for item in china_cb_constants::CHINA_CB.iter() {
        push_constant_target(&mut targets, item.cid_full_name, item.cid_number);
    }
    for item in china_ch_constants::CHINA_CH.iter() {
        push_constant_target(&mut targets, item.cid_full_name, item.cid_number);
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
            let Some(cid_number) =
                generate_official_template_cid(&seed, province.name, city.name, "ZF")
            else {
                continue;
            };
            targets.push(OfficialInstitutionTarget {
                cid_number,
                cid_full_name: format!("{}公民安全局", city.name),
                cid_short_name: format!("{}公安局", city.name),
                category: InstitutionCategory::PublicSecurity,
                subject_property: "G".to_string(),
                p1: "0".to_string(),
                province_name: province.name.to_string(),
                city_name: city.name.to_string(),
                town_name: String::new(),
                province_code: province.code.to_string(),
                city_code: city.code.to_string(),
                town_code: String::new(),
                institution_code: "ZF".to_string(),
                org_code: "CITY_POLICE".to_string(),
                education_type: None,
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
    cid_number: &'static str,
) {
    let Some((subject_property, province_code, city_code, institution_code, p1)) =
        parse_cid_institution_parts(cid_number)
    else {
        return;
    };
    let Some((province, city)) = province_city_by_codes(&province_code, &city_code) else {
        return;
    };
    let (cid_full_name, cid_short_name) = official_name_pair(name);
    let org_code = org_code_for_constant_name(name);
    let education_type = (org_code == "NATIONAL_EDU")
        .then(|| EDUCATION_TYPE_NATIONAL_CITIZEN_EDU_COMMITTEE.to_string());
    targets.push(OfficialInstitutionTarget {
        cid_number: cid_number.to_string(),
        cid_full_name,
        cid_short_name,
        category: InstitutionCategory::GovInstitution,
        subject_property,
        p1,
        province_name: province.to_string(),
        city_name: city.to_string(),
        town_name: String::new(),
        province_code,
        city_code,
        town_code: String::new(),
        institution_code,
        org_code: org_code.to_string(),
        education_type,
    });
}

/// 联邦注册局是全国唯一机构,cid_number 取自创世常量 china_zf.rs(总统府联邦注册局)。
/// 注意:该机构经 push_constant_target 落库时,org_code_for_constant_name 未单列其名,
/// 故 org_code 实为 PUBLIC_ORG;只读接口因此按 cid_number 直接定位,不依赖 org_code。
pub fn federal_registry_cid_number() -> Option<&'static str> {
    china_zf_constants::CHINA_ZF
        .iter()
        .find(|item| item.cid_full_name == "总统府联邦注册局")
        .map(|item| item.cid_number)
}

/// 中文注释:联邦注册局(全国唯一)内置管理员公钥集,取自创世常量 china_zf.rs。
/// 唯一真源是链上 admins-change::AdminAccounts;本取值器只服务 CID 侧的【重新创世后/
/// 链不可达】止血播种(admins::seed),稳态由 chain_sync 从链投影接管,见 ADR-023。
/// 仅取「总统府联邦注册局」单条的 admins,不会混入其它联邦机构(安全局/情报局等)的管理员。
pub(crate) fn federal_registry_admins() -> Option<&'static [[u8; 32]]> {
    china_zf_constants::CHINA_ZF
        .iter()
        .find(|item| item.cid_full_name == "总统府联邦注册局")
        .map(|item| item.admins)
}

fn official_name_pair(name: &str) -> (String, String) {
    const COUNTRY: &str = "中华民族联邦共和国";
    match name {
        "总统府" | "中华民族联邦共和国总统府" => {
            (format!("{COUNTRY}总统府"), "总统府".to_string())
        }
        "外交部" | "中华民族联邦共和国外事交流部" => {
            (format!("{COUNTRY}外事交流部"), "外交部".to_string())
        }
        "国防部" | "中华民族联邦共和国国家防务部" => {
            (format!("{COUNTRY}国家防务部"), "国防部".to_string())
        }
        "国安部" | "中华民族联邦共和国国土安全部" => {
            (format!("{COUNTRY}国土安全部"), "国安部".to_string())
        }
        "民生部" | "中华民族联邦共和国公民生活保障部" => {
            (format!("{COUNTRY}公民生活保障部"), "民生部".to_string())
        }
        "住建部" | "中华民族联邦共和国住房与城镇建设部" => {
            (format!("{COUNTRY}住房与城镇建设部"), "住建部".to_string())
        }
        "农业部" | "中华民族联邦共和国农业与农村发展部" => {
            (format!("{COUNTRY}农业与农村发展部"), "农业部".to_string())
        }
        "商贸部" | "中华民族联邦共和国商务与市场贸易部" => {
            (format!("{COUNTRY}商务与市场贸易部"), "商贸部".to_string())
        }
        "财税部" | "中华民族联邦共和国财政与税务部" => {
            (format!("{COUNTRY}财政与税务部"), "财税部".to_string())
        }
        "能源部" | "中华民族联邦共和国能源与环保发展部" => {
            (format!("{COUNTRY}能源与环保发展部"), "能源部".to_string())
        }
        "交通部" | "中华民族联邦共和国交通运输部" => {
            (format!("{COUNTRY}交通运输部"), "交通部".to_string())
        }
        "国家立法院" | "中华民族联邦共和国国家立法院" => {
            (format!("{COUNTRY}国家立法院"), "国家立法院".to_string())
        }
        "国家司法院" | "中华民族联邦共和国国家司法院" => {
            (format!("{COUNTRY}国家司法院"), "国家司法院".to_string())
        }
        "国家监察院" | "中华民族联邦共和国国家监察院" => {
            (format!("{COUNTRY}国家监察院"), "国家监察院".to_string())
        }
        "国家教育委员会" | "中华民族联邦共和国公民教育委员会" => {
            (format!("{COUNTRY}公民教育委员会"), "国教委会".to_string())
        }
        "国家公民储备委员会" => ("国家公民储备委员会".to_string(), "国储会".to_string()),
        "联邦廉政署" | "国家监察院联邦廉政署" => (
            format!("{COUNTRY}国家监察院联邦廉政署"),
            "联邦廉政署".to_string(),
        ),
        "联邦审计署" | "国家监察院联邦审计署" => (
            format!("{COUNTRY}国家监察院联邦审计署"),
            "联邦审计署".to_string(),
        ),
        "联邦调查署" | "国家监察院联邦调查署" => (
            format!("{COUNTRY}国家监察院联邦调查署"),
            "联邦调查署".to_string(),
        ),
        _ if name.ends_with("省联邦政府") => {
            (name.to_string(), name.replace("省联邦政府", "省政府"))
        }
        _ if name.ends_with("省公民储备委员会") => {
            (name.to_string(), name.replace("省公民储备委员会", "省储会"))
        }
        _ if name.ends_with("省公民储备银行") => {
            (name.to_string(), name.replace("省公民储备银行", "省储行"))
        }
        _ => (name.to_string(), name.to_string()),
    }
}

// 中文注释:常量机构(创世 china_zf/lf/sf/jc/cb/ch/jy)按"机构全名"推导细类 org_code。
// name 即各常量的 cid_full_name(china_jy 例外:循环里传字面 "国家教育委员会")。
// 历史踩坑:旧版用简称("住建部"/"省政府")匹配,但常量存的是全名("…住房与城镇建设部"/"…省联邦政府"),
// 简称对不上全名 → 国家级与部分省级全落 PUBLIC_ORG。此处改为按实际全名 + 现名后缀匹配。
fn org_code_for_constant_name(name: &str) -> &'static str {
    match name {
        // ── 总统府 + 10 部委(china_zf,全名) ──
        "中华民族联邦共和国总统府" => "NATIONAL_PRESIDENT_OFFICE",
        "中华民族联邦共和国外事交流部" => "MINISTRY_FOREIGN",
        "中华民族联邦共和国国家防务部" => "MINISTRY_DEFENSE",
        "中华民族联邦共和国国土安全部" => "MINISTRY_SECURITY",
        "中华民族联邦共和国公民生活保障部" => "MINISTRY_CIVIL_LIFE",
        "中华民族联邦共和国住房与城镇建设部" => "MINISTRY_HOUSING",
        "中华民族联邦共和国农业与农村发展部" => "MINISTRY_AGRICULTURE",
        "中华民族联邦共和国商务与市场贸易部" => "MINISTRY_COMMERCE",
        "中华民族联邦共和国财政与税务部" => "MINISTRY_FINANCE_TAX",
        "中华民族联邦共和国能源与环保发展部" => "MINISTRY_ENERGY",
        "中华民族联邦共和国交通运输部" => "MINISTRY_TRANSPORT",
        // ── 5 个总统府联邦局(china_zf) ──
        "总统府联邦安全局" => "FEDERAL_SECURITY",
        "总统府联邦情报局" => "FEDERAL_INTELLIGENCE",
        "总统府联邦特勤局" => "FEDERAL_SPECIAL_SERVICE",
        "总统府联邦人事局" => "FEDERAL_PERSONNEL",
        "总统府联邦注册局" => "FEDERAL_REGISTRY",
        // ── 国家两院 + 监察院下属联邦署(china_lf/sf/jc) ──
        "中华民族联邦共和国国家立法院" => "NATIONAL_LEGISLATURE",
        "中华民族联邦共和国国家司法院" => "NATIONAL_COURT",
        "中华民族联邦共和国国家监察院" => "NATIONAL_SUPERVISION",
        "国家监察院联邦廉政署" => "FEDERAL_INTEGRITY",
        "国家监察院联邦审计署" => "FEDERAL_AUDIT",
        "国家监察院联邦调查署" => "FEDERAL_INVESTIGATION",
        // ── 国家公民储备委员会(china_cb) / 国家公民教育委员会(china_jy 传字面) ──
        "国家公民储备委员会" => "NATIONAL_RESERVE",
        "国家教育委员会" | "公民教育委员会" => "NATIONAL_EDU",
        // ── 省级:按现名后缀匹配(省联邦政府/省公民储备委员会/省公民储备银行) ──
        _ if name.ends_with("省联邦政府") => "PROVINCE_GOV",
        _ if name.ends_with("省立法院") => "PROVINCE_LEGISLATURE",
        _ if name.ends_with("省司法院") => "PROVINCE_COURT",
        _ if name.ends_with("省监察院") => "PROVINCE_SUPERVISION",
        _ if name.ends_with("省公民储备委员会") => "PROVINCE_RESERVE",
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
    // 中文注释:5 个总统府联邦局(安全/情报/特勤/人事/注册)已作为创世常量收录于
    // china_zf.rs CHINA_ZF(带 main/fee/admins),由 :375 的常量循环单一 push;
    // 此处不再用区划模板重复生成,避免同号双定义触发 reconcile 21000。仅保留两院议会。
    for (cid_short_name, cid_full_name, org_code) in [
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
            institution_code: if cid_short_name.ends_with("议会") {
                "LF"
            } else {
                "ZF"
            },
            org_code,
            suffix: cid_short_name,
            full_suffix: cid_full_name,
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
    let cid_short_name = format!("{display_area_name}{}", template.suffix);
    let cid_full_name = format!("{display_area_name}{}", template.full_suffix);
    let account_seed = format!(
        "GOV-{seed_scope}-{province_code}-{city_code}-{town_code}-{}-{}",
        template.institution_code, template.org_code
    );
    let Some(cid_number) = generate_official_template_cid(
        &account_seed,
        province_name,
        city_name,
        template.institution_code,
    ) else {
        return;
    };
    targets.push(OfficialInstitutionTarget {
        cid_number,
        cid_full_name,
        cid_short_name,
        category: InstitutionCategory::GovInstitution,
        subject_property: "G".to_string(),
        p1: "0".to_string(),
        province_name: province_name.to_string(),
        city_name: city_name.to_string(),
        town_name: town_name.to_string(),
        province_code: province_code.to_string(),
        city_code: city_code.to_string(),
        town_code: town_code.to_string(),
        institution_code: template.institution_code.to_string(),
        org_code: template.org_code.to_string(),
        education_type: (template.org_code == "CITY_EDU")
            .then(|| EDUCATION_TYPE_CITY_CITIZEN_EDU_COMMITTEE.to_string()),
    });
}

fn generate_official_template_cid(
    account_seed: &str,
    province_name: &str,
    city_name: &str,
    institution_code: &str,
) -> Option<String> {
    generate_cid_number(GenerateCidInput {
        account_pubkey: account_seed,
        subject_property: "G",
        p1: "0",
        province_name,
        city_name,
        institution: institution_code,
    })
    .ok()
}

fn parse_cid_institution_parts(
    cid_number: &str,
) -> Option<(String, String, String, String, String)> {
    let mut segments = cid_number.split('-');
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
            a.cid_number.as_str(),
        )
            .cmp(&(
                b.province_code.as_str(),
                b.city_code.as_str(),
                b.town_code.as_str(),
                category_text(b.category),
                b.institution_code.as_str(),
                b.org_code.as_str(),
                b.cid_number.as_str(),
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
        hasher.update(target.cid_number.as_bytes());
        hasher.update(b"|");
        hasher.update(target.cid_full_name.as_bytes());
        hasher.update(b"|");
        hasher.update(target.cid_short_name.as_bytes());
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
        hasher.update(b"|");
        hasher.update(target.education_type.as_deref().unwrap_or("").as_bytes());
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

pub fn check_gov_manifest_db(db: &Db) -> Result<GovDirectoryManifestCheckReport, String> {
    let scope = OfficialReconcileScope::All;
    let kind = GovTargetKind::All;
    let targets = resolve_targets(db, &scope, kind)?;
    let china_hash = china_sqlite_hash()?;
    let catalog_hash = catalog_hash(china_hash.as_str(), &targets);
    let target_count = targets.len();
    let scope_key = scoped_manifest_key(&scope, kind);
    let manifest = {
        let scope_key = scope_key.clone();
        db.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT china_hash, catalog_hash, template_version, status, target_count
                     FROM gov_manifest
                     WHERE scope_key = $1
                     ORDER BY updated_at DESC
                     LIMIT 1",
                    &[&scope_key],
                )
                .map_err(|e| {
                    format!(
                        "query gov manifest current state failed: {}",
                        crate::core::db::postgres_error_text(&e)
                    )
                })?;
            Ok(row.map(|row| {
                (
                    row.get::<_, String>(0),
                    row.get::<_, String>(1),
                    row.get::<_, String>(2),
                    row.get::<_, String>(3),
                    row.get::<_, i64>(4),
                )
            }))
        })?
    };
    let (
        manifest_china_hash,
        manifest_catalog_hash,
        manifest_template_version,
        manifest_status,
        manifest_target_count,
    ) = match manifest {
        Some((china, catalog, template, status, count)) => (
            Some(china),
            Some(catalog),
            Some(template),
            Some(status),
            Some(count),
        ),
        None => (None, None, None, None, None),
    };
    let target_count_i64 =
        i64::try_from(target_count).map_err(|_| "gov target count exceeds i64".to_string())?;
    let ok = manifest_china_hash.as_deref() == Some(china_hash.as_str())
        && manifest_catalog_hash.as_deref() == Some(catalog_hash.as_str())
        && manifest_template_version.as_deref() == Some(GOV_TEMPLATE_VERSION)
        && manifest_status.as_deref() == Some("OK")
        && manifest_target_count == Some(target_count_i64);
    Ok(GovDirectoryManifestCheckReport {
        ok,
        scope_key,
        china_hash,
        catalog_hash,
        target_count,
        manifest_china_hash,
        manifest_catalog_hash,
        manifest_template_version,
        manifest_status,
        manifest_target_count,
    })
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
                "SELECT s.cid_number, COALESCE(s.name, ''), COALESCE(s.cid_full_name, ''),
                        COALESCE(s.cid_short_name, ''), s.category, s.province_name, s.city_name,
                        COALESCE(s.town_name, ''), s.province_code, s.city_code,
                        COALESCE(s.town_code, ''), s.institution_code, COALESCE(g.org_code, ''),
                        COALESCE(s.education_type, '')
                 FROM subjects s
                 JOIN gov g ON g.province_code = s.province_code AND g.cid_number = s.cid_number
                 WHERE s.kind = 'PUBLIC'
                   AND s.status = 'ACTIVE'
                   AND g.source = 'GENERATED'
                   AND ($1::text IS NULL OR s.province_code = $1)
                   AND ($2::text IS NULL OR s.city_code = $2)
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
                    row.get(13),
                ),
            );
        }
        Ok(output)
    })
}

fn account_counts(db: &Db, cids: &[String]) -> Result<BTreeMap<String, i64>, String> {
    if cids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let cids = cids.to_vec();
    db.with_client(move |conn| {
        let mut output = BTreeMap::new();
        // 中文注释:全量镇目录接近 30 万机构,账户校验按块查,避免超大数组压垮单条 SQL。
        for chunk in cids.chunks(10_000) {
            let chunk = chunk.to_vec();
            let rows = conn
                .query(
                    "SELECT cid_number, COUNT(*)::BIGINT
                     FROM accounts
                     WHERE cid_number = ANY($1)
                     GROUP BY cid_number",
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
    let target_cids = targets
        .iter()
        .map(|target| target.cid_number.clone())
        .collect::<BTreeSet<_>>();
    let counts = account_counts(db, &target_cids.iter().cloned().collect::<Vec<_>>())?;

    let mut missing_cids = Vec::new();
    let mut mismatched_cids = Vec::new();
    let mut missing_account_cids = Vec::new();
    for target in &targets {
        match active_rows.get(&target.cid_number) {
            Some((
                name,
                cid_full_name,
                cid_short_name,
                category,
                province,
                city,
                town,
                province_code,
                city_code,
                town_code,
                institution_code,
                org_code,
                education_type,
            )) => {
                if name != &target.cid_short_name
                    || cid_full_name != &target.cid_full_name
                    || cid_short_name != &target.cid_short_name
                    || category != category_text(target.category)
                    || province != &target.province_name
                    || city != &target.city_name
                    || town != &target.town_name
                    || province_code != &target.province_code
                    || city_code != &target.city_code
                    || town_code != &target.town_code
                    || institution_code != &target.institution_code
                    || org_code != &target.org_code
                    || education_type != target.education_type.as_deref().unwrap_or("")
                {
                    mismatched_cids.push(target.cid_number.clone());
                }
            }
            None => missing_cids.push(target.cid_number.clone()),
        }
        let expected_count = i64::try_from(default_account_names_for_target(target).len())
            .map_err(|_| "default account count exceeds i64".to_string())?;
        if counts.get(&target.cid_number).copied().unwrap_or(0) < expected_count {
            missing_account_cids.push(target.cid_number.clone());
        }
    }
    let obsolete_cids = active_rows
        .keys()
        .filter(|cid| !target_cids.contains(*cid))
        .cloned()
        .collect::<Vec<_>>();
    let ok = missing_cids.is_empty()
        && mismatched_cids.is_empty()
        && missing_account_cids.is_empty()
        && obsolete_cids.is_empty();
    Ok(GovDirectoryCheckReport {
        ok,
        scope_key,
        china_hash,
        catalog_hash,
        manifest_catalog_hash,
        template_version: GOV_TEMPLATE_VERSION,
        target_count: targets.len(),
        active_count: active_rows.len(),
        missing_cids,
        mismatched_cids,
        missing_account_cids,
        obsolete_cids,
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
    // 中文注释:changed-only 以省为单位减少写库范围,但部署守门看的是全局
    // all:all manifest。省级对账完成后必须刷新全局版本,否则 strict 会误判目录过期。
    let all_check = check_gov_catalog_db(db, OfficialReconcileScope::All, GovTargetKind::All)?;
    if all_check.ok {
        upsert_gov_manifest_from_check_db(db, &all_check)?;
    } else {
        reports.push(reconcile_gov_catalog_db(
            db,
            actor,
            OfficialReconcileScope::All,
            GovTargetKind::All,
        )?);
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
        province_name: province_name.to_string(),
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
    let target_cids = targets
        .iter()
        .map(|target| target.cid_number.clone())
        .collect::<HashSet<_>>();
    let target_cid_vec = target_cids.iter().cloned().collect::<Vec<_>>();
    let existing_public_count = count_existing_public_targets(db, &target_cid_vec)?;
    bulk_write_targets(db, actor, &targets)?;
    report.updated = existing_public_count.min(targets.len());
    report.inserted = targets.len().saturating_sub(report.updated);
    report.account_inserted = targets
        .iter()
        .map(|target| default_account_names_for_target(target).len())
        .sum::<usize>();
    let removed = revoke_obsolete_targets(db, &target_cids, &scope, kind)?;
    report.removed = removed.len();
    report.removed_cids = removed;
    report.total_after = target_cids.len();
    report.target_cids = target_cids.into_iter().collect();
    report.target_cids.sort();
    report.touched_cids = report.target_cids.clone();
    Ok(report)
}

fn count_existing_public_targets(db: &Db, target_cids: &[String]) -> Result<usize, String> {
    if target_cids.is_empty() {
        return Ok(0);
    }
    let target_cids = target_cids.to_vec();
    db.with_client(move |conn| {
        let mut total: usize = 0;
        // 中文注释:全量公权目录接近 30 万行,统计时也按块传参,避免超大数组触发驱动/数据库错误。
        for chunk in target_cids.chunks(10_000) {
            let chunk = chunk.to_vec();
            let row = conn
                .query_one(
                    "SELECT COUNT(*)::BIGINT
                     FROM subjects
                     WHERE kind = 'PUBLIC'
                       AND cid_number = ANY($1)",
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
    // 中文注释:号生成若在同一批 targets 内产生重复 cid_number(确定性 N9 碰撞或重复目标),
    // 后续 bulk upsert 会以 21000 cardinality_violation 报错且不带定位信息。这里提前全量探测,
    // 带出碰撞双方机构信息,便于判断是重复目标(同 seed)还是 N9 哈希碰撞(不同 seed)。
    {
        let mut seen: std::collections::HashMap<&str, &OfficialInstitutionTarget> =
            std::collections::HashMap::new();
        let mut collisions: Vec<String> = Vec::new();
        for target in targets {
            if let Some(prev) = seen.insert(target.cid_number.as_str(), target) {
                collisions.push(format!(
                    "{}: [{} | {}{} | inst={} org={}] vs [{} | {}{} | inst={} org={}]",
                    target.cid_number,
                    prev.cid_full_name,
                    prev.city_name,
                    prev.town_name,
                    prev.institution_code,
                    prev.org_code,
                    target.cid_full_name,
                    target.city_name,
                    target.town_name,
                    target.institution_code,
                    target.org_code,
                ));
            }
        }
        if !collisions.is_empty() {
            let shown = collisions
                .iter()
                .take(20)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n  ");
            return Err(format!(
                "gov reconcile produced {} duplicate cid_number(s) among {} targets; first {}:\n  {}",
                collisions.len(),
                targets.len(),
                collisions.len().min(20),
                shown
            ));
        }
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
    let cids = targets
        .iter()
        .map(|target| target.cid_number.clone())
        .collect::<Vec<_>>();
    let conflict = tx
        .query_opt(
            "SELECT cid_number, kind
             FROM ids
             WHERE cid_number = ANY($1)
               AND kind <> 'PUBLIC'
             LIMIT 1",
            &[&cids],
        )
        .map_err(|e| format!("query gov target id conflict failed: {e}"))?;
    if let Some(row) = conflict {
        let cid: String = row.get(0);
        let kind: String = row.get(1);
        return Err(format!(
            "cid_number {cid} already belongs to {kind}, cannot write PUBLIC"
        ));
    }

    let province_codes = targets
        .iter()
        .map(|target| target.province_code.clone())
        .collect::<Vec<_>>();
    let city_codes = targets
        .iter()
        .map(target_city_code)
        .collect::<Vec<Option<String>>>();
    let town_codes = targets
        .iter()
        .map(target_town_code)
        .collect::<Vec<Option<String>>>();
    let names = targets
        .iter()
        .map(|target| target.cid_short_name.clone())
        .collect::<Vec<_>>();
    let cid_names = targets
        .iter()
        .map(|target| target.cid_full_name.clone())
        .collect::<Vec<_>>();
    let short_names = targets
        .iter()
        .map(|target| target.cid_short_name.clone())
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
    let province_names = targets
        .iter()
        .map(|target| target.province_name.clone())
        .collect::<Vec<_>>();
    let city_names = targets
        .iter()
        .map(|target| target.city_name.clone())
        .collect::<Vec<_>>();
    let town_names = targets
        .iter()
        .map(|target| target.town_name.clone())
        .collect::<Vec<_>>();
    let institution_codes = targets
        .iter()
        .map(|target| target.institution_code.clone())
        .collect::<Vec<_>>();
    let org_codes = targets
        .iter()
        .map(|target| target.org_code.clone())
        .collect::<Vec<_>>();
    let education_types = targets
        .iter()
        .map(|target| target.education_type.clone())
        .collect::<Vec<_>>();
    let home_province_codes = vec![None::<String>; targets.len()];
    let home_city_codes = vec![None::<String>; targets.len()];

    // 中文注释:同一 cid 如果曾因行政区划修正落在旧分区,批量清掉旧分区行。
    for table in ["subjects", "gov", "accounts"] {
        let sql = format!(
            "DELETE FROM {table} t
             USING unnest($1::text[], $2::text[]) AS u(cid_number, province_code)
             WHERE t.cid_number = u.cid_number
               AND t.province_code <> u.province_code"
        );
        tx.execute(sql.as_str(), &[&cids, &province_codes])
            .map_err(|e| {
                format!(
                    "bulk delete {table} rows outside scope failed: {}",
                    crate::core::db::postgres_error_text(&e)
                )
            })?;
    }
    tx.execute("DELETE FROM private WHERE cid_number = ANY($1)", &[&cids])
        .map_err(|e| {
            format!(
                "bulk delete private rows for gov targets failed: {}",
                crate::core::db::postgres_error_text(&e)
            )
        })?;

    tx.execute(
        "INSERT INTO ids (cid_number, kind, province_code, city_code)
         SELECT cid_number, 'PUBLIC', province_code, city_code
         FROM unnest($1::text[], $2::text[], $3::text[]) AS u(cid_number, province_code, city_code)
         ON CONFLICT (cid_number) DO UPDATE SET
            province_code = EXCLUDED.province_code,
            city_code = EXCLUDED.city_code
         WHERE ids.kind = 'PUBLIC'",
        &[&cids, &province_codes, &city_codes],
    )
    .map_err(|e| {
        format!(
            "bulk upsert gov ids failed: {}",
            crate::core::db::postgres_error_text(&e)
        )
    })?;

    tx.execute(
        "INSERT INTO subjects (
            cid_number, kind, name, cid_full_name, cid_short_name,
            status, category, subject_property, p1, province_name, city_name, town_name,
            province_code, city_code, town_code, institution_code, org_code,
            education_type, private_type, partnership_kind, has_legal_personality,
            parent_cid_number, created_by, created_at, updated_at
         )
         SELECT
            cid_number, 'PUBLIC', name, cid_full_name, cid_short_name,
            'ACTIVE', category, subject_property, p1, province_name, city_name, town_name,
            province_code, COALESCE(city_code, ''), COALESCE(town_code, ''), institution_code, org_code,
            education_type, NULL::text, NULL::text, NULL::boolean, NULL::text, $17, now(), now()
         FROM unnest(
            $1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
            $6::text[], $7::text[], $8::text[], $9::text[], $10::text[],
            $11::text[], $12::text[], $13::text[], $14::text[], $15::text[],
            $16::text[]
         ) AS u(
            cid_number, name, cid_full_name, cid_short_name, category,
            subject_property, p1, province_name, city_name, town_name,
            institution_code, org_code, province_code, city_code, town_code,
            education_type
         )
         ON CONFLICT (province_code, cid_number) DO UPDATE SET
            kind = EXCLUDED.kind,
            name = EXCLUDED.name,
            cid_full_name = EXCLUDED.cid_full_name,
            cid_short_name = EXCLUDED.cid_short_name,
            status = EXCLUDED.status,
            category = EXCLUDED.category,
            subject_property = EXCLUDED.subject_property,
            p1 = EXCLUDED.p1,
            province_name = EXCLUDED.province_name,
            city_name = EXCLUDED.city_name,
            town_name = EXCLUDED.town_name,
            province_code = EXCLUDED.province_code,
            city_code = EXCLUDED.city_code,
            town_code = EXCLUDED.town_code,
            institution_code = EXCLUDED.institution_code,
            org_code = EXCLUDED.org_code,
            education_type = EXCLUDED.education_type,
            private_type = EXCLUDED.private_type,
            partnership_kind = EXCLUDED.partnership_kind,
            has_legal_personality = EXCLUDED.has_legal_personality,
            parent_cid_number = EXCLUDED.parent_cid_number,
            created_by = EXCLUDED.created_by,
            updated_at = now()",
        &[
            &cids,
            &names,
            &cid_names,
            &short_names,
            &categories,
            &subject_property_values,
            &p1_values,
            &province_names,
            &city_names,
            &town_names,
            &institution_codes,
            &org_codes,
            &province_codes,
            &city_codes,
            &town_codes,
            &education_types,
            &actor,
        ],
    )
    .map_err(|e| {
        format!(
            "bulk upsert gov subjects failed: {}",
            crate::core::db::postgres_error_text(&e)
        )
    })?;

    tx.execute(
        "INSERT INTO gov (
            cid_number, province_code, city_code, town_code, institution_code, org_code,
            source, home_p, home_c
         )
         SELECT cid_number, province_code, city_code, town_code, institution_code, org_code,
                'GENERATED',
                home_p, home_c
         FROM unnest(
            $1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
            $6::text[], $7::text[], $8::text[]
         ) AS u(
            cid_number, province_code, city_code, town_code, institution_code,
            org_code, home_p, home_c
         )
         ON CONFLICT (province_code, cid_number) DO UPDATE SET
            city_code = EXCLUDED.city_code,
            town_code = EXCLUDED.town_code,
            institution_code = EXCLUDED.institution_code,
            org_code = EXCLUDED.org_code,
            source = EXCLUDED.source,
            home_p = EXCLUDED.home_p,
            home_c = EXCLUDED.home_c",
        &[
            &cids,
            &province_codes,
            &city_codes,
            &town_codes,
            &institution_codes,
            &org_codes,
            &home_province_codes,
            &home_city_codes,
        ],
    )
    .map_err(|e| {
        format!(
            "bulk upsert gov rows failed: {}",
            crate::core::db::postgres_error_text(&e)
        )
    })?;

    let default_account_total = targets
        .iter()
        .map(|target| default_account_names_for_target(target).len())
        .sum::<usize>();
    let mut account_cids = Vec::with_capacity(default_account_total);
    let mut account_p_codes = Vec::with_capacity(account_cids.capacity());
    let mut account_c_codes = Vec::with_capacity(account_cids.capacity());
    let mut account_names = Vec::with_capacity(account_cids.capacity());
    let mut account_addresses = Vec::with_capacity(account_cids.capacity());
    for target in targets {
        for account in build_default_accounts_for_codes(
            target.cid_number.as_str(),
            actor,
            target.institution_code.as_str(),
            Some(target.org_code.as_str()),
        ) {
            account_cids.push(target.cid_number.clone());
            account_p_codes.push(target.province_code.clone());
            account_c_codes.push(target_city_code(target));
            account_names.push(account.account_name);
            account_addresses.push(account.account);
        }
    }
    tx.execute(
        "INSERT INTO accounts (
            cid_number, province_code, city_code, account_name, account, chain_status, created_at
         )
         SELECT cid_number, province_code, city_code, account_name, account, 'NOT_ON_CHAIN', now()
         FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[])
              AS u(cid_number, province_code, city_code, account_name, account)
         ON CONFLICT (province_code, cid_number, account_name) DO UPDATE SET
            city_code = EXCLUDED.city_code,
            account = EXCLUDED.account,
            chain_status = EXCLUDED.chain_status,
            created_at = EXCLUDED.created_at",
        &[
            &account_cids,
            &account_p_codes,
            &account_c_codes,
            &account_names,
            &account_addresses,
        ],
    )
    .map_err(|e| {
        format!(
            "bulk upsert gov accounts failed: {}",
            crate::core::db::postgres_error_text(&e)
        )
    })?;

    Ok(())
}

fn default_account_names_for_target(target: &OfficialInstitutionTarget) -> &'static [&'static str] {
    default_account_names_for_codes(
        target.institution_code.as_str(),
        Some(target.org_code.as_str()),
    )
}

fn target_city_code(target: &OfficialInstitutionTarget) -> Option<String> {
    (!target.city_code.is_empty() && target.city_code != "000").then(|| target.city_code.clone())
}

fn target_town_code(target: &OfficialInstitutionTarget) -> Option<String> {
    (!target.town_code.is_empty()).then(|| target.town_code.clone())
}

fn revoke_obsolete_targets(
    db: &Db,
    target_cids: &HashSet<String>,
    scope: &OfficialReconcileScope,
    kind: GovTargetKind,
) -> Result<Vec<String>, String> {
    let target_cids = target_cids.clone();
    let scope = scope.clone();
    let category_filter = target_category_sql(kind).map(str::to_string);
    let candidates = db.with_client(move |conn| {
        let rows = conn
            .query(
                "SELECT s.cid_number, s.category, s.province_code, s.city_code
                 FROM subjects s
                 JOIN gov g ON g.province_code = s.province_code AND g.cid_number = s.cid_number
                 WHERE s.kind = 'PUBLIC'
                   AND g.source = 'GENERATED'
                   AND ($1::text IS NULL OR s.category = $1)",
                &[&category_filter],
            )
            .map_err(|e| format!("query obsolete gov candidates failed: {e}"))?;
        let mut output = Vec::new();
        for row in rows {
            let cid: String = row.get(0);
            if target_cids.contains(&cid) {
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
                output.push(cid);
            }
        }
        Ok(output)
    })?;
    delete_obsolete_generated_targets(db, &candidates)?;
    Ok(candidates)
}

fn delete_obsolete_generated_targets(db: &Db, cids: &[String]) -> Result<(), String> {
    if cids.is_empty() {
        return Ok(());
    }
    let cids = cids.to_vec();
    db.with_client(move |conn| {
        let mut tx = conn
            .transaction()
            .map_err(|e| format!("begin obsolete generated gov cleanup failed: {e}"))?;
        for chunk in cids.chunks(10_000) {
            let chunk = chunk.to_vec();
            // 中文注释:obsolete 只来自 gov.source=GENERATED 的确定性目录。行政区 code
            // 删除/合并后,这些行不再是目标目录的一部分,必须连同账户和索引一起清掉。
            tx.execute("DELETE FROM accounts WHERE cid_number = ANY($1)", &[&chunk])
                .map_err(|e| format!("delete obsolete generated gov accounts failed: {e}"))?;
            tx.execute("DELETE FROM docs WHERE cid_number = ANY($1)", &[&chunk])
                .map_err(|e| format!("delete obsolete generated gov docs failed: {e}"))?;
            tx.execute("DELETE FROM audit WHERE target_cid = ANY($1)", &[&chunk])
                .map_err(|e| format!("delete obsolete generated gov audit failed: {e}"))?;
            tx.execute(
                "DELETE FROM gov
                 WHERE cid_number = ANY($1)
                   AND source = 'GENERATED'",
                &[&chunk],
            )
            .map_err(|e| format!("delete obsolete generated gov rows failed: {e}"))?;
            tx.execute("DELETE FROM private WHERE cid_number = ANY($1)", &[&chunk])
                .map_err(|e| format!("delete obsolete generated private residuals failed: {e}"))?;
            tx.execute(
                "DELETE FROM ids
                 WHERE cid_number = ANY($1)
                   AND kind = 'PUBLIC'",
                &[&chunk],
            )
            .map_err(|e| format!("delete obsolete generated gov ids failed: {e}"))?;
            tx.execute(
                "DELETE FROM subjects
                 WHERE cid_number = ANY($1)
                   AND kind = 'PUBLIC'",
                &[&chunk],
            )
            .map_err(|e| format!("delete obsolete generated gov subjects failed: {e}"))?;
        }
        tx.commit()
            .map_err(|e| format!("commit obsolete generated gov cleanup failed: {e}"))?;
        Ok(())
    })
}
