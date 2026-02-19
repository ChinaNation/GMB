//! 43个初始省级行政区代码常量
//! 三元组顺序：(中文名, 简称码, 全称码)
//! 中文名用于前端显示，简称码用于CPMS系统省代码，全称码用于生成省储行质押地址

#![allow(dead_code)]

pub const PROVINCES: [(&str, &str, &str); 43] = [
    ("中枢省", "ZS", "01_ZHONGSHU"),
    ("岭南省", "LN", "02_LINGNAN"),
    ("广东省", "GD", "03_GUANGDONG"),
    ("广西省", "GX", "04_GUANGXI"),
    ("福建省", "FJ", "05_FUJIAN"),
    ("海南省", "HN", "06_HAINAN"),
    ("云南省", "YN", "07_YUNNAN"),
    ("贵州省", "GZ", "08_GUIZHOU"),
    ("湖南省", "HU", "09_HUNAN"),
    ("江西省", "JX", "10_JIANGXI"),
    ("浙江省", "ZJ", "11_ZHEJIANG"),
    ("江苏省", "JS", "12_JIANGSU"),
    ("山东省", "SD", "13_SHANDONG"),
    ("山西省", "SX", "14_SHANXI"),
    ("河南省", "HE", "15_HENAN"),
    ("河北省", "HB", "16_HEBEI"),
    ("湖北省", "HI", "17_HUBEI"),
    ("陕西省", "SI", "18_SHAANXI"),
    ("重庆省", "CQ", "19_CHONGQING"),
    ("四川省", "SC", "20_SICHUAN"),
    ("甘肃省", "GS", "21_GANSU"),
    ("北平省", "BP", "22_BEIPING"),
    ("海滨省", "HA", "23_HAIBIN"),
    ("松江省", "SJ", "24_SONGJIANG"),
    ("龙江省", "LJ", "25_LONGJIANG"),
    ("吉林省", "JL", "26_JILIN"),
    ("辽宁省", "LI", "27_LIAONING"),
    ("宁夏省", "NX", "28_NINGXIA"),
    ("青海省", "QH", "29_QINGHAI"),
    ("安徽省", "AH", "30_ANHUI"),
    ("台湾省", "TW", "31_TAIWAN"),
    ("西藏省", "XZ", "32_XIZANG"),
    ("新疆省", "XJ", "33_XINJIANG"),
    ("西康省", "XK", "34_XIKANG"),
    ("阿里省", "AL", "35_ALI"),
    ("葱岭省", "CL", "36_CONGLING"),
    ("天山省", "TS", "37_TIANSHAN"),
    ("河西省", "HX", "38_HEXI"),
    ("昆仑省", "KL", "39_KUNLUN"),
    ("河套省", "HT", "40_HETAO"),
    ("热河省", "RH", "41_REHE"),
    ("兴安省", "XA", "42_XINGAN"),
    ("合江省", "HJ", "43_HEJIANG"),
];

pub fn zh_name_by_index(index: usize) -> Option<&'static str> {
    PROVINCES.get(index).map(|(zh, _, _)| *zh)
}

pub fn short_code_by_index(index: usize) -> Option<&'static str> {
    PROVINCES.get(index).map(|(_, short, _)| *short)
}

pub fn full_code_by_index(index: usize) -> Option<&'static str> {
    PROVINCES.get(index).map(|(_, _, full)| *full)
}

pub fn full_code_by_short_code(short_code: &str) -> Option<&'static str> {
    PROVINCES
        .iter()
        .find_map(|(_, short, full)| if *short == short_code { Some(*full) } else { None })
}

pub fn zh_name_by_short_code(short_code: &str) -> Option<&'static str> {
    PROVINCES
        .iter()
        .find_map(|(zh, short, _)| if *short == short_code { Some(*zh) } else { None })
}
