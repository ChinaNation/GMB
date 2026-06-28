//! 国家码、省级行政区码与 CID 机构码的全仓唯一常量真源。
//!
//! 中文注释(铁律):
//! - 国家码、省级行政区码、机构码只在本文件维护。
//! - CID 号生成、解析、校验的核心协议在 `citizenchain/runtime/primitives/cid/`。
//! - registry 只能在 `citizenchain/registry/src/cid/` 做 SQLite 行政区、当前年份、
//!   UUID 与数据库查重等运行态适配,不得手写第二份机构码表或省码表。
//! - 省级行政区代码来自 `citizenchain/registry/src/cid/china/china.sqlite` 现有 43 省抽离结果;
//!   市、镇行政区代码仍由 registry CID 行政区数据按省管理。
//! - 字段命名必须使用 `country_full_name` / `country_short_name` /
//!   `province_name` / `cid_short_name`,不得恢复 `name`、`label` 等泛化字段。
//!
//! ## 字节表示
//! - 国家码、省级行政区码:2 个大写 ASCII 字符,如 `CN`、`GD`。
//! - 机构码:3~4 个大写 ASCII 字符。链上统一用 `[u8; 4]`,3 字符码右补 `0`:
//!   `NRC` → `*b"NRC\0"`;`CGOV` → `*b"CGOV"`;`PMUL` → `*b"PMUL"`。

/// 国家码:2 字符大写 ASCII。
pub type CountryCode = [u8; 2];

/// 省级行政区码:2 字符大写 ASCII。
pub type ProvinceCode = [u8; 2];

/// 机构码链上表示:3~4 字符大写 ASCII,3 字符码右补 `0`。
pub type InstitutionCode = [u8; 4];

/// 国家代码元数据。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountryCodeInfo {
    pub country_code: CountryCode,
    pub country_full_name: &'static str,
    pub country_short_name: &'static str,
}

/// 省级行政区代码元数据。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProvinceCodeInfo {
    pub province_code: ProvinceCode,
    pub province_name: &'static str,
}

/// 机构码元数据。`cid_short_name` 是机构实体中文简称,也是机构码对应中文名。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstitutionCodeInfo {
    pub institution_code: InstitutionCode,
    pub institution_code_text: &'static str,
    pub cid_short_name: &'static str,
}

/// 机构码所属行政层级(由机构码本身派生)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminLevel {
    /// 国家级(26 个 A 组国家级单体)。
    National,
    /// 省级(17 个 B 组省级类型)。
    Province,
    /// 市级(17 个 C 组市级类型)。
    City,
    /// 镇级(14 个 D 组镇级类型)。
    Town,
}

/// 机构码的盈利策略(决定号码 M1 / 盈利位如何生成与校验)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfitPolicy {
    /// 固定非盈利(公权机构、公益组织)。
    NonProfit,
    /// 固定盈利(经营性私权实体、公民人/自然人)。
    Profit,
    /// 按实例可变(注册协会、智能人、私立/教会学校)。
    Variable,
    /// 继承父级法人盈利属性(非法人组织)。
    InheritParent,
}

/// 国家代码:中华民族联邦共和国。
pub const COUNTRY_CN: CountryCode = *b"CN";

/// 国家代码信息。
pub const COUNTRY_CN_INFO: CountryCodeInfo = CountryCodeInfo {
    country_code: COUNTRY_CN,
    country_full_name: "中华民族联邦共和国",
    country_short_name: "中华联邦",
};

/// 从字符串机构码构造链上字节表示(3 字符右补 `0`)。
pub const fn code_bytes(s: &str) -> InstitutionCode {
    let b = s.as_bytes();
    let mut out = [0u8; 4];
    let mut i = 0;
    while i < b.len() && i < 4 {
        out[i] = b[i];
        i += 1;
    }
    out
}

