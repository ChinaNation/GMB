//! 中文注释:SFID 行政区 / 选项元数据 DTO(管理员控制台元信息接口使用)。

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct SfidOptionItem {
    pub(crate) label: &'static str,
    pub(crate) value: &'static str,
}

#[derive(Serialize)]
pub(crate) struct SfidProvinceItem {
    pub(crate) name: String,
    pub(crate) code: String,
}

#[derive(Serialize)]
pub(crate) struct SfidCityItem {
    pub(crate) name: String,
    pub(crate) code: String,
}

#[derive(Serialize)]
pub(crate) struct AdminSfidMetaOutput {
    pub(crate) a3_options: Vec<SfidOptionItem>,
    pub(crate) institution_options: Vec<SfidOptionItem>,
    pub(crate) provinces: Vec<SfidProvinceItem>,
    pub(crate) scoped_province: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminSfidCitiesQuery {
    pub(crate) province: String,
}
