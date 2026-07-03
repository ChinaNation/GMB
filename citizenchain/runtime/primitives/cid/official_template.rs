//! 公权机构命名与机构集模板(全仓单源,ADR-031 卡3)。
//!
//! 用户统一命名规则(gov-deterministic-v8)后,把确定性模板从 onchina 收归
//! primitives 单源,创世直铸与 onchina 目录共享,杜绝两处漂移。
//!
//! 组装规则(282 常量逆向验证零例外):
//!   简称 = 行政区显示名 + `suffix`
//!   全称 = 行政区显示名 + `full_suffix`
//! 显示名:省级部门=省名,市级=市名,镇级=镇名,国家参众议会=国名前缀已含在 full。

use alloc::format;
use alloc::string::String;

/// 一个机构模板:机构码 + 简称后缀 + 全称后缀。
pub struct OfficialOrgTemplate {
    pub institution_code: &'static str,
    pub suffix: &'static str,
    pub full_suffix: &'static str,
}

impl OfficialOrgTemplate {
    /// 简称 = 显示名 + suffix。
    pub fn short_name(&self, display_area_name: &str) -> String {
        format!("{display_area_name}{}", self.suffix)
    }

    /// 全称 = 显示名 + full_suffix。
    pub fn full_name(&self, display_area_name: &str) -> String {
        format!("{display_area_name}{}", self.full_suffix)
    }
}

/// 省级部门模板(11 类;省核心治理 6 类为 china_*.rs 常量,不在此)。
pub const PROVINCE_DEPARTMENT_TEMPLATES: &[OfficialOrgTemplate] = &[
    OfficialOrgTemplate {
        institution_code: "PDF",
        suffix: "国防厅",
        full_suffix: "国家防务厅",
    },
    OfficialOrgTemplate {
        institution_code: "PHS",
        suffix: "国安厅",
        full_suffix: "国土安全厅",
    },
    OfficialOrgTemplate {
        institution_code: "PCW",
        suffix: "民生厅",
        full_suffix: "公民生活保障厅",
    },
    OfficialOrgTemplate {
        institution_code: "PHU",
        suffix: "住建厅",
        full_suffix: "住房与城镇建设厅",
    },
    OfficialOrgTemplate {
        institution_code: "PAG",
        suffix: "农业厅",
        full_suffix: "农业与农村发展厅",
    },
    OfficialOrgTemplate {
        institution_code: "PCM",
        suffix: "商贸厅",
        full_suffix: "商务与市场贸易厅",
    },
    OfficialOrgTemplate {
        institution_code: "PFT",
        suffix: "财税厅",
        full_suffix: "财政与税务厅",
    },
    OfficialOrgTemplate {
        institution_code: "PEN",
        suffix: "能源厅",
        full_suffix: "能源与环保发展厅",
    },
    OfficialOrgTemplate {
        institution_code: "PTR",
        suffix: "交通厅",
        full_suffix: "交通运输厅",
    },
    OfficialOrgTemplate {
        institution_code: "PSN",
        suffix: "参议会",
        full_suffix: "联邦立法院参议会",
    },
    OfficialOrgTemplate {
        institution_code: "PRP",
        suffix: "众议会",
        full_suffix: "联邦立法院众议会",
    },
];

/// 市级机构模板(C 族 17 类,全)。
pub const CITY_TEMPLATES: &[OfficialOrgTemplate] = &[
    OfficialOrgTemplate {
        institution_code: "CGOV",
        suffix: "政府",
        full_suffix: "自治政府",
    },
    OfficialOrgTemplate {
        institution_code: "CLEG",
        suffix: "立法会",
        full_suffix: "公民立法委员会",
    },
    OfficialOrgTemplate {
        institution_code: "CSUP",
        suffix: "监察院",
        full_suffix: "自治监察院",
    },
    OfficialOrgTemplate {
        institution_code: "CJUD",
        suffix: "司法院",
        full_suffix: "自治司法院",
    },
    OfficialOrgTemplate {
        institution_code: "CEDU",
        suffix: "教委会",
        full_suffix: "公民教育委员会",
    },
    OfficialOrgTemplate {
        institution_code: "CSLF",
        suffix: "自治会",
        full_suffix: "公民自治委员会",
    },
    OfficialOrgTemplate {
        institution_code: "CDEF",
        suffix: "国防局",
        full_suffix: "国家防务局",
    },
    OfficialOrgTemplate {
        institution_code: "CHSC",
        suffix: "国安局",
        full_suffix: "国土安全局",
    },
    OfficialOrgTemplate {
        institution_code: "CPOL",
        suffix: "公安局",
        full_suffix: "公民安全局",
    },
    OfficialOrgTemplate {
        institution_code: "CCWF",
        suffix: "民生局",
        full_suffix: "公民生活保障局",
    },
    OfficialOrgTemplate {
        institution_code: "CHUD",
        suffix: "住建局",
        full_suffix: "住房与城镇建设局",
    },
    OfficialOrgTemplate {
        institution_code: "CAGR",
        suffix: "农业局",
        full_suffix: "农业与农村发展局",
    },
    OfficialOrgTemplate {
        institution_code: "CCOM",
        suffix: "商贸局",
        full_suffix: "商务与市场贸易局",
    },
    OfficialOrgTemplate {
        institution_code: "CFIN",
        suffix: "财税局",
        full_suffix: "财政与税务局",
    },
    OfficialOrgTemplate {
        institution_code: "CENR",
        suffix: "能源局",
        full_suffix: "能源与环保发展局",
    },
    OfficialOrgTemplate {
        institution_code: "CTRN",
        suffix: "交通局",
        full_suffix: "交通运输局",
    },
    OfficialOrgTemplate {
        institution_code: "CREG",
        suffix: "注册局",
        full_suffix: "身份注册局",
    },
];