/// 省级行政区代码表:国家/省级行政区统一两位大写字母。
pub const PROVINCE_CODE_INFOS: [ProvinceCodeInfo; 43] = [
    ProvinceCodeInfo {
        province_code: *b"ZS",
        province_name: "中枢省",
    },
    ProvinceCodeInfo {
        province_code: *b"LN",
        province_name: "岭南省",
    },
    ProvinceCodeInfo {
        province_code: *b"GD",
        province_name: "广东省",
    },
    ProvinceCodeInfo {
        province_code: *b"GX",
        province_name: "广西省",
    },
    ProvinceCodeInfo {
        province_code: *b"FJ",
        province_name: "福建省",
    },
    ProvinceCodeInfo {
        province_code: *b"HN",
        province_name: "海南省",
    },
    ProvinceCodeInfo {
        province_code: *b"YN",
        province_name: "云南省",
    },
    ProvinceCodeInfo {
        province_code: *b"GZ",
        province_name: "贵州省",
    },
    ProvinceCodeInfo {
        province_code: *b"HU",
        province_name: "湖南省",
    },
    ProvinceCodeInfo {
        province_code: *b"JX",
        province_name: "江西省",
    },
    ProvinceCodeInfo {
        province_code: *b"ZJ",
        province_name: "浙江省",
    },
    ProvinceCodeInfo {
        province_code: *b"JS",
        province_name: "江苏省",
    },
    ProvinceCodeInfo {
        province_code: *b"SD",
        province_name: "山东省",
    },
    ProvinceCodeInfo {
        province_code: *b"SX",
        province_name: "山西省",
    },
    ProvinceCodeInfo {
        province_code: *b"HE",
        province_name: "河南省",
    },
    ProvinceCodeInfo {
        province_code: *b"HB",
        province_name: "河北省",
    },
    ProvinceCodeInfo {
        province_code: *b"HI",
        province_name: "湖北省",
    },
    ProvinceCodeInfo {
        province_code: *b"SI",
        province_name: "陕西省",
    },
    ProvinceCodeInfo {
        province_code: *b"CQ",
        province_name: "重庆省",
    },
    ProvinceCodeInfo {
        province_code: *b"SC",
        province_name: "四川省",
    },
    ProvinceCodeInfo {
        province_code: *b"GS",
        province_name: "甘肃省",
    },
    ProvinceCodeInfo {
        province_code: *b"BP",
        province_name: "北平省",
    },
    ProvinceCodeInfo {
        province_code: *b"HA",
        province_name: "海滨省",
    },
    ProvinceCodeInfo {
        province_code: *b"SJ",
        province_name: "松江省",
    },
    ProvinceCodeInfo {
        province_code: *b"LJ",
        province_name: "龙江省",
    },
    ProvinceCodeInfo {
        province_code: *b"JL",
        province_name: "吉林省",
    },
    ProvinceCodeInfo {
        province_code: *b"LI",
        province_name: "辽宁省",
    },
    ProvinceCodeInfo {
        province_code: *b"NX",
        province_name: "宁夏省",
    },
    ProvinceCodeInfo {
        province_code: *b"QH",
        province_name: "青海省",
    },
    ProvinceCodeInfo {
        province_code: *b"AH",
        province_name: "安徽省",
    },
    ProvinceCodeInfo {
        province_code: *b"TW",
        province_name: "台湾省",
    },
    ProvinceCodeInfo {
        province_code: *b"XZ",
        province_name: "西藏省",
    },
    ProvinceCodeInfo {
        province_code: *b"XJ",
        province_name: "新疆省",
    },
    ProvinceCodeInfo {
        province_code: *b"XK",
        province_name: "西康省",
    },
    ProvinceCodeInfo {
        province_code: *b"AL",
        province_name: "阿里省",
    },
    ProvinceCodeInfo {
        province_code: *b"CL",
        province_name: "葱岭省",
    },
    ProvinceCodeInfo {
        province_code: *b"YL",
        province_name: "伊犁省",
    },
    ProvinceCodeInfo {
        province_code: *b"HX",
        province_name: "河西省",
    },
    ProvinceCodeInfo {
        province_code: *b"KL",
        province_name: "昆仑省",
    },
    ProvinceCodeInfo {
        province_code: *b"HT",
        province_name: "河套省",
    },
    ProvinceCodeInfo {
        province_code: *b"RH",
        province_name: "热河省",
    },
    ProvinceCodeInfo {
        province_code: *b"XA",
        province_name: "兴安省",
    },
    ProvinceCodeInfo {
        province_code: *b"HJ",
        province_name: "合江省",
    },
];

const fn province_codes_from_infos() -> [ProvinceCode; 43] {
    let mut out = [[0u8; 2]; 43];
    let mut i = 0;
    while i < PROVINCE_CODE_INFOS.len() {
        out[i] = PROVINCE_CODE_INFOS[i].province_code;
        i += 1;
    }
    out
}

/// 全部省级行政区代码,顺序与 `PROVINCE_CODE_INFOS` 一致。
pub const PROVINCE_CODES: [ProvinceCode; 43] = province_codes_from_infos();

/// 国家码转文本。
pub fn country_code_text(code: &CountryCode) -> Option<&str> {
    core::str::from_utf8(code).ok()
}

