//! 行政区划运行时模型。

/// 镇行政区划。
///
/// 中文注释:SFID 公权机构名录需要覆盖到镇目录。村/路数据仍只保存在
/// `china.sqlite` 中,本系统不把村/路作为公权机构目录范围。
#[derive(Debug)]
pub struct TownCode {
    pub name: &'static str,
    pub code: &'static str,
}

#[derive(Debug)]
pub struct CityCode {
    pub name: &'static str,
    pub code: &'static str,
    pub towns: &'static [TownCode],
}

#[derive(Debug)]
pub struct ProvinceCode {
    pub name: &'static str,
    pub code: &'static str,
    pub cities: &'static [CityCode],
}
