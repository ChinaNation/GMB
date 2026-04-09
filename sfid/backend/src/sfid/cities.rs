// 中文注释:任务卡 1 基础设施,部分 API 在任务卡 2 才会有调用点。
#![allow(dead_code)]

//! 城市清单查询(按省聚合)
//!
//! 数据源:`sfid/city_codes/*.rs` 静态表,经 `sfid/province.rs::PROVINCES` 的
//! `cities` 字段引用。本文件提供高层 API,避免业务模块直接访问 PROVINCES。

use crate::sfid::province::{CityCode, PROVINCES};

/// 返回某省的所有市(包括 code="000" 的"本省统一市级段"占位)。
/// 找不到省时返回 None。
pub fn cities_of(province_name: &str) -> Option<&'static [CityCode]> {
    PROVINCES
        .iter()
        .find(|p| p.name == province_name)
        .map(|p| p.cities)
}

/// 返回某省的所有市,**过滤掉** code=="000" 的保留占位(供前端下拉使用)。
pub fn real_cities_of(province_name: &str) -> Vec<&'static CityCode> {
    cities_of(province_name)
        .map(|cities| cities.iter().filter(|c| c.code != "000").collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guangdong_has_cities() {
        let cities = cities_of("广东省").expect("province exists");
        assert!(!cities.is_empty());
        let real = real_cities_of("广东省");
        assert!(real.len() < cities.len() || cities.iter().all(|c| c.code != "000"));
    }

    #[test]
    fn unknown_province_returns_none() {
        assert!(cities_of("不存在省").is_none());
    }
}
