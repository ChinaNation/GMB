//! 中国行政区划真源。
//!
//! 中文注释:行政区划由 `china/data/china.sqlite` 承载,不再在 Rust 源码里维护
//! 第二套省市镇村静态表。SFID 编码只通过本模块查询省、市代码。

pub(crate) mod admin;
pub(crate) mod model;
mod store;

pub(crate) use store::{
    china_sqlite_hash, city_code_by_name, province_code_by_name, province_name_by_code, provinces,
};
