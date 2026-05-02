//! 中文注释:SFID 号码生成使用的省/市/镇/村行政代码表。
//!
//! 本文件只保存编码与行政区层级,不再承载省管理员公钥、槽位或链上名册逻辑。
//! 省管理员归属统一放在 `crate::sheng_admins::province_admins`。

pub struct VillageCode {
    pub name: &'static str,
    pub code: &'static str,
}

pub struct TownCode {
    pub name: &'static str,
    pub code: &'static str,
    pub villages: &'static [VillageCode],
}

pub struct CityCode {
    pub name: &'static str,
    pub code: &'static str,
    pub towns: &'static [TownCode],
}

pub struct ProvinceCode {
    pub name: &'static str,
    pub code: &'static str,
    pub cities: &'static [CityCode],
}

#[rustfmt::skip]
pub const PROVINCES: [ProvinceCode; 43] = [
    ProvinceCode {name: "中枢省",code: "ZS",cities: &m01_zs::CITIES_ZS,},
    ProvinceCode {name: "岭南省",code: "LN",cities: &m02_ln::CITIES_LN,},
    ProvinceCode {name: "广东省",code: "GD",cities: &m03_gd::CITIES_GD,},
    ProvinceCode {name: "广西省",code: "GX",cities: &m04_gx::CITIES_GX,},
    ProvinceCode {name: "福建省",code: "FJ",cities: &m05_fj::CITIES_FJ,},
    ProvinceCode {name: "海南省",code: "HN",cities: &m06_hn::CITIES_HN,},
    ProvinceCode {name: "云南省",code: "YN",cities: &m07_yn::CITIES_YN,},
    ProvinceCode {name: "贵州省",code: "GZ",cities: &m08_gz::CITIES_GZ,},
    ProvinceCode {name: "湖南省",code: "HU",cities: &m09_hu::CITIES_HU,},
    ProvinceCode {name: "江西省",code: "JX",cities: &m10_jx::CITIES_JX,},
    ProvinceCode {name: "浙江省",code: "ZJ",cities: &m11_zj::CITIES_ZJ,},
    ProvinceCode {name: "江苏省",code: "JS",cities: &m12_js::CITIES_JS,},
    ProvinceCode {name: "山东省",code: "SD",cities: &m13_sd::CITIES_SD,},
    ProvinceCode {name: "山西省",code: "SX",cities: &m14_sx::CITIES_SX,},
    ProvinceCode {name: "河南省",code: "HE",cities: &m15_he::CITIES_HE,},
    ProvinceCode {name: "河北省",code: "HB",cities: &m16_hb::CITIES_HB,},
    ProvinceCode {name: "湖北省",code: "HI",cities: &m17_hi::CITIES_HI,},
    ProvinceCode {name: "陕西省",code: "SI",cities: &m18_si::CITIES_SI,},
    ProvinceCode {name: "重庆省",code: "CQ",cities: &m19_cq::CITIES_CQ,},
    ProvinceCode {name: "四川省",code: "SC",cities: &m20_sc::CITIES_SC,},
    ProvinceCode {name: "甘肃省",code: "GS",cities: &m21_gs::CITIES_GS,},
    ProvinceCode {name: "北平省",code: "BP",cities: &m22_bp::CITIES_BP,},
    ProvinceCode {name: "海滨省",code: "HA",cities: &m23_ha::CITIES_HA,},
    ProvinceCode {name: "松江省",code: "SJ",cities: &m24_sj::CITIES_SJ,},
    ProvinceCode {name: "龙江省",code: "LJ",cities: &m25_lj::CITIES_LJ,},
    ProvinceCode {name: "吉林省",code: "JL",cities: &m26_jl::CITIES_JL,},
    ProvinceCode {name: "辽宁省",code: "LI",cities: &m27_li::CITIES_LI,},
    ProvinceCode {name: "宁夏省",code: "NX",cities: &m28_nx::CITIES_NX,},
    ProvinceCode {name: "青海省",code: "QH",cities: &m29_qh::CITIES_QH,},
    ProvinceCode {name: "安徽省",code: "AH",cities: &m30_ah::CITIES_AH,},
    ProvinceCode {name: "台湾省",code: "TW",cities: &m31_tw::CITIES_TW,},
    ProvinceCode {name: "西藏省",code: "XZ",cities: &m32_xz::CITIES_XZ,},
    ProvinceCode {name: "新疆省",code: "XJ",cities: &m33_xj::CITIES_XJ,},
    ProvinceCode {name: "西康省",code: "XK",cities: &m34_xk::CITIES_XK,},
    ProvinceCode {name: "阿里省",code: "AL",cities: &m35_al::CITIES_AL,},
    ProvinceCode {name: "葱岭省",code: "CL",cities: &m36_cl::CITIES_CL,},
    ProvinceCode {name: "天山省",code: "TS",cities: &m37_ts::CITIES_TS,},
    ProvinceCode {name: "河西省",code: "HX",cities: &m38_hx::CITIES_HX,},
    ProvinceCode {name: "昆仑省",code: "KL",cities: &m39_kl::CITIES_KL,},
    ProvinceCode {name: "河套省",code: "HT",cities: &m40_ht::CITIES_HT,},
    ProvinceCode {name: "热河省",code: "RH",cities: &m41_rh::CITIES_RH,},
    ProvinceCode {name: "兴安省",code: "XA",cities: &m42_xa::CITIES_XA,},
    ProvinceCode {name: "合江省",code: "HJ",cities: &m43_hj::CITIES_HJ,},
];

