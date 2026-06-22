//! 中文注释:CID 行政区 / 选项元数据 DTO(管理员控制台元信息接口使用)。

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct CidOptionItem {
    pub(crate) label: &'static str,
    pub(crate) value: &'static str,
}

#[derive(Serialize)]
pub(crate) struct CidProvinceItem {
    pub(crate) name: String,
    pub(crate) code: String,
}

#[derive(Serialize)]
pub(crate) struct CidCityItem {
    pub(crate) name: String,
    pub(crate) code: String,
}

#[derive(Serialize)]
pub(crate) struct AdminCidMetaOutput {
    pub(crate) institution_options: Vec<CidOptionItem>,
    pub(crate) provinces: Vec<CidProvinceItem>,
    pub(crate) scoped_province_name: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminCidCitiesQuery {
    pub(crate) province_name: String,
}
