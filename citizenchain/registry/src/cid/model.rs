//! 中文注释:CID 行政区 / 选项元数据 DTO(管理员控制台元信息接口使用)。

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct CidInstitutionCodeItem {
    pub(crate) institution_code: &'static str,
    pub(crate) cid_short_name: &'static str,
}

#[derive(Serialize)]
pub(crate) struct CidProvinceItem {
    pub(crate) province_name: String,
    pub(crate) province_code: String,
}

#[derive(Serialize)]
pub(crate) struct CidCityItem {
    pub(crate) city_name: String,
    pub(crate) city_code: String,
}

#[derive(Serialize)]
pub(crate) struct AdminCidMetaOutput {
    pub(crate) institution_options: Vec<CidInstitutionCodeItem>,
    pub(crate) provinces: Vec<CidProvinceItem>,
    pub(crate) scoped_province_name: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminCidCitiesQuery {
    pub(crate) province_name: String,
}