/// 省级行政区码转文本。
pub fn province_code_text(code: &ProvinceCode) -> Option<&'static str> {
    let matched = PROVINCE_CODE_INFOS
        .iter()
        .find(|info| info.province_code.eq_ignore_ascii_case(code))?;
    core::str::from_utf8(&matched.province_code).ok()
}

/// 省级行政区名称转两位省码。
pub fn province_code_by_name(province_name: &str) -> Option<ProvinceCode> {
    PROVINCE_CODE_INFOS
        .iter()
        .find(|info| info.province_name == province_name.trim())
        .map(|info| info.province_code)
}

/// 两位省码转省级行政区名称。
pub fn province_name_by_code(code: &ProvinceCode) -> Option<&'static str> {
    PROVINCE_CODE_INFOS
        .iter()
        .find(|info| info.province_code.eq_ignore_ascii_case(code))
        .map(|info| info.province_name)
}

// ──────────────────────────────────────────────────────────────────
// 机构码清单:A-I 九组,分组只用注释表达,不在数据项里另设 group/kind 字段。
// ──────────────────────────────────────────────────────────────────

/// 国家公民储备委员会(固定治理档)。
pub const NRC: InstitutionCode = *b"NRC\0";
/// 省公民储备委员会(固定治理档)。
pub const PRC: InstitutionCode = *b"PRC\0";
/// 省公民储备银行(固定治理档)。
pub const PRB: InstitutionCode = *b"PRB\0";

/// 个人多签账户(不发号,仅链上/后端分类常量)。
pub const PMUL: InstitutionCode = *b"PMUL";