#[path = "city_codes/01_ZS.rs"]
mod m01_zs;
#[path = "city_codes/02_LN.rs"]
mod m02_ln;
#[path = "city_codes/03_GD.rs"]
mod m03_gd;
#[path = "city_codes/04_GX.rs"]
mod m04_gx;
#[path = "city_codes/05_FJ.rs"]
mod m05_fj;
#[path = "city_codes/06_HN.rs"]
mod m06_hn;
#[path = "city_codes/07_YN.rs"]
mod m07_yn;
#[path = "city_codes/08_GZ.rs"]
mod m08_gz;
#[path = "city_codes/09_HU.rs"]
mod m09_hu;
#[path = "city_codes/10_JX.rs"]
mod m10_jx;
#[path = "city_codes/11_ZJ.rs"]
mod m11_zj;
#[path = "city_codes/12_JS.rs"]
mod m12_js;
#[path = "city_codes/13_SD.rs"]
mod m13_sd;
#[path = "city_codes/14_SX.rs"]
mod m14_sx;
#[path = "city_codes/15_HE.rs"]
mod m15_he;
#[path = "city_codes/16_HB.rs"]
mod m16_hb;
#[path = "city_codes/17_HI.rs"]
mod m17_hi;
#[path = "city_codes/18_SI.rs"]
mod m18_si;
#[path = "city_codes/19_CQ.rs"]
mod m19_cq;
#[path = "city_codes/20_SC.rs"]
mod m20_sc;
#[path = "city_codes/21_GS.rs"]
mod m21_gs;
#[path = "city_codes/22_BP.rs"]
mod m22_bp;
#[path = "city_codes/23_HA.rs"]
mod m23_ha;
#[path = "city_codes/24_SJ.rs"]
mod m24_sj;
#[path = "city_codes/25_LJ.rs"]
mod m25_lj;
#[path = "city_codes/26_JL.rs"]
mod m26_jl;
#[path = "city_codes/27_LI.rs"]
mod m27_li;
#[path = "city_codes/28_NX.rs"]
mod m28_nx;
#[path = "city_codes/29_QH.rs"]
mod m29_qh;
#[path = "city_codes/30_AH.rs"]
mod m30_ah;
#[path = "city_codes/31_TW.rs"]
mod m31_tw;
#[path = "city_codes/32_XZ.rs"]
mod m32_xz;
#[path = "city_codes/33_XJ.rs"]
mod m33_xj;
#[path = "city_codes/34_XK.rs"]
mod m34_xk;
#[path = "city_codes/35_AL.rs"]
mod m35_al;
#[path = "city_codes/36_CL.rs"]
mod m36_cl;
#[path = "city_codes/37_TS.rs"]
mod m37_ts;
#[path = "city_codes/38_HX.rs"]
mod m38_hx;
#[path = "city_codes/39_KL.rs"]
mod m39_kl;
#[path = "city_codes/40_HT.rs"]
mod m40_ht;
#[path = "city_codes/41_RH.rs"]
mod m41_rh;
#[path = "city_codes/42_XA.rs"]
mod m42_xa;
#[path = "city_codes/43_HJ.rs"]
mod m43_hj;

pub fn province_code_by_name(name: &str) -> Option<&'static str> {
    PROVINCES.iter().find(|p| p.name == name).map(|p| p.code)
}

pub fn city_code_by_name(province_name: &str, city_name: &str) -> Option<&'static str> {
    let p = PROVINCES.iter().find(|p| p.name == province_name)?;
    p.cities
        .iter()
        .find(|c| c.name == city_name)
        .map(|c| c.code)
}

pub fn province_name_by_code(code: &str) -> Option<&'static str> {
    PROVINCES
        .iter()
        .find(|p| p.code.eq_ignore_ascii_case(code))
        .map(|p| p.name)
}

pub fn provinces() -> &'static [ProvinceCode] {
    &PROVINCES
}
