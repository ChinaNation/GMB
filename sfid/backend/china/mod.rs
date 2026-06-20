//! 中国行政区划真源。
//!
//! 中文注释:行政区划由随包只读 `china/china.sqlite` 承载,不再在 Rust 源码里维护
//! 第二套省市镇静态表;镇下地址段只用于下游地址选择,不是行政区编码范围。
//! SFID 编码只通过本模块查询省、市、镇代码。

pub(crate) mod admin;
pub(crate) mod model;
mod store;

pub(crate) use store::{
    area_name_by_codes, china_sqlite_hash, city_code_by_name, province_code_by_name,
    province_name_by_code, provinces, town_exists,
};
