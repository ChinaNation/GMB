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
    ProvinceCode {name: "中枢省",code: "ZS",pubkey: "0xd641dbfe17fa3fb2427b974212a0fe821b12576e0eade088309d4f05f2cc9930",cities: &m01_zs::CITIES_ZS,},
    ProvinceCode {name: "岭南省",code: "LN",pubkey: "0xe28a39b8f9f9bdc7d0d5c2f6bf290f892a25aeeb34c57002cdb978d13c4efa26",cities: &m02_ln::CITIES_LN,},
    ProvinceCode {name: "广东省",code: "GD",pubkey: "0x5cdd16e9a9b63f2660ad7829c6d2004ddb713ea46ee5086e53edbda3dd175b42",cities: &m03_gd::CITIES_GD,},
    ProvinceCode {name: "广西省",code: "GX",pubkey: "0x1cb60c7ae7236b61ab6d678ee240978ba7653174f725cebe50db02642f2e9129",cities: &m04_gx::CITIES_GX,},
    ProvinceCode {name: "福建省",code: "FJ",pubkey: "0x02d25858d77d87bf0637bdf37e0ae45819bed00b06ed41dc3c2e4888512a7003",cities: &m05_fj::CITIES_FJ,},
    ProvinceCode {name: "海南省",code: "HN",pubkey: "0x94c8853d6090b02581659cae1ce33ce0b3c84078b606e53e052d8439e73fec1e",cities: &m06_hn::CITIES_HN,},
    ProvinceCode {name: "云南省",code: "YN",pubkey: "0xe658db8112f1ea0a7d2e63b7622e2514c5c65a89db441e3df507272ab2d6231e",cities: &m07_yn::CITIES_YN,},
    ProvinceCode {name: "贵州省",code: "GZ",pubkey: "0xfe7176d115b207356914f92e2da1391db92bc5a463be7f89f2b37d65e367895e",cities: &m08_gz::CITIES_GZ,},
    ProvinceCode {name: "湖南省",code: "HU",pubkey: "0x8aaa255eb6fc0ae304b89a55e93809092f897641917f78d0d1e360c198599105",cities: &m09_hu::CITIES_HU,},
    ProvinceCode {name: "江西省",code: "JX",pubkey: "0x6c11e617a58e56ba71a2d92b7e989de1a649e4103776dbd8465a3f729b66ca31",cities: &m10_jx::CITIES_JX,},
    ProvinceCode {name: "浙江省",code: "ZJ",pubkey: "0xf47373164ca9f7167e1da17955761b17e38823348c8aeecb5f259a25d3ad6d2f",cities: &m11_zj::CITIES_ZJ,},
    ProvinceCode {name: "江苏省",code: "JS",pubkey: "0x78bc0525055f37f2c7245e94dc95baa3dafc1dc051631f0333bd9dbf9818fb0e",cities: &m12_js::CITIES_JS,},
    ProvinceCode {name: "山东省",code: "SD",pubkey: "0x9edf2e0e022b9ff892175528d4a87ef466c0896cc2586b705523932cfd5a1777",cities: &m13_sd::CITIES_SD,},
    ProvinceCode {name: "山西省",code: "SX",pubkey: "0xac2d0d1ffed7aa373adefa5ddfbc4f377edc91b825b2b13464932bbbb264b40f",cities: &m14_sx::CITIES_SX,},
    ProvinceCode {name: "河南省",code: "HE",pubkey: "0xdc95de49abd2d371b368256939d15370d0f9915d738d52434431b0c763004b50",cities: &m15_he::CITIES_HE,},
    ProvinceCode {name: "河北省",code: "HB",pubkey: "0x604925f9cb49555816b880542cb8045ad4e50165351f5b14d1fd111171bb8617",cities: &m16_hb::CITIES_HB,},
    ProvinceCode {name: "湖北省",code: "HI",pubkey: "0x1ec98129b379e9f60bad6f0d0bc73e327c20424ac5392192518b71627f752e24",cities: &m17_hi::CITIES_HI,},
    ProvinceCode {name: "陕西省",code: "SI",pubkey: "0xf6c3e174783aeeea0afc736a42e52ebd2029b4a56de04e9a5301d98094332f45",cities: &m18_si::CITIES_SI,},
    ProvinceCode {name: "重庆省",code: "CQ",pubkey: "0x1c6f70806461448e7e2621cf29b0924aee483300f4554bea393c1b9c54e78442",cities: &m19_cq::CITIES_CQ,},
    ProvinceCode {name: "四川省",code: "SC",pubkey: "0x7ed7d3bd8ae09960884ff1a98db4493fc5f6818e900f45f66b6b7e76e11e8274",cities: &m20_sc::CITIES_SC,},
    ProvinceCode {name: "甘肃省",code: "GS",pubkey: "0x52be4ed7bf042b94a4f54ea74369133f5e6ced79e03e84020093c8ec73114c78",cities: &m21_gs::CITIES_GS,},
    ProvinceCode {name: "北平省",code: "BP",pubkey: "0x940e9a759ce49bee1a49eb8a32dbd03a8813e52f4632534d4cc5c4b7a4cea746",cities: &m22_bp::CITIES_BP,},
    ProvinceCode {name: "海滨省",code: "HA",pubkey: "0xfccb22b76f7fff0f05dbbab53cba7bbe1bbe0edfece43b139321bec88cb7aa1f",cities: &m23_ha::CITIES_HA,},
    ProvinceCode {name: "松江省",code: "SJ",pubkey: "0x1a1c763345d8bb2e08b30e18788c1bc8e977fd54ba61aa936a8c5db13cf09c03",cities: &m24_sj::CITIES_SJ,},
    ProvinceCode {name: "龙江省",code: "LJ",pubkey: "0x4a74ce94de45a80b73850750fd2b08c1782f8e6f4a2301fc2a72af7938a92436",cities: &m25_lj::CITIES_LJ,},
    ProvinceCode {name: "吉林省",code: "JL",pubkey: "0x9a2c2b408a0773c19cfc7207780571ab321dd285f11b7a1bb09e013fed73e737",cities: &m26_jl::CITIES_JL,},
    ProvinceCode {name: "辽宁省",code: "LI",pubkey: "0xdc3295a5e874ea91d6dcde444b698c5ecf183b16f11954c9fc71e91bfe87b377",cities: &m27_li::CITIES_LI,},
    ProvinceCode {name: "宁夏省",code: "NX",pubkey: "0xf05e4afa76f9d883151a6ef656013efef42a6821feef45b42b43f67eca6d6328",cities: &m28_nx::CITIES_NX,},
    ProvinceCode {name: "青海省",code: "QH",pubkey: "0x1af800fa82965b12fa04f7a87245cc9be5d3fb8cf88a1026e3dc45eacfec405d",cities: &m29_qh::CITIES_QH,},
    ProvinceCode {name: "安徽省",code: "AH",pubkey: "0x5498141113bf85eca686955162ee2912ac6c3b050ba9ffa102ac923ab0bb350b",cities: &m30_ah::CITIES_AH,},
    ProvinceCode {name: "台湾省",code: "TW",pubkey: "0xd81866ce95bc72bc7f66e67262e829dcde04b069df3f816faa2865a9382fbf25",cities: &m31_tw::CITIES_TW,},
    ProvinceCode {name: "西藏省",code: "XZ",pubkey: "0x506bb4c300584f13b4307e8cdc251e7756f212c2ee7c302bdd778688c47b201b",cities: &m32_xz::CITIES_XZ,},
    ProvinceCode {name: "新疆省",code: "XJ",pubkey: "0x9281ec501bb174b6a608e23fe74770643bdb14e9f26f1aee45f740e3e1d80657",cities: &m33_xj::CITIES_XJ,},
    ProvinceCode {name: "西康省",code: "XK",pubkey: "0xbc6215cb2b86840fb27864f72f08ba09a552e2dfcb38fe8ec010664c37e6b748",cities: &m34_xk::CITIES_XK,},
    ProvinceCode {name: "阿里省",code: "AL",pubkey: "0xb217302c1c6d099df4a440126df288b74c17ec6b59cd02952b772f47e8154c6d",cities: &m35_al::CITIES_AL,},
    ProvinceCode {name: "葱岭省",code: "CL",pubkey: "0x98db54a14cdb9015525467d129668eb58573103013ee9ec8ba380384e2b54b41",cities: &m36_cl::CITIES_CL,},
    ProvinceCode {name: "天山省",code: "TS",pubkey: "0x463d76ac7e1d3c4cb3355128395189d17bbafb6552a9fdacf075b1fe1f13c32c",cities: &m37_ts::CITIES_TS,},
    ProvinceCode {name: "河西省",code: "HX",pubkey: "0x2608cab4ded7bee2ac75d55d46d76904f1907b90a4ef768e03cc1663a04de062",cities: &m38_hx::CITIES_HX,},
    ProvinceCode {name: "昆仑省",code: "KL",pubkey: "0xc645ea0c6e3adb4809268d13cd9820fd759056b2382a5531406873638ce7044a",cities: &m39_kl::CITIES_KL,},
    ProvinceCode {name: "河套省",code: "HT",pubkey: "0x10972b4b6b227da8cb90cac066502d7210a50955256c83ec083f6b87e3abd71e",cities: &m40_ht::CITIES_HT,},
    ProvinceCode {name: "热河省",code: "RH",pubkey: "0x1e312af5890084151339ec37b9e7145211366c7ac3163a5ca3d7e8ccb809d674",cities: &m41_rh::CITIES_RH,},
    ProvinceCode {name: "兴安省",code: "XA",pubkey: "0x10e74326066fceebb3eb103182f36825dee56b077722900c4f718a1fe754823b",cities: &m42_xa::CITIES_XA,},
    ProvinceCode {name: "合江省",code: "HJ",pubkey: "0x8c72490d8774dc1c4305825d82788ad1bd1dc53b06360c2301974e6bc12df638",cities: &m43_hj::CITIES_HJ,},
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
    PROVINCES.iter().find(|p| p.code.eq_ignore_ascii_case(code)).map(|p| p.name)
}

pub fn provinces() -> &'static [ProvinceCode] {
    &PROVINCES
}

pub fn sheng_admin_province(pubkey: &str) -> Option<&'static str> {
    PROVINCES
        .iter()
        .find(|p| p.pubkey.eq_ignore_ascii_case(pubkey))
        .map(|p| p.name)
}

pub fn sheng_admin_display_name(pubkey: &str) -> Option<String> {
    let province_name = sheng_admin_province(pubkey)?;
    Some(format!("{province_name}省级管理员"))
}
