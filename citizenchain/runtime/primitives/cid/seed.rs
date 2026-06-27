//! CID 确定性种子协议。
//!
//! 中文注释:
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

/// 市公安局(CPOL)历史确定性 account seed。
///
/// 中文注释:不得改成 GOV-CITY 模板种子,否则会平移既有公安局 CID。
pub fn public_security_account_seed(province_code: &str, city_code: &str) -> String {
    format!("PS-{province_code}-{city_code}")
}