/// 镇级机构模板(D 族 14 类,全;镇无立法/教委,制度设计)。
pub const TOWN_TEMPLATES: &[OfficialOrgTemplate] = &[
    OfficialOrgTemplate {
        institution_code: "TGOV",
        suffix: "政府",
        full_suffix: "自治政府",
    },
    OfficialOrgTemplate {
        institution_code: "TCWF",
        suffix: "民生科",
        full_suffix: "公民生活保障科",
    },
    OfficialOrgTemplate {
        institution_code: "THUD",
        suffix: "住建科",
        full_suffix: "住房与城镇建设科",
    },
    OfficialOrgTemplate {
        institution_code: "TAGR",
        suffix: "农业科",
        full_suffix: "农业与农村发展科",
    },
    OfficialOrgTemplate {
        institution_code: "TFIN",
        suffix: "财税科",
        full_suffix: "财政与税务科",
    },
    OfficialOrgTemplate {
        institution_code: "TDEF",
        suffix: "国防科",
        full_suffix: "国家防务科",
    },
    OfficialOrgTemplate {
        institution_code: "THSC",
        suffix: "国安科",
        full_suffix: "国土安全科",
    },
    OfficialOrgTemplate {
        institution_code: "TCOM",
        suffix: "商贸科",
        full_suffix: "商务与市场贸易科",
    },
    OfficialOrgTemplate {
        institution_code: "TENR",
        suffix: "能源科",
        full_suffix: "能源与环保发展科",
    },
    OfficialOrgTemplate {
        institution_code: "TTRN",
        suffix: "交通科",
        full_suffix: "交通运输科",
    },
    OfficialOrgTemplate {
        institution_code: "TPOL",
        suffix: "公安科",
        full_suffix: "公民安全科",
    },
    OfficialOrgTemplate {
        institution_code: "TSLF",
        suffix: "自治会",
        full_suffix: "公民自治委员会",
    },
    OfficialOrgTemplate {
        institution_code: "TSUP",
        suffix: "监察院",
        full_suffix: "自治监察院",
    },
    OfficialOrgTemplate {
        institution_code: "TJUD",
        suffix: "司法院",
        full_suffix: "自治司法院",
    },
];

/// 国家级参众议会(模板派生,非 china_*.rs 常量);全称前缀含国名。
pub const NATIONAL_ASSEMBLY_TEMPLATES: &[OfficialOrgTemplate] = &[
    OfficialOrgTemplate {
        institution_code: "NSN",
        suffix: "国家参议会",
        full_suffix: "中华民族联邦共和国立法院参议会",
    },
    OfficialOrgTemplate {
        institution_code: "NRP",
        suffix: "国家众议会",
        full_suffix: "中华民族联邦共和国立法院众议会",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_family_counts() {
        assert_eq!(PROVINCE_DEPARTMENT_TEMPLATES.len(), 11);
        assert_eq!(CITY_TEMPLATES.len(), 17);
        assert_eq!(TOWN_TEMPLATES.len(), 14);
        assert_eq!(NATIONAL_ASSEMBLY_TEMPLATES.len(), 2);
    }

    #[test]
    fn name_composition_matches_reverse_verified_examples() {
        let gov = &CITY_TEMPLATES[0];
        assert_eq!(gov.short_name("荔湾市"), "荔湾市政府");
        assert_eq!(gov.full_name("荔湾市"), "荔湾市自治政府");
        let dept = &PROVINCE_DEPARTMENT_TEMPLATES[0];
        assert_eq!(dept.short_name("广东省"), "广东省国防厅");
        let town = &TOWN_TEMPLATES[0];
        assert_eq!(town.short_name("锦程镇"), "锦程镇政府");
    }
}