/// 全部 92 个机构码信息,按 A-I 九组排列。
pub const INSTITUTION_CODE_INFOS: [InstitutionCodeInfo; 92] = [
    // A 国家级单体(26,3 位,公法人,非盈利)
    InstitutionCodeInfo {
        institution_code: *b"PRS\0",
        institution_code_text: "PRS",
        cid_short_name: "总统府",
    },
    InstitutionCodeInfo {
        institution_code: *b"FSC\0",
        institution_code_text: "FSC",
        cid_short_name: "联邦安全局",
    },
    InstitutionCodeInfo {
        institution_code: *b"FIB\0",
        institution_code_text: "FIB",
        cid_short_name: "联邦情报局",
    },
    InstitutionCodeInfo {
        institution_code: *b"FSS\0",
        institution_code_text: "FSS",
        cid_short_name: "联邦特勤局",
    },
    InstitutionCodeInfo {
        institution_code: *b"FPR\0",
        institution_code_text: "FPR",
        cid_short_name: "联邦人事局",
    },
    InstitutionCodeInfo {
        institution_code: *b"FRG\0",
        institution_code_text: "FRG",
        cid_short_name: "联邦注册局",
    },
    InstitutionCodeInfo {
        institution_code: *b"MFA\0",
        institution_code_text: "MFA",
        cid_short_name: "外事交流部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MDF\0",
        institution_code_text: "MDF",
        cid_short_name: "国家防务部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MHS\0",
        institution_code_text: "MHS",
        cid_short_name: "国土安全部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MCW\0",
        institution_code_text: "MCW",
        cid_short_name: "公民生活保障部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MHU\0",
        institution_code_text: "MHU",
        cid_short_name: "住房与城镇建设部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MAG\0",
        institution_code_text: "MAG",
        cid_short_name: "农业与农村发展部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MCM\0",
        institution_code_text: "MCM",
        cid_short_name: "商务与市场贸易部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MFT\0",
        institution_code_text: "MFT",
        cid_short_name: "财政与税务部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MEN\0",
        institution_code_text: "MEN",
        cid_short_name: "能源与环保发展部",
    },
    InstitutionCodeInfo {
        institution_code: *b"MTR\0",
        institution_code_text: "MTR",
        cid_short_name: "交通运输部",
    },
    InstitutionCodeInfo {
        institution_code: *b"NLG\0",
        institution_code_text: "NLG",
        cid_short_name: "国家立法院",
    },
    InstitutionCodeInfo {
        institution_code: *b"NJD\0",
        institution_code_text: "NJD",
        cid_short_name: "国家司法院",
    },
    InstitutionCodeInfo {
        institution_code: *b"NSP\0",
        institution_code_text: "NSP",
        cid_short_name: "国家监察院",
    },
    InstitutionCodeInfo {
        institution_code: *b"FAC\0",
        institution_code_text: "FAC",
        cid_short_name: "联邦廉政署",
    },
    InstitutionCodeInfo {
        institution_code: *b"FAU\0",
        institution_code_text: "FAU",
        cid_short_name: "联邦审计署",
    },
    InstitutionCodeInfo {
        institution_code: *b"FIV\0",
        institution_code_text: "FIV",
        cid_short_name: "联邦调查署",
    },
    InstitutionCodeInfo {
        institution_code: *b"NED\0",
        institution_code_text: "NED",
        cid_short_name: "国家公民教育委员会",
    },
    InstitutionCodeInfo {
        institution_code: *b"NRC\0",
        institution_code_text: "NRC",
        cid_short_name: "国家公民储备委员会",
    },
    InstitutionCodeInfo {
        institution_code: *b"NSN\0",
        institution_code_text: "NSN",
        cid_short_name: "国家参议会",
    },
    InstitutionCodeInfo {
        institution_code: *b"NRP\0",
        institution_code_text: "NRP",
        cid_short_name: "国家众议会",
    },
    // B 省级类型(17,3 位,43 省共用,R5 省码区分实例,非盈利)
    InstitutionCodeInfo {
        institution_code: *b"PGV\0",
        institution_code_text: "PGV",
        cid_short_name: "省政府",
    },
    InstitutionCodeInfo {
        institution_code: *b"PLG\0",
        institution_code_text: "PLG",
        cid_short_name: "省立法院",
    },
    InstitutionCodeInfo {
        institution_code: *b"PJD\0",
        institution_code_text: "PJD",
        cid_short_name: "省司法院",
    },
    InstitutionCodeInfo {
        institution_code: *b"PSP\0",
        institution_code_text: "PSP",
        cid_short_name: "省监察院",
    },
    InstitutionCodeInfo {
        institution_code: *b"PRC\0",
        institution_code_text: "PRC",
        cid_short_name: "省储会",
    },
    InstitutionCodeInfo {
        institution_code: *b"PRB\0",
        institution_code_text: "PRB",
        cid_short_name: "省储行",
    },
    InstitutionCodeInfo {
        institution_code: *b"PDF\0",
        institution_code_text: "PDF",
        cid_short_name: "省防务厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PHS\0",
        institution_code_text: "PHS",
        cid_short_name: "省国安厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PCW\0",
        institution_code_text: "PCW",
        cid_short_name: "省民生厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PHU\0",
        institution_code_text: "PHU",
        cid_short_name: "省住建厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PAG\0",
        institution_code_text: "PAG",
        cid_short_name: "省农业厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PCM\0",
        institution_code_text: "PCM",
        cid_short_name: "省商贸厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PFT\0",
        institution_code_text: "PFT",
        cid_short_name: "省财税厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PEN\0",
        institution_code_text: "PEN",
        cid_short_name: "省能源厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PTR\0",
        institution_code_text: "PTR",
        cid_short_name: "省交通厅",
    },
    InstitutionCodeInfo {
        institution_code: *b"PSN\0",
        institution_code_text: "PSN",
        cid_short_name: "省参议会",
    },
    InstitutionCodeInfo {
        institution_code: *b"PRP\0",
        institution_code_text: "PRP",
        cid_short_name: "省众议会",
    },
    // C 市级类型(17,4 位,非盈利)
    InstitutionCodeInfo {
        institution_code: *b"CGOV",
        institution_code_text: "CGOV",
        cid_short_name: "市政府",
    },
    InstitutionCodeInfo {
        institution_code: *b"CLEG",
        institution_code_text: "CLEG",
        cid_short_name: "市立法委",
    },
    InstitutionCodeInfo {
        institution_code: *b"CSUP",
        institution_code_text: "CSUP",
        cid_short_name: "市监察院",
    },
    InstitutionCodeInfo {
        institution_code: *b"CJUD",
        institution_code_text: "CJUD",
        cid_short_name: "市司法院",
    },
    InstitutionCodeInfo {
        institution_code: *b"CEDU",
        institution_code_text: "CEDU",
        cid_short_name: "市教委",
    },
    InstitutionCodeInfo {
        institution_code: *b"CSLF",
        institution_code_text: "CSLF",
        cid_short_name: "市自治委",
    },
    InstitutionCodeInfo {
        institution_code: *b"CDEF",
        institution_code_text: "CDEF",
        cid_short_name: "市国防局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CHSC",
        institution_code_text: "CHSC",
        cid_short_name: "市国安局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CCWF",
        institution_code_text: "CCWF",
        cid_short_name: "市民生局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CHUD",
        institution_code_text: "CHUD",
        cid_short_name: "市住建局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CAGR",
        institution_code_text: "CAGR",
        cid_short_name: "市农业局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CCOM",
        institution_code_text: "CCOM",
        cid_short_name: "市商贸局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CFIN",
        institution_code_text: "CFIN",
        cid_short_name: "市财税局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CENR",
        institution_code_text: "CENR",
        cid_short_name: "市能源局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CTRN",
        institution_code_text: "CTRN",
        cid_short_name: "市交通局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CREG",
        institution_code_text: "CREG",
        cid_short_name: "市注册局",
    },
    InstitutionCodeInfo {
        institution_code: *b"CPOL",
        institution_code_text: "CPOL",
        cid_short_name: "市公安局",
    },
    // D 镇级类型(14,4 位,非盈利)
    InstitutionCodeInfo {
        institution_code: *b"TGOV",
        institution_code_text: "TGOV",
        cid_short_name: "镇政府",
    },
    InstitutionCodeInfo {
        institution_code: *b"TCWF",
        institution_code_text: "TCWF",
        cid_short_name: "镇民生科",
    },
    InstitutionCodeInfo {
        institution_code: *b"THUD",
        institution_code_text: "THUD",
        cid_short_name: "镇住建科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TAGR",
        institution_code_text: "TAGR",
        cid_short_name: "镇农业科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TFIN",
        institution_code_text: "TFIN",
        cid_short_name: "镇财税科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TDEF",
        institution_code_text: "TDEF",
        cid_short_name: "镇国防科",
    },
    InstitutionCodeInfo {
        institution_code: *b"THSC",
        institution_code_text: "THSC",
        cid_short_name: "镇国安科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TCOM",
        institution_code_text: "TCOM",
        cid_short_name: "镇商贸科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TENR",
        institution_code_text: "TENR",
        cid_short_name: "镇能源科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TTRN",
        institution_code_text: "TTRN",
        cid_short_name: "镇交通科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TPOL",
        institution_code_text: "TPOL",
        cid_short_name: "镇公安科",
    },
    InstitutionCodeInfo {
        institution_code: *b"TSLF",
        institution_code_text: "TSLF",
        cid_short_name: "镇自治委",
    },
    InstitutionCodeInfo {
        institution_code: *b"TSUP",
        institution_code_text: "TSUP",
        cid_short_name: "镇监察院",
    },
    InstitutionCodeInfo {
        institution_code: *b"TJUD",
        institution_code_text: "TJUD",
        cid_short_name: "镇司法院",
    },
    // E 私权机构(7,4 位)
    InstitutionCodeInfo {
        institution_code: *b"SFGT",
        institution_code_text: "SFGT",
        cid_short_name: "个体经营",
    },
    InstitutionCodeInfo {
        institution_code: *b"SFGP",
        institution_code_text: "SFGP",
        cid_short_name: "无限合伙",
    },
    InstitutionCodeInfo {
        institution_code: *b"SFLP",
        institution_code_text: "SFLP",
        cid_short_name: "有限合伙",
    },
    InstitutionCodeInfo {
        institution_code: *b"SFGQ",
        institution_code_text: "SFGQ",
        cid_short_name: "股权公司",
    },
    InstitutionCodeInfo {
        institution_code: *b"SFGF",
        institution_code_text: "SFGF",
        cid_short_name: "股份公司",
    },
    InstitutionCodeInfo {
        institution_code: *b"SFGY",
        institution_code_text: "SFGY",
        cid_short_name: "公益组织",
    },
    InstitutionCodeInfo {
        institution_code: *b"SFAS",
        institution_code_text: "SFAS",
        cid_short_name: "注册协会",
    },
    // F 教育学校(6:公私教大学 3 位 / 公私教中小初学 4 位)
    InstitutionCodeInfo {
        institution_code: *b"GUN\0",
        institution_code_text: "GUN",
        cid_short_name: "公立大学",
    },
    InstitutionCodeInfo {
        institution_code: *b"SUN\0",
        institution_code_text: "SUN",
        cid_short_name: "私立大学",
    },
    InstitutionCodeInfo {
        institution_code: *b"JUN\0",
        institution_code_text: "JUN",
        cid_short_name: "教会大学",
    },
    InstitutionCodeInfo {
        institution_code: *b"GSCH",
        institution_code_text: "GSCH",
        cid_short_name: "公立学校",
    },
    InstitutionCodeInfo {
        institution_code: *b"SFSC",
        institution_code_text: "SFSC",
        cid_short_name: "私立学校",
    },
    InstitutionCodeInfo {
        institution_code: *b"JSCH",
        institution_code_text: "JSCH",
        cid_short_name: "教会学校",
    },
    // G 个人主体(3,4 位)
    InstitutionCodeInfo {
        institution_code: *b"CTZN",
        institution_code_text: "CTZN",
        cid_short_name: "公民人",
    },
    InstitutionCodeInfo {
        institution_code: *b"NATP",
        institution_code_text: "NATP",
        cid_short_name: "自然人",
    },
    InstitutionCodeInfo {
        institution_code: *b"SMTP",
        institution_code_text: "SMTP",
        cid_short_name: "智能人",
    },
    // H 非法人组织(1,4 位)
    InstitutionCodeInfo {
        institution_code: *b"UNIN",
        institution_code_text: "UNIN",
        cid_short_name: "非法人组织",
    },
    // I 个人多签(1,4 位,不发号)
    InstitutionCodeInfo {
        institution_code: *b"PMUL",
        institution_code_text: "PMUL",
        cid_short_name: "个人多签",
    },
];

