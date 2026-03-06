pub struct CityCode {
    pub name: &'static str,
    pub code: &'static str,
}

pub struct ProvinceCode {
    pub name: &'static str,
    pub code: &'static str,
    pub pubkey: &'static str,
    pub cities: &'static [CityCode],
}

#[rustfmt::skip]
pub const PROVINCES: [ProvinceCode; 43] = [
    ProvinceCode {name: "中枢省",code: "ZS",pubkey: "0x4215f8b2dea54b3dfeaec3c267314ceb1c3e831ec5e808794e294c623ac5c12e",cities: &m01_zs::CITIES_ZS,},
    ProvinceCode {name: "岭南省",code: "LN",pubkey: "0xfaf75f9cb6945e1f61cb42a26d8b6c35614794830ecfc06477e8ace8e1c844b8",cities: &m02_ln::CITIES_LN,},
    ProvinceCode {name: "广东省",code: "GD",pubkey: "0x86eaef716945c2080b9348b8bc9aaede939be2bd875ddb2d3043edbcf2c42ddc",cities: &m03_gd::CITIES_GD,},
    ProvinceCode {name: "广西省",code: "GX",pubkey: "0x209c88e0f516c4bef0bd7295a28065cd05ac8c2d4ed40b157a67dc55dc8d3e53",cities: &m04_gx::CITIES_GX,},
    ProvinceCode {name: "福建省",code: "FJ",pubkey: "0x7a5d47815b1ae50c39ad89b68f8f7827dffedabbe0a349484a873cc0a3e94ba2",cities: &m05_fj::CITIES_FJ,},
    ProvinceCode {name: "海南省",code: "HN",pubkey: "0xe242a55b40fbf6c0a29e5aa316965a93ec67d09aafc266f7e8895bdb3e06b5d5",cities: &m06_hn::CITIES_HN,},
    ProvinceCode {name: "云南省",code: "YN",pubkey: "0x49542724fe089ab265daeceee4a6c9f5028ac623b9602230664578d2c1ea8442",cities: &m07_yn::CITIES_YN,},
    ProvinceCode {name: "贵州省",code: "GZ",pubkey: "0x523248ae37ea8689ca9e5a6d505e1610c67aa7e4646505b8b8750b82aeba9b6c",cities: &m08_gz::CITIES_GZ,},
    ProvinceCode {name: "湖南省",code: "HU",pubkey: "0xbae6d80a94732903af02202d0ada61d7e74f25f0a8cdf32b6d73260dc418e39f",cities: &m09_hu::CITIES_HU,},
    ProvinceCode {name: "江西省",code: "JX",pubkey: "0xfead3049fa97c8fd8b07f319cfe42bef096831cec0235dda5b4ad8af93827180",cities: &m10_jx::CITIES_JX,},
    ProvinceCode {name: "浙江省",code: "ZJ",pubkey: "0x0f327a7b55043d17b16596f0c04729667696a666c38b419a237d8961fb343a00",cities: &m11_zj::CITIES_ZJ,},
    ProvinceCode {name: "江苏省",code: "JS",pubkey: "0x65c81d68fc9970b3ae2ec820ebb494f9c0e571a46154f2beec36de05f15ec71c",cities: &m12_js::CITIES_JS,},
    ProvinceCode {name: "山东省",code: "SD",pubkey: "0x5213d43ecb0c0f9ef7156173cb4bb3310ccece5ae59259837b2238fc7ad7640e",cities: &m13_sd::CITIES_SD,},
    ProvinceCode {name: "山西省",code: "SX",pubkey: "0x76d4f36462eca50cdc3aa7f59650dd957200ba8fa88c3a9e27547cb79ec92266",cities: &m14_sx::CITIES_SX,},
    ProvinceCode {name: "河南省",code: "HE",pubkey: "0x682792dc6945be8fe5bdcdbe72f19fa40c42c39411d56a206516d4a223884d91",cities: &m15_he::CITIES_HE,},
    ProvinceCode {name: "河北省",code: "HB",pubkey: "0xbe824a50fb2e456cc8ad0dac169c6f42818525b6de16647284c433d5a36c05a5",cities: &m16_hb::CITIES_HB,},
    ProvinceCode {name: "湖北省",code: "HI",pubkey: "0xa8c7569f9fd0eea135a453b9b2f1e32f3222c4f5a4981ba59111f5220e67d7f1",cities: &m17_hi::CITIES_HI,},
    ProvinceCode {name: "陕西省",code: "SI",pubkey: "0xa3dbf6e743d0712ff6be8d852ec2a892891dc35d5d2ef260c112afe291c49b64",cities: &m18_si::CITIES_SI,},
    ProvinceCode {name: "重庆省",code: "CQ",pubkey: "0x95f0a21e4d83326867e8ce82287e91f9606075cc77c21f1b1799a6224dc058c6",cities: &m19_cq::CITIES_CQ,},
    ProvinceCode {name: "四川省",code: "SC",pubkey: "0x97c73ed34795384c351a59d81d876011805cf41a0b40da822d7d051137f9059d",cities: &m20_sc::CITIES_SC,},
    ProvinceCode {name: "甘肃省",code: "GS",pubkey: "0x37b05070ec70ad7444f208dc1b0f15432fe00bedb05f4fdb061ac83ee68800bc",cities: &m21_gs::CITIES_GS,},
    ProvinceCode {name: "北平省",code: "BP",pubkey: "0x4b724514f0c03b8289de69592687de7bd54096e6801ee1b55604b29789055e8e",cities: &m22_bp::CITIES_BP,},
    ProvinceCode {name: "海滨省",code: "HA",pubkey: "0xaf2cd60d5e63d8d1dff54c391c1567c89998ce4114fddb80d84d84a8f9e4db04",cities: &m23_ha::CITIES_HA,},
    ProvinceCode {name: "松江省",code: "SJ",pubkey: "0xf897f0119dfb14035a841713a7cf889f903d262d26c187fd329bb4ba6a7b5be3",cities: &m24_sj::CITIES_SJ,},
    ProvinceCode {name: "龙江省",code: "LJ",pubkey: "0x41f918fd7f8ec10f6fa1e4d67094583742585aae5ec5a2ac97fd046b4d4dc48c",cities: &m25_lj::CITIES_LJ,},
    ProvinceCode {name: "吉林省",code: "JL",pubkey: "0x0635c25df1cf9dd1fbe6d8e4bea1de71a0e8c1aab0d2dbf5fe2dbda39b64d798",cities: &m26_jl::CITIES_JL,},
    ProvinceCode {name: "辽宁省",code: "LI",pubkey: "0xb1684c88713edd73414eb23fb99608ba71495e2bed457da6d5b9afc1921e2fa4",cities: &m27_li::CITIES_LI,},
    ProvinceCode {name: "宁夏省",code: "NX",pubkey: "0x2dbfa78e5e41e7593ae96be8623cbfe019d181e1c93d598239713ad5bd3f5472",cities: &m28_nx::CITIES_NX,},
    ProvinceCode {name: "青海省",code: "QH",pubkey: "0x0d71aa43a60982e9baf30e2898644701680737814ceeb724845c1ed3874cd685",cities: &m29_qh::CITIES_QH,},
    ProvinceCode {name: "安徽省",code: "AH",pubkey: "0x8029f0568a5a803d635179e1b84b1e82140670b425ff058497577f4e898b9829",cities: &m30_ah::CITIES_AH,},
    ProvinceCode {name: "台湾省",code: "TW",pubkey: "0x5d489483bb48d1d447cbdcbc8fbb4c71e37eda4cf31340fe6c4f5aa640f72fc5",cities: &m31_tw::CITIES_TW,},
    ProvinceCode {name: "西藏省",code: "XZ",pubkey: "0x8f41d40cd896fa0af97c3052b6486353ab6f75d479f77ddf224af89a1c2977b6",cities: &m32_xz::CITIES_XZ,},
    ProvinceCode {name: "新疆省",code: "XJ",pubkey: "0x587bc76d45042bcd9ff52ceee72612e3b71a734059038c46e9234985f308c373",cities: &m33_xj::CITIES_XJ,},
    ProvinceCode {name: "西康省",code: "XK",pubkey: "0xf66395985ccc73d95fb290504973b376f02d7d98da93b20d95c343da8d152de7",cities: &m34_xk::CITIES_XK,},
    ProvinceCode {name: "阿里省",code: "AL",pubkey: "0xbc789a1c394a5e485d38432bcdb59fad3411cfd4699a7983f5853997867401c4",cities: &m35_al::CITIES_AL,},
    ProvinceCode {name: "葱岭省",code: "CL",pubkey: "0xed3f67c63bc0f2140205473c17890e73688babdf7684f9287ce7cddfcd22ca09",cities: &m36_cl::CITIES_CL,},
    ProvinceCode {name: "天山省",code: "TS",pubkey: "0xdd464573534b2eb6e0d7a6fcac549a51c63e5847b23c299135e36b4ea3e24adb",cities: &m37_ts::CITIES_TS,},
    ProvinceCode {name: "河西省",code: "HX",pubkey: "0x8f8b9105949a108774ff845ba4df0f7a921f5beae6e814717fabf9b99a1ce8bd",cities: &m38_hx::CITIES_HX,},
    ProvinceCode {name: "昆仑省",code: "KL",pubkey: "0xe7c1fa0801ca5b74c6939de99a519de7023dc602ab55231b640e3b1b7f5895f0",cities: &m39_kl::CITIES_KL,},
    ProvinceCode {name: "河套省",code: "HT",pubkey: "0x2ffbe3ed5ee5134149adec09698affcea20f5cca51dbf95cdb0616430eee84aa",cities: &m40_ht::CITIES_HT,},
    ProvinceCode {name: "热河省",code: "RH",pubkey: "0x6974ef45d1495159dbbacfc2a114e284b4538775dfd2c3e7296ed681285406f8",cities: &m41_rh::CITIES_RH,},
    ProvinceCode {name: "兴安省",code: "XA",pubkey: "0x10ac83be1c5cdc8c5e762323d037eb792bd783d362d77b6e52e4547e42174a3e",cities: &m42_xa::CITIES_XA,},
    ProvinceCode {name: "合江省",code: "HJ",pubkey: "0xeb85e6981f71269f7dc22f4715119f943b675bb98a0ae5427ede0004f2bad626",cities: &m43_hj::CITIES_HJ,},
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

pub fn provinces() -> &'static [ProvinceCode] {
    &PROVINCES
}

pub fn super_admin_province(pubkey: &str) -> Option<&'static str> {
    PROVINCES
        .iter()
        .find(|p| p.pubkey.eq_ignore_ascii_case(pubkey))
        .map(|p| p.name)
}

pub fn super_admin_display_name(pubkey: &str) -> Option<String> {
    let province_name = super_admin_province(pubkey)?;
    Some(format!("{province_name}超级管理员"))
}
