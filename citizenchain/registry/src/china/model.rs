//! 行政区划运行时模型。

/// 镇行政区划。
///
/// 中文注释:CID 公权机构名录需要覆盖到镇目录。镇下面是地址段,
/// 只保存在 `china.sqlite` 的 `address_units` 中,不作为公权机构目录范围。
#[derive(Debug)]
pub struct TownDivision {
    pub town_name: &'static str,
    pub town_code: &'static str,
}

#[derive(Debug)]
pub struct CityDivision {
    pub city_name: &'static str,
    pub city_code: &'static str,
    pub towns: &'static [TownDivision],
}

#[derive(Debug)]
pub struct ProvinceDivision {
    pub province_name: &'static str,
    pub province_code: &'static str,
    pub cities: &'static [CityDivision],
}