const fn institution_codes_from_infos() -> [InstitutionCode; 92] {
    let mut out = [[0u8; 4]; 92];
    let mut i = 0;
    while i < INSTITUTION_CODE_INFOS.len() {
        out[i] = INSTITUTION_CODE_INFOS[i].institution_code;
        i += 1;
    }
    out
}

/// 全部 92 个机构码,顺序与 `INSTITUTION_CODE_INFOS` 一致。
pub const ALL_CODES: [InstitutionCode; 92] = institution_codes_from_infos();

fn institution_info(code: &InstitutionCode) -> Option<&'static InstitutionCodeInfo> {
    INSTITUTION_CODE_INFOS
        .iter()
        .find(|info| info.institution_code == *code)
}

/// 机构码字节转 3/4 字符文本。
pub fn institution_code_text(code: &InstitutionCode) -> Option<&'static str> {
    institution_info(code).map(|info| info.institution_code_text)
}

/// 机构码对应的机构实体中文简称。
pub fn cid_short_name(code: &InstitutionCode) -> Option<&'static str> {
    institution_info(code).map(|info| info.cid_short_name)
}

/// 解析机构码:接受 3/4 字符机构码或机构实体中文简称。
pub fn institution_code_from_str(value: &str) -> Option<InstitutionCode> {
    let v = value.trim();
    INSTITUTION_CODE_INFOS
        .iter()
        .find(|info| info.institution_code_text == v || info.cid_short_name == v)
        .map(|info| info.institution_code)
}

