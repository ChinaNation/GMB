//! CID 确定性种子协议。
//!
//!
//! runtime 只保护种子字节格式,不做随机 UUID、数据库查重和撞号重试。

use alloc::{format, string::String};

/// 公权机构(政府模板)CID 的确定性 account seed。
pub fn official_institution_account_seed(
    scope: &str,
    province_code: &str,
    city_code: &str,
    town_code: &str,
    institution_code: &str,
) -> String {
    format!("GOV-{scope}-{province_code}-{city_code}-{town_code}-{institution_code}")
}
