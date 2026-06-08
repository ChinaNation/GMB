// 中文注释:本模块集中导出 SFID 工具 API;部分导出由业务模块按需使用,
// 保留 allow 抑制"未使用重导出"告警。
#![allow(unused_imports)]

//! SFID 编码协议模块 — SFID 号码常量、枚举、生成、校验的唯一入口。
//!
//! 中文注释(铁律):
//! 任何新增的 SFID 编码逻辑(主体属性、机构码、分类、校验、生成)都必须放在
//! 本模块下。行政区划真源属于 `crate::china`,本模块只在生成号码时读取省市代码。
//!
//! ## 子模块
//!
//! - [`category`]         — 主体属性与机构分类
//! - [`institution_code`] — 机构类型枚举(ZG/ZF/LF/SF/JC/JY/CB/CH/TG)
//! - [`validator`]        — SFID 号格式校验
//! - [`generator`]        — SFID 号生成
//! - [`model`]            — 身份 ID 编码元信息 DTO
//! - [`admin`]            — 身份 ID 编码元信息接口

pub(crate) mod admin;
pub mod category;
pub mod generator;
pub mod institution_code;
pub(crate) mod model;
pub mod validator;

// 中文注释:对外聚合导出,方便业务模块只写 `use crate::number::*`。
pub use category::{
    all_subject_properties, classify, InstitutionCategory, SubjectProperty,
    PUBLIC_SECURITY_INSTITUTION_SUFFIX,
};
pub use generator::{generate_sfid_number, GenerateSfidInput};
pub use institution_code::InstitutionCode;
#[allow(unused_imports)]
pub(crate) use model::*;
pub use validator::{
    parse_sfid_number_parts, validate_sfid_number_format, SfidNumberParts,
    SFID_NUMBER_SEGMENT_COUNT, SFID_NUMBER_SEGMENT_D4_LEN, SFID_NUMBER_SEGMENT_K3P1C1_LEN,
    SFID_NUMBER_SEGMENT_N9_LEN, SFID_NUMBER_SEGMENT_R5_LEN,
};