/// 机构码字符长度(3 = 国家/省部布局,4 = 市镇/私权/个人布局)。
pub fn institution_code_len(code: &InstitutionCode) -> Option<usize> {
    institution_code_text(code).map(str::len)
}

/// 是否为 3 字符码(国家级单体、省级类型、大学类)。
pub fn is_three_char_code(code: &InstitutionCode) -> bool {
    institution_code_len(code) == Some(3)
}

/// 从 CID 号(`R5-seg2-N9-D4`)解析机构码。
///
/// 机构码在第二段 seg2:3 字符码布局 = `码(3)+盈利位(1)+校验(1)`;
/// 4 字符码布局 = `码(4)+M1(1)`。靠 seg2 索引 3 区分(数字→3 字符,字母→4 字符)。
pub fn institution_code_from_cid_number(cid_number: &str) -> Option<InstitutionCode> {
    let seg2 = cid_number.split('-').nth(1)?;
    let b = seg2.as_bytes();
    if b.len() < 4 {
        return None;
    }
    let code_len = if b[3].is_ascii_alphabetic() { 4 } else { 3 };
    let mut out = [0u8; 4];
    let mut i = 0;
    while i < code_len {
        out[i] = b[i];
        i += 1;
    }
    Some(out)
}

fn text_matches(code: &InstitutionCode, values: &[&str]) -> bool {
    let Some(text) = institution_code_text(code) else {
        return false;
    };
    values.iter().any(|value| *value == text)
}

/// 盈利策略(决定号码 M1 / 盈利位的生成与校验)。
pub fn profit_policy(code: &InstitutionCode) -> Option<ProfitPolicy> {
    let text = institution_code_text(code)?;
    let policy = match text {
        "SFGT" | "SFGP" | "SFLP" | "SFGQ" | "SFGF" | "CTZN" | "NATP" => ProfitPolicy::Profit,
        "SFAS" | "SMTP" | "SUN" | "SFSC" | "JUN" | "JSCH" => ProfitPolicy::Variable,
        "UNIN" => ProfitPolicy::InheritParent,
        _ => ProfitPolicy::NonProfit,
    };
    Some(policy)
}

