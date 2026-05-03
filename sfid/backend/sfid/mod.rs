// 中文注释:本模块里的 pub use 是给任务卡 2~5 的业务模块预留的对外 API,
// 任务卡 1 只建基础设施,暂时没调用点,暂时用 allow 抑制"未使用重导出"告警。
#![allow(unused_imports)]

//! SFID 工具模块 — sfid 系统所有 SFID 相关常量、枚举、生成、校验的**唯一入口**
//!
//! 中文注释(铁律):
//! 任何新增的 SFID 工具逻辑(A3 类型、机构码、省市清单、分类、校验、生成)
//! 都必须放在本模块下,**不能**散在 `sheng-admins/` / `operate/` /
//! `chain/` / `app_core/` 等业务模块里。
//!
//! 参见 `feedback_sfid_module_is_single_entry.md`。
//!
//! ## 子模块
//!
//! - [`a3`]               — A3 主体属性枚举(GMR/ZRR/ZNR/GFR/SFR/FFR)
//! - [`institution_code`] — 机构类型枚举(ZG/ZF/LF/SF/JC/JY/CB/CH/TG)
//! - [`province`]         — 43 省常量表 + 省/市代码查询
//! - [`cities`]           — 按省查询城市清单(高层 API)
//! - [`category`]         — 机构分类(公安局/公权/私权),任务卡 2 使用
//! - [`validator`]        — SFID 号格式校验
//! - [`generator`]        — SFID 号生成
//! - [`model`]            — SFID admin 元信息 DTO
//! - [`admin`]            — SFID admin 相关(legacy)

pub mod a3;
pub(crate) mod admin;
pub mod category;
pub mod cities;
pub mod generator;
pub mod institution_code;
pub(crate) mod model;
pub mod province;
pub mod validator;

// 中文注释:对外聚合导出,方便业务模块只写 `use crate::sfid::*` 就能拿到全部工具。
pub use a3::{all_a3, A3};
pub use category::{classify, InstitutionCategory, PUBLIC_SECURITY_INSTITUTION_NAME};
pub use cities::{cities_of, real_cities_of};
pub use generator::{generate_sfid_code, GenerateSfidInput};
pub use institution_code::InstitutionCode;
#[allow(unused_imports)]
pub(crate) use model::*;
pub use province::{
    city_code_by_name, province_code_by_name, province_name_by_code, provinces, CityCode,
    ProvinceCode,
};
pub use validator::{
    validate_sfid_id_format, SFID_ID_MAX_BYTES, SFID_ID_SEGMENT_A3_LEN, SFID_ID_SEGMENT_COUNT,
    SFID_ID_SEGMENT_D8_LEN, SFID_ID_SEGMENT_N9_LEN, SFID_ID_SEGMENT_R5_LEN,
    SFID_ID_SEGMENT_T2P1C1_LEN,
};
