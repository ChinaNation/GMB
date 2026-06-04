//! 行政区划运行时模型。

/// 中文注释:当前业务只需要省/市层级;镇村完整数据保存在 SQLite 表中。
#[derive(Debug)]
pub struct CityCode {
    pub name: &'static str,
    pub code: &'static str,
}

#[derive(Debug)]
pub struct ProvinceCode {
    pub name: &'static str,
    pub code: &'static str,
    pub cities: &'static [CityCode],
}