/// 个人主体(公民人/自然人/智能人)。
pub fn is_person_code(code: &InstitutionCode) -> bool {
    text_matches(code, &["CTZN", "NATP", "SMTP"])
}

/// 非法人(个体经营/无限合伙/非法人组织)。
pub fn is_unincorporated_code(code: &InstitutionCode) -> bool {
    text_matches(code, &["SFGT", "SFGP", "UNIN"])
}

/// 私法人(有限合伙/股权/股份/公益/协会/私立大学/私立学校/教会大学/教会学校)。
pub fn is_private_legal_code(code: &InstitutionCode) -> bool {
    text_matches(
        code,
        &[
            "SFLP", "SFGQ", "SFGF", "SFGY", "SFAS", "SUN", "JUN", "SFSC", "JSCH",
        ],
    )
}

/// 机构码所属行政层级。
pub fn admin_level(code: &InstitutionCode) -> Option<AdminLevel> {
    let text = institution_code_text(code)?;
    match text {
        // A 国家级单体(26)
        "PRS" | "FSC" | "FIB" | "FSS" | "FPR" | "FRG" | "MFA" | "MDF" | "MHS" | "MCW" | "MHU"
        | "MAG" | "MCM" | "MFT" | "MEN" | "MTR" | "NLG" | "NJD" | "NSP" | "FAC" | "FAU" | "FIV"
        | "NED" | "NRC" | "NSN" | "NRP" => Some(AdminLevel::National),
        // B 省级类型(17)
        "PGV" | "PLG" | "PJD" | "PSP" | "PRC" | "PRB" | "PDF" | "PHS" | "PCW" | "PHU" | "PAG"
        | "PCM" | "PFT" | "PEN" | "PTR" | "PSN" | "PRP" => Some(AdminLevel::Province),
        // C 市级类型(17)
        "CGOV" | "CLEG" | "CSUP" | "CJUD" | "CEDU" | "CSLF" | "CDEF" | "CHSC" | "CCWF" | "CHUD"
        | "CAGR" | "CCOM" | "CFIN" | "CENR" | "CTRN" | "CREG" | "CPOL" => Some(AdminLevel::City),
        // D 镇级类型(14)
        "TGOV" | "TCWF" | "THUD" | "TAGR" | "TFIN" | "TDEF" | "THSC" | "TCOM" | "TENR" | "TTRN"
        | "TPOL" | "TSLF" | "TSUP" | "TJUD" => Some(AdminLevel::Town),
        _ => None,
    }
}

/// 公法人(国家/省部/市镇公权机构、公立大学/学校)。
pub fn is_public_legal_code(code: &InstitutionCode) -> bool {
    admin_level(code).is_some() || text_matches(code, &["GUN", "GSCH"])
}

/// 是否教育机构(公私大学/学校)。
pub fn is_education_institution_code(code: &InstitutionCode) -> bool {
    text_matches(code, &["GUN", "SUN", "JUN", "GSCH", "SFSC", "JSCH"])
}

/// 是否基础教育学校(初学/小学/中学),需要 education_type 级别字段。
pub fn requires_education_level(code: &InstitutionCode) -> bool {
    text_matches(code, &["GSCH", "SFSC", "JSCH"])
}

/// 是否为固定治理档机构码(国储会/省储会/省储行)。
pub fn is_fixed_governance_code(code: &InstitutionCode) -> bool {
    matches!(*code, NRC | PRC | PRB)
}

