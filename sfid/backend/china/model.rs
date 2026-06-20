//! 行政区划运行时模型。

/// 镇行政区划。
///
/// 中文注释:SFID 公权机构名录需要覆盖到镇目录。镇下面是地址段,
/// 只保存在 `china.sqlite` 的 `address_units` 中,不作为公权机构目录范围。
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