/// 固定治理档机构码的制度阈值(国储会 13 / 省储会 6 / 省储行 6)。
pub fn fixed_governance_pass_threshold(code: &InstitutionCode) -> Option<u32> {
    use crate::count_const::{
        NRC_INTERNAL_THRESHOLD, PRB_INTERNAL_THRESHOLD, PRC_INTERNAL_THRESHOLD,
    };
    match *code {
        NRC => Some(NRC_INTERNAL_THRESHOLD),
        PRC => Some(PRC_INTERNAL_THRESHOLD),
        PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

/// 是否为个人多签账户机构码(PMUL)。管理员来自 personal-admins。
pub fn is_personal_code(code: &InstitutionCode) -> bool {
    *code == PMUL
}

/// 是否为机构账户机构码(公权法人/私权法人/非法人实体)。
pub fn is_institution_code(code: &InstitutionCode) -> bool {
    !is_fixed_governance_code(code)
        && (is_public_legal_code(code)
            || is_private_legal_code(code)
            || is_unincorporated_code(code))
}

/// 是否为注册多签动态阈值账户机构码(个人多签 或 机构账户)。
pub fn is_registered_multisig_code(code: &InstitutionCode) -> bool {
    is_personal_code(code) || is_institution_code(code)
}

/// 是否为内部投票支持的治理机构码(固定治理档 或 注册多签账户)。
pub fn is_valid_governance_code(code: &InstitutionCode) -> bool {
    is_fixed_governance_code(code) || is_registered_multisig_code(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_code_uses_constitution_name() {
        assert_eq!(country_code_text(&COUNTRY_CN), Some("CN"));
        assert_eq!(COUNTRY_CN_INFO.country_full_name, "中华民族联邦共和国");
        assert_eq!(COUNTRY_CN_INFO.country_short_name, "中华联邦");
    }

    #[test]
    fn province_codes_are_two_uppercase_and_unique() {
        assert_eq!(PROVINCE_CODE_INFOS.len(), 43);
        for info in PROVINCE_CODE_INFOS {
            let text = province_code_text(&info.province_code).expect("province code ascii");
            assert_eq!(text.len(), 2);
            assert!(text.chars().all(|ch| ch.is_ascii_uppercase()));
        }
        for i in 0..PROVINCE_CODES.len() {
            for j in (i + 1)..PROVINCE_CODES.len() {
                assert_ne!(PROVINCE_CODES[i], PROVINCE_CODES[j], "province duplicate");
            }
        }
        assert_eq!(province_code_by_name("广东省"), Some(*b"GD"));
        assert_eq!(province_name_by_code(b"HJ"), Some("合江省"));
    }

    #[test]
    fn institution_codes_are_three_or_four_uppercase_and_unique() {
        assert_eq!(INSTITUTION_CODE_INFOS.len(), 92);
        for info in INSTITUTION_CODE_INFOS {
            let text = info.institution_code_text;
            assert!(text.len() == 3 || text.len() == 4);
            assert!(text.chars().all(|ch| ch.is_ascii_uppercase()));
            assert_eq!(institution_code_from_str(text), Some(info.institution_code));
            assert_eq!(
                institution_code_from_str(info.cid_short_name),
                Some(info.institution_code)
            );
        }
        for i in 0..ALL_CODES.len() {
            for j in (i + 1)..ALL_CODES.len() {
                assert_ne!(ALL_CODES[i], ALL_CODES[j], "institution duplicate");
            }
        }
    }

    #[test]
    fn classification_spot_check() {
        assert_eq!(institution_code_text(&NRC), Some("NRC"));
        assert_eq!(cid_short_name(&NRC), Some("国家公民储备委员会"));
        assert!(is_fixed_governance_code(&NRC));
        assert!(!is_registered_multisig_code(&NRC));
        assert!(is_public_legal_code(&NRC));

        assert!(is_personal_code(&PMUL));
        assert!(is_registered_multisig_code(&PMUL));
        assert!(!is_institution_code(&PMUL));

        assert!(is_institution_code(b"CGOV"));
        assert!(is_institution_code(b"SFLP"));
        assert!(is_unincorporated_code(b"UNIN"));
        assert!(is_person_code(b"CTZN"));
        assert!(!is_registered_multisig_code(b"CTZN"));
    }

    #[test]
    fn profit_policy_spot_check() {
        assert_eq!(profit_policy(b"SFGQ"), Some(ProfitPolicy::Profit));
        assert_eq!(profit_policy(b"SFGY"), Some(ProfitPolicy::NonProfit));
        assert_eq!(profit_policy(b"SFAS"), Some(ProfitPolicy::Variable));
        assert_eq!(profit_policy(b"SMTP"), Some(ProfitPolicy::Variable));
        assert_eq!(profit_policy(b"UNIN"), Some(ProfitPolicy::InheritParent));
    }

    #[test]
    fn fixed_governance_thresholds_match_constants() {
        assert_eq!(fixed_governance_pass_threshold(&NRC), Some(13));
        assert_eq!(fixed_governance_pass_threshold(&PRC), Some(6));
        assert_eq!(fixed_governance_pass_threshold(&PRB), Some(6));
        assert_eq!(fixed_governance_pass_threshold(&PMUL), None);
        assert_eq!(fixed_governance_pass_threshold(b"CGOV"), None);
    }

    #[test]
    fn code_bytes_pads_three_char() {
        assert_eq!(code_bytes("NRC"), *b"NRC\0");
        assert_eq!(code_bytes("CGOV"), *b"CGOV");
    }
}
